use iced::{
  widget::{button, container, radio, row, scrollable, text},
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

/// Card with a fixed 480px width — for simple forms (e.g. Settings).
pub fn card_narrow<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
  container(content)
    .padding(32)
    .width(Length::Fixed(480.0))
    .style(|_| container::Style {
      background: Some(Background::Color(theme::card_bg())),
      border: Border {
        radius: 12.0.into(),
        ..Default::default()
      },
      ..Default::default()
    })
    .into()
}

/// Card with a fixed 560px width — for extended forms (e.g. Import, CreateKey, Health).
pub fn card_medium<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
  container(content)
    .padding(32)
    .width(Length::Fixed(560.0))
    .style(|_| container::Style {
      background: Some(Background::Color(theme::card_bg())),
      border: Border {
        radius: 12.0.into(),
        ..Default::default()
      },
      ..Default::default()
    })
    .into()
}

/// Card that fills up to 760px — for complex views (e.g. Encrypt, Sign, Verify).
pub fn card_wide<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
  container(content)
    .padding(32)
    .width(Length::Fill)
    .max_width(760)
    .style(|_| container::Style {
      background: Some(Background::Color(theme::card_bg())),
      border: Border {
        radius: 12.0.into(),
        ..Default::default()
      },
      ..Default::default()
    })
    .into()
}
