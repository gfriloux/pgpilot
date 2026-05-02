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
  AddSubkey(usize, String, String),
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

  fn reload_keys() -> Task<Message> {
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
      Message::KeysLoaded(Ok((keys, card_connected))) => {
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
      }
      Message::KeysLoaded(Err(e)) => {
        self.error = Some(e);
        self.loading = false;
      }
      Message::NavChanged(view) => {
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
      }
      Message::KeySelected(i) => {
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
      }
      Message::ExportPublicKeyMenu(i) => {
        self.pending_export_pub = Some(i);
        self.pending_publish = None;
        self.pending_migration = None;
        self.pending_delete = None;
        self.pending_renewal = None;
        self.status = None;
      }
      Message::ExportPublicKeyMenuCancel => {
        self.pending_export_pub = None;
      }
      Message::ExportPublicKey(i) => {
        self.pending_export_pub = None;
        let fp = self.keys[i].fingerprint.clone();
        let name = self.keys[i].name.replace(' ', "_");
        return Task::perform(
          blocking_task(move || export_key_to_file(fp, name)),
          Message::ExportDone,
        );
      }
      Message::ExportPublicKeyClipboard(i) => {
        self.pending_export_pub = None;
        let fp = self.keys[i].fingerprint.clone();
        return Task::perform(
          blocking_task(move || crate::gpg::export_public_key_armored(&fp)),
          Message::ExportPublicKeyClipboardDone,
        );
      }
      Message::ExportPublicKeyClipboardDone(Ok(armored)) => {
        self.status = Some("Clef copiée dans le presse-papier".to_string());
        return iced::clipboard::write(armored);
      }
      Message::ExportPublicKeyClipboardDone(Err(e)) => {
        self.status = Some(format!("Erreur : {e}"));
      }
      Message::ExportPublicKeyUpload(i) => {
        self.pending_export_pub = None;
        let fp = self.keys[i].fingerprint.clone();
        return Task::perform(
          blocking_task(move || crate::gpg::upload_public_key(&fp)),
          Message::ExportPublicKeyUploadDone,
        );
      }
      Message::ExportPublicKeyUploadDone(Ok(url)) => {
        self.status = Some(format!("Lien copié : {url}"));
        return iced::clipboard::write(url);
      }
      Message::ExportPublicKeyUploadDone(Err(e)) => {
        self.status = Some(format!("Erreur upload : {e}"));
      }
      Message::BackupKey(i) => {
        let fp = self.keys[i].fingerprint.clone();
        let short_id = self.keys[i].short_id.clone();
        return Task::perform(
          blocking_task(move || backup_key_to_dir(fp, short_id)),
          Message::BackupDone,
        );
      }
      Message::BackupDone(Ok(None)) => {}
      Message::BackupDone(Ok(Some(summary))) => {
        self.status = Some(format!("Sauvegardé : {summary}"));
      }
      Message::BackupDone(Err(e)) => {
        self.status = Some(format!("Erreur : {e}"));
      }
      Message::ExportDone(Ok(None)) => {}
      Message::ExportDone(Ok(Some(filename))) => {
        self.status = Some(format!("Exporté : {filename}"));
      }
      Message::ExportDone(Err(e)) => {
        self.status = Some(format!("Erreur : {e}"));
      }
      Message::CreateKeyNameChanged(v) => self.create_form.name = v,
      Message::CreateKeyEmailChanged(v) => self.create_form.email = v,
      Message::CreateKeySubkeyExpiryChanged(v) => self.create_form.subkey_expiry = v,
      Message::CreateKeyIncludeAuthToggled(v) => self.create_form.include_auth = v,
      Message::CreateKeySubmit => {
        let name = self.create_form.name.clone();
        let email = self.create_form.email.clone();
        let subkey_expiry = self.create_form.subkey_expiry.clone();
        let include_auth = self.create_form.include_auth;
        self.create_form.submitting = true;
        return Task::perform(
          blocking_task(move || {
            crate::gpg::create_key(&name, &email, &subkey_expiry, include_auth)
          }),
          Message::CreateKeyDone,
        );
      }
      Message::CreateKeyDone(Ok(())) => {
        self.view = View::MyKeys;
        self.create_form = CreateKeyForm::default();
        self.loading = true;
        self.selected = None;
        return Self::reload_keys();
      }
      Message::CreateKeyDone(Err(e)) => {
        self.create_form.submitting = false;
        self.status = Some(format!("Erreur : {e}"));
      }
      Message::ImportKey => {
        return Task::perform(
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
        );
      }
      Message::ImportKeyDone(Ok(None)) => {}
      Message::ImportKeyDone(Ok(Some(filename))) => {
        self.status = Some(format!("Clef importée depuis {filename}"));
        self.view = View::PublicKeys;
        self.loading = true;
        self.selected = None;
        return Self::reload_keys();
      }
      Message::ImportKeyDone(Err(e)) => {
        self.status = Some(format!("Erreur import : {e}"));
      }
      Message::ImportUrlChanged(v) => self.import_form.url = v,
      Message::ImportFromUrl => {
        self.import_form.submitting = true;
        let url = self.import_form.url.clone();
        return Task::perform(
          blocking_task(move || crate::gpg::import_key_from_url(&url)),
          Message::ImportFromUrlDone,
        );
      }
      Message::ImportFromUrlDone(Ok(())) => {
        self.import_form = ImportForm::default();
        self.view = View::PublicKeys;
        self.loading = true;
        self.selected = None;
        return Self::reload_keys();
      }
      Message::ImportFromUrlDone(Err(e)) => {
        self.import_form.submitting = false;
        self.status = Some(format!("Erreur import URL : {e}"));
      }
      Message::ImportKeyserverQueryChanged(v) => self.import_form.keyserver_query = v,
      Message::ImportKeyserverChanged(ks) => self.import_form.keyserver = ks,
      Message::ImportFromKeyserver => {
        self.import_form.submitting = true;
        let query = self.import_form.keyserver_query.clone();
        let url = self.import_form.keyserver.url().to_string();
        return Task::perform(
          blocking_task(move || crate::gpg::import_key_from_keyserver(&query, &url)),
          Message::ImportFromKeyserverDone,
        );
      }
      Message::ImportFromKeyserverDone(Ok(())) => {
        self.import_form = ImportForm::default();
        self.view = View::PublicKeys;
        self.loading = true;
        self.selected = None;
        return Self::reload_keys();
      }
      Message::ImportFromKeyserverDone(Err(e)) => {
        self.import_form.submitting = false;
        self.status = Some(format!("Erreur import keyserver : {e}"));
      }
      Message::ImportPastedKeyChanged(action) => {
        self.import_form.pasted_key.perform(action);
      }
      Message::ImportFromPaste => {
        self.import_form.submitting = true;
        let content = self.import_form.pasted_key.text();
        return Task::perform(
          blocking_task(move || crate::gpg::import_key_from_text(&content)),
          Message::ImportFromPasteDone,
        );
      }
      Message::ImportFromPasteDone(Ok(())) => {
        self.import_form = ImportForm::default();
        self.view = View::PublicKeys;
        self.loading = true;
        self.selected = None;
        return Self::reload_keys();
      }
      Message::ImportFromPasteDone(Err(e)) => {
        self.import_form.submitting = false;
        self.status = Some(format!("Erreur import : {e}"));
      }
      Message::MoveToCard(i) => {
        self.pending_migration = Some(i);
        self.pending_delete = None;
        self.pending_renewal = None;
        self.pending_export_pub = None;
        self.pending_publish = None;
        self.status = None;
      }
      Message::MoveToCardCancel => {
        self.pending_migration = None;
      }
      Message::MoveToCardExecute(i) => {
        self.pending_migration = None;
        let fp = self.keys[i].fingerprint.clone();
        return Task::perform(
          blocking_task(move || crate::gpg::move_key_to_card(&fp)),
          Message::MoveToCardDone,
        );
      }
      Message::MoveToCardDone(Ok(())) => {
        self.status = Some("Clef migrée sur YubiKey avec succès".to_string());
        self.loading = true;
        self.selected = None;
        return Self::reload_keys();
      }
      Message::MoveToCardDone(Err(e)) => {
        self.status = Some(format!("Erreur migration : {e}"));
      }
      Message::DeleteKey(i) => {
        self.pending_delete = Some(i);
        self.pending_migration = None;
        self.pending_renewal = None;
        self.pending_export_pub = None;
        self.pending_publish = None;
        self.status = None;
      }
      Message::DeleteKeyCancel => {
        self.pending_delete = None;
      }
      Message::DeleteKeyExecute(i) => {
        self.pending_delete = None;
        let fp = self.keys[i].fingerprint.clone();
        let has_secret = self.keys[i].has_secret || self.keys[i].on_card;
        return Task::perform(
          blocking_task(move || crate::gpg::delete_key(&fp, has_secret)),
          Message::DeleteKeyDone,
        );
      }
      Message::DeleteKeyDone(Ok(())) => {
        self.status = Some("Clef supprimée".to_string());
        self.loading = true;
        self.selected = None;
        return Self::reload_keys();
      }
      Message::DeleteKeyDone(Err(e)) => {
        self.status = Some(format!("Erreur suppression : {e}"));
      }
      Message::CopyToClipboard(text) => {
        self.status = Some("Copié dans le presse-papier".to_string());
        return iced::clipboard::write(text);
      }
      Message::RenewSubkey(key_idx, subkey_idx) => {
        self.pending_renewal = Some(PendingRenewal {
          key_idx,
          subkey_idx,
          expiry: KeyExpiry::TwoYears,
        });
        self.pending_migration = None;
        self.pending_delete = None;
        self.pending_export_pub = None;
        self.pending_publish = None;
        self.status = None;
      }
      Message::RenewSubkeyExpiryChanged(expiry) => {
        if let Some(ref mut r) = self.pending_renewal {
          r.expiry = expiry;
        }
      }
      Message::RenewSubkeyCancel => {
        self.pending_renewal = None;
      }
      Message::RenewSubkeyExecute => {
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
      }
      Message::RenewSubkeyDone(Ok(())) => {
        self.status = Some("Sous-clef renouvelée".to_string());
        self.loading = true;
        let reload = Self::reload_keys();
        if let Some(i) = self.selected {
          if let Some(publish) = self.auto_republish_task(i) {
            return Task::batch([reload, publish]);
          }
        }
        return reload;
      }
      Message::RenewSubkeyDone(Err(e)) => {
        self.status = Some(format!("Erreur renouvellement : {e}"));
      }
      Message::KeyserverStatusLoaded(Ok((fp, found))) => {
        self.keyserver_statuses.insert(
          fp,
          if found {
            KeyserverStatus::Published
          } else {
            KeyserverStatus::NotPublished
          },
        );
      }
      Message::KeyserverStatusLoaded(Err(_)) => {
        for status in self.keyserver_statuses.values_mut() {
          if *status == KeyserverStatus::Checking {
            *status = KeyserverStatus::Unknown;
          }
        }
      }
      Message::PublishKey => {
        self.pending_publish = Some(Keyserver::default());
        self.pending_migration = None;
        self.pending_delete = None;
        self.pending_renewal = None;
        self.status = None;
      }
      Message::PublishKeyserverChanged(ks) => {
        self.pending_publish = Some(ks);
      }
      Message::PublishKeyCancel => {
        self.pending_export_pub = None;
        self.pending_publish = None;
      }
      Message::PublishKeyExecute(i) => {
        let keyserver = self.pending_publish.take().unwrap_or_default();
        let fp = self.keys[i].fingerprint.clone();
        let url = keyserver.url().to_string();
        return Task::perform(
          blocking_task(move || crate::gpg::publish_key(&fp, &url)),
          Message::PublishKeyDone,
        );
      }
      Message::PublishKeyDone(Ok(url)) => {
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
      }
      Message::PublishKeyDone(Err(e)) => {
        self.status = Some(format!("Erreur publication : {e}"));
      }
      Message::AddSubkey(key_idx, algo, usage) => {
        let master_fp = self.keys[key_idx].fingerprint.clone();
        return Task::perform(
          blocking_task(move || {
            crate::gpg::add_subkey(&master_fp, &algo, &usage, &KeyExpiry::TwoYears)
          }),
          Message::AddSubkeyDone,
        );
      }
      Message::AddSubkeyDone(Ok(())) => {
        self.status = Some("Sous-clef créée".to_string());
        self.loading = true;
        let reload = Self::reload_keys();
        if let Some(i) = self.selected {
          if let Some(publish) = self.auto_republish_task(i) {
            return Task::batch([reload, publish]);
          }
        }
        return reload;
      }
      Message::AddSubkeyDone(Err(e)) => {
        self.status = Some(format!("Erreur création sous-clef : {e}"));
      }
      Message::AutoRepublishDone(Ok(())) => {
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
      }
      Message::AutoRepublishDone(Err(e)) => {
        self.status = Some(format!("Erreur republication : {e}"));
      }
      Message::RotateSubkeyExecute(key_idx, subkey_idx) => {
        let expiry = self
          .pending_renewal
          .take()
          .map(|r| r.expiry)
          .unwrap_or_default();
        let master_fp = self.keys[key_idx].fingerprint.clone();
        let old_fp = self.keys[key_idx].subkeys[subkey_idx].fingerprint.clone();
        let usage = self.keys[key_idx].subkeys[subkey_idx].usage.clone();
        let (algo, gpg_usage) = if usage.contains('E') {
          ("cv25519".to_string(), "encr".to_string())
        } else if usage.contains('A') {
          ("ed25519".to_string(), "auth".to_string())
        } else {
          ("ed25519".to_string(), "sign".to_string())
        };
        return Task::perform(
          blocking_task(move || {
            crate::gpg::rotate_subkey(&master_fp, &old_fp, &algo, &gpg_usage, &expiry)
          }),
          Message::RotateSubkeyDone,
        );
      }
      Message::RotateSubkeyDone(Ok(())) => {
        self.status = Some("Sous-clef remplacée avec succès".to_string());
        self.loading = true;
        let reload = Self::reload_keys();
        if let Some(i) = self.selected {
          if let Some(publish) = self.auto_republish_task(i) {
            return Task::batch([reload, publish]);
          }
        }
        return reload;
      }
      Message::RotateSubkeyDone(Err(e)) => {
        self.status = Some(format!("Erreur rotation : {e}"));
      }
      Message::HealthChecksLoaded(checks) => {
        self.health_report = checks;
        self.health_loading = false;
      }
    }
    Task::none()
  }

  pub fn view(&self) -> iced::Element<'_, Message> {
    ui::root(self)
  }
}
