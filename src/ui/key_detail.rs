use chrono::{Duration, Utc};
use iced::{
  font,
  widget::{button, column, container, row, rule, text, vertical_rule, Column, Row},
  Alignment, Background, Border, Color, Element, Font, Length,
};

use crate::app::{KeyserverStatus, Message};
use crate::gpg::types::SubkeyInfo;
use crate::gpg::{KeyExpiry, KeyInfo, Keyserver, SubkeyType};
use crate::ui::theme;

pub struct ViewCtx {
  pub card_connected: bool,
  pub confirming: bool,
  pub delete_confirming: bool,
  pub export_pub_menu: bool,
  pub renewing_subkey: Option<(String, KeyExpiry)>,
  pub publish_confirming: Option<Keyserver>,
  pub keyserver_status: KeyserverStatus,
}

pub fn view(key: &KeyInfo, ctx: ViewCtx) -> Element<'_, Message> {
  let ViewCtx {
    card_connected,
    confirming,
    delete_confirming,
    export_pub_menu,
    renewing_subkey,
    publish_confirming,
    keyserver_status,
  } = ctx;

  let bold = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
  };

  let mono = Font {
    family: font::Family::Monospace,
    ..Font::DEFAULT
  };

  let can_edit = key.has_secret && (!key.on_card || card_connected);

  let left_col = left_column_items(
    key,
    keyserver_status,
    card_connected,
    confirming,
    delete_confirming,
    export_pub_menu,
    publish_confirming,
    bold,
    mono,
  );

  if key.subkeys.is_empty() && !can_edit {
    return left_col.into();
  }

  let right_col = subkey_column(key, can_edit, renewing_subkey, bold, mono);

  row![
    left_col,
    vertical_rule(1).style(|_: &iced::Theme| rule::Style {
      color: theme::BORDER,
      width: 1,
      radius: 0.0.into(),
      fill_mode: rule::FillMode::Full,
    }),
    right_col,
  ]
  .width(Length::Fill)
  .into()
}

fn action_buttons(
  key: &KeyInfo,
  keyserver_status: KeyserverStatus,
  card_connected: bool,
) -> Vec<Element<'_, Message>> {
  let icon_row = |icon: &'static str, label: &'static str| {
    row![text(icon).font(theme::ICONS).size(12), text(label).size(12),]
      .spacing(6)
      .align_y(Alignment::Center)
  };

  let mut btns: Vec<Element<Message>> = vec![button(icon_row("\u{f019}", "Exporter pub"))
    .on_press(Message::ExportPublicKeyMenu(key.fingerprint.clone()))
    .style(|_: &iced::Theme, status: button::Status| button::Style {
      background: Some(Background::Color(match status {
        button::Status::Hovered | button::Status::Pressed => theme::ACCENT_HOVER,
        _ => theme::ACCENT,
      })),
      text_color: theme::TEXT_ON_ACCENT,
      border: Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: 6.0.into(),
      },
      shadow: Default::default(),
    })
    .into()];

  if key.has_secret {
    btns.push(
      button(icon_row("\u{f0c7}", "Sauvegarder"))
        .on_press(Message::BackupKey(key.fingerprint.clone()))
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
            _ => theme::TEXT_ON_ACCENT,
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
        migrate_btn.on_press(Message::MoveToCard(key.fingerprint.clone()))
      } else {
        migrate_btn
      };
      btns.push(migrate_btn.into());
    }
  }

  if keyserver_status != KeyserverStatus::Published {
    btns.push(
      button(icon_row("\u{f1d8}", "Publier"))
        .on_press(Message::PublishKey)
        .style(|_: &iced::Theme, status: button::Status| button::Style {
          background: Some(Background::Color(match status {
            button::Status::Hovered | button::Status::Pressed => theme::ACCENT_HOVER,
            _ => Color::TRANSPARENT,
          })),
          text_color: theme::ACCENT,
          border: Border {
            color: theme::ACCENT_BORDER,
            width: 1.0,
            radius: 6.0.into(),
          },
          shadow: Default::default(),
        })
        .into(),
    );
  }

  btns.push(
    button(icon_row("\u{f1f8}", "Supprimer"))
      .on_press(Message::DeleteKey(key.fingerprint.clone()))
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

  btns
}

