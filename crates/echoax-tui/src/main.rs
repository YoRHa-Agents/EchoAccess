#![allow(dead_code, unused_imports)]

mod app;
mod theme;
mod views;

use app::{App, View};

fn main() {
    let mut app = App::new();
    println!("EchoAccess TUI (NieR: Automata style)");
    println!("Use --tui flag to launch interactive mode");
    println!("Views: Dashboard, Sync, Profiles");
    app.switch_view(View::Dashboard);

    // TODO: integrate with crossterm + ratatui event loop
    // For now, the TUI module structure is in place with NieR theme
    // Real terminal UI rendering will be activated when connected to the sync engine
    app.quit();
}
