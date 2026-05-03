use iced::{
  font,
  widget::{column, container, mouse_area, row, rule, scrollable, text, Column},
  Alignment, Background, Border, Element, Font, Length,
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
    .filter(|k| match app.view {
      View::MyKeys => k.has_secret,
      View::PublicKeys => !k.has_secret,
      _ => false,
    })
    .collect();

  if keys.is_empty() {
    return container(text("Aucune clef trouvée.").size(14))
      .padding(24)
      .center_x(Length::Fill)
      .height(Length::Fill)
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
      text("Nom / Email").size(11).width(Length::Fill).font(bold),
      text("Expire").size(11).width(80).font(bold),
      text("").size(11).width(56),
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
        KeyserverStatus::Published => ("\u{f058}", theme::SUCCESS),
        KeyserverStatus::NotPublished => ("\u{f10c}", theme::TEXT_MUTED),
        _ => ("", theme::TEXT_MUTED),
      };

      let (trust_icon, trust_color) = if key.has_secret || key.on_card {
        ("", theme::TEXT_MUTED)
      } else if key.trust.is_sufficient() {
        ("\u{f058}", theme::SUCCESS)
      } else {
        ("\u{f071}", theme::PEACH)
      };

      let name_col = column![
        text(key.name.clone()).size(13),
        text(key.email.clone()).size(11).style(|_: &iced::Theme| {
          iced::widget::text::Style {
            color: Some(theme::TEXT_MUTED),
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

  let list_panel = column![header, list_scrollable.height(Length::Fill)]
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
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::DETAIL_BG)),
      ..Default::default()
    })
    .into()
  } else {
    container(
      text("Sélectionnez une clef pour voir les détails.")
        .size(13)
        .style(|_: &iced::Theme| iced::widget::text::Style {
          color: Some(theme::TEXT_MUTED),
        }),
    )
    .padding(24)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::DETAIL_BG)),
      ..Default::default()
    })
    .into()
  };

  row![
    list_panel,
    rule::vertical(1).style(|_: &iced::Theme| rule::Style {
      color: theme::BORDER,
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
