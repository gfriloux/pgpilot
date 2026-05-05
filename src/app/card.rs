use iced::Task;

use crate::gpg::TrustLevel;

use super::{blocking_task, truncate_error, App, Message, PendingOp, StatusKind};

/// Async helper: shows a save-file dialog then copies the revocation cert to
/// the chosen destination.  Returns `Ok(None)` when the user cancelled.
async fn export_rev_cert_to_file(src: std::path::PathBuf) -> Result<Option<String>, String> {
  let default_name = src
    .file_name()
    .and_then(|n| n.to_str())
    .unwrap_or("revocation.rev")
    .to_string();

  let handle = rfd::AsyncFileDialog::new()
    .set_file_name(&default_name)
    .add_filter("Revocation certificate", &["rev", "asc"])
    .save_file()
    .await;

  let dest = match handle {
    None => return Ok(None),
    Some(h) => h.path().to_path_buf(),
  };

  let filename = dest
    .file_name()
    .and_then(|n| n.to_str())
    .unwrap_or("revocation.rev")
    .to_string();

  tokio::task::spawn_blocking(move || -> anyhow::Result<Option<String>> {
    std::fs::copy(&src, &dest).map_err(|e| anyhow::anyhow!("Copy failed: {e}"))?;
    Ok(Some(filename))
  })
  .await
  .unwrap_or_else(|e| Err(anyhow::anyhow!(e)))
  .map_err(|e| e.to_string())
}

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

  pub(super) fn on_export_revocation_cert(&mut self, fp: String) -> Task<Message> {
    // Resolve the cert path synchronously — it is a deterministic filesystem check.
    let homedir = match crate::gpg::gnupg_dir() {
      Ok(h) => h,
      Err(e) => {
        return self.set_status(
          StatusKind::Error,
          truncate_error(format!("GPG dir error: {e}")),
        );
      }
    };
    let src = match crate::gpg::revocation_cert_path(&homedir, &fp) {
      Ok(Some(p)) => p,
      Ok(None) => {
        return self.set_status(
          StatusKind::Error,
          self.strings.revocation_cert_missing().to_string(),
        );
      }
      Err(e) => {
        return self.set_status(StatusKind::Error, truncate_error(format!("Error: {e}")));
      }
    };
    Task::perform(export_rev_cert_to_file(src), |result| match result {
      Ok(None) => Message::DismissStatus(u32::MAX),
      Ok(Some(name)) => Message::RevocationCertGenerated(Ok(name)),
      Err(e) => Message::RevocationCertGenerated(Err(e)),
    })
  }

  pub(super) fn on_copy_revocation_cert_path(&mut self, path: String) -> Task<Message> {
    let s = self.set_status(
      StatusKind::Success,
      self.strings.status_key_copied().to_string(),
    );
    Task::batch([s, iced::clipboard::write(path)])
  }

  pub(super) fn on_generate_revocation_cert(&mut self, fp: String) -> Task<Message> {
    Task::perform(
      blocking_task(move || {
        let homedir = crate::gpg::gnupg_dir()?;
        crate::gpg::generate_revocation_cert(&homedir, &fp)
          .map(|p| p.to_string_lossy().into_owned())
      }),
      Message::RevocationCertGenerated,
    )
  }

  pub(super) fn on_revocation_cert_generated(
    &mut self,
    result: Result<String, String>,
  ) -> Task<Message> {
    match result {
      Ok(_path) => {
        let reload = self.reload_keys();
        let s = self.set_status(
          StatusKind::Success,
          self.strings.status_revocation_cert_generated().to_string(),
        );
        Task::batch([s, reload])
      }
      Err(e) => self.set_status(StatusKind::Error, truncate_error(format!("Error: {e}"))),
    }
  }
}
