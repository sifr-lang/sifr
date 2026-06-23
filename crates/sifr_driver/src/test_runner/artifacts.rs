use crate::build::{
    generate_dependency_cargo_toml_for_cache_key, try_generate_sysroot_dependency_plan,
};
use crate::project::top_level_module_declarations;
use sifr_codegen::InteropBuildPlan;
#[cfg(test)]
use sifr_stdlib_model::try_sysroot_dependency_plan;
use sifr_stdlib_model::{CargoVendorMode, StdlibFeature, SysrootDependencyPlan};
use sifr_sysroot::SysrootError;
use std::collections::HashSet;

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
    generate_dependency_cargo_toml_for_cache_key(
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
        generate_dependency_cargo_toml_for_cache_key("sifr_tests", &dependency_plan, &interop);
    Ok(TestRunnerCargoPlan {
        cargo_toml,
        dependency_plan,
    })
}
