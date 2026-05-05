#![allow(dead_code)]
//! Cryptographie PGP in-process pour le chat.
//!
//! [`ChatCryptoCtx`] est le contexte cryptographique par session. Il est
//! chargé une fois (via [`ChatCryptoCtx::load`] dans un `blocking_task`) et
//! maintenu en mémoire dans `App.chat_crypto: Option<Arc<ChatCryptoCtx>>`.
//!
//! ## Compromis de sécurité assumés (v0.6.0)
//!
//! - La clef privée reste en RAM toute la session (pas de zeroize).
//! - Les clefs YubiKey ne sont pas exportables → refus à l'ouverture du
//!   premier salon avec [`ChatError::SignFailed`].
//! - Pas de forward secrecy (cf. hors-scope §12 de la spec).

use std::collections::HashMap;
use std::io::Write as _;

use sequoia_openpgp::cert::CertParser;
use sequoia_openpgp::parse::stream::{
  DecryptionHelper, DecryptorBuilder, MessageLayer, MessageStructure, VerificationHelper,
};
use sequoia_openpgp::parse::Parse;
use sequoia_openpgp::policy::StandardPolicy;
use sequoia_openpgp::serialize::stream::{Armorer, Encryptor, LiteralWriter, Message, Signer};
use sequoia_openpgp::types::SymmetricAlgorithm;
use sequoia_openpgp::{Cert, KeyHandle};

use crate::chat::{ChatError, ChatPayload, ChatResult, VerifiedMessage};
use crate::gpg::{gnupg_dir, gpg_command};

/// Alias de type pour un fingerprint PGP 40 hex.
pub type Fingerprint = String;

// ---------------------------------------------------------------------------
// ChatCryptoCtx
// ---------------------------------------------------------------------------

/// Contexte cryptographique de session pour le chat.
///
/// Chargé une fois au premier accès au chat et stocké dans
/// `App.chat_crypto: Option<Arc<ChatCryptoCtx>>`.
///
/// # Thread safety
///
/// `ChatCryptoCtx` est `Send + Sync` car `sequoia_openpgp::Cert` l'est.
/// Il est enveloppé dans `Arc` pour éviter de cloner les `Cert` (volumineux)
/// à chaque envoi de message.
pub struct ChatCryptoCtx {
  /// Certificat sequoia complet (clef publique + privée) de l'utilisateur local.
  pub local_cert: Cert,
  /// Fingerprint 40 hex de l'identité locale.
  pub local_fp: Fingerprint,
  /// Certificats publics des participants connus (clef = fingerprint 40 hex).
  pub peers: HashMap<Fingerprint, Cert>,
}

impl std::fmt::Debug for ChatCryptoCtx {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ChatCryptoCtx")
      .field("local_fp", &self.local_fp)
      .field("peers_count", &self.peers.len())
      .finish_non_exhaustive()
  }
}