fn keyserver_badge(status: KeyserverStatus, fingerprint: &str) -> Element<'_, Message> {
  match status {
    KeyserverStatus::Published => {
      let share_url = format!(
        "https://keys.openpgp.org/search?q={}",
        fingerprint.to_uppercase()
      );
      container(
        row![
          text("\u{f058}")
            .font(theme::ICONS)
            .size(11)
            .color(theme::SUCCESS),
          text("Publiée sur keys.openpgp.org")
            .size(11)
            .color(theme::SUCCESS)
            .width(Length::Fill),
          button(
            row![
              text("\u{f0c1}").font(theme::ICONS).size(10),
              text("Lien").size(10),
            ]
            .spacing(3)
            .align_y(Alignment::Center),
          )
          .on_press(Message::CopyToClipboard(share_url))
          .style(|_: &iced::Theme, status: button::Status| button::Style {
            background: Some(Background::Color(match status {
              button::Status::Hovered | button::Status::Pressed => theme::ACCENT_SUBTLE,
              _ => Color::TRANSPARENT,
            })),
            text_color: theme::ACCENT,
            border: Border {
              color: theme::ACCENT_BORDER,
              width: 1.0,
              radius: 4.0.into(),
            },
            shadow: Default::default(),
          }),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
      )
      .into()
    }
    KeyserverStatus::NotPublished => container(
      row![
        text("\u{f10c}")
          .font(theme::ICONS)
          .size(11)
          .color(theme::TEXT_MUTED),
        text("Pas encore publiée").size(11).color(theme::TEXT_MUTED),
      ]
      .spacing(5)
      .align_y(Alignment::Center),
    )
    .into(),
    KeyserverStatus::Checking => container(
      text("Vérification sur keys.openpgp.org\u{2026}")
        .size(11)
        .color(theme::TEXT_MUTED),
    )
    .into(),
    KeyserverStatus::Unknown => container(text("").size(0)).into(),
  }
}

fn migration_modal(key: &KeyInfo, bold: Font) -> Element<'_, Message> {
  let icon_row = |icon: &'static str, label: &'static str| {
    row![text(icon).font(theme::ICONS).size(12), text(label).size(12),]
      .spacing(6)
      .align_y(Alignment::Center)
  };

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
        button(icon_row("\u{f0c7}", "Sauvegarder d'abord"))
          .on_press(Message::BackupKey(key.fingerprint.clone()))
          .style(|_: &iced::Theme, status: button::Status| button::Style {
            background: Some(Background::Color(match status {
              button::Status::Hovered | button::Status::Pressed => theme::ACCENT_HOVER,
              _ => theme::ACCENT,
            })),
            text_color: theme::TEXT_ON_ACCENT,
            border: Border {
              color: Color::TRANSPARENT,
              width: 0.0,
              radius: 6.0.into(),
            },
            shadow: Default::default(),
          }),
        button(icon_row("\u{f00c}", "J'ai un backup \u{2192} Continuer"))
          .on_press(Message::MoveToCardExecute(key.fingerprint.clone()))
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
            text_color: theme::TEXT_ON_ACCENT,
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
  .into()
}

