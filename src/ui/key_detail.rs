use iced::{
  widget::{button, column, row, text, Row},
  Element, Length,
};

use crate::app::Message;
use crate::gpg::KeyInfo;

pub fn view(key: &KeyInfo, idx: usize) -> Element<'_, Message> {
  let expires = key.expires.as_deref().unwrap_or("Aucune expiration");
  let key_type = if key.has_secret {
    "Publique + Privée"
  } else {
    "Publique"
  };

  let mut action_buttons: Vec<Element<Message>> = vec![button(text("Export pub").size(13))
    .on_press(Message::ExportPublicKey(idx))
    .into()];

  if key.has_secret {
    action_buttons.push(
      button(text("Export privée").size(13))
        .on_press(Message::ExportSecretKey(idx))
        .into(),
    );
  }

  column![
    row![
      text(&key.name).size(15),
      text(format!("  <{}>", key.email)).size(13),
    ],
    text(format_fingerprint(&key.fingerprint)).size(11),
    row![
      text(format!("Algorithme : {}", key.algo)),
      text(format!("  ·  Créée : {}", key.created)),
      text(format!("  ·  Expire : {}", expires)),
      text(format!("  ·  {}", key_type)),
    ]
    .spacing(0),
    Row::with_children(action_buttons).spacing(8),
  ]
  .spacing(6)
  .padding(12)
  .width(Length::Fill)
  .into()
}

fn format_fingerprint(fp: &str) -> String {
  fp.chars()
    .collect::<Vec<_>>()
    .chunks(4)
    .map(|c| c.iter().collect::<String>())
    .collect::<Vec<_>>()
    .join(" ")
}
