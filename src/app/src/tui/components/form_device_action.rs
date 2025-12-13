use crate::tui::App;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Beep
            Constraint::Length(3), // Red LED
            Constraint::Length(3), // Green LED
            Constraint::Length(3), // Yellow LED
            Constraint::Length(3), // Time
            Constraint::Min(0),    // Instructions
        ])
        .margin(1)
        .split(area);

    // Draw form
    let form_block = Block::default()
        .borders(Borders::ALL)
        .title("Device Actions")
        .border_style(Style::default().fg(Color::Blue));
    f.render_widget(form_block, area);

    // Beep checkbox
    let beep_style = if app.active_input_field == 0 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let beep_checkbox = if app.action_beep { "[X]" } else { "[ ]" };
    let beep = Paragraph::new(format!("{beep_checkbox} Beep (Space to toggle)")).style(beep_style);
    f.render_widget(beep, chunks[0]);

    // Red LED checkbox
    let red_style = if app.active_input_field == 1 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let red_checkbox = if app.action_red { "[X]" } else { "[ ]" };
    let red = Paragraph::new(format!("{red_checkbox} Red LED")).style(red_style);
    f.render_widget(red, chunks[1]);

    // Green LED checkbox
    let green_style = if app.active_input_field == 2 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let green_checkbox = if app.action_green { "[X]" } else { "[ ]" };
    let green = Paragraph::new(format!("{green_checkbox} Green LED")).style(green_style);
    f.render_widget(green, chunks[2]);

    // Yellow LED checkbox
    let yellow_style = if app.active_input_field == 3 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let yellow_checkbox = if app.action_yellow { "[X]" } else { "[ ]" };
    let yellow = Paragraph::new(format!("{yellow_checkbox} Yellow LED")).style(yellow_style);
    f.render_widget(yellow, chunks[3]);

    // Time input
    let time_style = if app.active_input_field == 4 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let time =
        Paragraph::new(format!("Duration (in 10ms units): {}", app.action_time)).style(time_style);
    f.render_widget(time, chunks[4]);

    // Instructions
    let instructions = Paragraph::new(
        "Space to toggle options, Tab to navigate fields.\n\
         Enter to perform the action, Esc to cancel.\n\
         The duration is in deciseconds (10ms units), so 50 = 500ms.",
    )
    .style(Style::default().fg(Color::Green));
    f.render_widget(instructions, chunks[5]);
}
