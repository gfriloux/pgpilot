use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use sequoia_openpgp::{
  cert::{amalgamation::ValidAmalgamation, CertParser},
  parse::Parse,
  policy::{NullPolicy, StandardPolicy},
  types::RevocationStatus,
  Cert,
};

use super::card::card_status;
use super::types::{format_date, KeyExpiry, KeyInfo, SubkeyInfo, TrustLevel};
use super::{display_path, sanitize_gpg_stderr};

const MAX_RESPONSE_BYTES: u64 = 1 << 20; // 1 MiB

fn safe_get(url: &str) -> Result<String> {
  if !url.starts_with("https://") {
    anyhow::bail!("URL non sécurisée : seul https:// est autorisé");
  }
  let agent = ureq::Agent::config_builder()
    .max_redirects(3)
    .max_redirects_will_error(true)
    .timeout_connect(Some(std::time::Duration::from_secs(10)))
    .timeout_recv_response(Some(std::time::Duration::from_secs(15)))
    .build()
    .new_agent();
  let mut resp = agent
    .get(url)
    .header("User-Agent", "pgpilot/1.0")
    .call()
    .map_err(|e| anyhow::anyhow!("{e}"))?;
  resp
    .body_mut()
    .with_config()
    .limit(MAX_RESPONSE_BYTES)
    .read_to_string()
    .map_err(|e| anyhow::anyhow!("{e}"))
}

fn expiry_to_str(expiry: &KeyExpiry) -> &'static str {
  match expiry {
    KeyExpiry::OneYear => "1y",
    KeyExpiry::TwoYears => "2y",
    KeyExpiry::FiveYears => "5y",
  }
}

pub(super) fn validate_fp(fp: &str) -> Result<()> {
  if fp.len() != 40 || !fp.chars().all(|c| c.is_ascii_hexdigit()) {
    anyhow::bail!("Fingerprint invalide : doit être 40 caractères hexadécimaux");
  }
  Ok(())
}

fn validate_keyserver_query(query: &str) -> Result<()> {
  if query.len() == 40 && query.chars().all(|c| c.is_ascii_hexdigit()) {
    return Ok(());
  }
  if query.len() == 16 && query.chars().all(|c| c.is_ascii_hexdigit()) {
    return Ok(());
  }
  if query.contains('@')
    && query
      .chars()
      .all(|c| c.is_alphanumeric() || "@._+-".contains(c))
  {
    return Ok(());
  }
  anyhow::bail!(
    "Requête invalide : entrez un fingerprint 40 hex, un ID long 16 hex, ou une adresse email"
  );
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
  validate_fp(fingerprint)?;
  let output = Command::new("gpg")
    .args(["--export", "--armor", fingerprint])
    .output()
    .context("failed to run gpg --export")?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(anyhow::anyhow!("{}", sanitize_gpg_stderr(&stderr)));
  }
  if output.stdout.is_empty() {
    return Err(anyhow::anyhow!("Aucune clef trouvée pour ce fingerprint"));
  }
  String::from_utf8(output.stdout).context("invalid UTF-8 in key")
}

pub fn upload_public_key(fingerprint: &str) -> Result<String> {
  const PASTE_URL: &str = "https://paste.rs/";
  let armored = export_public_key_armored(fingerprint)?;
  let agent = ureq::Agent::config_builder()
    .max_redirects(3)
    .max_redirects_will_error(true)
    .timeout_connect(Some(std::time::Duration::from_secs(10)))
    .timeout_recv_response(Some(std::time::Duration::from_secs(15)))
    .build()
    .new_agent();
  let mut resp = agent
    .post(PASTE_URL)
    .header("Content-Type", "text/plain")
    .header("User-Agent", "pgpilot/1.0")
    .send(&armored)
    .map_err(|e| match e {
      ureq::Error::StatusCode(code) => {
        anyhow::anyhow!("paste.rs a refusé la requête (HTTP {code})")
      }
      other => anyhow::anyhow!("Connexion à paste.rs impossible : {other}"),
    })?;
  let url = resp
    .body_mut()
    .with_config()
    .limit(MAX_RESPONSE_BYTES)
    .read_to_string()
    .context("impossible de lire la réponse")?
    .trim()
    .to_string();
  if !url.starts_with("http") {
    return Err(anyhow::anyhow!("Réponse inattendue de paste.rs : {url}"));
  }
  Ok(url)
}

