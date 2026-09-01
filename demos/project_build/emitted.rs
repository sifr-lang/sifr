// src/main.rs
pub mod formatter;
pub mod helper;
use crate::helper::render;
fn main() {
    println!("{}", render());
}

// src/formatter.rs
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub fn render_value() -> String {
    let value: SifrInt = SifrInt::from_i64(42);
    value.to_string()
}

// src/helper.rs
pub use crate::formatter::render_value;
#[must_use]
pub fn render() -> String {
    render_value()
}
