use iced::{
  font,
  widget::{button, column, container, row, text, Column, Row},
  Alignment, Background, Border, Color, Element, Font, Length,
};

use crate::app::Message;
use crate::gpg::KeyInfo;
use crate::ui::theme;

pub fn view(
  key: &KeyInfo,
  idx: usize,
  card_connected: bool,
  confirming: bool,
) -> Element<'_, Message> {
  let expires = key.expires.as_deref().unwrap_or("Aucune expiration");
  let key_type = if key.on_card {
    "Sur YubiKey"
  } else if key.has_secret {
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

  let icon_row = |icon: &'static str, label: &'static str| {
    row![text(icon).font(theme::ICONS).size(12), text(label).size(12),]
      .spacing(6)
      .align_y(Alignment::Center)
  };

  let mut action_buttons: Vec<Element<Message>> =
    vec![button(icon_row("\u{f019}", "Exporter pub"))
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
      button(icon_row("\u{f023}", "Exporter privée"))
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

    if !key.on_card {
      let migrate_btn = button(icon_row("\u{f287}", "Migrer vers YubiKey")).style(
        |_: &iced::Theme, status: button::Status| button::Style {
          background: Some(Background::Color(match status {
            button::Status::Hovered | button::Status::Pressed => theme::ACCENT_HOVER,
            button::Status::Disabled => theme::DISABLED_BG,
            _ => theme::ACCENT,
          })),
          text_color: match status {
            button::Status::Disabled => theme::TEXT_MUTED,
            _ => Color::WHITE,
          },
          border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.0.into(),
          },
          shadow: Default::default(),
        },
      );
      let migrate_btn = if card_connected {
        migrate_btn.on_press(Message::MoveToCard(idx))
      } else {
        migrate_btn
      };
      action_buttons.push(migrate_btn.into());
    }
  }

  let mut items: Vec<Element<Message>> = vec![
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
    })
    .into(),
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
    })
    .into(),
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
    })
    .into(),
  ];

  if let (true, Some(serial)) = (key.on_card, &key.card_serial) {
    items.push(
      container(
        row![
          text("\u{f283}").font(theme::ICONS).size(12),
          text(format!("YubiKey · {serial}")).size(12),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
      )
      .style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::ACCENT),
        ..Default::default()
      })
      .into(),
    );
  }

  if confirming {
    items.push(
      container(
        column![
          text("Opération irréversible : la clef privée va être déplacée sur la YubiKey.")
            .size(12)
            .font(bold),
          text(
            "Sans backup, si la YubiKey est perdue ou détruite, \
             les données chiffrées seront irrécupérables.",
          )
          .size(12),
          row![
            button(icon_row("\u{f019}", "Exporter d'abord"))
              .on_press(Message::ExportSecretKey(idx))
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
              }),
            button(icon_row("\u{f00c}", "J'ai un backup \u{2192} Continuer"))
              .on_press(Message::MoveToCardExecute(idx))
              .style(|_: &iced::Theme, status: button::Status| button::Style {
                background: Some(Background::Color(match status {
                  button::Status::Hovered | button::Status::Pressed => Color {
                    r: theme::SUCCESS.r * 0.8,
                    g: theme::SUCCESS.g * 0.8,
                    b: theme::SUCCESS.b * 0.8,
                    a: 1.0,
                  },
                  _ => theme::SUCCESS,
                })),
                text_color: Color::WHITE,
                border: Border {
                  color: Color::TRANSPARENT,
                  width: 0.0,
                  radius: 6.0.into(),
                },
                shadow: Default::default(),
              }),
            button(icon_row("\u{f00d}", "Annuler"))
              .on_press(Message::MoveToCardCancel)
              .style(|_: &iced::Theme, status: button::Status| button::Style {
                background: Some(Background::Color(match status {
                  button::Status::Hovered | button::Status::Pressed => Color {
                    a: 0.08,
                    ..theme::TEXT_SECONDARY
                  },
                  _ => Color::TRANSPARENT,
                })),
                text_color: theme::TEXT_SECONDARY,
                border: Border {
                  color: theme::TEXT_SECONDARY,
                  width: 1.0,
                  radius: 6.0.into(),
                },
                shadow: Default::default(),
              }),
          ]
          .spacing(8),
        ]
        .spacing(8),
      )
      .padding(12)
      .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(theme::ERROR_BG)),
        text_color: Some(theme::TEXT_STRONG),
        border: Border {
          color: theme::ERROR,
          width: 1.0,
          radius: 6.0.into(),
        },
        ..Default::default()
      })
      .into(),
    );
  } else {
    items.push(Row::with_children(action_buttons).spacing(8).into());
  }

  Column::with_children(items)
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
