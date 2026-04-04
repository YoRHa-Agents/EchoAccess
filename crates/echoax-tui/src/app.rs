use std::path::PathBuf;

use crossterm::event::KeyCode;
use echoax_core::config::model::AppConfig;

pub struct App {
    pub running: bool,
    pub current_view: View,
    pub status_message: String,
    pub config: AppConfig,
    pub config_dir: PathBuf,
    pub profile_names: Vec<String>,
    pub tracked_count: usize,
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
        let config_dir = dirs::config_dir()
            .map(|p| p.join("echoax"))
            .unwrap_or_else(|| PathBuf::from("."));

        let config_path = config_dir.join("config.toml");
        let config = AppConfig::load(&config_path).unwrap_or_default();

        let profiles_dir = config_dir.join("profiles");
        let mut profile_names = Vec::new();
        let mut tracked_count = 0;

        if let Ok(entries) = std::fs::read_dir(&profiles_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        profile_names.push(stem.to_string());
                    }
                    if let Ok(profile) = echoax_core::profile::load_profile(&path) {
                        tracked_count += profile.sync_rules.len();
                    }
                }
            }
        }

        profile_names.sort();

        Self {
            running: true,
            current_view: View::Dashboard,
            status_message: String::new(),
            config,
            config_dir,
            profile_names,
            tracked_count,
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
