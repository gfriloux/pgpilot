mod card;
mod chat;
mod create;
mod decrypt;
mod encrypt;
mod export;
mod import;
mod keyserver;
mod nav;
mod settings;
mod sign;
mod subkeys;

use std::collections::HashMap;
use std::path::PathBuf;

use iced::widget::text_editor;
use iced::Task;

use crate::config::Config;
use crate::gpg::{
  ExpiryWarning, HealthCheck, KeyExpiry, KeyInfo, Keyserver, TrustLevel, VerifyResult,
};
use crate::i18n::{self, Language, Strings};
use crate::ui;
use crate::ui::theme::ThemeVariant;

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
  Settings,
  // --- v0.6.0 Chat ---
  /// Liste des salons de chat.
  ChatList,
  /// Conversation dans un salon (room_id UUID).
  ChatRoom(String),
  /// Formulaire de création d'un nouveau salon.
  // UI à câbler dans l'axe 5.
  #[allow(dead_code)]
  ChatNewRoom,
  /// Formulaire de jointure via code d'invitation.
  // UI à câbler dans l'axe 5.
  #[allow(dead_code)]
  ChatJoinRoom,
}

// ---------------------------------------------------------------------------
// Chat v0.6.0 — types d'état
// ---------------------------------------------------------------------------

/// État de connexion au broker MQTT.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum MqttState {
  /// Pas de connexion établie (état initial).
  #[default]
  Disconnected,
  /// Connexion en cours d'établissement.
  Connecting,
  /// Connexion active.
  Connected,
  /// Tentative de reconnexion en cours.
  Reconnecting {
    /// Numéro de la tentative (commence à 1).
    attempt: u32,
  },
  /// Erreur non récupérable (URL malformée, auth refusée définitivement…).
  Failed(String),
}

/// Formulaire de création / jointure d'un salon de chat.
#[derive(Debug, Clone, Default)]
pub struct ChatNewForm {
  /// Nom local du salon en cours de création.
  pub name: String,
  /// URL du broker MQTT (pré-remplie depuis `Config.mqtt_default_relay`).
  pub relay: String,
  /// Fingerprints des participants sélectionnés depuis le keyring.
  pub selected_participants: Vec<String>,
  /// Identité locale choisie pour cette room (fingerprint de la clef privée).
  pub my_fp: Option<String>,
  /// Code d'invitation en cours de saisie pour rejoindre un salon.
  pub join_code: String,
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
  // --- v0.6.0 Chat ---
  /// Sélection d'identité avant d'entrer dans un salon (multi-clefs privées).
  // UI câblée dans l'axe 5 ; handlers dans les axes suivants.
  #[allow(dead_code)]
  IdentitySelection {
    room_id: String,
    selected_fp: Option<String>,
  },
  /// Confirmation de sortie d'un salon.
  // UI câblée dans l'axe 5 ; handlers dans les axes suivants.
  #[allow(dead_code)]
  LeaveRoom(String),
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
  pub config: Config,
  pub strings: &'static dyn Strings,
  pub expiry_warnings: Vec<ExpiryWarning>,

  // --- Chat (v0.6.0) ---
  /// Salons persistés, chargés au démarrage depuis `~/.config/pgpilot/rooms.yaml`.
  pub rooms: Vec<crate::chat::Room>,
  /// Salon actif (room_id UUID). `None` hors section chat.
  pub active_room: Option<String>,
  /// Messages en RAM par room_id. Jamais persistés. Borné à 500/room (FIFO).
  pub chat_messages:
    std::collections::HashMap<String, std::collections::VecDeque<crate::chat::ChatMessage>>,
  /// Tracker de présence agrégé pour tous les fingerprints connus.
  pub presence: crate::chat::PresenceTracker,
  /// État de connexion MQTT.
  pub mqtt_state: MqttState,
  /// Handle vers le client MQTT (None tant que pas démarré).
  pub mqtt: Option<crate::chat::MqttHandle>,
  /// Saisie courante dans la room active. Vidé à chaque changement de room.
  pub chat_input: String,
  /// Formulaire dédié pour création / jointure de salon.
  pub chat_new_form: ChatNewForm,
  /// Contexte crypto (Cert local + peers), chargé une fois par session.
  pub chat_crypto: Option<std::sync::Arc<crate::chat::ChatCryptoCtx>>,
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
  ExportRevocationCert(String),
  CopyRevocationCertPath(String),
  GenerateRevocationCert(String),
  RevocationCertGenerated(Result<String, String>),
  DismissStatus(u32),
  NavBack,
  ChangeLanguage(Language),
  ScaleFactorChanged(f64),
  ThemeChanged(ThemeVariant),

