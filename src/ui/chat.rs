use std::collections::HashMap;

use iced::{
  widget::{button, column, container, mouse_area, row, rule, scrollable, text, Column, Space},
  Alignment, Background, Border, Element, Length,
};

use crate::app::{App, Message, MqttState, PendingOp, View};
use crate::chat::rooms::{Room, RoomParticipant};
use crate::chat::{AckStatus, ChatMessage, PresenceStatus, PresenceTracker};
use crate::ui::{button_styles, common, theme, ussr_assets};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Main entry point for `View::ChatList` and `View::ChatRoom(_)`.
pub fn view(app: &App) -> Element<'_, Message> {
  let room_list = room_list_panel(app);
  let sep = rule::vertical(1).style(|_: &iced::Theme| rule::Style {
    color: theme::border(),
    radius: 0.0.into(),
    fill_mode: rule::FillMode::Full,
    snap: false,
  });
  let chat = right_panel(app);

  row![room_list, sep, chat]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn right_panel(app: &App) -> Element<'_, Message> {
  if let Some(room_id) = &app.active_room {
    let room = app.rooms.iter().find(|r| &r.id == room_id);
    if let Some(r) = room {
      // Check leave modal first
      if let Some(PendingOp::LeaveRoom(ref rid)) = app.pending {
        if rid == &r.id {
          return leave_room_modal(r, app.strings);
        }
      }
      // Collect messages as owned Vec of refs from the VecDeque
      return chat_panel_for_room(r, app);
    }
    empty_chat_state(app.strings)
  } else if let Some(PendingOp::IdentitySelection {
    room_id,
    selected_fp,
  }) = &app.pending
  {
    let room = app.rooms.iter().find(|r| &r.id == room_id);
    if let Some(r) = room {
      identity_selection_modal(r.id.as_str(), selected_fp, app)
    } else {
      empty_chat_state(app.strings)
    }
  } else {
    empty_chat_state(app.strings)
  }
}

fn chat_panel_for_room<'a>(room: &'a Room, app: &'a App) -> Element<'a, Message> {
  let s = app.strings;

  let maybe_banner: Option<Element<Message>> = if app.mqtt_state != MqttState::Connected {
    Some(mqtt_warning_banner(s))
  } else {
    None
  };

  let header = chat_header(room, app);

  let sep_h = rule::horizontal(1).style(|_: &iced::Theme| rule::Style {
    color: theme::border(),
    radius: 0.0.into(),
    fill_mode: rule::FillMode::Full,
    snap: true,
  });

  let msg_area = message_list_for_room(room, app);

  let sep_compose = rule::horizontal(1).style(|_: &iced::Theme| rule::Style {
    color: theme::border(),
    radius: 0.0.into(),
    fill_mode: rule::FillMode::Full,
    snap: true,
  });

  let compose = compose_bar(room.id.as_str(), app);

  let mut col = column![];
  if let Some(banner) = maybe_banner {
    col = col.push(banner);
  }
  col = col
    .push(header)
    .push(sep_h)
    .push(msg_area)
    .push(sep_compose)
    .push(compose);

  container(col)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::detail_bg())),
      ..Default::default()
    })
    .into()
}

// ---------------------------------------------------------------------------
// Room list panel (280px left column)
// ---------------------------------------------------------------------------

fn room_list_panel(app: &App) -> Element<'_, Message> {
  let s = app.strings;

  let header = room_list_header(app);

  let body: Element<Message> = if app.rooms.is_empty() {
    empty_rooms_state(s)
  } else {
    let rows: Vec<Element<Message>> = app.rooms.iter().map(|room| room_row(room, app)).collect();
    scrollable(Column::with_children(rows).spacing(2).padding([4, 8]))
      .style(common::scroll_style)
      .height(Length::Fill)
      .into()
  };

  let footer = mqtt_status_bar(&app.mqtt_state, s);

  let banner = common::panel_banner(ussr_assets::banner(26), theme::list_panel_bg());

  column![header, banner, body, footer]
    .width(Length::Fixed(280.0))
    .height(Length::Fill)
    .into()
}

