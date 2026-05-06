use std::path::PathBuf;

use iced::Task;

use super::{blocking_task, truncate_error, App, Message, StatusKind, VerifyResult};

impl App {
  pub(super) fn on_sign_pick_file(&mut self) -> Task<Message> {
    let title = self.strings.dialog_choose_file_sign();
    Task::perform(
      async move {
        Ok(
          rfd::AsyncFileDialog::new()
            .set_title(title)
            .pick_file()
            .await
            .map(|h| h.path().to_path_buf()),
        )
      },
      Message::SignFilePicked,
    )
  }

  pub(super) fn on_sign_file_picked(
    &mut self,
    result: Result<Option<PathBuf>, String>,
  ) -> Task<Message> {
    match result {
      Ok(Some(path)) => {
        self.sign_form.file = Some(path);
        self.sign_form.sign_result = None;
        self.status = None;
      }
      Ok(None) => {}
      Err(e) => return self.set_status(StatusKind::Error, e),
    }
    Task::none()
  }

  pub(super) fn on_sign_select_signer(&mut self, fp: String) -> Task<Message> {
    self.sign_form.signer_fp = Some(fp);
    Task::none()
  }

  pub(super) fn on_sign_execute(&mut self) -> Task<Message> {
    let (Some(file), Some(fp)) = (
      self.sign_form.file.clone(),
      self.sign_form.signer_fp.clone(),
    ) else {
      return Task::none();
    };
    self.sign_form.signing = true;
    self.sign_form.sign_result = None;
    self.status = None;
    Task::perform(
      blocking_task(move || crate::gpg::sign_file(file, &fp)),
      Message::SignDone,
    )
  }

  pub(super) fn on_sign_done(&mut self, result: Result<PathBuf, String>) -> Task<Message> {
    self.sign_form.signing = false;
    match result {
      Ok(sig_path) => {
        self.sign_form.sign_result = Some(sig_path);
        self.status = None;
        Task::none()
      }
      Err(e) => self.set_status(
        StatusKind::Error,
        truncate_error(format!("{}: {e}", self.strings.err_sign_failed())),
      ),
    }
  }

  pub(super) fn on_verify_pick_file(&mut self) -> Task<Message> {
    let title = self.strings.dialog_choose_file_verify();
    Task::perform(
      async move {
        Ok(
          rfd::AsyncFileDialog::new()
            .set_title(title)
            .pick_file()
            .await
            .map(|h| h.path().to_path_buf()),
        )
      },
      Message::VerifyFilePicked,
    )
  }

  pub(super) fn on_verify_file_picked(
    &mut self,
    result: Result<Option<PathBuf>, String>,
  ) -> Task<Message> {
    match result {
      Ok(Some(path)) => {
        self.sign_form.verify_file = Some(path);
        self.sign_form.verify_result = None;
        self.status = None;
      }
      Ok(None) => {}
      Err(e) => return self.set_status(StatusKind::Error, e),
    }
    Task::none()
  }

  pub(super) fn on_verify_pick_sig(&mut self) -> Task<Message> {
    let title = self.strings.dialog_choose_sig_file();
    Task::perform(
      async move {
        Ok(
          rfd::AsyncFileDialog::new()
            .set_title(title)
            .add_filter("Signature", &["sig", "asc"])
            .pick_file()
            .await
            .map(|h| h.path().to_path_buf()),
        )
      },
      Message::VerifySigPicked,
    )
  }

  pub(super) fn on_verify_sig_picked(
    &mut self,
    result: Result<Option<PathBuf>, String>,
  ) -> Task<Message> {
    match result {
      Ok(Some(path)) => {
        self.sign_form.verify_sig_file = Some(path);
        self.sign_form.verify_result = None;
        self.status = None;
      }
      Ok(None) => {}
      Err(e) => return self.set_status(StatusKind::Error, e),
    }
    Task::none()
  }

  pub(super) fn on_verify_execute(&mut self) -> Task<Message> {
    let Some(file) = self.sign_form.verify_file.clone() else {
      return Task::none();
    };
    let sig = self.sign_form.verify_sig_file.clone();
    self.sign_form.verifying = true;
    self.sign_form.verify_result = None;
    self.status = None;
    Task::perform(
      blocking_task(move || crate::gpg::verify_signature(file, sig)),
      Message::VerifyDone,
    )
  }

  pub(super) fn on_verify_done(&mut self, result: Result<VerifyResult, String>) -> Task<Message> {
    self.sign_form.verifying = false;
    self.sign_form.verify_result = Some(result);
    self.status = None;
    Task::none()
  }
}
