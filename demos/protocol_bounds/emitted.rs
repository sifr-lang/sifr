// src/main.rs
use ::sifr_runtime::SifrInt;
pub trait SifrGeneratedAdd: Sized {
    #[must_use]
    fn sifr_generated_add(self, rhs: Self) -> Self;
}
impl SifrGeneratedAdd for ::sifr_runtime::SifrInt {
    fn sifr_generated_add(self, rhs: Self) -> Self {
        self + rhs
    }
}
fn keep_comparable<T: Clone + 'static + PartialOrd>(x: &T) -> T {
    x.clone()
}
fn relay_comparable<U: Clone + 'static + PartialOrd>(x: &U) -> U {
    keep_comparable(x)
}
fn add_same<T: Clone + 'static + SifrGeneratedAdd>(left: &T, right: &T) -> T {
    SifrGeneratedAdd::sifr_generated_add(left.clone(), right.clone())
}
fn relay_add<U: Clone + 'static + SifrGeneratedAdd>(left: &U, right: &U) -> U {
    add_same(left, right)
}
fn main() {
    println!("protocol_bounds protocol bound strictness closure demo:");
    println!("{}", relay_comparable(&SifrInt::from_i64(9)));
    println!("{}", relay_comparable(&"ok".to_string()));
    println!(
        "{}",
        relay_add(&SifrInt::from_i64(4), &SifrInt::from_i64(5))
    );
    println!("{}", relay_add(&"sifr".to_string(), &" rust".to_string()));
}
