use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// Surimpression de saisie du mot de passe sudo réclamé par une cible.
/// Contextualisée : elle nomme la cible qui demande et rappelle l'invite.
pub fn render(frame: &mut Frame, area: Rect, target: &str, prompt: Option<&str>, input: &str) {
    let popup_width = 70.min(area.width);
    let popup_height = 6.min(area.height);
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    let masked: String = "•".repeat(input.chars().count());
    let lines = vec![
        Line::from(Span::styled(
            prompt.unwrap_or("Mot de passe sudo :"),
            Style::default().fg(Color::Gray),
        )),
        Line::from(vec![
            Span::styled("› ", Style::default().fg(Color::Yellow)),
            Span::styled(masked, Style::default().fg(Color::White)),
            Span::styled("▌", Style::default().fg(Color::Yellow)),
        ]),
    ];

    let title = format!(" Mot de passe sudo — « {target} » — Enter pour valider, Esc pour annuler ");
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    frame.render_widget(Clear, popup_area);
    frame.render_widget(Paragraph::new(lines).block(block), popup_area);
}
