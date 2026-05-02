use std::collections::HashMap;

use iced::widget::text_editor;
use iced::Task;

use crate::gpg::{HealthCheck, KeyExpiry, KeyInfo, Keyserver};
use crate::ui;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum View {
  #[default]
  MyKeys,
  PublicKeys,
  CreateKey,
  Import,
  Health,
}

pub struct ImportForm {
  pub url: String,
  pub keyserver_query: String,
  pub keyserver: Keyserver,
  pub pasted_key: text_editor::Content,
  pub submitting: bool,
}

impl Default for ImportForm {
  fn default() -> Self {
    Self {
      url: String::new(),
      keyserver_query: String::new(),
      keyserver: Keyserver::default(),
      pasted_key: text_editor::Content::new(),
      submitting: false,
    }
  }
}

#[derive(Default)]
pub struct CreateKeyForm {
  pub name: String,
  pub email: String,
  pub subkey_expiry: KeyExpiry,
  pub include_auth: bool,
  pub submitting: bool,
}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum KeyserverStatus {
  #[default]
  Unknown,
  Checking,
  Published,
  NotPublished,
}

pub struct PendingRenewal {
  pub key_idx: usize,
  pub subkey_idx: usize,
  pub expiry: KeyExpiry,
}

#[derive(Default)]
pub struct App {
  pub view: View,
  pub keys: Vec<KeyInfo>,
  pub selected: Option<usize>,
  pub error: Option<String>,
  pub status: Option<String>,
  pub loading: bool,
  pub card_connected: bool,
  pub pending_migration: Option<usize>,
  pub pending_delete: Option<usize>,
  pub pending_renewal: Option<PendingRenewal>,
  pub pending_export_pub: Option<usize>,
  pub pending_publish: Option<Keyserver>,
  pub keyserver_statuses: HashMap<String, KeyserverStatus>,
  pub create_form: CreateKeyForm,
  pub import_form: ImportForm,
  pub health_report: Vec<HealthCheck>,
  pub health_loading: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
  KeysLoaded(Result<(Vec<KeyInfo>, bool), String>),
  NavChanged(View),
  KeySelected(usize),
  ExportPublicKeyMenu(usize),
  ExportPublicKeyMenuCancel,
  ExportPublicKey(usize),
  ExportPublicKeyClipboard(usize),
  ExportPublicKeyClipboardDone(Result<String, String>),
  ExportPublicKeyUpload(usize),
  ExportPublicKeyUploadDone(Result<String, String>),
  BackupKey(usize),
  BackupDone(Result<Option<String>, String>),
  ExportDone(Result<Option<String>, String>),
  CreateKeyNameChanged(String),
  CreateKeyEmailChanged(String),
  CreateKeySubkeyExpiryChanged(KeyExpiry),
  CreateKeyIncludeAuthToggled(bool),
  CreateKeySubmit,
  CreateKeyDone(Result<(), String>),
  ImportKey,
  ImportKeyDone(Result<Option<String>, String>),
  ImportUrlChanged(String),
  ImportFromUrl,
  ImportFromUrlDone(Result<(), String>),
  ImportKeyserverQueryChanged(String),
  ImportKeyserverChanged(Keyserver),
  ImportFromKeyserver,
  ImportFromKeyserverDone(Result<(), String>),
  ImportPastedKeyChanged(text_editor::Action),
  ImportFromPaste,
  ImportFromPasteDone(Result<(), String>),
  HealthChecksLoaded(Vec<HealthCheck>),
  MoveToCard(usize),
  MoveToCardCancel,
  MoveToCardExecute(usize),
  MoveToCardDone(Result<(), String>),
  DeleteKey(usize),
  DeleteKeyCancel,
  DeleteKeyExecute(usize),
  DeleteKeyDone(Result<(), String>),
  CopyToClipboard(String),
  KeyserverStatusLoaded(Result<(String, bool), String>),
  PublishKey,
  PublishKeyserverChanged(Keyserver),
  PublishKeyExecute(usize),
  PublishKeyCancel,
  PublishKeyDone(Result<String, String>),
  AutoRepublishDone(Result<(), String>),
  RotateSubkeyExecute(usize, usize),
  RotateSubkeyDone(Result<(), String>),
  AddSubkey(usize, crate::gpg::SubkeyType),
  AddSubkeyDone(Result<(), String>),
  RenewSubkey(usize, usize),
  RenewSubkeyExpiryChanged(KeyExpiry),
  RenewSubkeyExecute,
  RenewSubkeyCancel,
  RenewSubkeyDone(Result<(), String>),
}

