//! Native materialization and qualification over the shared metadata reader.
use sifr_metadata as wire;
pub use sifr_semantic_services::metadata::reader::{
    Decode, Decoder, StdlibNavigation, StdlibNavigationSymbol, decode_project_results, nominal,
};
use wire::Result;
mod provider;
mod qualification;
pub mod selection;
mod support;
mod support_projection;
mod templates;
pub use provider::Provider;
pub use qualification::qualify_development_metadata;
pub use selection::select;
