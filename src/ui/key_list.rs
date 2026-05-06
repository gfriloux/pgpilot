use iced::{
  font,
  widget::{button, column, container, mouse_area, row, rule, scrollable, text, Column},
  Alignment, Background, Border, Element, Font, Length,
};

use crate::gpg::SubkeyType;

use crate::app::{App, KeyserverStatus, Message, View};
use crate::ui::key_detail::ViewCtx;
use crate::ui::{common, key_detail, theme};

pub fn view(app: &App) -> Element<'_, Message> {
  let s = app.strings;

  if app.loading {
    return container(text(s.loading()).size(14))
      .padding(24)
      .style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::text_muted()),
        ..Default::default()
      })
      .into();
  }

  if let Some(ref err) = app.error {
    return container(text(s.key_list_error(err)).size(14))
      .padding(24)
      .style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::error()),
        ..Default::default()
      })
      .into();
  }

  let keys: Vec<_> = app
    .keys
    .iter()
    .filter(|k| match app.view {
      View::MyKeys => k.has_secret,
      View::PublicKeys => !k.has_secret,
      _ => false,
    })
    .collect();

  if keys.is_empty() {
    return container(text(s.no_keys()).size(14))
      .padding(24)
      .center_x(Length::Fill)
      .height(Length::Fill)
      .style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::text_muted()),
        ..Default::default()
      })
      .into();
  }

  let bold = Font {
    weight: font::Weight::Bold,
    ..theme::heading_font()
  };

  let header = container(
    row![
      text(s.key_list_header_name())
        .size(11)
        .width(Length::Fill)
        .font(bold),
      text(s.key_list_header_expires())
        .size(11)
        .width(80)
        .font(bold),
      text(s.key_list_header_status())
        .size(11)
        .width(56)
        .font(theme::heading_font()),
    ]
    .padding([0, 12])
    .spacing(8),
  )
  .padding([8, 0])
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::header_bg())),
    text_color: Some(theme::text_header()),
    ..Default::default()
  });

  let key_rows: Vec<Element<Message>> = keys
    .iter()
    .map(|key| {
      let selected = app.selected.as_deref() == Some(key.fingerprint.as_str());

      let expires = key.expires.as_deref().unwrap_or("—");
      let card_icon = if key.on_card { "\u{f283}" } else { "" };

      let (pub_icon, pub_color) = match app
        .keyserver_statuses
        .get(&key.fingerprint)
        .copied()
        .unwrap_or_default()
      {
        KeyserverStatus::Published => (theme::icon_published(), theme::success()),
        KeyserverStatus::NotPublished => ("\u{f10c}", theme::text_muted()),
        _ => ("", theme::text_muted()),
      };

      let (trust_icon, trust_color) = if key.has_secret || key.on_card {
        ("", theme::text_muted())
      } else if key.trust.is_sufficient() {
        ("\u{f058}", theme::success())
      } else {
        ("\u{f071}", theme::peach())
      };

      let name_col = column![
        text(key.name.clone()).size(13).font(theme::heading_font()),
        text(key.email.clone()).size(11).style(|_: &iced::Theme| {
          iced::widget::text::Style {
            color: Some(theme::text_muted()),
          }
        }),
      ]
      .spacing(1)
      .width(Length::Fill);

      let icons = row![
        text(card_icon).font(theme::ICONS).size(11).width(16),
        text(pub_icon)
          .font(theme::ICONS)
          .size(11)
          .color(pub_color)
          .width(16),
        text(trust_icon)
          .font(theme::ICONS)
          .size(11)
          .color(trust_color)
          .width(16),
      ]
      .spacing(4)
      .width(56);

      let row_content = row![name_col, text(expires).size(11).width(80), icons,]
        .spacing(8)
        .align_y(Alignment::Center);

      let styled = container(row_content)
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
        .on_press(Message::KeySelected(key.fingerprint.clone()))
        .into()
    })
    .collect();

  let list_scrollable = scrollable(Column::with_children(key_rows).spacing(2).padding([4, 8]))
    .style(common::scroll_style);

  let mut list_col = column![header];

  if !app.expiry_warnings.is_empty() {
    let warning_rows: Vec<Element<Message>> = app
      .expiry_warnings
      .iter()
      .map(|w| {
        let days_left = (w.expires_at - chrono::Utc::now()).num_days();
        let days_label = if days_left < 1 {
          "< 1 day".to_string()
        } else if days_left == 1 {
          "1 day".to_string()
        } else {
          format!("{days_left} days")
        };
        let type_label = match w.subkey_type {
          Some(SubkeyType::Sign) => "Sign",
          Some(SubkeyType::Encr) => "Encr",
          Some(SubkeyType::Auth) => "Auth",
          None => "?",
        };
        let key_fp = w.key_fp.clone();
        let label = format!("{} · {} · {}", w.key_name, type_label, days_label);
        let renew_fp = w.key_fp.clone();
        let row_content = row![
          container(
            mouse_area(
              text(label)
                .size(12)
                .style(|_: &iced::Theme| iced::widget::text::Style {
                  color: Some(theme::text_strong()),
                })
            )
            .on_press(Message::KeySelected(key_fp))
          )
          .width(Length::Fill),
          button(
            text(s.expiry_warning_renew())
              .size(11)
              .style(|_: &iced::Theme| iced::widget::text::Style {
                color: Some(theme::text_on_accent()),
              })
          )
          .on_press(Message::KeySelected(renew_fp))
          .padding([2, 8])
          .style(|_: &iced::Theme, _| button::Style {
            background: Some(Background::Color(theme::accent())),
            border: Border {
              radius: 4.0.into(),
              ..Default::default()
            },
            text_color: theme::text_on_accent(),
            ..Default::default()
          }),
        ]
        .spacing(8)
        .align_y(Alignment::Center);
        row_content.into()
      })
      .collect();

    let warning_icon = text("\u{f071}")
      .font(theme::ICONS)
      .size(13)
      .style(|_: &iced::Theme| iced::widget::text::Style {
        color: Some(theme::text_strong()),
      });
    let warning_title = text(s.expiry_warning_title())
      .size(12)
      .font(Font {
        weight: font::Weight::Bold,
        ..Font::DEFAULT
      })
      .style(|_: &iced::Theme| iced::widget::text::Style {
        color: Some(theme::text_strong()),
      });
    let title_row = row![warning_icon, warning_title]
      .spacing(6)
      .align_y(Alignment::Center);

    let warning_body = column(
      std::iter::once(title_row.into())
        .chain(warning_rows)
        .collect::<Vec<_>>(),
    )
    .spacing(4);

    let banner = container(warning_body)
      .padding([8, 12])
      .width(Length::Fill)
      .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(theme::warning_bg())),
        ..Default::default()
      });

    list_col = list_col.push(banner);
  }

  let list_panel = list_col
    .push(list_scrollable.height(Length::Fill))
    .spacing(0)
    .width(Length::Fixed(320.0))
    .height(Length::Fill);

  let selected_key = app
    .selected
    .as_ref()
    .and_then(|fp| app.keys.iter().find(|k| &k.fingerprint == fp));

  let detail_panel: Element<'_, Message> = if let Some(key) = selected_key {
    let key_fp = &key.fingerprint;
    container(key_detail::view(
      key,
      ViewCtx {
        card_connected: app.card_connected,
        confirming: matches!(&app.pending, Some(crate::app::PendingOp::Migration(fp)) if fp == key_fp),
        delete_confirming: matches!(&app.pending, Some(crate::app::PendingOp::Delete(fp)) if fp == key_fp),
        export_pub_menu: matches!(&app.pending, Some(crate::app::PendingOp::ExportPubMenu(fp)) if fp == key_fp),
        renewing_subkey: match &app.pending {
          Some(crate::app::PendingOp::Renewal(r)) if r.key_fp == *key_fp => {
            Some((r.subkey_fp.clone(), r.expiry.clone()))
          }
          _ => None,
        },
        publish_confirming: match &app.pending {
          Some(crate::app::PendingOp::Publish(ks)) => Some(ks.clone()),
          _ => None,
        },
        keyserver_status: app
          .keyserver_statuses
          .get(key_fp)
          .copied()
          .unwrap_or_default(),
      },
      s,
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::detail_bg())),
      ..Default::default()
    })
    .into()
  } else {
    container(
      text(s.key_list_select_hint())
        .size(13)
        .style(|_: &iced::Theme| iced::widget::text::Style {
          color: Some(theme::text_muted()),
        }),
    )
    .padding(24)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::detail_bg())),
      ..Default::default()
    })
    .into()
  };

  row![
    list_panel,
    rule::vertical(1).style(|_: &iced::Theme| rule::Style {
      color: theme::border(),
      radius: 0.0.into(),
      fill_mode: rule::FillMode::Full,
      snap: false,
    }),
    detail_panel,
  ]
  .width(Length::Fill)
  .height(Length::Fill)
  .into()
}
