use crate::app::StatusKind;
use crate::app::{App, Message};
use crate::i18n::{self, Language};
use crate::ui::theme::{self, ThemeVariant};
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

  pub(super) fn on_scale_factor_changed(&mut self, v: f64) -> Task<Message> {
    self.config.scale_factor = v;
    if let Err(e) = self.config.save() {
      return self.set_status(
        StatusKind::Error,
        format!("{}: {e}", self.strings.err_save_config_failed()),
      );
    }
    // NOTE: iced 0.14 does not expose a `.scale_factor()` method on the
    // application builder that reads from app state at runtime. The scale
    // factor is persisted to config and will take effect on the next launch.
    // When iced adds dynamic scale-factor support, wire it up here.
    self.set_status(
      StatusKind::Success,
      self.strings.status_preferences_saved().to_string(),
    )
  }

  pub(super) fn on_theme_changed(&mut self, v: ThemeVariant) -> Task<Message> {
    self.config.theme = v;
    theme::set_active(v);
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
