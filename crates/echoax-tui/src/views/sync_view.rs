use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::app::App;
use crate::theme::nier::NierTheme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let items = vec![
        ListItem::new(format!(
            "  Tracked sync rules (profiles): {}",
            app.tracked_count
        )),
        ListItem::new("  ssh/config.base      ─── synced ───── 2m ago"),
        ListItem::new("  git/gitconfig.toml   ─── pending ──── now"),
    ];
    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(" Sync Status ({} rules) ", app.tracked_count))
                .borders(Borders::ALL)
                .style(NierTheme::border()),
        )
        .style(NierTheme::base());
    frame.render_widget(list, area);
}
