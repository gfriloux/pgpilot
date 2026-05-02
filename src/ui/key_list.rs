use iced::{
  widget::{column, container, horizontal_rule, mouse_area, row, scrollable, text, Column},
  Background, Color, Element, Length,
};

use crate::app::{App, Message};
use crate::ui::key_detail;

pub fn view(app: &App) -> Element<'_, Message> {
  if app.loading {
    return text("Chargement...").into();
  }

  if let Some(ref err) = app.error {
    return text(format!("Erreur : {err}")).into();
  }

  if app.keys.is_empty() {
    return text("Aucune clef trouvée.").into();
  }

  let header = row![
    text("Nom").width(200),
    text("Email").width(250),
    text("ID").width(120),
    text("Expire").width(100),
  ]
  .padding([4, 12])
  .spacing(8);

  let key_rows: Vec<Element<Message>> = app
    .keys
    .iter()
    .enumerate()
    .map(|(i, key)| {
      let name = if key.has_secret {
        format!("★ {}", key.name)
      } else {
        key.name.clone()
      };
      let expires = key.expires.as_deref().unwrap_or("—");

      let row_content = row![
        text(name).width(200),
        text(key.email.clone()).width(250),
        text(key.short_id.clone()).width(120),
        text(expires).width(100),
      ]
      .padding([4, 12])
      .spacing(8);

      let styled = if app.selected == Some(i) {
        container(row_content)
          .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(Color::from_rgba(0.3, 0.5, 1.0, 0.15))),
            ..Default::default()
          })
          .width(Length::Fill)
      } else {
        container(row_content).width(Length::Fill)
      };

      mouse_area(styled).on_press(Message::KeySelected(i)).into()
    })
    .collect();

  let list_scrollable = scrollable(Column::with_children(key_rows).spacing(2));

  let list_view = column![header, list_scrollable.height(Length::Fill)]
    .spacing(4)
    .padding(12)
    .width(Length::Fill);

  if let Some(idx) = app.selected {
    column![
      list_view.height(Length::Fill),
      horizontal_rule(1),
      key_detail::view(&app.keys[idx], idx),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
  } else {
    list_view.height(Length::Fill).into()
  }
}
