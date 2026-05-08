// src/main.rs
mod helper;

use crate::helper::render;

fn main() {
    println!("{}", render());
}

// src/helper.rs
pub fn render() -> String {
    let value: i64 = 42 as i64;
    if (format!("{}", value) == "42".to_string()) {
        return "adhoc milestone 3 manifest unification demo: pass".to_string();
    }
    return "adhoc milestone 3 manifest unification demo: fail".to_string();
}
