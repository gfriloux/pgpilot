use iced::Task;

use crate::gpg::KeyExpiry;

use super::{blocking_task, App, Message, PendingOp, PendingRenewal, StatusKind};

impl App {
  pub(super) fn on_renew_subkey(&mut self, key_fp: String, subkey_fp: String) -> Task<Message> {
    self.reset_pending_ops();
    self.pending = Some(PendingOp::Renewal(PendingRenewal {
      key_fp,
      subkey_fp,
      expiry: KeyExpiry::TwoYears,
    }));
    Task::none()
  }

  pub(super) fn on_renew_subkey_execute(&mut self) -> Task<Message> {
    if let Some(PendingOp::Renewal(renewal)) = self.pending.take() {
      let master_fp = renewal.key_fp;
      let subkey_fp = renewal.subkey_fp;
      let expiry = renewal.expiry;
      return Task::perform(
        blocking_task(move || crate::gpg::renew_subkey(&master_fp, &subkey_fp, &expiry)),
        Message::RenewSubkeyDone,
      );
    }
    Task::none()
  }

  pub(super) fn on_renew_subkey_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        let s = self.set_status(StatusKind::Success, "Sous-clef renouvelée".to_string());
        let reload = self.reload_keys();
        if let Some(ref fp) = self.selected.clone() {
          if let Some(publish) = self.auto_republish_task(fp) {
            return Task::batch([s, reload, publish]);
          }
        }
        Task::batch([s, reload])
      }
      Err(e) => self.set_status(StatusKind::Error, format!("Erreur renouvellement : {e}")),
    }
  }

  pub(super) fn on_add_subkey(
    &mut self,
    key_fp: String,
    subkey_type: crate::gpg::SubkeyType,
  ) -> Task<Message> {
    Task::perform(
      blocking_task(move || {
        crate::gpg::add_subkey(
          &key_fp,
          subkey_type.algo(),
          subkey_type.usage(),
          &KeyExpiry::TwoYears,
        )
      }),
      Message::AddSubkeyDone,
    )
  }

  pub(super) fn on_add_subkey_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        let s = self.set_status(StatusKind::Success, "Sous-clef créée".to_string());
        let reload = self.reload_keys();
        if let Some(ref fp) = self.selected.clone() {
          if let Some(publish) = self.auto_republish_task(fp) {
            return Task::batch([s, reload, publish]);
          }
        }
        Task::batch([s, reload])
      }
      Err(e) => self.set_status(
        StatusKind::Error,
        format!("Erreur création sous-clef : {e}"),
      ),
    }
  }

  pub(super) fn on_rotate_subkey_execute(
    &mut self,
    key_fp: String,
    subkey_fp: String,
  ) -> Task<Message> {
    let expiry = match self.pending.take() {
      Some(PendingOp::Renewal(r)) => r.expiry,
      _ => KeyExpiry::default(),
    };
    let Some(key) = self.key_by_fp(&key_fp) else {
      return Task::none();
    };
    let subkey_usage = key
      .subkeys
      .iter()
      .find(|s| s.fingerprint == subkey_fp)
      .map(|s| s.usage.clone())
      .unwrap_or_default();
    let subkey_type = crate::gpg::SubkeyType::from_usage_flags(&subkey_usage);
    Task::perform(
      blocking_task(move || {
        crate::gpg::rotate_subkey(
          &key_fp,
          &subkey_fp,
          subkey_type.algo(),
          subkey_type.usage(),
          &expiry,
        )
      }),
      Message::RotateSubkeyDone,
    )
  }

  pub(super) fn on_rotate_subkey_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        let s = self.set_status(
          StatusKind::Success,
          "Sous-clef remplacée avec succès".to_string(),
        );
        let reload = self.reload_keys();
        if let Some(ref fp) = self.selected.clone() {
          if let Some(publish) = self.auto_republish_task(fp) {
            return Task::batch([s, reload, publish]);
          }
        }
        Task::batch([s, reload])
      }
      Err(e) => self.set_status(StatusKind::Error, format!("Erreur rotation : {e}")),
    }
  }
}
