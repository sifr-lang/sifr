// src/main.rs
pub mod helper;
pub mod shared;
use crate::helper::value;
fn main() {
    println!("project_test_discovery project/test discovery parity behavior demo:");
    println!("{}", value());
}

// src/helper.rs
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub const fn value() -> SifrInt {
    crate::shared::sifr_generated_const_42415345()
}

// src/shared.rs
pub use ::sifr_runtime::SifrInt;
#[must_use]
pub const fn sifr_generated_const_42415345() -> SifrInt {
    SifrInt::from_i64(42)
}
