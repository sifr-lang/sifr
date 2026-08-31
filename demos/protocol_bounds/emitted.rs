// src/main.rs
use ::sifr_runtime::SifrInt;

fn keep_comparable<T: Clone + 'static + PartialOrd>(x: &T) -> T {
    x.clone()
}

fn relay_comparable<U: Clone + 'static + PartialOrd>(x: &U) -> U {
    keep_comparable(x)
}

fn add_same<T: Clone + 'static + ::std::ops::Add<Output = T>>(left: &T, right: &T) -> T {
    left.clone() + right.clone()
}

fn relay_add<U: Clone + 'static + ::std::ops::Add<Output = U>>(left: &U, right: &U) -> U {
    add_same(left, right)
}

fn main() {
    println!("protocol_bounds protocol bound strictness closure demo:");
    println!("{}", relay_comparable(&SifrInt::from_i64(9)));
    println!("{}", relay_comparable(&"ok".to_string()));
    println!("{}", relay_add(&SifrInt::from_i64(4), &SifrInt::from_i64(5)));
}
