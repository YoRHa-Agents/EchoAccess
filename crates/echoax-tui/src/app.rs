pub struct App {
    pub running: bool,
    pub current_view: View,
}

pub enum View {
    Dashboard,
    Sync,
    Profiles,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            current_view: View::Dashboard,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn switch_view(&mut self, view: View) {
        self.current_view = view;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