fn delete_modal(key: &KeyInfo, bold: Font) -> Element<'_, Message> {
  let icon_row = |icon: &'static str, label: &'static str| {
    row![text(icon).font(theme::ICONS).size(12), text(label).size(12),]
      .spacing(6)
      .align_y(Alignment::Center)
  };

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
        .on_press(Message::BackupKey(key.fingerprint.clone()))
        .style(|_: &iced::Theme, status: button::Status| button::Style {
          background: Some(Background::Color(match status {
            button::Status::Hovered | button::Status::Pressed => theme::ACCENT_HOVER,
            _ => theme::ACCENT,
          })),
          text_color: theme::TEXT_ON_ACCENT,
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
      .on_press(Message::DeleteKeyExecute(key.fingerprint.clone()))
      .style(|_: &iced::Theme, status: button::Status| button::Style {
        background: Some(Background::Color(match status {
          button::Status::Hovered | button::Status::Pressed => theme::DESTRUCTIVE_HOVER_BG,
          _ => theme::DESTRUCTIVE,
        })),
        text_color: theme::TEXT_ON_ACCENT,
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
  .into()
}

fn publish_modal(key: &KeyInfo, selected_ks: Keyserver, bold: Font) -> Element<'_, Message> {
  let icon_row = |icon: &'static str, label: &'static str| {
    row![text(icon).font(theme::ICONS).size(12), text(label).size(12),]
      .spacing(6)
      .align_y(Alignment::Center)
  };

  let description = match selected_ks {
    Keyserver::Openpgp => format!(
      "Recommandé · Respecte le RGPD. \
       Un email de validation sera envoyé à {} \
       pour rendre votre identité visible dans les recherches.",
      key.email
    ),
    Keyserver::Ubuntu => "Publication immédiate sans validation. \
       Votre clef sera visible dans les recherches instantanément \
       et ne pourra jamais être supprimée du keyserver."
      .to_string(),
  };

  let ks_btn = |label: &'static str, value: Keyserver| {
    let selected = selected_ks == value;
    button(text(label).size(11))
      .on_press(Message::PublishKeyserverChanged(value))
      .style(move |_: &iced::Theme, _| button::Style {
        background: Some(Background::Color(if selected {
          theme::ACCENT
        } else {
          Color::TRANSPARENT
        })),
        text_color: if selected {
          theme::TEXT_ON_ACCENT
        } else {
          theme::TEXT_SECONDARY
        },
        border: Border {
          color: if selected {
            Color::TRANSPARENT
          } else {
            theme::ACCENT_BORDER
          },
          width: 1.0,
          radius: 4.0.into(),
        },
        shadow: Default::default(),
      })
  };

  container(
    column![
      row![
        text("\u{f1d8}")
          .font(theme::ICONS)
          .size(12)
          .color(theme::ACCENT),
        text("Publier sur un keyserver")
          .size(12)
          .font(bold)
          .color(theme::ACCENT),
      ]
      .spacing(6)
      .align_y(Alignment::Center),
      row![
        ks_btn("keys.openpgp.org", Keyserver::Openpgp),
        ks_btn("keyserver.ubuntu.com", Keyserver::Ubuntu),
      ]
      .spacing(8),
      container(text(description).size(11))
        .padding([8, 10])
        .style(|_: &iced::Theme| container::Style {
          background: Some(Background::Color(theme::ACCENT_SUBTLE)),
          border: Border {
            color: theme::ACCENT_BORDER,
            width: 1.0,
            radius: 4.0.into(),
          },
          text_color: Some(theme::TEXT_SECONDARY),
          ..Default::default()
        }),
      row![
        button(icon_row("\u{f1d8}", "Publier"))
          .on_press(Message::PublishKeyExecute(key.fingerprint.clone()))
          .style(|_: &iced::Theme, status: button::Status| button::Style {
            background: Some(Background::Color(match status {
              button::Status::Hovered | button::Status::Pressed => theme::ACCENT_HOVER,
              _ => theme::ACCENT,
            })),
            text_color: theme::TEXT_ON_ACCENT,
            border: Border {
              color: Color::TRANSPARENT,
              width: 0.0,
              radius: 6.0.into(),
            },
            shadow: Default::default(),
          }),
        button(icon_row("\u{f00d}", "Annuler"))
          .on_press(Message::PublishKeyCancel)
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
              color: theme::BORDER,
              width: 1.0,
              radius: 6.0.into(),
            },
            shadow: Default::default(),
          }),
      ]
      .spacing(8),
    ]
    .spacing(10),
  )
  .padding(12)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::ACCENT_SUBTLE)),
    border: Border {
      color: theme::ACCENT_BORDER,
      width: 1.0,
      radius: 6.0.into(),
    },
    ..Default::default()
  })
  .into()
}

