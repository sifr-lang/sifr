// src/main.rs
mod formatter;
mod helper;

use crate::helper::render;

fn main() {
    println!("{}", render());
}

// src/formatter.rs
pub fn render_value() -> String {
    let value: i64 = 42 as i64;
    return format!("{}", value);
}

// src/helper.rs
use crate::formatter::render_value;

pub fn render() -> String {
    return render_value();
}
