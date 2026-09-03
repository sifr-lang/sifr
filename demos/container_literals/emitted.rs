// src/main.rs
use ::sifr_runtime::SifrInt;
use ::std::collections::HashMap;
fn frequency_score(nums: &[SifrInt]) -> SifrInt {
    let mut counts: HashMap<SifrInt, SifrInt> = HashMap::from([]);
    for n in nums.iter().cloned() {
        {
            let sifr_generated_assign_value =
                &SifrInt::from_i64(1) + &counts.get(&n).cloned().unwrap_or(SifrInt::from_i64(0));
            {
                let sifr_generated_assign_key = n.clone();
                counts.insert(sifr_generated_assign_key, sifr_generated_assign_value);
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
    assert_eq!(
        frequency_score(&vec![
            SifrInt::from_i64(1),
            SifrInt::from_i64(2),
            SifrInt::from_i64(1)
        ])
        .to_string(),
        "5"
    );
    assert_eq!(
        frequency_score(&vec![
            SifrInt::from_i64(4),
            SifrInt::from_i64(4),
            SifrInt::from_i64(4)
        ])
        .to_string(),
        "9"
    );
}
