pub mod card;
pub mod health;
pub mod keyring;
pub mod types;

pub(crate) fn gnupg_dir() -> String {
  std::env::var("GNUPGHOME")
    .unwrap_or_else(|_| format!("{}/.gnupg", std::env::var("HOME").unwrap_or_default()))
}

pub(crate) fn display_path(path: &std::path::Path) -> String {
  let home = std::env::var("HOME").unwrap_or_default();
  if home.is_empty() {
    return path.display().to_string();
  }
  match path.strip_prefix(&home) {
    Ok(rel) => format!("~/{}", rel.display()),
    Err(_) => path.display().to_string(),
  }
}

pub(crate) fn sanitize_gpg_stderr(stderr: &str) -> String {
  stderr
    .lines()
    .filter(|l| !l.starts_with("[GNUPG:]"))
    .filter(|l| !l.is_empty())
    .collect::<Vec<_>>()
    .join("\n")
}

pub use card::move_key_to_card;
pub use health::{run_all_checks, CheckStatus, HealthCheck};
pub use keyring::{
  add_subkey, backup_key, check_keyserver, create_key, decrypt_files, delete_key, encrypt_files,
  export_public_key, export_public_key_armored, import_key, import_key_from_keyserver,
  import_key_from_text, import_key_from_url, inspect_decrypt, list_keys, publish_key, renew_subkey,
  rotate_subkey, set_key_trust, sign_file, upload_public_key, verify_signature,
};
pub use types::{
  DecryptStatus, KeyExpiry, KeyInfo, Keyserver, SubkeyType, TrustLevel, VerifyOutcome, VerifyResult,
};