fn export_pub_modal(key: &KeyInfo, bold: Font) -> Element<'_, Message> {
  let menu_btn = |icon: &'static str, label: &'static str, msg: Message| {
    button(
      row![text(icon).font(theme::ICONS).size(12), text(label).size(12)]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .on_press(msg)
    .width(Length::Fill)
    .style(|_: &iced::Theme, status: button::Status| button::Style {
      background: Some(Background::Color(match status {
        button::Status::Hovered | button::Status::Pressed => theme::ACCENT_SUBTLE,
        _ => Color::TRANSPARENT,
      })),
      text_color: theme::TEXT_STRONG,
      border: Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: 6.0.into(),
      },
      shadow: Default::default(),
    })
  };

  container(
    column![
      row![
        text("\u{f019}")
          .font(theme::ICONS)
          .size(12)
          .color(theme::ACCENT),
        text("Exporter la clef publique")
          .size(12)
          .font(bold)
          .color(theme::ACCENT),
      ]
      .spacing(6)
      .align_y(Alignment::Center),
      menu_btn(
        "\u{f0c7}",
        "Enregistrer sur le disque",
        Message::ExportPublicKey(key.fingerprint.clone())
      ),
      menu_btn(
        "\u{f0c5}",
        "Copier dans le presse-papier",
        Message::ExportPublicKeyClipboard(key.fingerprint.clone())
      ),
      menu_btn(
        "\u{f0c1}",
        "Obtenir un lien public (paste.rs)",
        Message::ExportPublicKeyUpload(key.fingerprint.clone())
      ),
      button(text("Annuler").size(12))
        .on_press(Message::ExportPublicKeyMenuCancel)
        .style(|_: &iced::Theme, status: button::Status| button::Style {
          background: Some(Background::Color(match status {
            button::Status::Hovered | button::Status::Pressed => Color {
              a: 0.06,
              ..theme::TEXT_SECONDARY
            },
            _ => Color::TRANSPARENT,
          })),
          text_color: theme::TEXT_SECONDARY,
          border: Border {
            color: theme::BORDER,
            width: 1.0,
            radius: 6.0.into(),
          },
          shadow: Default::default(),
        }),
    ]
    .spacing(6),
  )
  .padding(12)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::ACCENT_SUBTLE)),
    border: Border {
      color: theme::ACCENT_BORDER,
      width: 1.0,
      radius: 6.0.into(),
    },
    ..Default::default()
  })
  .into()
}

#[allow(clippy::too_many_arguments)]
fn left_column_items(
  key: &KeyInfo,
  keyserver_status: KeyserverStatus,
  card_connected: bool,
  confirming: bool,
  delete_confirming: bool,
  export_pub_menu: bool,
  publish_confirming: Option<Keyserver>,
  bold: Font,
  mono: Font,
) -> Column<'_, Message> {
  let expires = key.expires.as_deref().unwrap_or("Aucune expiration");
  let key_type = if key.on_card {
    "Sur YubiKey"
  } else if key.has_secret {
    "Publique + Privée"
  } else {
    "Publique"
  };

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
    keyserver_badge(keyserver_status, &key.fingerprint),
    container(
      row![
        text(key.algo.clone()).size(12),
        text("·").size(12),
        text(format!("Créée : {}", key.created)).size(12),
        text("·").size(12),
        text(format!("Expire : {expires}")).size(12),
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
    items.push(migration_modal(key, bold));
  } else if delete_confirming {
    items.push(delete_modal(key, bold));
  } else if let Some(selected_ks) = publish_confirming {
    items.push(publish_modal(key, selected_ks, bold));
  } else if export_pub_menu {
    items.push(export_pub_modal(key, bold));
  } else {
    items.push(
      Row::with_children(action_buttons(key, keyserver_status, card_connected))
        .spacing(8)
        .into(),
    );
  }

  Column::with_children(items)
    .spacing(10)
    .padding(16)
    .width(Length::Fill)
}