  // --- Chat : navigation / création / jointure (UI câblée dans les axes 5–8) ---
  /// Soumet le formulaire de création de salon.
  #[allow(dead_code)]
  ChatRoomCreate,
  #[allow(dead_code)]
  ChatRoomNameChanged(String),
  #[allow(dead_code)]
  ChatRoomRelayChanged(String),
  /// Sélection de l'identité locale (my_fp) pour le salon en cours de création/jointure.
  ChatRoomMyFpChanged(String),
  /// Toggle d'un participant (fingerprint) dans la sélection du nouveau salon.
  #[allow(dead_code)]
  ChatRoomParticipantToggled(String),
  ChatRoomCreated(Result<crate::chat::Room, String>),
  #[allow(dead_code)]
  ChatJoinCodeChanged(String),
  /// Soumet le formulaire de jointure via join code.
  #[allow(dead_code)]
  ChatRoomJoin,
  ChatRoomJoined(Result<crate::chat::Room, String>),
  /// Clic sur un salon dans la liste.
  #[allow(dead_code)]
  ChatRoomSelected(String),
  #[allow(dead_code)]
  ChatRoomLeave(String),
  /// room_id du salon quitté.
  ChatRoomLeft(Result<String, String>),

  // --- Chat : envoi / réception ---
  #[allow(dead_code)]
  ChatInputChanged(String),
  /// Bouton "Envoyer" ou touche Entrée dans la saisie de message.
  #[allow(dead_code)]
  ChatSend,
  ChatSent(Result<crate::chat::ChatMessage, String>),
  /// Message déchiffré reçu : (room_id, message).
  ChatReceived(String, crate::chat::ChatMessage),

  // --- Chat : identité ---
  /// Sélection d'une clef privée dans le modal IdentitySelection.
  #[allow(dead_code)]
  ChatIdentitySelected(String),
  /// Confirmation de l'identité sélectionnée — sauvegarde en config et démarre le chat.
  ChatIdentityConfirm,

  // --- Chat : partage du join code ---
  /// Encode et copie le join code du salon dans le presse-papier.
  #[allow(dead_code)]
  ChatJoinCodeCopy(String),
  ChatJoinCodeCopied(Result<String, String>),

  // --- MQTT infra ---
  MqttEvent(crate::chat::MqttEvent),
  MqttCryptoLoaded(Result<std::sync::Arc<crate::chat::ChatCryptoCtx>, String>),

  // --- Présence ---
  PresenceUpdated(crate::chat::PresenceUpdate),

  // --- ACK applicatif ---
  /// ACK reçu : (room_id, msg_id, sender_fp).
  ChatAckReceived(String, String, String),
  ChatAckSent(Result<(), String>),

  /// Signal interne — opération de fond sans résultat pertinent pour l'UI.
  ChatBackgroundDone,
}