async fn blocking_task<T, F>(f: F) -> Result<T, String>
where
  T: Send + 'static,
  F: FnOnce() -> anyhow::Result<T> + Send + 'static,
{
  tokio::task::spawn_blocking(f)
    .await
    .unwrap_or_else(|e| Err(anyhow::anyhow!(e)))
    .map_err(|e| e.to_string())
}

fn export_key_to_file(fp: String, name: String) -> anyhow::Result<Option<String>> {
  let path = match rfd::FileDialog::new()
    .set_file_name(format!("{name}.pub.asc"))
    .add_filter("PGP Key", &["asc"])
    .save_file()
  {
    None => return Ok(None),
    Some(p) => p,
  };
  let filename = path
    .file_name()
    .and_then(|n| n.to_str())
    .unwrap_or("key.asc")
    .to_string();
  crate::gpg::export_public_key(&fp, &path)?;
  Ok(Some(filename))
}

fn backup_key_to_dir(fp: String, short_id: String) -> anyhow::Result<Option<String>> {
  let dir = match rfd::FileDialog::new()
    .set_title("Choisir un dossier de sauvegarde")
    .pick_folder()
  {
    None => return Ok(None),
    Some(d) => d,
  };
  let (key_file, rev_file) = crate::gpg::backup_key(&fp, &dir, &short_id)?;
  let summary = match rev_file {
    Some(rev) => format!("{key_file} + {rev}"),
    None => format!("{key_file} (certificat de révocation introuvable)"),
  };
  Ok(Some(summary))
}

impl App {
  pub fn new() -> (Self, Task<Message>) {
    let task = Task::perform(blocking_task(crate::gpg::list_keys), Message::KeysLoaded);
    (
      Self {
        loading: true,
        ..Default::default()
      },
      task,
    )
  }

  fn reload_keys(&mut self) -> Task<Message> {
    self.loading = true;
    Task::perform(blocking_task(crate::gpg::list_keys), Message::KeysLoaded)
  }

  fn reset_pending_ops(&mut self) {
    self.status = None;
    self.pending_migration = None;
    self.pending_delete = None;
    self.pending_renewal = None;
    self.pending_export_pub = None;
    self.pending_publish = None;
  }

  fn auto_republish_task(&self, key_idx: usize) -> Option<Task<Message>> {
    let fp = self.keys[key_idx].fingerprint.clone();
    if self.keyserver_statuses.get(&fp) == Some(&KeyserverStatus::Published) {
      Some(Task::perform(
        blocking_task(move || crate::gpg::publish_key(&fp, "keys.openpgp.org").map(|_| ())),
        Message::AutoRepublishDone,
      ))
    } else {
      None
    }
  }

