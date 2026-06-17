fn inferred(flag: bool) -> i64 {
    if flag {
        return 1 as i64;
    }
    return 2 as i64;
}

fn consume(n: i64) -> i64 {
    return n + (1 as i64);
}

fn main() {
    println!("unreachable_returns diagnostics and consumer integration demo:");
    println!("{}", consume(inferred(true)));
    println!("{}", consume(inferred(false)));
}
