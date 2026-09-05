// src/main.rs
use ::sifr_runtime::SifrInt;
fn apply_twice(f: impl Fn(SifrInt) -> SifrInt, value: &SifrInt) -> SifrInt {
    f(f((*value).clone()))
}
fn score(base: &SifrInt) -> SifrInt {
    let offset: SifrInt = SifrInt::from_i64(3);
    let add_offset = |x: SifrInt| ::std::ops::Add::add(&x, &offset);
    let amplify = |x: SifrInt| ::std::ops::Mul::mul(&x, &SifrInt::from_i64(2));
    let adjusted: SifrInt = apply_twice(add_offset, base);
    amplify(adjusted)
}
fn bounded_sum(limit: &SifrInt) -> SifrInt {
    fn helper(i: &SifrInt, acc: &SifrInt, limit: &SifrInt) -> SifrInt {
        if i > limit {
            return (*acc).clone();
        }
        helper(
            &::std::ops::Add::add(i, &SifrInt::from_i64(1)),
            &::std::ops::Add::add(acc, i),
            limit,
        )
    }
    helper(&SifrInt::from_i64(1), &SifrInt::from_i64(0), limit)
}
fn main() {
    assert_eq!(score(&SifrInt::from_i64(4)), SifrInt::from_i64(20));
    assert_eq!(bounded_sum(&SifrInt::from_i64(5)), SifrInt::from_i64(15));
}
