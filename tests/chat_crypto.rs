//! Tests pour `ChatCryptoCtx` (`src/chat/crypto.rs`).
//!
//! La majorité des tests nécessite GPG → `#[ignore]`.
//! Exécuter avec : `cargo test --test chat_crypto -- --ignored`
//!
//! Pattern : `setup_test_gnupghome()` + clef créée on-the-fly via GPG dans
//! le homedir temporaire. Ne jamais toucher `$GNUPGHOME` réel.
//!
//! Les tests 2-4 construisent `ChatCryptoCtx` directement (champs `pub`)
//! pour éviter les races sur `GNUPGHOME` lors de l'exécution parallèle.
//! Seul le test 1 passe par `ChatCryptoCtx::load()` et utilise un mutex.

#![allow(dead_code)]

mod common;

use std::sync::Mutex;

use pgpilot::chat::crypto::ChatCryptoCtx;
use pgpilot::chat::ChatPayload;

/// Sérialise les tests qui doivent temporairement toucher `GNUPGHOME`.
static ENV_LOCK: Mutex<()> = Mutex::new(());

// ---------------------------------------------------------------------------
// Helper : génère une clef ed25519 + cv25519 dans le homedir de test.
//
// Utilise `--quick-gen-key` + `--quick-add-key`, même pattern que
// `pgpilot::gpg::create_key`. Retourne le fingerprint 40 hex.
// `--passphrase ""` évite pinentry dans un environnement sans terminal.
// ---------------------------------------------------------------------------

fn generate_test_key(homedir: &str, name: &str, email: &str) -> String {
  use std::process::Command;

  let user_id = format!("{name} <{email}>");

  // 1. Clef maître Ed25519 (cert only), sans expiration.
  let status = Command::new("gpg")
    .args([
      "--homedir",
      homedir,
      "--batch",
      "--passphrase",
      "",
      "--quick-gen-key",
      &user_id,
      "ed25519",
      "cert",
      "0",
    ])
    .status()
    .expect("gpg --quick-gen-key spawn");
  assert!(status.success(), "gpg --quick-gen-key a échoué");

  // Récupérer le fingerprint de la clef générée.
  let out = Command::new("gpg")
    .args(["--homedir", homedir, "--list-keys", "--with-colons"])
    .output()
    .expect("gpg list-keys");

  let fp = String::from_utf8_lossy(&out.stdout)
    .lines()
    .filter(|l| l.starts_with("fpr"))
    .map(|l| l.split(':').nth(9).unwrap_or("").to_string())
    .find(|f| !f.is_empty())
    .expect("fingerprint introuvable après --quick-gen-key");

  // 2. Sous-clef de signature Ed25519.
  let status = Command::new("gpg")
    .args([
      "--homedir",
      homedir,
      "--batch",
      "--passphrase",
      "",
      "--quick-add-key",
      &fp,
      "ed25519",
      "sign",
      "0",
    ])
    .status()
    .expect("gpg --quick-add-key sign spawn");
  assert!(status.success(), "ajout sous-clef sign a échoué");

  // 3. Sous-clef de chiffrement Cv25519.
  let status = Command::new("gpg")
    .args([
      "--homedir",
      homedir,
      "--batch",
      "--passphrase",
      "",
      "--quick-add-key",
      &fp,
      "cv25519",
      "encr",
      "0",
    ])
    .status()
    .expect("gpg --quick-add-key encr spawn");
  assert!(status.success(), "ajout sous-clef encr a échoué");

  fp
}

// ---------------------------------------------------------------------------
// 1. ChatCryptoCtx::load() réussit si la clef secrète existe
//
// Doit passer par `GNUPGHOME` → sérialisé par ENV_LOCK.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "nécessite la création d'une clef GPG (~5 s) — exécuter avec --ignored"]
fn crypto_ctx_load_succeeds_with_secret_key() {
  let _guard = ENV_LOCK.lock().expect("ENV_LOCK empoisonné");

  let (_tmp, homedir) = common::setup_test_gnupghome();
  // Surcharger GNUPGHOME pour que gnupg_dir() pointe vers notre homedir temp.
  // SAFETY : ENV_LOCK garantit l'exécution séquentielle pour ce test.
  unsafe {
    std::env::set_var("GNUPGHOME", &homedir);
  }

  let fp = generate_test_key(&homedir, "Alice Test", "alice@test.local");
  assert_eq!(fp.len(), 40, "fingerprint doit avoir 40 chars");

  // `load()` appelle `gnupg_dir()` → lit GNUPGHOME → notre homedir temp.
  let ctx = ChatCryptoCtx::load(&fp, &[]);
  assert!(
    ctx.is_ok(),
    "load() doit réussir quand la clef secrète existe, erreur : {:?}",
    ctx.err()
  );

  let ctx = ctx.unwrap();
  assert_eq!(
    ctx.local_fp, fp,
    "local_fp doit correspondre au fingerprint fourni"
  );

  unsafe {
    std::env::remove_var("GNUPGHOME");
  }
  // `_tmp` est droppé ici — le répertoire temporaire est supprimé.
}

