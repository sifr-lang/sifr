mod api;
mod module_lowering;

pub use api::{
    check, compile, compile_with_metadata, lower_source, parse_source, type_check_source,
};

pub(crate) use api::FrontendCompiled;
pub(crate) use module_lowering::{
    lower_frontend_module, FrontendDiagnosticStyle, FrontendModuleDiagnostics,
};
