use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::theme::nier::NierTheme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(4)])
        .split(area);

    let info_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let cloud_status = if app.config.cloud.enabled {
        "Enabled"
    } else {
        "Disabled"
    };
    let session_info = Paragraph::new(format!(
        "  Session : Locked\n  Cloud   : {}\n  Version : v{}",
        cloud_status,
        env!("CARGO_PKG_VERSION"),
    ))
    .style(NierTheme::base())
    .block(
        Block::default()
            .title(" Status ")
            .borders(Borders::ALL)
            .style(NierTheme::border()),
    );
    frame.render_widget(session_info, info_chunks[0]);

    let stats = Paragraph::new(format!(
        "  Files   : {} tracked\n  Profiles: {}\n  Synced  : 0 (demo)\n  Pending : 0 (demo)",
        app.tracked_count,
        app.profile_names.len(),
    ))
    .style(NierTheme::base())
    .block(
        Block::default()
            .title(" Overview ")
            .borders(Borders::ALL)
            .style(NierTheme::border()),
    );
    frame.render_widget(stats, info_chunks[1]);

    let main_content = Paragraph::new("\n  Welcome to EchoAccess\n\n  Use [Tab] to navigate views, [u] to upload, [d] to download.\n  Press [q] to quit.")
        .style(NierTheme::panel())
        .block(
            Block::default()
                .title(" Dashboard ")
                .borders(Borders::ALL)
                .style(NierTheme::border()),
        );
    frame.render_widget(main_content, chunks[1]);
}
