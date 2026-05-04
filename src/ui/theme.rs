use iced::{font, Color, Font};
use std::cell::Cell;

// Re-export ThemeVariant from config so UI code can use theme::ThemeVariant.
pub use crate::config::ThemeVariant;

/// Icon font — FA4 range only (\u{f000}–\u{f2e0}); codepoints above that range render as blank.
pub const ICONS: Font = Font::with_name("Symbols Nerd Font Mono");
/// Monospace font for hex identifiers (fingerprints, key IDs).
/// Uses the iced default font — do not point this to ICONS (symbol-only font,
/// lacks ASCII characters and will render fingerprints as blank boxes).
pub const MONO: Font = Font::DEFAULT;
/// Bebas Neue — used for sidebar/navigation labels in the USSR theme (all-caps condensed).
pub const USSR_NAV_FONT: Font = Font::with_name("Bebas Neue");
/// Russo One — used for headings and body text in the USSR theme.
pub const USSR_HEADING_FONT: Font = Font::with_name("Russo One");

// ---------------------------------------------------------------------------
// Active theme — thread-local so UI rendering always reads current selection.
// ---------------------------------------------------------------------------

thread_local! {
  static ACTIVE: Cell<ThemeVariant> = const { Cell::new(ThemeVariant::Catppuccin) };
}

pub fn set_active(v: ThemeVariant) {
  ACTIVE.with(|c| c.set(v));
}

pub(crate) fn active() -> ThemeVariant {
  ACTIVE.with(|c| c.get())
}

// ---------------------------------------------------------------------------
// Catppuccin Frappé — original constants (kept for backwards compatibility
// in places that still use them, e.g. the status bar in ui/mod.rs which
// reads them inside closures that don't call our functions).
// ---------------------------------------------------------------------------

// Sidebar (Crust)
pub const SIDEBAR_BG: Color = Color {
  r: 0.137,
  g: 0.149,
  b: 0.204,
  a: 1.0,
};
pub const SIDEBAR_TEXT: Color = Color {
  r: 0.776,
  g: 0.816,
  b: 0.961,
  a: 1.0,
};
pub const SIDEBAR_HOVER_BG: Color = Color {
  r: 1.0,
  g: 1.0,
  b: 1.0,
  a: 0.07,
};

// Content text
pub const TEXT_STRONG: Color = Color {
  r: 0.776,
  g: 0.816,
  b: 0.961,
  a: 1.0,
};
pub const TEXT_SECONDARY: Color = Color {
  r: 0.710,
  g: 0.749,
  b: 0.886,
  a: 1.0,
};
pub const TEXT_MUTED: Color = Color {
  r: 0.647,
  g: 0.678,
  b: 0.808,
  a: 1.0,
};
pub const TEXT_HEADER: Color = Color {
  r: 0.514,
  g: 0.545,
  b: 0.655,
  a: 1.0,
};

// Accent (Mauve #ca9ee6)
pub const ACCENT: Color = Color {
  r: 0.792,
  g: 0.620,
  b: 0.902,
  a: 1.0,
};
pub const ACCENT_HOVER: Color = Color {
  r: 0.675,
  g: 0.525,
  b: 0.769,
  a: 1.0,
};
pub const ACCENT_SUBTLE: Color = Color {
  r: 0.792,
  g: 0.620,
  b: 0.902,
  a: 0.12,
};
pub const ACCENT_BORDER: Color = Color {
  r: 0.792,
  g: 0.620,
  b: 0.902,
  a: 0.35,
};
pub const TEXT_ON_ACCENT: Color = Color {
  r: 0.137,
  g: 0.149,
  b: 0.204,
  a: 1.0,
};

// State
pub const SUCCESS: Color = Color {
  r: 0.651,
  g: 0.820,
  b: 0.537,
  a: 1.0,
};
pub const SUCCESS_BG: Color = Color {
  r: 0.651,
  g: 0.820,
  b: 0.537,
  a: 0.12,
};
pub const SUCCESS_HOVER: Color = Color {
  r: 0.521,
  g: 0.656,
  b: 0.430,
  a: 1.0,
};
pub const ERROR: Color = Color {
  r: 0.906,
  g: 0.510,
  b: 0.518,
  a: 1.0,
};
pub const ERROR_BG: Color = Color {
  r: 0.906,
  g: 0.510,
  b: 0.518,
  a: 0.12,
};

