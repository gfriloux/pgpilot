use iced::{
  font,
  widget::{button, column, container, horizontal_rule, pick_list, row, text, text_input},
  Background, Border, Color, Element, Font, Length,
};

use crate::app::{CreateKeyForm, Message, View};
use crate::gpg::{KeyAlgo, KeyExpiry};
use crate::ui::theme;

pub fn view(form: &CreateKeyForm) -> Element<'_, Message> {
  let bold = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
  };

  let algo_list = pick_list(
    vec![KeyAlgo::Ed25519, KeyAlgo::Rsa4096],
    Some(form.algo.clone()),
    Message::CreateKeyAlgoChanged,
  )
  .width(Length::Fill);

  let expiry_list = pick_list(
    vec![
      KeyExpiry::Never,
      KeyExpiry::OneYear,
      KeyExpiry::TwoYears,
      KeyExpiry::FiveYears,
    ],
    Some(form.expiry.clone()),
    Message::CreateKeyExpiryChanged,
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
        text_color: Color::WHITE,
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

  let card = container(
    column![
      column![
        text("Nouvelle clef PGP").size(22).font(bold),
        container(text("Renseignez les informations pour générer votre clef.").size(13)).style(
          |_: &iced::Theme| container::Style {
            text_color: Some(theme::TEXT_SECONDARY),
            ..Default::default()
          }
        ),
      ]
      .spacing(6),
      horizontal_rule(1),
      column![
        text("Nom").size(12).font(bold),
        text_input("Alice Martin", &form.name)
          .on_input(Message::CreateKeyNameChanged)
          .size(14)
          .width(Length::Fill),
      ]
      .spacing(6),
      column![
        text("Email").size(12).font(bold),
        text_input("alice@example.com", &form.email)
          .on_input(Message::CreateKeyEmailChanged)
          .size(14)
          .width(Length::Fill),
      ]
      .spacing(6),
      row![
        column![text("Algorithme").size(12).font(bold), algo_list]
          .spacing(6)
          .width(Length::Fill),
        column![text("Expiration").size(12).font(bold), expiry_list]
          .spacing(6)
          .width(Length::Fill),
      ]
      .spacing(16),
      horizontal_rule(1),
      row![cancel_btn, submit_btn].spacing(8),
    ]
    .spacing(20),
  )
  .padding(32)
  .width(480)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(Color::WHITE)),
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
      background: Some(Background::Color(theme::HEADER_BG)),
      ..Default::default()
    })
    .into()
}
