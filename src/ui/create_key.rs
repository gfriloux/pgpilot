use iced::{
  widget::{button, column, pick_list, row, text, text_input},
  Element, Length,
};

use crate::app::{CreateKeyForm, Message, View};
use crate::gpg::{KeyAlgo, KeyExpiry};

pub fn view(form: &CreateKeyForm) -> Element<'_, Message> {
  let algo_list = pick_list(
    vec![KeyAlgo::Ed25519, KeyAlgo::Rsa4096],
    Some(form.algo.clone()),
    Message::CreateKeyAlgoChanged,
  );

  let expiry_list = pick_list(
    vec![
      KeyExpiry::Never,
      KeyExpiry::OneYear,
      KeyExpiry::TwoYears,
      KeyExpiry::FiveYears,
    ],
    Some(form.expiry.clone()),
    Message::CreateKeyExpiryChanged,
  );

  let label = if form.submitting {
    "Génération..."
  } else {
    "Créer"
  };
  let can_submit = !form.name.is_empty() && !form.email.is_empty() && !form.submitting;
  let submit_btn = button(text(label));
  let submit_btn = if can_submit {
    submit_btn.on_press(Message::CreateKeySubmit)
  } else {
    submit_btn
  };

  column![
    text("Créer une nouvelle clef").size(18),
    column![
      text("Nom").size(13),
      text_input("Alice Martin", &form.name)
        .on_input(Message::CreateKeyNameChanged)
        .width(400),
    ]
    .spacing(4),
    column![
      text("Email").size(13),
      text_input("alice@example.com", &form.email)
        .on_input(Message::CreateKeyEmailChanged)
        .width(400),
    ]
    .spacing(4),
    row![
      column![text("Algorithme").size(13), algo_list].spacing(4),
      column![text("Expiration").size(13), expiry_list].spacing(4),
    ]
    .spacing(16),
    row![
      submit_btn,
      button(text("Annuler")).on_press(Message::NavChanged(View::MyKeys)),
    ]
    .spacing(8),
  ]
  .spacing(16)
  .padding(24)
  .width(Length::Fill)
  .into()
}
