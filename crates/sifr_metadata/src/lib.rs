//! Private, versioned compiler metadata transport and bounded record storage.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

mod metadata;
pub use metadata::*;

mod compiled_identity;
#[doc(hidden)]
pub use compiled_identity::compiled_input_tokens;
