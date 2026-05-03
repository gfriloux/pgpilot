use iced::{
  font,
  widget::{button, column, container, row, rule, scrollable, text},
  Alignment, Background, Border, Color, Element, Font, Length, Shadow,
};

use crate::app::{EncryptForm, Message};
use crate::gpg::KeyInfo;
use crate::ui::theme;

fn key_row(key: &KeyInfo, selected: bool) -> Element<'static, Message> {
  let fp = key.fingerprint.clone();
  let label = format!("{} <{}>", key.name, key.email);
  let short_id = key.key_id[key.key_id.len().saturating_sub(8)..].to_string();
  let trusted = key.trust.is_sufficient();

  let trust_icon = text(if trusted { "\u{f058}" } else { "\u{f071}" })
    .font(theme::ICONS)
    .size(12)
    .style(move |_: &iced::Theme| iced::widget::text::Style {
      color: Some(if trusted {
        theme::SUCCESS
      } else {
        theme::PEACH
      }),
    });

  button(
    row![
      text(if selected { "\u{f046}" } else { "\u{f096}" })
        .font(theme::ICONS)
        .size(14),
      column![
        text(label).size(13),
        container(text(short_id).size(11)).style(|_: &iced::Theme| container::Style {
          text_color: Some(theme::TEXT_MUTED),
          ..Default::default()
        }),
      ]
      .spacing(1)
      .width(Length::Fill),
      trust_icon,
    ]
    .spacing(8)
    .align_y(Alignment::Center),
  )
  .on_press(Message::EncryptToggleRecipient(fp))
  .padding([6, 8])
  .width(Length::Fill)
  .style(move |_: &iced::Theme, status| button::Style {
    background: Some(Background::Color(if selected {
      theme::ACCENT_SUBTLE
    } else {
      match status {
        button::Status::Hovered | button::Status::Pressed => theme::HEADER_BG,
        _ => Color::TRANSPARENT,
      }
    })),
    text_color: if selected {
      theme::ACCENT
    } else {
      theme::TEXT_STRONG
    },
    border: Border {
      color: if selected {
        theme::ACCENT_BORDER
      } else {
        Color::TRANSPARENT
      },
      width: if selected { 1.0 } else { 0.0 },
      radius: 6.0.into(),
    },
    shadow: Shadow::default(),
    snap: false,
  })
  .into()
}

