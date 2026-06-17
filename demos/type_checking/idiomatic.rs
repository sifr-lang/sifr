fn identity(x: i64) -> i64 {
    x
}

fn main() {
    let value = identity(17);
    println!("type_checking frontend-only check path demo:");
    println!("{}", value);
}
