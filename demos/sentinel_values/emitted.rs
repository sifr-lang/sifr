// src/main.rs
fn smallest_or_zero(values: &Vec<i64>) -> i64 {
    let mut best: i64 = 9223372036854775807_i64;
    for value in values.iter().copied() {
        if value < best {
            best = value;
        }
    }
    if best != (9223372036854775807_i64) { best } else { 0_i64 }
}

fn main() {
    assert!((smallest_or_zero(&vec![8_i64, 3_i64, 7_i64]) == (3_i64)));
    assert!((smallest_or_zero(&vec![]) == (0_i64)));
    println!("sentinel_values: ok");
}
