fn smallest_or_zero(values: &[i64]) -> i64 {
    values.iter().copied().min().unwrap_or(0)
}

fn main() {
    assert_eq!(smallest_or_zero(&[8, 3, 7]), 3);
    assert_eq!(smallest_or_zero(&[]), 0);
    println!("sentinel_values: ok");
}
