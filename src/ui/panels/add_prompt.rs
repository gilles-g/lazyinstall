use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// Surimpression de saisie du chemin d'un dossier à suivre.
pub fn render(frame: &mut Frame, area: Rect, input: &str) {
    let popup_width = 70.min(area.width);
    let popup_height = 5.min(area.height);
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    let lines = vec![
        Line::from(Span::styled(
            "Path of the folder to track:",
            Style::default().fg(Color::Gray),
        )),
        Line::from(vec![
            Span::styled("› ", Style::default().fg(Color::Cyan)),
            Span::styled(input, Style::default().fg(Color::White)),
            Span::styled("▌", Style::default().fg(Color::Cyan)),
        ]),
    ];

    let block = Block::default()
        .title(" Add a folder — Enter to confirm, Esc to cancel ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(Clear, popup_area);
    frame.render_widget(Paragraph::new(lines).block(block), popup_area);
}