fn subkey_renewal_form<'a>(
  renewal_expiry: &KeyExpiry,
  type_color: Color,
  key_fp: &'a str,
  subkey_fp: &'a str,
) -> Element<'a, Message> {
  let until = expiry_until_date(renewal_expiry);

  let expiry_btn = |label: &'static str, value: KeyExpiry| {
    let selected = renewal_expiry == &value;
    button(text(label).size(11))
      .on_press(Message::RenewSubkeyExpiryChanged(value))
      .style(move |_: &iced::Theme, _| button::Style {
        background: Some(Background::Color(if selected {
          type_color
        } else {
          Color::TRANSPARENT
        })),
        text_color: if selected {
          theme::TEXT_ON_ACCENT
        } else {
          theme::SIDEBAR_TEXT_MUTED
        },
        border: Border {
          color: if selected {
            Color::TRANSPARENT
          } else {
            Color {
              a: 0.3,
              ..theme::SIDEBAR_TEXT_MUTED
            }
          },
          width: 1.0,
          radius: 4.0.into(),
        },
        shadow: Default::default(),
      })
  };

  let key_fp = key_fp.to_string();
  let subkey_fp = subkey_fp.to_string();

  column![
    text(format!("Valide jusqu'au : {until}"))
      .size(11)
      .color(theme::SIDEBAR_TEXT),
    row![
      expiry_btn("1 an", KeyExpiry::OneYear),
      expiry_btn("2 ans", KeyExpiry::TwoYears),
      expiry_btn("5 ans", KeyExpiry::FiveYears),
    ]
    .spacing(4),
    row![
      button(text("↺ Renouveler").size(11))
        .on_press(Message::RenewSubkeyExecute)
        .width(Length::Fill)
        .style(
          move |_: &iced::Theme, status: button::Status| button::Style {
            background: Some(Background::Color(match status {
              button::Status::Hovered | button::Status::Pressed => Color {
                a: 0.85,
                ..type_color
              },
              _ => type_color,
            })),
            text_color: theme::TEXT_ON_ACCENT,
            border: Border {
              color: Color::TRANSPARENT,
              width: 0.0,
              radius: 4.0.into(),
            },
            shadow: Default::default(),
          }
        ),
      button(text("⟲ Remplacer").size(11))
        .on_press(Message::RotateSubkeyExecute(
          key_fp.clone(),
          subkey_fp.clone()
        ))
        .width(Length::Fill)
        .style(|_: &iced::Theme, status: button::Status| button::Style {
          background: Some(Background::Color(match status {
            button::Status::Hovered | button::Status::Pressed => theme::SIDEBAR_HOVER_BG,
            _ => Color::TRANSPARENT,
          })),
          text_color: theme::SIDEBAR_TEXT_MUTED,
          border: Border {
            color: Color {
              a: 0.4,
              ..theme::SIDEBAR_TEXT_MUTED
            },
            width: 1.0,
            radius: 4.0.into(),
          },
          shadow: Default::default(),
        }),
    ]
    .spacing(4),
    button(text("Annuler").size(11))
      .on_press(Message::RenewSubkeyCancel)
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
  .spacing(8)
  .into()
}

fn subkey_card_body<'a>(
  sk: &'a SubkeyInfo,
  key_fp: &'a str,
  can_edit: bool,
  mono: Font,
) -> Element<'a, Message> {
  let expires_str = sk.expires.as_deref().unwrap_or("Aucune expiration");
  column![
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
    if can_edit {
      row![
        text(expires_str)
          .size(10)
          .color(theme::SIDEBAR_TEXT_MUTED)
          .width(Length::Fill),
        button(text("\u{f021}").font(theme::ICONS).size(10))
          .on_press(Message::RenewSubkey(
            key_fp.to_string(),
            sk.fingerprint.clone()
          ))
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
      .align_y(Alignment::Center)
    } else {
      row![text(expires_str).size(10).color(theme::SIDEBAR_TEXT_MUTED)].spacing(0)
    },
  ]
  .spacing(4)
  .into()
}

