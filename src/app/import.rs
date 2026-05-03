use iced::Task;

use super::{blocking_task, App, ImportForm, Message, StatusKind, View};

impl App {
  pub(super) fn on_import_key(&mut self) -> Task<Message> {
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

  pub(super) fn on_import_key_done(
    &mut self,
    result: Result<Option<String>, String>,
  ) -> Task<Message> {
    match result {
      Ok(None) => Task::none(),
      Ok(Some(filename)) => {
        self.status = Some((
          StatusKind::Success,
          format!("Clef importée depuis {filename}"),
        ));
        self.view = View::PublicKeys;
        self.selected = None;
        self.reload_keys()
      }
      Err(e) => {
        self.status = Some((StatusKind::Error, format!("Erreur import : {e}")));
        Task::none()
      }
    }
  }

  pub(super) fn on_import_from_url(&mut self) -> Task<Message> {
    self.import_form.submitting = true;
    let url = self.import_form.url.clone();
    Task::perform(
      blocking_task(move || crate::gpg::import_key_from_url(&url)),
      Message::ImportFromUrlDone,
    )
  }

  pub(super) fn on_import_from_url_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        self.import_form = ImportForm::default();
        self.view = View::PublicKeys;
        self.selected = None;
        self.reload_keys()
      }
      Err(e) => {
        self.import_form.submitting = false;
        self.status = Some((StatusKind::Error, format!("Erreur import URL : {e}")));
        Task::none()
      }
    }
  }

  pub(super) fn on_import_from_keyserver(&mut self) -> Task<Message> {
    self.import_form.submitting = true;
    let query = self.import_form.keyserver_query.clone();
    let url = self.import_form.keyserver.url().to_string();
    Task::perform(
      blocking_task(move || crate::gpg::import_key_from_keyserver(&query, &url)),
      Message::ImportFromKeyserverDone,
    )
  }

  pub(super) fn on_import_from_keyserver_done(
    &mut self,
    result: Result<(), String>,
  ) -> Task<Message> {
    match result {
      Ok(()) => {
        self.import_form = ImportForm::default();
        self.view = View::PublicKeys;
        self.selected = None;
        self.reload_keys()
      }
      Err(e) => {
        self.import_form.submitting = false;
        self.status = Some((StatusKind::Error, format!("Erreur import keyserver : {e}")));
        Task::none()
      }
    }
  }

  pub(super) fn on_import_from_paste(&mut self) -> Task<Message> {
    self.import_form.submitting = true;
    let content = self.import_form.pasted_key.text();
    Task::perform(
      blocking_task(move || crate::gpg::import_key_from_text(&content)),
      Message::ImportFromPasteDone,
    )
  }

  pub(super) fn on_import_from_paste_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        self.import_form = ImportForm::default();
        self.view = View::PublicKeys;
        self.selected = None;
        self.reload_keys()
      }
      Err(e) => {
        self.import_form.submitting = false;
        self.status = Some((StatusKind::Error, format!("Erreur import : {e}")));
        Task::none()
      }
    }
  }
}
