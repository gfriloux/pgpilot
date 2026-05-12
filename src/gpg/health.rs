use std::fs;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

use super::types::KeyInfo;
use super::{gnupg_dir, gpg_command};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
  Ok,
  Info,
  Warning,
  Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
  pub category: &'static str,
  pub name: &'static str,
  pub status: CheckStatus,
  pub current_value: Option<String>,
  pub explanation: &'static str,
  pub fix: Option<&'static str>,
}

fn check_gpg_installed() -> HealthCheck {
  let result = gnupg_dir()
    .ok()
    .and_then(|homedir| gpg_command(&homedir).arg("--version").output().ok());
  match result {
    Some(o) if o.status.success() => {
      let first_line = String::from_utf8_lossy(&o.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
      HealthCheck {
        category: "Installation",
        name: "GPG installé",
        status: CheckStatus::Ok,
        current_value: Some(first_line),
        explanation: "GnuPG est requis pour toutes les opérations de clef.",
        fix: None,
      }
    }
    _ => HealthCheck {
      category: "Installation",
      name: "GPG installé",
      status: CheckStatus::Error,
      current_value: None,
      explanation: "GnuPG est requis pour toutes les opérations de clef.",
      fix: Some("Installez GnuPG via votre gestionnaire de paquets."),
    },
  }
}

fn check_gpg_version() -> HealthCheck {
  let output = match gnupg_dir()
    .ok()
    .and_then(|homedir| gpg_command(&homedir).arg("--version").output().ok())
    .filter(|o| o.status.success())
  {
    Some(o) => o,
    None => {
      return HealthCheck {
        category: "Installation",
        name: "Version GPG ≥ 2.1",
        status: CheckStatus::Error,
        current_value: None,
        explanation: "GPG 2.1+ est requis pour Ed25519 et la gestion moderne des sous-clefs.",
        fix: Some("Mettez à jour GnuPG."),
      }
    }
  };

  let text = String::from_utf8_lossy(&output.stdout);
  let version_str = text
    .lines()
    .next()
    .unwrap_or("")
    .split_whitespace()
    .last()
    .unwrap_or("0.0.0")
    .to_string();

  let parts: Vec<u32> = version_str
    .split('.')
    .filter_map(|s| s.parse().ok())
    .collect();

  let major = parts.first().copied().unwrap_or(0);
  let minor = parts.get(1).copied().unwrap_or(0);

  let ok = major > 2 || (major == 2 && minor >= 1);
  HealthCheck {
    category: "Installation",
    name: "Version GPG ≥ 2.1",
    status: if ok {
      CheckStatus::Ok
    } else {
      CheckStatus::Warning
    },
    current_value: Some(version_str),
    explanation: "GPG 2.1+ est requis pour Ed25519 et la gestion moderne des sous-clefs.",
    fix: if ok {
      None
    } else {
      Some("Mettez à jour GnuPG vers la version 2.1 ou supérieure.")
    },
  }
}

fn check_agent_running() -> HealthCheck {
  let ok = Command::new("gpg-connect-agent")
    .arg("/bye")
    .output()
    .map(|o| o.status.success())
    .unwrap_or(false);

  HealthCheck {
    category: "Agent GPG",
    name: "gpg-agent actif",
    status: if ok {
      CheckStatus::Ok
    } else {
      CheckStatus::Error
    },
    current_value: None,
    explanation:
      "L'agent met en cache votre mot de passe et orchestre toutes les opérations cryptographiques.",
    fix: if ok {
      None
    } else {
      Some("gpgconf --launch gpg-agent")
    },
  }
}

fn check_pinentry() -> HealthCheck {
  let configured = fs::read_to_string(format!(
    "{}/gpg-agent.conf",
    gnupg_dir().unwrap_or_default()
  ))
  .ok()
  .and_then(|conf| {
    conf
      .lines()
      .filter(|l| !l.trim_start().starts_with('#'))
      .find_map(|l| {
        l.trim()
          .strip_prefix("pinentry-program")
          .map(|v| v.trim().to_string())
      })
  });

  if let Some(path) = configured {
    if Path::new(&path).exists() {
      return HealthCheck {
        category: "Agent GPG",
        name: "pinentry configuré",
        status: CheckStatus::Ok,
        current_value: Some(path),
        explanation: "pinentry gère la saisie sécurisée du mot de passe hors du terminal.",
        fix: None,
      };
    }
    return HealthCheck {
      category: "Agent GPG",
      name: "pinentry configuré",
      status: CheckStatus::Error,
      current_value: Some(format!("{path} (introuvable)")),
      explanation: "Le chemin configuré dans gpg-agent.conf ne pointe pas sur un binaire valide.",
      fix: Some("Corrigez pinentry-program dans ~/.gnupg/gpg-agent.conf."),
    };
  }

  let output = Command::new("which").arg("pinentry").output();
  match output {
    Ok(o) if o.status.success() => HealthCheck {
      category: "Agent GPG",
      name: "pinentry disponible",
      status: CheckStatus::Ok,
      current_value: Some(String::from_utf8_lossy(&o.stdout).trim().to_string()),
      explanation: "pinentry gère la saisie sécurisée du mot de passe hors du terminal.",
      fix: None,
    },
    _ => HealthCheck {
      category: "Agent GPG",
      name: "pinentry disponible",
      status: CheckStatus::Error,
      current_value: None,
      explanation: "pinentry gère la saisie sécurisée du mot de passe hors du terminal.",
      fix: Some("Installez pinentry (pinentry-gtk2, pinentry-qt ou pinentry-curses)."),
    },
  }
}

fn parse_agent_conf_value(key: &str) -> Option<u64> {
  let conf = fs::read_to_string(format!(
    "{}/gpg-agent.conf",
    gnupg_dir().unwrap_or_default()
  ))
  .ok()?;
  conf
    .lines()
    .filter(|l| !l.trim_start().starts_with('#'))
    .find_map(|l| {
      let l = l.trim();
      l.strip_prefix(key)?.trim().parse().ok()
    })
}

fn check_default_cache_ttl() -> HealthCheck {
  let value = parse_agent_conf_value("default-cache-ttl");
  let (status, display, fix) = match value {
    None => (
      CheckStatus::Info,
      "Non configuré (défaut GPG : 600s / 10 min)".to_string(),
      None,
    ),
    Some(0) => (
      CheckStatus::Warning,
      "0s — cache désactivé, passphrase requise à chaque opération".to_string(),
      Some("default-cache-ttl 3600  # dans ~/.gnupg/gpg-agent.conf"),
    ),
    Some(ttl) if ttl > 86400 => (
      CheckStatus::Warning,
      format!("{ttl}s ({} h) — durée très longue", ttl / 3600),
      None,
    ),
    Some(ttl) => (CheckStatus::Ok, format!("{ttl}s ({} min)", ttl / 60), None),
  };
  HealthCheck {
    category: "Agent GPG",
    name: "Délai de cache (default-cache-ttl)",
    status,
    current_value: Some(display),
    explanation: "Durée avant que le mot de passe soit redemandé après inactivité.",
    fix,
  }
}

fn check_max_cache_ttl() -> HealthCheck {
  let value = parse_agent_conf_value("max-cache-ttl");
  let (status, display, fix) = match value {
    None => (
      CheckStatus::Info,
      "Non configuré (défaut GPG : 7200s / 2 h)".to_string(),
      None,
    ),
    Some(0) => (
      CheckStatus::Warning,
      "0s — cache désactivé".to_string(),
      Some("max-cache-ttl 28800  # dans ~/.gnupg/gpg-agent.conf"),
    ),
    Some(ttl) if ttl > 86400 => (
      CheckStatus::Warning,
      format!(
        "{ttl}s ({} h) — durée très longue, risque si la machine est partagée",
        ttl / 3600
      ),
      None,
    ),
    Some(ttl) => (CheckStatus::Ok, format!("{ttl}s ({} h)", ttl / 3600), None),
  };
  HealthCheck {
    category: "Agent GPG",
    name: "Délai maximum (max-cache-ttl)",
    status,
    current_value: Some(display),
    explanation: "Durée maximale absolue avant que le mot de passe soit obligatoirement re-saisi.",
    fix,
  }
}

fn check_gnupg_permissions() -> HealthCheck {
  let dir = gnupg_dir().unwrap_or_default();

  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    return match fs::metadata(&dir) {
      Ok(meta) => {
        let mode = meta.permissions().mode() & 0o777;
        if mode == 0o700 {
          HealthCheck {
            category: "Sécurité",
            name: "Permissions ~/.gnupg",
            status: CheckStatus::Ok,
            current_value: Some(format!("{mode:o}")),
            explanation: "Le répertoire GPG doit être lisible uniquement par vous — GPG refusera de fonctionner autrement.",
            fix: None,
          }
        } else {
          HealthCheck {
            category: "Sécurité",
            name: "Permissions ~/.gnupg",
            status: CheckStatus::Error,
            current_value: Some(format!("{mode:o} (doit être 700)")),
            explanation: "Le répertoire GPG doit être lisible uniquement par vous — GPG refusera de fonctionner autrement.",
            fix: Some("chmod 700 ~/.gnupg"),
          }
        }
      }
      Err(_) => HealthCheck {
        category: "Sécurité",
        name: "Permissions ~/.gnupg",
        status: CheckStatus::Warning,
        current_value: Some("Répertoire introuvable".to_string()),
        explanation: "Le répertoire GPG doit exister avec les bonnes permissions.",
        fix: Some("mkdir -m 700 ~/.gnupg"),
      },
    };
  }

  #[allow(unreachable_code)]
  HealthCheck {
    category: "Sécurité",
    name: "Permissions ~/.gnupg",
    status: CheckStatus::Ok,
    current_value: Some("Non applicable".to_string()),
    explanation: "Vérification disponible sur Unix uniquement.",
    fix: None,
  }
}

