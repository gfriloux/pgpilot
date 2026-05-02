use iced::{
  font,
  widget::{button, column, container, row, text, vertical_rule, Column, Row},
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
  delete_confirming: bool,
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

  action_buttons.push(
    button(icon_row("\u{f1f8}", "Supprimer"))
      .on_press(Message::DeleteKey(idx))
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

  let mut left_items: Vec<Element<Message>> = vec![
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
    left_items.push(
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
    left_items.push(
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
  } else if delete_confirming {
    let (warn_title, warn_body) = if key.on_card {
      (
        "Seul le stub local de la clef sera supprimé.",
        "La clef physique sur la YubiKey ne sera pas affectée.",
      )
    } else if key.has_secret {
      (
        "Opération irréversible : la clef privée sera détruite.",
        "Sans backup, vos données chiffrées seront définitivement irrécupérables.",
      )
    } else {
      (
        "La clef publique sera supprimée de votre trousseau.",
        "Cette opération peut être annulée en réimportant la clef.",
      )
    };

    let mut del_btns: Vec<Element<Message>> = Vec::new();
    if key.has_secret && !key.on_card {
      del_btns.push(
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
          })
          .into(),
      );
    }
    del_btns.push(
      button(icon_row("\u{f1f8}", "Confirmer la suppression"))
        .on_press(Message::DeleteKeyExecute(idx))
        .style(|_: &iced::Theme, status: button::Status| button::Style {
          background: Some(Background::Color(match status {
            button::Status::Hovered | button::Status::Pressed => theme::DESTRUCTIVE_HOVER_BG,
            _ => theme::DESTRUCTIVE,
          })),
          text_color: Color::WHITE,
          border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.0.into(),
          },
          shadow: Default::default(),
        })
        .into(),
    );
    del_btns.push(
      button(icon_row("\u{f00d}", "Annuler"))
        .on_press(Message::DeleteKeyCancel)
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
        })
        .into(),
    );

    left_items.push(
      container(
        column![
          text(warn_title).size(12).font(bold),
          text(warn_body).size(12),
          Row::with_children(del_btns).spacing(8),
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
    left_items.push(Row::with_children(action_buttons).spacing(8).into());
  }

  let left_col = Column::with_children(left_items)
    .spacing(10)
    .padding(16)
    .width(Length::Fill);

  if key.subkeys.is_empty() {
    return left_col.into();
  }

  let subkey_cards: Vec<Element<Message>> = key
    .subkeys
    .iter()
    .map(|sk| {
      let (icon, type_label, type_color) = match sk.usage.as_str() {
        "S" => ("\u{f040}", "Signature", theme::ACCENT),
        "E" => ("\u{f023}", "Chiffrement", theme::SUCCESS),
        "A" => (
          "\u{f084}",
          "Auth SSH",
          Color {
            r: 0.96,
            g: 0.62,
            b: 0.11,
            a: 1.0,
          },
        ),
        _ => ("\u{f0c0}", "Autre", theme::SIDEBAR_TEXT),
      };
      let expires_str = sk.expires.as_deref().unwrap_or("Aucune expiration");

      container(
        column![
          row![
            text(icon).font(theme::ICONS).size(12).color(type_color),
            text(type_label).size(12).font(bold).color(type_color),
          ]
          .spacing(6)
          .align_y(Alignment::Center),
          row![
            column![
              text(&sk.algo).size(10),
              text(format_fingerprint(&sk.short_id)).font(mono).size(10),
            ]
            .spacing(2)
            .width(Length::Fill),
            button(text("\u{f0c5}").font(theme::ICONS).size(11))
              .on_press(Message::CopyToClipboard(sk.fingerprint.clone()))
              .style(|_: &iced::Theme, status: button::Status| button::Style {
                background: Some(Background::Color(match status {
                  button::Status::Hovered | button::Status::Pressed => theme::SIDEBAR_HOVER_BG,
                  _ => Color::TRANSPARENT,
                })),
                text_color: theme::SIDEBAR_TEXT_MUTED,
                border: Border {
                  color: Color::TRANSPARENT,
                  width: 0.0,
                  radius: 4.0.into(),
                },
                shadow: Default::default(),
              }),
          ]
          .spacing(4)
          .align_y(Alignment::Center),
          text(expires_str).size(10).color(theme::SIDEBAR_TEXT_MUTED),
        ]
        .spacing(4),
      )
      .padding(8)
      .width(Length::Fill)
      .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(theme::SIDEBAR_BG)),
        border: Border {
          color: Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.08,
          },
          width: 1.0,
          radius: 6.0.into(),
        },
        text_color: Some(theme::SIDEBAR_TEXT),
        ..Default::default()
      })
      .into()
    })
    .collect();

  let right_col = Column::with_children(subkey_cards)
    .spacing(8)
    .padding([16, 12])
    .width(220);

  row![left_col, vertical_rule(1), right_col]
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
