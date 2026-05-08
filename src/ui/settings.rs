use iced::{
  font,
  widget::{button, column, container, row, slider, text, Space},
  Background, Border, Color, Element, Font, Length,
};

use crate::app::{App, Message};
use crate::i18n::Language;
use crate::ui::theme::ThemeVariant;
use crate::ui::{button_styles, common, theme, ussr_assets};

const SCALE_VALUES: [f64; 7] = [0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];
const SCALE_LABELS: [&str; 7] = ["50%", "75%", "100%", "125%", "150%", "175%", "200%"];

pub fn view(app: &App) -> Element<'_, Message> {
  let s = app.strings;
  let bold = Font {
    weight: font::Weight::Bold,
    ..theme::heading_font()
  };

  // --- 1. Sélecteur de thème avec mini-prévisualisation ---
  let theme_section = {
    let mk_preview = |variant: ThemeVariant| -> Element<Message> {
      let selected = app.config.theme == variant;

      // Mini-aperçu du thème : sidebar + barre accent + lignes contenu
      let (sidebar_col, content_col, accent_bar, line_color) = match variant {
        ThemeVariant::Catppuccin => (
          Color::from_rgb(0.161, 0.173, 0.275),
          Color::from_rgb(0.188, 0.204, 0.322),
          Color::from_rgb(0.792, 0.620, 0.902),
          Color::from_rgb(0.310, 0.337, 0.510),
        ),
        ThemeVariant::Ussr => (
          Color::from_rgb(0.078, 0.031, 0.031), // #140808 sidebar
          Color::from_rgb(0.933, 0.902, 0.851), // #eee6d9 contenu
          Color::from_rgb(0.800, 0.200, 0.200), // #cc3333 accent rouge
          Color::from_rgb(0.545, 0.271, 0.075), // bordure USSR
        ),
      };

      let mini_sidebar = container(Space::new().width(Length::Fill).height(Length::Fill))
        .width(28)
        .height(Length::Fill)
        .style(move |_| container::Style {
          background: Some(Background::Color(sidebar_col)),
          border: Border {
            color: accent_bar,
            width: 2.0,
            radius: 0.0.into(),
          },
          ..Default::default()
        });

      let mini_bar = container(Space::new().width(Length::Fill).height(4))
        .width(Length::Fill)
        .style(move |_| container::Style {
          background: Some(Background::Color(accent_bar)),
          ..Default::default()
        });

      let mini_line = |w: f32| {
        container(Space::new().width(w).height(3.0_f32)).style(move |_| container::Style {
          background: Some(Background::Color(line_color)),
          ..Default::default()
        })
      };

      let preview_body = container(row![
        mini_sidebar,
        column![
          mini_bar,
          Space::new().height(6),
          mini_line(50.0),
          Space::new().height(4),
          mini_line(35.0),
        ]
        .spacing(0)
        .padding([8, 10]),
      ])
      .width(Length::Fill)
      .height(70)
      .style(move |_| container::Style {
        background: Some(Background::Color(content_col)),
        ..Default::default()
      });

      let name = text(match variant {
        ThemeVariant::Catppuccin => s.settings_theme_catppuccin(),
        ThemeVariant::Ussr => s.settings_theme_ussr(),
      })
      .size(12)
      .font(theme::nav_font())
      .color(if selected {
        theme::accent()
      } else {
        theme::text_muted()
      });

      let card = column![preview_body, container(name).padding([6, 10])].spacing(0);

      button(card)
        .on_press(Message::ThemeChanged(variant))
        .padding(0)
        .style(move |_, _| iced::widget::button::Style {
          background: Some(Background::Color(theme::card_bg())),
          border: Border {
            color: if selected {
              theme::accent()
            } else {
              theme::border()
            },
            width: if selected { 2.0 } else { 1.0 },
            radius: 6.0.into(),
          },
          text_color: theme::text_strong(),
          ..Default::default()
        })
        .into()
    };

    column![
      text(s.settings_theme()).size(12).font(bold),
      row![
        mk_preview(ThemeVariant::Catppuccin),
        mk_preview(ThemeVariant::Ussr)
      ]
      .spacing(12),
    ]
    .spacing(10)
  };

  // --- 2. Échelle UI — règle graduée ---
  let scale_index = SCALE_VALUES
    .iter()
    .position(|&v| (v - app.config.scale_factor).abs() < 0.01)
    .unwrap_or(3) as u8;

  let tick_labels: Vec<Element<Message>> = SCALE_LABELS
    .iter()
    .enumerate()
    .flat_map(|(i, &label)| {
      let is_active = i == scale_index as usize;
      let t: Element<Message> = text(label)
        .size(10)
        .color(if is_active {
          theme::accent()
        } else {
          theme::text_muted()
        })
        .into();
      if i < SCALE_LABELS.len() - 1 {
        vec![t, Space::new().width(Length::Fill).into()]
      } else {
        vec![t]
      }
    })
    .collect();

  let scale_section = column![
    text(s.settings_scale_factor()).size(12).font(bold),
    container(text(s.settings_scale_factor_hint()).size(11)).style(|_: &iced::Theme| {
      container::Style {
        text_color: Some(theme::text_muted()),
        ..Default::default()
      }
    }),
    slider(0..=6u8, scale_index, move |i| {
      Message::ScaleFactorChanged(SCALE_VALUES[i as usize])
    })
    .style(|_: &iced::Theme, _| iced::widget::slider::Style {
      rail: iced::widget::slider::Rail {
        backgrounds: (
          Background::Color(theme::accent()),
          Background::Color(theme::border()),
        ),
        width: 4.0,
        border: Border::default(),
      },
      handle: iced::widget::slider::Handle {
        shape: iced::widget::slider::HandleShape::Circle { radius: 8.0 },
        background: Background::Color(theme::accent()),
        border_width: 2.0,
        border_color: theme::card_bg(),
      },
    }),
    row(tick_labels),
  ]
  .spacing(8);

  // --- 3. Langue — grille de boutons ---
  let is_en = app.config.language == Language::English;
  let is_fr = app.config.language == Language::French;

  let lang_section = column![
    text(s.settings_language()).size(12).font(bold),
    row![
      button(text(s.settings_language_english()).size(13))
        .on_press(Message::ChangeLanguage(Language::English))
        .padding([8, 20])
        .style(move |theme, status| if is_en {
          button_styles::primary()(theme, status)
        } else {
          button_styles::ghost_neutral()(theme, status)
        }),
      button(text(s.settings_language_french()).size(13))
        .on_press(Message::ChangeLanguage(Language::French))
        .padding([8, 20])
        .style(move |theme, status| if is_fr {
          button_styles::primary()(theme, status)
        } else {
          button_styles::ghost_neutral()(theme, status)
        }),
    ]
    .spacing(8),
  ]
  .spacing(8);

  // --- Assemblage de la card ---
  let card_content = column![
    text(s.settings_title())
      .size(22)
      .font(theme::heading_font()),
    common::star_separator(),
    theme_section,
    common::star_separator(),
    scale_section,
    common::star_separator(),
    lang_section,
  ]
  .spacing(16);

  common::page_layout(common::card_medium_with_banner(
    card_content,
    ussr_assets::banner(29),
  ))
}
