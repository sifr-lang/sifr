// src/main.rs
use ::sifr_runtime::SifrInt;

pub trait __SifrAdd: Sized {
    fn __sifr_add(self, rhs: Self) -> Self;
}

impl __SifrAdd for ::sifr_runtime::SifrInt {
    fn __sifr_add(self, rhs: Self) -> Self {
        self + rhs
    }
}

impl __SifrAdd for f64 {
    fn __sifr_add(self, rhs: Self) -> Self {
        self + rhs
    }
}

impl __SifrAdd for String {
    fn __sifr_add(mut self, rhs: Self) -> Self {
        self.push_str(&rhs);
        self
    }
}

fn keep_comparable<T: Clone + 'static + PartialOrd>(x: &T) -> T {
    x.clone()
}

fn relay_comparable<U: Clone + 'static + PartialOrd>(x: &U) -> U {
    keep_comparable(x)
}

fn add_same<T: Clone + 'static + __SifrAdd>(left: &T, right: &T) -> T {
    __SifrAdd::__sifr_add(left.clone(), right.clone())
}

fn relay_add<U: Clone + 'static + __SifrAdd>(left: &U, right: &U) -> U {
    add_same(left, right)
}

fn main() {
    println!("protocol_bounds protocol bound strictness closure demo:");
    println!("{}", relay_comparable(&SifrInt::from_i64(9)));
    println!("{}", relay_comparable(&"ok".to_string()));
    println!("{}", relay_add(&SifrInt::from_i64(4), &SifrInt::from_i64(5)));
    println!("{}", relay_add(&"sifr".to_string(), &" rust".to_string()));
}
