use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::theme::nier::NierTheme;

pub fn render(frame: &mut Frame, area: Rect) {
    let items = vec![
        ListItem::new("  ssh/config.base      ─── synced ───── 2m ago"),
        ListItem::new("  git/gitconfig.toml   ─── pending ──── now"),
    ];
    let list = List::new(items)
        .block(
            Block::default()
                .title(" Sync Status ")
                .borders(Borders::ALL)
                .style(NierTheme::border()),
        )
        .style(NierTheme::base());
    frame.render_widget(list, area);
}
