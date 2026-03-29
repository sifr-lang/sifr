fn sum_iterable(values: impl IntoIterator<Item = i64>) -> i64 {
    values.into_iter().sum()
}

#[allow(dead_code)]
fn passthrough<T>(it: T) -> T
where
    T: Iterator<Item = i64>,
{
    it
}

fn main() {
    let nums = vec![2_i64, 4, 6];
    println!("{}", sum_iterable(nums));
}
