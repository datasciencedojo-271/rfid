use crate::tui::App;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw_device_info(f: &mut Frame, app: &mut App, area: Rect) {
    // Draw device info
    let device_block = Block::default()
        .borders(Borders::ALL)
        .title("Device Information")
        .border_style(Style::default().fg(Color::Blue));
    f.render_widget(device_block.clone(), area);

    if let Some(ref device) = app.device {
        let info = device.get_info();
        let info_text = format!(
            "Device: {} ({:04x}:{:04x})\n\
             Manufacturer: {}\n\
             Serial Number: {}\n\
             Connection: Interface {}, IN endpoint 0x{:02X}, OUT endpoint 0x{:02X}\n\
             Is connected: {}",
            info.product,
            info.vendor_id,
            info.product_id,
            info.manufacturer,
            info.serial_number,
            device.get_interface(),
            device.get_in_endpoint(),
            device.get_out_endpoint(),
            if device.is_connected() { "Yes" } else { "No" }
        );

        let info_widget = Paragraph::new(info_text)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Left);

        let inner_area = device_block.inner(area);
        f.render_widget(info_widget, inner_area);
    } else {
        let no_device = Paragraph::new("No device connected.")
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);

        let inner_area = device_block.inner(area);
        f.render_widget(no_device, inner_area);
    }
}
