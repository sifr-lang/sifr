// src/main.rs
use ::sifr_runtime::SifrInt;
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn power_two(exp: SifrInt) -> SifrInt {
    fn helper(n: &SifrInt) -> SifrInt {
        if n == &SifrInt::from_i64(0) {
            return SifrInt::from_i64(1);
        }
        ::std::ops::Mul::mul(
            &SifrInt::from_i64(2),
            &helper(&::std::ops::Sub::sub(n, &SifrInt::from_i64(1))),
        )
    }
    helper(&exp)
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn sum_to(limit: SifrInt) -> SifrInt {
    fn helper(i: &SifrInt, acc: &SifrInt, limit: &SifrInt) -> SifrInt {
        if i > limit {
            return acc.clone();
        }
        helper(
            &::std::ops::Add::add(i, &SifrInt::from_i64(1)),
            &::std::ops::Add::add(acc, i),
            &limit.clone(),
        )
    }
    helper(&SifrInt::from_i64(1), &SifrInt::from_i64(0), &limit)
}
fn main() {
    assert_eq!(power_two(SifrInt::from_i64(10)), SifrInt::from_i64(1024));
    assert_eq!(sum_to(SifrInt::from_i64(10)), SifrInt::from_i64(55));
}
