fn is_even(value: i64) -> bool {
    value % 2 == 0
}

fn sorted(values: &[i64], reverse: bool) -> Vec<i64> {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    if reverse {
        sorted.reverse();
    }
    sorted
}

fn main() {
    let nums = [1_i64, 2, 3, 4];

    let evens: Vec<i64> = nums
        .iter()
        .copied()
        .filter(|value| is_even(*value))
        .collect();
    println!("{evens:?}");

    let rev: Vec<i64> = nums.iter().rev().copied().collect();
    println!("{rev:?}");

    let indexed: Vec<(i64, i64)> = nums
        .iter()
        .copied()
        .enumerate()
        .map(|(index, value)| (index as i64 + 10, value))
        .collect();
    println!("{indexed:?}");

    println!("{}", nums.iter().copied().sum::<i64>());
    println!("{:?}", sorted(&nums, true));

    let collected: Vec<i64> = nums
        .iter()
        .copied()
        .filter(|value| is_even(*value))
        .collect();
    println!("{collected:?}");
}
