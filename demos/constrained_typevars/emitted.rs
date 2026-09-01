// src/main.rs
use ::sifr_runtime::SifrInt;
fn echo<T: Clone + 'static>(x: &T) -> T {
    x.clone()
}
fn smallest<U: Clone + 'static + PartialOrd>(a: &U, b: &U) -> U {
    if *a < *b {
        return a.clone();
    }
    b.clone()
}
fn main() {
    println!("constrained_typevars typevar constraint enforcement demo:");
    println!("{}", echo(&SifrInt::from_i64(7)));
    println!("{}", echo(&"ok".to_string()));
    println!(
        "{}",
        smallest(&SifrInt::from_i64(10), &SifrInt::from_i64(3))
    );
}