  pub fn update(&mut self, message: Message) -> Task<Message> {
    match message {
      // Trivial field updates — kept inline
      Message::CreateKeyNameChanged(v) => {
        self.create_form.name = v;
        Task::none()
      }
      Message::CreateKeyEmailChanged(v) => {
        self.create_form.email = v;
        Task::none()
      }
      Message::CreateKeySubkeyExpiryChanged(v) => {
        self.create_form.subkey_expiry = v;
        Task::none()
      }
      Message::CreateKeyIncludeAuthToggled(v) => {
        self.create_form.include_auth = v;
        Task::none()
      }
      Message::ImportUrlChanged(v) => {
        self.import_form.url = v;
        Task::none()
      }
      Message::ImportKeyserverQueryChanged(v) => {
        self.import_form.keyserver_query = v;
        Task::none()
      }
      Message::ImportKeyserverChanged(ks) => {
        self.import_form.keyserver = ks;
        Task::none()
      }
      Message::ImportPastedKeyChanged(action) => {
        self.import_form.pasted_key.perform(action);
        Task::none()
      }
      Message::PublishKeyserverChanged(ks) => {
        self.pending_publish = Some(ks);
        Task::none()
      }
      Message::ExportPublicKeyMenuCancel => {
        self.pending_export_pub = None;
        Task::none()
      }
      Message::MoveToCardCancel => {
        self.pending_migration = None;
        Task::none()
      }
      Message::DeleteKeyCancel => {
        self.pending_delete = None;
        Task::none()
      }
      Message::RenewSubkeyCancel => {
        self.pending_renewal = None;
        Task::none()
      }
      Message::RenewSubkeyExpiryChanged(e) => {
        if let Some(ref mut r) = self.pending_renewal {
          r.expiry = e;
        }
        Task::none()
      }
      Message::HealthChecksLoaded(checks) => {
        self.health_report = checks;
        self.health_loading = false;
        Task::none()
      }
      // Delegated handlers
      Message::KeysLoaded(r) => self.on_keys_loaded(r),
      Message::NavChanged(v) => self.on_nav_changed(v),
      Message::KeySelected(i) => self.on_key_selected(i),
      Message::KeyserverStatusLoaded(r) => self.on_keyserver_status_loaded(r),
      Message::ExportPublicKeyMenu(i) => self.on_export_pub_menu(i),
      Message::ExportPublicKey(i) => self.on_export_public(i),
      Message::ExportPublicKeyClipboard(i) => self.on_export_clipboard(i),
      Message::ExportPublicKeyClipboardDone(r) => self.on_export_clipboard_done(r),
      Message::ExportPublicKeyUpload(i) => self.on_export_upload(i),
      Message::ExportPublicKeyUploadDone(r) => self.on_export_upload_done(r),
      Message::BackupKey(i) => self.on_backup_key(i),
      Message::BackupDone(r) => self.on_backup_done(r),
      Message::ExportDone(r) => self.on_export_done(r),
      Message::CreateKeySubmit => self.on_create_key_submit(),
      Message::CreateKeyDone(r) => self.on_create_key_done(r),
      Message::ImportKey => self.on_import_key(),
      Message::ImportKeyDone(r) => self.on_import_key_done(r),
      Message::ImportFromUrl => self.on_import_from_url(),
      Message::ImportFromUrlDone(r) => self.on_import_from_url_done(r),
      Message::ImportFromKeyserver => self.on_import_from_keyserver(),
      Message::ImportFromKeyserverDone(r) => self.on_import_from_keyserver_done(r),
      Message::ImportFromPaste => self.on_import_from_paste(),
      Message::ImportFromPasteDone(r) => self.on_import_from_paste_done(r),
      Message::MoveToCard(i) => self.on_move_to_card(i),
      Message::MoveToCardExecute(i) => self.on_move_to_card_execute(i),
      Message::MoveToCardDone(r) => self.on_move_to_card_done(r),
      Message::DeleteKey(i) => self.on_delete_key(i),
      Message::DeleteKeyExecute(i) => self.on_delete_key_execute(i),
      Message::DeleteKeyDone(r) => self.on_delete_key_done(r),
      Message::CopyToClipboard(text) => self.on_copy_to_clipboard(text),
      Message::RenewSubkey(ki, si) => self.on_renew_subkey(ki, si),
      Message::RenewSubkeyExecute => self.on_renew_subkey_execute(),
      Message::RenewSubkeyDone(r) => self.on_renew_subkey_done(r),
      Message::AddSubkey(ki, st) => self.on_add_subkey(ki, st),
      Message::AddSubkeyDone(r) => self.on_add_subkey_done(r),
      Message::RotateSubkeyExecute(ki, si) => self.on_rotate_subkey_execute(ki, si),
      Message::RotateSubkeyDone(r) => self.on_rotate_subkey_done(r),
      Message::PublishKey => self.on_publish_key(),
      Message::PublishKeyExecute(i) => self.on_publish_key_execute(i),
      Message::PublishKeyCancel => self.on_publish_key_cancel(),
      Message::PublishKeyDone(r) => self.on_publish_key_done(r),
      Message::AutoRepublishDone(r) => self.on_auto_republish_done(r),
    }
  }

  // --- Navigation ---

