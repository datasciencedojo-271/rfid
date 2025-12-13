use crate::tui::{App, MenuItem};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;
use strum::IntoEnumIterator;

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    // Split into two columns
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Menu
            Constraint::Percentage(70), // Tag list
        ])
        .split(area);

    // Draw menu
    let menu_items: Vec<ListItem> = MenuItem::iter()
        .map(|item| {
            let style = if item == app.selected_menu_item {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(item.to_string()).style(style)
        })
        .collect();

    let menu = List::new(menu_items)
        .block(Block::default().borders(Borders::ALL).title("Menu"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_widget(menu, chunks[0]);

    // Draw tag list
    let tag_items: Vec<ListItem> = app
        .tags
        .iter()
        .enumerate()
        .map(|(i, tag)| {
            let bank_info = match tag.data_bank {
                Some(bank) => format!(" [{bank}]"),
                None => String::new(),
            };
            ListItem::new(format!("{}. {}{}", i + 1, tag.epc, bank_info))
                .style(Style::default().fg(Color::Green))
        })
        .collect();

    let tags = List::new(tag_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Tags in Range"),
    );

    f.render_widget(tags, chunks[1]);
}
