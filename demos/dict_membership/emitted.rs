// src/main.rs
use ::sifr_runtime::SifrInt;
use ::std::collections::HashMap;
fn guarded_lookup(table: &HashMap<SifrInt, SifrInt>, key: SifrInt) -> SifrInt {
    let Some(sifr_generated_checked_value_0) = table.get(&key) else {
        return -&SifrInt::from_i64(1);
    };
    let value: SifrInt = (*sifr_generated_checked_value_0).clone();
    value.clone()
}
fn expression_lookup(table: &HashMap<SifrInt, SifrInt>, base: SifrInt) -> SifrInt {
    let Some(sifr_generated_checked_value_2) = table.get(&(&base + &SifrInt::from_i64(1))) else {
        return -&SifrInt::from_i64(1);
    };
    let value: SifrInt = (*sifr_generated_checked_value_2).clone();
    value.clone()
}
fn sum_known_keys(table: &HashMap<SifrInt, SifrInt>, keys: &[SifrInt]) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for key in keys.iter().cloned() {
        if let Some(sifr_generated_checked_value_3) = table.get(&key) {
            total = &total + &(*sifr_generated_checked_value_3).clone();
        }
    }
    total.clone()
}
fn main() {
    let t: HashMap<SifrInt, SifrInt> = HashMap::from([
        (SifrInt::from_i64(1), SifrInt::from_i64(10)),
        (SifrInt::from_i64(2), SifrInt::from_i64(20)),
        (SifrInt::from_i64(4), SifrInt::from_i64(40)),
    ]);
    assert_eq!(
        &guarded_lookup(&t, SifrInt::from_i64(2)),
        &SifrInt::from_i64(20)
    );
    assert_eq!(
        &guarded_lookup(&t, SifrInt::from_i64(3)),
        &-SifrInt::from_i64(1)
    );
    assert_eq!(
        &expression_lookup(&t, SifrInt::from_i64(1)),
        &SifrInt::from_i64(20)
    );
    assert_eq!(
        &expression_lookup(&t, SifrInt::from_i64(2)),
        &-SifrInt::from_i64(1)
    );
    assert_eq!(
        &sum_known_keys(
            &t,
            &vec![
                SifrInt::from_i64(0),
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(5)
            ]
        ),
        &SifrInt::from_i64(30)
    );
}
