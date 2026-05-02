pub mod create_key;
pub mod health;
pub mod import;
pub mod key_detail;
pub mod key_list;
pub mod theme;

use iced::{
  font,
  widget::{button, column, container, row, text},
  Alignment, Background, Border, Color, Element, Font, Length,
};

use crate::app::{App, Message, View};

pub fn root(app: &App) -> Element<'_, Message> {
  let content = match app.view {
    View::MyKeys | View::PublicKeys => key_list::view(app),
    View::CreateKey => create_key::view(&app.create_form),
    View::Import => import::view(&app.import_form),
    View::Health => health::view(&app.health_report, app.health_loading),
  };

  let main: Element<Message> = match &app.status {
    Some(status) => {
      let is_error = status.starts_with("Erreur");
      let (bg, fg) = if is_error {
        (theme::ERROR_BG, theme::ERROR)
      } else {
        (theme::SUCCESS_BG, theme::SUCCESS)
      };
      column![
        content,
        container(text(status.as_str()).size(12))
          .padding([8, 16])
          .width(Length::Fill)
          .style(move |_: &iced::Theme| container::Style {
            background: Some(Background::Color(bg)),
            text_color: Some(fg),
            ..Default::default()
          }),
      ]
      .height(Length::Fill)
      .width(Length::Fill)
      .into()
    }
    None => content,
  };

  let sidebar_el = container(sidebar(app))
    .height(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::SIDEBAR_BG)),
      text_color: Some(theme::SIDEBAR_TEXT),
      ..Default::default()
    });

  row![sidebar_el, main]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn sidebar(app: &App) -> Element<'_, Message> {
  let nav_btn = |icon: &'static str, label: &'static str, view: View| {
    let active = app.view == view;
    button(
      row![text(icon).font(theme::ICONS).size(14), text(label).size(13),]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .on_press(Message::NavChanged(view))
    .width(Length::Fill)
    .style(
      move |_: &iced::Theme, status: button::Status| button::Style {
        background: if active {
          Some(Background::Color(theme::ACCENT))
        } else {
          match status {
            button::Status::Hovered | button::Status::Pressed => {
              Some(Background::Color(theme::SIDEBAR_HOVER_BG))
            }
            _ => None,
          }
        },
        text_color: if active {
          theme::TEXT_ON_ACCENT
        } else {
          theme::SIDEBAR_TEXT
        },
        border: Border {
          color: Color::TRANSPARENT,
          width: 0.0,
          radius: 6.0.into(),
        },
        shadow: Default::default(),
      },
    )
  };

  let title_font = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
  };

  column![
    row![
      text("\u{f084}").font(theme::ICONS).size(18),
      text("pgpilot").size(20).font(title_font),
    ]
    .spacing(8)
    .align_y(Alignment::Center),
    column![
      nav_btn("\u{f084}", "Mes clefs", View::MyKeys),
      nav_btn("\u{f0c0}", "Clefs publiques", View::PublicKeys),
    ]
    .spacing(2),
    column![
      nav_btn("\u{f093}", "Importer", View::Import),
      nav_btn("\u{f067}", "Créer une clef", View::CreateKey),
    ]
    .spacing(2),
    nav_btn("\u{f132}", "Diagnostic", View::Health),
  ]
  .spacing(16)
  .padding(12)
  .width(180)
  .height(Length::Fill)
  .into()
}
