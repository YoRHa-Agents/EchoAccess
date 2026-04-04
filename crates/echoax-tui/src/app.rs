use crossterm::event::KeyCode;

pub struct App {
    pub running: bool,
    pub current_view: View,
    pub status_message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Sync,
    Profiles,
}

impl View {
    pub fn label(&self) -> &str {
        match self {
            View::Dashboard => "Dashboard",
            View::Sync => "Sync",
            View::Profiles => "Profiles",
        }
    }

    pub fn next(&self) -> View {
        match self {
            View::Dashboard => View::Sync,
            View::Sync => View::Profiles,
            View::Profiles => View::Dashboard,
        }
    }

    pub fn prev(&self) -> View {
        match self {
            View::Dashboard => View::Profiles,
            View::Sync => View::Dashboard,
            View::Profiles => View::Sync,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            current_view: View::Dashboard,
            status_message: String::new(),
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn switch_view(&mut self, view: View) {
        self.current_view = view;
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.quit(),
            KeyCode::Tab | KeyCode::Right => {
                let next = self.current_view.next();
                self.switch_view(next);
            }
            KeyCode::BackTab | KeyCode::Left => {
                let prev = self.current_view.prev();
                self.switch_view(prev);
            }
            KeyCode::Char('1') => self.switch_view(View::Dashboard),
            KeyCode::Char('2') => self.switch_view(View::Sync),
            KeyCode::Char('3') => self.switch_view(View::Profiles),
            KeyCode::Char('u') => {
                self.status_message = "Upload triggered (sync engine not connected)".into();
            }
            KeyCode::Char('d') => {
                self.status_message = "Download triggered (sync engine not connected)".into();
            }
            KeyCode::Char('r') => {
                self.status_message = "Refreshing...".into();
            }
            _ => {}
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
