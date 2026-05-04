#![allow(dead_code)]

use iced::{widget::button, Background, Border, Color, Shadow};

use crate::ui::theme;

pub fn primary() -> impl Fn(&iced::Theme, button::Status) -> button::Style {
  |_: &iced::Theme, status| button::Style {
    background: Some(Background::Color(match status {
      button::Status::Hovered | button::Status::Pressed => theme::accent_hover(),
      _ => theme::accent(),
    })),
    text_color: theme::text_on_accent(),
    border: Border {
      color: Color::TRANSPARENT,
      width: 0.0,
      radius: 6.0.into(),
    },
    shadow: Shadow::default(),
    snap: false,
  }
}

pub fn primary_toggle(enabled: bool) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
  move |_: &iced::Theme, status| button::Style {
    background: Some(Background::Color(if enabled {
      match status {
        button::Status::Hovered | button::Status::Pressed => theme::accent_hover(),
        _ => theme::accent(),
      }
    } else {
      theme::disabled_bg()
    })),
    text_color: if enabled {
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
  }
}

pub fn ghost_neutral() -> impl Fn(&iced::Theme, button::Status) -> button::Style {
  |_: &iced::Theme, status| button::Style {
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
  }
}

pub fn ghost_accent() -> impl Fn(&iced::Theme, button::Status) -> button::Style {
  |_: &iced::Theme, status| button::Style {
    background: Some(Background::Color(match status {
      button::Status::Hovered | button::Status::Pressed => theme::accent_subtle(),
      _ => Color::TRANSPARENT,
    })),
    text_color: theme::accent(),
    border: Border {
      color: theme::accent_border(),
      width: 1.0,
      radius: 6.0.into(),
    },
    shadow: Shadow::default(),
    snap: false,
  }
}

pub fn ghost_destructive() -> impl Fn(&iced::Theme, button::Status) -> button::Style {
  |_: &iced::Theme, status| button::Style {
    background: Some(Background::Color(match status {
      button::Status::Hovered | button::Status::Pressed => theme::destructive_hover_bg(),
      _ => Color::TRANSPARENT,
    })),
    text_color: theme::destructive(),
    border: Border {
      color: theme::destructive(),
      width: 1.0,
      radius: 6.0.into(),
    },
    shadow: Shadow::default(),
    snap: false,
  }
}
