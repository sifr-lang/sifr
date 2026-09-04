// src/main.rs
use ::sifr_runtime::SifrInt;
use ::std::collections::HashMap;
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn guarded_lookup(table: &HashMap<SifrInt, SifrInt>, key: SifrInt) -> SifrInt {
    let Some(sifr_generated_checked_value_0) = table.get(&key) else {
        return ::std::ops::Neg::neg(&SifrInt::from_i64(1));
    };
    (*sifr_generated_checked_value_0).clone()
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn expression_lookup(table: &HashMap<SifrInt, SifrInt>, base: SifrInt) -> SifrInt {
    let Some(sifr_generated_checked_value_2) =
        table.get(&::std::ops::Add::add(&base, &SifrInt::from_i64(1)))
    else {
        return ::std::ops::Neg::neg(&SifrInt::from_i64(1));
    };
    (*sifr_generated_checked_value_2).clone()
}
fn sum_known_keys(table: &HashMap<SifrInt, SifrInt>, keys: &[SifrInt]) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    #[expect(
        clippy::explicit_iter_loop,
        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
    )]
    for key in keys.iter() {
        if let Some(sifr_generated_checked_value_3) = table.get(key) {
            total = ::std::ops::Add::add(&total, &(*sifr_generated_checked_value_3).clone());
        }
    }
    total
}
fn main() {
    let t: HashMap<SifrInt, SifrInt> = HashMap::from([
        (SifrInt::from_i64(1), SifrInt::from_i64(10)),
        (SifrInt::from_i64(2), SifrInt::from_i64(20)),
        (SifrInt::from_i64(4), SifrInt::from_i64(40)),
    ]);
    assert_eq!(
        guarded_lookup(&t, SifrInt::from_i64(2)),
        SifrInt::from_i64(20)
    );
    assert_eq!(
        guarded_lookup(&t, SifrInt::from_i64(3)),
        ::std::ops::Neg::neg(SifrInt::from_i64(1))
    );
    assert_eq!(
        expression_lookup(&t, SifrInt::from_i64(1)),
        SifrInt::from_i64(20)
    );
    assert_eq!(
        expression_lookup(&t, SifrInt::from_i64(2)),
        ::std::ops::Neg::neg(SifrInt::from_i64(1))
    );
    assert_eq!(
        sum_known_keys(
            &t,
            &[
                SifrInt::from_i64(0),
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(5)
            ]
        ),
        SifrInt::from_i64(30)
    );
}
