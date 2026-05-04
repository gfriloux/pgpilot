use iced::{
  font,
  widget::{button, column, container, row, rule, scrollable, text},
  Alignment, Background, Border, Color, Element, Font, Length, Shadow,
};

use crate::app::{Message, SignForm};
use crate::gpg::KeyInfo;
use crate::i18n::Strings;
use crate::ui::{common, theme};

pub fn view<'a>(
  form: &'a SignForm,
  keys: &'a [KeyInfo],
  s: &'static dyn Strings,
) -> Element<'a, Message> {
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
            color: Some(theme::success()),
          }),
        text(name).size(13),
      ]
      .spacing(6)
      .align_y(Alignment::Center),
    )
    .style(|_: &iced::Theme| container::Style {
      text_color: Some(theme::text_strong()),
      ..Default::default()
    })
    .into()
  } else {
    container(text(s.no_file_selected()).size(13))
      .style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::text_muted()),
        ..Default::default()
      })
      .into()
  };

  let file_row: Element<'_, Message> = row![
    common::pick_btn("\u{f15b}", s.sign_select_file(), Message::SignPickFile),
    sign_file_label,
  ]
  .spacing(12)
  .align_y(Alignment::Center)
  .into();

  let signer_items: Vec<Element<'_, Message>> = if sign_keys.is_empty() {
    vec![
      container(text("Aucune clef privée avec capacité de signature.").size(12))
        .style(|_: &iced::Theme| container::Style {
          text_color: Some(theme::text_muted()),
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
                  color: Some(theme::text_muted()),
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
            color: Some(theme::success()),
          }),
        column![
          text(s.status_file_signed()).size(14).font(bold),
          text(sig_name).size(12).style(|_: &iced::Theme| {
            iced::widget::text::Style {
              color: Some(theme::text_secondary()),
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
      background: Some(Background::Color(theme::success_bg())),
      border: Border {
        color: theme::success(),
        width: 1.0,
        radius: 8.0.into(),
      },
      text_color: Some(theme::success()),
      ..Default::default()
    })
    .into()
  } else if form.signing {
    container(text(s.verify_in_progress()).size(13))
      .style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::text_muted()),
        ..Default::default()
      })
      .into()
  } else {
    iced::widget::Space::new().into()
  };

  let can_sign = form.file.is_some() && form.signer_fp.is_some() && !form.signing;

  let rule_sep = || {
    rule::horizontal(1).style(|_: &iced::Theme| rule::Style {
      color: theme::border(),
      radius: 0.0.into(),
      fill_mode: rule::FillMode::Full,
      snap: true,
    })
  };

  let card = container(
    column![
      column![
        row![
          text("\u{f14b}").font(theme::ICONS).size(20),
          text(s.sign_title()).size(22).font(bold),
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
          text_color: Some(theme::text_secondary()),
          ..Default::default()
        }),
      ]
      .spacing(6),
      rule_sep(),
      file_row,
      rule_sep(),
      column![
        container(text(s.sign_select_key()).size(12).font(bold)).style(|_: &iced::Theme| {
          container::Style {
            text_color: Some(theme::text_secondary()),
            ..Default::default()
          }
        }),
        container(scrollable(column(signer_items).spacing(2).padding([0, 2])).height(140))
          .padding(4)
          .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(theme::header_bg())),
            border: Border {
              color: theme::border(),
              width: 1.0,
              radius: 4.0.into(),
            },
            ..Default::default()
          }),
      ]
      .spacing(6),
      rule_sep(),
      {
        let sign_action: Element<'_, Message> = row![
          iced::widget::Space::new().width(Length::Fill),
          common::action_btn(s.btn_sign(), can_sign, Message::SignExecute)
        ]
        .into();
        sign_action
      },
      rule_sep(),
      result_el,
    ]
    .spacing(16),
  )
  .padding(32)
  .width(640)
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
    .width(Length::Fill),
  )
  .height(Length::Fill)
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::sidebar_bg())),
    ..Default::default()
  })
  .into()
}
