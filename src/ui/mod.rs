pub mod key_detail;
pub mod key_list;

use iced::{
  widget::{button, column, container, row, text},
  Element, Length,
};

use crate::app::{App, Message, View};

pub fn root(app: &App) -> Element<'_, Message> {
  let sidebar = sidebar(app);
  let content = match app.view {
    View::MyKeys | View::PublicKeys => key_list::view(app),
  };

  row![sidebar, content]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn sidebar(_app: &App) -> Element<'_, Message> {
  let my_keys_btn = button(text("Mes clefs").size(14))
    .on_press(Message::NavChanged(View::MyKeys))
    .width(Length::Fill);

  let pub_keys_btn = button(text("Clefs publiques").size(14))
    .on_press(Message::NavChanged(View::PublicKeys))
    .width(Length::Fill);

  container(
    column![text("pgpilot").size(20), my_keys_btn, pub_keys_btn]
      .spacing(8)
      .padding(12),
  )
  .width(180)
  .height(Length::Fill)
  .into()
}
