use chrono::Utc;

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

#[derive(Debug, Clone)]
pub struct CardInfo {
  pub serial: String,
  pub sig_fp: Option<String>,
  pub enc_fp: Option<String>,
  pub auth_fp: Option<String>,
}

pub fn format_date(t: std::time::SystemTime) -> String {
  let dt: chrono::DateTime<Utc> = t.into();
  dt.format("%Y-%m-%d").to_string()
}
