use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::app::App;
use crate::theme::nier::NierTheme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = if app.profile_names.is_empty() {
        vec![ListItem::new("  (no profiles in profiles/)")]
    } else {
        app.profile_names
            .iter()
            .map(|name| ListItem::new(format!("  {name}")))
            .collect()
    };
    let list = List::new(items)
        .block(
            Block::default()
                .title(" Profiles ")
                .borders(Borders::ALL)
                .style(NierTheme::border()),
        )
        .style(NierTheme::base());
    frame.render_widget(list, area);
}
