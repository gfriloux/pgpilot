use iced::Task;

use super::{
  backup_key_to_dir, blocking_task, export_key_to_file, App, Message, PendingOp, StatusKind,
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
    Task::perform(
      blocking_task(move || export_key_to_file(fp, name)),
      Message::ExportDone,
    )
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
        self.status = Some((
          StatusKind::Success,
          "Clef copiée dans le presse-papier".to_string(),
        ));
        iced::clipboard::write(armored)
      }
      Err(e) => {
        self.status = Some((StatusKind::Error, format!("Erreur export : {e}")));
        Task::none()
      }
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
        self.status = Some((StatusKind::Success, format!("Lien copié : {url}")));
        iced::clipboard::write(url)
      }
      Err(e) => {
        self.status = Some((StatusKind::Error, format!("Erreur upload : {e}")));
        Task::none()
      }
    }
  }

  pub(super) fn on_backup_key(&mut self, fp: String) -> Task<Message> {
    let key_id = self
      .key_by_fp(&fp)
      .map(|k| k.key_id.clone())
      .unwrap_or_default();
    Task::perform(
      blocking_task(move || backup_key_to_dir(fp, key_id)),
      Message::BackupDone,
    )
  }

  pub(super) fn on_backup_done(&mut self, result: Result<Option<String>, String>) -> Task<Message> {
    match result {
      Ok(None) => {}
      Ok(Some(summary)) => {
        self.status = Some((StatusKind::Success, format!("Sauvegardé : {summary}")))
      }
      Err(e) => self.status = Some((StatusKind::Error, format!("Erreur sauvegarde : {e}"))),
    }
    Task::none()
  }

  pub(super) fn on_export_done(&mut self, result: Result<Option<String>, String>) -> Task<Message> {
    match result {
      Ok(None) => {}
      Ok(Some(filename)) => {
        self.status = Some((StatusKind::Success, format!("Exporté : {filename}")))
      }
      Err(e) => self.status = Some((StatusKind::Error, format!("Erreur export : {e}"))),
    }
    Task::none()
  }
}