pub fn view<'a>(form: &'a EncryptForm, keys: &'a [KeyInfo]) -> Element<'a, Message> {
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

  let section_header = |label: &'static str| {
    container(text(label).size(11).font(bold)).style(|_: &iced::Theme| container::Style {
      text_color: Some(theme::TEXT_MUTED),
      ..Default::default()
    })
  };

  let encr_keys: Vec<&KeyInfo> = keys
    .iter()
    .filter(|k| k.subkeys.iter().any(|sk| sk.usage.contains('E')))
    .collect();

  let mut own_keys: Vec<&KeyInfo> = Vec::new();
  let mut public_keys: Vec<&KeyInfo> = Vec::new();
  for key in &encr_keys {
    if key.has_secret || key.on_card {
      own_keys.push(key);
    } else {
      public_keys.push(key);
    }
  }

  // Build recipient list
  let mut recipient_items: Vec<Element<'static, Message>> = Vec::new();

  if !own_keys.is_empty() {
    recipient_items.push(section_header("Mes clefs").padding([2, 8]).into());
    for key in &own_keys {
      recipient_items.push(key_row(key, form.recipients.contains(&key.fingerprint)));
    }
  }

  if !public_keys.is_empty() {
    if !own_keys.is_empty() {
      recipient_items.push(
        container(rule::horizontal(1).style(|_: &iced::Theme| rule::Style {
          color: theme::BORDER,
          radius: 0.0.into(),
          fill_mode: rule::FillMode::Full,
          snap: false,
        }))
        .padding([4, 0])
        .into(),
      );
    }
    recipient_items.push(section_header("Clefs publiques").padding([2, 8]).into());
    for key in &public_keys {
      recipient_items.push(key_row(key, form.recipients.contains(&key.fingerprint)));
    }
  }

  if encr_keys.is_empty() {
    recipient_items.push(
      container(text("Aucune clef avec capacité de chiffrement.").size(12))
        .padding([8, 8])
        .style(|_: &iced::Theme| container::Style {
          text_color: Some(theme::TEXT_MUTED),
          ..Default::default()
        })
        .into(),
    );
  }

  let recipients_col: Element<'_, Message> = column![
    container(text("Destinataires").size(12).font(bold)).style(|_: &iced::Theme| {
      container::Style {
        text_color: Some(theme::TEXT_SECONDARY),
        ..Default::default()
      }
    }),
    scrollable(column(recipient_items).spacing(2).padding([0, 4])).height(280),
  ]
  .spacing(8)
  .width(Length::FillPortion(45))
  .into();

  // Build file list
  let n_recipients = form.recipients.len();
  let file_items: Vec<Element<'_, Message>> = form
    .files
    .iter()
    .enumerate()
    .map(|(i, path)| {
      let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("fichier")
        .to_string();
      row![
        text("\u{f15b}").font(theme::ICONS).size(12),
        text(name).size(13).width(Length::Fill),
        button(text("\u{f1f8}").font(theme::ICONS).size(12))
          .on_press(Message::EncryptRemoveFile(i))
          .padding([3, 6])
          .style(|_: &iced::Theme, status| button::Style {
            background: Some(Background::Color(match status {
              button::Status::Hovered | button::Status::Pressed => theme::DESTRUCTIVE_HOVER_BG,
              _ => Color::TRANSPARENT,
            })),
            text_color: theme::ERROR,
            border: Border {
              color: Color::TRANSPARENT,
              width: 0.0,
              radius: 4.0.into()
            },
            shadow: Shadow::default(),
            snap: false,
          }),
      ]
      .spacing(6)
      .align_y(Alignment::Center)
      .padding([4, 6])
      .into()
    })
    .collect();

  let add_files_btn = button(
    row![
      text("\u{f067}").font(theme::ICONS).size(12),
      text("Choisir des fichiers...").size(13),
    ]
    .spacing(6)
    .align_y(Alignment::Center),
  )
  .on_press(Message::EncryptPickFiles)
  .width(Length::Fill)
  .padding([8, 12])
  .style(|_: &iced::Theme, status| button::Style {
    background: Some(Background::Color(match status {
      button::Status::Hovered | button::Status::Pressed => theme::ACCENT_SUBTLE,
      _ => Color::TRANSPARENT,
    })),
    text_color: theme::TEXT_STRONG,
    border: Border {
      color: theme::BORDER,
      width: 1.0,
      radius: 6.0.into(),
    },
    shadow: Shadow::default(),
    snap: false,
  });

  let header_label =
    container(text("Fichiers à chiffrer").size(12).font(bold)).style(|_: &iced::Theme| {
      container::Style {
        text_color: Some(theme::TEXT_SECONDARY),
        ..Default::default()
      }
    });

  let files_col: Element<'_, Message> = if form.files.is_empty() {
    let drop_zone = container(
      column![
        text("\u{f093}")
          .font(theme::ICONS)
          .size(28)
          .style(|_: &iced::Theme| {
            iced::widget::text::Style {
              color: Some(theme::TEXT_MUTED),
            }
          }),
        text("Glissez des fichiers ici")
          .size(13)
          .style(|_: &iced::Theme| {
            iced::widget::text::Style {
              color: Some(theme::TEXT_MUTED),
            }
          }),
        button(
          row![
            text("\u{f067}").font(theme::ICONS).size(12),
            text("Choisir des fichiers...").size(13),
          ]
          .spacing(6)
          .align_y(Alignment::Center),
        )
        .on_press(Message::EncryptPickFiles)
        .width(Length::Fill)
        .padding([8, 12])
        .style(|_: &iced::Theme, status| button::Style {
          background: Some(Background::Color(match status {
            button::Status::Hovered | button::Status::Pressed => theme::ACCENT_SUBTLE,
            _ => Color::TRANSPARENT,
          })),
          text_color: theme::TEXT_STRONG,
          border: Border {
            color: theme::BORDER,
            width: 1.0,
            radius: 6.0.into(),
          },
          shadow: Shadow::default(),
          snap: false,
        }),
      ]
      .spacing(10)
      .align_x(Alignment::Center),
    )
    .center_x(Length::Fill)
    .padding([32, 24])
    .style(|_: &iced::Theme| container::Style {
      border: Border {
        color: theme::BORDER,
        width: 1.0,
        radius: 6.0.into(),
      },
      ..Default::default()
    });

    column![header_label, drop_zone,]
      .spacing(8)
      .width(Length::FillPortion(55))
      .into()
  } else {
    column![
      header_label,
      scrollable(column(file_items).spacing(2).padding([0, 4])).height(232),
      add_files_btn,
    ]
    .spacing(8)
    .width(Length::FillPortion(55))
    .into()
  };

  // Action bar
  let can_encrypt = n_recipients > 0 && !form.files.is_empty() && !form.encrypting;

  let n = form.files.len();
  let encrypt_label = if form.encrypting {
    "Chiffrement en cours...".to_string()
  } else if n == 0 {
    "Chiffrer".to_string()
  } else if n == 1 {
    "Chiffrer 1 fichier".to_string()
  } else {
    format!("Chiffrer {n} fichiers")
  };

  let armor = form.armor;

  let fmt_btn = |label: &'static str, active: bool, msg: Message| {
    button(text(label).size(12))
      .on_press(msg)
      .padding([4, 10])
      .style(move |_: &iced::Theme, status| button::Style {
        background: Some(Background::Color(if active {
          theme::ACCENT
        } else {
          match status {
            button::Status::Hovered | button::Status::Pressed => theme::HEADER_BG,
            _ => Color::TRANSPARENT,
          }
        })),
        text_color: if active {
          theme::TEXT_ON_ACCENT
        } else {
          theme::TEXT_SECONDARY
        },
        border: Border {
          color: if active {
            Color::TRANSPARENT
          } else {
            theme::BORDER
          },
          width: 1.0,
          radius: 4.0.into(),
        },
        shadow: Shadow::default(),
        snap: false,
      })
  };

  let encrypt_btn = {
    let style_enabled = can_encrypt;
    let btn = button(text(encrypt_label).size(13)).padding([8, 16]).style(
      move |_: &iced::Theme, status| button::Style {
        background: Some(Background::Color(if style_enabled {
          match status {
            button::Status::Hovered | button::Status::Pressed => theme::ACCENT_HOVER,
            _ => theme::ACCENT,
          }
        } else {
          theme::DISABLED_BG
        })),
        text_color: if style_enabled {
          theme::TEXT_ON_ACCENT
        } else {
          theme::TEXT_MUTED
        },
        border: Border {
          color: Color::TRANSPARENT,
          width: 0.0,
          radius: 6.0.into(),
        },
        shadow: Shadow::default(),
        snap: false,
      },
    );
    if can_encrypt {
      btn.on_press(Message::EncryptExecute)
    } else {
      btn
    }
  };

  let fmt_hint = if armor {
    "Texte ASCII — pour coller dans un email ou un message."
  } else {
    "Binaire compact — pour pièces jointes et stockage."
  };

  let action_bar: Element<'_, Message> = row![
    column![
      row![
        fmt_btn(".gpg (binaire)", !armor, Message::EncryptSetArmor(false)),
        fmt_btn(".asc (ASCII)", armor, Message::EncryptSetArmor(true)),
      ]
      .spacing(4),
      container(text(fmt_hint).size(11)).style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::TEXT_MUTED),
        ..Default::default()
      }),
    ]
    .spacing(4),
    iced::widget::Space::new().width(Length::Fill),
    encrypt_btn,
  ]
  .align_y(Alignment::Center)
  .into();

  let bottom_section: Element<'_, Message> = if let Some(ref untrusted_fps) = form.trust_prompt {
    let key_labels: Vec<Element<'_, Message>> = untrusted_fps
      .iter()
      .filter_map(|fp| keys.iter().find(|k| &k.fingerprint == fp))
      .map(|k| {
        row![
          text("\u{f071}")
            .font(theme::ICONS)
            .size(12)
            .style(|_: &iced::Theme| iced::widget::text::Style {
              color: Some(theme::PEACH),
            }),
          text(format!("{} <{}>", k.name, k.email)).size(13),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .into()
      })
      .collect();

    let cancel_btn = button(text("Annuler").size(13))
      .on_press(Message::EncryptTrustPromptCancel)
      .padding([6, 14])
      .style(|_: &iced::Theme, status| button::Style {
        background: Some(Background::Color(match status {
          button::Status::Hovered | button::Status::Pressed => theme::HEADER_BG,
          _ => Color::TRANSPARENT,
        })),
        text_color: theme::TEXT_SECONDARY,
        border: Border {
          color: theme::BORDER,
          width: 1.0,
          radius: 6.0.into(),
        },
        shadow: Shadow::default(),
        snap: false,
      });

    let confirm_btn = button(text("Chiffrer quand même").size(13))
      .on_press(Message::EncryptTrustPromptConfirm)
      .padding([6, 14])
      .style(|_: &iced::Theme, status| button::Style {
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
        shadow: Shadow::default(),
        snap: false,
      });

    container(
      column![
        row![
          text("\u{f071}")
            .font(theme::ICONS)
            .size(14)
            .style(|_: &iced::Theme| iced::widget::text::Style {
              color: Some(theme::PEACH),
            }),
          text("Clefs non vérifiées").size(13).font(bold),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
        container(
          text(
            "GPG ne peut pas confirmer que ces clefs appartiennent aux personnes indiquées. \
             Vous pouvez chiffrer quand même — le destinataire pourra toujours déchiffrer, \
             mais vous ne pouvez pas garantir son identité."
          )
          .size(12)
        )
        .style(|_: &iced::Theme| container::Style {
          text_color: Some(theme::TEXT_SECONDARY),
          ..Default::default()
        }),
        column(key_labels).spacing(4),
        row![cancel_btn, confirm_btn].spacing(8),
      ]
      .spacing(10),
    )
    .padding([12, 16])
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::ERROR_BG)),
      border: Border {
        color: theme::PEACH,
        width: 1.0,
        radius: 8.0.into(),
      },
      text_color: Some(theme::TEXT_STRONG),
      ..Default::default()
    })
    .into()
  } else {
    action_bar
  };

  // Info banner
  let info_banner: Element<'_, Message> = container(
    row![
      text("\u{f05a}").font(theme::ICONS).size(14),
      text(
        "Chaque destinataire peut déchiffrer le fichier indépendamment avec sa propre clef. \
         Pensez à vous ajouter pour conserver un accès au fichier chiffré."
      )
      .size(12),
    ]
    .spacing(8)
    .align_y(Alignment::Center),
  )
  .padding([8, 12])
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::ACCENT_SUBTLE)),
    border: Border {
      color: theme::ACCENT_BORDER,
      width: 1.0,
      radius: 6.0.into(),
    },
    text_color: Some(theme::TEXT_SECONDARY),
    ..Default::default()
  })
  .into();

  let vsep = rule::vertical(1).style(|_: &iced::Theme| rule::Style {
    color: theme::BORDER,
    radius: 0.0.into(),
    fill_mode: rule::FillMode::Full,
    snap: false,
  });

  let card = container(
    column![
      column![
        text("Chiffrement de fichiers").size(22).font(bold),
        container(text("Sélectionnez les destinataires et les fichiers.").size(13)).style(
          |_: &iced::Theme| container::Style {
            text_color: Some(theme::TEXT_SECONDARY),
            ..Default::default()
          }
        ),
      ]
      .spacing(6),
      separator(),
      info_banner,
      separator(),
      row![recipients_col, container(vsep).padding([0, 8]), files_col,],
      separator(),
      bottom_section,
    ]
    .spacing(16),
  )
  .padding(32)
  .width(720)
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
    scrollable(
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