pub fn export_public_key(fingerprint: &str, path: &std::path::Path) -> Result<()> {
  validate_fp(fingerprint)?;
  let output = Command::new("gpg")
    .args(["--export", "--armor", fingerprint])
    .output()
    .context("failed to run gpg --export")?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(anyhow::anyhow!("{}", sanitize_gpg_stderr(&stderr)));
  }
  if output.stdout.is_empty() {
    return Err(anyhow::anyhow!("Aucune clef trouvée pour ce fingerprint"));
  }

  std::fs::write(path, &output.stdout).context("failed to write key file")
}

fn export_secret_key(fingerprint: &str, path: &std::path::Path) -> Result<()> {
  validate_fp(fingerprint)?;
  let output = Command::new("gpg")
    .args(["--export-secret-keys", "--armor", fingerprint])
    .output()
    .context("failed to run gpg --export-secret-keys")?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(anyhow::anyhow!("{}", sanitize_gpg_stderr(&stderr)));
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
  validate_fp(fingerprint)?;
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
  validate_fp(fingerprint)?;
  let url = format!(
    "https://keys.openpgp.org/vks/v1/by-fingerprint/{}",
    fingerprint.to_uppercase()
  );
  match safe_get(&url) {
    Ok(_) => Ok((fingerprint.to_string(), true)),
    Err(_) => Ok((fingerprint.to_string(), false)),
  }
}

pub fn publish_key(fingerprint: &str, keyserver_url: &str) -> Result<String> {
  validate_fp(fingerprint)?;
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
  let cmds = format!("key {pos}\nrevkey\ny\n2\n\ny\nsave\n");

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
    return Err(anyhow::anyhow!(
      "Révocation échouée : {}",
      sanitize_gpg_stderr(&stderr)
    ));
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
  validate_fp(master_fp)?;
  add_subkey(master_fp, algo, usage, expiry)?;
  let pos = subkey_position(master_fp, old_subkey_fp)?;
  revoke_subkey_at_pos(master_fp, pos)
}

pub fn add_subkey(master_fp: &str, algo: &str, usage: &str, expiry: &KeyExpiry) -> Result<()> {
  validate_fp(master_fp)?;
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
  validate_fp(master_fp)?;
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
  validate_fp(fingerprint)?;
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
    return Err(anyhow::anyhow!("{}", sanitize_gpg_stderr(&stderr)));
  }
  Ok(())
}

pub fn import_key_from_url(url: &str) -> Result<()> {
  let content = safe_get(url).map_err(|e| anyhow::anyhow!("Impossible de charger l'URL : {e}"))?;
  import_key_from_text(&content)
}

