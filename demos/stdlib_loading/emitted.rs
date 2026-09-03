// src/main.rs
#[expect(
    clippy::approx_constant,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
const PI: f64 = 3.141_592_653_589_793_f64;
fn main() {
    println!("{PI}");
}
