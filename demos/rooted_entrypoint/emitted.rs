// src/main.rs
pub mod helper;
pub mod shared;
use crate::helper::render;
fn main() {
    println!("{}", render());
}

// src/helper.rs
pub use crate::shared::label;
#[must_use]
pub fn render() -> String {
    label()
}

// src/shared.rs
#[must_use]
pub fn label() -> String {
    "rooted entrypoint demo: pass".to_string()
}
