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
//!
//! Les méthodes métier sont implémentées dans l'axe 4 (crypto).

use std::collections::HashMap;

use crate::chat::{ChatPayload, ChatResult, VerifiedMessage};

/// Alias de type pour un fingerprint PGP 40 hex.
pub type Fingerprint = String;

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
  pub local_cert: sequoia_openpgp::Cert,
  /// Fingerprint 40 hex de l'identité locale.
  pub local_fp: Fingerprint,
  /// Certificats publics des participants connus (clef = fingerprint 40 hex).
  pub peers: HashMap<Fingerprint, sequoia_openpgp::Cert>,
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
  pub fn load(_local_fp: &str, _peers: &[Fingerprint]) -> ChatResult<Self> {
    todo!("impl in axe 4 (crypto)")
  }

  /// Chiffre `plaintext` pour tous les `recipients` et produit un
  /// [`ChatPayload`] (ciphertext armored + signature armored).
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
    _plaintext: &str,
    _recipients: &[Fingerprint],
  ) -> ChatResult<ChatPayload> {
    todo!("impl in axe 4 (crypto)")
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
  pub fn decrypt_message(&self, _payload: &ChatPayload) -> ChatResult<VerifiedMessage> {
    todo!("impl in axe 4 (crypto)")
  }
}
