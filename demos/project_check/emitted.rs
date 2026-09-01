// src/main.rs
pub mod helper;
use crate::helper::area_like;
fn main() {
    println!("project_check project-aware check parity demo:");
    println!("{}", area_like(3.0_f64));
}

// src/helper.rs
#[expect(
    clippy::approx_constant,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
pub const PI: f64 = 3.141_592_653_589_793_f64;
#[must_use]
pub fn area_like(r: f64) -> f64 {
    PI * r
}
