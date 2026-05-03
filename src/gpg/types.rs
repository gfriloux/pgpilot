use chrono::Utc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecryptStatus {
  CanDecrypt,
  NoKey,
  Checking,
  Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubkeyType {
  Sign,
  Encr,
  Auth,
}

impl SubkeyType {
  pub fn algo(self) -> &'static str {
    match self {
      Self::Sign | Self::Auth => "ed25519",
      Self::Encr => "cv25519",
    }
  }

  pub fn usage(self) -> &'static str {
    match self {
      Self::Sign => "sign",
      Self::Encr => "encr",
      Self::Auth => "auth",
    }
  }

  pub fn usage_char(self) -> char {
    match self {
      Self::Sign => 'S',
      Self::Encr => 'E',
      Self::Auth => 'A',
    }
  }

  pub fn from_usage_flags(flags: &str) -> Self {
    if flags.contains('E') {
      Self::Encr
    } else if flags.contains('A') {
      Self::Auth
    } else {
      Self::Sign
    }
  }
}

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
  pub key_id: String,
  pub algo: String,
  pub usage: String,
  pub expires: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TrustLevel {
  #[default]
  Undefined,
  Marginal,
  Full,
  Ultimate,
}

impl TrustLevel {
  pub fn is_sufficient(&self) -> bool {
    matches!(self, TrustLevel::Full | TrustLevel::Ultimate)
  }

  pub(crate) fn from_char(c: char) -> Self {
    match c {
      'u' => TrustLevel::Ultimate,
      'f' => TrustLevel::Full,
      'm' => TrustLevel::Marginal,
      _ => TrustLevel::Undefined,
    }
  }
}

#[derive(Debug, Clone)]
pub struct KeyInfo {
  pub fingerprint: String,
  pub key_id: String,
  pub name: String,
  pub email: String,
  pub algo: String,
  pub created: String,
  pub expires: Option<String>,
  pub has_secret: bool,
  pub on_card: bool,
  pub card_serial: Option<String>,
  pub subkeys: Vec<SubkeyInfo>,
  pub trust: TrustLevel,
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

impl std::fmt::Display for Keyserver {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.url())
  }
}

pub fn format_date(t: std::time::SystemTime) -> String {
  let dt: chrono::DateTime<Utc> = t.into();
  dt.format("%Y-%m-%d").to_string()
}

#[derive(Debug, Clone)]
pub enum VerifyOutcome {
  Valid,
  BadSig,
  UnknownKey,
  ExpiredKey,
  RevokedKey,
  Error(String),
}

#[derive(Debug, Clone)]
pub struct VerifyResult {
  pub outcome: VerifyOutcome,
  pub signer_name: Option<String>,
  pub signer_fp: Option<String>,
  pub signed_at: Option<String>,
  pub detail: String,
  pub signer_trust: TrustLevel,
}
