use iced::Task;

use crate::gpg::{ExpiryWarning, SubkeyType};

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
        self.expiry_warnings = self.compute_expiry_warnings();
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

  pub(super) fn on_nav_back(&mut self) -> Task<Message> {
    let dest = self.previous_view.take().unwrap_or(View::MyKeys);
    self.on_nav_changed(dest)
  }

  pub(super) fn on_nav_changed(&mut self, view: View) -> Task<Message> {
    if matches!(
      view,
      View::CreateKey | View::Import | View::ChatRoom(_) | View::ChatNewRoom | View::ChatJoinRoom
    ) {
      self.previous_view = Some(self.view.clone());
    }
    let is_health = view == View::Health;

    // Logique additionnelle pour les vues chat.
    match &view {
      View::ChatRoom(room_id) => {
        self.active_room = Some(room_id.clone());
        self.chat_input.clear();
        let rid = room_id.clone();
        self.view = view;
        self.selected = None;
        self.reset_pending_ops();
        self.decrypt_form = crate::app::DecryptForm::default();
        return self.ensure_chat_started(rid);
      }
      View::ChatList | View::ChatNewRoom | View::ChatJoinRoom => {
        self.active_room = None;
      }
      _ => {
        // Pour les vues hors chat, on ne touche pas à active_room.
      }
    }

    self.view = view;
    self.selected = None;
    self.reset_pending_ops();
    self.decrypt_form = crate::app::DecryptForm::default();
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

  fn compute_expiry_warnings(&self) -> Vec<ExpiryWarning> {
    let now = chrono::Utc::now();
    let threshold = now + chrono::Duration::days(90);
    let mut warnings = vec![];
    for key in &self.keys {
      for subkey in &key.subkeys {
        let Some(ref expires_str) = subkey.expires else {
          continue;
        };
        let Ok(naive_date) = chrono::NaiveDate::parse_from_str(expires_str, "%Y-%m-%d") else {
          continue;
        };
        let expires_at: chrono::DateTime<chrono::Utc> = chrono::DateTime::from_naive_utc_and_offset(
          naive_date.and_hms_opt(0, 0, 0).unwrap_or_default(),
          chrono::Utc,
        );
        if expires_at > now && expires_at < threshold {
          let subkey_type = if subkey.usage.contains('E') {
            Some(SubkeyType::Encr)
          } else if subkey.usage.contains('A') {
            Some(SubkeyType::Auth)
          } else if subkey.usage.contains('S') {
            Some(SubkeyType::Sign)
          } else {
            None
          };
          warnings.push(ExpiryWarning {
            key_fp: key.fingerprint.clone(),
            key_name: key.name.clone(),
            subkey_type,
            expires_at,
          });
        }
      }
    }
    warnings
  }
}
