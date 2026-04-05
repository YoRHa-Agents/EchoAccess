pub mod dashboard;
pub mod profiles;
pub mod sync_view;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use ratatui::Frame;

use crate::app::{App, View};
use crate::theme::nier::NierTheme;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let header = Paragraph::new("  E C H O A C C E S S")
        .style(NierTheme::highlight())
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .style(NierTheme::border()),
        );
    frame.render_widget(header, chunks[0]);

    let tab_titles = vec!["[1] Dashboard", "[2] Sync", "[3] Profiles"];
    let selected = match app.current_view {
        View::Dashboard => 0,
        View::Sync => 1,
        View::Profiles => 2,
    };
    let tabs = Tabs::new(tab_titles)
        .select(selected)
        .style(NierTheme::secondary())
        .highlight_style(NierTheme::accent())
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .style(NierTheme::border()),
        )
        .divider(" │ ");
    frame.render_widget(tabs, chunks[1]);

    match app.current_view {
        View::Dashboard => dashboard::render(frame, chunks[2], app),
        View::Sync => sync_view::render(frame, chunks[2], app),
        View::Profiles => profiles::render(frame, chunks[2], app),
    }

    let status_text = if app.status_message.is_empty() {
        "  [Tab] Switch view  [u] Upload  [d] Download  [r] Refresh  [q] Quit".to_string()
    } else {
        format!("  {} │ [q] Quit", app.status_message)
    };
    let help = Paragraph::new(status_text)
        .style(NierTheme::secondary())
        .block(
            Block::default()
                .borders(Borders::TOP)
                .style(NierTheme::border()),
        );
    frame.render_widget(help, chunks[3]);
}
