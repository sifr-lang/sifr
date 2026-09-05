fn accumulate(values: &[i64]) -> i64 {
    let mut total = 0;
    let mut apply = || {
        for value in values.iter().copied() {
            total += value;
        }
    };
    apply();
    total
}

fn main() {
    assert_eq!(accumulate(&[3, 1, 4, 1, 5]).to_string(), "14");
}
