use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::ui::keys::HELP;

pub fn render(frame: &mut Frame, area: Rect) {
    let popup_width = 58.min(area.width);
    let popup_height = (HELP.len() as u16 + 4).min(area.height);
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    let lines: Vec<Line> = HELP
        .iter()
        .map(|b| {
            Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{:<16}", b.keys), Style::default().fg(Color::Cyan)),
                Span::raw(b.desc),
            ])
        })
        .collect();

    let block = Block::default()
        .title(" Help — Esc to close ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(Clear, popup_area);
    frame.render_widget(Paragraph::new(lines).block(block), popup_area);
}
