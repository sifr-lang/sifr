fn greater_than_two(value: i64) -> bool {
    value > 2
}

fn main() {
    let nums = [5_i64, 1, 3, 4];
    let flags = [false, true, false];

    println!("{}", flags.iter().copied().any(|flag| flag));
    println!(
        "{:?}",
        nums.iter()
            .copied()
            .filter(|value| greater_than_two(*value))
            .collect::<Vec<_>>()
    );

    let mut sorted = nums.to_vec();
    sorted.sort_unstable();
    println!("{sorted:?}");
}
