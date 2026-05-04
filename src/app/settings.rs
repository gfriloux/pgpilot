use crate::app::StatusKind;
use crate::app::{App, Message};
use crate::i18n::{self, Language};
use iced::Task;

impl App {
  pub(super) fn on_language_changed(&mut self, lang: Language) -> Task<Message> {
    self.config.language = lang;
    self.strings = i18n::strings_for(lang);

    if let Err(e) = self.config.save() {
      return self.set_status(
        StatusKind::Error,
        format!("{}: {e}", self.strings.err_save_config_failed()),
      );
    }

    self.set_status(
      StatusKind::Success,
      self.strings.status_preferences_saved().to_string(),
    )
  }
}