fn room_list_header(app: &App) -> Element<'_, Message> {
  let s = app.strings;

  let title = text(theme::flavor(s.nav_chat_rooms(), s.nav_chat_rooms_ussr()))
    .size(13)
    .font(theme::flavor_title_font());

  let btn_new = button(
    row![
      text("\u{f067}").font(theme::ICONS).size(11),
      text(s.chat_create_room()).size(12),
    ]
    .spacing(4)
    .align_y(Alignment::Center),
  )
  .on_press(Message::NavChanged(View::ChatNewRoom))
  .padding([4, 8])
  .style(button_styles::ghost_accent());

  let btn_join = button(text(s.chat_join_room()).size(12))
    .on_press(Message::NavChanged(View::ChatJoinRoom))
    .padding([4, 8])
    .style(button_styles::ghost_neutral());

  container(
    column![
      row![title, Space::new().width(Length::Fill), btn_new, btn_join,]
        .spacing(4)
        .align_y(Alignment::Center),
    ]
    .spacing(4)
    .padding([8, 12]),
  )
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::header_bg())),
    ..Default::default()
  })
  .into()
}

fn room_row<'a>(room: &'a Room, app: &'a App) -> Element<'a, Message> {
  let selected = app.active_room.as_deref() == Some(room.id.as_str());

  let dots = presence_dots(&room.participants, &app.presence, &room.my_fp);

  let ts_label: Element<Message> = {
    let ts = app
      .chat_messages
      .get(&room.id)
      .and_then(|msgs| msgs.back())
      .map(|m| format_time_short(m.ts))
      .unwrap_or_default();
    text(ts)
      .size(11)
      .style(|_: &iced::Theme| iced::widget::text::Style {
        color: Some(theme::text_muted()),
      })
      .into()
  };

  let line1 = row![
    text(room.name.as_str())
      .size(13)
      .font(theme::heading_font())
      .width(Length::Fill),
    dots,
    ts_label,
  ]
  .spacing(6)
  .align_y(Alignment::Center);

  let preview: Element<Message> = {
    let p = app
      .chat_messages
      .get(&room.id)
      .and_then(|msgs| msgs.back())
      .map(|m| truncate_preview(&m.text, 40))
      .unwrap_or_default();
    text(p)
      .size(11)
      .style(|_: &iced::Theme| iced::widget::text::Style {
        color: Some(theme::text_secondary()),
      })
      .into()
  };

  let content = column![line1, preview].spacing(2).width(Length::Fill);

  let styled = container(content)
    .padding([7, 12])
    .width(Length::Fill)
    .style(move |_: &iced::Theme| {
      if selected {
        container::Style {
          background: Some(Background::Color(theme::accent_subtle())),
          border: Border {
            color: theme::accent_border(),
            width: 1.0,
            radius: 6.0.into(),
          },
          ..Default::default()
        }
      } else {
        container::Style::default()
      }
    });

  mouse_area(styled)
    .on_press(Message::ChatRoomSelected(room.id.clone()))
    .into()
}

fn presence_dots<'a>(
  participants: &'a [RoomParticipant],
  tracker: &'a PresenceTracker,
  my_fp: &str,
) -> Element<'a, Message> {
  let others: Vec<_> = participants.iter().filter(|p| p.fp != my_fp).collect();

  let displayed = &others[..others.len().min(4)];
  let overflow = others.len().saturating_sub(4);

  let mut dot_elems: Vec<Element<Message>> = displayed
    .iter()
    .map(|p| {
      let online = tracker
        .get(&p.fp)
        .map(|s| *s == PresenceStatus::Online)
        .unwrap_or(false);
      let (symbol, color) = if online {
        ("●", theme::success())
      } else {
        ("○", theme::text_muted())
      };
      text(symbol).size(11).color(color).into()
    })
    .collect();

  if overflow > 0 {
    dot_elems.push(
      text(format!("+{overflow}"))
        .size(10)
        .style(|_: &iced::Theme| iced::widget::text::Style {
          color: Some(theme::text_muted()),
        })
        .into(),
    );
  }

  row(dot_elems).spacing(2).into()
}

