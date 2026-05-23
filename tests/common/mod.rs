#![allow(dead_code)]
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use tempfile::TempDir;

/// Mutex partagé pour sérialiser les tests qui modifient des variables
/// d'environnement (ex. GNUPGHOME). Chaque fichier de test est un binaire
/// séparé, donc ce lock protège uniquement les threads au sein du même binaire.
pub static ENV_LOCK: Mutex<()> = Mutex::new(());

pub fn setup_test_gnupghome() -> (TempDir, String) {
  let dir = TempDir::new().expect("tempdir");
  use std::os::unix::fs::PermissionsExt;
  std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).expect("chmod 700");
  let path = dir.path().to_str().unwrap().to_string();
  (dir, path)
}

pub fn import_armored(homedir: &str, armored: &str) -> String {
  let mut child = Command::new("gpg")
    .args(["--homedir", homedir, "--batch", "--import"])
    .stdin(Stdio::piped())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .expect("gpg spawn");
  child
    .stdin
    .as_mut()
    .unwrap()
    .write_all(armored.as_bytes())
    .unwrap();
  child.wait().expect("gpg wait");
  list_fingerprints(homedir)
    .into_iter()
    .next()
    .expect("imported key fp")
}

pub fn list_fingerprints(homedir: &str) -> Vec<String> {
  let output = Command::new("gpg")
    .args(["--homedir", homedir, "--list-keys", "--with-colons"])
    .output()
    .expect("gpg list-keys");
  String::from_utf8_lossy(&output.stdout)
    .lines()
    .filter(|l| l.starts_with("fpr"))
    .map(|l| l.split(':').nth(9).unwrap_or("").to_string())
    .filter(|fp| !fp.is_empty())
    .collect()
}