fn subkey_ghost_card<'a>(
  subkey_type: SubkeyType,
  icon: &'static str,
  type_label: &'static str,
  type_color: Color,
  key_fp: &'a str,
  bold: Font,
) -> Element<'a, Message> {
  let dimmed = Color {
    a: 0.45,
    ..type_color
  };
  container(
    row![
      row![
        text(icon).font(theme::ICONS).size(12).color(dimmed),
        text(type_label).size(12).font(bold).color(dimmed),
      ]
      .spacing(6)
      .align_y(Alignment::Center)
      .width(Length::Fill),
      button(
        row![
          text("\u{f067}").font(theme::ICONS).size(11),
          text("Créer").size(11),
        ]
        .spacing(4)
        .align_y(Alignment::Center),
      )
      .on_press(Message::AddSubkey(key_fp.to_string(), subkey_type))
      .style(
        move |_: &iced::Theme, status: button::Status| button::Style {
          background: Some(Background::Color(match status {
            button::Status::Hovered | button::Status::Pressed => Color {
              a: 0.20,
              ..type_color
            },
            _ => Color {
              a: 0.10,
              ..type_color
            },
          })),
          text_color: type_color,
          border: Border {
            color: Color {
              a: 0.30,
              ..type_color
            },
            width: 1.0,
            radius: 4.0.into(),
          },
          shadow: Default::default(),
        }
      ),
    ]
    .spacing(8)
    .align_y(Alignment::Center),
  )
  .padding(8)
  .width(Length::Fill)
  .style(move |_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::SIDEBAR_BG)),
    border: Border {
      color: Color {
        a: 0.25,
        ..type_color
      },
      width: 1.0,
      radius: 6.0.into(),
    },
    ..Default::default()
  })
  .into()
}

fn subkey_column(
  key: &KeyInfo,
  can_edit: bool,
  renewing_subkey: Option<(String, KeyExpiry)>,
  bold: Font,
  mono: Font,
) -> Column<'_, Message> {
  let standard_types = [
    (SubkeyType::Sign, "\u{f040}", "Signature", theme::ACCENT),
    (SubkeyType::Encr, "\u{f023}", "Chiffrement", theme::SUCCESS),
    (SubkeyType::Auth, "\u{f084}", "Auth SSH", theme::PEACH),
  ];

  let find_subkey = |usage_char: char| -> Option<&SubkeyInfo> {
    key.subkeys.iter().find(|sk| sk.usage.contains(usage_char))
  };

  let subkey_cards: Vec<Element<Message>> = standard_types
    .iter()
    .filter_map(|(subkey_type, icon, type_label, type_color)| {
      let (subkey_type, icon, type_label, type_color) =
        (*subkey_type, *icon, *type_label, *type_color);

      if let Some(sk) = find_subkey(subkey_type.usage_char()) {
        let header = row![
          text(icon).font(theme::ICONS).size(12).color(type_color),
          text(type_label).size(12).font(bold).color(type_color),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        let body: Element<Message> = if renewing_subkey
          .as_ref()
          .is_some_and(|(r, _)| r == &sk.fingerprint)
        {
          let renewal_expiry = renewing_subkey
            .as_ref()
            .map_or(&KeyExpiry::TwoYears, |(_, e)| e);
          subkey_renewal_form(
            renewal_expiry,
            type_color,
            &key.fingerprint,
            &sk.fingerprint,
          )
        } else {
          subkey_card_body(sk, &key.fingerprint, can_edit, mono)
        };

        Some(
          container(column![header, body].spacing(6))
            .padding(8)
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
              background: Some(Background::Color(theme::SIDEBAR_BG)),
              border: Border {
                color: theme::BORDER,
                width: 1.0,
                radius: 6.0.into(),
              },
              text_color: Some(theme::SIDEBAR_TEXT),
              ..Default::default()
            })
            .into(),
        )
      } else if can_edit {
        Some(subkey_ghost_card(
          subkey_type,
          icon,
          type_label,
          type_color,
          &key.fingerprint,
          bold,
        ))
      } else {
        None
      }
    })
    .collect();

  Column::with_children(subkey_cards)
    .spacing(8)
    .padding([16, 12])
    .width(220)
}

fn expiry_until_date(expiry: &KeyExpiry) -> String {
  let days = match expiry {
    KeyExpiry::OneYear => 365,
    KeyExpiry::TwoYears => 730,
    KeyExpiry::FiveYears => 1825,
  };
  let future = Utc::now() + Duration::days(days);
  future.format("%Y-%m-%d").to_string()
}

fn format_fingerprint(fp: &str) -> String {
  fp.chars()
    .collect::<Vec<_>>()
    .chunks(4)
    .map(|c| c.iter().collect::<String>())
    .collect::<Vec<_>>()
    .join(" ")
}
