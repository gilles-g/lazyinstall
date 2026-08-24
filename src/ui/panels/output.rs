use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::install::tracked::TrackedTarget;

/// Affiche la sortie de la mise à jour de la cible sélectionnée.
pub fn render(frame: &mut Frame, area: Rect, selected: Option<&TrackedTarget>) {
    let title = match selected {
        Some(tracked) => format!(" Output: {} ", tracked.name()),
        None => " Output ".to_string(),
    };

    let inner_height = area.height.saturating_sub(2) as usize;

    let items: Vec<ListItem> = match selected {
        Some(tracked) if !tracked.logs().is_empty() => {
            let logs = tracked.logs();
            // On ne garde que les dernières lignes visibles : la fin du flux
            // est ce qui intéresse (on reste « collé » en bas).
            let skip = logs.len().saturating_sub(inner_height.max(1));
            logs.iter()
                .skip(skip)
                .map(|line| {
                    ListItem::new(Line::from(Span::styled(
                        format!("  {}", line),
                        style_for_line(line),
                    )))
                })
                .collect()
        }
        Some(_) => vec![placeholder("  No output yet — [Enter/u] to run the update")],
        None => vec![placeholder("  No folder tracked")],
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    frame.render_widget(List::new(items).block(block), area);
}

fn placeholder(text: &str) -> ListItem<'_> {
    ListItem::new(Line::from(Span::styled(
        text.to_string(),
        Style::default().fg(Color::DarkGray),
    )))
}

fn style_for_line(line: &str) -> Style {
    let lower = line.to_ascii_lowercase();
    if lower.contains("erreur")
        || lower.contains("error")
        || lower.contains("échec")
        || lower.contains("failed")
    {
        Style::default().fg(Color::Red)
    } else if lower.contains("warn") || lower.contains("attention") {
        Style::default().fg(Color::Yellow)
    } else if lower.contains("à jour")
        || lower.contains("up to date")
        || lower.contains("updated")
        || lower.contains("mis à jour")
    {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Gray)
    }
}
