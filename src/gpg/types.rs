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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone)]
pub struct ExpiryWarning {
  pub key_fp: String,
  pub key_name: String,
  pub subkey_type: Option<SubkeyType>,
  pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn trust_level_is_sufficient() {
    assert!(!TrustLevel::Undefined.is_sufficient());
    assert!(!TrustLevel::Marginal.is_sufficient());
    assert!(TrustLevel::Full.is_sufficient());
    assert!(TrustLevel::Ultimate.is_sufficient());
  }

  #[test]
  fn subkey_type_from_usage_flags() {
    // Production code stores usage as uppercase chars: 'S', 'E', 'A'
    // (populated by keyring.rs from sequoia key_flags)
    assert_eq!(SubkeyType::from_usage_flags("S"), SubkeyType::Sign);
    assert_eq!(SubkeyType::from_usage_flags("E"), SubkeyType::Encr);
    assert_eq!(SubkeyType::from_usage_flags("A"), SubkeyType::Auth);
    // Multi-flag strings: E takes priority over A, A over S
    assert_eq!(SubkeyType::from_usage_flags("SE"), SubkeyType::Encr);
    assert_eq!(SubkeyType::from_usage_flags("SA"), SubkeyType::Auth);
    // Default to Sign if none of E or A are present
    assert_eq!(SubkeyType::from_usage_flags("C"), SubkeyType::Sign);
    assert_eq!(SubkeyType::from_usage_flags(""), SubkeyType::Sign);
  }

  #[test]
  fn verify_outcome_variants_not_equal() {
    assert_ne!(VerifyOutcome::Valid, VerifyOutcome::BadSig);
    assert_ne!(VerifyOutcome::UnknownKey, VerifyOutcome::ExpiredKey);
  }

  #[test]
  fn subkey_type_algo() {
    assert_eq!(SubkeyType::Sign.algo(), "ed25519");
    assert_eq!(SubkeyType::Auth.algo(), "ed25519");
    assert_eq!(SubkeyType::Encr.algo(), "cv25519");
  }

  #[test]
  fn subkey_type_usage() {
    assert_eq!(SubkeyType::Sign.usage(), "sign");
    assert_eq!(SubkeyType::Encr.usage(), "encr");
    assert_eq!(SubkeyType::Auth.usage(), "auth");
  }

  #[test]
  fn trust_level_from_char() {
    assert_eq!(TrustLevel::from_char('u'), TrustLevel::Ultimate);
    assert_eq!(TrustLevel::from_char('f'), TrustLevel::Full);
    assert_eq!(TrustLevel::from_char('m'), TrustLevel::Marginal);
    assert_eq!(TrustLevel::from_char('x'), TrustLevel::Undefined);
  }
}
