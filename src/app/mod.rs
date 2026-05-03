mod card;
mod create;
mod decrypt;
mod encrypt;
mod export;
mod import;
mod keyserver;
mod nav;
mod sign;
mod subkeys;

use std::collections::HashMap;
use std::path::PathBuf;

use iced::widget::text_editor;
use iced::Task;

use crate::gpg::{HealthCheck, KeyExpiry, KeyInfo, Keyserver, TrustLevel, VerifyResult};
use crate::ui;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum View {
  #[default]
  MyKeys,
  PublicKeys,
  CreateKey,
  Import,
  Health,
  Encrypt,
  Decrypt,
  Sign,
  Verify,
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

#[derive(Default)]
pub struct DecryptForm {
  pub files: Vec<PathBuf>,
  pub file_statuses: HashMap<PathBuf, crate::gpg::DecryptStatus>,
  pub decrypting: bool,
}

#[derive(Default)]
pub struct EncryptForm {
  pub recipients: Vec<String>,
  pub files: Vec<PathBuf>,
  pub armor: bool,
  pub encrypting: bool,
  pub trust_prompt: Option<Vec<String>>,
}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum KeyserverStatus {
  #[default]
  Unknown,
  Checking,
  Published,
  NotPublished,
}

#[derive(Debug, Clone)]
pub struct PendingRenewal {
  pub key_fp: String,
  pub subkey_fp: String,
  pub expiry: KeyExpiry,
}

#[derive(Debug, Clone)]
pub enum PendingOp {
  Migration(String),
  Delete(String),
  Renewal(PendingRenewal),
  ExportPubMenu(String),
  Publish(Keyserver),
}

#[derive(Debug, Clone)]
pub enum StatusKind {
  Success,
  Error,
}

#[derive(Default)]
pub struct SignForm {
  pub file: Option<PathBuf>,
  pub signer_fp: Option<String>,
  pub sign_result: Option<PathBuf>,
  pub verify_file: Option<PathBuf>,
  pub verify_sig_file: Option<PathBuf>,
  pub verify_result: Option<Result<VerifyResult, String>>,
  pub signing: bool,
  pub verifying: bool,
}

#[derive(Default)]
pub struct App {
  pub view: View,
  pub keys: Vec<KeyInfo>,
  pub selected: Option<String>,
  pub error: Option<String>,
  pub status: Option<(StatusKind, String)>,
  pub status_generation: u32,
  pub loading: bool,
  pub card_connected: bool,
  pub pending: Option<PendingOp>,
  pub keyserver_statuses: HashMap<String, KeyserverStatus>,
  pub create_form: CreateKeyForm,
  pub import_form: ImportForm,
  pub encrypt_form: EncryptForm,
  pub decrypt_form: DecryptForm,
  pub health_report: Vec<HealthCheck>,
  pub health_loading: bool,
  pub sign_form: SignForm,
  pub previous_view: Option<View>,
}

#[derive(Debug, Clone)]
pub enum Message {
  KeysLoaded(Result<(Vec<KeyInfo>, bool), String>),
  NavChanged(View),
  KeySelected(String),
  ExportPublicKeyMenu(String),
  ExportPublicKeyMenuCancel,
  ExportPublicKey(String),
  ExportPublicKeyClipboard(String),
  ExportPublicKeyClipboardDone(Result<String, String>),
  ExportPublicKeyUpload(String),
  ExportPublicKeyUploadDone(Result<String, String>),
  BackupKey(String),
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
  HealthChecksLoaded(Result<Vec<HealthCheck>, String>),
  MoveToCard(String),
  MoveToCardCancel,
  MoveToCardExecute(String),
  MoveToCardDone(Result<(), String>),
  DeleteKey(String),
  DeleteKeyCancel,
  DeleteKeyExecute(String),
  DeleteKeyDone(Result<(), String>),
  CopyToClipboard(String),
  KeyserverStatusLoaded(Result<(String, bool), String>),
  PublishKey,
  PublishKeyserverChanged(Keyserver),
  PublishKeyExecute(String),
  PublishKeyCancel,
  PublishKeyDone(Result<String, String>),
  AutoRepublishDone(Result<(), String>),
  RotateSubkeyExecute(String, String),
  RotateSubkeyDone(Result<(), String>),
  AddSubkey(String, crate::gpg::SubkeyType),
  AddSubkeyDone(Result<(), String>),
  RenewSubkey(String, String),
  RenewSubkeyExpiryChanged(KeyExpiry),
  RenewSubkeyExecute,
  RenewSubkeyCancel,
  RenewSubkeyDone(Result<(), String>),
  EncryptToggleRecipient(String),
  EncryptPickFiles,
  EncryptFilesPicked(Result<Vec<PathBuf>, String>),
  EncryptRemoveFile(usize),
  EncryptSetArmor(bool),
  EncryptExecute,
  EncryptDone(Result<Vec<String>, String>),
  EncryptTrustPromptConfirm,
  EncryptTrustPromptCancel,
  SetKeyTrust(String, TrustLevel),
  SetKeyTrustDone(Result<(), String>),
  DecryptPickFiles,
  DecryptFilesPicked(Result<Vec<PathBuf>, String>),
  DecryptFileInspected(PathBuf, Result<crate::gpg::DecryptStatus, String>),
  DecryptRemoveFile(usize),
  DecryptExecute,
  DecryptDone(Result<Vec<String>, String>),
  FileDropped(PathBuf),
  SignPickFile,
  SignFilePicked(Result<Option<PathBuf>, String>),
  SignSelectSigner(String),
  SignExecute,
  SignDone(Result<PathBuf, String>),
  VerifyPickFile,
  VerifyFilePicked(Result<Option<PathBuf>, String>),
  VerifyPickSig,
  VerifySigPicked(Result<Option<PathBuf>, String>),
  VerifyExecute,
  VerifyDone(Result<VerifyResult, String>),
  DismissStatus(u32),
  NavBack,
}

pub(crate) async fn blocking_task<T, F>(f: F) -> Result<T, String>
where
  T: Send + 'static,
  F: FnOnce() -> anyhow::Result<T> + Send + 'static,
{
  tokio::task::spawn_blocking(f)
    .await
    .unwrap_or_else(|e| Err(anyhow::anyhow!(e)))
    .map_err(|e| e.to_string())
}

pub(crate) async fn export_key_to_file(fp: String, name: String) -> Result<Option<String>, String> {
  let handle = rfd::AsyncFileDialog::new()
    .set_file_name(format!("{name}.pub.asc"))
    .add_filter("PGP Key", &["asc"])
    .save_file()
    .await;
  let path = match handle {
    None => return Ok(None),
    Some(h) => h.path().to_path_buf(),
  };
  let filename = path
    .file_name()
    .and_then(|n| n.to_str())
    .unwrap_or("key.asc")
    .to_string();
  tokio::task::spawn_blocking(move || -> anyhow::Result<Option<String>> {
    crate::gpg::export_public_key(&fp, &path)?;
    Ok(Some(filename))
  })
  .await
  .unwrap_or_else(|e| Err(anyhow::anyhow!(e)))
  .map_err(|e| e.to_string())
}

pub(crate) async fn backup_key_to_dir(
  fp: String,
  key_id: String,
) -> Result<Option<String>, String> {
  let handle = rfd::AsyncFileDialog::new()
    .set_title("Choisir un dossier de sauvegarde")
    .pick_folder()
    .await;
  let dir = match handle {
    None => return Ok(None),
    Some(h) => h.path().to_path_buf(),
  };
  tokio::task::spawn_blocking(move || -> anyhow::Result<Option<String>> {
    let (key_file, rev_file) = crate::gpg::backup_key(&fp, &dir, &key_id)?;
    let summary = match rev_file {
      Some(rev) => format!("{key_file} + {rev}"),
      None => format!("{key_file} (certificat de révocation introuvable)"),
    };
    Ok(Some(summary))
  })
  .await
  .unwrap_or_else(|e| Err(anyhow::anyhow!(e)))
  .map_err(|e| e.to_string())
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

  pub(crate) fn reload_keys(&mut self) -> Task<Message> {
    self.loading = true;
    Task::perform(blocking_task(crate::gpg::list_keys), Message::KeysLoaded)
  }

  pub(crate) fn reset_pending_ops(&mut self) {
    self.status = None;
    self.pending = None;
  }

  pub(crate) fn set_status(&mut self, kind: StatusKind, msg: String) -> Task<Message> {
    self.status_generation = self.status_generation.wrapping_add(1);
    let gen = self.status_generation;
    self.status = Some((kind, msg));
    Task::perform(
      async move {
        tokio::time::sleep(std::time::Duration::from_secs(4)).await;
      },
      move |_| Message::DismissStatus(gen),
    )
  }

  pub(crate) fn key_by_fp(&self, fp: &str) -> Option<&KeyInfo> {
    self.keys.iter().find(|k| k.fingerprint == fp)
  }

  pub(crate) fn auto_republish_task(&self, fp: &str) -> Option<Task<Message>> {
    if self.keyserver_statuses.get(fp) == Some(&KeyserverStatus::Published) {
      let fp = fp.to_string();
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
        self.pending = Some(PendingOp::Publish(ks));
        Task::none()
      }
      Message::ExportPublicKeyMenuCancel => {
        self.pending = None;
        Task::none()
      }
      Message::MoveToCardCancel => {
        self.pending = None;
        Task::none()
      }
      Message::DeleteKeyCancel => {
        self.pending = None;
        Task::none()
      }
      Message::RenewSubkeyCancel => {
        self.pending = None;
        Task::none()
      }
      Message::RenewSubkeyExpiryChanged(e) => {
        if let Some(PendingOp::Renewal(ref mut r)) = self.pending {
          r.expiry = e;
        }
        Task::none()
      }
      Message::HealthChecksLoaded(Ok(checks)) => {
        self.health_report = checks;
        self.health_loading = false;
        Task::none()
      }
      Message::HealthChecksLoaded(Err(e)) => {
        self.health_loading = false;
        self.set_status(StatusKind::Error, format!("Erreur diagnostic : {e}"))
      }
      // Delegated handlers
      Message::KeysLoaded(r) => self.on_keys_loaded(r),
      Message::NavChanged(v) => self.on_nav_changed(v),
      Message::NavBack => self.on_nav_back(),
      Message::KeySelected(fp) => self.on_key_selected(fp),
      Message::KeyserverStatusLoaded(r) => self.on_keyserver_status_loaded(r),
      Message::ExportPublicKeyMenu(fp) => self.on_export_pub_menu(fp),
      Message::ExportPublicKey(fp) => self.on_export_public(fp),
      Message::ExportPublicKeyClipboard(fp) => self.on_export_clipboard(fp),
      Message::ExportPublicKeyClipboardDone(r) => self.on_export_clipboard_done(r),
      Message::ExportPublicKeyUpload(fp) => self.on_export_upload(fp),
      Message::ExportPublicKeyUploadDone(r) => self.on_export_upload_done(r),
      Message::BackupKey(fp) => self.on_backup_key(fp),
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
      Message::MoveToCard(fp) => self.on_move_to_card(fp),
      Message::MoveToCardExecute(fp) => self.on_move_to_card_execute(fp),
      Message::MoveToCardDone(r) => self.on_move_to_card_done(r),
      Message::DeleteKey(fp) => self.on_delete_key(fp),
      Message::DeleteKeyExecute(fp) => self.on_delete_key_execute(fp),
      Message::DeleteKeyDone(r) => self.on_delete_key_done(r),
      Message::CopyToClipboard(text) => self.on_copy_to_clipboard(text),
      Message::RenewSubkey(kfp, sfp) => self.on_renew_subkey(kfp, sfp),
      Message::RenewSubkeyExecute => self.on_renew_subkey_execute(),
      Message::RenewSubkeyDone(r) => self.on_renew_subkey_done(r),
      Message::AddSubkey(kfp, st) => self.on_add_subkey(kfp, st),
      Message::AddSubkeyDone(r) => self.on_add_subkey_done(r),
      Message::RotateSubkeyExecute(kfp, sfp) => self.on_rotate_subkey_execute(kfp, sfp),
      Message::RotateSubkeyDone(r) => self.on_rotate_subkey_done(r),
      Message::PublishKey => self.on_publish_key(),
      Message::PublishKeyExecute(fp) => self.on_publish_key_execute(fp),
      Message::PublishKeyCancel => self.on_publish_key_cancel(),
      Message::PublishKeyDone(r) => self.on_publish_key_done(r),
      Message::AutoRepublishDone(r) => self.on_auto_republish_done(r),
      Message::EncryptToggleRecipient(fp) => {
        if let Some(pos) = self.encrypt_form.recipients.iter().position(|r| r == &fp) {
          self.encrypt_form.recipients.remove(pos);
        } else {
          self.encrypt_form.recipients.push(fp);
        }
        Task::none()
      }
      Message::EncryptRemoveFile(idx) => {
        if idx < self.encrypt_form.files.len() {
          self.encrypt_form.files.remove(idx);
        }
        Task::none()
      }
      Message::EncryptSetArmor(v) => {
        self.encrypt_form.armor = v;
        Task::none()
      }
      Message::EncryptPickFiles => self.on_encrypt_pick_files(),
      Message::EncryptFilesPicked(r) => self.on_encrypt_files_picked(r),
      Message::EncryptExecute => self.on_encrypt_execute(),
      Message::EncryptDone(r) => self.on_encrypt_done(r),
      Message::EncryptTrustPromptConfirm => self.on_encrypt_trust_confirm(),
      Message::EncryptTrustPromptCancel => {
        self.encrypt_form.trust_prompt = None;
        Task::none()
      }
      Message::SetKeyTrust(fp, trust) => self.on_set_key_trust(fp, trust),
      Message::SetKeyTrustDone(r) => self.on_set_key_trust_done(r),
      Message::DecryptRemoveFile(idx) => {
        if idx < self.decrypt_form.files.len() {
          let path = self.decrypt_form.files.remove(idx);
          self.decrypt_form.file_statuses.remove(&path);
        }
        Task::none()
      }
      Message::DecryptPickFiles => self.on_decrypt_pick_files(),
      Message::DecryptFilesPicked(r) => self.on_decrypt_files_picked(r),
      Message::DecryptFileInspected(path, r) => self.on_decrypt_file_inspected(path, r),
      Message::DecryptExecute => self.on_decrypt_execute(),
      Message::DecryptDone(r) => self.on_decrypt_done(r),
      Message::SignPickFile => self.on_sign_pick_file(),
      Message::SignFilePicked(r) => self.on_sign_file_picked(r),
      Message::SignSelectSigner(fp) => self.on_sign_select_signer(fp),
      Message::SignExecute => self.on_sign_execute(),
      Message::SignDone(r) => self.on_sign_done(r),
      Message::VerifyPickFile => self.on_verify_pick_file(),
      Message::VerifyFilePicked(r) => self.on_verify_file_picked(r),
      Message::VerifyPickSig => self.on_verify_pick_sig(),
      Message::VerifySigPicked(r) => self.on_verify_sig_picked(r),
      Message::VerifyExecute => self.on_verify_execute(),
      Message::VerifyDone(r) => self.on_verify_done(r),
      Message::DismissStatus(gen) => {
        if self.status_generation == gen {
          self.status = None;
        }
        Task::none()
      }
      Message::FileDropped(path) => {
        if self.view == View::Encrypt && !self.encrypt_form.files.contains(&path) {
          self.encrypt_form.files.push(path);
          Task::none()
        } else if self.view == View::Decrypt && !self.decrypt_form.files.contains(&path) {
          self.decrypt_form.files.push(path.clone());
          self
            .decrypt_form
            .file_statuses
            .insert(path.clone(), crate::gpg::DecryptStatus::Checking);
          let p = path.clone();
          Task::perform(
            blocking_task(move || crate::gpg::inspect_decrypt(&p)),
            move |r| Message::DecryptFileInspected(path.clone(), r),
          )
        } else {
          Task::none()
        }
      }
    }
  }

  pub fn subscription(&self) -> iced::Subscription<Message> {
    iced::event::listen_with(|event, _, _| match event {
      iced::Event::Window(iced::window::Event::FileDropped(path)) => {
        Some(Message::FileDropped(path))
      }
      _ => None,
    })
  }

  pub fn view(&self) -> iced::Element<'_, Message> {
    ui::root(self)
  }
}
