use std::collections::HashSet;
use std::process::Command;

use anyhow::{Context, Result};
use sequoia_openpgp::{cert::CertParser, parse::Parse, policy::StandardPolicy, Cert};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum KeyAlgo {
  #[default]
  Ed25519,
  Rsa4096,
}

impl std::fmt::Display for KeyAlgo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      KeyAlgo::Ed25519 => write!(f, "Ed25519"),
      KeyAlgo::Rsa4096 => write!(f, "RSA 4096"),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum KeyExpiry {
  #[default]
  Never,
  OneYear,
  TwoYears,
  FiveYears,
}

impl std::fmt::Display for KeyExpiry {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      KeyExpiry::Never => write!(f, "Aucune expiration"),
      KeyExpiry::OneYear => write!(f, "1 an"),
      KeyExpiry::TwoYears => write!(f, "2 ans"),
      KeyExpiry::FiveYears => write!(f, "5 ans"),
    }
  }
}

pub fn create_key(name: &str, email: &str, algo: &KeyAlgo, expiry: &KeyExpiry) -> Result<()> {
  let user_id = format!("{name} <{email}>");
  let algo_str = match algo {
    KeyAlgo::Ed25519 => "ed25519",
    KeyAlgo::Rsa4096 => "rsa4096",
  };
  let expire_str = match expiry {
    KeyExpiry::Never => "0",
    KeyExpiry::OneYear => "1y",
    KeyExpiry::TwoYears => "2y",
    KeyExpiry::FiveYears => "5y",
  };

  let status = Command::new("gpg")
    .args([
      "--batch",
      "--quick-gen-key",
      &user_id,
      algo_str,
      "-",
      expire_str,
    ])
    .status()
    .context("failed to run gpg --quick-gen-key")?;

  if !status.success() {
    return Err(anyhow::anyhow!("La génération de clef a échoué"));
  }
  Ok(())
}

#[derive(Debug, Clone)]
pub struct KeyInfo {
  pub fingerprint: String,
  pub short_id: String,
  pub name: String,
  pub email: String,
  pub algo: String,
  pub created: String,
  pub expires: Option<String>,
  pub has_secret: bool,
}

pub fn export_public_key(fingerprint: &str, path: &std::path::Path) -> Result<()> {
  let output = Command::new("gpg")
    .args(["--export", "--armor", fingerprint])
    .output()
    .context("failed to run gpg --export")?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(anyhow::anyhow!("{stderr}"));
  }
  if output.stdout.is_empty() {
    return Err(anyhow::anyhow!("Aucune clef trouvée pour ce fingerprint"));
  }

  std::fs::write(path, &output.stdout).context("failed to write key file")
}

pub fn export_secret_key(fingerprint: &str, path: &std::path::Path) -> Result<()> {
  let output = Command::new("gpg")
    .args(["--export-secret-keys", "--armor", fingerprint])
    .output()
    .context("failed to run gpg --export-secret-keys")?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(anyhow::anyhow!("{stderr}"));
  }
  if output.stdout.is_empty() {
    return Err(anyhow::anyhow!(
      "Aucune clef secrète trouvée pour ce fingerprint"
    ));
  }

  std::fs::write(path, &output.stdout).context("failed to write key file")
}

pub fn list_keys() -> Result<Vec<KeyInfo>> {
  let pub_bytes = Command::new("gpg")
    .args(["--export"])
    .output()
    .context("failed to run gpg --export")?
    .stdout;

  let secret_fps = secret_key_fingerprints()?;

  let keys = CertParser::from_bytes(&pub_bytes)
    .context("failed to parse keyring")?
    .filter_map(|r| r.ok())
    .map(|cert| {
      let fp = cert.fingerprint().to_hex();
      let has_secret = secret_fps.contains(&fp);
      cert_to_key_info(cert, has_secret)
    })
    .collect();

  Ok(keys)
}

fn secret_key_fingerprints() -> Result<HashSet<String>> {
  let output = Command::new("gpg")
    .args(["--list-secret-keys", "--with-colons"])
    .output()
    .context("failed to run gpg --list-secret-keys")?;

  Ok(
    String::from_utf8(output.stdout)?
      .lines()
      .filter_map(|line| {
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() > 9 && fields[0] == "fpr" {
          Some(fields[9].to_string())
        } else {
          None
        }
      })
      .collect(),
  )
}

fn cert_to_key_info(cert: Cert, has_secret: bool) -> KeyInfo {
  let fp = cert.fingerprint().to_hex();
  let short_id = fp[fp.len().saturating_sub(8)..].to_string();

  let (name, email) = cert
    .userids()
    .next()
    .map(|ua| {
      let uid = ua.userid();
      let name = uid.name2().ok().flatten().unwrap_or_default().to_string();
      let email = uid.email2().ok().flatten().unwrap_or_default().to_string();
      (name, email)
    })
    .unwrap_or_default();

  let primary_key = cert.primary_key().key();
  let algo = format!("{}", primary_key.pk_algo());
  let created = {
    let dt: chrono::DateTime<chrono::Utc> = primary_key.creation_time().into();
    dt.format("%Y-%m-%d").to_string()
  };

  let policy = StandardPolicy::new();
  let expires = cert
    .with_policy(&policy, None)
    .ok()
    .and_then(|vc| vc.primary_key().key_expiration_time())
    .map(|t| {
      let dt: chrono::DateTime<chrono::Utc> = t.into();
      dt.format("%Y-%m-%d").to_string()
    });

  KeyInfo {
    fingerprint: fp,
    short_id,
    name,
    email,
    algo,
    created,
    expires,
    has_secret,
  }
}
