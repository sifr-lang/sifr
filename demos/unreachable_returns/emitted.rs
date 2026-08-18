// src/main.rs
fn inferred(flag: bool) -> i64 {
    if flag {
        return 1_i64;
    }
    2_i64
}

fn consume(n: i64) -> i64 {
    n + (1_i64)
}

fn main() {
    println!("unreachable_returns diagnostics and consumer integration demo:");
    println!("{}", consume(inferred(true)));
    println!("{}", consume(inferred(false)));
}
