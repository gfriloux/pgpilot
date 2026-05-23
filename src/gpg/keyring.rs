use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::Stdio;

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
use super::{display_path, gnupg_dir, gpg_command, sanitize_gpg_stderr};

const MAX_RESPONSE_BYTES: u64 = 1 << 20; // 1 MiB

const ALLOWED_KEYSERVERS: &[&str] = &["https://keys.openpgp.org", "https://keyserver.ubuntu.com"];

fn validate_keyserver_url(url: &str) -> Result<String> {
  let url = url.trim().trim_end_matches('/');
  if !url.starts_with("https://") {
    anyhow::bail!("Keyserver URL must use https://");
  }
  if !ALLOWED_KEYSERVERS.contains(&url) {
    anyhow::bail!(
      "Keyserver not allowed: {url}. Permitted: {}",
      ALLOWED_KEYSERVERS.join(", ")
    );
  }
  Ok(url.to_string())
}

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

pub fn validate_fp(fp: &str) -> Result<()> {
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

fn check_gpg_output(output: &std::process::Output) -> anyhow::Result<()> {
  if output.status.success() {
    return Ok(());
  }
  let stderr = String::from_utf8_lossy(&output.stderr);
  Err(anyhow::anyhow!("{}", sanitize_gpg_stderr(&stderr)))
}

fn all_public_fingerprints() -> Result<HashSet<String>> {
  let homedir = gnupg_dir()?;
  let output = gpg_command(&homedir)
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
  // Bound lengths
  if name.is_empty() || name.len() > 64 {
    anyhow::bail!("Name must be 1–64 characters");
  }
  if email.is_empty() || email.len() > 254 {
    anyhow::bail!("Email must be 1–254 characters");
  }

  // Reject control bytes and dangerous characters in the UID
  let forbidden: &[char] = &['\n', '\r', '\0', '<', '>'];
  if name
    .chars()
    .any(|c| forbidden.contains(&c) || c.is_control())
  {
    anyhow::bail!("Name contains forbidden characters");
  }
  if email
    .chars()
    .any(|c| forbidden.contains(&c) || c.is_control())
  {
    anyhow::bail!("Email contains forbidden characters");
  }

  // Basic email format validation
  if !email.contains('@') || !email.contains('.') {
    anyhow::bail!("Invalid email format");
  }

  let homedir = gnupg_dir()?;
  let user_id = format!("{name} <{email}>");
  let sub_expire = expiry_to_str(subkey_expiry);

  let fps_before = all_public_fingerprints()?;

  let status = gpg_command(&homedir)
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
    let status = gpg_command(&homedir)
      .args(["--batch", "--quick-add-key", &fp, algo, usage, sub_expire])
      .status()
      .context("failed to run gpg --quick-add-key")?;
    if !status.success() {
      return Err(anyhow::anyhow!("L'ajout de la sous-clef {usage} a échoué"));
    }
  }

  if include_auth {
    let status = gpg_command(&homedir)
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
  let homedir = gnupg_dir()?;
  let output = gpg_command(&homedir)
    .args(["--export", "--armor", fingerprint])
    .output()
    .context("failed to run gpg --export")?;

  check_gpg_output(&output)?;
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
  let homedir = gnupg_dir()?;
  let output = gpg_command(&homedir)
    .args(["--export", "--armor", fingerprint])
    .output()
    .context("failed to run gpg --export")?;

  check_gpg_output(&output)?;
  if output.stdout.is_empty() {
    return Err(anyhow::anyhow!("Aucune clef trouvée pour ce fingerprint"));
  }

  use std::io::Write as _;
  let mut f = std::fs::OpenOptions::new()
    .write(true)
    .create_new(true)
    .open(path)
    .map_err(|e| {
      if e.kind() == std::io::ErrorKind::AlreadyExists {
        anyhow::anyhow!("File already exists: {}", display_path(path))
      } else {
        anyhow::anyhow!("Cannot create file: {e}")
      }
    })?;
  f.write_all(&output.stdout)
    .context("failed to write key file")
}

fn export_secret_key(fingerprint: &str, path: &std::path::Path) -> Result<()> {
  validate_fp(fingerprint)?;
  let homedir = gnupg_dir()?;
  let output = gpg_command(&homedir)
    .args(["--export-secret-keys", "--armor", fingerprint])
    .output()
    .context("failed to run gpg --export-secret-keys")?;

  check_gpg_output(&output)?;
  if output.stdout.is_empty() {
    return Err(anyhow::anyhow!(
      "Aucune clef secrète trouvée pour ce fingerprint"
    ));
  }

  std::fs::OpenOptions::new()
    .write(true)
    .create_new(true)
    .open(path)
    .and_then(|mut f| std::io::Write::write_all(&mut f, &output.stdout))
    .context("failed to write key file")
}

pub fn backup_key(
  fingerprint: &str,
  dir: &std::path::Path,
  key_id: &str,
) -> Result<(String, Option<String>)> {
  validate_fp(fingerprint)?;
  let key_filename = format!("{key_id}_secret.asc");
  let secret_path = dir.join(&key_filename);
  export_secret_key(fingerprint, &secret_path)?;

  // Restrict permissions on the exported secret key to owner-read/write only
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    let meta =
      std::fs::metadata(&secret_path).context("failed to read metadata for secret key file")?;
    let mut perms = meta.permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(&secret_path, perms)
      .context("failed to set permissions on secret key file")?;
  }

  let homedir = gnupg_dir()?;
  let rev_src = format!(
    "{homedir}/openpgp-revocs.d/{}.rev",
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
  let keyserver_url = validate_keyserver_url(keyserver_url)?;
  let homedir = gnupg_dir()?;
  let status = gpg_command(&homedir)
    .args(["--keyserver", &keyserver_url, "--send-keys", fingerprint])
    .status()
    .context("failed to run gpg --send-keys")?;

  if !status.success() {
    return Err(anyhow::anyhow!("L'envoi de la clef a échoué"));
  }
  Ok(keyserver_url)
}

fn subkey_position(master_fp: &str, subkey_fp: &str) -> Result<usize> {
  let homedir = gnupg_dir()?;
  let output = gpg_command(&homedir)
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
  let homedir = gnupg_dir()?;

  let mut child = gpg_command(&homedir)
    .args([
      "--no-tty",
      "--status-fd",
      "2",
      "--command-fd",
      "0",
      "--edit-key",
      master_fp,
    ])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .context("failed to spawn gpg --edit-key")?;

  let mut stdin = child.stdin.take().context("failed to open gpg stdin")?;
  let stderr = child.stderr.take().context("failed to open gpg stderr")?;
  let reader = BufReader::new(stderr);

  // prompt_count tracks how many times GET_LINE keyedit.prompt has been seen:
  //   0 → respond with "key {pos}"
  //   1 → respond with "revkey"
  //   2 → respond with "save"
  let mut prompt_count: usize = 0;

  for raw in reader.lines() {
    let line = raw.context("failed to read gpg stderr")?;

    if !line.contains("[GNUPG:]") {
      continue;
    }

    if line.contains("KEY_CONSIDERED") || line.contains("GOT_IT") {
      continue;
    }

    if line.contains("GET_LINE keyedit.prompt") {
      let cmd = match prompt_count {
        0 => format!("key {pos}\n"),
        1 => "revkey\n".to_string(),
        _ => "save\n".to_string(),
      };
      stdin
        .write_all(cmd.as_bytes())
        .context("failed to write to gpg stdin")?;
      prompt_count += 1;
      if prompt_count > 2 {
        break;
      }
      continue;
    }

    if line.contains("GET_BOOL keyedit.revoke.subkey") {
      stdin
        .write_all(b"y\n")
        .context("failed to write to gpg stdin")?;
      continue;
    }

    if line.contains("GET_LINE ask_revocation_reason.code") {
      stdin
        .write_all(b"2\n")
        .context("failed to write to gpg stdin")?;
      continue;
    }

    if line.contains("GET_LINE ask_revocation_reason.text") {
      stdin
        .write_all(b"\n")
        .context("failed to write to gpg stdin")?;
      continue;
    }

    if line.contains("GET_BOOL ask_revocation_reason.okay") {
      stdin
        .write_all(b"y\n")
        .context("failed to write to gpg stdin")?;
      continue;
    }

    if line.contains("GET_") {
      let _ = stdin.write_all(b"quit\n");
      drop(stdin);
      let _ = child.wait();
      return Err(anyhow::anyhow!(
        "Révocation échouée : token gpg inattendu : {line}"
      ));
    }
  }

  drop(stdin);
  let status = child.wait().context("failed to wait for gpg")?;
  if !status.success() {
    return Err(anyhow::anyhow!(
      "Révocation échouée : gpg a terminé avec un code d'erreur"
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

  let snapshot = tempfile::NamedTempFile::new().context("failed to create snapshot tempfile")?;
  export_secret_key(master_fp, snapshot.path())?;

  add_subkey(master_fp, algo, usage, expiry)?;

  let pos = subkey_position(master_fp, old_subkey_fp)?;
  if let Err(e) = revoke_subkey_at_pos(master_fp, pos) {
    let homedir = gnupg_dir()?;
    let reimport_ok = gpg_command(&homedir)
      .args(["--batch", "--import", "--"])
      .arg(snapshot.path())
      .status()
      .map(|s| s.success())
      .unwrap_or(false);
    drop(snapshot);
    if reimport_ok {
      anyhow::bail!("Révocation échouée — keyring restauré à l'état précédent : {e}");
    } else {
      anyhow::bail!(
        "Révocation échouée ET restauration impossible — vérifiez votre keyring manuellement : {e}"
      );
    }
  }

  drop(snapshot);
  Ok(())
}

pub fn add_subkey(master_fp: &str, algo: &str, usage: &str, expiry: &KeyExpiry) -> Result<()> {
  validate_fp(master_fp)?;
  let homedir = gnupg_dir()?;
  let expire = expiry_to_str(expiry);
  let status = gpg_command(&homedir)
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
  let homedir = gnupg_dir()?;
  let expire = expiry_to_str(expiry);
  let status = gpg_command(&homedir)
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
  let homedir = gnupg_dir()?;
  let cmd = if has_secret {
    "--delete-secret-and-public-keys"
  } else {
    "--delete-keys"
  };

  let status = gpg_command(&homedir)
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
  let homedir = gnupg_dir()?;
  let mut child = gpg_command(&homedir)
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
  check_gpg_output(&output)?;
  Ok(())
}

pub fn import_key_from_url(url: &str) -> Result<()> {
  let content = safe_get(url).map_err(|e| anyhow::anyhow!("Impossible de charger l'URL : {e}"))?;
  import_key_from_text(&content)
}

pub fn import_key_from_keyserver(query: &str, keyserver_url: &str) -> Result<()> {
  validate_keyserver_query(query)?;
  let keyserver_url = validate_keyserver_url(keyserver_url)?;
  if query.contains('@') {
    let encoded = utf8_percent_encode(query, NON_ALPHANUMERIC).to_string();
    let url = format!("{keyserver_url}/pks/lookup?op=get&search={encoded}");
    let content =
      safe_get(&url).map_err(|e| anyhow::anyhow!("Impossible de joindre le keyserver : {e}"))?;
    import_key_from_text(&content)
  } else {
    let homedir = gnupg_dir()?;
    let output = gpg_command(&homedir)
      .args(["--keyserver", &keyserver_url, "--recv-keys", query])
      .output()
      .context("failed to run gpg --recv-keys")?;
    check_gpg_output(&output)?;
    Ok(())
  }
}

pub fn import_key(path: &std::path::Path) -> Result<()> {
  let homedir = gnupg_dir()?;
  let output = gpg_command(&homedir)
    .args(["--import", "--"])
    .arg(path)
    .output()
    .context("failed to run gpg --import")?;

  check_gpg_output(&output)?;
  Ok(())
}

fn key_ownertrusts() -> Result<std::collections::HashMap<String, TrustLevel>> {
  let homedir = gnupg_dir()?;
  let output = gpg_command(&homedir)
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
  let homedir = gnupg_dir()?;
  let pub_bytes = gpg_command(&homedir)
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
  let homedir = gnupg_dir()?;
  let output = gpg_command(&homedir)
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

  // SAFETY: NullPolicy skips algorithm checks — used only to enumerate subkey metadata
  // (read-only), never to verify signatures. Required for legacy SHA-1 keys.
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
  let homedir = gnupg_dir()?;
  let input = format!("{fingerprint}:{level}:\n");

  let mut child = gpg_command(&homedir)
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
  let homedir = gnupg_dir()?;
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

    let mut cmd = gpg_command(&homedir);
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
  let homedir = gnupg_dir()?;
  let mut sig_path = file.with_extension("sig");
  let mut counter = 1u32;
  while sig_path.exists() {
    let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    sig_path = file.with_file_name(format!("{stem}_{counter}.sig"));
    counter += 1;
  }

  let out = gpg_command(&homedir)
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
  let homedir = gnupg_dir()?;
  let output = gpg_command(&homedir)
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

fn has_gnupg_token(stdout: &str, token: &str) -> bool {
  stdout.lines().any(|l| {
    let mut f = l.split_whitespace();
    f.next() == Some("[GNUPG:]") && f.next() == Some(token)
  })
}

fn parse_verify_status(stdout: &str, stderr: &str) -> super::types::VerifyResult {
  use super::types::{TrustLevel, VerifyOutcome, VerifyResult};
  let detail = format!("{stdout}{stderr}").trim().to_string();

  let has_goodsig = has_gnupg_token(stdout, "GOODSIG");
  let has_validsig = has_gnupg_token(stdout, "VALIDSIG");

  let outcome = if has_goodsig && has_validsig {
    VerifyOutcome::Valid
  } else if stdout.lines().any(|l| {
    let mut f = l.split_whitespace();
    f.next() == Some("[GNUPG:]") && matches!(f.next(), Some("NO_PUBKEY") | Some("ERRSIG"))
  }) {
    VerifyOutcome::UnknownKey
  } else if has_gnupg_token(stdout, "EXPKEYSIG") {
    VerifyOutcome::ExpiredKey
  } else if has_gnupg_token(stdout, "REVKEYSIG") {
    VerifyOutcome::RevokedKey
  } else if has_gnupg_token(stdout, "BADSIG") {
    VerifyOutcome::BadSig
  } else {
    VerifyOutcome::Error(detail.clone())
  };

  // VALIDSIG émet le fingerprint 40 hex en champ 2 (fingerprint complet de la sous-clef).
  // BADSIG/EXPKEYSIG/REVKEYSIG/GOODSIG émettent seulement le key ID 16 hex en champ 2.
  let signer_fp = stdout.lines().find_map(|line| {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() >= 3 && fields[0] == "[GNUPG:]" && fields[1] == "VALIDSIG" {
      let fp = fields[2];
      if fp.len() == 40 && fp.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(fp.to_string())
      } else {
        None
      }
    } else {
      None
    }
  });

  // Tokens GOODSIG, BADSIG, EXPKEYSIG, REVKEYSIG : `[GNUPG:] <TOKEN> <keyid> <name...>`
  let signer_name = stdout.lines().find_map(|line| {
    for token in ["GOODSIG", "BADSIG", "EXPKEYSIG", "REVKEYSIG"] {
      let prefix = format!("[GNUPG:] {token} ");
      if let Some(rest) = line.strip_prefix(&prefix) {
        let mut parts = rest.splitn(2, ' ');
        parts.next();
        return parts.next().map(str::to_string);
      }
    }
    None
  });

  // VALIDSIG format: [GNUPG:] VALIDSIG <fp> <date> <timestamp_unix> ...
  // fields[4] is the unix timestamp (fields[3] is the ISO date string).
  let signed_at = stdout.lines().find_map(|line| {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() >= 5 && fields[0] == "[GNUPG:]" && fields[1] == "VALIDSIG" {
      if let Ok(ts) = fields[4].parse::<i64>() {
        use chrono::TimeZone;
        let dt = chrono::Utc.timestamp_opt(ts, 0).single()?;
        return Some(dt.format("%Y-%m-%d %H:%M UTC").to_string());
      }
    }
    None
  });

  VerifyResult {
    outcome,
    signer_name,
    signer_fp,
    signed_at,
    detail,
    signer_trust: TrustLevel::Undefined,
  }
}

pub fn verify_signature(
  file: PathBuf,
  sig_file: Option<PathBuf>,
) -> Result<super::types::VerifyResult> {
  let homedir = gnupg_dir()?;
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

  let out = gpg_command(&homedir)
    .args(["--batch", "--status-fd", "1", "--verify", "--"])
    .arg(&sig)
    .arg(&file)
    .output()
    .context("failed to run gpg --verify")?;

  let stdout = String::from_utf8_lossy(&out.stdout).to_string();
  let stderr = String::from_utf8_lossy(&out.stderr).to_string();

  let mut result = parse_verify_status(&stdout, &stderr);
  result.signer_trust = resolve_signer_trust(&result.signer_fp);
  Ok(result)
}

pub fn inspect_decrypt(file: &std::path::Path) -> Result<super::types::DecryptStatus> {
  use super::types::DecryptStatus;
  let homedir = gnupg_dir()?;
  let out = gpg_command(&homedir)
    .args([
      "--batch",
      "--status-fd",
      "1",
      "--list-only",
      "--decrypt",
      "--",
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

/// Returns the path to the revocation certificate if it exists.
pub fn revocation_cert_path(homedir: &str, fp: &str) -> Result<Option<std::path::PathBuf>> {
  validate_fp(fp)?;
  let path = std::path::PathBuf::from(homedir)
    .join("openpgp-revocs.d")
    .join(format!("{fp}.rev"));
  Ok(if path.exists() { Some(path) } else { None })
}

/// Generates a revocation certificate via `gpg --gen-revoke`.
///
/// Returns the path of the generated certificate.
pub fn generate_revocation_cert(homedir: &str, fp: &str) -> Result<std::path::PathBuf> {
  validate_fp(fp)?;
  let out_path = std::path::PathBuf::from(homedir)
    .join("openpgp-revocs.d")
    .join(format!("{fp}.rev"));

  let output = gpg_command(homedir)
    .args([
      "--batch",
      "--yes",
      "--gen-revoke",
      "--output",
      out_path.to_str().unwrap_or(""),
      fp,
    ])
    .output()
    .context("gpg spawn error")?;

  if output.status.success() || out_path.exists() {
    Ok(out_path)
  } else {
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(anyhow::anyhow!("{}", sanitize_gpg_stderr(&stderr)))
  }
}

pub fn decrypt_files(files: &[PathBuf]) -> Result<Vec<String>> {
  let homedir = gnupg_dir()?;
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

    let mut cmd = gpg_command(&homedir);
    cmd.arg("--batch");
    cmd.arg("--yes");
    cmd.arg("--output").arg(&candidate);
    cmd.arg("--decrypt");
    cmd.arg("--");
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

#[cfg(test)]
mod tests {
  use super::*;

  // ── validate_fp ──────────────────────────────────────────────────────────

  #[test]
  fn fp_valid_40_uppercase_hex() {
    assert!(validate_fp("A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2").is_ok());
  }

  #[test]
  fn fp_valid_40_lowercase_hex() {
    assert!(validate_fp("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2").is_ok());
  }

  #[test]
  fn fp_valid_40_mixed_case() {
    assert!(validate_fp("A1b2C3d4E5f6A1b2C3d4E5f6A1b2C3d4E5f6A1b2").is_ok());
  }

  #[test]
  fn fp_invalid_too_short() {
    assert!(validate_fp("A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1").is_err()); // 39 chars
  }

  #[test]
  fn fp_invalid_too_long() {
    assert!(validate_fp("A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2X").is_err()); // 41 chars
  }

  #[test]
  fn fp_invalid_non_hex_char() {
    assert!(validate_fp("A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1GG").is_err());
  }

  #[test]
  fn fp_invalid_empty() {
    assert!(validate_fp("").is_err());
  }

  #[test]
  fn fp_invalid_with_spaces() {
    assert!(validate_fp("A1B2 C3D4 E5F6 A1B2 C3D4 E5F6 A1B2 C3D4 E5F6 A1B2").is_err());
  }

  // ── validate_keyserver_url ───────────────────────────────────────────────

  #[test]
  fn keyserver_url_openpgp_ok() {
    assert!(validate_keyserver_url("https://keys.openpgp.org").is_ok());
  }

  #[test]
  fn keyserver_url_ubuntu_ok() {
    assert!(validate_keyserver_url("https://keyserver.ubuntu.com").is_ok());
  }

  #[test]
  fn keyserver_url_trailing_slash_ok() {
    // La fonction trim_end_matches('/') doit accepter les slashes traînants.
    assert!(validate_keyserver_url("https://keys.openpgp.org/").is_ok());
  }

  #[test]
  fn keyserver_url_http_rejected() {
    assert!(validate_keyserver_url("http://keys.openpgp.org").is_err());
  }

  #[test]
  fn keyserver_url_unknown_host_rejected() {
    assert!(validate_keyserver_url("https://evil.example.com").is_err());
  }

  #[test]
  fn keyserver_url_empty_rejected() {
    assert!(validate_keyserver_url("").is_err());
  }

  // ── validate_keyserver_query ─────────────────────────────────────────────

  #[test]
  fn keyserver_query_full_fp_ok() {
    assert!(validate_keyserver_query("A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2").is_ok());
  }

  #[test]
  fn keyserver_query_long_id_ok() {
    assert!(validate_keyserver_query("DEADBEEF12345678").is_ok()); // 16 hex
  }

  #[test]
  fn keyserver_query_email_ok() {
    assert!(validate_keyserver_query("alice@example.com").is_ok());
  }

  #[test]
  fn keyserver_query_email_with_plus_ok() {
    assert!(validate_keyserver_query("alice+pgp@example.com").is_ok());
  }

  #[test]
  fn keyserver_query_short_id_rejected() {
    // 8 hex (short ID) n'est plus accepté — trop ambiguë.
    assert!(validate_keyserver_query("DEADBEEF").is_err());
  }

  #[test]
  fn keyserver_query_illegal_chars_rejected() {
    assert!(validate_keyserver_query("alice; rm -rf /").is_err());
  }

  #[test]
  fn keyserver_query_empty_rejected() {
    assert!(validate_keyserver_query("").is_err());
  }

  // ── parse_verify_status ───────────────────────────────────────────────────

  #[test]
  fn parse_verify_valid_signature() {
    let stdout = "\
[GNUPG:] GOODSIG DEADBEEF12345678 Alice Dupont <alice@example.com>\n\
[GNUPG:] VALIDSIG A1B2C3D4E5F6A1B2C3D4AAAA1234567890ABCDEF 2024-01-15 1705296000 0 4 0 22 8 00 A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4E5F6A1B2\n\
[GNUPG:] TRUST_FULL 0 pgp\n";
    let result = parse_verify_status(stdout, "");
    assert_eq!(result.outcome, super::super::types::VerifyOutcome::Valid);
    assert_eq!(
      result.signer_fp.as_deref(),
      Some("A1B2C3D4E5F6A1B2C3D4AAAA1234567890ABCDEF")
    );
    assert_eq!(
      result.signer_name.as_deref(),
      Some("Alice Dupont <alice@example.com>")
    );
    assert!(result.signed_at.is_some());
  }

  #[test]
  fn parse_verify_bad_signature() {
    let stdout = "\
[GNUPG:] BADSIG DEADBEEF12345678 Alice Dupont <alice@example.com>\n";
    let result = parse_verify_status(stdout, "");
    assert_eq!(result.outcome, super::super::types::VerifyOutcome::BadSig);
    assert!(result.signer_fp.is_none());
    assert_eq!(
      result.signer_name.as_deref(),
      Some("Alice Dupont <alice@example.com>")
    );
  }

  #[test]
  fn parse_verify_unknown_key() {
    let stdout = "\
[GNUPG:] NO_PUBKEY DEADBEEF12345678\n\
[GNUPG:] ERRSIG DEADBEEF12345678 22 8 00 1705296000 9\n";
    let result = parse_verify_status(stdout, "");
    assert_eq!(
      result.outcome,
      super::super::types::VerifyOutcome::UnknownKey
    );
    assert!(result.signer_fp.is_none());
    assert!(result.signer_name.is_none());
  }

  #[test]
  fn parse_verify_expired_key() {
    let stdout = "\
[GNUPG:] EXPKEYSIG DEADBEEF12345678 Alice Dupont <alice@example.com>\n";
    let result = parse_verify_status(stdout, "");
    assert_eq!(
      result.outcome,
      super::super::types::VerifyOutcome::ExpiredKey
    );
    assert_eq!(
      result.signer_name.as_deref(),
      Some("Alice Dupont <alice@example.com>")
    );
  }

  #[test]
  fn parse_verify_revoked_key() {
    let stdout = "\
[GNUPG:] REVKEYSIG DEADBEEF12345678 Alice Dupont <alice@example.com>\n";
    let result = parse_verify_status(stdout, "");
    assert_eq!(
      result.outcome,
      super::super::types::VerifyOutcome::RevokedKey
    );
  }

  #[test]
  fn parse_verify_empty_stdout_returns_error() {
    let result = parse_verify_status("", "gpg: some error");
    assert!(matches!(
      result.outcome,
      super::super::types::VerifyOutcome::Error(_)
    ));
    assert!(result.signer_fp.is_none());
  }

  #[test]
  fn parse_verify_validsig_timestamp_parsed() {
    // ts = 1705296000 → 2024-01-15
    let stdout = "\
[GNUPG:] GOODSIG DEADBEEF12345678 Alice\n\
[GNUPG:] VALIDSIG ABCDEF1234567890ABCDEF1234567890ABCDEF12 2024-01-15 1705296000 0 4 0 22 8 00 ABCDEF1234567890ABCDEF1234567890ABCDEF12\n";
    let result = parse_verify_status(stdout, "");
    assert!(result
      .signed_at
      .as_deref()
      .map(|s| s.contains("2024-01-15"))
      .unwrap_or(false));
  }

  #[test]
  fn parse_verify_goodsig_without_validsig_is_error() {
    // GOODSIG seul sans VALIDSIG ne doit pas être considéré comme valide.
    let stdout = "[GNUPG:] GOODSIG DEADBEEF12345678 Alice\n";
    let result = parse_verify_status(stdout, "");
    assert_ne!(result.outcome, super::super::types::VerifyOutcome::Valid);
  }
}
