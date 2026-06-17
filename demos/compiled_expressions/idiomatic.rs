fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn floor(n: f64) -> i64 {
    n.floor() as i64
}

fn main() {
    let total = add(10, 11);
    if total > 20 {
        println!("compiled_expressions lower decomposition demo:");
    }
    println!("{total}");
    println!("{}", floor(3.9));
}