  fn on_keys_loaded(&mut self, result: Result<(Vec<KeyInfo>, bool), String>) -> Task<Message> {
    match result {
      Ok((keys, card_connected)) => {
        self.keys = keys;
        self.card_connected = card_connected;
        self.loading = false;
        let new_fps: Vec<String> = self
          .keys
          .iter()
          .filter(|k| !self.keyserver_statuses.contains_key(&k.fingerprint))
          .map(|k| k.fingerprint.clone())
          .collect();
        for fp in &new_fps {
          self
            .keyserver_statuses
            .insert(fp.clone(), KeyserverStatus::Checking);
        }
        if !new_fps.is_empty() {
          return Task::batch(new_fps.into_iter().map(|fp| {
            Task::perform(
              blocking_task(move || crate::gpg::check_keyserver(&fp)),
              Message::KeyserverStatusLoaded,
            )
          }));
        }
        Task::none()
      }
      Err(e) => {
        self.error = Some(e);
        self.loading = false;
        Task::none()
      }
    }
  }

  fn on_nav_changed(&mut self, view: View) -> Task<Message> {
    let is_health = view == View::Health;
    self.view = view;
    self.selected = None;
    self.reset_pending_ops();
    if is_health {
      self.health_loading = true;
      let keys = self.keys.clone();
      return Task::perform(
        blocking_task(move || Ok(crate::gpg::run_all_checks(&keys))),
        |r| Message::HealthChecksLoaded(r.unwrap_or_default()),
      );
    }
    Task::none()
  }

  fn on_key_selected(&mut self, i: usize) -> Task<Message> {
    self.selected = Some(i);
    self.reset_pending_ops();
    let fp = self.keys[i].fingerprint.clone();
    let unknown = matches!(
      self.keyserver_statuses.get(&fp),
      None | Some(KeyserverStatus::Unknown)
    );
    if unknown {
      self
        .keyserver_statuses
        .insert(fp.clone(), KeyserverStatus::Checking);
      return Task::perform(
        blocking_task(move || crate::gpg::check_keyserver(&fp)),
        Message::KeyserverStatusLoaded,
      );
    }
    Task::none()
  }

  fn on_keyserver_status_loaded(
    &mut self,
    result: Result<(String, bool), String>,
  ) -> Task<Message> {
    match result {
      Ok((fp, found)) => {
        self.keyserver_statuses.insert(
          fp,
          if found {
            KeyserverStatus::Published
          } else {
            KeyserverStatus::NotPublished
          },
        );
      }
      Err(_) => {
        for status in self.keyserver_statuses.values_mut() {
          if *status == KeyserverStatus::Checking {
            *status = KeyserverStatus::Unknown;
          }
        }
      }
    }
    Task::none()
  }

  // --- Export / Backup ---

  fn on_export_pub_menu(&mut self, i: usize) -> Task<Message> {
    self.reset_pending_ops();
    self.pending_export_pub = Some(i);
    Task::none()
  }

  fn on_export_public(&mut self, i: usize) -> Task<Message> {
    self.pending_export_pub = None;
    let fp = self.keys[i].fingerprint.clone();
    let name = self.keys[i].name.replace(' ', "_");
    Task::perform(
      blocking_task(move || export_key_to_file(fp, name)),
      Message::ExportDone,
    )
  }

  fn on_export_clipboard(&mut self, i: usize) -> Task<Message> {
    self.pending_export_pub = None;
    let fp = self.keys[i].fingerprint.clone();
    Task::perform(
      blocking_task(move || crate::gpg::export_public_key_armored(&fp)),
      Message::ExportPublicKeyClipboardDone,
    )
  }

  fn on_export_clipboard_done(&mut self, result: Result<String, String>) -> Task<Message> {
    match result {
      Ok(armored) => {
        self.status = Some("Clef copiée dans le presse-papier".to_string());
        iced::clipboard::write(armored)
      }
      Err(e) => {
        self.status = Some(format!("Erreur : {e}"));
        Task::none()
      }
    }
  }

  fn on_export_upload(&mut self, i: usize) -> Task<Message> {
    self.pending_export_pub = None;
    let fp = self.keys[i].fingerprint.clone();
    Task::perform(
      blocking_task(move || crate::gpg::upload_public_key(&fp)),
      Message::ExportPublicKeyUploadDone,
    )
  }

  fn on_export_upload_done(&mut self, result: Result<String, String>) -> Task<Message> {
    match result {
      Ok(url) => {
        self.status = Some(format!("Lien copié : {url}"));
        iced::clipboard::write(url)
      }
      Err(e) => {
        self.status = Some(format!("Erreur upload : {e}"));
        Task::none()
      }
    }
  }