fn empty_rooms_state(s: &'static dyn crate::i18n::Strings) -> Element<'static, Message> {
  container(
    column![
      text(theme::flavor(s.chat_no_rooms(), s.chat_no_rooms_ussr()))
        .size(13)
        .style(|_: &iced::Theme| iced::widget::text::Style {
          color: Some(theme::text_muted()),
        }),
      button(text(s.chat_create_room()).size(12))
        .on_press(Message::NavChanged(View::ChatNewRoom))
        .padding([6, 12])
        .style(button_styles::ghost_accent()),
      button(text(s.chat_join_room()).size(12))
        .on_press(Message::NavChanged(View::ChatJoinRoom))
        .padding([6, 12])
        .style(button_styles::ghost_neutral()),
    ]
    .spacing(12)
    .align_x(Alignment::Center),
  )
  .center_x(Length::Fill)
  .padding([24, 16])
  .height(Length::Fill)
  .into()
}

fn mqtt_status_bar<'a>(
  state: &'a MqttState,
  s: &'static dyn crate::i18n::Strings,
) -> Element<'a, Message> {
  let (dot, label, color) = match state {
    MqttState::Connected => ("●", s.chat_mqtt_connected(), theme::success()),
    MqttState::Connecting => ("◌", s.chat_mqtt_connecting(), theme::peach()),
    MqttState::Reconnecting { .. } => ("◌", s.chat_mqtt_reconnecting(), theme::peach()),
    MqttState::Disconnected => ("✗", s.chat_mqtt_disconnected(), theme::error()),
    MqttState::Failed(_) => ("✗", s.chat_mqtt_failed(), theme::error()),
  };

  container(
    row![
      text(dot).size(10).color(color),
      text(label).size(11).color(color),
    ]
    .spacing(6)
    .align_y(Alignment::Center),
  )
  .padding([6, 12])
  .width(Length::Fill)
  .into()
}

fn chat_header<'a>(room: &'a Room, app: &'a App) -> Element<'a, Message> {
  let s = app.strings;

  let title = text(room.name.as_str())
    .size(15)
    .font(theme::heading_font());

  let participant_badges: Vec<Element<Message>> = room
    .participants
    .iter()
    .filter(|p| p.fp != room.my_fp)
    .map(|p| {
      let online = app
        .presence
        .get(&p.fp)
        .map(|st| *st == PresenceStatus::Online)
        .unwrap_or(false);
      let (dot, color) = if online {
        ("●", theme::success())
      } else {
        ("○", theme::text_muted())
      };
      let short = p.fp.get(..8).unwrap_or(&p.fp);
      row![
        text(dot).size(11).color(color),
        text(short)
          .size(10)
          .font(theme::MONO)
          .style(|_: &iced::Theme| {
            iced::widget::text::Style {
              color: Some(theme::text_muted()),
            }
          }),
      ]
      .spacing(3)
      .align_y(Alignment::Center)
      .into()
    })
    .collect();

  let participants_row = row(participant_badges)
    .spacing(10)
    .align_y(Alignment::Center);

  let btn_copy = button(
    row![
      text("\u{f0c5}").font(theme::ICONS).size(11),
      text(s.chat_copy_invite()).size(12),
    ]
    .spacing(4)
    .align_y(Alignment::Center),
  )
  .on_press(Message::ChatJoinCodeCopy(room.id.clone()))
  .padding([4, 8])
  .style(button_styles::ghost_neutral());

  let btn_leave = button(text(s.chat_leave_room()).size(12))
    .on_press(Message::ChatRoomLeave(room.id.clone()))
    .padding([4, 8])
    .style(button_styles::ghost_destructive());

  container(
    row![
      title,
      Space::new().width(8),
      participants_row,
      Space::new().width(Length::Fill),
      btn_copy,
      btn_leave,
    ]
    .spacing(8)
    .align_y(Alignment::Center),
  )
  .padding([10, 16])
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::header_bg())),
    ..Default::default()
  })
  .into()
}

