mod app;
mod config;
mod gpg;
mod i18n;
mod ui;

use app::App;

fn main() -> iced::Result {
  iced::application(App::new, App::update, App::view)
    .title("pgpilot")
    .font(include_bytes!("../assets/SymbolsNerdFontMono-Regular.ttf").as_slice())
    .subscription(App::subscription)
    .window(iced::window::Settings {
      min_size: Some(iced::Size::new(1000.0, 540.0)),
      ..Default::default()
    })
    .run()
}
