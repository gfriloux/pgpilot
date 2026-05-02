use chrono::Utc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum KeyExpiry {
  OneYear,
  #[default]
  TwoYears,
  FiveYears,
}

impl std::fmt::Display for KeyExpiry {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      KeyExpiry::OneYear => write!(f, "1 an"),
      KeyExpiry::TwoYears => write!(f, "2 ans"),
      KeyExpiry::FiveYears => write!(f, "5 ans"),
    }
  }
}

#[derive(Debug, Clone)]
pub struct SubkeyInfo {
  pub fingerprint: String,
  pub short_id: String,
  pub algo: String,
  pub usage: String,
  pub expires: Option<String>,
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
  pub subkeys: Vec<SubkeyInfo>,
}

#[derive(Debug, Clone)]
pub struct CardInfo {
  pub serial: String,
  pub sig_fp: Option<String>,
  pub enc_fp: Option<String>,
  pub auth_fp: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Keyserver {
  #[default]
  Openpgp,
  Ubuntu,
}

impl Keyserver {
  pub fn url(&self) -> &'static str {
    match self {
      Self::Openpgp => "keys.openpgp.org",
      Self::Ubuntu => "keyserver.ubuntu.com",
    }
  }
}

pub fn format_date(t: std::time::SystemTime) -> String {
  let dt: chrono::DateTime<Utc> = t.into();
  dt.format("%Y-%m-%d").to_string()
}
