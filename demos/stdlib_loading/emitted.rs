// src/main.rs
mod sifr_generated_generated_support {
    #[expect(
        clippy::approx_constant,
        reason = "generated Rust preserves this exact typed Sifr source contract"
    )]
    pub const PI: f64 = 3.141_592_653_589_793_f64;
}
use crate::sifr_generated_generated_support::PI;
fn main() {
    println!("{PI}");
}
