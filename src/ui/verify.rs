use iced::{
  font,
  widget::{column, container, row, rule, scrollable, text},
  Alignment, Background, Border, Color, Element, Font, Length,
};

use crate::app::{Message, SignForm};
use crate::gpg::VerifyOutcome;
use crate::ui::{common, theme};

fn result_card<'a>(
  bold: Font,
  icon: &'static str,
  accent: Color,
  bg: Color,
  title: &'static str,
  message: &'a str,
  detail: &'a str,
) -> Element<'a, Message> {
  container(
    column![
      row![
        text(icon)
          .font(theme::ICONS)
          .size(16)
          .style(move |_: &iced::Theme| iced::widget::text::Style {
            color: Some(accent),
          }),
        text(title).size(16).font(bold),
      ]
      .spacing(8)
      .align_y(Alignment::Center),
      container(text(message).size(12)).style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::TEXT_SECONDARY),
        ..Default::default()
      }),
      container(text(detail).size(11).font(Font::MONOSPACE)).style(|_: &iced::Theme| {
        container::Style {
          text_color: Some(theme::TEXT_MUTED),
          ..Default::default()
        }
      }),
    ]
    .spacing(8),
  )
  .padding([12, 14])
  .width(Length::Fill)
  .style(move |_: &iced::Theme| container::Style {
    background: Some(Background::Color(bg)),
    border: Border {
      color: accent,
      width: 1.0,
      radius: 8.0.into(),
    },
    text_color: Some(theme::TEXT_STRONG),
    ..Default::default()
  })
  .into()
}