impl ChatCryptoCtx {
  /// Charge le contexte cryptographique depuis le keyring GPG local.
  ///
  /// Exécute `gpg --export-secret-keys --armor <local_fp>` et charge les
  /// certificats publics des participants via `gpg --export`.
  ///
  /// **Doit être appelé dans un `blocking_task`** (opérations GPG bloquantes).
  ///
  /// # Errors
  ///
  /// - [`ChatError::NoSigningKey`] — fingerprint introuvable dans le keyring.
  /// - [`ChatError::SignFailed`] — clef sur smartcard non exportable.
  /// - [`ChatError::ParticipantNotInKeyring`] — un peer est absent.
  pub fn load(local_fp: &str, peers: &[Fingerprint]) -> ChatResult<Self> {
    let homedir = gnupg_dir().map_err(|e| ChatError::InvalidConfig(format!("GPG homedir: {e}")))?;

    // Export de la clef privée locale via gpg.
    // Si la clef est sur une smartcard, gpg ne peut pas l'exporter → erreur.
    let secret_output = gpg_command(&homedir)
      .args(["--export-secret-keys", "--armor", local_fp])
      .output()
      .map_err(|e| ChatError::SignFailed(format!("Impossible de lancer gpg: {e}")))?;

    if !secret_output.status.success() {
      let stderr = String::from_utf8_lossy(&secret_output.stderr);
      if stderr.contains("smartcard") || stderr.contains("card") {
        return Err(ChatError::SignFailed(
          "clef sur smartcard non supportée pour le chat v0.6.0 \
          (créez ou importez une clef logicielle)"
            .to_string(),
        ));
      }
      return Err(ChatError::NoSigningKey);
    }

    let secret_armor = String::from_utf8(secret_output.stdout)
      .map_err(|e| ChatError::SignFailed(format!("UTF-8 invalide pour la clef secrète: {e}")))?;

    if secret_armor.trim().is_empty() || !secret_armor.contains("BEGIN PGP") {
      return Err(ChatError::NoSigningKey);
    }

    // Parse du Cert sequoia depuis l'armored secret key.
    let mut certs: Vec<Cert> = CertParser::from_reader(secret_armor.as_bytes())
      .map_err(|e| ChatError::SignFailed(format!("Parse clef secrète: {e}")))?
      .filter_map(|c| c.ok())
      .collect();

    let local_cert = certs.pop().ok_or(ChatError::NoSigningKey)?;

    // Chargement des clefs publiques des peers.
    let mut peer_map: HashMap<Fingerprint, Cert> = HashMap::new();
    for peer_fp in peers {
      if peer_fp.to_uppercase() == local_fp.to_uppercase() {
        // L'utilisateur local est un participant — on réutilise local_cert.
        peer_map.insert(peer_fp.clone(), local_cert.clone());
        continue;
      }

      let pub_output = gpg_command(&homedir)
        .args(["--export", "--armor", peer_fp])
        .output()
        .map_err(|e| {
          ChatError::ParticipantNotInKeyring(format!("gpg --export {peer_fp} a échoué: {e}"))
        })?;

      let pub_armor = String::from_utf8(pub_output.stdout)
        .map_err(|_| ChatError::ParticipantNotInKeyring(peer_fp.clone()))?;

      if pub_armor.trim().is_empty() || !pub_armor.contains("BEGIN PGP") {
        return Err(ChatError::ParticipantNotInKeyring(peer_fp.clone()));
      }

      let peer_certs: Vec<Cert> = CertParser::from_reader(pub_armor.as_bytes())
        .map_err(|e| {
          ChatError::ParticipantNotInKeyring(format!("Parse clef publique {peer_fp}: {e}"))
        })?
        .filter_map(|c| c.ok())
        .collect();

      let peer_cert = peer_certs
        .into_iter()
        .next()
        .ok_or_else(|| ChatError::ParticipantNotInKeyring(peer_fp.clone()))?;

      peer_map.insert(peer_fp.clone(), peer_cert);
    }

    Ok(Self {
      local_cert,
      local_fp: local_fp.to_string(),
      peers: peer_map,
    })
  }

