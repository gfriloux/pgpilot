use iced::{
  widget::{button, row, text},
  Alignment, Element,
};

use crate::app::Message;
use crate::ui::{button_styles, theme};

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