// Borders & surfaces
pub const BORDER: Color = Color {
  r: 0.3176,
  g: 0.341,
  b: 0.427,
  a: 1.0,
};
pub const HEADER_BG: Color = Color {
  r: 0.255,
  g: 0.271,
  b: 0.349,
  a: 1.0,
};
pub const DETAIL_BG: Color = Color {
  r: 0.161,
  g: 0.173,
  b: 0.235,
  a: 1.0,
};
pub const CARD_BG: Color = Color {
  r: 0.188,
  g: 0.204,
  b: 0.275,
  a: 1.0,
};

// Destructive (Red #e78284)
pub const DESTRUCTIVE: Color = Color {
  r: 0.85,
  g: 0.40,
  b: 0.41,
  a: 1.0,
};
pub const DESTRUCTIVE_HOVER_BG: Color = Color {
  r: 0.906,
  g: 0.510,
  b: 0.518,
  a: 0.15,
};

// Disabled
pub const DISABLED_BG: Color = Color {
  r: 0.3176,
  g: 0.341,
  b: 0.427,
  a: 1.0,
};

// Warning (Peach #ef9f76)
pub const PEACH: Color = Color {
  r: 0.937,
  g: 0.624,
  b: 0.463,
  a: 1.0,
};
pub const WARNING_BG: Color = Color {
  r: 0.937,
  g: 0.624,
  b: 0.463,
  a: 0.12,
};

// ---------------------------------------------------------------------------
// USSR palette constants
// ---------------------------------------------------------------------------

const USSR_SIDEBAR_BG: Color = Color {
  r: 0.102,
  g: 0.031,
  b: 0.031,
  a: 1.0,
};
const USSR_SIDEBAR_TEXT: Color = Color {
  r: 0.949,
  g: 0.910,
  b: 0.816,
  a: 1.0,
};
const USSR_SIDEBAR_HOVER_BG: Color = Color {
  r: 1.0,
  g: 1.0,
  b: 1.0,
  a: 0.08,
};
const USSR_TEXT_STRONG: Color = Color {
  r: 0.102,
  g: 0.031,
  b: 0.000,
  a: 1.0,
};
const USSR_TEXT_SECONDARY: Color = Color {
  r: 0.239,
  g: 0.102,
  b: 0.000,
  a: 1.0,
};
const USSR_TEXT_MUTED: Color = Color {
  r: 0.478,
  g: 0.251,
  b: 0.125,
  a: 1.0,
};
const USSR_TEXT_HEADER: Color = Color {
  r: 0.604,
  g: 0.376,
  b: 0.251,
  a: 1.0,
};
const USSR_ACCENT: Color = Color {
  r: 0.800,
  g: 0.133,
  b: 0.000,
  a: 1.0,
};
const USSR_ACCENT_HOVER: Color = Color {
  r: 0.667,
  g: 0.102,
  b: 0.000,
  a: 1.0,
};
const USSR_ACCENT_SUBTLE: Color = Color {
  r: 0.800,
  g: 0.133,
  b: 0.000,
  a: 0.15,
};
const USSR_ACCENT_BORDER: Color = Color {
  r: 0.800,
  g: 0.133,
  b: 0.000,
  a: 0.45,
};
const USSR_TEXT_ON_ACCENT: Color = Color {
  r: 0.949,
  g: 0.910,
  b: 0.816,
  a: 1.0,
};
const USSR_SUCCESS: Color = Color {
  r: 0.290,
  g: 0.408,
  b: 0.125,
  a: 1.0,
};
const USSR_SUCCESS_BG: Color = Color {
  r: 0.290,
  g: 0.408,
  b: 0.125,
  a: 0.15,
};
const USSR_SUCCESS_HOVER: Color = Color {
  r: 0.220,
  g: 0.314,
  b: 0.063,
  a: 1.0,
};
const USSR_ERROR: Color = Color {
  r: 0.667,
  g: 0.000,
  b: 0.000,
  a: 1.0,
};
const USSR_ERROR_BG: Color = Color {
  r: 0.667,
  g: 0.000,
  b: 0.000,
  a: 0.15,
};
const USSR_BORDER: Color = Color {
  r: 0.545,
  g: 0.271,
  b: 0.075,
  a: 1.0,
};
const USSR_HEADER_BG: Color = Color {
  r: 0.831,
  g: 0.753,
  b: 0.627,
  a: 1.0,
};
const USSR_DETAIL_BG: Color = Color {
  r: 0.929,
  g: 0.878,
  b: 0.769,
  a: 1.0,
};
const USSR_CARD_BG: Color = Color {
  r: 0.961,
  g: 0.929,
  b: 0.831,
  a: 1.0,
};
const USSR_DESTRUCTIVE: Color = Color {
  r: 0.533,
  g: 0.000,
  b: 0.000,
  a: 1.0,
};
const USSR_DESTRUCTIVE_HOVER_BG: Color = Color {
  r: 0.533,
  g: 0.000,
  b: 0.000,
  a: 0.18,
};
const USSR_DISABLED_BG: Color = Color {
  r: 0.769,
  g: 0.659,
  b: 0.471,
  a: 1.0,
};
const USSR_PEACH: Color = Color {
  r: 0.800,
  g: 0.400,
  b: 0.000,
  a: 1.0,
};
const USSR_WARNING_BG: Color = Color {
  r: 0.800,
  g: 0.400,
  b: 0.000,
  a: 0.15,
};

