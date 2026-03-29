fn main() {
    let values = vec![1_i64, 2, 3, 4];
    let mut it = values.iter().copied();

    match it.next() {
        Some(first) => println!("{first}"),
        None => println!("None"),
    }

    let remaining_total: i64 = it.sum();
    println!("{remaining_total}");

    let pair_total: i64 = values
        .iter()
        .copied()
        .enumerate()
        .map(|(index, value)| index as i64 + value)
        .sum();
    println!("{pair_total}");
}