pub fn view<'a>(form: &'a SignForm) -> Element<'a, Message> {
  let bold = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
  };

  let verify_file_label: Element<'_, Message> = if let Some(path) = &form.verify_file {
    let name = path
      .file_name()
      .and_then(|n| n.to_str())
      .unwrap_or("fichier")
      .to_string();
    container(text(name).size(13))
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
    common::pick_btn("\u{f15b}", "Fichier à vérifier...", Message::VerifyPickFile),
    verify_file_label,
  ]
  .spacing(12)
  .align_y(Alignment::Center)
  .into();

  let verify_sig_label: Element<'_, Message> = if let Some(path) = &form.verify_sig_file {
    let name = path
      .file_name()
      .and_then(|n| n.to_str())
      .unwrap_or("fichier.sig")
      .to_string();
    container(text(name).size(13))
      .style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::TEXT_STRONG),
        ..Default::default()
      })
      .into()
  } else {
    let hint = if let Some(file) = &form.verify_file {
      let auto_name = format!(
        "{}.sig",
        file
          .file_name()
          .and_then(|n| n.to_str())
          .unwrap_or("fichier")
      );
      format!("Optionnel — cherchera automatiquement {auto_name}")
    } else {
      "Optionnel — cherche automatiquement <fichier>.sig".to_string()
    };
    container(text(hint).size(12))
      .style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::TEXT_MUTED),
        ..Default::default()
      })
      .into()
  };

  let sig_row: Element<'_, Message> = row![
    common::pick_btn("\u{f0c1}", "Fichier .sig...", Message::VerifyPickSig),
    verify_sig_label,
  ]
  .spacing(12)
  .align_y(Alignment::Center)
  .into();

  let result_el: Element<'_, Message> = match &form.verify_result {
    None if form.verifying => container(text("Vérification en cours...").size(13))
      .style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::TEXT_MUTED),
        ..Default::default()
      })
      .into(),
    None => iced::widget::Space::new().into(),
    Some(Err(e)) => container(
      row![
        text("\u{f057}")
          .font(theme::ICONS)
          .size(14)
          .style(|_: &iced::Theme| iced::widget::text::Style {
            color: Some(theme::ERROR),
          }),
        text(format!("Erreur : {e}")).size(13),
      ]
      .spacing(8)
      .align_y(Alignment::Center),
    )
    .padding([10, 14])
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::ERROR_BG)),
      border: Border {
        color: theme::ERROR,
        width: 1.0,
        radius: 8.0.into(),
      },
      text_color: Some(theme::ERROR),
      ..Default::default()
    })
    .into(),
    Some(Ok(vr)) => match &vr.outcome {
      VerifyOutcome::Valid => {
        let mut details: Vec<Element<'_, Message>> = vec![row![
          text("\u{f058}")
            .font(theme::ICONS)
            .size(20)
            .style(|_: &iced::Theme| iced::widget::text::Style {
              color: Some(theme::SUCCESS),
            }),
          text("Signature valide").size(18).font(bold),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()];
        if let Some(name) = &vr.signer_name {
          details.push(
            row![
              text("Signataire :").size(12).style(|_: &iced::Theme| {
                iced::widget::text::Style {
                  color: Some(theme::TEXT_MUTED),
                }
              }),
              text(name.as_str()).size(13),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .into(),
          );
        }
        if let Some(fp) = &vr.signer_fp {
          let short_fp = &fp[fp.len().saturating_sub(16)..];
          details.push(
            row![
              text("Fingerprint :").size(12).style(|_: &iced::Theme| {
                iced::widget::text::Style {
                  color: Some(theme::TEXT_MUTED),
                }
              }),
              text(short_fp.to_string()).size(12).font(Font::MONOSPACE),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .into(),
          );
        }
        if let Some(date) = &vr.signed_at {
          details.push(
            row![
              text("Signé le :").size(12).style(|_: &iced::Theme| {
                iced::widget::text::Style {
                  color: Some(theme::TEXT_MUTED),
                }
              }),
              text(date.as_str()).size(13),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .into(),
          );
        }
        container(column(details).spacing(6))
          .padding([12, 14])
          .width(Length::Fill)
          .style(|_: &iced::Theme| container::Style {
            background: Some(Background::Color(theme::SUCCESS_BG)),
            border: Border {
              color: theme::SUCCESS,
              width: 1.0,
              radius: 8.0.into(),
            },
            text_color: Some(theme::TEXT_STRONG),
            ..Default::default()
          })
          .into()
      }
      VerifyOutcome::BadSig => result_card(
        bold,
        "\u{f057}",
        theme::ERROR,
        theme::ERROR_BG,
        "Signature incorrecte",
        "La signature ne correspond pas à ce fichier. \
         Vérifiez que vous avez sélectionné le bon fichier et la bonne signature.",
        &vr.detail,
      ),
      VerifyOutcome::UnknownKey => result_card(
        bold,
        "\u{f071}",
        theme::PEACH,
        theme::WARNING_BG,
        "Clef de signature inconnue",
        "La clef publique du signataire n'est pas dans votre trousseau. \
         Importez-la pour vérifier l'identité du signataire.",
        &vr.detail,
      ),
      VerifyOutcome::ExpiredKey => result_card(
        bold,
        "\u{f071}",
        theme::PEACH,
        theme::WARNING_BG,
        "Clef expirée",
        "La signature est mathématiquement valide, mais la clef du signataire \
         était expirée au moment de la vérification.",
        &vr.detail,
      ),
      VerifyOutcome::RevokedKey => result_card(
        bold,
        "\u{f057}",
        theme::ERROR,
        theme::ERROR_BG,
        "Clef révoquée",
        "La clef ayant signé ce fichier a été révoquée. \
         La signature n'est plus considérée comme fiable.",
        &vr.detail,
      ),
      VerifyOutcome::Error(msg) => result_card(
        bold,
        "\u{f057}",
        theme::ERROR,
        theme::ERROR_BG,
        "Erreur",
        msg.as_str(),
        &vr.detail,
      ),
    },
  };

  let can_verify = form.verify_file.is_some() && !form.verifying;

  let rule_sep = || {
    rule::horizontal(1).style(|_: &iced::Theme| rule::Style {
      color: theme::BORDER,
      radius: 0.0.into(),
      fill_mode: rule::FillMode::Full,
      snap: true,
    })
  };

  let card = container(
    column![
      column![
        row![
          text("\u{f00c}").font(theme::ICONS).size(20),
          text("Vérifier une signature").size(22).font(bold),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        container(
          text(
            "Vérifier une signature confirme que le fichier n'a pas été modifié \
             et identifie son auteur."
          )
          .size(13)
        )
        .style(|_: &iced::Theme| container::Style {
          text_color: Some(theme::TEXT_SECONDARY),
          ..Default::default()
        }),
      ]
      .spacing(6),
      rule_sep(),
      file_row,
      rule_sep(),
      sig_row,
      rule_sep(),
      {
        let verify_action: Element<'_, Message> = row![
          iced::widget::Space::new().width(Length::Fill),
          common::action_btn("Vérifier", can_verify, Message::VerifyExecute)
        ]
        .into();
        verify_action
      },
      rule_sep(),
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
