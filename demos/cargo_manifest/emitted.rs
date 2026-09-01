// src/main.rs
pub mod helper;
use crate::helper::render;
fn main() {
    println!("{}", render());
}

// src/helper.rs
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub fn render() -> String {
    let value: SifrInt = SifrInt::from_i64(42);
    if value.to_string() == "42" {
        return "manifest unification demo: pass".to_string();
    }
    "manifest unification demo: fail".to_string()
}
