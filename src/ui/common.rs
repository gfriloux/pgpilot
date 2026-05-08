use iced::{
  widget::{button, column, container, image, radio, row, rule, scrollable, text, Space},
  Alignment, Background, Border, Color, Element, Length,
};

use crate::app::Message;
use crate::ui::{button_styles, theme};

/// Scrollbar style cohérente avec le thème actif (Catppuccin ou USSR).
///
/// Active  : piste invisible, poignée accent à 25 % d'opacité.
/// Hovered : piste légèrement visible, poignée à 55 %.
/// Dragged : poignée accent_hover opaque.
pub fn scroll_style(_theme: &iced::Theme, status: scrollable::Status) -> scrollable::Style {
  let (track_alpha, scroller_alpha, scroller_color) = match status {
    scrollable::Status::Active { .. } => (0.0_f32, 0.25_f32, theme::accent()),
    scrollable::Status::Hovered { .. } => (0.08_f32, 0.55_f32, theme::accent()),
    scrollable::Status::Dragged { .. } => (0.12_f32, 1.0_f32, theme::accent_hover()),
  };

  let accent = theme::accent();
  let make_rail = || scrollable::Rail {
    background: Some(Background::Color(Color {
      a: track_alpha,
      ..accent
    })),
    border: Border {
      radius: 3.0.into(),
      ..Default::default()
    },
    scroller: scrollable::Scroller {
      background: Background::Color(Color {
        a: scroller_alpha,
        ..scroller_color
      }),
      border: Border {
        radius: 3.0.into(),
        ..Default::default()
      },
    },
  };

  scrollable::Style {
    container: iced::widget::container::Style::default(),
    vertical_rail: make_rail(),
    horizontal_rail: make_rail(),
    gap: None,
    auto_scroll: scrollable::AutoScroll {
      background: Background::Color(Color::TRANSPARENT),
      border: Border::default(),
      shadow: iced::Shadow::default(),
      icon: Color::TRANSPARENT,
    },
  }
}

/// Style function for radio buttons — adapts to the active theme colours.
///
/// Active border and dot use `theme::accent()`. Hovered uses `theme::accent_hover()`.
/// Background switches to `theme::accent_subtle()` on hover when unselected.
pub fn radio_style(_theme: &iced::Theme, status: radio::Status) -> radio::Style {
  let is_selected = match status {
    radio::Status::Active { is_selected } => is_selected,
    radio::Status::Hovered { is_selected } => is_selected,
  };
  let is_hovered = matches!(status, radio::Status::Hovered { .. });

  let border_color = if is_selected {
    theme::accent()
  } else if is_hovered {
    theme::accent_hover()
  } else {
    theme::border()
  };

  let dot_color = if is_selected {
    theme::accent()
  } else {
    Color::TRANSPARENT
  };

  let bg = if is_hovered && !is_selected {
    theme::accent_subtle()
  } else {
    theme::detail_bg()
  };

  radio::Style {
    background: Background::Color(bg),
    dot_color,
    border_width: 2.0,
    border_color,
    text_color: None,
  }
}

pub fn pick_btn<'a>(
  icon: &'static str,
  label: &'static str,
  on_press: Message,
) -> Element<'a, Message> {
  button(
    row![text(icon).font(theme::ICONS).size(12), text(label).size(13),]
      .spacing(6)
      .align_y(Alignment::Center),
  )
  .on_press(on_press)
  .padding([8, 12])
  .style(button_styles::ghost_neutral())
  .into()
}

pub fn action_btn<'a>(
  label: &'static str,
  enabled: bool,
  on_press: Message,
) -> Element<'a, Message> {
  let btn = button(text(label).size(13))
    .padding([8, 16])
    .style(button_styles::primary_toggle(enabled));
  if enabled {
    btn.on_press(on_press).into()
  } else {
    btn.into()
  }
}

