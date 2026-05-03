mod api;
mod module_lowering;
mod parser_diagnostics;

pub use api::{
    check, compile, compile_with_metadata, lower_source, parse_source, type_check_source,
};

pub(crate) use api::FrontendCompiled;
#[cfg(test)]
pub(crate) use module_lowering::lower_frontend_module;
pub(crate) use module_lowering::{
    lower_frontend_module_with_source, reveal_type_diagnostics, warning_diagnostics,
    FrontendDiagnosticStyle, FrontendModuleDiagnostics, FrontendSourceContext,
};
pub(crate) use parser_diagnostics::parse_module_with_diagnostics;
