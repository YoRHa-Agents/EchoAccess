pub mod app;
pub mod theme;
pub mod views;

use std::io;

use app::App;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;

pub async fn run() -> echoax_core::Result<()> {
    let mut app = App::new();

    enable_raw_mode().map_err(echoax_core::EchoAccessError::Io)?;
    io::stdout()
        .execute(EnterAlternateScreen)
        .map_err(echoax_core::EchoAccessError::Io)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).map_err(echoax_core::EchoAccessError::Io)?;
    terminal.clear().map_err(echoax_core::EchoAccessError::Io)?;

    let result = run_loop(&mut terminal, &mut app).await;

    disable_raw_mode().ok();
    io::stdout().execute(LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> echoax_core::Result<()> {
    while app.running {
        terminal
            .draw(|frame| {
                views::render(frame, app);
            })
            .map_err(echoax_core::EchoAccessError::Io)?;

        if event::poll(std::time::Duration::from_millis(100))
            .map_err(echoax_core::EchoAccessError::Io)?
        {
            if let Event::Key(key) = event::read().map_err(echoax_core::EchoAccessError::Io)? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn app_key_handling() {
        let mut app = app::App::new();
        assert_eq!(app.current_view, app::View::Dashboard);

        app.handle_key(crossterm::event::KeyCode::Tab);
        assert_eq!(app.current_view, app::View::Sync);

        app.handle_key(crossterm::event::KeyCode::Tab);
        assert_eq!(app.current_view, app::View::Profiles);

        app.handle_key(crossterm::event::KeyCode::Char('1'));
        assert_eq!(app.current_view, app::View::Dashboard);

        app.handle_key(crossterm::event::KeyCode::Char('q'));
        assert!(!app.running);
    }

    #[test]
    fn view_navigation() {
        assert_eq!(app::View::Dashboard.next(), app::View::Sync);
        assert_eq!(app::View::Sync.next(), app::View::Profiles);
        assert_eq!(app::View::Profiles.next(), app::View::Dashboard);

        assert_eq!(app::View::Dashboard.prev(), app::View::Profiles);
        assert_eq!(app::View::Sync.prev(), app::View::Dashboard);
        assert_eq!(app::View::Profiles.prev(), app::View::Sync);
    }
}
