mod api;

pub use api::{
    check, compile, compile_sql_migration_source, compile_with_metadata, lower_source,
    parse_source, type_check_source,
};

pub(crate) use api::FrontendCompiled;
#[cfg(test)]
pub(crate) use sifr_frontend::FrontendDiagnosticStyle;
