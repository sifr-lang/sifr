// src/main.rs
use ::sifr_runtime::SifrInt;
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn double(x: SifrInt) -> SifrInt {
    ::std::ops::Mul::mul(&x, &SifrInt::from_i64(2))
}
fn greet(name: &str) -> String {
    {
        let mut sifr_generated_concat: String =
            String::with_capacity(6usize.saturating_add(name.len()));
        sifr_generated_concat.push_str("hello ");
        sifr_generated_concat.push_str(name);
        sifr_generated_concat
    }
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn is_positive(x: SifrInt) -> bool {
    x > SifrInt::from_i64(0)
}
fn log_value(x: SifrInt) {
    println!("{x}");
}
fn main() {
    println!("{}", double(SifrInt::from_i64(21)));
    println!("{}", greet("sifr"));
    println!("{}", is_positive(SifrInt::from_i64(5)));
    log_value(SifrInt::from_i64(99));
}