  fn on_backup_key(&mut self, i: usize) -> Task<Message> {
    let fp = self.keys[i].fingerprint.clone();
    let short_id = self.keys[i].short_id.clone();
    Task::perform(
      blocking_task(move || backup_key_to_dir(fp, short_id)),
      Message::BackupDone,
    )
  }

  fn on_backup_done(&mut self, result: Result<Option<String>, String>) -> Task<Message> {
    match result {
      Ok(None) => {}
      Ok(Some(summary)) => self.status = Some(format!("Sauvegardé : {summary}")),
      Err(e) => self.status = Some(format!("Erreur : {e}")),
    }
    Task::none()
  }

  fn on_export_done(&mut self, result: Result<Option<String>, String>) -> Task<Message> {
    match result {
      Ok(None) => {}
      Ok(Some(filename)) => self.status = Some(format!("Exporté : {filename}")),
      Err(e) => self.status = Some(format!("Erreur : {e}")),
    }
    Task::none()
  }

  // --- Create key ---

  fn on_create_key_submit(&mut self) -> Task<Message> {
    let name = self.create_form.name.clone();
    let email = self.create_form.email.clone();
    let subkey_expiry = self.create_form.subkey_expiry.clone();
    let include_auth = self.create_form.include_auth;
    self.create_form.submitting = true;
    Task::perform(
      blocking_task(move || crate::gpg::create_key(&name, &email, &subkey_expiry, include_auth)),
      Message::CreateKeyDone,
    )
  }

