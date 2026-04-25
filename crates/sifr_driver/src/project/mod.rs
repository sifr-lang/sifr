mod assembly;
mod compile_order;
mod discovery;
mod exports;
mod frontend;
mod rust_module_layout;

pub(crate) use assembly::{assemble_project_main_rs, ordered_non_main_module_names};
pub(crate) use discovery::{
    discover_test_root_modules, parse_import_closure_modules, DiscoveryDiagnosticStyle,
    ModuleResolver,
};
pub(crate) use frontend::{
    collect_project_hir_modules, compile_frontend_modules, emit_project_frontend_diagnostics,
    ProjectLowering,
};
pub(crate) use rust_module_layout::{
    namespace_module_files, rust_module_file_path, top_level_module_declarations,
};

#[cfg(test)]
pub(crate) use compile_order::compute_module_compile_order;
