use crate::build::generate_dependency_cargo_toml;
use crate::project::top_level_module_declarations;
use sifr_stdlib::StdlibFeature;
use std::collections::HashSet;

pub(crate) fn compose_test_runner_lib(
    support_module_names: &[String],
    all_rust_code: &str,
) -> String {
    let mut test_lib = String::from("#![cfg(test)]\n\n");
    for module_name in top_level_module_declarations(support_module_names) {
        test_lib.push_str("mod ");
        test_lib.push_str(&module_name);
        test_lib.push_str(";\n");
    }
    if !support_module_names.is_empty() {
        test_lib.push('\n');
    }
    test_lib.push_str(all_rust_code);
    test_lib
}

pub(crate) fn generate_test_runner_cargo_toml(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> String {
    generate_dependency_cargo_toml("sifr_tests", stdlib_modules, required_features)
}
