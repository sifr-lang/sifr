// src/main.rs
mod helper;
mod shared;

use crate::helper::value;

fn main() {
    println!("project_test_discovery project/test discovery parity behavior demo:");
    println!("{}", value());
}

// src/helper.rs
pub use ::sifr_runtime::SifrInt;
pub fn value() -> SifrInt {
    crate::shared::__const_BASE()
}

// src/shared.rs
pub use ::sifr_runtime::SifrInt;
pub fn __const_BASE() -> SifrInt {
    SifrInt::from_i64(42)
}
