use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use sequoia_openpgp::{cert::CertParser, parse::Parse, policy::StandardPolicy, Cert};

use super::types::CardInfo;

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

pub fn move_key_to_card(fingerprint: &str) -> Result<()> {
  super::keyring::validate_fp(fingerprint)?;
  let pub_bytes = Command::new("gpg")
    .args(["--export", fingerprint])
    .output()
    .context("failed to export key for inspection")?
    .stdout;

  let cert = CertParser::from_bytes(&pub_bytes)
    .context("failed to parse key")?
    .find_map(std::result::Result::ok)
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
    .is_some_and(|f| f.for_signing());
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
