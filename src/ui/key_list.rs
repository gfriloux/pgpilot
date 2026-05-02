use iced::{
  font,
  widget::{column, container, horizontal_rule, mouse_area, row, rule, scrollable, text, Column},
  Background, Border, Element, Font, Length,
};

use crate::app::{App, KeyserverStatus, Message, View};
use crate::ui::key_detail::ViewCtx;
use crate::ui::{key_detail, theme};

pub fn view(app: &App) -> Element<'_, Message> {
  if app.loading {
    return container(text("Chargement...").size(14))
      .padding(24)
      .style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::TEXT_MUTED),
        ..Default::default()
      })
      .into();
  }

  if let Some(ref err) = app.error {
    return container(text(format!("Erreur : {err}")).size(14))
      .padding(24)
      .style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::ERROR),
        ..Default::default()
      })
      .into();
  }

  let keys: Vec<_> = app
    .keys
    .iter()
    .enumerate()
    .filter(|(_, k)| match app.view {
      View::MyKeys => k.has_secret,
      View::PublicKeys => !k.has_secret,
      _ => false,
    })
    .collect();

  if keys.is_empty() {
    return container(text("Aucune clef trouvée.").size(14))
      .padding(24)
      .style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::TEXT_MUTED),
        ..Default::default()
      })
      .into();
  }

  let bold = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
  };

  let header = container(
    row![
      text("Nom").size(11).width(200).font(bold),
      text("Email").size(11).width(250).font(bold),
      text("ID").size(11).width(120).font(bold),
      text("Expire").size(11).width(100).font(bold),
    ]
    .padding([0, 12])
    .spacing(8),
  )
  .padding([8, 0])
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::HEADER_BG)),
    text_color: Some(theme::TEXT_HEADER),
    ..Default::default()
  });

  let key_rows: Vec<Element<Message>> = keys
    .iter()
    .map(|(_, key)| {
      let selected = app.selected.as_deref() == Some(key.fingerprint.as_str());

      let name = key.name.clone();
      let expires = key.expires.as_deref().unwrap_or("—");
      let card_icon = if key.on_card { "\u{f283}" } else { "" };

      let (pub_icon, pub_color) = match app
        .keyserver_statuses
        .get(&key.fingerprint)
        .copied()
        .unwrap_or_default()
      {
        KeyserverStatus::Published => ("\u{f058}", theme::SUCCESS),
        KeyserverStatus::NotPublished => ("\u{f10c}", theme::TEXT_MUTED),
        _ => ("", theme::TEXT_MUTED),
      };

      let row_content = row![
        text(name).size(13).width(200),
        text(key.email.clone()).size(13).width(250),
        text(key.short_id.clone()).size(12).width(120),
        text(expires).size(12).width(100),
        text(card_icon).font(theme::ICONS).size(12).width(20),
        text(pub_icon)
          .font(theme::ICONS)
          .size(11)
          .color(pub_color)
          .width(20),
      ]
      .spacing(8);

      let styled = container(row_content)
        .padding([7, 12])
        .width(Length::Fill)
        .style(move |_: &iced::Theme| {
          if selected {
            container::Style {
              background: Some(Background::Color(theme::ACCENT_SUBTLE)),
              border: Border {
                color: theme::ACCENT_BORDER,
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

  let list_scrollable = scrollable(Column::with_children(key_rows).spacing(2).padding([4, 8]));

  let list_view = column![header, list_scrollable.height(Length::Fill)]
    .spacing(0)
    .width(Length::Fill);

  let selected_key = app
    .selected
    .as_ref()
    .and_then(|fp| app.keys.iter().find(|k| &k.fingerprint == fp));

  if let Some(key) = selected_key {
    let key_fp = &key.fingerprint;
    column![
      list_view.height(Length::Fill),
      horizontal_rule(1).style(|_: &iced::Theme| rule::Style {
        color: theme::BORDER,
        width: 1,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
      }),
      container(key_detail::view(
        key,
        ViewCtx {
          card_connected: app.card_connected,
          confirming: app.pending_migration.as_deref() == Some(key_fp),
          delete_confirming: app.pending_delete.as_deref() == Some(key_fp),
          export_pub_menu: app.pending_export_pub.as_deref() == Some(key_fp),
          renewing_subkey: app.pending_renewal.as_ref().and_then(|r| {
            if r.key_fp == *key_fp {
              Some((r.subkey_fp.clone(), r.expiry.clone()))
            } else {
              None
            }
          }),
          publish_confirming: app.pending_publish.clone(),
          keyserver_status: app
            .keyserver_statuses
            .get(key_fp)
            .copied()
            .unwrap_or_default(),
        },
      ))
      .width(Length::Fill)
      .style(|_: &iced::Theme| container::Style {
        background: Some(Background::Color(theme::DETAIL_BG)),
        ..Default::default()
      }),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
  } else {
    list_view.height(Length::Fill).into()
  }
}
