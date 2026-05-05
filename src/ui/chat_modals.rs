use iced::{
  widget::{button, column, container, row, text, text_input, Space},
  Alignment, Background, Border, Element, Length,
};

use crate::app::{App, Message};
use crate::ui::{button_styles, common, theme};

/// View for `View::ChatNewRoom` — create a new chat room.
pub fn create_room_view(app: &App) -> Element<'_, Message> {
  let s = app.strings;
  let form = &app.chat_new_form;

  let can_submit = !form.name.trim().is_empty()
    && !form.relay.trim().is_empty()
    && !form.participants_input.trim().is_empty();

  let label_style = |_: &iced::Theme| iced::widget::text::Style {
    color: Some(theme::text_muted()),
  };

  let content = column![
    // Page title
    text(theme::flavor(
      s.chat_create_room_title(),
      s.chat_create_room_title_ussr()
    ))
    .size(22)
    .font(theme::flavor_title_font()),
    // Room name field
    column![
      text(s.chat_room_name_label()).size(12).style(label_style),
      text_input(s.chat_room_name_placeholder(), &form.name)
        .on_input(Message::ChatRoomNameChanged)
        .padding([8, 12])
        .size(13),
    ]
    .spacing(4),
    // Relay field
    column![
      text(s.chat_relay_label()).size(12).style(label_style),
      text_input(s.chat_relay_placeholder(), &form.relay)
        .on_input(Message::ChatRoomRelayChanged)
        .padding([8, 12])
        .size(13),
      text(s.chat_relay_hint()).size(11).style(label_style),
    ]
    .spacing(4),
    // Participants field (text_input, one fingerprint per line hint)
    column![
      text(s.chat_participants_label())
        .size(12)
        .style(label_style),
      text_input(s.chat_participants_hint(), &form.participants_input)
        .on_input(Message::ChatRoomParticipantsChanged)
        .padding([8, 12])
        .size(13),
      text(s.chat_participants_hint()).size(11).style(label_style),
    ]
    .spacing(4),
    // Buttons
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

  let can_submit = !form.join_code.trim().is_empty();

  let label_style = |_: &iced::Theme| iced::widget::text::Style {
    color: Some(theme::text_muted()),
  };

  let content = column![
    // Page title
    text(s.chat_join_room_title())
      .size(22)
      .font(theme::flavor_title_font()),
    // Join code field
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