fn mqtt_warning_banner(s: &'static dyn crate::i18n::Strings) -> Element<'static, Message> {
  container(
    row![
      text("\u{f071}")
        .font(theme::ICONS)
        .size(12)
        .color(theme::peach()),
      text(s.chat_mqtt_disconnected_banner())
        .size(12)
        .color(theme::peach()),
    ]
    .spacing(8)
    .align_y(Alignment::Center),
  )
  .padding([6, 16])
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::warning_bg())),
    ..Default::default()
  })
  .into()
}

fn message_list_for_room<'a>(room: &'a Room, app: &'a App) -> Element<'a, Message> {
  let local_fp = room.my_fp.as_str();

  let bubbles: Vec<Element<Message>> = app
    .chat_messages
    .get(&room.id)
    .into_iter()
    .flat_map(|deque| deque.iter())
    .map(|msg| message_bubble(msg, msg.sender_fp == local_fp, app))
    .collect();

  scrollable(Column::with_children(bubbles).spacing(8).padding([12, 16]))
    .style(common::scroll_style)
    .height(Length::Fill)
    .width(Length::Fill)
    .into()
}

fn message_bubble<'a>(msg: &'a ChatMessage, is_own: bool, app: &'a App) -> Element<'a, Message> {
  let s = app.strings;
  let ts = format_time_short(msg.ts);

  let sender_name: String = app
    .keys
    .iter()
    .find(|k| k.fingerprint == msg.sender_fp)
    .map(|k| k.name.clone())
    .unwrap_or_else(|| msg.sender_fp.get(..8).unwrap_or(&msg.sender_fp).to_string());

  // Chercher la clef dans le keyring pour construire le tooltip d'identité.
  let key_info = app.keys.iter().find(|k| k.fingerprint == msg.sender_fp);
  let is_known = key_info.is_some();

  // Tooltip d'identité : affiché au survol du bloc signature+nom.
  let identity_tooltip: Element<Message> = {
    let muted = |_: &iced::Theme| iced::widget::text::Style {
      color: Some(theme::text_muted()),
    };
    let strong = |_: &iced::Theme| iced::widget::text::Style {
      color: Some(theme::text_strong()),
    };

    let header = if is_known {
      row![
        text("\u{f023}")
          .font(theme::ICONS)
          .size(11)
          .color(theme::success()),
        text("Verified signature")
          .size(11)
          .style(|_: &iced::Theme| iced::widget::text::Style {
            color: Some(theme::success()),
          }),
      ]
      .spacing(4)
      .align_y(Alignment::Center)
    } else {
      row![
        text("\u{f071}")
          .font(theme::ICONS)
          .size(11)
          .color(theme::peach()),
        text("Unverified — key not in keyring")
          .size(11)
          .style(|_: &iced::Theme| iced::widget::text::Style {
            color: Some(theme::peach()),
          }),
      ]
      .spacing(4)
      .align_y(Alignment::Center)
    };

    // Fingerprint formaté en groupes de 4
    let fp_display = if msg.sender_fp.len() == 40 {
      let s = &msg.sender_fp;
      format!(
        "{} {} {} {} {}\n{} {} {} {} {}",
        &s[0..4],
        &s[4..8],
        &s[8..12],
        &s[12..16],
        &s[16..20],
        &s[20..24],
        &s[24..28],
        &s[28..32],
        &s[32..36],
        &s[36..40]
      )
    } else {
      msg.sender_fp.clone()
    };

    let mut col = column![
      header,
      rule::horizontal(1).style(|_: &iced::Theme| rule::Style {
        color: theme::border(),
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: false,
      }),
    ]
    .spacing(6);

    if let Some(k) = key_info {
      col = col
        .push(
          column![
            text(k.name.as_str()).size(12).style(strong),
            text(k.email.as_str()).size(11).style(muted),
          ]
          .spacing(2),
        )
        .push(
          column![
            text("Fingerprint").size(10).style(muted),
            text(fp_display.clone())
              .size(10)
              .font(theme::MONO)
              .style(strong),
          ]
          .spacing(2),
        )
        .push(row![
          text("Trust: ").size(10).style(muted),
          text(format!("{:?}", k.trust)).size(10).style(strong),
        ]);
    } else {
      col = col.push(
        column![
          text("Fingerprint").size(10).style(muted),
          text(fp_display.clone())
            .size(10)
            .font(theme::MONO)
            .style(strong),
          text("Import this key to verify identity.")
            .size(10)
            .style(muted),
        ]
        .spacing(2),
      );
    }

    container(col.padding(10))
      .max_width(260)
      .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(theme::card_bg())),
        border: Border {
          color: theme::border(),
          width: 1.0,
          radius: 8.0.into(),
        },
        ..Default::default()
      })
      .into()
  };

  // Bloc cliquable avec tooltip : cadenas + nom.
  let sig_icon: Element<Message> = if is_known {
    text("\u{f023}")
      .font(theme::ICONS)
      .size(10)
      .color(theme::success())
      .into()
  } else if msg.sender_fp.is_empty() {
    text("\u{f071}")
      .font(theme::ICONS)
      .size(10)
      .color(theme::peach())
      .into()
  } else {
    text("\u{f29c}")
      .font(theme::ICONS)
      .size(10)
      .color(theme::text_muted())
      .into()
  };

  // Identifié par msg.id (pas sender_fp) pour que chaque message ait son propre toggle.
  let popup_active = app.chat_identity_popup.as_deref() == Some(msg.id.as_str());
  let msg_id_for_toggle = msg.id.clone();

  let sig_and_name: Element<Message> = mouse_area(
    row![
      sig_icon,
      text(sender_name)
        .size(11)
        .style(|_: &iced::Theme| iced::widget::text::Style {
          color: Some(theme::text_muted()),
        }),
    ]
    .spacing(4)
    .align_y(Alignment::Center),
  )
  .on_press(Message::ChatIdentityPopupToggle(msg_id_for_toggle))
  .into();

  let meta = row![
    sig_and_name,
    text(" · ")
      .size(10)
      .style(|_: &iced::Theme| iced::widget::text::Style {
        color: Some(theme::text_muted()),
      }),
    text(ts)
      .size(10)
      .style(|_: &iced::Theme| iced::widget::text::Style {
        color: Some(theme::text_muted()),
      }),
  ]
  .spacing(4)
  .align_y(Alignment::Center);

  // Detect decrypt failure marker
  let body_text: Element<Message> = if msg.text.starts_with("[DECRYPT_FAILED]") {
    container(
      row![
        text("\u{f071}")
          .font(theme::ICONS)
          .size(11)
          .color(theme::text_muted()),
        text(s.chat_decrypt_failed())
          .size(13)
          .style(|_: &iced::Theme| {
            iced::widget::text::Style {
              color: Some(theme::text_muted()),
            }
          }),
      ]
      .spacing(6)
      .align_y(Alignment::Center),
    )
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::disabled_bg())),
      border: Border {
        radius: 4.0.into(),
        ..Default::default()
      },
      ..Default::default()
    })
    .into()
  } else {
    text(msg.text.as_str())
      .size(13)
      .style(|_: &iced::Theme| iced::widget::text::Style {
        color: Some(theme::text_strong()),
      })
      .into()
  };

  let bubble_content: Element<Message> = {
    let mut col = column![meta, body_text].spacing(4);

    if is_own {
      col = col.push(ack_indicators(&msg.acks, app));
    }

    // Panneau d'identité inline, visible après un clic sur le cadenas/nom.
    if popup_active {
      let sep = rule::horizontal(1).style(|_: &iced::Theme| rule::Style {
        color: theme::border(),
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: false,
      });
      col = col.push(sep).push(identity_tooltip);
    }

    col.into()
  };

  let bubble_style = move |_: &iced::Theme| {
    if is_own {
      container::Style {
        background: Some(Background::Color(theme::accent_subtle())),
        border: Border {
          radius: 8.0.into(),
          ..Default::default()
        },
        ..Default::default()
      }
    } else {
      container::Style {
        background: Some(Background::Color(theme::card_bg())),
        border: Border {
          color: theme::accent_border(),
          width: 2.0,
          radius: 8.0.into(),
        },
        ..Default::default()
      }
    }
  };

  let bubble = container(bubble_content)
    .padding([8, 12])
    .max_width(480)
    .style(bubble_style);

  if is_own {
    row![Space::new().width(Length::Fill), bubble]
      .spacing(8)
      .align_y(Alignment::Start)
      .into()
  } else {
    row![bubble, Space::new().width(Length::Fill)]
      .spacing(8)
      .align_y(Alignment::Start)
      .into()
  }
}