  /// Chiffre `plaintext` pour tous les `recipients` et produit un
  /// [`ChatPayload`] (ciphertext armored + signature détachée armored).
  ///
  /// **Doit être appelé dans un `blocking_task`** (~10–50 ms par message).
  ///
  /// # Errors
  ///
  /// - [`ChatError::EncryptFailed`] — échec du chiffrement sequoia.
  /// - [`ChatError::SignFailed`] — échec de la signature.
  /// - [`ChatError::ParticipantNotInKeyring`] — un destinataire est inconnu.
  pub fn encrypt_for_room(
    &self,
    plaintext: &str,
    recipients: &[Fingerprint],
  ) -> ChatResult<ChatPayload> {
    let policy = StandardPolicy::new();

    // Construire la liste des recipients sequoia (sous-clefs de chiffrement valides).
    let mut sq_recipients: Vec<sequoia_openpgp::serialize::stream::Recipient<'_>> = Vec::new();
    for fp in recipients {
      let cert = self
        .peers
        .get(fp)
        .ok_or_else(|| ChatError::ParticipantNotInKeyring(fp.clone()))?;

      let enc_keys: Vec<_> = cert
        .keys()
        .with_policy(&policy, None)
        .supported()
        .alive()
        .revoked(false)
        .for_transport_encryption()
        .collect();

      if enc_keys.is_empty() {
        return Err(ChatError::EncryptFailed(format!(
          "Aucune clef de chiffrement valide pour le destinataire {fp}"
        )));
      }
      for ka in enc_keys {
        sq_recipients.push(ka.into());
      }
    }

    // Clef de signature locale.
    let signing_keypair = self
      .local_cert
      .keys()
      .secret()
      .with_policy(&policy, None)
      .supported()
      .alive()
      .revoked(false)
      .for_signing()
      .next()
      .ok_or(ChatError::NoSigningKey)?
      .key()
      .clone()
      .into_keypair()
      .map_err(|e| ChatError::SignFailed(format!("Keypair: {e}")))?;

    // Chiffrement + signature intégrée via sequoia stream.
    let mut ciphertext_buf: Vec<u8> = Vec::new();
    {
      let message = Message::new(&mut ciphertext_buf);
      let message = Armorer::new(message)
        .build()
        .map_err(|e| ChatError::EncryptFailed(format!("Armorer: {e}")))?;
      let message = Encryptor::for_recipients(message, sq_recipients)
        .build()
        .map_err(|e| ChatError::EncryptFailed(format!("Encryptor: {e}")))?;
      let message = Signer::new(message, signing_keypair)
        .map_err(|e| ChatError::SignFailed(format!("Signer::new: {e}")))?
        .build()
        .map_err(|e| ChatError::SignFailed(format!("Signer::build: {e}")))?;
      let mut literal = LiteralWriter::new(message)
        .build()
        .map_err(|e| ChatError::EncryptFailed(format!("LiteralWriter: {e}")))?;
      literal
        .write_all(plaintext.as_bytes())
        .map_err(|e| ChatError::EncryptFailed(format!("write_all: {e}")))?;
      literal
        .finalize()
        .map_err(|e| ChatError::EncryptFailed(format!("finalize: {e}")))?;
    }

    let ciphertext_armored =
      String::from_utf8(ciphertext_buf).map_err(|e| ChatError::EncryptFailed(e.to_string()))?;

    // Signature détachée sur le ciphertext pour la vérification canonicalisée
    // du WireMessage côté récepteur (spec §6.1).
    let sign_keypair2 = self
      .local_cert
      .keys()
      .secret()
      .with_policy(&policy, None)
      .supported()
      .alive()
      .revoked(false)
      .for_signing()
      .next()
      .ok_or(ChatError::NoSigningKey)?
      .key()
      .clone()
      .into_keypair()
      .map_err(|e| ChatError::SignFailed(format!("Keypair sig détachée: {e}")))?;

    let mut sig_buf: Vec<u8> = Vec::new();
    {
      let message = Message::new(&mut sig_buf);
      let message = Armorer::new(message)
        .kind(sequoia_openpgp::armor::Kind::Signature)
        .build()
        .map_err(|e| ChatError::SignFailed(format!("Armorer sig: {e}")))?;
      let mut signer = Signer::new(message, sign_keypair2)
        .map_err(|e| ChatError::SignFailed(format!("Signer::new détaché: {e}")))?
        .detached()
        .build()
        .map_err(|e| ChatError::SignFailed(format!("Signer::build détaché: {e}")))?;
      signer
        .write_all(ciphertext_armored.as_bytes())
        .map_err(|e| ChatError::SignFailed(format!("write sig: {e}")))?;
      signer
        .finalize()
        .map_err(|e| ChatError::SignFailed(format!("finalize sig: {e}")))?;
    }

    let signature_armored =
      String::from_utf8(sig_buf).map_err(|e| ChatError::SignFailed(e.to_string()))?;

    Ok(ChatPayload {
      ciphertext_armored,
      signature_armored,
    })
  }

