use iced::{
  font,
  widget::{button, column, container, row, rule, scrollable, text},
  Alignment, Background, Border, Color, Element, Font, Length, Shadow,
};

use crate::app::{DecryptForm, Message};
use crate::gpg::DecryptStatus;
use crate::i18n::Strings;
use crate::ui::{common, theme};

pub fn view<'a>(form: &'a DecryptForm, s: &'static dyn Strings) -> Element<'a, Message> {
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

  let info_banner: Element<'_, Message> = container(
    row![
      text("\u{f05a}").font(theme::ICONS).size(14),
      text(
        "GPG utilisera automatiquement votre clef privée. \
         Si elle est protégée par un mot de passe, une fenêtre \
         s'ouvrira pour vous le demander."
      )
      .size(12),
    ]
    .spacing(8)
    .align_y(Alignment::Center),
  )
  .padding([8, 12])
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::accent_subtle())),
    border: Border {
      color: theme::accent_border(),
      width: 1.0,
      radius: 6.0.into(),
    },
    text_color: Some(theme::text_secondary()),
    ..Default::default()
  })
  .into();

  let has_no_key = form
    .files
    .iter()
    .any(|f| form.file_statuses.get(f) == Some(&DecryptStatus::NoKey));

  let file_items: Vec<Element<'_, Message>> = if form.files.is_empty() {
    vec![container(
      text("Glissez des fichiers .gpg ou .asc ici, ou utilisez le bouton ci-dessous.").size(13),
    )
    .padding([16, 0])
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      text_color: Some(theme::text_muted()),
      ..Default::default()
    })
    .into()]
  } else {
    form
      .files
      .iter()
      .enumerate()
      .map(|(i, path)| {
        let name = path
          .file_name()
          .and_then(|n| n.to_str())
          .unwrap_or("fichier")
          .to_string();

        let status = form.file_statuses.get(path);
        let (badge_icon, badge_color, badge_label) = match status {
          Some(DecryptStatus::CanDecrypt) => ("\u{f058}", theme::success(), "Clef disponible"),
          Some(DecryptStatus::NoKey) => ("\u{f057}", theme::error(), "Clef manquante"),
          Some(DecryptStatus::Checking) => ("\u{f110}", theme::text_muted(), "Vérification..."),
          _ => ("\u{f059}", theme::text_muted(), ""),
        };

        row![
          text(badge_icon)
            .font(theme::ICONS)
            .size(13)
            .color(badge_color),
          text(name).size(13).width(Length::Fill),
          text(badge_label).size(11).color(theme::text_muted()),
          button(text("\u{f1f8}").font(theme::ICONS).size(12))
            .on_press(Message::DecryptRemoveFile(i))
            .padding([3, 6])
            .style(|_: &iced::Theme, status| button::Style {
              background: Some(Background::Color(match status {
                button::Status::Hovered | button::Status::Pressed => theme::destructive_hover_bg(),
                _ => Color::TRANSPARENT,
              })),
              text_color: theme::error(),
              border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
              },
              shadow: Shadow::default(),
              snap: false,
            }),
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .padding([4, 6])
        .into()
      })
      .collect()
  };

  let add_files_btn = button(
    row![
      text("\u{f067}").font(theme::ICONS).size(12),
      text("Choisir des fichiers...").size(13),
    ]
    .spacing(6)
    .align_y(Alignment::Center),
  )
  .on_press(Message::DecryptPickFiles)
  .width(Length::Fill)
  .padding([8, 12])
  .style(|_: &iced::Theme, status| button::Style {
    background: Some(Background::Color(match status {
      button::Status::Hovered | button::Status::Pressed => theme::accent_subtle(),
      _ => Color::TRANSPARENT,
    })),
    text_color: theme::text_strong(),
    border: Border {
      color: theme::border(),
      width: 1.0,
      radius: 6.0.into(),
    },
    shadow: Shadow::default(),
    snap: false,
  });

  let mut files_section_children: Vec<Element<'_, Message>> = vec![
    container(text(s.decrypt_add_files()).size(12).font(bold))
      .style(|_: &iced::Theme| container::Style {
        text_color: Some(theme::text_secondary()),
        ..Default::default()
      })
      .into(),
    scrollable(column(file_items).spacing(2).padding([0, 4]))
      .height(220)
      .style(common::scroll_style)
      .into(),
    add_files_btn.into(),
  ];

  if has_no_key {
    let warning: Element<'_, Message> = container(
      row![
        text("\u{f071}")
          .font(theme::ICONS)
          .size(13)
          .color(theme::error()),
        text(
          "Certains fichiers ne peuvent pas être déchiffrés — vous ne possédez pas \
           la clef privée correspondante. Ces fichiers seront ignorés."
        )
        .size(12),
      ]
      .spacing(8)
      .align_y(Alignment::Center),
    )
    .padding([8, 12])
    .width(Length::Fill)
    .style(|_: &iced::Theme| container::Style {
      background: Some(Background::Color(theme::error_bg())),
      border: Border {
        color: theme::error(),
        width: 1.0,
        radius: 6.0.into(),
      },
      text_color: Some(theme::error()),
      ..Default::default()
    })
    .into();
    files_section_children.push(warning);
  }

  let files_section: Element<'_, Message> = column(files_section_children).spacing(8).into();

  let all_no_key = !form.files.is_empty()
    && form
      .files
      .iter()
      .all(|f| form.file_statuses.get(f) == Some(&DecryptStatus::NoKey));
  let can_decrypt = !form.files.is_empty() && !form.decrypting && !all_no_key;

  let n = form.files.len();
  let decrypt_label = if form.decrypting {
    s.decrypt_in_progress().to_string()
  } else if n == 1 {
    format!("{} 1 fichier", s.btn_decrypt())
  } else {
    format!("{} {n} fichier(s)", s.btn_decrypt())
  };

  let decrypt_btn = {
    let style_enabled = can_decrypt;
    let btn = button(text(decrypt_label).size(13)).padding([8, 16]).style(
      move |_: &iced::Theme, status| button::Style {
        background: Some(Background::Color(if style_enabled {
          match status {
            button::Status::Hovered | button::Status::Pressed => theme::accent_hover(),
            _ => theme::accent(),
          }
        } else {
          theme::disabled_bg()
        })),
        text_color: if style_enabled {
          theme::text_on_accent()
        } else {
          theme::text_muted()
        },
        border: Border {
          color: Color::TRANSPARENT,
          width: 0.0,
          radius: 6.0.into(),
        },
        shadow: Shadow::default(),
        snap: false,
      },
    );
    if can_decrypt {
      btn.on_press(Message::DecryptExecute)
    } else {
      btn
    }
  };

  let action_bar: Element<'_, Message> =
    row![iced::widget::Space::new().width(Length::Fill), decrypt_btn,]
      .align_y(Alignment::Center)
      .into();

  let card = container(
    column![
      column![
        row![
          text("\u{f13e}").font(theme::ICONS).size(20),
          text(s.decrypt_title()).size(22).font(bold),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        container(text("Déchiffrez des fichiers chiffrés avec GPG.").size(13)).style(
          |_: &iced::Theme| container::Style {
            text_color: Some(theme::text_secondary()),
            ..Default::default()
          }
        ),
      ]
      .spacing(6),
      separator(),
      info_banner,
      separator(),
      files_section,
      separator(),
      action_bar,
    ]
    .spacing(16),
  )
  .padding(32)
  .width(600)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::card_bg())),
    border: Border {
      color: theme::border(),
      width: 1.0,
      radius: 12.0.into(),
    },
    text_color: Some(theme::text_strong()),
    ..Default::default()
  });

  container(
    scrollable(
      container(card)
        .center_x(Length::Fill)
        .padding([24, 0])
        .width(Length::Fill),
    )
    .height(Length::Fill)
    .width(Length::Fill)
    .style(common::scroll_style),
  )
  .height(Length::Fill)
  .width(Length::Fill)
  .style(|_: &iced::Theme| container::Style {
    background: Some(Background::Color(theme::sidebar_bg())),
    ..Default::default()
  })
  .into()
}
