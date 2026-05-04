use iced::Task;

use super::{blocking_task, App, CreateKeyForm, Message, StatusKind, View};

impl App {
  pub(super) fn on_create_key_submit(&mut self) -> Task<Message> {
    let name = self.create_form.name.clone();
    let email = self.create_form.email.clone();
    let subkey_expiry = self.create_form.subkey_expiry.clone();
    let include_auth = self.create_form.include_auth;
    self.create_form.submitting = true;
    Task::perform(
      blocking_task(move || crate::gpg::create_key(&name, &email, &subkey_expiry, include_auth)),
      Message::CreateKeyDone,
    )
  }

  pub(super) fn on_create_key_done(&mut self, result: Result<(), String>) -> Task<Message> {
    match result {
      Ok(()) => {
        self.view = View::MyKeys;
        self.create_form = CreateKeyForm::default();
        self.selected = None;
        self.reload_keys()
      }
      Err(e) => {
        self.create_form.submitting = false;
        self.set_status(
          StatusKind::Error,
          format!("{}: {e}", self.strings.err_create_failed()),
        )
      }
    }
  }
}
