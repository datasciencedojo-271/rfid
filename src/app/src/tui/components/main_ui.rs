//! UI rendering for the TUI

use crate::tui::app::{App, AppState};
use crate::tui::components::{
    form_device_action, form_lock, form_password, form_raw, form_read, form_write, menu_main,
    panel_device_info,
};
use ratatui::prelude::*;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn draw(f: &mut Frame, app: &mut App) {
    // Create a layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title bar
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(f.area());

    // Draw title bar
    let title = Paragraph::new("UHF RFID Programmer")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Draw main content based on the current state
    match app.state {
        AppState::Main => {
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(chunks[1]);

            menu_main::draw(f, app, main_chunks[0]);
            panel_device_info::draw_device_info(f, app, main_chunks[1]);
        }
        AppState::Read => form_read::draw(f, app, chunks[1]),
        AppState::Write => form_write::draw(f, app, chunks[1]),
        AppState::Lock => form_lock::draw(f, app, chunks[1]),
        AppState::Password => form_password::draw(f, app, chunks[1]),
        AppState::Action => form_device_action::draw(f, app, chunks[1]),
        AppState::Raw => form_raw::draw(f, app, chunks[1]),
        AppState::Test => {
            let block = Paragraph::new("Press Enter to run the built-in test.")
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Run Test"));
            f.render_widget(block, chunks[1]);
        }
    }

    // Draw the status bar
    let status = Paragraph::new(app.status_message.clone())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(status, chunks[2]);
}
