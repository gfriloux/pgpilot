#![allow(dead_code)]
//! Salons de chat et persistance `rooms.yaml`.
//!
//! Ce module gère le CRUD des salons persistés dans
//! `~/.config/pgpilot/rooms.yaml` ainsi que l'encodage/décodage/vérification
//! des codes d'invitation ([`JoinCode`]).

use std::io::Write as _;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::chat::{ChatError, ChatResult, JOIN_CODE_PREFIX};

// ---------------------------------------------------------------------------
// Types de données
// ---------------------------------------------------------------------------

/// Un participant à un salon, identifié par son fingerprint PGP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomParticipant {
  /// Fingerprint PGP 40 hex du participant.
  pub fp: String,
  /// Date à laquelle le participant a rejoint le salon (RFC 3339).
  pub joined_at: DateTime<Utc>,
}

/// Un salon de chat persisté.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
  /// Identifiant unique du salon (UUID v4).
  pub id: String,
  /// Nom local du salon (non signé, éditable).
  pub name: String,
  /// URL du broker MQTT (`mqtts://host:8883` recommandé).
  pub relay: String,
  /// Fingerprint 40 hex de l'identité locale utilisée dans ce salon.
  /// Nécessaire pour les utilisateurs ayant plusieurs clefs privées.
  pub my_fp: String,
  /// Date de création du salon (RFC 3339).
  pub created_at: DateTime<Utc>,
  /// Liste des participants (inclut l'utilisateur local).
  pub participants: Vec<RoomParticipant>,
}

impl Room {
  /// Retourne le topic MQTT pour les messages de ce salon.
  ///
  /// Forme : `pgpilot/chat/{sha256_hex(room_id)[..16]}`
  ///
  /// Le `room_id` est haché pour qu'un observateur extérieur ne puisse pas
  /// reconstituer le nom du salon depuis le topic.
  #[must_use]
  pub fn chat_topic(&self) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(self.id.as_bytes());
    format!(
      "{}/{:016x}",
      crate::chat::CHAT_TOPIC_PREFIX,
      u64::from_be_bytes(hash[..8].try_into().unwrap())
    )
  }
}

/// Enveloppe de persistance YAML contenant tous les salons.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RoomStore {
  pub rooms: Vec<Room>,
}

impl RoomStore {
  /// Retourne le chemin vers `~/.config/pgpilot/rooms.yaml`.
  #[must_use]
  pub fn path() -> PathBuf {
    dirs::config_dir()
      .unwrap_or_else(|| PathBuf::from("."))
      .join("pgpilot")
      .join("rooms.yaml")
  }

  /// Taille maximale autorisée pour le fichier `rooms.yaml` (1 Mio).
  const MAX_ROOMS_YAML_BYTES: u64 = 1_048_576;

  /// Charge le store depuis le disque.
  ///
  /// Si le fichier est absent, retourne `Self::default()` (pas d'erreur).
  ///
  /// # Errors
  ///
  /// Retourne [`ChatError::RoomsYamlLoadFailed`] si le fichier existe mais
  /// ne peut pas être lu, dépasse la taille maximale, ou n'est pas parsable.
  pub fn load() -> ChatResult<Self> {
    let path = Self::path();
    if !path.exists() {
      return Ok(Self::default());
    }
    // Correction 7 : vérifier la taille avant lecture pour éviter d'allouer
    // une quantité arbitraire de mémoire sur un fichier corrompu ou malveillant.
    let meta =
      std::fs::metadata(&path).map_err(|e| ChatError::RoomsYamlLoadFailed(e.to_string()))?;
    if meta.len() > Self::MAX_ROOMS_YAML_BYTES {
      return Err(ChatError::RoomsYamlLoadFailed(
        "rooms.yaml trop volumineux (> 1 Mio)".to_string(),
      ));
    }
    let content =
      std::fs::read_to_string(&path).map_err(|e| ChatError::RoomsYamlLoadFailed(e.to_string()))?;
    serde_yaml::from_str(&content).map_err(|e| ChatError::RoomsYamlLoadFailed(e.to_string()))
  }

