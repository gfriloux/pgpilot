use std::collections::HashSet;
use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use sequoia_openpgp::{
  cert::CertParser,
  parse::Parse,
  policy::{NullPolicy, StandardPolicy},
  Cert,
};

use super::card::card_status;
use super::types::{format_date, KeyExpiry, KeyInfo, SubkeyInfo};

fn expiry_to_str(expiry: &KeyExpiry) -> &'static str {
  match expiry {
    KeyExpiry::OneYear => "1y",
    KeyExpiry::TwoYears => "2y",
    KeyExpiry::FiveYears => "5y",
  }
}

fn all_public_fingerprints() -> Result<HashSet<String>> {
  let output = Command::new("gpg")
    .args(["--list-keys", "--with-colons"])
    .output()
    .context("failed to list keys")?;

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

pub fn create_key(
  name: &str,
  email: &str,
  subkey_expiry: &KeyExpiry,
  include_auth: bool,
) -> Result<()> {
  let user_id = format!("{name} <{email}>");
  let sub_expire = expiry_to_str(subkey_expiry);

  let fps_before = all_public_fingerprints()?;

  let status = Command::new("gpg")
    .args([
      "--batch",
      "--quick-gen-key",
      &user_id,
      "ed25519",
      "cert",
      "0",
    ])
    .status()
    .context("failed to run gpg --quick-gen-key")?;

  if !status.success() {
    return Err(anyhow::anyhow!("La génération de la clef maître a échoué"));
  }

  let fp = all_public_fingerprints()?
    .difference(&fps_before)
    .next()
    .cloned()
    .ok_or_else(|| anyhow::anyhow!("Fingerprint de la nouvelle clef introuvable"))?;

  for (algo, usage) in [("ed25519", "sign"), ("cv25519", "encr")] {
    let status = Command::new("gpg")
      .args(["--batch", "--quick-add-key", &fp, algo, usage, sub_expire])
      .status()
      .context("failed to run gpg --quick-add-key")?;
    if !status.success() {
      return Err(anyhow::anyhow!("L'ajout de la sous-clef {usage} a échoué"));
    }
  }

  if include_auth {
    let status = Command::new("gpg")
      .args([
        "--batch",
        "--quick-add-key",
        &fp,
        "ed25519",
        "auth",
        sub_expire,
      ])
      .status()
      .context("failed to run gpg --quick-add-key (auth)")?;
    if !status.success() {
      return Err(anyhow::anyhow!(
        "L'ajout de la sous-clef d'authentification a échoué"
      ));
    }
  }

  Ok(())
}

pub fn export_public_key_armored(fingerprint: &str) -> Result<String> {
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
  String::from_utf8(output.stdout).context("invalid UTF-8 in key")
}

pub fn upload_public_key(fingerprint: &str) -> Result<String> {
  let armored = export_public_key_armored(fingerprint)?;
  let resp = ureq::post("https://paste.rs/")
    .set("Content-Type", "text/plain")
    .set("User-Agent", "pgpilot/1.0")
    .send_string(&armored)
    .map_err(|e| match e {
      ureq::Error::Status(code, _) => anyhow::anyhow!("paste.rs a refusé la requête (HTTP {code})"),
      other => anyhow::anyhow!("Connexion à paste.rs impossible : {other}"),
    })?;
  let url = resp
    .into_string()
    .context("impossible de lire la réponse")?
    .trim()
    .to_string();
  if !url.starts_with("http") {
    return Err(anyhow::anyhow!("Réponse inattendue de paste.rs : {url}"));
  }
  Ok(url)
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

fn export_secret_key(fingerprint: &str, path: &std::path::Path) -> Result<()> {
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

pub fn backup_key(
  fingerprint: &str,
  dir: &std::path::Path,
  short_id: &str,
) -> Result<(String, Option<String>)> {
  let key_filename = format!("{short_id}_secret.asc");
  export_secret_key(fingerprint, &dir.join(&key_filename))?;

  let gnupg_dir = super::gnupg_dir();
  let rev_src = format!(
    "{gnupg_dir}/openpgp-revocs.d/{}.rev",
    fingerprint.to_uppercase()
  );

  let rev_filename = if std::path::Path::new(&rev_src).exists() {
    let name = format!("{short_id}_revocation.rev");
    std::fs::copy(&rev_src, dir.join(&name)).context("failed to copy revocation certificate")?;
    Some(name)
  } else {
    None
  };

  Ok((key_filename, rev_filename))
}

pub fn check_keyserver(fingerprint: &str) -> Result<(String, bool)> {
  let url = format!(
    "https://keys.openpgp.org/vks/v1/by-fingerprint/{}",
    fingerprint.to_uppercase()
  );
  match ureq::get(&url).call() {
    Ok(_) => Ok((fingerprint.to_string(), true)),
    Err(ureq::Error::Status(404, _)) => Ok((fingerprint.to_string(), false)),
    Err(e) => Err(anyhow::anyhow!(
      "Impossible de joindre keys.openpgp.org : {e}"
    )),
  }
}

pub fn publish_key(fingerprint: &str, keyserver_url: &str) -> Result<String> {
  let status = Command::new("gpg")
    .args(["--keyserver", keyserver_url, "--send-keys", fingerprint])
    .status()
    .context("failed to run gpg --send-keys")?;

  if !status.success() {
    return Err(anyhow::anyhow!("L'envoi de la clef a échoué"));
  }
  Ok(keyserver_url.to_string())
}

fn subkey_position(master_fp: &str, subkey_fp: &str) -> Result<usize> {
  let output = Command::new("gpg")
    .args(["--list-keys", "--with-colons", master_fp])
    .output()
    .context("failed to list key")?;

  let text = String::from_utf8(output.stdout)?;
  let mut pos: usize = 0;
  let mut last_was_key = false;

  for line in text.lines() {
    let fields: Vec<&str> = line.split(':').collect();
    match fields.first().copied().unwrap_or("") {
      "pub" => {
        pos = 0;
        last_was_key = true;
      }
      "sub" | "ssb" => {
        pos += 1;
        last_was_key = true;
      }
      "fpr" => {
        if last_was_key {
          last_was_key = false;
          if fields.len() > 9 && fields[9].to_uppercase() == subkey_fp.to_uppercase() {
            return Ok(pos);
          }
        }
      }
      _ => {
        last_was_key = false;
      }
    }
  }
  Err(anyhow::anyhow!("Position de la sous-clef introuvable"))
}

fn revoke_subkey_at_pos(master_fp: &str, pos: usize) -> Result<()> {
  let cmds = format!("key {pos}\nrevkey\n2\n\ny\nsave\n");

  let mut child = Command::new("gpg")
    .args(["--no-tty", "--command-fd", "0", "--edit-key", master_fp])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .context("failed to spawn gpg --edit-key")?;

  {
    let stdin = child.stdin.as_mut().context("failed to open gpg stdin")?;
    stdin
      .write_all(cmds.as_bytes())
      .context("failed to write to gpg stdin")?;
  }

  let output = child.wait_with_output().context("failed to wait for gpg")?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(anyhow::anyhow!("Révocation échouée : {stderr}"));
  }
  Ok(())
}

pub fn rotate_subkey(
  master_fp: &str,
  old_subkey_fp: &str,
  algo: &str,
  usage: &str,
  expiry: &KeyExpiry,
) -> Result<()> {
  add_subkey(master_fp, algo, usage, expiry)?;
  let pos = subkey_position(master_fp, old_subkey_fp)?;
  revoke_subkey_at_pos(master_fp, pos)
}

pub fn add_subkey(master_fp: &str, algo: &str, usage: &str, expiry: &KeyExpiry) -> Result<()> {
  let expire = expiry_to_str(expiry);
  let status = Command::new("gpg")
    .args(["--batch", "--quick-add-key", master_fp, algo, usage, expire])
    .status()
    .context("failed to run gpg --quick-add-key")?;

  if !status.success() {
    return Err(anyhow::anyhow!("L'ajout de la sous-clef a échoué"));
  }
  Ok(())
}

pub fn renew_subkey(master_fp: &str, subkey_fp: &str, expiry: &KeyExpiry) -> Result<()> {
  let expire = expiry_to_str(expiry);
  let status = Command::new("gpg")
    .args([
      "--batch",
      "--quick-set-expire",
      master_fp,
      expire,
      subkey_fp,
    ])
    .status()
    .context("failed to run gpg --quick-set-expire")?;

  if !status.success() {
    return Err(anyhow::anyhow!(
      "Le renouvellement de la sous-clef a échoué"
    ));
  }
  Ok(())
}

pub fn delete_key(fingerprint: &str, has_secret: bool) -> Result<()> {
  let cmd = if has_secret {
    "--delete-secret-and-public-keys"
  } else {
    "--delete-keys"
  };

  let status = Command::new("gpg")
    .args(["--batch", "--yes", cmd, fingerprint])
    .status()
    .context("failed to run gpg delete")?;

  if !status.success() {
    return Err(anyhow::anyhow!("La suppression de la clef a échoué"));
  }
  Ok(())
}

pub fn import_key_from_text(content: &str) -> Result<()> {
  if !content.contains("-----BEGIN PGP") {
    return Err(anyhow::anyhow!(
      "Le contenu ne ressemble pas à une clef PGP (en-tête -----BEGIN PGP introuvable)"
    ));
  }
  let mut child = Command::new("gpg")
    .args(["--import"])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .context("failed to spawn gpg --import")?;
  {
    let stdin = child.stdin.as_mut().context("failed to open stdin")?;
    stdin
      .write_all(content.as_bytes())
      .context("failed to write to stdin")?;
  }
  let output = child.wait_with_output().context("failed to wait for gpg")?;
  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(anyhow::anyhow!("{stderr}"));
  }
  Ok(())
}

pub fn import_key_from_url(url: &str) -> Result<()> {
  let resp = ureq::get(url)
    .set("User-Agent", "pgpilot/1.0")
    .call()
    .map_err(|e| anyhow::anyhow!("Impossible de charger l'URL : {e}"))?;
  let content = resp
    .into_string()
    .context("impossible de lire le contenu")?;
  import_key_from_text(&content)
}

pub fn import_key_from_keyserver(query: &str, keyserver_url: &str) -> Result<()> {
  if query.contains('@') {
    let encoded = query.replace('@', "%40");
    let url = format!("https://{keyserver_url}/pks/lookup?op=get&search={encoded}");
    let resp = ureq::get(&url)
      .set("User-Agent", "pgpilot/1.0")
      .call()
      .map_err(|e| match e {
        ureq::Error::Status(404, _) => {
          anyhow::anyhow!("Aucune clef trouvée pour '{query}'")
        }
        ureq::Error::Status(code, _) => anyhow::anyhow!("Keyserver : HTTP {code}"),
        other => anyhow::anyhow!("Impossible de joindre le keyserver : {other}"),
      })?;
    let content = resp
      .into_string()
      .context("impossible de lire la réponse")?;
    import_key_from_text(&content)
  } else {
    let output = Command::new("gpg")
      .args(["--keyserver", keyserver_url, "--recv-keys", query])
      .output()
      .context("failed to run gpg --recv-keys")?;
    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      return Err(anyhow::anyhow!("{stderr}"));
    }
    Ok(())
  }
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
        .filter_map(std::clone::Clone::clone)
        .collect();
      (fps, Some(c.serial.clone()))
    }
    None => (HashSet::new(), None),
  };

  let keys = CertParser::from_bytes(&pub_bytes)
    .context("failed to parse keyring")?
    .filter_map(std::result::Result::ok)
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

  let expires = cert
    .with_policy(&StandardPolicy::new(), None)
    .ok()
    .and_then(|vc| vc.primary_key().key_expiration_time())
    .map(format_date);

  let subkeys = cert
    .with_policy(&NullPolicy::new(), None)
    .map(|vc| {
      vc.keys()
        .subkeys()
        .map(|ka| {
          let k = ka.key();
          let sfp = k.fingerprint().to_hex();
          let short_id = sfp[sfp.len().saturating_sub(16)..].to_string();
          let usage = ka
            .key_flags()
            .map(|f| {
              let mut u = String::new();
              if f.for_signing() {
                u.push('S');
              }
              if f.for_transport_encryption() || f.for_storage_encryption() {
                u.push('E');
              }
              if f.for_authentication() {
                u.push('A');
              }
              u
            })
            .unwrap_or_default();
          SubkeyInfo {
            fingerprint: sfp,
            short_id,
            algo: format!("{}", k.pk_algo()),
            usage,
            expires: ka.key_expiration_time().map(format_date),
          }
        })
        .collect()
    })
    .unwrap_or_default();

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
    subkeys,
  }
}
