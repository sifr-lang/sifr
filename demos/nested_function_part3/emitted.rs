// src/main.rs
fn accumulate(values: &Vec<i64>) -> i64 {
    let mut total: i64 = 0_i64;
    let mut apply = || {
    for value in values.iter().copied() {
        total += value;
    }
};
    apply();
    total
}

fn main() {
    assert!((format!("{}", accumulate(&vec![3_i64, 1_i64, 4_i64, 1_i64, 5_i64])) == "14"));
}