  /// Persiste le store sur le disque.
  ///
  /// Crée le répertoire parent si nécessaire.
  ///
  /// # Errors
  ///
  /// Retourne [`ChatError::RoomsYamlSaveFailed`] en cas d'erreur I/O ou de
  /// sérialisation.
  pub fn save(&self) -> ChatResult<()> {
    let path = Self::path();
    if let Some(parent) = path.parent() {
      std::fs::create_dir_all(parent).map_err(|e| ChatError::RoomsYamlSaveFailed(e.to_string()))?;
    }
    let yaml =
      serde_yaml::to_string(self).map_err(|e| ChatError::RoomsYamlSaveFailed(e.to_string()))?;
    std::fs::write(&path, yaml).map_err(|e| ChatError::RoomsYamlSaveFailed(e.to_string()))
  }

  /// Retourne une référence vers un salon par son identifiant.
  #[must_use]
  pub fn get(&self, id: &str) -> Option<&Room> {
    self.rooms.iter().find(|r| r.id == id)
  }

  /// Insère ou remplace un salon (upsert par `id`).
  pub fn upsert(&mut self, room: Room) {
    if let Some(existing) = self.rooms.iter_mut().find(|r| r.id == room.id) {
      *existing = room;
    } else {
      self.rooms.push(room);
    }
  }

  /// Supprime un salon par son identifiant et le retourne, ou `None` s'il
  /// n'existait pas.
  pub fn remove(&mut self, id: &str) -> Option<Room> {
    let pos = self.rooms.iter().position(|r| r.id == id)?;
    Some(self.rooms.remove(pos))
  }
}

// ---------------------------------------------------------------------------
// Code d'invitation (JoinCode)
// ---------------------------------------------------------------------------

/// Code d'invitation partageable hors-bande pour rejoindre un salon.
///
/// Encodage : `serde_json(JoinCode) → bytes → base64url-no-pad →
/// "pgpilot:join:<base64>"`
///
/// La signature PGP couvre `room_id || \x00 || relay || \x00 || invited_by`
/// afin d'empêcher la forgerie d'invitation (redirection vers un broker
/// malveillant). `room_name` n'est **pas** signé : c'est un hint d'affichage
/// que chaque destinataire peut remplacer localement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinCode {
  /// UUID v4 du salon.
  pub room_id: String,
  /// URL du broker MQTT (`mqtts://host:8883` — TLS exigé par défaut).
  pub relay: String,
  /// Fingerprint 40 hex de l'invitant.
  pub invited_by: String,
  /// Nom indicatif du salon (non signé, peut être `None`).
  pub room_name: Option<String>,
  /// Signature PGP détachée (armored) sur la partie signée du code.
  pub sig: String,
}

impl JoinCode {
  /// Retourne les octets à signer/vérifier.
  ///
  /// Forme : `room_id || \x00 || relay || \x00 || invited_by`
  #[must_use]
  pub fn signed_bytes(&self) -> Vec<u8> {
    let mut buf =
      Vec::with_capacity(self.room_id.len() + 1 + self.relay.len() + 1 + self.invited_by.len());
    buf.extend_from_slice(self.room_id.as_bytes());
    buf.push(b'\x00');
    buf.extend_from_slice(self.relay.as_bytes());
    buf.push(b'\x00');
    buf.extend_from_slice(self.invited_by.as_bytes());
    buf
  }

