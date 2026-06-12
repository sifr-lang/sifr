mod api;

pub use api::{
    check, compile, compile_with_metadata, compile_with_metadata_allowing_http_transport_harness,
    lower_source, parse_source, type_check_source,
};

pub(crate) use api::FrontendCompiled;
#[cfg(test)]
pub(crate) use sifr_frontend::FrontendDiagnosticStyle;