  /// Déchiffre et vérifie un [`ChatPayload`].
  ///
  /// Retourne le plaintext, le fingerprint du signataire et l'horodatage
  /// de signature.
  ///
  /// **Doit être appelé dans un `blocking_task`**.
  ///
  /// # Errors
  ///
  /// - [`ChatError::DecryptFailed`] — impossible de déchiffrer.
  /// - [`ChatError::SignatureInvalid`] — signature non vérifiable.
  /// - [`ChatError::UnknownSender`] — signataire absent des peers connus.
  pub fn decrypt_message(&self, payload: &ChatPayload) -> ChatResult<VerifiedMessage> {
    let policy = StandardPolicy::new();

    // Déchiffrement via sequoia DecryptorBuilder.
    // Note : DecryptorBuilder::with_policy prend ownership du helper, on
    // récupère les métadonnées via decryptor.helper_ref() après lecture.
    let helper = ChatDecryptHelper {
      local_cert: &self.local_cert,
      peers: &self.peers,
      policy: &policy,
      verified_signer_fp: None,
      verified_signed_at: None,
    };

    let mut decrypted_buf: Vec<u8> = Vec::new();

    use std::io::Read as _;
    let mut decryptor = DecryptorBuilder::from_reader(payload.ciphertext_armored.as_bytes())
      .map_err(|e| ChatError::DecryptFailed(format!("Lecture ciphertext: {e}")))?
      .with_policy(&policy, None, helper)
      .map_err(|e| ChatError::DecryptFailed(format!("Politique déchiffrement: {e}")))?;

    decryptor
      .read_to_end(&mut decrypted_buf)
      .map_err(|e| ChatError::DecryptFailed(format!("Lecture plaintext: {e}")))?;

    // Récupérer les métadonnées de vérification depuis le helper.
    let h = decryptor.helper_ref();
    let signer_fp = h
      .verified_signer_fp
      .clone()
      .ok_or(ChatError::SignatureInvalid)?;
    let signed_at = h.verified_signed_at.ok_or(ChatError::SignatureInvalid)?;

    // Vérifier que le signataire est connu.
    if !self.peers.contains_key(&signer_fp) {
      return Err(ChatError::UnknownSender(signer_fp));
    }

    let plaintext = String::from_utf8(decrypted_buf)
      .map_err(|e| ChatError::DecryptFailed(format!("UTF-8 plaintext: {e}")))?;

    Ok(VerifiedMessage {
      plaintext,
      signer_fp,
      signed_at,
    })
  }

  /// Vérifie une signature détachée (armored) sur `data` bytes.
  ///
  /// Utilise [`sequoia_openpgp::parse::stream::DetachedVerifierBuilder`].
  /// Retourne `(signer_fp_40hex, signed_at)` si valide.
  ///
  /// # Errors
  ///
  /// - [`ChatError::SignatureInvalid`] — signature invalide ou aucun signataire.
  /// - [`ChatError::UnknownSender`] — signataire absent des peers connus.
  pub fn verify_detached_sig(
    &self,
    data: &[u8],
    sig_armored: &str,
  ) -> ChatResult<(Fingerprint, chrono::DateTime<chrono::Utc>)> {
    use sequoia_openpgp::parse::stream::DetachedVerifierBuilder;

    let policy = StandardPolicy::new();
    let helper = DetachedHelper {
      peers: &self.peers,
      found_fp: None,
      found_ts: None,
    };

    let mut verifier = DetachedVerifierBuilder::from_reader(sig_armored.as_bytes())
      .map_err(|_| ChatError::SignatureInvalid)?
      .with_policy(&policy, None, helper)
      .map_err(|_| ChatError::SignatureInvalid)?;

    verifier
      .verify_bytes(data)
      .map_err(|_| ChatError::SignatureInvalid)?;

    let final_helper = verifier.into_helper();
    let signer_fp = final_helper.found_fp.ok_or(ChatError::SignatureInvalid)?;
    let signed_at = final_helper.found_ts.ok_or(ChatError::SignatureInvalid)?;

    if !self.peers.contains_key(&signer_fp) {
      return Err(ChatError::UnknownSender(signer_fp));
    }

    Ok((signer_fp, signed_at))
  }
}

// ---------------------------------------------------------------------------
// Helper sequoia pour le déchiffrement + vérification intégrés
// ---------------------------------------------------------------------------