  /// Encode le join code en chaîne partageable.
  ///
  /// Forme : `"pgpilot:join:<base64url-no-pad>"`
  ///
  /// # Errors
  ///
  /// Retourne [`ChatError::InvalidJoinCode`] si la sérialisation JSON échoue
  /// (ne devrait jamais se produire).
  pub fn encode(&self) -> ChatResult<String> {
    use base64::Engine as _;
    let json = serde_json::to_vec(self).map_err(|_| ChatError::InvalidJoinCode)?;
    let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&json);
    Ok(format!("{JOIN_CODE_PREFIX}{b64}"))
  }

  /// Décode une chaîne de join code.
  ///
  /// # Errors
  ///
  /// - [`ChatError::InvalidJoinCode`] si le préfixe est absent, si le
  ///   base64 est invalide, si le JSON est malformé, ou si les champs ne
  ///   passent pas la validation.
  /// - [`ChatError::JoinCodeSignatureInvalid`] si le champ `sig` est vide.
  pub fn decode(s: &str) -> ChatResult<Self> {
    use base64::Engine as _;
    let b64 = s
      .strip_prefix(JOIN_CODE_PREFIX)
      .ok_or(ChatError::InvalidJoinCode)?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
      .decode(b64)
      .map_err(|_| ChatError::InvalidJoinCode)?;
    let jc: Self = serde_json::from_slice(&bytes).map_err(|_| ChatError::InvalidJoinCode)?;

    // Correction 5 : validation des champs après désérialisation.

    // Valider room_id comme UUID.
    uuid::Uuid::parse_str(&jc.room_id).map_err(|_| ChatError::InvalidJoinCode)?;

    // Valider invited_by comme fingerprint 40-hex.
    if jc.invited_by.len() != 40 || !jc.invited_by.chars().all(|c| c.is_ascii_hexdigit()) {
      return Err(ChatError::InvalidJoinCode);
    }

    // Valider relay via parse_relay_url (rejette mqtt://localhost.evil.com, schémas inconnus…).
    crate::chat::mqtt::parse_relay_url(&jc.relay).map_err(|_| ChatError::InvalidJoinCode)?;

    // Borner room_name à 256 octets.
    if jc.room_name.as_ref().map_or(0, |n| n.len()) > 256 {
      return Err(ChatError::InvalidJoinCode);
    }

    // sig non vide (la vérification cryptographique a lieu dans verify()).
    if jc.sig.is_empty() {
      return Err(ChatError::JoinCodeSignatureInvalid);
    }

    Ok(jc)
  }

  /// Signe les octets du join code avec la clef locale et retourne la signature armored.
  ///
  /// Utilise `gpg --batch --armor --detach-sign --local-user <signer_fp>`.
  ///
  /// # Errors
  ///
  /// - [`ChatError::InvalidJoinCode`] — erreur I/O lors de la signature.
  /// - [`ChatError::JoinCodeSignatureInvalid`] — gpg a échoué à signer.
  pub fn sign(&self, homedir: &str, signer_fp: &str) -> ChatResult<String> {
    use crate::gpg::gpg_command;

    let signed_bytes = self.signed_bytes();

    let mut child = gpg_command(homedir)
      .args([
        "--batch",
        "--armor",
        "--detach-sign",
        "--local-user",
        signer_fp,
      ])
      .stdin(std::process::Stdio::piped())
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::null())
      .spawn()
      .map_err(|_| ChatError::InvalidJoinCode)?;

    if let Some(stdin) = child.stdin.as_mut() {
      stdin
        .write_all(&signed_bytes)
        .map_err(|_| ChatError::InvalidJoinCode)?;
    }

    let out = child
      .wait_with_output()
      .map_err(|_| ChatError::JoinCodeSignatureInvalid)?;

    if !out.status.success() {
      return Err(ChatError::JoinCodeSignatureInvalid);
    }

    String::from_utf8(out.stdout).map_err(|_| ChatError::JoinCodeSignatureInvalid)
  }

  /// Vérifie la signature du code d'invitation via le keyring GPG local.
  ///
  /// Utilise `gpg --batch --verify` pour valider la signature détachée
  /// portée par le champ `sig` sur les octets retournés par `signed_bytes()`.
  ///
  /// # Errors
  ///
  /// - [`ChatError::JoinCodeSignatureInvalid`] — signature absente ou invalide.
  /// - [`ChatError::InvalidJoinCode`] — erreur I/O lors de la vérification.
  pub fn verify(&self, homedir: &str) -> ChatResult<()> {
    use crate::gpg::gpg_command;

    if self.sig.is_empty() {
      return Err(ChatError::JoinCodeSignatureInvalid);
    }

    let data = self.signed_bytes();

    // Écrire les données dans un fichier temporaire.
    let mut data_tmp = tempfile::NamedTempFile::new().map_err(|_| ChatError::InvalidJoinCode)?;
    data_tmp
      .write_all(&data)
      .map_err(|_| ChatError::InvalidJoinCode)?;

    // Écrire la signature dans un fichier temporaire.
    let mut sig_tmp = tempfile::NamedTempFile::new().map_err(|_| ChatError::InvalidJoinCode)?;
    sig_tmp
      .write_all(self.sig.as_bytes())
      .map_err(|_| ChatError::InvalidJoinCode)?;

    let out = gpg_command(homedir)
      .args([
        "--batch",
        "--verify",
        sig_tmp.path().to_str().unwrap_or(""),
        data_tmp.path().to_str().unwrap_or(""),
      ])
      .output()
      .map_err(|_| ChatError::JoinCodeSignatureInvalid)?;

    if out.status.success() {
      Ok(())
    } else {
      Err(ChatError::JoinCodeSignatureInvalid)
    }
  }
}
