use std::path::PathBuf;

use iced::Task;

use super::{blocking_task, truncate_error, App, Message, StatusKind};

impl App {
  pub(super) fn on_encrypt_pick_files(&mut self) -> Task<Message> {
    Task::perform(
      async {
        let handles = rfd::AsyncFileDialog::new()
          .set_title("Choisir des fichiers à chiffrer")
          .pick_files()
          .await
          .unwrap_or_default();
        Ok(
          handles
            .iter()
            .map(|h| h.path().to_path_buf())
            .collect::<Vec<_>>(),
        )
      },
      Message::EncryptFilesPicked,
    )
  }

  pub(super) fn on_encrypt_files_picked(
    &mut self,
    result: Result<Vec<PathBuf>, String>,
  ) -> Task<Message> {
    match result {
      Ok(files) => {
        for f in files {
          if !self.encrypt_form.files.contains(&f) {
            self.encrypt_form.files.push(f);
          }
        }
        Task::none()
      }
      Err(e) => self.set_status(StatusKind::Error, e),
    }
  }

  pub(super) fn on_encrypt_execute(&mut self) -> Task<Message> {
    let untrusted: Vec<String> = self
      .encrypt_form
      .recipients
      .iter()
      .filter_map(|fp| self.key_by_fp(fp))
      .filter(|k| !k.trust.is_sufficient())
      .map(|k| k.fingerprint.clone())
      .collect();

    if !untrusted.is_empty() {
      self.encrypt_form.trust_prompt = Some(untrusted);
      return Task::none();
    }

    self.do_encrypt(false)
  }

  pub(super) fn on_encrypt_trust_confirm(&mut self) -> Task<Message> {
    self.encrypt_form.trust_prompt = None;
    self.do_encrypt(true)
  }

  pub(super) fn do_encrypt(&mut self, force_trust: bool) -> Task<Message> {
    let files = self.encrypt_form.files.clone();
    let recipients = self.encrypt_form.recipients.clone();
    let armor = self.encrypt_form.armor;
    self.encrypt_form.encrypting = true;
    self.status = None;
    Task::perform(
      blocking_task(move || crate::gpg::encrypt_files(&files, &recipients, armor, force_trust)),
      Message::EncryptDone,
    )
  }

  pub(super) fn on_encrypt_done(&mut self, result: Result<Vec<String>, String>) -> Task<Message> {
    self.encrypt_form.encrypting = false;
    match result {
      Ok(names) => {
        let summary = format!(
          "{}: {}",
          self.strings.status_files_encrypted(),
          names.join(", ")
        );
        self.encrypt_form.files.clear();
        self.set_status(StatusKind::Success, summary)
      }
      Err(e) => self.set_status(
        StatusKind::Error,
        truncate_error(format!("{}: {e}", self.strings.err_encrypt_failed())),
      ),
    }
  }
}
