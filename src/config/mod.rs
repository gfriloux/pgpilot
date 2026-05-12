use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
  #[default]
  English,
  French,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ThemeVariant {
  #[default]
  Catppuccin,
  Ussr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
  pub language: Language,
  #[serde(default = "default_scale_factor")]
  pub scale_factor: f64,
  pub theme: ThemeVariant,
  /// URL du broker MQTT par défaut.
  #[serde(default)]
  pub mqtt_default_relay: Option<String>,
  /// Fingerprint 40 hex de la clef locale utilisée pour le chat.
  #[serde(default)]
  pub chat_local_fp: Option<String>,
}

fn default_scale_factor() -> f64 {
  1.0
}

impl Default for Config {
  fn default() -> Self {
    Self {
      language: detect_system_language(),
      scale_factor: 1.0,
      theme: ThemeVariant::default(),
      mqtt_default_relay: None,
      chat_local_fp: None,
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

fn detect_language_from_locale(locale: &str) -> Language {
  if locale.starts_with("fr") {
    Language::French
  } else {
    Language::English
  }
}

fn detect_system_language() -> Language {
  let locale = std::env::var("LANG")
    .or_else(|_| std::env::var("LC_ALL"))
    .unwrap_or_default();
  detect_language_from_locale(&locale)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn detect_language_fr() {
    assert_eq!(detect_language_from_locale("fr_FR.UTF-8"), Language::French);
    assert_eq!(detect_language_from_locale("fr_BE.UTF-8"), Language::French);
    assert_eq!(detect_language_from_locale("fr"), Language::French);
  }

  #[test]
  fn detect_language_en() {
    assert_eq!(
      detect_language_from_locale("en_US.UTF-8"),
      Language::English
    );
    assert_eq!(
      detect_language_from_locale("en_GB.UTF-8"),
      Language::English
    );
  }

  #[test]
  fn detect_language_fallback() {
    assert_eq!(detect_language_from_locale(""), Language::English);
    assert_eq!(
      detect_language_from_locale("de_DE.UTF-8"),
      Language::English
    );
  }

  #[test]
  fn config_roundtrip_yaml() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("config.yaml");
    let cfg = Config {
      language: Language::French,
      scale_factor: 1.25,
      theme: ThemeVariant::Ussr,
      mqtt_default_relay: None,
      chat_local_fp: None,
    };
    let yaml = serde_yaml::to_string(&cfg).unwrap();
    std::fs::write(&path, &yaml).unwrap();
    let loaded: Config = serde_yaml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(loaded.language, Language::French);
    assert!((loaded.scale_factor - 1.25).abs() < f64::EPSILON);
    assert_eq!(loaded.theme, ThemeVariant::Ussr);
  }

  #[test]
  fn config_defaults_scale_factor() {
    let cfg = Config::default();
    assert!((cfg.scale_factor - 1.0).abs() < f64::EPSILON);
  }
}
