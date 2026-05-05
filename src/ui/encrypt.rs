use iced::{
  font,
  widget::{button, column, container, row, rule, scrollable, text},
  Alignment, Background, Border, Color, Element, Font, Length, Shadow,
};

use crate::app::{EncryptForm, Message};
use crate::gpg::KeyInfo;
use crate::i18n::Strings;
use crate::ui::{common, theme};

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
        theme::success()
      } else {
        theme::peach()
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
          text_color: Some(theme::text_muted()),
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
      theme::accent_subtle()
    } else {
      match status {
        button::Status::Hovered | button::Status::Pressed => theme::header_bg(),
        _ => Color::TRANSPARENT,
      }
    })),
    text_color: if selected {
      theme::accent()
    } else {
      theme::text_strong()
    },
    border: Border {
      color: if selected {
        theme::accent_border()
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

pub fn view<'a>(
  form: &'a EncryptForm,
  keys: &'a [KeyInfo],
  s: &'static dyn Strings,
) -> Element<'a, Message> {
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

  let section_header = |label: &'static str| {
    container(text(label).size(11).font(bold)).style(|_: &iced::Theme| container::Style {
      text_color: Some(theme::text_muted()),
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
    recipient_items.push(
      section_header(s.encrypt_tab_my_keys())
        .padding([2, 8])
        .into(),
    );
    for key in &own_keys {
      recipient_items.push(key_row(key, form.recipients.contains(&key.fingerprint)));
    }
  }

  if !public_keys.is_empty() {
    if !own_keys.is_empty() {
      recipient_items.push(
        container(rule::horizontal(1).style(|_: &iced::Theme| rule::Style {
          color: theme::border(),
          radius: 0.0.into(),
          fill_mode: rule::FillMode::Full,
          snap: false,
        }))
        .padding([4, 0])
        .into(),
      );
    }
    recipient_items.push(
      section_header(s.encrypt_tab_public_keys())
        .padding([2, 8])
        .into(),
    );
    for key in &public_keys {
      recipient_items.push(key_row(key, form.recipients.contains(&key.fingerprint)));
    }
  }

  if encr_keys.is_empty() {
    recipient_items.push(
      container(text(s.encrypt_no_keys()).size(12))
        .padding([8, 8])
        .style(|_: &iced::Theme| container::Style {
          text_color: Some(theme::text_muted()),
          ..Default::default()
        })
        .into(),
    );
  }

  let recipients_col: Element<'_, Message> = column![
    container(text(s.encrypt_recipients()).size(12).font(bold)).style(|_: &iced::Theme| {
      container::Style {
        text_color: Some(theme::text_secondary()),
        ..Default::default()
      }
    }),
    scrollable(column(recipient_items).spacing(2).padding([0, 4]))
      .height(280)
      .style(common::scroll_style),
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
              button::Status::Hovered | button::Status::Pressed => theme::destructive_hover_bg(),
              _ => Color::TRANSPARENT,
            })),
            text_color: theme::error(),
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
      text(s.encrypt_choose_files()).size(13),
    ]
    .spacing(6)
    .align_y(Alignment::Center),
  )
  .on_press(Message::EncryptPickFiles)
  .width(Length::Fill)
  .padding([8, 12])
  .style(|_: &iced::Theme, status| button::Style {
    background: Some(Background::Color(match status {
      button::Status::Hovered | button::Status::Pressed => theme::accent_subtle(),
      _ => Color::TRANSPARENT,
    })),
    text_color: theme::text_strong(),
    border: Border {
      color: theme::border(),
      width: 1.0,
      radius: 6.0.into(),
    },
    shadow: Shadow::default(),
    snap: false,
  });

  let header_label =
    container(text(s.encrypt_add_files()).size(12).font(bold)).style(|_: &iced::Theme| {
      container::Style {
        text_color: Some(theme::text_secondary()),
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
              color: Some(theme::text_muted()),
            }
          }),
        text(s.encrypt_drop_hint())
          .size(13)
          .style(|_: &iced::Theme| {
            iced::widget::text::Style {
              color: Some(theme::text_muted()),
            }
          }),
        button(
          row![
            text("\u{f067}").font(theme::ICONS).size(12),
            text(s.encrypt_choose_files()).size(13),
          ]
          .spacing(6)
          .align_y(Alignment::Center),
        )
        .on_press(Message::EncryptPickFiles)
        .width(Length::Fill)
        .padding([8, 12])
        .style(|_: &iced::Theme, status| button::Style {
          background: Some(Background::Color(match status {
            button::Status::Hovered | button::Status::Pressed => theme::accent_subtle(),
            _ => Color::TRANSPARENT,
          })),
          text_color: theme::text_strong(),
          border: Border {
            color: theme::border(),
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
        color: theme::border(),
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
      scrollable(column(file_items).spacing(2).padding([0, 4]))
        .height(232)
        .style(common::scroll_style),
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
    s.encrypt_in_progress().to_string()
  } else if n == 0 {
    s.btn_encrypt().to_string()
  } else if n == 1 {
    format!("{} 1 fichier", s.btn_encrypt())
  } else {
    format!("{} {n} fichiers", s.btn_encrypt())
  };

  let armor = form.armor;

  let fmt_btn = |label: &'static str, active: bool, msg: Message| {
    button(text(label).size(12))
      .on_press(msg)
      .padding([4, 10])
      .style(move |_: &iced::Theme, status| button::Style {
        background: Some(Background::Color(if active {
          theme::accent()
        } else {
          match status {
            button::Status::Hovered | button::Status::Pressed => theme::header_bg(),
            _ => Color::TRANSPARENT,
          }
        })),
        text_color: if active {
          theme::text_on_accent()
        } else {
          theme::text_secondary()
        },
        border: Border {
          color: if active {
            Color::TRANSPARENT
          } else {
            theme::border()
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
            button::Status::Hovered | button::Status::Pressed => theme::accent_hover(),
            _ => theme::accent(),
          }
        } else {
          theme::disabled_bg()
        })),
        text_color: if style_enabled {
          theme::text_on_accent()
        } else {
          theme::text_muted()
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
    s.encrypt_format_ascii_desc()
  } else {
    s.encrypt_format_binary_desc()
  };

  let action_bar: Element<'_, Message> = row![
    column![
      row![
        fmt_btn(
          s.encrypt_format_binary(),
          !armor,
          Message::EncryptSetArmor(false)
        ),
        fmt_btn(
          s.encrypt_format_armor(),
          armor,
          Message::EncryptSetArmor(true)
        ),
      ]
      .spacing(4),
      container(text(fmt_hint).size(11)).style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::text_muted()),
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
              color: Some(theme::peach()),
            }),
          text(format!("{} <{}>", k.name, k.email)).size(13),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .into()
      })
      .collect();

    let cancel_btn = button(text(s.btn_cancel()).size(13))
      .on_press(Message::EncryptTrustPromptCancel)
      .padding([6, 14])
      .style(|_: &iced::Theme, status| button::Style {
        background: Some(Background::Color(match status {
          button::Status::Hovered | button::Status::Pressed => theme::header_bg(),
          _ => Color::TRANSPARENT,
        })),
        text_color: theme::text_secondary(),
        border: Border {
          color: theme::border(),
          width: 1.0,
          radius: 6.0.into(),
        },
        shadow: Shadow::default(),
        snap: false,
      });

    let confirm_btn = button(text(s.btn_confirm()).size(13))
      .on_press(Message::EncryptTrustPromptConfirm)
      .padding([6, 14])
      .style(|_: &iced::Theme, status| button::Style {
        background: Some(Background::Color(match status {
          button::Status::Hovered | button::Status::Pressed => theme::accent_hover(),
          _ => theme::accent(),
        })),
        text_color: theme::text_on_accent(),
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
              color: Some(theme::peach()),
            }),
          text(s.encrypt_trust_warning_title()).size(13).font(bold),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
        container(text(s.encrypt_trust_warning_body()).size(12)).style(|_: &iced::Theme| {
          container::Style {
            text_color: Some(theme::text_secondary()),
            ..Default::default()
          }
        }),
        column(key_labels).spacing(4),
        row![cancel_btn, confirm_btn].spacing(8),
      ]
      .spacing(10),
    )
    .padding([12, 16])
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::error_bg())),
      border: Border {
        color: theme::peach(),
        width: 1.0,
        radius: 8.0.into(),
      },
      text_color: Some(theme::text_strong()),
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
      text(s.encrypt_multi_recipient_hint()).size(12),
    ]
    .spacing(8)
    .align_y(Alignment::Center),
  )
  .padding([8, 12])
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::accent_subtle())),
    border: Border {
      color: theme::accent_border(),
      width: 1.0,
      radius: 6.0.into(),
    },
    text_color: Some(theme::text_secondary()),
    ..Default::default()
  })
  .into();

  let vsep = rule::vertical(1).style(|_: &iced::Theme| rule::Style {
    color: theme::border(),
    radius: 0.0.into(),
    fill_mode: rule::FillMode::Full,
    snap: false,
  });

  let card = container(
    column![
      column![
        text(theme::flavor(s.encrypt_title(), "Encrypt for the People"))
          .size(22)
          .font(theme::flavor_title_font()),
        container(text(s.encrypt_select_hint()).size(13)).style(|_: &iced::Theme| {
          container::Style {
            text_color: Some(theme::text_secondary()),
            ..Default::default()
          }
        }),
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
    scrollable(
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
