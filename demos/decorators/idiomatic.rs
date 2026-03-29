// @log
fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

// @validate
// @log
fn process(x: i64) -> i64 {
    x * 2
}

fn sum_all(nums: &[i64]) -> i64 {
    nums.iter().sum()
}

fn max_of(values: &[i64]) -> i64 {
    values.iter().copied().max().unwrap_or(0)
}

fn join_strings(sep: &str, parts: &[&str]) -> String {
    parts.join(sep)
}

fn main() {
    println!("{}", greet("World"));
    println!("{}", process(21));
    println!("{}", sum_all(&[1, 2, 3, 4, 5]));
    println!("{}", sum_all(&[10, 20]));
    println!("{}", max_of(&[3, 7, 2, 9, 1]));
    println!("{}", join_strings(", ", &["Alice", "Bob", "Charlie"]));
}
