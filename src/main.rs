mod app;
mod gpg;
mod ui;

use app::App;

fn main() -> iced::Result {
  iced::application("pgpilot", App::update, App::view)
    .font(include_bytes!("../assets/SymbolsNerdFontMono-Regular.ttf").as_slice())
    .subscription(App::subscription)
    .window(iced::window::Settings {
      min_size: Some(iced::Size::new(960.0, 540.0)),
      ..Default::default()
    })
    .run_with(App::new)
}
