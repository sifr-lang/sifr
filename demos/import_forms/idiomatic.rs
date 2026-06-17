mod helper {
    pub fn value() -> i64 {
        17
    }
}

use helper::value;

fn main() {
    println!("import_forms import-form semantics demo:");
    println!("{}", value());
}