// ---------------------------------------------------------------------------
// Dynamic theme functions — use these in UI code for theme-aware colours.
// ---------------------------------------------------------------------------

pub fn sidebar_bg() -> Color {
  match active() {
    ThemeVariant::Catppuccin => SIDEBAR_BG,
    ThemeVariant::Ussr => USSR_SIDEBAR_BG,
  }
}

pub fn sidebar_text() -> Color {
  match active() {
    ThemeVariant::Catppuccin => SIDEBAR_TEXT,
    ThemeVariant::Ussr => USSR_SIDEBAR_TEXT,
  }
}

pub fn sidebar_hover_bg() -> Color {
  match active() {
    ThemeVariant::Catppuccin => SIDEBAR_HOVER_BG,
    ThemeVariant::Ussr => USSR_SIDEBAR_HOVER_BG,
  }
}

pub fn text_strong() -> Color {
  match active() {
    ThemeVariant::Catppuccin => TEXT_STRONG,
    ThemeVariant::Ussr => USSR_TEXT_STRONG,
  }
}

pub fn text_secondary() -> Color {
  match active() {
    ThemeVariant::Catppuccin => TEXT_SECONDARY,
    ThemeVariant::Ussr => USSR_TEXT_SECONDARY,
  }
}

pub fn text_muted() -> Color {
  match active() {
    ThemeVariant::Catppuccin => TEXT_MUTED,
    ThemeVariant::Ussr => USSR_TEXT_MUTED,
  }
}

pub fn text_header() -> Color {
  match active() {
    ThemeVariant::Catppuccin => TEXT_HEADER,
    ThemeVariant::Ussr => USSR_TEXT_HEADER,
  }
}

pub fn accent() -> Color {
  match active() {
    ThemeVariant::Catppuccin => ACCENT,
    ThemeVariant::Ussr => USSR_ACCENT,
  }
}

pub fn accent_hover() -> Color {
  match active() {
    ThemeVariant::Catppuccin => ACCENT_HOVER,
    ThemeVariant::Ussr => USSR_ACCENT_HOVER,
  }
}

pub fn accent_subtle() -> Color {
  match active() {
    ThemeVariant::Catppuccin => ACCENT_SUBTLE,
    ThemeVariant::Ussr => USSR_ACCENT_SUBTLE,
  }
}

pub fn accent_border() -> Color {
  match active() {
    ThemeVariant::Catppuccin => ACCENT_BORDER,
    ThemeVariant::Ussr => USSR_ACCENT_BORDER,
  }
}

pub fn text_on_accent() -> Color {
  match active() {
    ThemeVariant::Catppuccin => TEXT_ON_ACCENT,
    ThemeVariant::Ussr => USSR_TEXT_ON_ACCENT,
  }
}

pub fn success() -> Color {
  match active() {
    ThemeVariant::Catppuccin => SUCCESS,
    ThemeVariant::Ussr => USSR_SUCCESS,
  }
}

