use iced::{
  widget::{button, column, container, pick_list, row, scrollable, text, text_input, Space},
  Alignment, Background, Border, Element, Length,
};

use crate::app::{App, Message};
use crate::ui::{button_styles, common, theme};

/// Option de clef privée pour le pick_list d'identité.
#[derive(Debug, Clone, PartialEq, Eq)]
struct KeyOption {
  pub fp: String,
  pub label: String,
}

impl std::fmt::Display for KeyOption {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.label)
  }
}

/// Section commune : pick_list de clef privée pour choisir l'identité de la room.
fn identity_picker(app: &App) -> Element<'_, Message> {
  let form = &app.chat_new_form;
  let label_style = |_: &iced::Theme| iced::widget::text::Style {
    color: Some(theme::text_muted()),
  };

  let options: Vec<KeyOption> = app
    .keys
    .iter()
    .filter(|k| k.has_secret)
    .map(|k| KeyOption {
      fp: k.fingerprint.clone(),
      label: format!(
        "{} <{}> [{}…{}]",
        k.name,
        k.email,
        &k.fingerprint[..8],
        &k.fingerprint[k.fingerprint.len() - 4..],
      ),
    })
    .collect();

  let selected: Option<KeyOption> = form
    .my_fp
    .as_ref()
    .and_then(|fp| options.iter().find(|o| &o.fp == fp).cloned());

  let pl = pick_list(options, selected, |opt: KeyOption| {
    Message::ChatRoomMyFpChanged(opt.fp)
  })
  .width(Length::Fill)
  .placeholder("Select your identity…")
  .style(|_: &iced::Theme, status| {
    let border_color = match status {
      pick_list::Status::Opened { .. } => theme::accent(),
      pick_list::Status::Hovered => theme::accent_border(),
      _ => theme::border(),
    };
    pick_list::Style {
      text_color: theme::text_strong(),
      placeholder_color: theme::text_muted(),
      handle_color: theme::text_muted(),
      background: iced::Background::Color(theme::header_bg()),
      border: iced::Border {
        color: border_color,
        width: 1.0,
        radius: 6.0.into(),
      },
    }
  });

  column![
    text("Your identity in this room")
      .size(12)
      .style(label_style),
    pl,
  ]
  .spacing(4)
  .into()
}

/// View for `View::ChatNewRoom` — create a new chat room.
pub fn create_room_view(app: &App) -> Element<'_, Message> {
  let s = app.strings;
  let form = &app.chat_new_form;

  let can_submit =
    !form.name.trim().is_empty() && !form.relay.trim().is_empty() && form.my_fp.is_some();

  let label_style = |_: &iced::Theme| iced::widget::text::Style {
    color: Some(theme::text_muted()),
  };

  // Participant picker — liste des clefs publiques du keyring.
  let participants_list: Vec<Element<Message>> = app
    .keys
    .iter()
    .map(|key| {
      let fp = key.fingerprint.clone();
      let is_selected = form.selected_participants.contains(&fp);
      let label = format!(
        "{} <{}> — {}…{}",
        key.name,
        key.email,
        &key.fingerprint[..8],
        &key.fingerprint[key.fingerprint.len() - 4..],
      );
      let btn = button(
        row![
          text(if is_selected { "\u{f046}" } else { "\u{f096}" })
            .font(theme::ICONS)
            .size(14),
          text(label).size(12),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
      )
      .on_press(Message::ChatRoomParticipantToggled(fp))
      .padding([4, 8])
      .style(button_styles::primary_toggle(is_selected));
      btn.into()
    })
    .collect();

  let participants_section = column![
    text(s.chat_participants_label())
      .size(12)
      .style(label_style),
    if participants_list.is_empty() {
      {
        let e: Element<Message> =
          container(text("No keys in keyring.").size(12).style(label_style)).into();
        e
      }
    } else {
      scrollable(column(participants_list).spacing(4))
        .height(160)
        .style(common::scroll_style)
        .into()
    },
  ]
  .spacing(4);

  let content = column![
    text(theme::flavor(
      s.chat_create_room_title(),
      s.chat_create_room_title_ussr()
    ))
    .size(22)
    .font(theme::flavor_title_font()),
    column![
      text(s.chat_room_name_label()).size(12).style(label_style),
      text_input(s.chat_room_name_placeholder(), &form.name)
        .on_input(Message::ChatRoomNameChanged)
        .padding([8, 12])
        .size(13),
    ]
    .spacing(4),
    column![
      text(s.chat_relay_label()).size(12).style(label_style),
      text_input(s.chat_relay_placeholder(), &form.relay)
        .on_input(Message::ChatRoomRelayChanged)
        .padding([8, 12])
        .size(13),
      text(s.chat_relay_hint()).size(11).style(label_style),
    ]
    .spacing(4),
    identity_picker(app),
    participants_section,
    row![
      button(text(s.btn_cancel()).size(13))
        .on_press(Message::NavBack)
        .padding([8, 16])
        .style(button_styles::ghost_neutral()),
      Space::new().width(Length::Fill),
      button(text(s.chat_create_room_btn()).size(13))
        .on_press_maybe(can_submit.then_some(Message::ChatRoomCreate))
        .padding([8, 16])
        .style(button_styles::primary_toggle(can_submit)),
    ]
    .align_y(Alignment::Center),
  ]
  .spacing(20);

  common::page_layout(common::card_medium(container(content).style(
    |_: &iced::Theme| container::Style {
      text_color: Some(theme::text_strong()),
      ..Default::default()
    },
  )))
}

/// View for `View::ChatJoinRoom` — join an existing room via invite code.
pub fn join_room_view(app: &App) -> Element<'_, Message> {
  let s = app.strings;
  let form = &app.chat_new_form;

  let can_submit = !form.join_code.trim().is_empty() && form.my_fp.is_some();

  let label_style = |_: &iced::Theme| iced::widget::text::Style {
    color: Some(theme::text_muted()),
  };

  let content = column![
    text(s.chat_join_room_title())
      .size(22)
      .font(theme::flavor_title_font()),
    identity_picker(app),
    column![
      text(s.chat_join_code_label()).size(12).style(label_style),
      text_input(s.chat_join_code_placeholder(), &form.join_code)
        .on_input(Message::ChatJoinCodeChanged)
        .padding([8, 12])
        .size(13),
      text(s.chat_join_code_hint()).size(11).style(label_style),
    ]
    .spacing(4),
    // Buttons
    row![
      button(text(s.btn_cancel()).size(13))
        .on_press(Message::NavBack)
        .padding([8, 16])
        .style(button_styles::ghost_neutral()),
      Space::new().width(Length::Fill),
      button(text(s.chat_join_btn()).size(13))
        .on_press_maybe(can_submit.then_some(Message::ChatRoomJoin))
        .padding([8, 16])
        .style(button_styles::primary_toggle(can_submit)),
    ]
    .align_y(Alignment::Center),
  ]
  .spacing(20);

  common::page_layout(common::card_medium(container(content).style(
    |_: &iced::Theme| container::Style {
      text_color: Some(theme::text_strong()),
      background: Some(Background::Color(theme::card_bg())),
      border: Border {
        radius: 12.0.into(),
        ..Default::default()
      },
      ..Default::default()
    },
  )))
}
