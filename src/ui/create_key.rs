use iced::{
  font,
  widget::{
    button, checkbox, column, container, horizontal_rule, pick_list, row, text, text_input,
  },
  Background, Border, Color, Element, Font, Length,
};

use crate::app::{CreateKeyForm, Message, View};
use crate::gpg::KeyExpiry;
use crate::ui::theme;

pub fn view(form: &CreateKeyForm) -> Element<'_, Message> {
  let bold = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
  };

  let expiry_list = pick_list(
    vec![
      KeyExpiry::OneYear,
      KeyExpiry::TwoYears,
      KeyExpiry::FiveYears,
    ],
    Some(form.subkey_expiry.clone()),
    Message::CreateKeySubkeyExpiryChanged,
  )
  .width(Length::Fill);

  let label = if form.submitting {
    "Génération..."
  } else {
    "Créer la clef"
  };
  let can_submit = !form.name.is_empty() && !form.email.is_empty() && !form.submitting;

  let submit_btn = {
    let btn = button(text(label).size(13)).style(move |_: &iced::Theme, status: button::Status| {
      button::Style {
        background: Some(Background::Color(if can_submit {
          match status {
            button::Status::Hovered | button::Status::Pressed => theme::ACCENT_HOVER,
            _ => theme::ACCENT,
          }
        } else {
          theme::DISABLED_BG
        })),
        text_color: if can_submit {
          theme::TEXT_ON_ACCENT
        } else {
          theme::TEXT_MUTED
        },
        border: Border {
          color: Color::TRANSPARENT,
          width: 0.0,
          radius: 6.0.into(),
        },
        shadow: Default::default(),
      }
    });
    if can_submit {
      btn.on_press(Message::CreateKeySubmit)
    } else {
      btn
    }
  };

  let cancel_btn = button(text("Annuler").size(13))
    .on_press(Message::NavChanged(View::MyKeys))
    .style(|_: &iced::Theme, status: button::Status| button::Style {
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
      shadow: Default::default(),
    });

  let hint = |s: &'static str| {
    container(text(s).size(11)).style(|_: &iced::Theme| container::Style {
      text_color: Some(theme::TEXT_MUTED),
      ..Default::default()
    })
  };

  let card = container(
    column![
      column![
        text("Nouvelle clef PGP").size(22).font(bold),
        container(text("Génère une clef maître et ses sous-clefs dédiées.").size(13),).style(
          |_: &iced::Theme| container::Style {
            text_color: Some(theme::TEXT_SECONDARY),
            ..Default::default()
          }
        ),
      ]
      .spacing(6),
      horizontal_rule(1),
      column![
        text("Identité").size(12).font(bold),
        column![
          text("Nom").size(12),
          text_input("Alice Martin", &form.name)
            .on_input(Message::CreateKeyNameChanged)
            .size(14)
            .width(Length::Fill),
        ]
        .spacing(4),
        column![
          text("Email").size(12),
          text_input("alice@example.com", &form.email)
            .on_input(Message::CreateKeyEmailChanged)
            .size(14)
            .width(Length::Fill),
        ]
        .spacing(4),
      ]
      .spacing(10),
      horizontal_rule(1),
      column![
        text("Sous-clefs").size(12).font(bold),
        column![
          text("Expiration").size(12),
          expiry_list,
          hint(
            "Les sous-clefs expirent automatiquement. \
             Une courte durée limite les dégâts en cas de compromission \
             — vous pourrez les renouveler avant échéance.",
          ),
        ]
        .spacing(6),
        column![
          checkbox("Inclure une clef d'authentification SSH", form.include_auth,)
            .on_toggle(Message::CreateKeyIncludeAuthToggled)
            .text_size(13)
            .size(16),
          hint(
            "Permet de vous authentifier sur des serveurs distants sans mot de passe, \
             en utilisant votre clef PGP comme clef SSH.",
          ),
        ]
        .spacing(6),
      ]
      .spacing(14),
      horizontal_rule(1),
      container(
        column![
          text("À propos de la clef maître").size(12).font(bold),
          hint(
            "La clef maître définit votre identité PGP à long terme — elle ne sert qu'à \
             certifier vos sous-clefs. Elle n'expire jamais. \
             Conservez-la hors ligne avec son certificat de révocation.",
          ),
        ]
        .spacing(6),
      )
      .padding([4, 0]),
      horizontal_rule(1),
      row![cancel_btn, submit_btn].spacing(8),
    ]
    .spacing(20),
  )
  .padding(32)
  .width(520)
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

  container(card)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::SIDEBAR_BG)),
      ..Default::default()
    })
    .into()
}