struct ChatDecryptHelper<'a> {
  local_cert: &'a Cert,
  peers: &'a HashMap<Fingerprint, Cert>,
  policy: &'a StandardPolicy<'a>,
  /// Fingerprint du signataire extrait lors de `check()`.
  verified_signer_fp: Option<Fingerprint>,
  /// Horodatage de signature extrait lors de `check()`.
  verified_signed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl<'a> VerificationHelper for ChatDecryptHelper<'a> {
  fn get_certs(&mut self, ids: &[KeyHandle]) -> sequoia_openpgp::Result<Vec<Cert>> {
    let mut result = Vec::new();
    for id in ids {
      for cert in self.peers.values() {
        if cert
          .keys()
          .any(|ka| id.aliases(KeyHandle::from(ka.key().fingerprint())))
        {
          result.push(cert.clone());
        }
      }
    }
    Ok(result)
  }

  fn check(&mut self, structure: MessageStructure<'_>) -> sequoia_openpgp::Result<()> {
    for layer in structure {
      if let MessageLayer::SignatureGroup { results } = layer {
        // sequoia 2.x: GoodChecksum a les champs `sig` et `ka`.
        // Prendre la première signature valide.
        if let Some(good_sig) = results.into_iter().flatten().next() {
          let fp = good_sig.ka.key().fingerprint().to_hex().to_uppercase();
          self.verified_signer_fp = Some(fp);
          let ts = good_sig
            .sig
            .signature_creation_time()
            .and_then(|t| {
              t.duration_since(std::time::SystemTime::UNIX_EPOCH)
                .ok()
                .and_then(|d| {
                  chrono::DateTime::<chrono::Utc>::from_timestamp(d.as_secs() as i64, 0)
                })
            })
            .unwrap_or_else(chrono::Utc::now);
          self.verified_signed_at = Some(ts);
        }
      }
    }
    // Aucune signature valide — verified_signer_fp restera None → SignatureInvalid.
    Ok(())
  }
}

impl<'a> DecryptionHelper for ChatDecryptHelper<'a> {
  fn decrypt(
    &mut self,
    pkesks: &[sequoia_openpgp::packet::PKESK],
    _skesks: &[sequoia_openpgp::packet::SKESK],
    sym_algo: Option<SymmetricAlgorithm>,
    decrypt: &mut dyn FnMut(
      Option<SymmetricAlgorithm>,
      &sequoia_openpgp::crypto::SessionKey,
    ) -> bool,
  ) -> sequoia_openpgp::Result<Option<Cert>> {
    // Collecter les sous-clefs de déchiffrement de la clef locale.
    let keys: Vec<_> = self
      .local_cert
      .keys()
      .secret()
      .with_policy(self.policy, None)
      .supported()
      .alive()
      .revoked(false)
      .for_transport_encryption()
      .collect();

    for pkesk in pkesks {
      for ka in &keys {
        if let Ok(mut keypair) = ka.key().clone().into_keypair() {
          if let Some((algo, sk)) = pkesk.decrypt(&mut keypair, sym_algo) {
            // algo is already Option<SymmetricAlgorithm> from pkesk.decrypt().
            if decrypt(algo, &sk) {
              return Ok(Some(self.local_cert.clone()));
            }
          }
        }
      }
    }

    Err(anyhow::anyhow!(
      "Impossible de déchiffrer : aucune clef locale correspondante"
    ))
  }
}

// ---------------------------------------------------------------------------
// Helper sequoia pour la vérification de signature détachée
// ---------------------------------------------------------------------------

struct DetachedHelper<'a> {
  peers: &'a HashMap<Fingerprint, Cert>,
  found_fp: Option<Fingerprint>,
  found_ts: Option<chrono::DateTime<chrono::Utc>>,
}

impl<'a> VerificationHelper for DetachedHelper<'a> {
  fn get_certs(&mut self, ids: &[KeyHandle]) -> sequoia_openpgp::Result<Vec<Cert>> {
    let mut result = Vec::new();
    for id in ids {
      for cert in self.peers.values() {
        if cert
          .keys()
          .any(|ka| id.aliases(KeyHandle::from(ka.key().fingerprint())))
        {
          result.push(cert.clone());
        }
      }
    }
    Ok(result)
  }

  fn check(&mut self, structure: MessageStructure<'_>) -> sequoia_openpgp::Result<()> {
    for layer in structure {
      if let MessageLayer::SignatureGroup { results } = layer {
        if let Some(good_sig) = results.into_iter().flatten().next() {
          let fp = good_sig.ka.key().fingerprint().to_hex().to_uppercase();
          self.found_fp = Some(fp);
          let ts = good_sig
            .sig
            .signature_creation_time()
            .and_then(|t| {
              t.duration_since(std::time::SystemTime::UNIX_EPOCH)
                .ok()
                .and_then(|d| {
                  chrono::DateTime::<chrono::Utc>::from_timestamp(d.as_secs() as i64, 0)
                })
            })
            .unwrap_or_else(chrono::Utc::now);
          self.found_ts = Some(ts);
        }
      }
    }
    Ok(())
  }
}
