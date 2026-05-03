use iced::Task;

use crate::gpg::Keyserver;

use super::{blocking_task, App, KeyserverStatus, Message, PendingOp, StatusKind};

impl App {
  pub(super) fn on_publish_key(&mut self) -> Task<Message> {
    self.reset_pending_ops();
    self.pending = Some(PendingOp::Publish(Keyserver::default()));
    Task::none()
  }

  pub(super) fn on_publish_key_execute(&mut self, fp: String) -> Task<Message> {
    let keyserver = match self.pending.take() {
      Some(PendingOp::Publish(ks)) => ks,
      _ => Keyserver::default(),
    };
    let url = keyserver.url().to_string();
    Task::perform(
      blocking_task(move || crate::gpg::publish_key(&fp, &url)),
      Message::PublishKeyDone,
    )
  }

  pub(super) fn on_publish_key_cancel(&mut self) -> Task<Message> {
    self.pending = None;
    Task::none()
  }

  pub(super) fn on_publish_key_done(&mut self, result: Result<String, String>) -> Task<Message> {
    match result {
      Ok(url) => {
        let msg = if url == "keys.openpgp.org" {
          "Clef publiée. Vérifiez votre email pour valider la publication sur keys.openpgp.org."
            .to_string()
        } else {
          "Clef publiée avec succès.".to_string()
        };
        let s = self.set_status(StatusKind::Success, msg);
        if let Some(ref fp) = self.selected.clone() {
          self
            .keyserver_statuses
            .insert(fp.clone(), KeyserverStatus::Checking);
          let fp2 = fp.clone();
          let check = Task::perform(
            blocking_task(move || crate::gpg::check_keyserver(&fp2)),
            Message::KeyserverStatusLoaded,
          );
          return Task::batch([s, check]);
        }
        s
      }
      Err(e) => self.set_status(StatusKind::Error, format!("Erreur publication : {e}")),
    }
  }

  pub(super) fn on_auto_republish_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        if let Some(ref fp) = self.selected.clone() {
          self
            .keyserver_statuses
            .insert(fp.clone(), KeyserverStatus::Checking);
          let fp2 = fp.clone();
          return Task::perform(
            blocking_task(move || crate::gpg::check_keyserver(&fp2)),
            Message::KeyserverStatusLoaded,
          );
        }
        Task::none()
      }
      Err(e) => self.set_status(StatusKind::Error, format!("Erreur republication : {e}")),
    }
  }
}
