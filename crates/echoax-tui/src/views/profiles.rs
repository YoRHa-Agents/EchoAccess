use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::theme::nier::NierTheme;

pub fn render(frame: &mut Frame, area: Rect) {
    let items = vec![
        ListItem::new("  linux-server-01 (srv-01)"),
        ListItem::new("  macos-dev (mac-1)"),
    ];
    let list = List::new(items)
        .block(Block::default().title(" Profiles ").borders(Borders::ALL).style(NierTheme::border()))
        .style(NierTheme::base());
    frame.render_widget(list, area);
}
