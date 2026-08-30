// src/main.rs
use ::sifr_runtime::SifrInt;

fn power_two(exp: SifrInt) -> SifrInt {
    fn helper(n: SifrInt) -> SifrInt {
        if &n == &SifrInt::from_i64(0) {
            return SifrInt::from_i64(1);
        }
        return &SifrInt::from_i64(2) * &helper(&n - &SifrInt::from_i64(1));
    }
    helper((exp).clone())
}

fn sum_to(limit: SifrInt) -> SifrInt {
    fn helper(i: SifrInt, acc: SifrInt, limit: SifrInt) -> SifrInt {
        if &i > &limit {
            return acc.clone();
        }
        return helper(&i + &SifrInt::from_i64(1), &acc + &i, limit.clone());
    }
    helper(SifrInt::from_i64(1), SifrInt::from_i64(0), limit.clone())
}

fn main() {
    assert!((&power_two(SifrInt::from_i64(10)) == &SifrInt::from_i64(1024)));
    assert!((&sum_to(SifrInt::from_i64(10)) == &SifrInt::from_i64(55)));
}