pub(crate) fn truncate_error(msg: String) -> String {
  if msg.len() <= 120 {
    return msg;
  }
  let cut = msg
    .char_indices()
    .map(|(i, c)| i + c.len_utf8())
    .take_while(|&end| end <= 120)
    .last()
    .unwrap_or(0);
  format!("{}…", &msg[..cut])
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
  title: &'static str,
) -> Result<Option<String>, String> {
  let handle = rfd::AsyncFileDialog::new()
    .set_title(title)
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
    let config = Config::load().unwrap_or_default();
    let strings = i18n::strings_for(config.language);
    // Initialise the theme from persisted config before first frame renders.
    crate::ui::theme::set_active(config.theme);
    let task = Task::perform(blocking_task(crate::gpg::list_keys), Message::KeysLoaded);

    // Charger les salons persistés (rooms.yaml). Tolérer l'absence du fichier.
    let rooms = crate::chat::RoomStore::load()
      .map(|store| store.rooms)
      .unwrap_or_default();

    // Pré-remplir le relay dans le formulaire de création si configuré.
    let default_relay = config.mqtt_default_relay.clone().unwrap_or_default();

    (
      Self {
        view: View::MyKeys,
        keys: Vec::new(),
        selected: None,
        error: None,
        status: None,
        status_generation: 0,
        loading: true,
        card_connected: false,
        pending: None,
        keyserver_statuses: HashMap::new(),
        create_form: CreateKeyForm::default(),
        import_form: ImportForm::default(),
        encrypt_form: EncryptForm::default(),
        decrypt_form: DecryptForm::default(),
        health_report: Vec::new(),
        health_loading: false,
        sign_form: SignForm::default(),
        previous_view: None,
        config,
        strings,
        expiry_warnings: Vec::new(),
        // Chat v0.6.0
        rooms,
        active_room: None,
        chat_messages: std::collections::HashMap::new(),
        presence: crate::chat::PresenceTracker::new(),
        mqtt_state: MqttState::Disconnected,
        mqtt: None,
        chat_input: String::new(),
        chat_new_form: ChatNewForm {
          relay: default_relay,
          ..ChatNewForm::default()
        },
        chat_crypto: None,
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
        self.set_status(
          StatusKind::Error,
          format!("{}: {e}", self.strings.err_diagnostic_failed()),
        )
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
      Message::ExportRevocationCert(fp) => self.on_export_revocation_cert(fp),
      Message::CopyRevocationCertPath(path) => self.on_copy_revocation_cert_path(path),
      Message::GenerateRevocationCert(fp) => self.on_generate_revocation_cert(fp),
      Message::RevocationCertGenerated(result) => self.on_revocation_cert_generated(result),
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
      Message::ChangeLanguage(lang) => self.on_language_changed(lang),
      Message::ScaleFactorChanged(v) => self.on_scale_factor_changed(v),
      Message::ThemeChanged(v) => self.on_theme_changed(v),

      // --- Chat : trivial (inline) ---
      Message::ChatInputChanged(v) => {
        self.chat_input = v;
        Task::none()
      }
      Message::ChatRoomNameChanged(v) => {
        self.chat_new_form.name = v;
        Task::none()
      }
      Message::ChatRoomRelayChanged(v) => {
        self.chat_new_form.relay = v;
        Task::none()
      }
      Message::ChatRoomMyFpChanged(fp) => {
        self.chat_new_form.my_fp = Some(fp);
        Task::none()
      }
      Message::ChatRoomParticipantToggled(fp) => {
        let participants = &mut self.chat_new_form.selected_participants;
        if let Some(pos) = participants.iter().position(|p| p == &fp) {
          participants.remove(pos);
        } else {
          participants.push(fp);
        }
        Task::none()
      }
      Message::ChatJoinCodeChanged(v) => {
        self.chat_new_form.join_code = v;
        Task::none()
      }

      // --- Chat : délégués à app/chat.rs ---
      Message::ChatIdentitySelected(fp) => {
        if let Some(PendingOp::IdentitySelection {
          ref mut selected_fp,
          ..
        }) = self.pending
        {
          *selected_fp = Some(fp);
        }
        Task::none()
      }
      Message::ChatIdentityConfirm => self.on_chat_identity_confirm(),
      Message::ChatRoomCreate => self.on_chat_room_create(),
      Message::ChatRoomCreated(r) => self.on_chat_room_created(r),
      Message::ChatRoomJoin => self.on_chat_room_join(),
      Message::ChatRoomJoined(r) => self.on_chat_room_joined(r),
      Message::ChatRoomSelected(id) => self.on_chat_room_selected(id),
      Message::ChatRoomLeave(id) => self.on_chat_room_leave(id),
      Message::ChatRoomLeft(r) => self.on_chat_room_left(r),
      Message::ChatSend => self.on_chat_send(),
      Message::ChatSent(r) => self.on_chat_sent(r),
      Message::ChatReceived(id, m) => self.on_chat_received(id, m),
      Message::ChatJoinCodeCopy(id) => self.on_chat_join_code_copy(id),
      Message::ChatJoinCodeCopied(r) => self.on_chat_join_code_copied(r),
      Message::MqttEvent(e) => self.on_mqtt_event(e),
      Message::MqttCryptoLoaded(r) => self.on_mqtt_crypto_loaded(r),
      Message::PresenceUpdated(u) => self.on_presence_updated(u),
      Message::ChatAckReceived(rid, mid, sfp) => self.on_chat_ack_received(rid, mid, sfp),
      Message::ChatAckSent(r) => self.on_chat_ack_sent(r),
      Message::ChatBackgroundDone => Task::none(),
    }
  }

  pub fn subscription(&self) -> iced::Subscription<Message> {
    let file_drop = iced::event::listen_with(|event, _, _| match event {
      iced::Event::Window(iced::window::Event::FileDropped(path)) => {
        Some(Message::FileDropped(path))
      }
      _ => None,
    });

    let mut subs = vec![file_drop];

    if let Some(handle) = &self.mqtt {
      subs.push(crate::chat::mqtt::subscription(handle.clone()));
    }

    iced::Subscription::batch(subs)
  }

  pub fn view(&self) -> iced::Element<'_, Message> {
    ui::root(self)
  }
}
