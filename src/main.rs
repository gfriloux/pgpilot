mod app;
mod gpg;
mod ui;

use app::App;

fn main() -> iced::Result {
  iced::application("pgpilot", App::update, App::view).run_with(App::new)
}
