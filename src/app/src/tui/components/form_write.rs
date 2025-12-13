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
            Constraint::Length(3), // Address
            Constraint::Length(3), // Data
            Constraint::Min(0),    // Instructions
        ])
        .margin(1)
        .split(area);

    // Draw form
    let form_block = Block::default()
        .borders(Borders::ALL)
        .title("Write Tag Data")
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
        app.write_bank
    ))
    .style(bank_style);
    f.render_widget(bank, chunks[0]);

    // Address input
    let address_style = if app.active_input_field == 1 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let address = Paragraph::new(format!("Address: {}", app.write_address)).style(address_style);
    f.render_widget(address, chunks[1]);

    // Data input
    let data_style = if app.active_input_field == 2 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let data = Paragraph::new(format!("Data (hex): {}", app.write_data)).style(data_style);
    f.render_widget(data, chunks[2]);

    // Instructions
    let instructions = Paragraph::new(
        "Enter hex data to write to the tag (e.g., 01020304).\n\n\
         Warning: Writing to the wrong memory bank or address may permanently damage the tag.",
    )
    .style(Style::default().fg(Color::Red));
    f.render_widget(instructions, chunks[3]);
}
