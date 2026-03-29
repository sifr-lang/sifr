use std::collections::HashMap;

fn frequency_score(nums: &[i64]) -> i64 {
    let mut counts = HashMap::new();
    for n in nums.iter().copied() {
        *counts.entry(n).or_insert(0) += 1;
    }
    nums.iter()
        .copied()
        .map(|n| counts.get(&n).copied().unwrap_or(0))
        .sum()
}

fn main() {
    assert_eq!(frequency_score(&[1, 2, 1]), 5);
    assert_eq!(frequency_score(&[4, 4, 4]), 9);
}