pub fn success_bg() -> Color {
  match active() {
    ThemeVariant::Catppuccin => SUCCESS_BG,
    ThemeVariant::Ussr => USSR_SUCCESS_BG,
  }
}

pub fn success_hover() -> Color {
  match active() {
    ThemeVariant::Catppuccin => SUCCESS_HOVER,
    ThemeVariant::Ussr => USSR_SUCCESS_HOVER,
  }
}

pub fn error() -> Color {
  match active() {
    ThemeVariant::Catppuccin => ERROR,
    ThemeVariant::Ussr => USSR_ERROR,
  }
}

pub fn error_bg() -> Color {
  match active() {
    ThemeVariant::Catppuccin => ERROR_BG,
    ThemeVariant::Ussr => USSR_ERROR_BG,
  }
}

pub fn border() -> Color {
  match active() {
    ThemeVariant::Catppuccin => BORDER,
    ThemeVariant::Ussr => USSR_BORDER,
  }
}

pub fn header_bg() -> Color {
  match active() {
    ThemeVariant::Catppuccin => HEADER_BG,
    ThemeVariant::Ussr => USSR_HEADER_BG,
  }
}

pub fn detail_bg() -> Color {
  match active() {
    ThemeVariant::Catppuccin => DETAIL_BG,
    ThemeVariant::Ussr => USSR_DETAIL_BG,
  }
}

pub fn card_bg() -> Color {
  match active() {
    ThemeVariant::Catppuccin => CARD_BG,
    ThemeVariant::Ussr => USSR_CARD_BG,
  }
}

pub fn destructive() -> Color {
  match active() {
    ThemeVariant::Catppuccin => DESTRUCTIVE,
    ThemeVariant::Ussr => USSR_DESTRUCTIVE,
  }
}

pub fn destructive_hover_bg() -> Color {
  match active() {
    ThemeVariant::Catppuccin => DESTRUCTIVE_HOVER_BG,
    ThemeVariant::Ussr => USSR_DESTRUCTIVE_HOVER_BG,
  }
}

pub fn disabled_bg() -> Color {
  match active() {
    ThemeVariant::Catppuccin => DISABLED_BG,
    ThemeVariant::Ussr => USSR_DISABLED_BG,
  }
}

pub fn peach() -> Color {
  match active() {
    ThemeVariant::Catppuccin => PEACH,
    ThemeVariant::Ussr => USSR_PEACH,
  }
}

pub fn warning_bg() -> Color {
  match active() {
    ThemeVariant::Catppuccin => WARNING_BG,
    ThemeVariant::Ussr => USSR_WARNING_BG,
  }
}

/// Returns the navigation/sidebar font for the active theme.
/// USSR: Bebas Neue (all-caps condensed). Others: default.
pub fn nav_font() -> Font {
  match active() {
    ThemeVariant::Ussr => USSR_NAV_FONT,
    ThemeVariant::Catppuccin => Font::DEFAULT,
  }
}

/// Returns the heading/body font for the active theme.
/// USSR: Russo One (bold geometric). Others: default.
pub fn heading_font() -> Font {
  match active() {
    ThemeVariant::Ussr => USSR_HEADING_FONT,
    ThemeVariant::Catppuccin => Font::DEFAULT,
  }
}

/// Returns `ussr` string when the USSR theme is active, `normal` otherwise.
/// Used for Soviet-flavored UI copy in the USSR theme.
pub fn flavor(normal: &'static str, ussr: &'static str) -> &'static str {
  match active() {
    ThemeVariant::Ussr => ussr,
    ThemeVariant::Catppuccin => normal,
  }
}

/// Font for Soviet flavor page titles.
/// USSR: Bebas Neue (all-caps condensed, same as nav). Catppuccin: default bold.
pub fn flavor_title_font() -> Font {
  match active() {
    ThemeVariant::Ussr => nav_font(),
    ThemeVariant::Catppuccin => Font {
      weight: font::Weight::Bold,
      ..Font::DEFAULT
    },
  }
}

/// FA4 icon for the "published on keyserver" badge.
/// USSR: red star (\u{f005}). Others: check-circle (\u{f058}).
pub fn icon_published() -> &'static str {
  match active() {
    ThemeVariant::Ussr => "\u{f005}",
    ThemeVariant::Catppuccin => "\u{f058}",
  }
}
