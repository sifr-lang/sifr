//! Item lowering scaffolds for the IR lowering.

mod module_constants;
pub use module_constants::*;
#[cfg(test)]
#[path = "../tests/unit/lower_item/tests.rs"]
mod tests;
