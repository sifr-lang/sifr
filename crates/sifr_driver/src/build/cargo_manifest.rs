use sifr_codegen::generate_project_with_deps_and_crates;
use sifr_hir::HirModule;
use std::collections::{HashMap, HashSet};

pub(crate) fn generate_dependency_cargo_toml(
    project_name: &str,
    stdlib_modules: &HashSet<String>,
    required_crates: &HashSet<String>,
) -> String {
    let (cargo_toml, _) = generate_project_with_deps_and_crates(
        &empty_hir_module(),
        project_name,
        stdlib_modules,
        required_crates,
    );
    cargo_toml
}

fn empty_hir_module() -> HirModule {
    HirModule {
        functions: vec![],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    }
}
