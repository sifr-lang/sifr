//! Driver test and API reexports for the lower metadata reader.
#[cfg(test)]
use sifr_compiler_services::metadata::reader::Decoder;
pub(crate) use sifr_compiler_services::metadata::reader::Provider;
#[cfg(test)]
use sifr_compiler_services::metadata::reader::{Decode, selection};
pub use sifr_compiler_services::metadata::reader::{
    StdlibNavigation, StdlibNavigationSymbol, decode_project_results, qualify_development_metadata,
};
#[cfg(test)]
use sifr_sysroot::metadata as wire;
#[cfg(test)]
mod corpus_tests;
#[cfg(all(test, unix))]
mod generation_tests;
#[cfg(test)]
mod tests;
