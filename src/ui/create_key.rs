use iced::{
  font,
  widget::{
    button, checkbox, column, container, horizontal_rule, pick_list, row, rule, text, text_input,
  },
  Background, Border, Color, Element, Font, Length,
};

use crate::app::{CreateKeyForm, Message, View};
use crate::gpg::KeyExpiry;
use crate::ui::theme;

pub fn view(form: &CreateKeyForm) -> Element<'_, Message> {
  let bold = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
  };

  let separator = || {
    horizontal_rule(1).style(|_: &iced::Theme| rule::Style {
      color: theme::BORDER,
      width: 1,
      radius: 0.0.into(),
      fill_mode: rule::FillMode::Full,
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
      pick_list::Status::Opened => theme::ACCENT,
      pick_list::Status::Hovered => theme::ACCENT_BORDER,
      _ => theme::BORDER,
    };
    pick_list::Style {
      text_color: theme::TEXT_STRONG,
      placeholder_color: theme::TEXT_MUTED,
      handle_color: theme::TEXT_MUTED,
      background: Background::Color(theme::HEADER_BG),
      border: Border {
        color: border,
        width: 1.0,
        radius: 6.0.into(),
      },
    }
  })
  .menu_style(|_: &iced::Theme| iced::overlay::menu::Style {
    text_color: theme::TEXT_STRONG,
    background: Background::Color(theme::CARD_BG),
    border: Border {
      color: theme::BORDER,
      width: 1.0,
      radius: 6.0.into(),
    },
    selected_text_color: theme::TEXT_ON_ACCENT,
    selected_background: Background::Color(theme::ACCENT),
  });

  let label = if form.submitting {
    "Génération..."
  } else {
    "Créer la clef"
  };
  let can_submit = !form.name.is_empty() && !form.email.is_empty() && !form.submitting;

  let submit_btn = {
    let btn = button(text(label).size(13)).style(move |_: &iced::Theme, status: button::Status| {
      button::Style {
        background: Some(Background::Color(if can_submit {
          match status {
            button::Status::Hovered | button::Status::Pressed => theme::ACCENT_HOVER,
            _ => theme::ACCENT,
          }
        } else {
          theme::DISABLED_BG
        })),
        text_color: if can_submit {
          theme::TEXT_ON_ACCENT
        } else {
          theme::TEXT_MUTED
        },
        border: Border {
          color: Color::TRANSPARENT,
          width: 0.0,
          radius: 6.0.into(),
        },
        shadow: Default::default(),
      }
    });
    if can_submit {
      btn.on_press(Message::CreateKeySubmit)
    } else {
      btn
    }
  };

  let cancel_btn = button(text("Annuler").size(13))
    .on_press(Message::NavChanged(View::MyKeys))
    .style(|_: &iced::Theme, status: button::Status| button::Style {
      background: Some(Background::Color(match status {
        button::Status::Hovered | button::Status::Pressed => theme::HEADER_BG,
        _ => Color::TRANSPARENT,
      })),
      text_color: theme::TEXT_SECONDARY,
      border: Border {
        color: theme::BORDER,
        width: 1.0,
        radius: 6.0.into(),
      },
      shadow: Default::default(),
    });

  let hint = |s: &'static str| {
    container(text(s).size(11)).style(|_: &iced::Theme| container::Style {
      text_color: Some(theme::TEXT_MUTED),
      ..Default::default()
    })
  };

  let card = container(
    column![
      column![
        text("Nouvelle clef PGP").size(22).font(bold),
        container(text("Génère une clef maître et ses sous-clefs dédiées.").size(13),).style(
          |_: &iced::Theme| container::Style {
            text_color: Some(theme::TEXT_SECONDARY),
            ..Default::default()
          }
        ),
      ]
      .spacing(6),
      separator(),
      column![
        text("Identité").size(12).font(bold),
        column![
          text("Nom").size(12),
          text_input("Alice Martin", &form.name)
            .on_input(Message::CreateKeyNameChanged)
            .size(14)
            .width(Length::Fill)
            .style(|_: &iced::Theme, status| {
              let border = match status {
                text_input::Status::Focused => theme::ACCENT,
                text_input::Status::Hovered => theme::ACCENT_BORDER,
                _ => theme::BORDER,
              };
              text_input::Style {
                background: Background::Color(theme::HEADER_BG),
                border: Border {
                  color: border,
                  width: 1.0,
                  radius: 6.0.into(),
                },
                icon: theme::TEXT_MUTED,
                placeholder: theme::TEXT_MUTED,
                value: theme::TEXT_STRONG,
                selection: theme::ACCENT_SUBTLE,
              }
            }),
        ]
        .spacing(4),
        column![
          text("Email").size(12),
          text_input("alice@example.com", &form.email)
            .on_input(Message::CreateKeyEmailChanged)
            .size(14)
            .width(Length::Fill)
            .style(|_: &iced::Theme, status| {
              let border = match status {
                text_input::Status::Focused => theme::ACCENT,
                text_input::Status::Hovered => theme::ACCENT_BORDER,
                _ => theme::BORDER,
              };
              text_input::Style {
                background: Background::Color(theme::HEADER_BG),
                border: Border {
                  color: border,
                  width: 1.0,
                  radius: 6.0.into(),
                },
                icon: theme::TEXT_MUTED,
                placeholder: theme::TEXT_MUTED,
                value: theme::TEXT_STRONG,
                selection: theme::ACCENT_SUBTLE,
              }
            }),
        ]
        .spacing(4),
      ]
      .spacing(10),
      separator(),
      column![
        text("Sous-clefs").size(12).font(bold),
        column![
          text("Expiration").size(12),
          expiry_list,
          hint(
            "Les sous-clefs expirent automatiquement. \
             Une courte durée limite les dégâts en cas de compromission \
             — vous pourrez les renouveler avant échéance.",
          ),
        ]
        .spacing(6),
        column![
          checkbox("Inclure une clef d'authentification SSH", form.include_auth,)
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
                  theme::ACCENT
                } else {
                  theme::HEADER_BG
                }),
                icon_color: theme::TEXT_ON_ACCENT,
                border: Border {
                  color: if is_checked {
                    theme::ACCENT
                  } else if is_hovered {
                    theme::ACCENT_BORDER
                  } else {
                    theme::BORDER
                  },
                  width: 1.0,
                  radius: 3.0.into(),
                },
                text_color: Some(theme::TEXT_STRONG),
              }
            }),
          hint(
            "Permet de vous authentifier sur des serveurs distants sans mot de passe, \
             en utilisant votre clef PGP comme clef SSH.",
          ),
        ]
        .spacing(6),
      ]
      .spacing(14),
      separator(),
      container(
        column![
          text("À propos de la clef maître").size(12).font(bold),
          hint(
            "La clef maître définit votre identité PGP à long terme — elle ne sert qu'à \
             certifier vos sous-clefs. Elle n'expire jamais. \
             Conservez-la hors ligne avec son certificat de révocation.",
          ),
        ]
        .spacing(6),
      )
      .padding([4, 0]),
      separator(),
      row![cancel_btn, submit_btn].spacing(8),
    ]
    .spacing(20),
  )
  .padding(32)
  .width(520)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::CARD_BG)),
    border: Border {
      color: theme::BORDER,
      width: 1.0,
      radius: 12.0.into(),
    },
    text_color: Some(theme::TEXT_STRONG),
    ..Default::default()
  });

  container(card)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::SIDEBAR_BG)),
      ..Default::default()
    })
    .into()
}