pub fn import_key_from_keyserver(query: &str, keyserver_url: &str) -> Result<()> {
  validate_keyserver_query(query)?;
  if query.contains('@') {
    let encoded = utf8_percent_encode(query, NON_ALPHANUMERIC).to_string();
    let url = format!("https://{keyserver_url}/pks/lookup?op=get&search={encoded}");
    let content =
      safe_get(&url).map_err(|e| anyhow::anyhow!("Impossible de joindre le keyserver : {e}"))?;
    import_key_from_text(&content)
  } else {
    let output = Command::new("gpg")
      .args(["--keyserver", keyserver_url, "--recv-keys", query])
      .output()
      .context("failed to run gpg --recv-keys")?;
    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      return Err(anyhow::anyhow!("{}", sanitize_gpg_stderr(&stderr)));
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
    return Err(anyhow::anyhow!("{}", sanitize_gpg_stderr(&stderr)));
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
      let name = uid.name().ok().flatten().unwrap_or_default().to_string();
      let email = uid.email().ok().flatten().unwrap_or_default().to_string();
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
    .with_policy(unsafe { &NullPolicy::new() }, None)
    .map(|vc| {
      vc.keys()
        .subkeys()
        .filter(|ka| !matches!(ka.revocation_status(), RevocationStatus::Revoked(_)))
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
  validate_fp(fingerprint)?;
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
  let ext_str = if armor { "asc" } else { "gpg" };
  let mut results = Vec::new();

  for file in files {
    let orig_ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
    let new_ext = if orig_ext.is_empty() {
      ext_str.to_string()
    } else {
      format!("{orig_ext}.{ext_str}")
    };
    let mut output = file.with_extension(&new_ext);
    let mut counter = 1u32;
    while output.exists() {
      let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
      output = file.with_file_name(format!("{stem}_{counter}.{new_ext}"));
      counter += 1;
    }

    let mut cmd = Command::new("gpg");
    cmd.arg("--homedir").arg(&gnupg);
    cmd.arg("--batch");
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
    cmd.arg("--");
    cmd.arg(file);

    let out = cmd.output().context("failed to run gpg --encrypt")?;
    if !out.status.success() {
      let stderr = String::from_utf8_lossy(&out.stderr);
      return Err(anyhow::anyhow!(
        "Échec du chiffrement de {} : {}",
        file.file_name().unwrap_or_default().to_string_lossy(),
        sanitize_gpg_stderr(stderr.trim())
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

pub fn sign_file(file: PathBuf, signer_fp: &str) -> Result<PathBuf> {
  validate_fp(signer_fp)?;
  let gnupg = super::gnupg_dir();
  let mut sig_path = file.with_extension("sig");
  let mut counter = 1u32;
  while sig_path.exists() {
    let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    sig_path = file.with_file_name(format!("{stem}_{counter}.sig"));
    counter += 1;
  }

  let out = Command::new("gpg")
    .arg("--homedir")
    .arg(&gnupg)
    .args(["--batch", "--detach-sign", "--armor"])
    .args(["--local-user", signer_fp])
    .arg("--output")
    .arg(&sig_path)
    .arg("--")
    .arg(&file)
    .output()
    .context("failed to run gpg --detach-sign")?;

  if !out.status.success() {
    let stderr = String::from_utf8_lossy(&out.stderr);
    return Err(anyhow::anyhow!(
      "Échec de la signature de {} : {}",
      file.file_name().unwrap_or_default().to_string_lossy(),
      sanitize_gpg_stderr(stderr.trim())
    ));
  }
  Ok(sig_path)
}

/// Builds a map from every key fingerprint (primary + subkeys) to the primary key's TrustLevel.
/// VALIDSIG emits the subkey fingerprint, so we need to resolve subkeys back to their primary.
fn all_fp_to_trust() -> Result<std::collections::HashMap<String, TrustLevel>> {
  let gnupg = super::gnupg_dir();
  let output = Command::new("gpg")
    .arg("--homedir")
    .arg(&gnupg)
    .args(["--list-keys", "--with-colons"])
    .output()
    .context("failed to run gpg --list-keys")?;

  let mut map = std::collections::HashMap::new();
  let mut current_trust = TrustLevel::Undefined;
  let mut last_was_key = false;

  for line in String::from_utf8(output.stdout)?.lines() {
    let fields: Vec<&str> = line.split(':').collect();
    match fields.first().copied() {
      Some("pub") if fields.len() > 8 => {
        current_trust = TrustLevel::from_char(fields[8].chars().next().unwrap_or('-'));
        last_was_key = true;
      }
      Some("sub") | Some("ssb") => {
        last_was_key = true;
      }
      Some("fpr") if fields.len() > 9 && last_was_key => {
        map.insert(fields[9].to_string(), current_trust.clone());
        last_was_key = false;
      }
      _ => {
        last_was_key = false;
      }
    }
  }

  Ok(map)
}

fn resolve_signer_trust(signer_fp: &Option<String>) -> TrustLevel {
  let fp = match signer_fp {
    Some(f) => f,
    None => return TrustLevel::Undefined,
  };

  all_fp_to_trust()
    .ok()
    .and_then(|m| m.get(fp.as_str()).cloned())
    .unwrap_or(TrustLevel::Undefined)
}

pub fn verify_signature(
  file: PathBuf,
  sig_file: Option<PathBuf>,
) -> Result<super::types::VerifyResult> {
  use super::types::{VerifyOutcome, VerifyResult};

  let gnupg = super::gnupg_dir();
  let sig = match sig_file {
    Some(s) => s,
    None => PathBuf::from(format!("{}.sig", file.display())),
  };

  if !sig.exists() {
    return Err(anyhow::anyhow!(
      "Fichier de signature introuvable : {}",
      display_path(&sig)
    ));
  }

  let out = Command::new("gpg")
    .arg("--homedir")
    .arg(&gnupg)
    .args(["--batch", "--status-fd", "1", "--verify"])
    .arg(&sig)
    .arg(&file)
    .output()
    .context("failed to run gpg --verify")?;

  let stdout = String::from_utf8_lossy(&out.stdout).to_string();
  let stderr = String::from_utf8_lossy(&out.stderr).to_string();
  let detail = format!("{stdout}{stderr}").trim().to_string();

  let has_goodsig = stdout.lines().any(|l| {
    let mut f = l.split_whitespace();
    f.next() == Some("[GNUPG:]") && f.next() == Some("GOODSIG")
  });
  let has_validsig = stdout.lines().any(|l| {
    let mut f = l.split_whitespace();
    f.next() == Some("[GNUPG:]") && f.next() == Some("VALIDSIG")
  });

  let outcome = if has_goodsig && has_validsig {
    VerifyOutcome::Valid
  } else if stdout.lines().any(|l| {
    let mut f = l.split_whitespace();
    f.next() == Some("[GNUPG:]") && matches!(f.next(), Some("NO_PUBKEY") | Some("ERRSIG"))
  }) {
    VerifyOutcome::UnknownKey
  } else if stdout.lines().any(|l| {
    let mut f = l.split_whitespace();
    f.next() == Some("[GNUPG:]") && f.next() == Some("EXPKEYSIG")
  }) {
    VerifyOutcome::ExpiredKey
  } else if stdout.lines().any(|l| {
    let mut f = l.split_whitespace();
    f.next() == Some("[GNUPG:]") && f.next() == Some("REVKEYSIG")
  }) {
    VerifyOutcome::RevokedKey
  } else if stdout.lines().any(|l| {
    let mut f = l.split_whitespace();
    f.next() == Some("[GNUPG:]") && f.next() == Some("BADSIG")
  }) {
    VerifyOutcome::BadSig
  } else {
    VerifyOutcome::Error(detail.clone())
  };

  // VALIDSIG émet le fingerprint 40 hex en champ 2 (fingerprint complet de la sous-clef).
  // BADSIG/EXPKEYSIG/REVKEYSIG/GOODSIG émettent seulement le key ID 16 hex en champ 2.
  let signer_fp = stdout.lines().find_map(|line| {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() >= 3 && fields[0] == "[GNUPG:]" && fields[1] == "VALIDSIG" {
      Some(fields[2].to_string())
    } else {
      None
    }
  });

  // Tokens GOODSIG, BADSIG, EXPKEYSIG, REVKEYSIG : `[GNUPG:] <TOKEN> <keyid> <name...>`
  let signer_name = stdout.lines().find_map(|line| {
    let tokens = ["GOODSIG", "BADSIG", "EXPKEYSIG", "REVKEYSIG"];
    for token in tokens {
      let prefix = format!("[GNUPG:] {token} ");
      if let Some(rest) = line.strip_prefix(&prefix) {
        let mut parts = rest.splitn(2, ' ');
        parts.next();
        return parts.next().map(str::to_string);
      }
    }
    None
  });

  let signed_at = stdout.lines().find_map(|line| {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() >= 4 && fields[0] == "[GNUPG:]" && fields[1] == "VALIDSIG" {
      if let Ok(ts) = fields[3].parse::<i64>() {
        use chrono::TimeZone;
        let dt = chrono::Utc.timestamp_opt(ts, 0).single()?;
        return Some(dt.format("%Y-%m-%d %H:%M UTC").to_string());
      }
    }
    None
  });

  let signer_trust = resolve_signer_trust(&signer_fp);

  Ok(VerifyResult {
    outcome,
    signer_name,
    signer_fp,
    signed_at,
    detail,
    signer_trust,
  })
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
        sanitize_gpg_stderr(stderr.trim())
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
