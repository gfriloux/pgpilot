use crate::i18n::Language;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
  pub language: Language,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      language: detect_system_language(),
    }
  }
}

impl Config {
  pub fn load() -> Result<Self, String> {
    let path = Self::path();
    if !path.exists() {
      return Ok(Config::default());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| format!("Cannot read config: {e}"))?;
    serde_yaml::from_str(&content).map_err(|e| format!("Invalid config YAML: {e}"))
  }

  pub fn save(&self) -> Result<(), String> {
    let path = Self::path();
    if let Some(parent) = path.parent() {
      std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create config dir: {e}"))?;
    }
    let content =
      serde_yaml::to_string(self).map_err(|e| format!("Cannot serialize config: {e}"))?;
    std::fs::write(&path, content).map_err(|e| format!("Cannot write config: {e}"))
  }

  pub fn path() -> PathBuf {
    dirs::config_dir()
      .unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".config")
      })
      .join("pgpilot")
      .join("config.yaml")
  }
}

fn detect_system_language() -> Language {
  let locale = std::env::var("LANG")
    .or_else(|_| std::env::var("LC_ALL"))
    .unwrap_or_default();
  if locale.starts_with("fr") {
    Language::French
  } else {
    Language::English
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn detect_language_fr() {
    let orig_lang = std::env::var("LANG").ok();
    let orig_lc = std::env::var("LC_ALL").ok();
    std::env::set_var("LANG", "fr_FR.UTF-8");
    std::env::remove_var("LC_ALL");
    assert_eq!(detect_system_language(), Language::French);
    match orig_lang {
      Some(v) => std::env::set_var("LANG", v),
      None => std::env::remove_var("LANG"),
    }
    if let Some(v) = orig_lc {
      std::env::set_var("LC_ALL", v);
    }
  }

  #[test]
  fn detect_language_en() {
    let orig_lang = std::env::var("LANG").ok();
    let orig_lc = std::env::var("LC_ALL").ok();
    std::env::set_var("LANG", "en_US.UTF-8");
    std::env::remove_var("LC_ALL");
    assert_eq!(detect_system_language(), Language::English);
    match orig_lang {
      Some(v) => std::env::set_var("LANG", v),
      None => std::env::remove_var("LANG"),
    }
    if let Some(v) = orig_lc {
      std::env::set_var("LC_ALL", v);
    }
  }

  #[test]
  fn detect_language_fallback() {
    let orig_lang = std::env::var("LANG").ok();
    let orig_lc = std::env::var("LC_ALL").ok();
    std::env::remove_var("LANG");
    std::env::remove_var("LC_ALL");
    assert_eq!(detect_system_language(), Language::English);
    if let Some(v) = orig_lang {
      std::env::set_var("LANG", v);
    }
    if let Some(v) = orig_lc {
      std::env::set_var("LC_ALL", v);
    }
  }

  #[test]
  fn config_roundtrip_yaml() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    let cfg = Config {
      language: Language::French,
    };
    let yaml = serde_yaml::to_string(&cfg).unwrap();
    std::fs::write(&path, &yaml).unwrap();
    let loaded: Config = serde_yaml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(loaded.language, Language::French);
  }

  #[test]
  fn all_english_strings_non_empty() {
    use crate::i18n::{strings_for, Language};
    let s = strings_for(Language::English);
    assert!(!s.nav_my_keys().is_empty());
    assert!(!s.btn_cancel().is_empty());
    assert!(!s.err_export_failed().is_empty());
    assert!(!s.settings_title().is_empty());
    assert!(!s.status_key_deleted().is_empty());
  }

  #[test]
  fn all_french_strings_non_empty() {
    use crate::i18n::{strings_for, Language};
    let s = strings_for(Language::French);
    assert!(!s.nav_my_keys().is_empty());
    assert!(!s.btn_cancel().is_empty());
    assert!(!s.err_export_failed().is_empty());
    assert!(!s.settings_title().is_empty());
    assert!(!s.status_key_deleted().is_empty());
  }
}
