use iced::{
  font,
  widget::{button, checkbox, column, container, pick_list, row, rule, text, text_input},
  Background, Border, Color, Element, Font, Length,
};

use crate::app::{CreateKeyForm, Message};
use crate::gpg::KeyExpiry;
use crate::i18n::Strings;
use crate::ui::{common, theme};

pub fn view<'a>(form: &'a CreateKeyForm, s: &'static dyn Strings) -> Element<'a, Message> {
  let bold = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
  };

  let separator = || {
    rule::horizontal(1).style(|_: &iced::Theme| rule::Style {
      color: theme::border(),
      radius: 0.0.into(),
      fill_mode: rule::FillMode::Full,
      snap: false,
    })
  };

  let expiry_list = pick_list(
    vec![
      KeyExpiry::OneYear,
      KeyExpiry::TwoYears,
      KeyExpiry::FiveYears,
    ],
    Some(form.subkey_expiry.clone()),
    Message::CreateKeySubkeyExpiryChanged,
  )
  .width(Length::Fill)
  .style(|_: &iced::Theme, status| {
    let border = match status {
      pick_list::Status::Opened { .. } => theme::accent(),
      pick_list::Status::Hovered => theme::accent_border(),
      _ => theme::border(),
    };
    pick_list::Style {
      text_color: theme::text_strong(),
      placeholder_color: theme::text_muted(),
      handle_color: theme::text_muted(),
      background: Background::Color(theme::header_bg()),
      border: Border {
        color: border,
        width: 1.0,
        radius: 6.0.into(),
      },
    }
  })
  .menu_style(|_: &iced::Theme| iced::overlay::menu::Style {
    text_color: theme::text_strong(),
    background: Background::Color(theme::card_bg()),
    border: Border {
      color: theme::border(),
      width: 1.0,
      radius: 6.0.into(),
    },
    selected_text_color: theme::text_on_accent(),
    selected_background: Background::Color(theme::accent()),
    shadow: iced::Shadow::default(),
  });

  let label = if form.submitting {
    s.create_key_generating()
  } else {
    s.btn_create()
  };
  let can_submit = !form.name.is_empty() && !form.email.is_empty() && !form.submitting;

  let submit_btn = {
    let btn = button(text(label).size(13)).style(move |_: &iced::Theme, status: button::Status| {
      button::Style {
        background: Some(Background::Color(if can_submit {
          match status {
            button::Status::Hovered | button::Status::Pressed => theme::accent_hover(),
            _ => theme::accent(),
          }
        } else {
          theme::disabled_bg()
        })),
        text_color: if can_submit {
          theme::text_on_accent()
        } else {
          theme::text_muted()
        },
        border: Border {
          color: Color::TRANSPARENT,
          width: 0.0,
          radius: 6.0.into(),
        },
        shadow: Default::default(),
        snap: false,
      }
    });
    if can_submit {
      btn.on_press(Message::CreateKeySubmit)
    } else {
      btn
    }
  };

  let cancel_btn = button(text(s.btn_cancel()).size(13))
    .on_press(Message::NavBack)
    .style(|_: &iced::Theme, status: button::Status| button::Style {
      background: Some(Background::Color(match status {
        button::Status::Hovered | button::Status::Pressed => theme::header_bg(),
        _ => Color::TRANSPARENT,
      })),
      text_color: theme::text_secondary(),
      border: Border {
        color: theme::border(),
        width: 1.0,
        radius: 6.0.into(),
      },
      shadow: Default::default(),
      snap: false,
    });

  let hint = |s: &'static str| {
    container(text(s).size(11)).style(|_: &iced::Theme| container::Style {
      text_color: Some(theme::text_muted()),
      ..Default::default()
    })
  };

  let card_content = column![
    column![
      text(theme::flavor(
        s.create_key_title(),
        "Forge a Weapon for the People"
      ))
      .size(22)
      .font(theme::flavor_title_font()),
      container(text(s.create_key_subtitle()).size(13),).style(|_: &iced::Theme| {
        container::Style {
          text_color: Some(theme::text_secondary()),
          ..Default::default()
        }
      }),
    ]
    .spacing(6),
    separator(),
    column![
      text(s.create_key_section_identity()).size(12).font(bold),
      column![
        text(s.create_key_field_name()).size(12),
        text_input("Alice Martin", &form.name)
          .on_input(Message::CreateKeyNameChanged)
          .size(14)
          .width(Length::Fill)
          .style(|_: &iced::Theme, status| {
            let border = match status {
              text_input::Status::Focused { .. } => theme::accent(),
              text_input::Status::Hovered => theme::accent_border(),
              _ => theme::border(),
            };
            text_input::Style {
              background: Background::Color(theme::header_bg()),
              border: Border {
                color: border,
                width: 1.0,
                radius: 6.0.into(),
              },
              icon: theme::text_muted(),
              placeholder: theme::text_muted(),
              value: theme::text_strong(),
              selection: theme::accent_subtle(),
            }
          }),
      ]
      .spacing(4),
      column![
        text(s.create_key_field_email()).size(12),
        text_input("alice@example.com", &form.email)
          .on_input(Message::CreateKeyEmailChanged)
          .size(14)
          .width(Length::Fill)
          .style(|_: &iced::Theme, status| {
            let border = match status {
              text_input::Status::Focused { .. } => theme::accent(),
              text_input::Status::Hovered => theme::accent_border(),
              _ => theme::border(),
            };
            text_input::Style {
              background: Background::Color(theme::header_bg()),
              border: Border {
                color: border,
                width: 1.0,
                radius: 6.0.into(),
              },
              icon: theme::text_muted(),
              placeholder: theme::text_muted(),
              value: theme::text_strong(),
              selection: theme::accent_subtle(),
            }
          }),
      ]
      .spacing(4),
    ]
    .spacing(10),
    separator(),
    column![
      text(s.create_key_section_subkeys()).size(12).font(bold),
      column![
        text(s.create_key_section_expiration()).size(12),
        expiry_list,
        hint(s.create_key_hint_expiry()),
      ]
      .spacing(6),
      column![
        checkbox(form.include_auth)
          .label(s.create_key_include_ssh())
          .on_toggle(Message::CreateKeyIncludeAuthToggled)
          .text_size(13)
          .size(16)
          .style(|_: &iced::Theme, status| {
            let (is_checked, is_hovered) = match status {
              checkbox::Status::Active { is_checked } => (is_checked, false),
              checkbox::Status::Hovered { is_checked } => (is_checked, true),
              checkbox::Status::Disabled { is_checked } => (is_checked, false),
            };
            checkbox::Style {
              background: Background::Color(if is_checked {
                theme::accent()
              } else {
                theme::header_bg()
              }),
              icon_color: theme::text_on_accent(),
              border: Border {
                color: if is_checked {
                  theme::accent()
                } else if is_hovered {
                  theme::accent_border()
                } else {
                  theme::border()
                },
                width: 1.0,
                radius: 3.0.into(),
              },
              text_color: Some(theme::text_strong()),
            }
          }),
        hint(s.create_key_hint_ssh()),
      ]
      .spacing(6),
    ]
    .spacing(14),
    separator(),
    container(
      column![
        text(s.create_key_about_master()).size(12).font(bold),
        hint(s.create_key_hint_master()),
      ]
      .spacing(6),
    )
    .padding([4, 0]),
    separator(),
    row![cancel_btn, submit_btn].spacing(8),
  ]
  .spacing(20);

  common::page_layout(common::card_medium(card_content))
}