fn ack_indicators<'a>(acks: &'a HashMap<String, AckStatus>, app: &'a App) -> Element<'a, Message> {
  let my_fp = app
    .rooms
    .iter()
    .find(|r| Some(r.id.as_str()) == app.active_room.as_deref())
    .map(|r| r.my_fp.as_str())
    .unwrap_or("");

  let indicators: Vec<Element<Message>> = acks
    .iter()
    .filter(|(fp, _)| fp.as_str() != my_fp)
    .map(|(fp, status)| {
      let short = fp.get(..8).unwrap_or(fp.as_str());
      let (symbol, color) = match status {
        AckStatus::Received => ("\u{2713}", theme::success()),
        AckStatus::Pending => ("\u{23f3}", theme::text_muted()),
      };
      row![
        text(symbol).size(10).color(color),
        text(short).size(10).font(theme::MONO).color(color),
      ]
      .spacing(2)
      .align_y(Alignment::Center)
      .into()
    })
    .collect();

  row(indicators).spacing(8).into()
}

fn compose_bar<'a>(room_id: &str, app: &'a App) -> Element<'a, Message> {
  use iced::widget::text_input;

  let s = app.strings;
  let can_send = !app.chat_input.is_empty() && app.mqtt_state == MqttState::Connected;

  let input = text_input(s.chat_type_message(), &app.chat_input)
    .on_input(Message::ChatInputChanged)
    .on_submit(Message::ChatSend)
    .padding([8, 12])
    .size(13)
    .width(Length::Fill)
    .style(|_: &iced::Theme, _status| text_input::Style {
      background: Background::Color(theme::detail_bg()),
      border: Border {
        color: theme::border(),
        width: 1.0,
        radius: 6.0.into(),
      },
      placeholder: theme::text_muted(),
      value: theme::text_strong(),
      selection: theme::accent_subtle(),
      icon: theme::text_muted(),
    });

  let _ = room_id; // room_id used for context, ChatSend reads from app state

  let send_btn = button(
    text("\u{f1d8}")
      .font(theme::ICONS)
      .size(14)
      .color(theme::text_on_accent()),
  )
  .padding([8, 12])
  .style(button_styles::primary_toggle(can_send));

  let send_btn = if can_send {
    send_btn.on_press(Message::ChatSend)
  } else {
    send_btn
  };

  container(row![input, send_btn].spacing(8).align_y(Alignment::Center))
    .padding([8, 12])
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::card_bg())),
      ..Default::default()
    })
    .into()
}