/// Wraps a card element in the standard full-page scrollable layout.
///
/// The outer container fills the window with `sidebar_bg()`. The inner
/// scrollable centres the card horizontally with 24px vertical padding.
/// Never use `center_y` — it looks wrong on tall windows.
pub fn page_layout<'a, M: 'a>(card: impl Into<Element<'a, M>>) -> Element<'a, M> {
  container(
    scrollable(
      container(card)
        .center_x(Length::Fill)
        .padding([24, 0])
        .width(Length::Fill),
    )
    .style(scroll_style)
    .height(Length::Fill)
    .width(Length::Fill),
  )
  .height(Length::Fill)
  .width(Length::Fill)
  .style(|_| container::Style {
    background: Some(Background::Color(theme::sidebar_bg())),
    ..Default::default()
  })
  .into()
}

/// Card with a fixed 700px width — for extended forms (e.g. Import, CreateKey, Health).
pub fn card_medium<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
  container(content)
    .padding(iced::Padding {
      top: 32.0,
      right: 32.0,
      bottom: 32.0,
      left: 32.0,
    })
    .width(Length::Fixed(700.0))
    .style(card_style)
    .into()
}

/// Card that fills up to 700px — for complex views (e.g. Encrypt, Sign, Verify).
pub fn card_wide<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
  container(content)
    .padding(iced::Padding {
      top: 32.0,
      right: 32.0,
      bottom: 32.0,
      left: 32.0,
    })
    .width(Length::Fill)
    .max_width(700)
    .style(card_style)
    .into()
}

fn card_style(_: &iced::Theme) -> container::Style {
  container::Style {
    background: Some(Background::Color(theme::card_bg())),
    border: Border {
      radius: 12.0.into(),
      ..Default::default()
    },
    ..Default::default()
  }
}

/// Medium card (700px) with a propaganda banner flush at the bottom edge (full card width).
/// The banner PNG has pre-baked transparent bottom corners (radius 13 image-px ≈ 12 screen-px)
/// so the card's background shows through, creating natural rounded corners.
/// In Catppuccin, falls back to `card_medium` — no banner, standard padding.
pub fn card_medium_with_banner<'a, M: 'a>(
  content: impl Into<Element<'a, M>>,
  handle: image::Handle,
) -> Element<'a, M> {
  if !matches!(theme::active(), theme::ThemeVariant::Ussr) {
    return card_medium(content);
  }
  let inner = container(content)
    .padding(iced::Padding {
      top: 32.0,
      right: 32.0,
      bottom: 32.0,
      left: 32.0,
    })
    .width(Length::Fill);
  container(column![inner, image(handle).width(Length::Fill)].spacing(0))
    .width(Length::Fixed(700.0))
    .style(card_style)
    .into()
}

/// Wide card (max 700px) with a propaganda banner flush at the bottom edge (full card width).
/// In Catppuccin, falls back to `card_wide`.
pub fn card_wide_with_banner<'a, M: 'a>(
  content: impl Into<Element<'a, M>>,
  handle: image::Handle,
) -> Element<'a, M> {
  if !matches!(theme::active(), theme::ThemeVariant::Ussr) {
    return card_wide(content);
  }
  let inner = container(content)
    .padding(iced::Padding {
      top: 32.0,
      right: 32.0,
      bottom: 32.0,
      left: 32.0,
    })
    .width(Length::Fill);
  container(column![inner, image(handle).width(Length::Fill)].spacing(0))
    .width(Length::Fill)
    .max_width(700)
    .style(card_style)
    .into()
}

/// Séparateur ─────★───── avec étoile rouge FA4 centrée.
/// Les règles horizontales remplissent l'espace disponible de chaque côté.
pub fn star_separator<'a, M: 'a>() -> Element<'a, M> {
  let rule_style = |_: &iced::Theme| rule::Style {
    color: theme::border(),
    radius: 0.0.into(),
    fill_mode: rule::FillMode::Full,
    snap: false,
  };
  row![
    container(rule::horizontal(1).style(rule_style)).width(Length::Fill),
    Space::new().width(8),
    text("\u{f005}")
      .font(theme::ICONS)
      .size(10)
      .color(theme::accent()),
    Space::new().width(8),
    container(rule::horizontal(1).style(rule_style)).width(Length::Fill),
  ]
  .align_y(Alignment::Center)
  .into()
}

/// Bannière pleine largeur pour les panels liste/rooms — visible uniquement en thème USSR.
pub fn panel_banner<'a, M: 'a>(handle: image::Handle, bg: Color) -> Element<'a, M> {
  if !matches!(theme::active(), theme::ThemeVariant::Ussr) {
    return Space::new().into();
  }
  container(image(handle).width(Length::Fill))
    .width(Length::Fill)
    .style(move |_| container::Style {
      background: Some(Background::Color(bg)),
      ..Default::default()
    })
    .into()
}
