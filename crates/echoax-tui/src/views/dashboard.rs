use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::theme::nier::NierTheme;

pub fn render(frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let header = Paragraph::new(" EchoAccess")
        .style(NierTheme::highlight())
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .style(NierTheme::border()),
        );
    frame.render_widget(header, chunks[0]);

    let status = Paragraph::new("  Session: Locked  |  Cloud: Disconnected  |  Version: v0.1.0")
        .style(NierTheme::panel());
    frame.render_widget(status, chunks[1]);

    let help = Paragraph::new("  [u] Upload  [d] Download  [r] Refresh  [q] Quit")
        .style(NierTheme::secondary())
        .block(
            Block::default()
                .borders(Borders::TOP)
                .style(NierTheme::border()),
        );
    frame.render_widget(help, chunks[2]);
}
