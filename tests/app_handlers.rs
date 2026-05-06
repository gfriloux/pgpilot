mod common;
use pgpilot::app::{
  App, CreateKeyForm, DecryptForm, EncryptForm, ImportForm, PendingOp, SignForm, StatusKind, View,
};

fn make_test_app() -> App {
  use pgpilot::config::Config;
  use pgpilot::i18n::{self, Language};
  let config = Config::default();
  let strings = i18n::strings_for(Language::English);
  App {
    view: View::MyKeys,
    keys: Vec::new(),
    selected: None,
    error: None,
    status: None,
    status_generation: 0,
    loading: false,
    card_connected: false,
    pending: None,
    keyserver_statuses: std::collections::HashMap::new(),
    create_form: CreateKeyForm::default(),
    import_form: ImportForm::default(),
    encrypt_form: EncryptForm::default(),
    decrypt_form: DecryptForm::default(),
    health_report: Vec::new(),
    health_loading: false,
    sign_form: SignForm::default(),
    previous_view: None,
    expiry_warnings: Vec::new(),
    config,
    strings,
    // Chat fields
    rooms: Vec::new(),
    active_room: None,
    chat_messages: std::collections::HashMap::new(),
    presence: pgpilot::chat::PresenceTracker::new(),
    mqtt_state: pgpilot::app::MqttState::Disconnected,
    mqtt: None,
    chat_input: String::new(),
    chat_new_form: pgpilot::app::ChatNewForm::default(),
    chat_crypto: None,
    chat_identity_popup: None,
  }
}

#[test]
fn key_by_fp_returns_none_when_empty() {
  let app = make_test_app();
  let fp = "A".repeat(40);
  // With an empty key list, no key should be found by fingerprint
  assert!(app.keys.iter().find(|k| k.fingerprint == fp).is_none());
}

#[test]
fn reset_pending_ops_clears_pending_and_status() {
  let mut app = make_test_app();
  app.pending = Some(PendingOp::Delete("A".repeat(40)));
  app.status = Some((StatusKind::Error, "some error".to_string()));
  // NavChanged triggers reset_pending_ops internally
  let _ = app.update(pgpilot::app::Message::NavChanged(View::MyKeys));
  assert!(app.pending.is_none());
  assert!(app.status.is_none());
}

#[test]
fn nav_to_create_key_saves_previous_view() {
  let mut app = make_test_app();
  app.view = View::MyKeys;
  let _ = app.update(pgpilot::app::Message::NavChanged(View::CreateKey));
  assert_eq!(app.previous_view, Some(View::MyKeys));
}

#[test]
fn nav_to_import_saves_previous_view() {
  let mut app = make_test_app();
  app.view = View::PublicKeys;
  let _ = app.update(pgpilot::app::Message::NavChanged(View::Import));
  assert_eq!(app.previous_view, Some(View::PublicKeys));
}

#[test]
fn nav_to_other_view_does_not_save_previous() {
  let mut app = make_test_app();
  app.view = View::MyKeys;
  app.previous_view = Some(View::CreateKey); // preset
  let _ = app.update(pgpilot::app::Message::NavChanged(View::Encrypt));
  // previous_view should not change when navigating to Encrypt
  assert_eq!(app.previous_view, Some(View::CreateKey));
}

#[test]
fn key_selection_stores_fingerprint() {
  let mut app = make_test_app();
  let fp = "A".repeat(40);
  let _ = app.update(pgpilot::app::Message::KeySelected(fp.clone()));
  assert_eq!(app.selected, Some(fp));
}
