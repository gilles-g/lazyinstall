use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub target_list: Rect,
    pub output: Rect,
    pub statusbar: Rect,
}

pub fn compute(area: Rect) -> AppLayout {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Percentage(45),
            Constraint::Length(1),
        ])
        .split(area);
    AppLayout {
        target_list: chunks[0],
        output: chunks[1],
        statusbar: chunks[2],
    }
}
