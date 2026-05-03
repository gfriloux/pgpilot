use iced::Task;

use super::{blocking_task, App, KeyserverStatus, Message, View};

impl App {
  pub(super) fn on_keys_loaded(
    &mut self,
    result: Result<(Vec<crate::gpg::KeyInfo>, bool), String>,
  ) -> Task<Message> {
    match result {
      Ok((keys, card_connected)) => {
        self.keys = keys;
        self.card_connected = card_connected;
        self.loading = false;
        let new_fps: Vec<String> = self
          .keys
          .iter()
          .filter(|k| !self.keyserver_statuses.contains_key(&k.fingerprint))
          .map(|k| k.fingerprint.clone())
          .collect();
        for fp in &new_fps {
          self
            .keyserver_statuses
            .insert(fp.clone(), KeyserverStatus::Checking);
        }
        if !new_fps.is_empty() {
          return Task::batch(new_fps.into_iter().map(|fp| {
            Task::perform(
              blocking_task(move || crate::gpg::check_keyserver(&fp)),
              Message::KeyserverStatusLoaded,
            )
          }));
        }
        Task::none()
      }
      Err(e) => {
        self.error = Some(e);
        self.loading = false;
        Task::none()
      }
    }
  }

  pub(super) fn on_nav_changed(&mut self, view: View) -> Task<Message> {
    let is_health = view == View::Health;
    self.view = view;
    self.selected = None;
    self.reset_pending_ops();
    if is_health {
      self.health_loading = true;
      let keys = self.keys.clone();
      return Task::perform(
        blocking_task(move || Ok(crate::gpg::run_all_checks(&keys))),
        Message::HealthChecksLoaded,
      );
    }
    Task::none()
  }

  pub(super) fn on_key_selected(&mut self, fp: String) -> Task<Message> {
    self.selected = Some(fp.clone());
    self.reset_pending_ops();
    let unknown = matches!(
      self.keyserver_statuses.get(&fp),
      None | Some(KeyserverStatus::Unknown)
    );
    if unknown {
      self
        .keyserver_statuses
        .insert(fp.clone(), KeyserverStatus::Checking);
      return Task::perform(
        blocking_task(move || crate::gpg::check_keyserver(&fp)),
        Message::KeyserverStatusLoaded,
      );
    }
    Task::none()
  }

  pub(super) fn on_keyserver_status_loaded(
    &mut self,
    result: Result<(String, bool), String>,
  ) -> Task<Message> {
    match result {
      Ok((fp, found)) => {
        self.keyserver_statuses.insert(
          fp,
          if found {
            KeyserverStatus::Published
          } else {
            KeyserverStatus::NotPublished
          },
        );
      }
      Err(_) => {
        for status in self.keyserver_statuses.values_mut() {
          if *status == KeyserverStatus::Checking {
            *status = KeyserverStatus::Unknown;
          }
        }
      }
    }
    Task::none()
  }
}
