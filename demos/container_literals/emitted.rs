// src/main.rs
use ::std::collections::HashMap;

use ::sifr_runtime::SifrInt;

fn frequency_score(nums: &Vec<SifrInt>) -> SifrInt {
    let mut counts: HashMap<SifrInt, SifrInt> = HashMap::from([]);
    for n in nums.iter().cloned() {
        {
            let __assign_value = &SifrInt::from_i64(1) + &counts.get(&n).cloned().unwrap_or(SifrInt::from_i64(0));
            {
                let __assign_key = n.clone();
                counts.insert(__assign_key, __assign_value);
            }
        }
    }
    let mut score: SifrInt = SifrInt::from_i64(0);
    for n in nums.iter().cloned() {
        score = &score + &counts.get(&n).cloned().unwrap_or(SifrInt::from_i64(0));
    }
    score.clone()
}

fn main() {
    assert!((format!("{}", frequency_score(&vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(1)])) == "5"));
    assert!((format!("{}", frequency_score(&vec![SifrInt::from_i64(4), SifrInt::from_i64(4), SifrInt::from_i64(4)])) == "9"));
}
