use crate::tui::App;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Bank
            Constraint::Length(3), // Action
            Constraint::Min(0),    // Warning
        ])
        .margin(1)
        .split(area);

    // Draw form
    let form_block = Block::default()
        .borders(Borders::ALL)
        .title("Lock Tag Memory")
        .border_style(Style::default().fg(Color::Blue));
    f.render_widget(form_block, area);

    // Bank selection
    let bank_style = if app.active_input_field == 0 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let bank = Paragraph::new(format!(
        "Memory Bank: {:?} (←/→ to change, Tab to navigate, Enter to submit)",
        app.lock_bank
    ))
    .style(bank_style);
    f.render_widget(bank, chunks[0]);

    // Lock action
    let action_style = if app.active_input_field == 1 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let action = Paragraph::new(format!(
        "Lock Action: {:?} (←/→ to change, Tab to navigate, Enter to submit)",
        app.lock_action
    ))
    .style(action_style);
    f.render_widget(action, chunks[1]);

    // Warning
    let warning = Paragraph::new(
        "⚠️  WARNING: Lock operations can be PERMANENT depending on settings! ⚠️\n\n\
         Some lock actions cannot be reversed. Proceed with caution.",
    )
    .style(Style::default().fg(Color::Red));
    f.render_widget(warning, chunks[2]);
}