// ---------------------------------------------------------------------------
// 2. encrypt_for_room() + decrypt_message() roundtrip
//
// Construit `ChatCryptoCtx` directement (champs pub) — pas de GNUPGHOME.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "nécessite la création d'une clef GPG (~5 s) — exécuter avec --ignored"]
fn encrypt_decrypt_roundtrip_self() {
  let (_tmp, homedir) = common::setup_test_gnupghome();
  let fp = generate_test_key(&homedir, "Bob Test", "bob@test.local");

  // Construire le contexte directement — évite la dépendance à GNUPGHOME.
  let ctx = ChatCryptoCtx {
    homedir: homedir.clone(),
    local_fp: fp.clone(),
  };

  let plaintext = "Bonjour, monde chiffré !";
  // Chiffrer pour soi-même (destinataire = clef locale).
  let payload = ctx
    .encrypt_for_room(plaintext, std::slice::from_ref(&fp))
    .expect("encrypt_for_room");

  assert!(
    !payload.ciphertext_armored.is_empty(),
    "ciphertext_armored ne doit pas être vide"
  );
  assert!(
    payload
      .ciphertext_armored
      .contains("-----BEGIN PGP MESSAGE-----"),
    "ciphertext doit être un bloc PGP armored"
  );

  // Déchiffrer — GPG utilise self.homedir pour trouver la clef privée.
  let verified = ctx.decrypt_message(&payload).expect("decrypt_message");

  assert_eq!(
    verified.plaintext, plaintext,
    "texte déchiffré doit correspondre au plaintext original"
  );
  assert!(
    !verified.signer_fp.is_empty(),
    "signer_fp ne doit pas être vide"
  );
  // GPG retourne le fingerprint de la sous-clef de signature (peut différer de fp).
  // On vérifie simplement que c'est du hex 40 chars.
  assert_eq!(
    verified.signer_fp.len(),
    40,
    "signer_fp doit être un fingerprint 40 hex, obtenu : {}",
    verified.signer_fp
  );
  assert!(
    verified.signer_fp.chars().all(|c| c.is_ascii_hexdigit()),
    "signer_fp doit être du hex ASCII, obtenu : {}",
    verified.signer_fp
  );
}

// ---------------------------------------------------------------------------
// 3. decrypt_message() échoue si le payload est corrompu
// ---------------------------------------------------------------------------

#[test]
#[ignore = "nécessite la création d'une clef GPG (~5 s) — exécuter avec --ignored"]
fn decrypt_fails_on_corrupted_payload() {
  let (_tmp, homedir) = common::setup_test_gnupghome();
  let fp = generate_test_key(&homedir, "Carol Test", "carol@test.local");

  let ctx = ChatCryptoCtx {
    homedir: homedir.clone(),
    local_fp: fp.clone(),
  };

  // Payload délibérément corrompu.
  let bad_payload = ChatPayload {
    ciphertext_armored: "-----BEGIN PGP MESSAGE-----\ncorrupted garbage\n-----END PGP MESSAGE-----"
      .to_string(),
    signature_armored: String::new(),
  };

  let result = ctx.decrypt_message(&bad_payload);
  assert!(
    result.is_err(),
    "decrypt_message doit échouer avec un payload corrompu"
  );
}

// ---------------------------------------------------------------------------
// 4. decrypt_message() échoue si on n'a pas la clef privée du destinataire
// ---------------------------------------------------------------------------

#[test]
#[ignore = "nécessite la création de deux clefs GPG (~10 s) — exécuter avec --ignored"]
fn decrypt_fails_without_recipient_private_key() {
  // Homedir Alice : possède la clef privée alice + clef publique bob.
  let (_tmp_alice, homedir_alice) = common::setup_test_gnupghome();
  // Homedir Eve (tierce) : n'a que la clef publique d'alice — pas de clef privée.
  let (_tmp_eve, homedir_eve) = common::setup_test_gnupghome();

  let fp_alice = generate_test_key(&homedir_alice, "Alice Sender", "alice.sender@test.local");

  // Exporter la clef publique d'Alice et l'importer chez Eve.
  let alice_pub = std::process::Command::new("gpg")
    .args([
      "--homedir",
      &homedir_alice,
      "--armor",
      "--export",
      &fp_alice,
    ])
    .output()
    .expect("gpg export alice");
  common::import_armored(&homedir_eve, &String::from_utf8_lossy(&alice_pub.stdout));

  // Alice construit son contexte et chiffre un message pour elle-même.
  let ctx_alice = ChatCryptoCtx {
    homedir: homedir_alice.clone(),
    local_fp: fp_alice.clone(),
  };

  let payload = ctx_alice
    .encrypt_for_room(
      "secret pour alice seulement",
      std::slice::from_ref(&fp_alice),
    )
    .expect("encrypt");

  // Eve essaie de déchiffrer le message d'Alice sans avoir sa clef privée.
  // Son homedir ne contient que la clef publique d'Alice.
  let ctx_eve = ChatCryptoCtx {
    homedir: homedir_eve.clone(),
    // Eve se prétend Alice mais n'a pas la clef privée.
    local_fp: fp_alice.clone(),
  };

  let result_eve = ctx_eve.decrypt_message(&payload);
  assert!(
    result_eve.is_err(),
    "decrypt_message doit échouer quand la clef privée est absente du keyring"
  );
}
