use iced::Task;

use crate::gpg::{KeyAlgo, KeyExpiry, KeyInfo};
use crate::ui;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum View {
  #[default]
  MyKeys,
  PublicKeys,
  CreateKey,
}

#[derive(Default)]
pub struct CreateKeyForm {
  pub name: String,
  pub email: String,
  pub algo: KeyAlgo,
  pub expiry: KeyExpiry,
  pub submitting: bool,
}

#[derive(Default)]
pub struct App {
  pub view: View,
  pub keys: Vec<KeyInfo>,
  pub selected: Option<usize>,
  pub error: Option<String>,
  pub status: Option<String>,
  pub loading: bool,
  pub create_form: CreateKeyForm,
}

#[derive(Debug, Clone)]
pub enum Message {
  KeysLoaded(Result<Vec<KeyInfo>, String>),
  NavChanged(View),
  KeySelected(usize),
  ExportPublicKey(usize),
  ExportSecretKey(usize),
  ExportDone(Result<Option<String>, String>),
  CreateKeyNameChanged(String),
  CreateKeyEmailChanged(String),
  CreateKeyAlgoChanged(KeyAlgo),
  CreateKeyExpiryChanged(KeyExpiry),
  CreateKeySubmit,
  CreateKeyDone(Result<(), String>),
}

impl App {
  pub fn new() -> (Self, Task<Message>) {
    let task = Task::perform(
      async {
        tokio::task::spawn_blocking(crate::gpg::list_keys)
          .await
          .unwrap_or_else(|e| Err(anyhow::anyhow!(e)))
      },
      |result| Message::KeysLoaded(result.map_err(|e| e.to_string())),
    );
    (
      Self {
        loading: true,
        ..Default::default()
      },
      task,
    )
  }

  fn reload_keys() -> Task<Message> {
    Task::perform(
      async {
        tokio::task::spawn_blocking(crate::gpg::list_keys)
          .await
          .unwrap_or_else(|e| Err(anyhow::anyhow!(e)))
      },
      |result| Message::KeysLoaded(result.map_err(|e| e.to_string())),
    )
  }

  pub fn update(&mut self, message: Message) -> Task<Message> {
    match message {
      Message::KeysLoaded(Ok(keys)) => {
        self.keys = keys;
        self.loading = false;
      }
      Message::KeysLoaded(Err(e)) => {
        self.error = Some(e);
        self.loading = false;
      }
      Message::NavChanged(view) => {
        self.view = view;
        self.selected = None;
        self.status = None;
      }
      Message::KeySelected(i) => {
        self.selected = Some(i);
        self.status = None;
      }
      Message::ExportPublicKey(i) => {
        let fp = self.keys[i].fingerprint.clone();
        let name = self.keys[i].name.replace(' ', "_");
        return Task::perform(
          async move {
            tokio::task::spawn_blocking(move || -> anyhow::Result<Option<String>> {
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
            })
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!(e)))
            .map_err(|e| e.to_string())
          },
          Message::ExportDone,
        );
      }
      Message::ExportSecretKey(i) => {
        let fp = self.keys[i].fingerprint.clone();
        let name = self.keys[i].name.replace(' ', "_");
        return Task::perform(
          async move {
            tokio::task::spawn_blocking(move || -> anyhow::Result<Option<String>> {
              let path = match rfd::FileDialog::new()
                .set_file_name(format!("{name}.sec.asc"))
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
              crate::gpg::export_secret_key(&fp, &path)?;
              Ok(Some(filename))
            })
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!(e)))
            .map_err(|e| e.to_string())
          },
          Message::ExportDone,
        );
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
      Message::CreateKeyAlgoChanged(v) => self.create_form.algo = v,
      Message::CreateKeyExpiryChanged(v) => self.create_form.expiry = v,
      Message::CreateKeySubmit => {
        let name = self.create_form.name.clone();
        let email = self.create_form.email.clone();
        let algo = self.create_form.algo.clone();
        let expiry = self.create_form.expiry.clone();
        self.create_form.submitting = true;
        return Task::perform(
          async move {
            tokio::task::spawn_blocking(move || {
              crate::gpg::create_key(&name, &email, &algo, &expiry)
            })
            .await
            .unwrap_or_else(|e| Err(anyhow::anyhow!(e)))
          },
          |result| Message::CreateKeyDone(result.map_err(|e| e.to_string())),
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
    }
    Task::none()
  }

  pub fn view(&self) -> iced::Element<'_, Message> {
    ui::root(self)
  }
}
