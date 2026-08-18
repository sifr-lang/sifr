// src/main.rs
mod a_provider;
mod consumer;
mod z_provider;

use crate::consumer::joined;

fn main() {
    println!("module_assembly deterministic assembly demo:");
    println!("{}", joined());
}

// src/a_provider.rs
pub fn a() -> String {
    "A".to_string()
}

// src/consumer.rs
pub use crate::z_provider::z;
pub use crate::a_provider::a;
pub fn joined() -> String {
    {
        let mut __sifr_concat: String = String::with_capacity(
            (0usize + 1usize) + 0usize,
        );
        __sifr_concat.push_str((a()).as_str());
        __sifr_concat.push('-');
        __sifr_concat.push_str((z()).as_str());
        __sifr_concat
    }
}

// src/z_provider.rs
pub fn z() -> String {
    "Z".to_string()
}
