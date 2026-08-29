// src/main.rs
use ::sifr_runtime::SifrInt;

fn keep_comparable<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(x: &T) -> T {
    x.clone()
}

fn relay_comparable<U: Clone + ::std::fmt::Display + PartialOrd + 'static>(x: &U) -> U {
    keep_comparable(x)
}

fn main() {
    println!("protocol_bounds protocol bound strictness closure demo:");
    println!("{}", relay_comparable(&SifrInt::from_i64(9)));
    println!("{}", relay_comparable(&"ok".to_string()));
}
