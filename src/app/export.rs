use iced::Task;

use super::{
  backup_key_to_dir, blocking_task, export_key_to_file, truncate_error, App, Message, PendingOp,
  StatusKind,
};

impl App {
  pub(super) fn on_export_pub_menu(&mut self, fp: String) -> Task<Message> {
    self.reset_pending_ops();
    self.pending = Some(PendingOp::ExportPubMenu(fp));
    Task::none()
  }

  pub(super) fn on_export_public(&mut self, fp: String) -> Task<Message> {
    self.pending = None;
    let name = self
      .key_by_fp(&fp)
      .map(|k| k.name.replace(' ', "_"))
      .unwrap_or_default();
    Task::perform(export_key_to_file(fp, name), Message::ExportDone)
  }

  pub(super) fn on_export_clipboard(&mut self, fp: String) -> Task<Message> {
    self.pending = None;
    Task::perform(
      blocking_task(move || crate::gpg::export_public_key_armored(&fp)),
      Message::ExportPublicKeyClipboardDone,
    )
  }

  pub(super) fn on_export_clipboard_done(
    &mut self,
    result: Result<String, String>,
  ) -> Task<Message> {
    match result {
      Ok(armored) => {
        let s = self.set_status(
          StatusKind::Success,
          self.strings.status_key_copied().to_string(),
        );
        Task::batch([s, iced::clipboard::write(armored)])
      }
      Err(e) => self.set_status(
        StatusKind::Error,
        truncate_error(format!("{}: {e}", self.strings.err_export_failed())),
      ),
    }
  }

  pub(super) fn on_export_upload(&mut self, fp: String) -> Task<Message> {
    self.pending = None;
    Task::perform(
      blocking_task(move || crate::gpg::upload_public_key(&fp)),
      Message::ExportPublicKeyUploadDone,
    )
  }

  pub(super) fn on_export_upload_done(&mut self, result: Result<String, String>) -> Task<Message> {
    match result {
      Ok(url) => {
        let msg = format!("{}: {url}", self.strings.status_link_copied());
        let s = self.set_status(StatusKind::Success, msg);
        Task::batch([s, iced::clipboard::write(url)])
      }
      Err(e) => self.set_status(
        StatusKind::Error,
        truncate_error(format!("{}: {e}", self.strings.err_upload_failed())),
      ),
    }
  }

  pub(super) fn on_backup_key(&mut self, fp: String) -> Task<Message> {
    let key_id = self
      .key_by_fp(&fp)
      .map(|k| k.key_id.clone())
      .unwrap_or_default();
    let title = self.strings.dialog_choose_backup_folder();
    Task::perform(backup_key_to_dir(fp, key_id, title), Message::BackupDone)
  }

  pub(super) fn on_backup_done(&mut self, result: Result<Option<String>, String>) -> Task<Message> {
    match result {
      Ok(None) => Task::none(),
      Ok(Some(summary)) => self.set_status(
        StatusKind::Success,
        format!("{}: {summary}", self.strings.status_backup_done()),
      ),
      Err(e) => self.set_status(
        StatusKind::Error,
        truncate_error(format!("{}: {e}", self.strings.err_backup_failed())),
      ),
    }
  }

  pub(super) fn on_export_done(&mut self, result: Result<Option<String>, String>) -> Task<Message> {
    match result {
      Ok(None) => Task::none(),
      Ok(Some(filename)) => self.set_status(
        StatusKind::Success,
        format!("{}: {filename}", self.strings.status_key_exported()),
      ),
      Err(e) => self.set_status(
        StatusKind::Error,
        truncate_error(format!("{}: {e}", self.strings.err_export_failed())),
      ),
    }
  }
}
