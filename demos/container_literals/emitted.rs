// src/main.rs
use ::std::collections::HashMap;

fn frequency_score(nums: &Vec<i64>) -> i64 {
    let mut counts: HashMap<i64, i64> = HashMap::from([]);
    for n in nums.iter().copied() {
        {
            let __assign_key = n;
            let __assign_value = (1_i64) + counts.get(&n).cloned().unwrap_or(0_i64);
            counts.insert(__assign_key, __assign_value);
        }
    }
    let mut score: i64 = 0_i64;
    for n in nums.iter().copied() {
        score += counts.get(&n).cloned().unwrap_or(0_i64);
    }
    score
}

fn main() {
    assert!((format!("{}", frequency_score(&vec![1_i64, 2_i64, 1_i64])) == "5"));
    assert!((format!("{}", frequency_score(&vec![4_i64, 4_i64, 4_i64])) == "9"));
}
