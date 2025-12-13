use crate::tui::App;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Password
            Constraint::Min(0),    // Warning
        ])
        .margin(1)
        .split(area);

    // Draw form
    let form_block = Block::default()
        .borders(Borders::ALL)
        .title("Set Access Password")
        .border_style(Style::default().fg(Color::Blue));
    f.render_widget(form_block, area);

    // Password input
    let password = Paragraph::new(format!("Password (hex): {}", app.password)).style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(password, chunks[0]);

    // Warning
    let warning = Paragraph::new(
        "⚠️  IMPORTANT: Make sure to write down this password! ⚠️\n\n\
         If you lose this password, you may not be able to access your tag.\n\n\
         Enter a hex password (e.g., 12345678) and press Enter to set it.",
    )
    .style(Style::default().fg(Color::Red));
    f.render_widget(warning, chunks[1]);
}
