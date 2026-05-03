use iced::{
  font,
  widget::{button, column, container, row, scrollable, text},
  Alignment, Background, Border, Color, Element, Font, Length, Shadow,
};

use crate::app::{Message, SignForm};
use crate::gpg::KeyInfo;
use crate::ui::theme;

fn pick_btn<'a>(
  icon: &'static str,
  label: &'static str,
  on_press: Message,
) -> Element<'a, Message> {
  button(
    row![text(icon).font(theme::ICONS).size(12), text(label).size(13),]
      .spacing(6)
      .align_y(Alignment::Center),
  )
  .on_press(on_press)
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
  })
  .into()
}

fn action_btn<'a>(label: &'static str, enabled: bool, on_press: Message) -> Element<'a, Message> {
  let btn = button(text(label).size(13))
    .padding([8, 16])
    .style(move |_: &iced::Theme, status| button::Style {
      background: Some(Background::Color(if enabled {
        match status {
          button::Status::Hovered | button::Status::Pressed => theme::ACCENT_HOVER,
          _ => theme::ACCENT,
        }
      } else {
        theme::DISABLED_BG
      })),
      text_color: if enabled {
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
    });
  if enabled {
    btn.on_press(on_press).into()
  } else {
    btn.into()
  }
}

pub fn view<'a>(form: &'a SignForm, keys: &'a [KeyInfo]) -> Element<'a, Message> {
  let bold = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
  };

  let sign_keys: Vec<&KeyInfo> = keys
    .iter()
    .filter(|k| (k.has_secret || k.on_card) && k.subkeys.iter().any(|sk| sk.usage.contains('S')))
    .collect();

  let sign_file_label: Element<'_, Message> = if let Some(path) = &form.file {
    let name = path
      .file_name()
      .and_then(|n| n.to_str())
      .unwrap_or("fichier")
      .to_string();
    container(
      row![
        text("\u{f058}")
          .font(theme::ICONS)
          .size(12)
          .style(|_: &iced::Theme| iced::widget::text::Style {
            color: Some(theme::SUCCESS),
          }),
        text(name).size(13),
      ]
      .spacing(6)
      .align_y(Alignment::Center),
    )
    .style(|_: &iced::Theme| container::Style {
      text_color: Some(theme::TEXT_STRONG),
      ..Default::default()
    })
    .into()
  } else {
    container(text("Aucun fichier sélectionné").size(13))
      .style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::TEXT_MUTED),
        ..Default::default()
      })
      .into()
  };

  let file_row: Element<'_, Message> = row![
    pick_btn("\u{f15b}", "Choisir un fichier...", Message::SignPickFile),
    sign_file_label,
  ]
  .spacing(12)
  .align_y(Alignment::Center)
  .into();

  let signer_items: Vec<Element<'_, Message>> = if sign_keys.is_empty() {
    vec![
      container(text("Aucune clef privée avec capacité de signature.").size(12))
        .style(|_: &iced::Theme| container::Style {
          text_color: Some(theme::TEXT_MUTED),
          ..Default::default()
        })
        .padding([4, 0])
        .into(),
    ]
  } else {
    sign_keys
      .iter()
      .map(|key| {
        let fp = key.fingerprint.clone();
        let selected = form.signer_fp.as_deref() == Some(&fp);
        let label = format!("{} <{}>", key.name, key.email);
        let short_id = key.key_id[key.key_id.len().saturating_sub(8)..].to_string();
        button(
          row![
            text(if selected { "\u{f192}" } else { "\u{f111}" })
              .font(theme::ICONS)
              .size(12),
            column![
              text(label).size(13),
              text(short_id).size(11).style(|_: &iced::Theme| {
                iced::widget::text::Style {
                  color: Some(theme::TEXT_MUTED),
                }
              }),
            ]
            .spacing(1),
          ]
          .spacing(8)
          .align_y(Alignment::Center),
        )
        .on_press(Message::SignSelectSigner(fp))
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
      })
      .collect()
  };

  let result_el: Element<'_, Message> = if let Some(sig_path) = &form.sign_result {
    let sig_name = sig_path
      .file_name()
      .and_then(|n| n.to_str())
      .unwrap_or("fichier.sig")
      .to_string();
    container(
      row![
        text("\u{f058}")
          .font(theme::ICONS)
          .size(16)
          .style(|_: &iced::Theme| iced::widget::text::Style {
            color: Some(theme::SUCCESS),
          }),
        column![
          text("Signature créée avec succès").size(14).font(bold),
          text(sig_name).size(12).style(|_: &iced::Theme| {
            iced::widget::text::Style {
              color: Some(theme::TEXT_SECONDARY),
            }
          }),
        ]
        .spacing(2),
      ]
      .spacing(10)
      .align_y(Alignment::Center),
    )
    .padding([10, 14])
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::SUCCESS_BG)),
      border: Border {
        color: theme::SUCCESS,
        width: 1.0,
        radius: 8.0.into(),
      },
      text_color: Some(theme::SUCCESS),
      ..Default::default()
    })
    .into()
  } else if form.signing {
    container(text("Signature en cours...").size(13))
      .style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::TEXT_MUTED),
        ..Default::default()
      })
      .into()
  } else {
    iced::widget::Space::new().into()
  };

  let can_sign = form.file.is_some() && form.signer_fp.is_some() && !form.signing;

  let card = container(
    column![
      column![
        row![
          text("\u{f14b}").font(theme::ICONS).size(20),
          text("Signer un fichier").size(22).font(bold),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        container(
          text(
            "Signer un fichier crée une preuve cryptographique que vous en êtes l'auteur. \
             Le fichier original n'est pas modifié — la signature est enregistrée dans un \
             fichier .sig séparé."
          )
          .size(13)
        )
        .style(|_: &iced::Theme| container::Style {
          text_color: Some(theme::TEXT_SECONDARY),
          ..Default::default()
        }),
      ]
      .spacing(6),
      file_row,
      column![
        container(text("Clef signataire").size(12).font(bold)).style(|_: &iced::Theme| {
          container::Style {
            text_color: Some(theme::TEXT_SECONDARY),
            ..Default::default()
          }
        }),
        scrollable(column(signer_items).spacing(2).padding([0, 2])).height(140),
      ]
      .spacing(6),
      {
        let sign_action: Element<'_, Message> = row![
          iced::widget::Space::new().width(Length::Fill),
          action_btn("Signer", can_sign, Message::SignExecute)
        ]
        .into();
        sign_action
      },
      result_el,
    ]
    .spacing(16),
  )
  .padding(32)
  .width(640)
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
