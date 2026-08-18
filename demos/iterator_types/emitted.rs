// src/main.rs
fn sum_iterable(values: &Vec<i64>) -> i64 {
    let mut total: i64 = 0_i64;
    for value in values.iter().copied() {
        total += value;
    }
    total
}

fn passthrough(it: Box<dyn Iterator<Item = i64>>) -> Box<dyn Iterator<Item = i64>> {
    it
}

fn main() {
    let nums: Vec<i64> = vec![2_i64, 4_i64, 6_i64];
    println!("{}", sum_iterable(&(nums).iter().copied().collect::<Vec<_>>()));
}
