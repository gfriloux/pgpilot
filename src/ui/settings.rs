use iced::{
  font,
  widget::{column, container, radio, rule, text, Column},
  Background, Border, Element, Font, Length,
};

use crate::app::{App, Message};
use crate::i18n::Language;
use crate::ui::theme::ThemeVariant;
use crate::ui::{common, theme};

pub fn view(app: &App) -> Element<'_, Message> {
  let s = app.strings;
  let bold = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
  };

  let separator = || {
    rule::horizontal(1).style(|_: &iced::Theme| rule::Style {
      color: theme::border(),
      radius: 0.0.into(),
      fill_mode: rule::FillMode::Full,
      snap: false,
    })
  };

  // --- Theme section ---
  let theme_catppuccin = radio(
    s.settings_theme_catppuccin(),
    ThemeVariant::Catppuccin,
    Some(app.config.theme),
    Message::ThemeChanged,
  );
  let theme_ussr = radio(
    s.settings_theme_ussr(),
    ThemeVariant::Ussr,
    Some(app.config.theme),
    Message::ThemeChanged,
  );

  // --- Scale factor section ---
  // Radio buttons for scale values
  let scale_values: &[(&str, f64)] = &[
    ("50%", 0.5),
    ("75%", 0.75),
    ("100%", 1.0),
    ("125%", 1.25),
    ("150%", 1.5),
    ("175%", 1.75),
    ("200%", 2.0),
  ];

  // Find the closest matching scale value for the radio selection
  let current_scale_selection: Option<u32> = scale_values
    .iter()
    .position(|(_, v)| (v - app.config.scale_factor).abs() < 0.01)
    .map(|i| i as u32);

  let scale_radios: Vec<Element<'_, Message>> = scale_values
    .iter()
    .enumerate()
    .map(|(i, (label, value))| {
      let v = *value;
      radio(*label, i as u32, current_scale_selection, move |_| {
        Message::ScaleFactorChanged(v)
      })
      .into()
    })
    .collect();

  // --- Language section ---
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

  let mut scale_section_children: Vec<Element<'_, Message>> = vec![
    text(s.settings_scale_factor()).size(12).font(bold).into(),
    container(text(s.settings_scale_factor_hint()).size(11))
      .style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::text_muted()),
        ..Default::default()
      })
      .into(),
  ];
  scale_section_children.extend(scale_radios);

  let card = container(
    column![
      text(s.settings_title()).size(22).font(bold),
      separator(),
      // Theme section (first — most visual)
      column![
        text(s.settings_theme()).size(12).font(bold),
        theme_catppuccin,
        theme_ussr,
      ]
      .spacing(8),
      separator(),
      // Scale factor section
      Column::with_children(scale_section_children).spacing(8),
      separator(),
      // Language section
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
    background: Some(Background::Color(theme::card_bg())),
    border: Border {
      color: theme::border(),
      width: 1.0,
      radius: 12.0.into(),
    },
    text_color: Some(theme::text_strong()),
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
    .width(Length::Fill)
    .style(common::scroll_style),
  )
  .height(Length::Fill)
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::sidebar_bg())),
    ..Default::default()
  })
  .into()
}
