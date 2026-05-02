use iced::{
  widget::{column, row, text},
  Element, Length,
};

use crate::app::Message;
use crate::gpg::KeyInfo;

pub fn view(key: &KeyInfo) -> Element<'_, Message> {
  let expires = key.expires.as_deref().unwrap_or("Aucune expiration");
  let key_type = if key.has_secret {
    "Publique + Privée"
  } else {
    "Publique"
  };

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
      text(format!("  ·  {} ", key_type)),
    ]
    .spacing(0),
  ]
  .spacing(4)
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
