use std::collections::HashSet;
use std::process::Command;

use anyhow::{Context, Result};
use sequoia_openpgp::{cert::CertParser, parse::Parse, policy::StandardPolicy, Cert};

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

  Ok(String::from_utf8(output.stdout)?
    .lines()
    .filter_map(|line| {
      let fields: Vec<&str> = line.split(':').collect();
      if fields.len() > 9 && fields[0] == "fpr" {
        Some(fields[9].to_string())
      } else {
        None
      }
    })
    .collect())
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