fn empty_chat_state(s: &'static dyn crate::i18n::Strings) -> Element<'static, Message> {
  container(
    text(s.chat_select_room())
      .size(13)
      .style(|_: &iced::Theme| iced::widget::text::Style {
        color: Some(theme::text_muted()),
      }),
  )
  .center_x(Length::Fill)
  .center_y(Length::Fill)
  .width(Length::Fill)
  .height(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::detail_bg())),
    ..Default::default()
  })
  .into()
}

// ---------------------------------------------------------------------------
// Modal — leave room (rendered inside chat_panel)
// ---------------------------------------------------------------------------

fn leave_room_modal<'a>(
  room: &'a Room,
  s: &'static dyn crate::i18n::Strings,
) -> Element<'a, Message> {
  let body_text: String = s.chat_leave_confirm_body_with_name(room.name.as_str());

  let content = column![
    text(s.chat_leave_confirm_title())
      .size(16)
      .font(theme::heading_font()),
    text(body_text)
      .size(13)
      .style(|_: &iced::Theme| iced::widget::text::Style {
        color: Some(theme::text_secondary()),
      }),
    row![
      button(text(s.btn_cancel()).size(13))
        .on_press(Message::MoveToCardCancel)
        .padding([8, 16])
        .style(button_styles::ghost_neutral()),
      Space::new().width(Length::Fill),
      button(text(s.chat_leave_room_btn()).size(13))
        .on_press(Message::ChatRoomLeave(room.id.clone()))
        .padding([8, 16])
        .style(button_styles::ghost_destructive()),
    ]
    .align_y(Alignment::Center),
  ]
  .spacing(16)
  .padding(32);

  container(
    container(content)
      .max_width(480)
      .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(theme::card_bg())),
        border: Border {
          radius: 12.0.into(),
          ..Default::default()
        },
        ..Default::default()
      }),
  )
  .center_x(Length::Fill)
  .center_y(Length::Fill)
  .width(Length::Fill)
  .height(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::detail_bg())),
    ..Default::default()
  })
  .into()
}

