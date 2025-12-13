use colorful::{Color, Colorful};

pub fn print_as_ascii(msg: &str, data: &Vec<u8>) {
    print!("{} ", msg.color(Color::Cyan));
    for b in data {
        if b.is_ascii() && !b.is_ascii_control() {
            print!("{}", (*b as char).to_string().color(Color::White));
        } else {
            print!("{}", ".".color(Color::DarkGray));
        }
    }
    println!();
}
