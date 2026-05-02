mod app;
mod gpg;
mod ui;

use app::App;

fn main() -> iced::Result {
  iced::application("pgpilot", App::update, App::view)
    .font(include_bytes!("../assets/SymbolsNerdFontMono-Regular.ttf").as_slice())
    .run_with(App::new)
}
