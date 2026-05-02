use iced::Task;

use crate::gpg::KeyInfo;
use crate::ui;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum View {
  #[default]
  MyKeys,
  PublicKeys,
}

pub struct App {
  pub view: View,
  pub keys: Vec<KeyInfo>,
  pub selected: Option<usize>,
  pub error: Option<String>,
  pub loading: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
  KeysLoaded(Result<Vec<KeyInfo>, String>),
  NavChanged(View),
  KeySelected(usize),
}

impl Default for App {
  fn default() -> Self {
    Self {
      view: View::default(),
      keys: Vec::new(),
      selected: None,
      error: None,
      loading: false,
    }
  }
}

impl App {
  pub fn new() -> (Self, Task<Message>) {
    let task = Task::perform(
      async {
        tokio::task::spawn_blocking(crate::gpg::list_keys)
          .await
          .unwrap_or_else(|e| Err(anyhow::anyhow!(e)))
      },
      |result| Message::KeysLoaded(result.map_err(|e| e.to_string())),
    );
    (Self { loading: true, ..Default::default() }, task)
  }

  pub fn update(&mut self, message: Message) -> Task<Message> {
    match message {
      Message::KeysLoaded(Ok(keys)) => {
        self.keys = keys;
        self.loading = false;
      }
      Message::KeysLoaded(Err(e)) => {
        self.error = Some(e);
        self.loading = false;
      }
      Message::NavChanged(view) => {
        self.view = view;
        self.selected = None;
      }
      Message::KeySelected(i) => {
        self.selected = Some(i);
      }
    }
    Task::none()
  }

  pub fn view(&self) -> iced::Element<'_, Message> {
    ui::root(self)
  }
}
