// src/main.rs
mod helper;
mod shared;

use crate::helper::render;

fn main() {
    println!("{}", render());
}

// src/helper.rs
pub use crate::shared::label;
pub fn render() -> String {
    label()
}

// src/shared.rs
pub fn label() -> String {
    "rooted entrypoint demo: pass".to_string()
}
