use std::collections::HashMap;

fn frequency_score(nums: &Vec<i64>) -> i64 {
    let mut counts: HashMap<i64, i64> = HashMap::new();
    for n in nums.iter().copied() {
        {
            let __assign_key = n;
            let __assign_value = (1 as i64) + counts.get(&n).cloned().unwrap_or(0 as i64);
            counts.insert(__assign_key, __assign_value);
        }
    }
    let mut score: i64 = 0 as i64;
    for n in nums.iter().copied() {
        score += counts.get(&n).cloned().unwrap_or(0 as i64);
    }
    return score;
}

fn main() {
    assert_eq!(
        frequency_score(&vec![1 as i64, 2 as i64, 1 as i64]),
        5 as i64
    );
    assert_eq!(
        frequency_score(&vec![4 as i64, 4 as i64, 4 as i64]),
        9 as i64
    );
}
