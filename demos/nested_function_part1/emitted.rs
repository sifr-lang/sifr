// src/main.rs
use ::sifr_runtime::SifrInt;
fn apply_twice(f: impl Fn(SifrInt) -> SifrInt, value: SifrInt) -> SifrInt {
    f(f(value.clone()))
}
fn score(base: SifrInt) -> SifrInt {
    let offset: SifrInt = SifrInt::from_i64(3);
    let add_offset = |x: SifrInt| &x + &offset;
    let amplify = |x: SifrInt| &x * &SifrInt::from_i64(2);
    let adjusted: SifrInt = apply_twice(add_offset, base.clone());
    amplify(adjusted.clone())
}
fn bounded_sum(limit: SifrInt) -> SifrInt {
    fn helper(i: SifrInt, acc: SifrInt, limit: SifrInt) -> SifrInt {
        if &i > &limit {
            return acc.clone();
        }
        helper(&i + &SifrInt::from_i64(1), &acc + &i, limit.clone())
    }
    helper(SifrInt::from_i64(1), SifrInt::from_i64(0), limit.clone())
}
fn main() {
    assert_eq!(&score(SifrInt::from_i64(4)), &SifrInt::from_i64(20));
    assert_eq!(&bounded_sum(SifrInt::from_i64(5)), &SifrInt::from_i64(15));
}