  fn on_create_key_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        self.view = View::MyKeys;
        self.create_form = CreateKeyForm::default();
        self.selected = None;
        self.reload_keys()
      }
      Err(e) => {
        self.create_form.submitting = false;
        self.status = Some(format!("Erreur : {e}"));
        Task::none()
      }
    }
  }

  // --- Import ---

  fn on_import_key(&mut self) -> Task<Message> {
    Task::perform(
      blocking_task(|| {
        let path = match rfd::FileDialog::new()
          .add_filter("PGP Key", &["asc", "gpg", "key"])
          .pick_file()
        {
          None => return Ok(None),
          Some(p) => p,
        };
        let filename = path
          .file_name()
          .and_then(|n| n.to_str())
          .unwrap_or("fichier")
          .to_string();
        crate::gpg::import_key(&path)?;
        Ok(Some(filename))
      }),
      Message::ImportKeyDone,
    )
  }

  fn on_import_key_done(&mut self, result: Result<Option<String>, String>) -> Task<Message> {
    match result {
      Ok(None) => Task::none(),
      Ok(Some(filename)) => {
        self.status = Some(format!("Clef importée depuis {filename}"));
        self.view = View::PublicKeys;
        self.selected = None;
        self.reload_keys()
      }
      Err(e) => {
        self.status = Some(format!("Erreur import : {e}"));
        Task::none()
      }
    }
  }

  fn on_import_from_url(&mut self) -> Task<Message> {
    self.import_form.submitting = true;
    let url = self.import_form.url.clone();
    Task::perform(
      blocking_task(move || crate::gpg::import_key_from_url(&url)),
      Message::ImportFromUrlDone,
    )
  }

  fn on_import_from_url_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        self.import_form = ImportForm::default();
        self.view = View::PublicKeys;
        self.selected = None;
        self.reload_keys()
      }
      Err(e) => {
        self.import_form.submitting = false;
        self.status = Some(format!("Erreur import URL : {e}"));
        Task::none()
      }
    }
  }

  fn on_import_from_keyserver(&mut self) -> Task<Message> {
    self.import_form.submitting = true;
    let query = self.import_form.keyserver_query.clone();
    let url = self.import_form.keyserver.url().to_string();
    Task::perform(
      blocking_task(move || crate::gpg::import_key_from_keyserver(&query, &url)),
      Message::ImportFromKeyserverDone,
    )
  }

  fn on_import_from_keyserver_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        self.import_form = ImportForm::default();
        self.view = View::PublicKeys;
        self.selected = None;
        self.reload_keys()
      }
      Err(e) => {
        self.import_form.submitting = false;
        self.status = Some(format!("Erreur import keyserver : {e}"));
        Task::none()
      }
    }
  }

  fn on_import_from_paste(&mut self) -> Task<Message> {
    self.import_form.submitting = true;
    let content = self.import_form.pasted_key.text();
    Task::perform(
      blocking_task(move || crate::gpg::import_key_from_text(&content)),
      Message::ImportFromPasteDone,
    )
  }

  fn on_import_from_paste_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        self.import_form = ImportForm::default();
        self.view = View::PublicKeys;
        self.selected = None;
        self.reload_keys()
      }
      Err(e) => {
        self.import_form.submitting = false;
        self.status = Some(format!("Erreur import : {e}"));
        Task::none()
      }
    }
  }

  // --- Key operations (YubiKey, delete, clipboard) ---

  fn on_move_to_card(&mut self, i: usize) -> Task<Message> {
    self.reset_pending_ops();
    self.pending_migration = Some(i);
    Task::none()
  }

  fn on_move_to_card_execute(&mut self, i: usize) -> Task<Message> {
    self.pending_migration = None;
    let fp = self.keys[i].fingerprint.clone();
    Task::perform(
      blocking_task(move || crate::gpg::move_key_to_card(&fp)),
      Message::MoveToCardDone,
    )
  }

  fn on_move_to_card_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        self.status = Some("Clef migrée sur YubiKey avec succès".to_string());
        self.selected = None;
        self.reload_keys()
      }
      Err(e) => {
        self.status = Some(format!("Erreur migration : {e}"));
        Task::none()
      }
    }
  }

  fn on_delete_key(&mut self, i: usize) -> Task<Message> {
    self.reset_pending_ops();
    self.pending_delete = Some(i);
    Task::none()
  }

  fn on_delete_key_execute(&mut self, i: usize) -> Task<Message> {
    self.pending_delete = None;
    let fp = self.keys[i].fingerprint.clone();
    let has_secret = self.keys[i].has_secret || self.keys[i].on_card;
    Task::perform(
      blocking_task(move || crate::gpg::delete_key(&fp, has_secret)),
      Message::DeleteKeyDone,
    )
  }

  fn on_delete_key_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        self.status = Some("Clef supprimée".to_string());
        self.selected = None;
        self.reload_keys()
      }
      Err(e) => {
        self.status = Some(format!("Erreur suppression : {e}"));
        Task::none()
      }
    }
  }

  fn on_copy_to_clipboard(&mut self, text: String) -> Task<Message> {
    self.status = Some("Copié dans le presse-papier".to_string());
    iced::clipboard::write(text)
  }

  // --- Subkeys ---

  fn on_renew_subkey(&mut self, key_idx: usize, subkey_idx: usize) -> Task<Message> {
    self.reset_pending_ops();
    self.pending_renewal = Some(PendingRenewal {
      key_idx,
      subkey_idx,
      expiry: KeyExpiry::TwoYears,
    });
    Task::none()
  }

  fn on_renew_subkey_execute(&mut self) -> Task<Message> {
    if let Some(renewal) = self.pending_renewal.take() {
      let master_fp = self.keys[renewal.key_idx].fingerprint.clone();
      let subkey_fp = self.keys[renewal.key_idx].subkeys[renewal.subkey_idx]
        .fingerprint
        .clone();
      let expiry = renewal.expiry;
      return Task::perform(
        blocking_task(move || crate::gpg::renew_subkey(&master_fp, &subkey_fp, &expiry)),
        Message::RenewSubkeyDone,
      );
    }
    Task::none()
  }

  fn on_renew_subkey_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        self.status = Some("Sous-clef renouvelée".to_string());
        let reload = self.reload_keys();
        if let Some(i) = self.selected {
          if let Some(publish) = self.auto_republish_task(i) {
            return Task::batch([reload, publish]);
          }
        }
        reload
      }
      Err(e) => {
        self.status = Some(format!("Erreur renouvellement : {e}"));
        Task::none()
      }
    }
  }

  fn on_add_subkey(
    &mut self,
    key_idx: usize,
    subkey_type: crate::gpg::SubkeyType,
  ) -> Task<Message> {
    let master_fp = self.keys[key_idx].fingerprint.clone();
    Task::perform(
      blocking_task(move || {
        crate::gpg::add_subkey(
          &master_fp,
          subkey_type.algo(),
          subkey_type.usage(),
          &KeyExpiry::TwoYears,
        )
      }),
      Message::AddSubkeyDone,
    )
  }

  fn on_add_subkey_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        self.status = Some("Sous-clef créée".to_string());
        let reload = self.reload_keys();
        if let Some(i) = self.selected {
          if let Some(publish) = self.auto_republish_task(i) {
            return Task::batch([reload, publish]);
          }
        }
        reload
      }
      Err(e) => {
        self.status = Some(format!("Erreur création sous-clef : {e}"));
        Task::none()
      }
    }
  }

  fn on_rotate_subkey_execute(&mut self, key_idx: usize, subkey_idx: usize) -> Task<Message> {
    let expiry = self
      .pending_renewal
      .take()
      .map(|r| r.expiry)
      .unwrap_or_default();
    let master_fp = self.keys[key_idx].fingerprint.clone();
    let old_fp = self.keys[key_idx].subkeys[subkey_idx].fingerprint.clone();
    let subkey_type =
      crate::gpg::SubkeyType::from_usage_flags(&self.keys[key_idx].subkeys[subkey_idx].usage);
    Task::perform(
      blocking_task(move || {
        crate::gpg::rotate_subkey(
          &master_fp,
          &old_fp,
          subkey_type.algo(),
          subkey_type.usage(),
          &expiry,
        )
      }),
      Message::RotateSubkeyDone,
    )
  }

  fn on_rotate_subkey_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        self.status = Some("Sous-clef remplacée avec succès".to_string());
        let reload = self.reload_keys();
        if let Some(i) = self.selected {
          if let Some(publish) = self.auto_republish_task(i) {
            return Task::batch([reload, publish]);
          }
        }
        reload
      }
      Err(e) => {
        self.status = Some(format!("Erreur rotation : {e}"));
        Task::none()
      }
    }
  }

  // --- Keyserver ---

  fn on_publish_key(&mut self) -> Task<Message> {
    self.reset_pending_ops();
    self.pending_publish = Some(Keyserver::default());
    Task::none()
  }

  fn on_publish_key_execute(&mut self, i: usize) -> Task<Message> {
    let keyserver = self.pending_publish.take().unwrap_or_default();
    let fp = self.keys[i].fingerprint.clone();
    let url = keyserver.url().to_string();
    Task::perform(
      blocking_task(move || crate::gpg::publish_key(&fp, &url)),
      Message::PublishKeyDone,
    )
  }

  fn on_publish_key_cancel(&mut self) -> Task<Message> {
    self.pending_export_pub = None;
    self.pending_publish = None;
    Task::none()
  }

  fn on_publish_key_done(&mut self, result: Result<String, String>) -> Task<Message> {
    match result {
      Ok(url) => {
        self.status = if url == "keys.openpgp.org" {
          Some(
            "Clef publiée. Vérifiez votre email pour valider la publication sur keys.openpgp.org."
              .to_string(),
          )
        } else {
          Some("Clef publiée avec succès.".to_string())
        };
        if let Some(i) = self.selected {
          let fp = self.keys[i].fingerprint.clone();
          self
            .keyserver_statuses
            .insert(fp.clone(), KeyserverStatus::Checking);
          return Task::perform(
            blocking_task(move || crate::gpg::check_keyserver(&fp)),
            Message::KeyserverStatusLoaded,
          );
        }
        Task::none()
      }
      Err(e) => {
        self.status = Some(format!("Erreur publication : {e}"));
        Task::none()
      }
    }
  }

  fn on_auto_republish_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        if let Some(i) = self.selected {
          let fp = self.keys[i].fingerprint.clone();
          self
            .keyserver_statuses
            .insert(fp.clone(), KeyserverStatus::Checking);
          return Task::perform(
            blocking_task(move || crate::gpg::check_keyserver(&fp)),
            Message::KeyserverStatusLoaded,
          );
        }
        Task::none()
      }
      Err(e) => {
        self.status = Some(format!("Erreur republication : {e}"));
        Task::none()
      }
    }
  }

  pub fn view(&self) -> iced::Element<'_, Message> {
    ui::root(self)
  }
}
