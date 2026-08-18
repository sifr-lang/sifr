// src/main.rs
mod formatter;
mod helper;

use crate::helper::render;

fn main() {
    println!("{}", render());
}

// src/formatter.rs
pub fn render_value() -> String {
    let value: i64 = 42_i64;
    format!("{}", value)
}

// src/helper.rs
pub use crate::formatter::render_value;
pub fn render() -> String {
    render_value()
}
