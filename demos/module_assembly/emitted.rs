// src/main.rs
pub mod a_provider;
pub mod consumer;
pub mod z_provider;
use crate::consumer::joined;
fn main() {
    println!("module_assembly deterministic assembly demo:");
    println!("{}", joined());
}

// src/a_provider.rs
#[must_use]
pub fn a() -> String {
    "A".to_string()
}

// src/consumer.rs
pub use crate::a_provider::a;
pub use crate::z_provider::z;
#[must_use]
pub fn joined() -> String {
    {
        let mut sifr_generated_concat: String = String::with_capacity(1usize);
        sifr_generated_concat.push_str(a().as_str());
        sifr_generated_concat.push('-');
        sifr_generated_concat.push_str(z().as_str());
        sifr_generated_concat
    }
}

// src/z_provider.rs
#[must_use]
pub fn z() -> String {
    "Z".to_string()
}