// ---------------------------------------------------------------------------
// Modal — identity selection (rendered when no active_room but pending op set)
// ---------------------------------------------------------------------------

fn identity_selection_modal<'a>(
  room_id: &str,
  selected_fp: &'a Option<String>,
  app: &'a App,
) -> Element<'a, Message> {
  use iced::widget::radio;

  let s = app.strings;
  let private_keys: Vec<_> = app.keys.iter().filter(|k| k.has_secret).collect();

  // Use index as the radio value since String is not Copy.
  let selected_idx: Option<usize> = selected_fp
    .as_ref()
    .and_then(|fp| private_keys.iter().position(|k| &k.fingerprint == fp));

  let radio_items: Vec<Element<Message>> = private_keys
    .iter()
    .enumerate()
    .map(|(idx, key)| {
      let fp = key.fingerprint.clone();
      column![
        radio(
          format!("{} <{}>", key.name, key.email),
          idx,
          selected_idx,
          move |_| Message::ChatIdentitySelected(fp.clone()),
        )
        .style(common::radio_style)
        .text_size(13),
        text(key.fingerprint.as_str())
          .size(10)
          .font(theme::MONO)
          .style(|_: &iced::Theme| iced::widget::text::Style {
            color: Some(theme::text_muted()),
          }),
      ]
      .spacing(2)
      .padding([4, 24])
      .into()
    })
    .collect();

  let can_enter = selected_fp.is_some();
  let is_global = room_id.is_empty();

  let content = column![
    text(s.chat_choose_identity_title())
      .size(16)
      .font(theme::heading_font()),
    text(s.chat_choose_identity_hint())
      .size(13)
      .style(|_: &iced::Theme| {
        iced::widget::text::Style {
          color: Some(theme::text_secondary()),
        }
      }),
    Column::with_children(radio_items).spacing(8),
    row![
      button(text(s.btn_cancel()).size(13))
        .on_press(Message::NavBack)
        .padding([8, 16])
        .style(button_styles::ghost_neutral()),
      Space::new().width(Length::Fill),
      button(
        text(if is_global {
          s.chat_confirm_identity_btn()
        } else {
          s.chat_enter_room_btn()
        })
        .size(13),
      )
      .on_press_maybe(can_enter.then_some(Message::ChatIdentityConfirm))
      .padding([8, 16])
      .style(button_styles::primary_toggle(can_enter)),
    ]
    .align_y(Alignment::Center),
  ]
  .spacing(16)
  .padding(32);

  container(
    container(content)
      .max_width(480)
      .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(theme::card_bg())),
        border: Border {
          radius: 12.0.into(),
          ..Default::default()
        },
        ..Default::default()
      }),
  )
  .center_x(Length::Fill)
  .center_y(Length::Fill)
  .width(Length::Fill)
  .height(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::detail_bg())),
    ..Default::default()
  })
  .into()
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Formats a UTC datetime as "HH:MM" in local time.
fn format_time_short(dt: chrono::DateTime<chrono::Utc>) -> String {
  use chrono::Local;
  dt.with_timezone(&Local).format("%H:%M").to_string()
}

/// Truncates text to `max_chars` Unicode scalar values, appending "…" if truncated.
fn truncate_preview(s: &str, max_chars: usize) -> String {
  let mut chars = s.chars();
  let truncated: String = chars.by_ref().take(max_chars).collect();
  if chars.next().is_some() {
    format!("{truncated}\u{2026}")
  } else {
    truncated
  }
}
