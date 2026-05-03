use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use sequoia_openpgp::{
  cert::CertParser,
  parse::Parse,
  policy::{NullPolicy, StandardPolicy},
  Cert,
};

use super::card::card_status;
use super::types::{format_date, KeyExpiry, KeyInfo, SubkeyInfo, TrustLevel};

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
  key_id: &str,
) -> Result<(String, Option<String>)> {
  let key_filename = format!("{key_id}_secret.asc");
  export_secret_key(fingerprint, &dir.join(&key_filename))?;

  let gnupg_dir = super::gnupg_dir();
  let rev_src = format!(
    "{gnupg_dir}/openpgp-revocs.d/{}.rev",
    fingerprint.to_uppercase()
  );

  let rev_filename = if std::path::Path::new(&rev_src).exists() {
    let name = format!("{key_id}_revocation.rev");
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

fn key_ownertrusts() -> Result<std::collections::HashMap<String, TrustLevel>> {
  let gnupg = super::gnupg_dir();
  let output = Command::new("gpg")
    .arg("--homedir")
    .arg(&gnupg)
    .args(["--list-keys", "--with-colons"])
    .output()
    .context("failed to run gpg --list-keys")?;

  let mut trusts = std::collections::HashMap::new();
  let mut pending: Option<TrustLevel> = None;

  for line in String::from_utf8(output.stdout)?.lines() {
    let fields: Vec<&str> = line.split(':').collect();
    match fields.first().copied() {
      Some("pub") if fields.len() > 8 => {
        pending = Some(TrustLevel::from_char(
          fields[8].chars().next().unwrap_or('-'),
        ));
      }
      Some("fpr") if fields.len() > 9 => {
        if let Some(trust) = pending.take() {
          trusts.insert(fields[9].to_string(), trust);
        }
      }
      Some("sub") => pending = None,
      _ => {}
    }
  }

  Ok(trusts)
}

pub fn list_keys() -> Result<(Vec<KeyInfo>, bool)> {
  let pub_bytes = Command::new("gpg")
    .args(["--export"])
    .output()
    .context("failed to run gpg --export")?
    .stdout;

  let secret_fps = secret_key_fingerprints()?;
  let trusts = key_ownertrusts().unwrap_or_default();
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
      let trust = trusts.get(&fp).cloned().unwrap_or_default();
      let on_card = cert
        .keys()
        .any(|ka| card_fps.contains(&ka.key().fingerprint().to_hex()));
      let serial = if on_card { card_serial.clone() } else { None };
      cert_to_key_info(cert, has_secret, on_card, serial, trust)
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
  trust: TrustLevel,
) -> KeyInfo {
  let fp = cert.fingerprint().to_hex();
  let key_id = fp[fp.len().saturating_sub(16)..].to_string();

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
          let key_id = sfp[sfp.len().saturating_sub(16)..].to_string();
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
            key_id,
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
    key_id,
    name,
    email,
    algo,
    created,
    expires,
    has_secret,
    on_card,
    card_serial,
    subkeys,
    trust,
  }
}

pub fn set_key_trust(fingerprint: &str, trust: &TrustLevel) -> Result<()> {
  let level: u8 = match trust {
    TrustLevel::Undefined => 2,
    TrustLevel::Marginal => 4,
    TrustLevel::Full => 5,
    TrustLevel::Ultimate => 6,
  };
  let gnupg = super::gnupg_dir();
  let input = format!("{fingerprint}:{level}:\n");

  let mut child = Command::new("gpg")
    .arg("--homedir")
    .arg(&gnupg)
    .arg("--import-ownertrust")
    .stdin(Stdio::piped())
    .spawn()
    .context("failed to spawn gpg --import-ownertrust")?;

  child
    .stdin
    .take()
    .context("failed to open stdin")?
    .write_all(input.as_bytes())
    .context("failed to write to gpg stdin")?;

  let status = child.wait().context("failed to wait for gpg")?;
  if !status.success() {
    return Err(anyhow::anyhow!(
      "Impossible de modifier le niveau de confiance"
    ));
  }
  Ok(())
}

pub fn encrypt_files(
  files: &[PathBuf],
  recipients: &[String],
  armor: bool,
  force_trust: bool,
) -> Result<Vec<String>> {
  let gnupg = super::gnupg_dir();
  let ext = if armor { "asc" } else { "gpg" };
  let mut results = Vec::new();

  for file in files {
    let output = PathBuf::from(format!("{}.{}", file.display(), ext));

    let mut cmd = Command::new("gpg");
    cmd.arg("--homedir").arg(&gnupg);
    cmd.arg("--batch");
    cmd.arg("--yes");
    if force_trust {
      cmd.arg("--trust-model").arg("always");
    }
    if armor {
      cmd.arg("--armor");
    }
    cmd.arg("--encrypt");
    for fp in recipients {
      cmd.arg("--recipient").arg(fp);
    }
    cmd.arg("--output").arg(&output);
    cmd.arg(file);

    let out = cmd.output().context("failed to run gpg --encrypt")?;
    if !out.status.success() {
      let stderr = String::from_utf8_lossy(&out.stderr);
      return Err(anyhow::anyhow!(
        "Échec du chiffrement de {} : {}",
        file.file_name().unwrap_or_default().to_string_lossy(),
        stderr.trim()
      ));
    }

    results.push(
      output
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string(),
    );
  }

  Ok(results)
}

pub fn inspect_decrypt(file: &std::path::Path) -> Result<super::types::DecryptStatus> {
  use super::types::DecryptStatus;
  let gnupg = super::gnupg_dir();
  let out = Command::new("gpg")
    .args([
      "--homedir",
      &gnupg,
      "--batch",
      "--status-fd",
      "1",
      "--list-only",
      "--decrypt",
    ])
    .arg(file)
    .output()
    .context("failed to run gpg --list-only --decrypt")?;

  let stdout = String::from_utf8_lossy(&out.stdout);
  let stderr = String::from_utf8_lossy(&out.stderr);

  if stdout.contains("[GNUPG:] BEGIN_DECRYPTION") {
    return Ok(DecryptStatus::CanDecrypt);
  }

  let enc_to_count = stdout
    .lines()
    .filter(|l| l.contains("[GNUPG:] ENC_TO"))
    .count();
  let no_seckey_count = stdout
    .lines()
    .filter(|l| l.contains("[GNUPG:] NO_SECKEY"))
    .count();

  if enc_to_count > 0 && no_seckey_count >= enc_to_count {
    return Ok(DecryptStatus::NoKey);
  }

  if stderr.contains("No secret key") || stderr.contains("no secret key") {
    return Ok(DecryptStatus::NoKey);
  }

  if out.status.success() {
    Ok(DecryptStatus::CanDecrypt)
  } else {
    Ok(DecryptStatus::Unknown)
  }
}

pub fn decrypt_files(files: &[PathBuf]) -> Result<Vec<String>> {
  let gnupg = super::gnupg_dir();
  let mut results = Vec::new();

  for file in files {
    let stem = file
      .file_stem()
      .unwrap_or_default()
      .to_string_lossy()
      .to_string();
    let ext = file
      .extension()
      .unwrap_or_default()
      .to_string_lossy()
      .to_lowercase();

    let base_name = if ext == "gpg" || ext == "asc" {
      stem
    } else {
      format!(
        "{}.decrypted",
        file.file_name().unwrap_or_default().to_string_lossy()
      )
    };

    let dir = file.parent().unwrap_or_else(|| std::path::Path::new("."));

    let mut candidate = dir.join(&base_name);
    let mut counter = 1u32;
    while candidate.exists() {
      candidate = dir.join(format!("{}_{}", base_name, counter));
      counter += 1;
    }

    let mut cmd = Command::new("gpg");
    cmd.arg("--homedir").arg(&gnupg);
    cmd.arg("--batch");
    cmd.arg("--yes");
    cmd.arg("--output").arg(&candidate);
    cmd.arg("--decrypt");
    cmd.arg(file);

    let out = cmd.output().context("failed to run gpg --decrypt")?;
    if !out.status.success() {
      let stderr = String::from_utf8_lossy(&out.stderr);
      return Err(anyhow::anyhow!(
        "Échec du déchiffrement de {} : {}",
        file.file_name().unwrap_or_default().to_string_lossy(),
        stderr.trim()
      ));
    }

    results.push(
      candidate
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string(),
    );
  }

  Ok(results)
}
