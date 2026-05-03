use iced::{
  font,
  widget::{
    button, column, container, horizontal_rule, horizontal_space, row, rule, scrollable, text,
  },
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
    });
  if enabled {
    btn.on_press(on_press).into()
  } else {
    btn.into()
  }
}

fn separator<'a>() -> Element<'a, Message> {
  horizontal_rule(1)
    .style(|_: &iced::Theme| rule::Style {
      color: theme::BORDER,
      width: 1,
      radius: 0.0.into(),
      fill_mode: rule::FillMode::Full,
    })
    .into()
}

fn sign_section<'a>(form: &'a SignForm, keys: &'a [KeyInfo]) -> Element<'a, Message> {
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
    iced::widget::Space::with_height(0).into()
  };

  let can_sign = form.file.is_some() && form.signer_fp.is_some() && !form.signing;

  column![
    column![
      text("Signer un fichier").size(18).font(bold),
      container(
        text(
          "Signer un fichier crée une preuve cryptographique que vous en êtes l'auteur. \
           Le fichier original n'est pas modifié — la signature est enregistrée dans un \
           fichier .sig séparé."
        )
        .size(12)
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
        horizontal_space(),
        action_btn("Signer", can_sign, Message::SignExecute)
      ]
      .into();
      sign_action
    },
    result_el,
  ]
  .spacing(12)
  .into()
}

fn verify_section<'a>(form: &'a SignForm) -> Element<'a, Message> {
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
    pick_btn("\u{f15b}", "Fichier à vérifier...", Message::VerifyPickFile),
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
    pick_btn("\u{f0c1}", "Fichier .sig...", Message::VerifyPickSig),
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
    None => iced::widget::Space::with_height(0).into(),
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
    Some(Ok(vr)) if vr.valid => {
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
    Some(Ok(vr)) => {
      let unknown_key = vr.detail.contains("No public key")
        || vr.detail.contains("public key not found")
        || vr.detail.contains("BAD_DATA")
        || vr.signer_fp.is_none() && !vr.valid;

      if unknown_key || (vr.signer_fp.is_some() && !vr.valid) {
        let (bg, border_c, icon, msg, sub) = if vr.signer_fp.is_none()
          || vr.detail.contains("No public key")
          || vr.detail.contains("public key not found")
        {
          (
            theme::ERROR_BG,
            theme::PEACH,
            "\u{f071}",
            "Clef de signature inconnue",
            "La signature est présente mais la clef publique du signataire \
             n'est pas dans votre trousseau. Importez sa clef pour vérifier l'identité.",
          )
        } else {
          (
            theme::ERROR_BG,
            theme::ERROR,
            "\u{f057}",
            "Signature INVALIDE",
            "Ce fichier a peut-être été modifié ou la signature ne correspond pas.",
          )
        };
        container(
          column![
            row![
              text(icon)
                .font(theme::ICONS)
                .size(16)
                .style(move |_: &iced::Theme| iced::widget::text::Style {
                  color: Some(border_c),
                }),
              text(msg).size(16).font(bold),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            container(text(sub).size(12)).style(|_: &iced::Theme| container::Style {
              text_color: Some(theme::TEXT_SECONDARY),
              ..Default::default()
            }),
            container(text(vr.detail.as_str()).size(11).font(Font::MONOSPACE)).style(
              |_: &iced::Theme| container::Style {
                text_color: Some(theme::TEXT_MUTED),
                ..Default::default()
              }
            ),
          ]
          .spacing(8),
        )
        .padding([12, 14])
        .width(Length::Fill)
        .style(move |_: &iced::Theme| container::Style {
          background: Some(Background::Color(bg)),
          border: Border {
            color: border_c,
            width: 1.0,
            radius: 8.0.into(),
          },
          text_color: Some(theme::TEXT_STRONG),
          ..Default::default()
        })
        .into()
      } else {
        container(
          column![
            row![
              text("\u{f057}")
                .font(theme::ICONS)
                .size(16)
                .style(|_: &iced::Theme| iced::widget::text::Style {
                  color: Some(theme::ERROR),
                }),
              text("Signature INVALIDE").size(16).font(bold),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            container(
              text("Ce fichier a peut-être été modifié ou la signature ne correspond pas.")
                .size(12)
            )
            .style(|_: &iced::Theme| container::Style {
              text_color: Some(theme::TEXT_SECONDARY),
              ..Default::default()
            }),
            container(text(vr.detail.as_str()).size(11).font(Font::MONOSPACE)).style(
              |_: &iced::Theme| container::Style {
                text_color: Some(theme::TEXT_MUTED),
                ..Default::default()
              }
            ),
          ]
          .spacing(8),
        )
        .padding([12, 14])
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
          background: Some(Background::Color(theme::ERROR_BG)),
          border: Border {
            color: theme::ERROR,
            width: 1.0,
            radius: 8.0.into(),
          },
          text_color: Some(theme::TEXT_STRONG),
          ..Default::default()
        })
        .into()
      }
    }
  };

  let can_verify = form.verify_file.is_some() && !form.verifying;

  column![
    column![
      text("Vérifier une signature").size(18).font(bold),
      container(
        text(
          "Vérifier une signature confirme que le fichier n'a pas été modifié \
           et identifie son auteur."
        )
        .size(12)
      )
      .style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::TEXT_SECONDARY),
        ..Default::default()
      }),
    ]
    .spacing(6),
    file_row,
    sig_row,
    {
      let verify_action: Element<'_, Message> = row![
        horizontal_space(),
        action_btn("Vérifier", can_verify, Message::VerifyExecute)
      ]
      .into();
      verify_action
    },
    result_el,
  ]
  .spacing(12)
  .into()
}

pub fn view<'a>(form: &'a SignForm, keys: &'a [KeyInfo]) -> Element<'a, Message> {
  let bold = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
  };

  let card = container(
    column![
      column![
        row![
          text("\u{f14b}").font(theme::ICONS).size(20),
          text("Signer / Vérifier").size(22).font(bold),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        container(text("Créez ou vérifiez des signatures cryptographiques de fichiers.").size(13))
          .style(|_: &iced::Theme| container::Style {
            text_color: Some(theme::TEXT_SECONDARY),
            ..Default::default()
          }),
      ]
      .spacing(6),
      separator(),
      sign_section(form, keys),
      separator(),
      verify_section(form),
    ]
    .spacing(20),
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
