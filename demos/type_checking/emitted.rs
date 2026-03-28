fn identity(x: i64) -> i64 {
    return x;
}

fn main() {
    let value: i64 = identity(17 as i64);
    println!("m17_1 frontend-only check path demo:");
    println!("{}", value);
}
