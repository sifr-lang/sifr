fn recurse(n: i64) -> i64 {
    if n > 0 {
        recurse(n - 1)
    } else {
        0
    }
}

fn main() {
    println!("recursive_calls semantic query layer standardization demo:");
    println!("{}", recurse(4));
}
