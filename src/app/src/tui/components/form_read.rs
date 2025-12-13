use crate::tui::App;
use api::api::uhf_rfid_api::UhfRfidApi;
use protocl::types::MemoryBank;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Bank
            Constraint::Length(3), // Address
            Constraint::Length(3), // Word count
            Constraint::Min(0),    // Results
        ])
        .margin(1)
        .split(area);

    // Draw form
    let form_block = Block::default()
        .borders(Borders::ALL)
        .title("Read Tag Data")
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

    // Show all bank options with current selection highlighted
    let banks = [
        MemoryBank::Reserved,
        MemoryBank::Epc,
        MemoryBank::Tid,
        MemoryBank::User,
    ];
    let bank_labels = banks
        .iter()
        .map(|bank| {
            let style = if *bank == app.read_bank {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let label = match bank {
                MemoryBank::Reserved => "[All]".to_string(),
                _ => format!("[{bank:?}]"),
            };
            Span::styled(label, style)
        })
        .collect::<Vec<_>>();

    let bank_text = Line::from(vec![
        Span::raw("Memory Bank: "),
        bank_labels[0].clone(),
        Span::raw(" "),
        bank_labels[1].clone(),
        Span::raw(" "),
        bank_labels[2].clone(),
        Span::raw(" "),
        bank_labels[3].clone(),
        Span::raw("    "),
        Span::raw("(←/→ to change, Tab to navigate, Enter to read)"),
    ]);

    let bank = Paragraph::new(bank_text).style(bank_style);
    f.render_widget(bank, chunks[0]);

    // Address input
    let address_style = if app.active_input_field == 1 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let address = Paragraph::new(format!("Address: {}", app.read_address)).style(address_style);
    f.render_widget(address, chunks[1]);

    // Word count input
    let word_count_style = if app.active_input_field == 2 {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let word_count =
        Paragraph::new(format!("Word Count: {}", app.read_word_count)).style(word_count_style);
    f.render_widget(word_count, chunks[2]);

    // Results panel (always visible so the user gets feedback even when no bytes are returned)
    if app.read_all_banks() {
        // Show per-bank sections and a combined length
        let total = app.result_epc.len() + app.result_tid.len() + app.result_user.len();
        let results_title = format!("Results from All Memory Banks (total {total} byte(s))");

        let section = |name: &str, data: &Vec<u8>| -> String {
            let count = data.len();
            if count == 0 {
                return format!("{name} (0 byte)\n    Hex: \n    ASCII: \n");
            }
            let hex = UhfRfidApi::hex_to_ascii(data);
            let mut ascii = String::new();
            for b in data {
                if b.is_ascii() && !b.is_ascii_control() {
                    ascii.push(*b as char);
                } else {
                    ascii.push('.');
                }
            }
            format!("{name} ({count} byte(s))\n    Hex: {hex}\n    ASCII: {ascii}\n")
        };

        let mut text = String::new();
        if total == 0 {
            text.push_str("No data yet. Press Enter to read.\n");
        }
        text.push_str(&section("EPC", &app.result_epc));
        text.push_str(&section("TID", &app.result_tid));
        text.push_str(&section("USER", &app.result_user));

        let results = Paragraph::new(text)
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::TOP).title(results_title));
        f.render_widget(results, chunks[3]);
    } else {
        let results_title = format!("Results from {:?}", app.read_bank);

        // For single-bank selection, prefer per-bank buffers and last-read flags
        let (name, data_ref, was_read) = match app.read_bank {
            MemoryBank::Epc => ("EPC", &app.result_epc, app.last_read_epc),
            MemoryBank::Tid => ("TID", &app.result_tid, app.last_read_tid),
            MemoryBank::User => ("USER", &app.result_user, app.last_read_user),
            MemoryBank::Reserved => ("", &app.result_data, false), // should not happen here
        };

        let (hex_data, ascii_data, header) = if was_read {
            let hex = UhfRfidApi::hex_to_ascii(data_ref);
            let mut ascii = String::new();
            for b in data_ref {
                if b.is_ascii() && !b.is_ascii_control() {
                    ascii.push(*b as char);
                } else {
                    ascii.push('.');
                }
            }
            let count = data_ref.len();
            (hex, ascii, format!("{name} ({count} byte(s))"))
        } else {
            (
                String::new(),
                String::new(),
                "This bank was not read in the last operation. Press Enter to read it.".to_string(),
            )
        };

        let results_text = format!("{header}\n    Hex: {hex_data}\n    ASCII: {ascii_data}");
        let results = Paragraph::new(results_text)
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::TOP).title(results_title));
        f.render_widget(results, chunks[3]);
    }
}
