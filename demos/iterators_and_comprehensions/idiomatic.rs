fn main() {
    let nums = [1_i64, 2, 3, 4, 5];

    let doubled: Vec<i64> = nums.iter().copied().map(|value| value * 2).collect();
    println!("{doubled:?}");

    let evens: Vec<i64> = nums
        .iter()
        .copied()
        .filter(|value| value % 2 == 0)
        .collect();
    println!("{evens:?}");

    let squares: Vec<i64> = nums.iter().copied().map(|value| value * value).collect();
    println!("{squares:?}");

    let big_squares: Vec<i64> = nums
        .iter()
        .copied()
        .filter(|value| *value > 2)
        .map(|value| value * value)
        .collect();
    println!("{big_squares:?}");

    let lo = nums.iter().copied().min();
    let hi = nums.iter().copied().max();
    if let Some(lo) = lo {
        println!("{lo}");
    }
    if let Some(hi) = hi {
        println!("{hi}");
    }
    println!("{}", nums.iter().copied().sum::<i64>());

    let unsorted = [5_i64, 3, 1, 4, 2];
    let mut sorted = unsorted.to_vec();
    sorted.sort_unstable();
    println!("{sorted:?}");

    let reversed: Vec<i64> = unsorted.iter().rev().copied().collect();
    println!("{reversed:?}");

    let letters = ["a", "b", "c"];
    let indexed: Vec<(i64, &str)> = letters
        .iter()
        .copied()
        .enumerate()
        .map(|(index, letter)| (index as i64, letter))
        .collect();
    println!("{indexed:?}");

    let names = ["Alice", "Bob"];
    let ages = [30_i64, 25];
    let paired: Vec<(&str, i64)> = names.iter().copied().zip(ages).collect();
    println!("{paired:?}");

    let bools = [true, false, true];
    println!("{}", bools.iter().copied().any(|value| value));
    println!("{}", bools.iter().copied().all(|value| value));
    println!("{}", [true, true, true].into_iter().all(|value| value));
}
