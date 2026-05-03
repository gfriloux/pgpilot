pub mod button_styles;
pub mod common;
pub mod create_key;
pub mod decrypt;
pub mod encrypt;
pub mod health;
pub mod import;
pub mod key_detail;
pub mod key_list;
pub mod sign;
pub mod theme;
pub mod verify;

use iced::{
  font,
  widget::{button, column, container, row, rule, text, Space},
  Alignment, Background, Border, Color, Element, Font, Length, Shadow,
};

use crate::app::{App, Message, StatusKind, View};

pub fn root(app: &App) -> Element<'_, Message> {
  let content = match app.view {
    View::MyKeys | View::PublicKeys => key_list::view(app),
    View::CreateKey => create_key::view(&app.create_form),
    View::Import => import::view(&app.import_form),
    View::Health => health::view(&app.health_report, app.health_loading),
    View::Encrypt => encrypt::view(&app.encrypt_form, &app.keys),
    View::Decrypt => decrypt::view(&app.decrypt_form),
    View::Sign => sign::view(&app.sign_form, &app.keys),
    View::Verify => verify::view(&app.sign_form),
  };

  let main: Element<Message> = match &app.status {
    Some((kind, msg)) => {
      let (bg, fg) = match kind {
        StatusKind::Error => (theme::ERROR_BG, theme::ERROR),
        StatusKind::Success => (theme::SUCCESS_BG, theme::SUCCESS),
      };
      let gen = app.status_generation;
      column![
        content,
        container(
          row![
            text(msg.as_str()).size(12),
            iced::widget::Space::new().width(Length::Fill),
            button(text("×").size(12))
              .on_press(Message::DismissStatus(gen))
              .padding([2, 8])
              .style(move |_: &iced::Theme, _| button::Style {
                background: None,
                text_color: fg,
                border: Border::default(),
                shadow: Shadow::default(),
                snap: false,
              }),
          ]
          .align_y(Alignment::Center),
        )
        .padding([6, 16])
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

  let main_el = container(main)
    .height(Length::Fill)
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::DETAIL_BG)),
      text_color: Some(theme::TEXT_STRONG),
      ..Default::default()
    });

  row![sidebar_el, main_el]
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
        snap: false,
      },
    )
  };

  let title_font = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
  };

  let section_label = |label: &'static str| {
    text(label)
      .size(10)
      .style(|_: &iced::Theme| iced::widget::text::Style {
        color: Some(theme::TEXT_MUTED),
      })
  };

  let sep = || {
    rule::horizontal(1).style(|_: &iced::Theme| rule::Style {
      color: theme::BORDER,
      radius: 0.0.into(),
      fill_mode: rule::FillMode::Full,
      snap: true,
    })
  };

  column![
    row![
      text("\u{f21b}").font(theme::ICONS).size(18),
      text("pgpilot").size(20).font(title_font),
    ]
    .spacing(8)
    .align_y(Alignment::Center),
    sep(),
    column![
      section_label("CLEFS"),
      nav_btn("\u{f084}", "Mes clefs", View::MyKeys),
      nav_btn("\u{f0c0}", "Clefs publiques", View::PublicKeys),
    ]
    .spacing(2),
    sep(),
    column![
      section_label("OPÉRATIONS"),
      nav_btn("\u{f023}", "Chiffrer", View::Encrypt),
      nav_btn("\u{f13e}", "Déchiffrer", View::Decrypt),
      nav_btn("\u{f14b}", "Signer", View::Sign),
      nav_btn("\u{f00c}", "Vérifier", View::Verify),
    ]
    .spacing(2),
    sep(),
    Space::new().height(Length::Fill),
    column![
      section_label("OUTILS"),
      nav_btn("\u{f093}", "Importer", View::Import),
      nav_btn("\u{f067}", "Créer une clef", View::CreateKey),
      nav_btn("\u{f132}", "Diagnostic", View::Health),
    ]
    .spacing(2),
  ]
  .spacing(8)
  .padding(12)
  .width(180)
  .height(Length::Fill)
  .into()
}
