use iced::Task;

use crate::gpg::TrustLevel;

use super::{blocking_task, App, Message, PendingOp, StatusKind};

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
        self.status = Some((
          StatusKind::Success,
          "Clef migrée sur YubiKey avec succès".to_string(),
        ));
        self.selected = None;
        self.reload_keys()
      }
      Err(e) => {
        self.status = Some((StatusKind::Error, format!("Erreur migration : {e}")));
        Task::none()
      }
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
        self.status = Some((StatusKind::Success, "Clef supprimée".to_string()));
        self.selected = None;
        self.reload_keys()
      }
      Err(e) => {
        self.status = Some((StatusKind::Error, format!("Erreur suppression : {e}")));
        Task::none()
      }
    }
  }

  pub(super) fn on_copy_to_clipboard(&mut self, text: String) -> Task<Message> {
    self.status = Some((
      StatusKind::Success,
      "Copié dans le presse-papier".to_string(),
    ));
    iced::clipboard::write(text)
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
        self.status = Some((
          StatusKind::Success,
          "Niveau de confiance mis à jour".to_string(),
        ));
        self.reload_keys()
      }
      Err(e) => {
        self.status = Some((StatusKind::Error, format!("Erreur confiance : {e}")));
        Task::none()
      }
    }
  }
}
