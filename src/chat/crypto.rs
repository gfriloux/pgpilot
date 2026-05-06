#![allow(dead_code)]
//! Cryptographie PGP pour le chat — via subprocesses GPG.
//!
//! Utilise `gpg` en subprocess (comme `keyring.rs`) plutôt que sequoia
//! in-process, ce qui permet de gérer les clefs protégées par passphrase via
//! pinentry, les YubiKey, etc.

use std::io::Write as _;

use crate::chat::{ChatError, ChatPayload, ChatResult, VerifiedMessage};
use crate::gpg::{gnupg_dir, gpg_command, sanitize_gpg_stderr};

/// Alias de type pour un fingerprint PGP 40 hex.
pub type Fingerprint = String;

// ---------------------------------------------------------------------------
// ChatCryptoCtx
// ---------------------------------------------------------------------------

/// Contexte cryptographique de session pour le chat.
///
/// Stocke uniquement le répertoire GPG et le fingerprint local.
/// Toutes les opérations crypto passent par des subprocesses `gpg`.
#[derive(Debug)]
pub struct ChatCryptoCtx {
  pub homedir: String,
  pub local_fp: Fingerprint,
}

impl ChatCryptoCtx {
  /// Vérifie que la clef privée locale est disponible dans le keyring.
  pub fn load(local_fp: &str, _peers: &[Fingerprint]) -> ChatResult<Self> {
    crate::gpg::validate_fp(local_fp).map_err(|e| ChatError::InvalidFingerprint(e.to_string()))?;
    let homedir = gnupg_dir().map_err(|e| ChatError::InvalidConfig(format!("GPG homedir: {e}")))?;

    // Vérifier que la clef secrète existe.
    let out = gpg_command(&homedir)
      .args(["--list-secret-keys", "--with-colons", local_fp])
      .output()
      .map_err(|_| ChatError::NoSigningKey)?;

    if !out.status.success() {
      return Err(ChatError::NoSigningKey);
    }

    Ok(Self {
      homedir,
      local_fp: local_fp.to_string(),
    })
  }

  /// Chiffre `plaintext` pour tous les `recipients` et signe avec la clef locale.
  ///
  /// Utilise `gpg --encrypt --sign --armor`.
  /// GPG gère la passphrase via pinentry si la clef est protégée.
  pub fn encrypt_for_room(
    &self,
    plaintext: &str,
    recipients: &[Fingerprint],
  ) -> ChatResult<ChatPayload> {
    let mut all_args: Vec<String> = vec![
      "--batch".into(),
      "--yes".into(),
      "--armor".into(),
      "--trust-model".into(),
      "always".into(), // contourne la web-of-trust ; l'utilisateur a choisi les destinataires
      "--encrypt".into(),
      "--sign".into(),
      "--local-user".into(),
      self.local_fp.clone(),
    ];
    for fp in recipients {
      all_args.push("--recipient".into());
      all_args.push(fp.clone());
    }

    let mut child = gpg_command(&self.homedir)
      .args(&all_args)
      .stdin(std::process::Stdio::piped())
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped())
      .spawn()
      .map_err(|e| ChatError::EncryptFailed(format!("gpg spawn: {e}")))?;

    if let Some(stdin) = child.stdin.as_mut() {
      stdin
        .write_all(plaintext.as_bytes())
        .map_err(|e| ChatError::EncryptFailed(format!("stdin write: {e}")))?;
    }

    let output = child
      .wait_with_output()
      .map_err(|e| ChatError::EncryptFailed(format!("gpg wait: {e}")))?;

    if !output.status.success() {
      return Err(ChatError::EncryptFailed(sanitize_gpg_stderr(
        &String::from_utf8_lossy(&output.stderr),
      )));
    }

    let ciphertext = String::from_utf8(output.stdout)
      .map_err(|e| ChatError::EncryptFailed(format!("utf8: {e}")))?;

    // La signature est intégrée dans le message PGP chiffré.
    // WireMessage.signature est réservé à une future signature détachée canonique.
    Ok(ChatPayload {
      ciphertext_armored: ciphertext,
      signature_armored: String::new(),
    })
  }

  /// Déchiffre un `ChatPayload` et retourne le texte clair + fingerprint signataire.
  ///
  /// Utilise `gpg --decrypt --armor --batch`.
  /// GPG vérifie automatiquement la signature intégrée.
  pub fn decrypt_message(&self, payload: &ChatPayload) -> ChatResult<VerifiedMessage> {
    let mut child = gpg_command(&self.homedir)
      .args(["--batch", "--yes", "--decrypt", "--status-fd", "2"])
      .stdin(std::process::Stdio::piped())
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::piped())
      .spawn()
      .map_err(|e| ChatError::DecryptFailed(format!("gpg spawn: {e}")))?;

    if let Some(stdin) = child.stdin.as_mut() {
      stdin
        .write_all(payload.ciphertext_armored.as_bytes())
        .map_err(|e| ChatError::DecryptFailed(format!("stdin write: {e}")))?;
    }

    let output = child
      .wait_with_output()
      .map_err(|e| ChatError::DecryptFailed(format!("gpg wait: {e}")))?;

    let stderr_str = String::from_utf8_lossy(&output.stderr);
    let decryption_ok = stderr_str.contains("[GNUPG:] DECRYPTION_OKAY");

    if !output.status.success() {
      // Distinguer "clef publique du signataire absente" de "déchiffrement échoué".
      // Si le déchiffrement a réussi mais la signature ne peut pas être vérifiée
      // (clef publique manquante), on retourne UnknownSender — le message EST lisible
      // mais l'identité de l'expéditeur ne peut pas être prouvée.
      if decryption_ok
        && (stderr_str.contains("[GNUPG:] NO_PUBKEY") || stderr_str.contains("[GNUPG:] ERRSIG"))
      {
        return Err(ChatError::UnknownSender(
          "public key not in keyring — import sender's key to verify".to_string(),
        ));
      }
      return Err(ChatError::DecryptFailed(sanitize_gpg_stderr(&stderr_str)));
    }

    let plaintext = String::from_utf8(output.stdout)
      .map_err(|e| ChatError::DecryptFailed(format!("utf8: {e}")))?;

    // Extraire le fingerprint 40-hex de la CLEF MAÎTRE depuis VALIDSIG.
    // Format : [GNUPG:] VALIDSIG <subkey_fp> <date> <ts> ... <primary_fp>
    // Le dernier champ est la clef maître — c'est ce qu'on compare avec wire.sender
    // (les participants sont identifiés par leur clef maître, pas leur sous-clef).
    let signer_fp = stderr_str
      .lines()
      .find(|l| l.contains("[GNUPG:] VALIDSIG"))
      .and_then(|l| l.split_whitespace().last())
      .filter(|fp| fp.len() == 40 && fp.chars().all(|c| c.is_ascii_hexdigit()))
      .map(|fp| fp.to_string())
      .ok_or(ChatError::SignatureInvalid)?;

    Ok(VerifiedMessage {
      plaintext,
      signer_fp,
      signed_at: chrono::Utc::now(),
    })
  }
}
