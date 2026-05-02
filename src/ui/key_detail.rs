use iced::{
  font,
  widget::{button, column, container, row, text, Row},
  Background, Border, Color, Element, Font, Length,
};

use crate::app::Message;
use crate::gpg::KeyInfo;
use crate::ui::theme;

pub fn view(key: &KeyInfo, idx: usize) -> Element<'_, Message> {
  let expires = key.expires.as_deref().unwrap_or("Aucune expiration");
  let key_type = if key.has_secret {
    "Publique + Privée"
  } else {
    "Publique"
  };

  let bold = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
  };

  let mono = Font {
    family: font::Family::Monospace,
    ..Font::DEFAULT
  };

  let mut action_buttons: Vec<Element<Message>> = vec![button(text("Exporter pub").size(12))
    .on_press(Message::ExportPublicKey(idx))
    .style(|_: &iced::Theme, status: button::Status| button::Style {
      background: Some(Background::Color(match status {
        button::Status::Hovered | button::Status::Pressed => theme::ACCENT_HOVER,
        _ => theme::ACCENT,
      })),
      text_color: Color::WHITE,
      border: Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: 6.0.into(),
      },
      shadow: Default::default(),
    })
    .into()];

  if key.has_secret {
    action_buttons.push(
      button(text("Exporter privée").size(12))
        .on_press(Message::ExportSecretKey(idx))
        .style(|_: &iced::Theme, status: button::Status| button::Style {
          background: Some(Background::Color(match status {
            button::Status::Hovered | button::Status::Pressed => theme::DESTRUCTIVE_HOVER_BG,
            _ => Color::TRANSPARENT,
          })),
          text_color: theme::DESTRUCTIVE,
          border: Border {
            color: theme::DESTRUCTIVE,
            width: 1.0,
            radius: 6.0.into(),
          },
          shadow: Default::default(),
        })
        .into(),
    );
  }

  column![
    container(
      row![
        text(&key.name).size(15).font(bold),
        text(format!("<{}>", key.email)).size(13),
      ]
      .spacing(6),
    )
    .style(|_: &iced::Theme| container::Style {
      text_color: Some(theme::TEXT_STRONG),
      ..Default::default()
    }),
    container(
      text(format_fingerprint(&key.fingerprint))
        .size(11)
        .font(mono),
    )
    .padding([4, 8])
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::HEADER_BG)),
      text_color: Some(theme::TEXT_SECONDARY),
      border: Border {
        color: theme::BORDER,
        width: 1.0,
        radius: 4.0.into(),
      },
      ..Default::default()
    }),
    container(
      row![
        text(key.algo.to_string()).size(12),
        text("·").size(12),
        text(format!("Créée : {}", key.created)).size(12),
        text("·").size(12),
        text(format!("Expire : {}", expires)).size(12),
        text("·").size(12),
        text(key_type).size(12),
      ]
      .spacing(6),
    )
    .style(|_: &iced::Theme| container::Style {
      text_color: Some(theme::TEXT_SECONDARY),
      ..Default::default()
    }),
    Row::with_children(action_buttons).spacing(8),
  ]
  .spacing(10)
  .padding(16)
  .width(Length::Fill)
  .into()
}

fn format_fingerprint(fp: &str) -> String {
  fp.chars()
    .collect::<Vec<_>>()
    .chunks(4)
    .map(|c| c.iter().collect::<String>())
    .collect::<Vec<_>>()
    .join(" ")
}
