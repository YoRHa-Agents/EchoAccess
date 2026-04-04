pub mod app;
pub mod theme;
pub mod views;

use app::{App, View};

pub async fn run() -> echoax_core::Result<()> {
    let mut app = App::new();
    println!("EchoAccess TUI (NieR: Automata style)");
    println!("Views: Dashboard, Sync, Profiles");
    app.switch_view(View::Dashboard);

    // TODO: integrate with crossterm + ratatui event loop
    // Real terminal UI rendering will be activated when connected to the sync engine
    app.quit();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_completes_without_error() {
        let result = run().await;
        assert!(result.is_ok());
    }

    #[test]
    fn app_lifecycle() {
        let mut app = app::App::new();
        assert!(app.running);
        app.switch_view(app::View::Dashboard);
        app.switch_view(app::View::Sync);
        app.switch_view(app::View::Profiles);
        app.quit();
        assert!(!app.running);
    }
}
