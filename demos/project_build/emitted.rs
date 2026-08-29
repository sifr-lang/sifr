// src/main.rs
mod formatter;
mod helper;

use crate::helper::render;

fn main() {
    println!("{}", render());
}

// src/formatter.rs
pub use ::sifr_runtime::SifrInt;
pub fn render_value() -> String {
    let value: SifrInt = SifrInt::from_i64(42);
    format!("{}", value)
}

// src/helper.rs
pub use crate::formatter::render_value;
pub fn render() -> String {
    render_value()
}
