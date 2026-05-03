pub mod card;
pub mod health;
pub mod keyring;
pub mod types;

pub(crate) fn gnupg_dir() -> anyhow::Result<String> {
  let dir = std::env::var("GNUPGHOME")
    .unwrap_or_else(|_| format!("{}/.gnupg", std::env::var("HOME").unwrap_or_default()));
  if dir.is_empty() || dir == "/.gnupg" {
    anyhow::bail!("GNUPGHOME ou HOME non défini — impossible de déterminer le répertoire GPG");
  }
  Ok(dir)
}

pub(crate) fn gpg_command(homedir: &str) -> std::process::Command {
  let mut cmd = std::process::Command::new("gpg");
  cmd.env_clear();
  for var in &[
    "HOME",
    "GNUPGHOME",
    "LANG",
    "LC_ALL",
    "LC_MESSAGES",
    "PATH",
    "DISPLAY",
    "WAYLAND_DISPLAY",
    "XDG_RUNTIME_DIR",
    "DBUS_SESSION_BUS_ADDRESS",
    "GPG_TTY",
    "PINENTRY_USER_DATA",
  ] {
    if let Ok(val) = std::env::var(var) {
      cmd.env(var, val);
    }
  }
  cmd.args(["--homedir", homedir]);
  cmd
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
