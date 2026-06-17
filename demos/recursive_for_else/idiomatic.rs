fn rec(n: i64) -> i64 {
    for _ in [1_i64] {}
    if n > 0 {
        rec(n - 1)
    } else {
        0
    }
}

fn main() {
    println!("recursive_for_else canonical walker coverage demo:");
    println!("{}", rec(3));
}
