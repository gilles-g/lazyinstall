use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::install::tracked::{TrackedTarget, UpdateState};

pub fn render(frame: &mut Frame, area: Rect, targets: &[TrackedTarget], cursor: usize) {
    let items: Vec<ListItem> = targets
        .iter()
        .enumerate()
        .map(|(i, tracked)| {
            let selected = i == cursor;
            let indicator = match tracked.state() {
                UpdateState::Succeeded => "●",
                UpdateState::Updating => "◌",
                UpdateState::Failed(_) => "✗",
                UpdateState::Idle => "○",
            };
            let state_style = style_for_state(tracked.state());
            let name_style = if selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            let folder = tracked.target().folder().display().to_string();

            let line = Line::from(vec![
                Span::styled(format!(" {} ", indicator), state_style),
                Span::styled(format!("{:<18}", tracked.name()), name_style),
                Span::styled(format!("{:<10}", tracked.state().label()), state_style),
                Span::styled(folder, Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let title = if targets.is_empty() {
        " lazyinstall — aucun dossier suivi ([a] pour en ajouter) "
    } else {
        " lazyinstall — dossiers suivis "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let mut state = ListState::default();
    if !targets.is_empty() {
        state.select(Some(cursor));
    }

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_stateful_widget(list, area, &mut state);
}

fn style_for_state(state: &UpdateState) -> Style {
    match state {
        UpdateState::Succeeded => Style::default().fg(Color::Green),
        UpdateState::Updating => Style::default().fg(Color::Yellow),
        UpdateState::Idle => Style::default().fg(Color::DarkGray),
        UpdateState::Failed(_) => Style::default().fg(Color::Red),
    }
}
