fn accumulate(values: &Vec<i64>) -> i64 {
    let mut total: i64 = 0 as i64;
    let mut apply = || {
    for value in values.iter().copied() {
        total += value;
    }
};
    apply();
    return total;
}

fn main() {
    assert!(format!("{}", accumulate(&vec![3 as i64, 1 as i64, 4 as i64, 1 as i64, 5 as i64])) == "14".to_string());
}
