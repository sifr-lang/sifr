mod discovery;
mod graph;
mod lowering;

pub(crate) use discovery::{
    discover_test_root_modules, parse_import_closure_modules, DiscoveryDiagnosticStyle,
};
pub(crate) use graph::{assemble_project_main_rs, ordered_non_main_module_names};
pub(crate) use lowering::{
    collect_project_hir_modules, compile_frontend_modules, emit_project_frontend_diagnostics,
    ProjectLowering,
};

#[cfg(test)]
pub(crate) use graph::compute_module_compile_order;
