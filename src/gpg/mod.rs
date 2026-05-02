use std::collections::HashSet;
use std::io::Write;
use std::process::{Command, Stdio};

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
pub struct CardInfo {
  pub serial: String,
  pub sig_fp: Option<String>,
  pub enc_fp: Option<String>,
  pub auth_fp: Option<String>,
}

pub fn card_status() -> Option<CardInfo> {
  let output = Command::new("gpg").args(["--card-status"]).output().ok()?;

  if !output.status.success() {
    return None;
  }

  let text = String::from_utf8(output.stdout).ok()?;
  let mut serial = String::new();
  let mut sig_fp = None;
  let mut enc_fp = None;
  let mut auth_fp = None;

  for line in text.lines() {
    if let Some(v) = strip_card_value(line, "Serial number") {
      serial = v.to_string();
    } else if let Some(v) = strip_card_value(line, "Signature key") {
      sig_fp = parse_card_fp(v);
    } else if let Some(v) = strip_card_value(line, "Encryption key") {
      enc_fp = parse_card_fp(v);
    } else if let Some(v) = strip_card_value(line, "Authentication key") {
      auth_fp = parse_card_fp(v);
    }
  }

  if serial.is_empty() {
    return None;
  }
  Some(CardInfo {
    serial,
    sig_fp,
    enc_fp,
    auth_fp,
  })
}

fn strip_card_value<'a>(line: &'a str, field: &str) -> Option<&'a str> {
  let rest = line.strip_prefix(field)?.split_once(':').map(|x| x.1)?;
  Some(rest.trim())
}

fn parse_card_fp(s: &str) -> Option<String> {
  if s == "[none]" || s.is_empty() {
    return None;
  }
  Some(
    s.chars()
      .filter(|c| !c.is_whitespace())
      .collect::<String>()
      .to_uppercase(),
  )
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
  pub on_card: bool,
  pub card_serial: Option<String>,
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

pub fn move_key_to_card(fingerprint: &str) -> Result<()> {
  let pub_bytes = Command::new("gpg")
    .args(["--export", fingerprint])
    .output()
    .context("failed to export key for inspection")?
    .stdout;

  let cert = CertParser::from_bytes(&pub_bytes)
    .context("failed to parse key")?
    .filter_map(|r| r.ok())
    .next()
    .ok_or_else(|| anyhow::anyhow!("Clef introuvable : {fingerprint}"))?;

  let stdin_cmds = build_keytocard_sequence(&cert)?;

  let mut child = Command::new("gpg")
    .args([
      "--no-tty",
      "--yes",
      "--command-fd",
      "0",
      "--edit-key",
      fingerprint,
    ])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .context("failed to spawn gpg --edit-key")?;

  {
    let stdin = child.stdin.as_mut().context("failed to open gpg stdin")?;
    stdin
      .write_all(stdin_cmds.as_bytes())
      .context("failed to write to gpg stdin")?;
  }

  let output = child.wait_with_output().context("failed to wait for gpg")?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(anyhow::anyhow!("Migration échouée : {stderr}"));
  }

  Ok(())
}

fn build_keytocard_sequence(cert: &Cert) -> Result<String> {
  let policy = StandardPolicy::new();
  let vc = cert
    .with_policy(&policy, None)
    .context("la clef ne satisfait pas la politique de sécurité")?;

  let mut cmds = String::new();

  let primary_signs = vc
    .primary_key()
    .key_flags()
    .map(|f| f.for_signing())
    .unwrap_or(false);
  if primary_signs {
    cmds.push_str("keytocard\n1\n");
  }

  for (i, subkey) in vc.keys().subkeys().enumerate() {
    let n = i + 1;
    let slot = subkey.key_flags().and_then(|f| {
      if f.for_transport_encryption() || f.for_storage_encryption() {
        Some(2u8)
      } else if f.for_authentication() {
        Some(3u8)
      } else {
        None
      }
    });
    if let Some(slot) = slot {
      cmds.push_str(&format!("key {n}\nkeytocard\n{slot}\nkey {n}\n"));
    }
  }

  cmds.push_str("save\n");
  Ok(cmds)
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
    on_card,
    card_serial,
  }
}
