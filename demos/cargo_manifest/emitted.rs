// src/main.rs
mod helper;

use crate::helper::render;

fn main() {
    println!("{}", render());
}

// src/helper.rs
pub use ::sifr_runtime::SifrInt;
pub fn render() -> String {
    let value: SifrInt = SifrInt::from_i64(42);
    if (format!("{}", value) == "42") {
        return "manifest unification demo: pass".to_string();
    }
    "manifest unification demo: fail".to_string()
}
