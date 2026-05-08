use iced::{
  font,
  widget::{button, column, container, pick_list, row, rule, text, text_editor, text_input},
  Background, Border, Color, Element, Font, Length,
};

use crate::app::{ImportForm, Message};
use crate::gpg::Keyserver;
use crate::i18n::Strings;
use crate::ui::{common, theme, ussr_assets};

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
      text(s.import_source_from_file()).size(13),
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

  let card_content = column![
    column![
      text(theme::flavor(s.import_title(), "Welcome a Foreign Comrade"))
        .size(22)
        .font(theme::flavor_title_font()),
      container(text(s.import_select_source()).size(13),).style(|_: &iced::Theme| {
        container::Style {
          text_color: Some(theme::text_secondary()),
          ..Default::default()
        }
      }),
    ]
    .spacing(6),
    separator(),
    file_btn,
    separator(),
    column![
      section_label(s.import_tab_url()),
      hint(s.import_url_hint()),
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
      action_btn(s.import_url_button(), Message::ImportFromUrl, url_ready),
    ]
    .spacing(8),
    separator(),
    column![
      section_label(s.import_tab_keyserver()),
      hint(s.import_keyserver_hint()),
      text_input(s.import_keyserver_hint(), &form.keyserver_query,)
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
        s.import_keyserver_button(),
        Message::ImportFromKeyserver,
        ks_ready,
      ),
    ]
    .spacing(8),
    separator(),
    column![
      section_label(s.import_tab_paste()),
      hint(s.import_paste_hint()),
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
        s.import_paste_button(),
        Message::ImportFromPaste,
        paste_ready
      ),
    ]
    .spacing(8),
    separator(),
    cancel_btn,
  ]
  .spacing(20);

  common::page_layout(common::card_medium_with_banner(
    card_content,
    ussr_assets::banner(24),
  ))
}
