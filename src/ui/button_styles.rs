#![allow(dead_code)]

use iced::{widget::button, Background, Border, Color, Shadow};

use crate::ui::theme;

pub fn primary() -> impl Fn(&iced::Theme, button::Status) -> button::Style {
  |_: &iced::Theme, status| button::Style {
    background: Some(Background::Color(match status {
      button::Status::Hovered | button::Status::Pressed => theme::ACCENT_HOVER,
      _ => theme::ACCENT,
    })),
    text_color: theme::TEXT_ON_ACCENT,
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
        button::Status::Hovered | button::Status::Pressed => theme::ACCENT_HOVER,
        _ => theme::ACCENT,
      }
    } else {
      theme::DISABLED_BG
    })),
    text_color: if enabled {
      theme::TEXT_ON_ACCENT
    } else {
      theme::TEXT_MUTED
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
      button::Status::Hovered | button::Status::Pressed => theme::ACCENT_SUBTLE,
      _ => Color::TRANSPARENT,
    })),
    text_color: theme::TEXT_STRONG,
    border: Border {
      color: theme::BORDER,
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
      button::Status::Hovered | button::Status::Pressed => theme::ACCENT_SUBTLE,
      _ => Color::TRANSPARENT,
    })),
    text_color: theme::ACCENT,
    border: Border {
      color: theme::ACCENT_BORDER,
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
      button::Status::Hovered | button::Status::Pressed => theme::DESTRUCTIVE_HOVER_BG,
      _ => Color::TRANSPARENT,
    })),
    text_color: theme::DESTRUCTIVE,
    border: Border {
      color: theme::DESTRUCTIVE,
      width: 1.0,
      radius: 6.0.into(),
    },
    shadow: Shadow::default(),
    snap: false,
  }
}
