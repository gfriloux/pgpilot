pub mod create_key;
pub mod key_detail;
pub mod key_list;

use iced::{
  widget::{button, column, container, horizontal_rule, row, text},
  Element, Length,
};

use crate::app::{App, Message, View};

pub fn root(app: &App) -> Element<'_, Message> {
  let sidebar = sidebar(app);
  let content = match app.view {
    View::MyKeys | View::PublicKeys => key_list::view(app),
    View::CreateKey => create_key::view(&app.create_form),
  };

  let main: Element<Message> = match &app.status {
    Some(status) => column![
      content,
      horizontal_rule(1),
      container(text(status.as_str()).size(12)).padding(4),
    ]
    .height(Length::Fill)
    .width(Length::Fill)
    .into(),
    None => content,
  };

  row![sidebar, main]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn sidebar(app: &App) -> Element<'_, Message> {
  let nav_btn = |label: &'static str, view: View| {
    let active = app.view == view;
    let btn = button(text(label).size(14))
      .on_press(Message::NavChanged(view))
      .width(Length::Fill);
    if active {
      btn.style(button::primary)
    } else {
      btn.style(button::text)
    }
  };

  let import_btn = button(text("↑ Importer une clef").size(14))
    .on_press(Message::ImportKey)
    .width(Length::Fill)
    .style(button::text);

  container(
    column![
      text("pgpilot").size(20),
      nav_btn("Mes clefs", View::MyKeys),
      nav_btn("Clefs publiques", View::PublicKeys),
      horizontal_rule(1),
      import_btn,
      nav_btn("+ Créer une clef", View::CreateKey),
    ]
    .spacing(8)
    .padding(12),
  )
  .width(180)
  .height(Length::Fill)
  .into()
}
