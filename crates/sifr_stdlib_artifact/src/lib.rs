//! Checked stdlib data shared by metadata services and native emission.
//! Rendering and syntax-session ownership belong to the emitter.

mod metadata;
mod rust_source;
pub use metadata::{StdlibCode, StdlibEmissionCode};
pub use rust_source::StdlibRustSource;

pub type FuncSignature = (
    Vec<(sifr_type_system::Type, sifr_type_system::ParamConvention)>,
    sifr_type_system::Type,
);
pub type ModuleFuncSignatures = std::collections::HashMap<String, FuncSignature>;

mod compiled_identity;
#[doc(hidden)]
pub use compiled_identity::compiled_input_tokens;
