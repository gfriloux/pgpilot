mod app;
mod config;
mod gpg;
mod i18n;
mod ui;

use app::App;

fn main() -> iced::Result {
  iced::application(App::new, App::update, App::view)
    .title("PGPilot")
    .font(include_bytes!("../assets/SymbolsNerdFontMono-Regular.ttf").as_slice())
    .font(include_bytes!("../assets/BebasNeue-Regular.ttf").as_slice())
    .font(include_bytes!("../assets/RussoOne-Regular.ttf").as_slice())
    .subscription(App::subscription)
    // Apply the persisted scale factor at startup.
    // iced 0.14 exposes `.scale_factor()` on the application builder;
    // the closure receives the window id and the app state.
    .scale_factor(|app: &App| app.config.scale_factor as f32)
    .window(iced::window::Settings {
      min_size: Some(iced::Size::new(1000.0, 540.0)),
      ..Default::default()
    })
    .run()
}