fn check_revocation_certs(keys: &[KeyInfo]) -> HealthCheck {
  let dir = gnupg_dir().unwrap_or_default();
  let rev_dir = format!("{dir}/openpgp-revocs.d");

  let owned: Vec<&KeyInfo> = keys.iter().filter(|k| k.has_secret).collect();
  let missing: Vec<String> = owned
    .iter()
    .filter(|k| {
      let path = format!("{}/{}.rev", rev_dir, k.fingerprint.to_uppercase());
      !Path::new(&path).exists()
    })
    .map(|k| {
      if k.name.is_empty() {
        k.key_id.clone()
      } else {
        format!("{} <{}>", k.name, k.email)
      }
    })
    .collect();

  if owned.is_empty() {
    return HealthCheck {
      category: "Sécurité",
      name: "Certificats de révocation",
      status: CheckStatus::Ok,
      current_value: Some("Aucune clef privée détectée".to_string()),
      explanation: "Permet d'invalider une clef compromise ou perdue.",
      fix: None,
    };
  }

  if missing.is_empty() {
    HealthCheck {
      category: "Sécurité",
      name: "Certificats de révocation",
      status: CheckStatus::Ok,
      current_value: Some(format!("{} clef(s) protégée(s)", owned.len())),
      explanation: "Permet d'invalider une clef compromise ou perdue.",
      fix: None,
    }
  } else {
    HealthCheck {
      category: "Sécurité",
      name: "Certificats de révocation",
      status: CheckStatus::Warning,
      current_value: Some(format!(
        "{} manquant(s) : {}",
        missing.len(),
        missing.join(", ")
      )),
      explanation: "Sans ce certificat, une clef compromise ou perdue ne peut pas être invalidée auprès de vos correspondants.",
      fix: Some(
        "gpg --output ~/.gnupg/openpgp-revocs.d/<fp>.rev --gen-revoke <fingerprint>\nConservez ce fichier hors ligne.",
      ),
    }
  }
}

pub fn run_all_checks(keys: &[KeyInfo]) -> Vec<HealthCheck> {
  let checks = vec![
    check_gpg_installed(),
    check_gpg_version(),
    check_agent_running(),
    check_pinentry(),
    check_default_cache_ttl(),
    check_max_cache_ttl(),
    check_gnupg_permissions(),
    check_revocation_certs(keys),
  ];
  checks
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn all_health_checks_run_without_panic() {
    let empty_keys: Vec<KeyInfo> = vec![];
    let results = run_all_checks(&empty_keys);

    // All checks should return a valid HealthCheck
    for r in &results {
      assert!(matches!(
        r.status,
        CheckStatus::Ok | CheckStatus::Info | CheckStatus::Warning | CheckStatus::Error
      ));
      assert!(!r.category.is_empty());
      assert!(!r.name.is_empty());
    }

    // Should have 8 checks
    assert_eq!(results.len(), 8);
  }

  #[test]
  fn check_status_enum_values() {
    assert_eq!(CheckStatus::Ok, CheckStatus::Ok);
    assert_ne!(CheckStatus::Ok, CheckStatus::Error);
    assert_ne!(CheckStatus::Info, CheckStatus::Warning);
  }
}
