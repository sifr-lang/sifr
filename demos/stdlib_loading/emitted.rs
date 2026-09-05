// src/main.rs
pub mod sifr_generated_generated_support {
    #[expect(
        clippy::approx_constant,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) const PI: f64 = 3.141_592_653_589_793_f64;
}
use crate::sifr_generated_generated_support::PI;
fn main() {
    println!("{PI}");
}
