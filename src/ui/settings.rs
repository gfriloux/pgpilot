use iced::{
  font,
  widget::{column, container, radio, rule, text},
  Background, Border, Element, Font, Length,
};

use crate::app::{App, Message};
use crate::i18n::Language;
use crate::ui::theme;

pub fn view(app: &App) -> Element<'_, Message> {
  let s = app.strings;
  let bold = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
  };

  let separator = || {
    rule::horizontal(1).style(|_: &iced::Theme| rule::Style {
      color: theme::BORDER,
      radius: 0.0.into(),
      fill_mode: rule::FillMode::Full,
      snap: false,
    })
  };

  let lang_en = radio(
    s.settings_language_english(),
    Language::English,
    Some(app.config.language),
    Message::ChangeLanguage,
  );

  let lang_fr = radio(
    s.settings_language_french(),
    Language::French,
    Some(app.config.language),
    Message::ChangeLanguage,
  );

  let card = container(
    column![
      text(s.settings_title()).size(22).font(bold),
      separator(),
      column![
        text(s.settings_language()).size(12).font(bold),
        lang_en,
        lang_fr,
      ]
      .spacing(8),
    ]
    .spacing(16),
  )
  .padding(32)
  .width(400)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::CARD_BG)),
    border: Border {
      color: theme::BORDER,
      width: 1.0,
      radius: 12.0.into(),
    },
    text_color: Some(theme::TEXT_STRONG),
    ..Default::default()
  });

  container(
    iced::widget::scrollable(
      container(card)
        .center_x(Length::Fill)
        .padding([24, 0])
        .width(Length::Fill),
    )
    .height(Length::Fill)
    .width(Length::Fill),
  )
  .height(Length::Fill)
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::SIDEBAR_BG)),
    ..Default::default()
  })
  .into()
}
