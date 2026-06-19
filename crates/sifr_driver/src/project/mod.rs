mod assembly;
mod compile_order;
mod discovery;
mod frontend;
mod package_discovery;
mod rust_module_layout;

pub(crate) use assembly::{assemble_project_main_rs, ordered_non_main_module_names};
#[cfg(test)]
pub(crate) use discovery::parse_import_closure_modules;
pub(crate) use discovery::{
    discover_test_root_modules, parse_import_closure_source_modules, DiscoveryDiagnosticStyle,
    ModuleResolver, ParsedProjectModule,
};
#[cfg(test)]
pub(crate) use frontend::{collect_project_hir_modules, compile_frontend_modules};
pub(crate) use frontend::{
    collect_project_hir_source_modules, collect_project_hir_source_modules_with_options,
    compile_single_frontend_module_with_source_and_options, emit_project_frontend_diagnostics,
    ProjectLowering,
};
pub(crate) use package_discovery::parse_package_import_closure_source_modules;
pub(crate) use rust_module_layout::{
    namespace_module_files, rust_module_file_path, top_level_module_declarations,
};

#[cfg(test)]
pub(crate) use compile_order::compute_module_compile_order;
