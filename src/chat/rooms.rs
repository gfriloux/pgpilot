#![allow(dead_code)]
//! Salons de chat et persistance `rooms.yaml`.
//!
//! Ce module gère le CRUD des salons persistés dans
//! `~/.config/pgpilot/rooms.yaml` ainsi que l'encodage/décodage/vérification
//! des codes d'invitation ([`JoinCode`]).

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
    // Utilisation de sha2 n'est pas disponible sans l'ajouter en dépendance.
    // On délègue au futur axe 4 (crypto) via todo!, mais on fournit une
    // implémentation minimale fonctionnelle via sha256 from sequoia.
    //
    // Pour l'instant on utilise une implémentation inline simple : sequoia
    // n'exporte pas sha256 directement depuis son API publique, et ajouter
    // sha2 serait hors-scope de cet axe. On utilise donc une approximation
    // reproducible via `std::collections::hash_map::DefaultHasher` pour les
    // stubs — NOTE : ce n'est PAS sha256, il faudra remplacer dans l'axe 4.
    //
    // TODO(axe4): remplacer par sha256(self.id.as_bytes())[..16]
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hasher::write(&mut hasher, self.id.as_bytes());
    let hash = std::hash::Hasher::finish(&hasher);
    format!("{}/{:016x}", crate::chat::CHAT_TOPIC_PREFIX, hash)
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

  /// Charge le store depuis le disque.
  ///
  /// Si le fichier est absent, retourne `Self::default()` (pas d'erreur).
  ///
  /// # Errors
  ///
  /// Retourne [`ChatError::RoomsYamlLoadFailed`] si le fichier existe mais
  /// ne peut pas être lu ou parsé.
  pub fn load() -> ChatResult<Self> {
    let path = Self::path();
    if !path.exists() {
      return Ok(Self::default());
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
  ///   base64 est invalide ou si le JSON est malformé.
  pub fn decode(s: &str) -> ChatResult<Self> {
    use base64::Engine as _;
    let b64 = s
      .strip_prefix(JOIN_CODE_PREFIX)
      .ok_or(ChatError::InvalidJoinCode)?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
      .decode(b64)
      .map_err(|_| ChatError::InvalidJoinCode)?;
    serde_json::from_slice(&bytes).map_err(|_| ChatError::InvalidJoinCode)
  }

  /// Vérifie la signature du code d'invitation.
  ///
  /// La vérification utilise `gpg::validate_fp` puis le keyring local.
  /// Si la clef de l'invitant est absente, retourne
  /// [`ChatError::JoinCodeInviterUnknown`].
  ///
  /// # Errors
  ///
  /// - [`ChatError::JoinCodeInviterUnknown`] — clef absente.
  /// - [`ChatError::JoinCodeSignatureInvalid`] — signature invalide.
  pub fn verify(&self) -> ChatResult<()> {
    // TODO(axe4): impl via ChatCryptoCtx::verify_detached
    todo!("impl in axe 4 (crypto)")
  }
}
