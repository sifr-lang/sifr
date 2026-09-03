use crate::build::{
    generate_dependency_cargo_toml_with_interop, try_generate_sysroot_dependency_plan,
};
use crate::project::{rust_module_file_path, top_level_module_declarations};
use sifr_codegen::InteropBuildPlan;
#[cfg(test)]
use sifr_stdlib_manifest::try_sysroot_dependency_plan;
use sifr_stdlib_manifest::{CargoVendorMode, StdlibFeature, SysrootDependencyPlan};
use sifr_sysroot::SysrootError;
use std::collections::HashSet;
use std::path::PathBuf;

const SUPPORT_MAIN_RUST_FILE: &str = "__sifr_support_main.rs";

pub(crate) struct TestRunnerCargoPlan {
    pub(crate) cargo_toml: String,
    pub(crate) dependency_plan: SysrootDependencyPlan,
}

pub(crate) fn compose_test_runner_lib(
    support_module_names: &[String],
    all_rust_code: &str,
) -> String {
    let mut test_lib = String::from("#![cfg(test)]\n\n");
    for module_name in top_level_module_declarations(support_module_names) {
        if module_name == "main" && support_module_names.iter().any(|name| name == "main") {
            test_lib.push_str("#[path = \"");
            test_lib.push_str(SUPPORT_MAIN_RUST_FILE);
            test_lib.push_str("\"]\n");
        }
        test_lib.push_str("pub mod ");
        test_lib.push_str(&module_name);
        test_lib.push_str(";\n");
    }
    if !support_module_names.is_empty() {
        test_lib.push('\n');
    }
    test_lib.push_str(all_rust_code);
    test_lib
}

pub(crate) fn test_support_module_file_path(module_name: &str) -> PathBuf {
    if module_name == "main" {
        return PathBuf::from(SUPPORT_MAIN_RUST_FILE);
    }
    rust_module_file_path(module_name)
}

#[cfg(test)]
pub(crate) fn generate_test_runner_cargo_toml(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> String {
    let dependency_plan = try_sysroot_dependency_plan(
        stdlib_modules,
        required_features,
        CargoVendorMode::SysrootOnly,
    )
    .expect("test sysroot dependency plan should resolve");
    generate_dependency_cargo_toml_with_interop(
        "sifr_tests",
        &dependency_plan,
        &InteropBuildPlan::default(),
    )
}

pub(crate) fn try_generate_test_runner_cargo_plan(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> Result<TestRunnerCargoPlan, SysrootError> {
    let interop = InteropBuildPlan::default();
    let dependency_plan = try_generate_sysroot_dependency_plan(
        stdlib_modules,
        required_features,
        &interop,
        CargoVendorMode::SysrootOnly,
    )?;
    let cargo_toml =
        generate_dependency_cargo_toml_with_interop("sifr_tests", &dependency_plan, &interop);
    Ok(TestRunnerCargoPlan {
        cargo_toml,
        dependency_plan,
    })
}
