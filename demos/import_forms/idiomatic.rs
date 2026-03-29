mod helper {
    pub fn value() -> i64 {
        17
    }
}

use helper::value;

fn main() {
    println!("m17_4 import-form semantics demo:");
    println!("{}", value());
}
