fn sum_iterable(values: &Vec<i64>) -> i64 {
    let mut total: i64 = 0 as i64;
    for value in values.iter().copied() {
        total = total + value;
    }
    return total;
}

fn passthrough(it: Box<dyn Iterator<Item = i64>>) -> Box<dyn Iterator<Item = i64>> {
    return it;
}

fn main() {
    let nums: Vec<i64> = vec![2 as i64, 4 as i64, 6 as i64];
    println!(
        "{}",
        sum_iterable(&(nums).iter().copied().collect::<Vec<_>>())
    );
}
