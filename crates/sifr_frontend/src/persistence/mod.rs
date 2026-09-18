//! Persistence-facing contracts. No disk cache or alternate checker lives here.
mod observations;
pub use observations::*;
mod results;
pub use results::*;
mod diagnostics;
pub use diagnostics::*;

#[cfg(test)]
mod tests;
