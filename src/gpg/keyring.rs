use std::collections::HashSet;
use std::process::Command;

use anyhow::{Context, Result};
use sequoia_openpgp::{cert::CertParser, parse::Parse, policy::StandardPolicy, Cert};

use super::card::card_status;
use super::types::{format_date, KeyAlgo, KeyExpiry, KeyInfo};

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

pub fn import_key(path: &std::path::Path) -> Result<()> {
  let output = Command::new("gpg")
    .args(["--import", &path.to_string_lossy()])
    .output()
    .context("failed to run gpg --import")?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(anyhow::anyhow!("{stderr}"));
  }
  Ok(())
}

pub fn list_keys() -> Result<(Vec<KeyInfo>, bool)> {
  let pub_bytes = Command::new("gpg")
    .args(["--export"])
    .output()
    .context("failed to run gpg --export")?
    .stdout;

  let secret_fps = secret_key_fingerprints()?;
  let card = card_status();
  let card_connected = card.is_some();

  let (card_fps, card_serial) = match &card {
    Some(c) => {
      let fps: HashSet<String> = [&c.sig_fp, &c.enc_fp, &c.auth_fp]
        .into_iter()
        .filter_map(|fp| fp.clone())
        .collect();
      (fps, Some(c.serial.clone()))
    }
    None => (HashSet::new(), None),
  };

  let keys = CertParser::from_bytes(&pub_bytes)
    .context("failed to parse keyring")?
    .filter_map(|r| r.ok())
    .map(|cert| {
      let fp = cert.fingerprint().to_hex();
      let has_secret = secret_fps.contains(&fp);
      let on_card = cert
        .keys()
        .any(|ka| card_fps.contains(&ka.key().fingerprint().to_hex()));
      let serial = if on_card { card_serial.clone() } else { None };
      cert_to_key_info(cert, has_secret, on_card, serial)
    })
    .collect();

  Ok((keys, card_connected))
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

fn cert_to_key_info(
  cert: Cert,
  has_secret: bool,
  on_card: bool,
  card_serial: Option<String>,
) -> KeyInfo {
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
  let created = format_date(primary_key.creation_time());

  let policy = StandardPolicy::new();
  let expires = cert
    .with_policy(&policy, None)
    .ok()
    .and_then(|vc| vc.primary_key().key_expiration_time())
    .map(format_date);

  KeyInfo {
    fingerprint: fp,
    short_id,
    name,
    email,
    algo,
    created,
    expires,
    has_secret,
    on_card,
    card_serial,
  }
}
