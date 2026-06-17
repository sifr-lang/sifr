fn inferred(flag: bool) -> i64 {
    if flag {
        1
    } else {
        2
    }
}

fn consume(n: i64) -> i64 {
    n + 1
}

fn main() {
    println!("unreachable_returns diagnostics and consumer integration demo:");
    println!("{}", consume(inferred(true)));
    println!("{}", consume(inferred(false)));
}
