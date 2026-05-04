use iced::Task;

use crate::gpg::TrustLevel;

use super::{blocking_task, truncate_error, App, Message, PendingOp, StatusKind};

impl App {
  pub(super) fn on_move_to_card(&mut self, fp: String) -> Task<Message> {
    self.reset_pending_ops();
    self.pending = Some(PendingOp::Migration(fp));
    Task::none()
  }

  pub(super) fn on_move_to_card_execute(&mut self, fp: String) -> Task<Message> {
    self.pending = None;
    Task::perform(
      blocking_task(move || crate::gpg::move_key_to_card(&fp)),
      Message::MoveToCardDone,
    )
  }

  pub(super) fn on_move_to_card_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        self.selected = None;
        let s = self.set_status(
          StatusKind::Success,
          self.strings.status_card_migrated().to_string(),
        );
        let reload = self.reload_keys();
        Task::batch([s, reload])
      }
      Err(e) => self.set_status(
        StatusKind::Error,
        truncate_error(format!("{}: {e}", self.strings.err_delete_failed())),
      ),
    }
  }

  pub(super) fn on_delete_key(&mut self, fp: String) -> Task<Message> {
    self.reset_pending_ops();
    self.pending = Some(PendingOp::Delete(fp));
    Task::none()
  }

  pub(super) fn on_delete_key_execute(&mut self, fp: String) -> Task<Message> {
    self.pending = None;
    let Some(key) = self.key_by_fp(&fp) else {
      return Task::none();
    };
    let has_secret = key.has_secret || key.on_card;
    Task::perform(
      blocking_task(move || crate::gpg::delete_key(&fp, has_secret)),
      Message::DeleteKeyDone,
    )
  }

  pub(super) fn on_delete_key_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        self.selected = None;
        let s = self.set_status(
          StatusKind::Success,
          self.strings.status_key_deleted().to_string(),
        );
        let reload = self.reload_keys();
        Task::batch([s, reload])
      }
      Err(e) => self.set_status(
        StatusKind::Error,
        truncate_error(format!("{}: {e}", self.strings.err_delete_failed())),
      ),
    }
  }

  pub(super) fn on_copy_to_clipboard(&mut self, text: String) -> Task<Message> {
    let s = self.set_status(
      StatusKind::Success,
      self.strings.status_key_copied().to_string(),
    );
    Task::batch([s, iced::clipboard::write(text)])
  }

  pub(super) fn on_set_key_trust(&mut self, fp: String, trust: TrustLevel) -> Task<Message> {
    Task::perform(
      blocking_task(move || crate::gpg::set_key_trust(&fp, &trust)),
      Message::SetKeyTrustDone,
    )
  }

  pub(super) fn on_set_key_trust_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        let s = self.set_status(
          StatusKind::Success,
          self.strings.status_trust_updated().to_string(),
        );
        let reload = self.reload_keys();
        Task::batch([s, reload])
      }
      Err(e) => self.set_status(
        StatusKind::Error,
        truncate_error(format!("{}: {e}", self.strings.err_trust_failed())),
      ),
    }
  }
}
