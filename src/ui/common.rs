use iced::{
  widget::{button, row, scrollable, text},
  Alignment, Background, Border, Color, Element,
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
