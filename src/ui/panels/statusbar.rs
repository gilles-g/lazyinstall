use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, toast: Option<&str>) {
    let (text, fg) = if let Some(msg) = toast {
        (format!(" {}", msg), Color::Yellow)
    } else {
        (
            " [Enter/u] update  [U] all  [a] add  [d] remove  [j/k] move  [q] quit  [?] help"
                .to_string(),
            Color::White,
        )
    };

    let bar = Paragraph::new(text).style(Style::default().fg(fg).bg(Color::DarkGray));
    frame.render_widget(bar, area);
}
