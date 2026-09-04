// src/main.rs
use ::sifr_runtime::SifrInt;
const fn inferred(flag: bool) -> SifrInt {
    if flag {
        return SifrInt::from_i64(1);
    }
    SifrInt::from_i64(2)
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn consume(n: SifrInt) -> SifrInt {
    ::std::ops::Add::add(&n, &SifrInt::from_i64(1))
}
fn main() {
    println!("unreachable_returns diagnostics and consumer integration demo:");
    println!("{}", consume(inferred(true)));
    println!("{}", consume(inferred(false)));
}
