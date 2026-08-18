// src/main.rs
fn summarize(values: &Vec<i64>) -> i64 {
    let mut total: i64 = 0_i64;
    for value in values.iter().copied() {
        if value > (10_i64) {
            total += value;
        } else {
            total += 1_i64;
        }
    }
    total
}

fn main() {
    println!("stable_codegen analysis/emission boundary hardening demo:");
    println!("{}", summarize(&vec![3_i64, 12_i64, 20_i64]));
}
