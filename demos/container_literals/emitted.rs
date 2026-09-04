// src/main.rs
use ::sifr_runtime::SifrInt;
use ::std::collections::HashMap;
fn frequency_score(nums: &[SifrInt]) -> SifrInt {
    let mut counts: HashMap<SifrInt, SifrInt> = HashMap::from([]);
    #[expect(
        clippy::explicit_iter_loop,
        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
    )]
    for n in nums.iter() {
        {
            let sifr_generated_assign_value = ::std::ops::Add::add(
                &SifrInt::from_i64(1),
                &counts.get(n).cloned().unwrap_or(SifrInt::from_i64(0)),
            );
            {
                let sifr_generated_assign_key = n.clone();
                counts.insert(sifr_generated_assign_key, sifr_generated_assign_value);
            }
        }
    }
    let mut score: SifrInt = SifrInt::from_i64(0);
    #[expect(
        clippy::explicit_iter_loop,
        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
    )]
    for n in nums.iter() {
        score = ::std::ops::Add::add(
            &score,
            &counts.get(n).cloned().unwrap_or(SifrInt::from_i64(0)),
        );
    }
    score
}
fn main() {
    assert_eq!(
        frequency_score(&[
            SifrInt::from_i64(1),
            SifrInt::from_i64(2),
            SifrInt::from_i64(1)
        ])
        .to_string(),
        "5"
    );
    assert_eq!(
        frequency_score(&[
            SifrInt::from_i64(4),
            SifrInt::from_i64(4),
            SifrInt::from_i64(4)
        ])
        .to_string(),
        "9"
    );
}
