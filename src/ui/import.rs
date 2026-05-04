use iced::{
  font,
  widget::{
    button, column, container, pick_list, row, rule, scrollable, text, text_editor, text_input,
  },
  Background, Border, Color, Element, Font, Length,
};

use crate::app::{ImportForm, Message};
use crate::gpg::Keyserver;
use crate::i18n::Strings;
use crate::ui::{common, theme};

pub fn view<'a>(form: &'a ImportForm, s: &'static dyn Strings) -> Element<'a, Message> {
  let bold = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
  };

  let separator = || {
    rule::horizontal(1).style(|_: &iced::Theme| rule::Style {
      color: theme::border(),
      radius: 0.0.into(),
      fill_mode: rule::FillMode::Full,
      snap: false,
    })
  };

  let section_label = |lbl: &'static str| text(lbl).size(12).font(bold);

  let hint = |lbl: &'static str| {
    container(text(lbl).size(11)).style(|_: &iced::Theme| container::Style {
      text_color: Some(theme::text_muted()),
      ..Default::default()
    })
  };

  let action_btn = |label: &'static str, msg: Message, enabled: bool| {
    let btn = button(text(label).size(13)).style(move |_: &iced::Theme, status: button::Status| {
      button::Style {
        background: Some(Background::Color(if enabled {
          match status {
            button::Status::Hovered | button::Status::Pressed => theme::accent_hover(),
            _ => theme::accent(),
          }
        } else {
          theme::disabled_bg()
        })),
        text_color: if enabled {
          theme::text_on_accent()
        } else {
          theme::text_muted()
        },
        border: Border {
          color: Color::TRANSPARENT,
          width: 0.0,
          radius: 6.0.into(),
        },
        shadow: Default::default(),
        snap: false,
      }
    });
    if enabled {
      btn.on_press(msg)
    } else {
      btn
    }
  };

  let cancel_btn = button(text(s.btn_cancel()).size(13))
    .on_press(Message::NavBack)
    .style(|_: &iced::Theme, status: button::Status| button::Style {
      background: Some(Background::Color(match status {
        button::Status::Hovered | button::Status::Pressed => theme::header_bg(),
        _ => Color::TRANSPARENT,
      })),
      text_color: theme::text_secondary(),
      border: Border {
        color: theme::border(),
        width: 1.0,
        radius: 6.0.into(),
      },
      shadow: Default::default(),
      snap: false,
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
      button::Status::Hovered | button::Status::Pressed => theme::accent_subtle(),
      _ => Color::TRANSPARENT,
    })),
    text_color: theme::text_strong(),
    border: Border {
      color: theme::border(),
      width: 1.0,
      radius: 6.0.into(),
    },
    shadow: Default::default(),
    snap: false,
  });

  let url_ready = !form.url.is_empty() && !form.submitting;
  let ks_ready = !form.keyserver_query.is_empty() && !form.submitting;
  let paste_ready = !form.pasted_key.text().trim().is_empty() && !form.submitting;

  let ks_list = pick_list(
    vec![Keyserver::Openpgp, Keyserver::Ubuntu],
    Some(form.keyserver.clone()),
    Message::ImportKeyserverChanged,
  )
  .width(Length::Fill)
  .style(|_: &iced::Theme, status| {
    let border = match status {
      pick_list::Status::Opened { .. } => theme::accent(),
      pick_list::Status::Hovered => theme::accent_border(),
      _ => theme::border(),
    };
    pick_list::Style {
      text_color: theme::text_strong(),
      placeholder_color: theme::text_muted(),
      handle_color: theme::text_muted(),
      background: Background::Color(theme::header_bg()),
      border: Border {
        color: border,
        width: 1.0,
        radius: 6.0.into(),
      },
    }
  })
  .menu_style(|_: &iced::Theme| iced::overlay::menu::Style {
    text_color: theme::text_strong(),
    background: Background::Color(theme::card_bg()),
    border: Border {
      color: theme::border(),
      width: 1.0,
      radius: 6.0.into(),
    },
    selected_text_color: theme::text_on_accent(),
    selected_background: Background::Color(theme::accent()),
    shadow: iced::Shadow::default(),
  });

  let card = container(
    column![
      column![
        text(s.import_title()).size(22).font(bold),
        container(text("Choisissez la source de la clef à importer.").size(13),).style(
          |_: &iced::Theme| container::Style {
            text_color: Some(theme::text_secondary()),
            ..Default::default()
          }
        ),
      ]
      .spacing(6),
      separator(),
      file_btn,
      separator(),
      column![
        section_label(s.import_tab_url()),
        hint("Collez une URL pointant vers une clef armored (paste.rs, page web, etc.)."),
        text_input("https://paste.rs/abc123", &form.url)
          .on_input(Message::ImportUrlChanged)
          .size(13)
          .width(Length::Fill)
          .style(|_: &iced::Theme, status| {
            let border = match status {
              text_input::Status::Focused { .. } => theme::accent(),
              text_input::Status::Hovered => theme::accent_border(),
              _ => theme::border(),
            };
            text_input::Style {
              background: Background::Color(theme::header_bg()),
              border: Border {
                color: border,
                width: 1.0,
                radius: 6.0.into(),
              },
              icon: theme::text_muted(),
              placeholder: theme::text_muted(),
              value: theme::text_strong(),
              selection: theme::accent_subtle(),
            }
          }),
        action_btn("Importer depuis l'URL", Message::ImportFromUrl, url_ready),
      ]
      .spacing(8),
      separator(),
      column![
        section_label(s.import_tab_keyserver()),
        hint("Fingerprint complet (40 hex), ID long (16 hex) ou adresse email."),
        text_input(
          "Fingerprint (40 hex), ID long (16 hex) ou email",
          &form.keyserver_query,
        )
        .on_input(Message::ImportKeyserverQueryChanged)
        .size(13)
        .width(Length::Fill)
        .style(|_: &iced::Theme, status| {
          let border = match status {
            text_input::Status::Focused { .. } => theme::accent(),
            text_input::Status::Hovered => theme::accent_border(),
            _ => theme::border(),
          };
          text_input::Style {
            background: Background::Color(theme::header_bg()),
            border: Border {
              color: border,
              width: 1.0,
              radius: 6.0.into(),
            },
            icon: theme::text_muted(),
            placeholder: theme::text_muted(),
            value: theme::text_strong(),
            selection: theme::accent_subtle(),
          }
        }),
        ks_list,
        action_btn(
          "Importer depuis le keyserver",
          Message::ImportFromKeyserver,
          ks_ready,
        ),
      ]
      .spacing(8),
      separator(),
      column![
        section_label(s.import_tab_paste()),
        hint("Collez directement le contenu d'une clef PGP armored (-----BEGIN PGP...)."),
        text_editor(&form.pasted_key)
          .on_action(Message::ImportPastedKeyChanged)
          .height(120)
          .style(|_: &iced::Theme, status| {
            let border = match status {
              text_editor::Status::Focused { .. } => theme::accent(),
              text_editor::Status::Hovered => theme::accent_border(),
              _ => theme::border(),
            };
            text_editor::Style {
              background: Background::Color(theme::header_bg()),
              border: Border {
                color: border,
                width: 1.0,
                radius: 6.0.into(),
              },
              placeholder: theme::text_muted(),
              value: theme::text_strong(),
              selection: theme::accent_subtle(),
            }
          }),
        action_btn(
          "Importer la clef collée",
          Message::ImportFromPaste,
          paste_ready
        ),
      ]
      .spacing(8),
      separator(),
      cancel_btn,
    ]
    .spacing(20),
  )
  .padding(32)
  .width(520)
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
    .width(Length::Fill)
    .style(common::scroll_style),
  )
  .height(Length::Fill)
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::sidebar_bg())),
    ..Default::default()
  })
  .into()
}
