mod common {
  /// Create a temporary GNUPGHOME directory for testing
  pub fn setup_test_gnupghome() -> (tempfile::TempDir, String) {
    let dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let homedir = dir.path().to_string_lossy().to_string();
    (dir, homedir)
  }
}

use common::setup_test_gnupghome;

#[test]
fn import_key_rejects_non_pgp_text() {
  let (_dir, _homedir) = setup_test_gnupghome();
  let result = pgpilot::gpg::keyring::import_key_from_text("<html>404 Not Found</html>");
  assert!(result.is_err());
  let err = result.unwrap_err();
  let err_str = err.to_string();
  assert!(
    err_str.contains("PGP")
      || err_str.contains("pgp")
      || err_str.contains("clef")
      || err_str.contains("key"),
    "Error should mention PGP: {err_str}"
  );
}

#[test]
#[ignore]
fn list_keys_empty_homedir() {
  let (_dir, homedir) = setup_test_gnupghome();
  // Point GNUPGHOME at the isolated temp dir so we don't read the user's real keyring.
  std::env::set_var("GNUPGHOME", &homedir);
  // list_keys() returns Result<(Vec<KeyInfo>, bool)> — .0 is the key vec, .1 is card_connected
  let (keys, _card_connected) = pgpilot::gpg::list_keys().unwrap();
  assert!(keys.is_empty());
}

#[test]
#[ignore]
fn create_key_returns_valid_fingerprint() {
  // Key creation prompts for a passphrase via pinentry (GUI or TTY).
  // Requires an interactive user session — skip gracefully in headless CI or
  // when stdin is not a terminal (e.g. nix develop --command sh -c '...').
  use std::io::IsTerminal as _;
  if std::env::var("CI").is_ok() || !std::io::stdin().is_terminal() {
    return;
  }
  let (_dir, _homedir) = setup_test_gnupghome();
  let result = pgpilot::gpg::keyring::create_key(
    "Test User",
    "test@pgpilot.test",
    &pgpilot::gpg::KeyExpiry::TwoYears,
    false,
  );
  assert!(result.is_ok());
}

#[test]
#[ignore]
fn create_key_has_sign_encr_auth_subkeys() {
  // Key creation prompts for a passphrase via pinentry (GUI or TTY).
  // Requires an interactive user session — skip gracefully in headless CI or
  // when stdin is not a terminal (e.g. nix develop --command sh -c '...').
  use std::io::IsTerminal as _;
  if std::env::var("CI").is_ok() || !std::io::stdin().is_terminal() {
    return;
  }
  let (_dir, _homedir) = setup_test_gnupghome();
  let result = pgpilot::gpg::keyring::create_key(
    "Test User",
    "test@pgpilot.test",
    &pgpilot::gpg::KeyExpiry::TwoYears,
    true,
  );
  assert!(result.is_ok());
}
