// src/main.rs
fn identity(x: i64) -> i64 {
    x
}

fn main() {
    let value: i64 = identity(17_i64);
    println!("type_checking frontend-only check path demo:");
    println!("{}", value);
}
