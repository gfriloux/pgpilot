use iced::{
  font,
  widget::{
    button, column, container, horizontal_rule, pick_list, row, scrollable, text, text_editor,
    text_input,
  },
  Background, Border, Color, Element, Font, Length,
};

use crate::app::{ImportForm, Message, View};
use crate::gpg::Keyserver;
use crate::ui::theme;

pub fn view(form: &ImportForm) -> Element<'_, Message> {
  let bold = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
  };

  let section_label = |s: &'static str| text(s).size(12).font(bold);

  let hint = |s: &'static str| {
    container(text(s).size(11)).style(|_: &iced::Theme| container::Style {
      text_color: Some(theme::TEXT_MUTED),
      ..Default::default()
    })
  };

  let action_btn = |label: &'static str, msg: Message, enabled: bool| {
    let btn = button(text(label).size(13)).style(move |_: &iced::Theme, status: button::Status| {
      button::Style {
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
        shadow: Default::default(),
      }
    });
    if enabled {
      btn.on_press(msg)
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

  let file_btn = button(
    row![
      text("\u{f0c7}").font(theme::ICONS).size(12),
      text("Depuis un fichier").size(13),
    ]
    .spacing(6),
  )
  .on_press(Message::ImportKey)
  .width(Length::Fill)
  .style(|_: &iced::Theme, status: button::Status| button::Style {
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
    shadow: Default::default(),
  });

  let url_ready = !form.url.is_empty() && !form.submitting;
  let ks_ready = !form.keyserver_query.is_empty() && !form.submitting;
  let paste_ready = !form.pasted_key.text().trim().is_empty() && !form.submitting;

  let ks_list = pick_list(
    vec![Keyserver::Openpgp, Keyserver::Ubuntu],
    Some(form.keyserver.clone()),
    Message::ImportKeyserverChanged,
  )
  .width(Length::Fill);

  let card = container(
    column![
      column![
        text("Importer une clef").size(22).font(bold),
        container(text("Choisissez la source de la clef à importer.").size(13),).style(
          |_: &iced::Theme| container::Style {
            text_color: Some(theme::TEXT_SECONDARY),
            ..Default::default()
          }
        ),
      ]
      .spacing(6),
      horizontal_rule(1),
      file_btn,
      horizontal_rule(1),
      column![
        section_label("Depuis une URL"),
        hint("Collez une URL pointant vers une clef armored (paste.rs, page web, etc.)."),
        text_input("https://paste.rs/abc123", &form.url)
          .on_input(Message::ImportUrlChanged)
          .size(13)
          .width(Length::Fill),
        action_btn("Importer depuis l'URL", Message::ImportFromUrl, url_ready),
      ]
      .spacing(8),
      horizontal_rule(1),
      column![
        section_label("Depuis un keyserver"),
        hint("Fingerprint complet, ID court (8 chars) ou adresse email."),
        text_input("ABC123... ou alice@example.com", &form.keyserver_query)
          .on_input(Message::ImportKeyserverQueryChanged)
          .size(13)
          .width(Length::Fill),
        ks_list,
        action_btn(
          "Importer depuis le keyserver",
          Message::ImportFromKeyserver,
          ks_ready,
        ),
      ]
      .spacing(8),
      horizontal_rule(1),
      column![
        section_label("Coller la clef"),
        hint("Collez directement le contenu d'une clef PGP armored (-----BEGIN PGP...)."),
        container(
          text_editor(&form.pasted_key)
            .on_action(Message::ImportPastedKeyChanged)
            .height(120),
        )
        .style(|_: &iced::Theme| container::Style {
          border: Border {
            color: theme::BORDER,
            width: 1.0,
            radius: 4.0.into(),
          },
          ..Default::default()
        }),
        action_btn(
          "Importer la clef collée",
          Message::ImportFromPaste,
          paste_ready
        ),
      ]
      .spacing(8),
      horizontal_rule(1),
      cancel_btn,
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

  scrollable(
    container(card)
      .center_x(Length::Fill)
      .padding([24, 0])
      .width(Length::Fill)
      .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(theme::SIDEBAR_BG)),
        ..Default::default()
      }),
  )
  .height(Length::Fill)
  .width(Length::Fill)
  .into()
}
