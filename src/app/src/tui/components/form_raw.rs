use crate::tui::App;
use api::api::uhf_rfid_api::UhfRfidApi;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input
            Constraint::Length(5), // Instructions
            Constraint::Min(0),    // Response
        ])
        .margin(1)
        .split(area);

    // Draw form block
    let form_block = Block::default()
        .borders(Borders::ALL)
        .title("Manual Raw Command")
        .border_style(Style::default().fg(Color::Blue));
    f.render_widget(form_block, area);

    // Input
    let input_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let input = Paragraph::new(format!("Command (hex): {}", app.raw_input)).style(input_style);
    f.render_widget(input, chunks[0]);

    // Instructions/warning
    let instructions = Paragraph::new(
        "WARNING: Sending raw commands may damage the device or tags.\n\
         Enter even-length hex string (e.g., 01020304). Press Enter to send. ESC to go back.",
    )
    .style(Style::default().fg(Color::Red));
    f.render_widget(instructions, chunks[1]);

    // Response
    if !app.raw_response.is_empty() {
        let hex = UhfRfidApi::hex_to_ascii(&app.raw_response);
        let mut ascii = String::new();
        for b in &app.raw_response {
            if b.is_ascii() && !b.is_ascii_control() {
                ascii.push(*b as char);
            } else {
                ascii.push('.');
            }
        }
        let resp = Paragraph::new(format!("Response (hex): {hex}\nResponse (ASCII): {ascii}"))
            .style(Style::default().fg(Color::Green))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .title("Last Response"),
            );
        f.render_widget(resp, chunks[2]);
    }
}
