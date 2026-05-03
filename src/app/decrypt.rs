use std::path::PathBuf;

use iced::Task;

use super::{blocking_task, App, Message, StatusKind};

impl App {
  pub(super) fn on_decrypt_pick_files(&mut self) -> Task<Message> {
    Task::perform(
      blocking_task(|| {
        Ok(
          rfd::FileDialog::new()
            .set_title("Choisir des fichiers à déchiffrer")
            .add_filter("Fichiers GPG", &["gpg", "asc"])
            .pick_files()
            .unwrap_or_default(),
        )
      }),
      Message::DecryptFilesPicked,
    )
  }

  pub(super) fn on_decrypt_files_picked(
    &mut self,
    result: Result<Vec<PathBuf>, String>,
  ) -> Task<Message> {
    let new_files = match result {
      Ok(files) => files,
      Err(e) => {
        self.status = Some((StatusKind::Error, e));
        return Task::none();
      }
    };

    let mut tasks = Vec::new();
    for f in new_files {
      if !self.decrypt_form.files.contains(&f) {
        self.decrypt_form.files.push(f.clone());
        self
          .decrypt_form
          .file_statuses
          .insert(f.clone(), crate::gpg::DecryptStatus::Checking);
        let path = f.clone();
        tasks.push(Task::perform(
          blocking_task(move || crate::gpg::inspect_decrypt(&path)),
          move |r| Message::DecryptFileInspected(f.clone(), r),
        ));
      }
    }

    if tasks.is_empty() {
      Task::none()
    } else {
      Task::batch(tasks)
    }
  }

  pub(super) fn on_decrypt_file_inspected(
    &mut self,
    path: PathBuf,
    result: Result<crate::gpg::DecryptStatus, String>,
  ) -> Task<Message> {
    let status = result.unwrap_or(crate::gpg::DecryptStatus::Unknown);
    self.decrypt_form.file_statuses.insert(path, status);
    Task::none()
  }

  pub(super) fn on_decrypt_execute(&mut self) -> Task<Message> {
    let files: Vec<PathBuf> = self
      .decrypt_form
      .files
      .iter()
      .filter(|f| {
        self.decrypt_form.file_statuses.get(*f) != Some(&crate::gpg::DecryptStatus::NoKey)
      })
      .cloned()
      .collect();

    if files.is_empty() {
      self.status = Some((
        StatusKind::Error,
        "Aucun fichier déchiffrable sélectionné.".to_string(),
      ));
      return Task::none();
    }

    self.decrypt_form.decrypting = true;
    self.status = None;
    Task::perform(
      blocking_task(move || crate::gpg::decrypt_files(&files)),
      Message::DecryptDone,
    )
  }

  pub(super) fn on_decrypt_done(&mut self, result: Result<Vec<String>, String>) -> Task<Message> {
    self.decrypt_form.decrypting = false;
    match result {
      Ok(names) => {
        let summary = if names.len() == 1 {
          format!("Déchiffré : {}", names[0])
        } else {
          format!("{} fichiers déchiffrés", names.len())
        };
        self.status = Some((StatusKind::Success, summary));
        self.decrypt_form.files.clear();
        self.decrypt_form.file_statuses.clear();
      }
      Err(e) => {
        self.status = Some((StatusKind::Error, format!("Erreur déchiffrement : {e}")));
      }
    }
    Task::none()
  }
}
