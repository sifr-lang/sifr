fn tail_first(values: impl DoubleEndedIterator<Item = i64>) -> i64 {
    values.rev().next().unwrap_or(0)
}

fn main() {
    let nums = [10_i64, 20, 30];
    println!("{}", tail_first(nums.into_iter()));

    let tup = (4_i64, 5, 6);
    let tuple_values = [tup.0, tup.1, tup.2];
    let total: i64 = tuple_values.into_iter().sum();
    println!("{total}");

    let rev_tup: Vec<i64> = [tup.0, tup.1, tup.2].into_iter().rev().collect();
    println!("{rev_tup:?}");
}
