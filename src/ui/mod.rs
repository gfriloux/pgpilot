pub mod button_styles;
pub mod common;
pub mod create_key;
pub mod decrypt;
pub mod encrypt;
pub mod health;
pub mod import;
pub mod key_detail;
pub mod key_list;
pub mod settings;
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
    View::CreateKey => create_key::view(&app.create_form, app.strings),
    View::Import => import::view(&app.import_form, app.strings),
    View::Health => health::view(&app.health_report, app.health_loading, app.strings),
    View::Encrypt => encrypt::view(&app.encrypt_form, &app.keys, app.strings),
    View::Decrypt => decrypt::view(&app.decrypt_form, app.strings),
    View::Sign => sign::view(&app.sign_form, &app.keys, app.strings),
    View::Verify => verify::view(&app.sign_form, app.strings),
    View::Settings => settings::view(app),
  };

  let main: Element<Message> = match &app.status {
    Some((kind, msg)) => {
      let (bg, fg) = match kind {
        StatusKind::Error => (theme::error_bg(), theme::error()),
        StatusKind::Success => (theme::success_bg(), theme::success()),
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
              .style(move |_: &iced::Theme, status| button::Style {
                background: Some(Background::Color(match status {
                  button::Status::Hovered | button::Status::Pressed => Color {
                    a: 0.15,
                    ..Color::WHITE
                  },
                  _ => Color::TRANSPARENT,
                })),
                text_color: fg,
                border: Border {
                  color: Color::TRANSPARENT,
                  width: 0.0,
                  radius: 4.0.into(),
                },
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
      background: Some(Background::Color(theme::sidebar_bg())),
      text_color: Some(theme::sidebar_text()),
      ..Default::default()
    });

  let main_el = container(main)
    .height(Length::Fill)
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::detail_bg())),
      text_color: Some(theme::text_strong()),
      ..Default::default()
    });

  row![sidebar_el, main_el]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn sidebar(app: &App) -> Element<'_, Message> {
  let s = app.strings;
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
          Some(Background::Color(theme::accent()))
        } else {
          match status {
            button::Status::Hovered | button::Status::Pressed => {
              Some(Background::Color(theme::sidebar_hover_bg()))
            }
            _ => None,
          }
        },
        text_color: if active {
          theme::text_on_accent()
        } else {
          theme::sidebar_text()
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
        color: Some(theme::text_muted()),
      })
  };

  let sep = || {
    rule::horizontal(1).style(|_: &iced::Theme| rule::Style {
      color: theme::border(),
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
      section_label(s.sidebar_section_keys()),
      nav_btn("\u{f084}", s.nav_my_keys(), View::MyKeys),
      nav_btn("\u{f0c0}", s.nav_public_keys(), View::PublicKeys),
    ]
    .spacing(2),
    sep(),
    column![
      section_label(s.sidebar_section_operations()),
      nav_btn("\u{f023}", s.nav_encrypt(), View::Encrypt),
      nav_btn("\u{f13e}", s.nav_decrypt(), View::Decrypt),
      nav_btn("\u{f14b}", s.nav_sign(), View::Sign),
      nav_btn("\u{f00c}", s.nav_verify(), View::Verify),
    ]
    .spacing(2),
    sep(),
    Space::new().height(Length::Fill),
    column![
      section_label(s.sidebar_section_tools()),
      nav_btn("\u{f093}", s.nav_import(), View::Import),
      nav_btn("\u{f067}", s.nav_create_key(), View::CreateKey),
      nav_btn("\u{f132}", s.nav_health(), View::Health),
      nav_btn("\u{f013}", s.nav_settings(), View::Settings),
    ]
    .spacing(2),
  ]
  .spacing(8)
  .padding(12)
  .width(180)
  .height(Length::Fill)
  .into()
}
