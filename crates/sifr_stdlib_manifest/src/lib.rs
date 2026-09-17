//! Compiler-side stdlib manifest and sysroot planning.
//!
//! Owns the source inventory, private declaration inventory, generated-project
//! dependency planning, and sysroot validation data used by the compiler.

mod features;
#[cfg(test)]
mod features_tests;
mod sources;
#[cfg(test)]
mod test_support;

pub use features::{
    CargoVendorMode, StdlibFeature, SysrootCrate, SysrootCrateDependency, SysrootDependencyPlan,
    feature_for_codegen_requirement, features_for_stdlib_module, planned_sifr_stdlib_features,
    sysroot_dependency_plan_with_sysroot, try_generated_cargo_dependencies,
    try_sysroot_dependency_plan,
};
pub use sources::{
    LoadedStdlibSource, LoadedStdlibSourceKind, PRIVATE_STDLIB_MODULES, STDLIB_SOURCES,
    StdlibSource, StdlibSourceInventoryError, load_stdlib_sources_from_sysroot,
    load_stdlib_tooling_sources_from_sysroot, validate_stdlib_source_inventory,
};

#[cfg(test)]
mod tests {
    use super::STDLIB_SOURCES;
    use crate::test_support::TestUnwrap as _;

    #[test]
    fn stdlib_source_inventory_contains_user_modules() {
        let json = STDLIB_SOURCES
            .iter()
            .find(|source| source.module == "sifr.json")
            .test_unwrap("sifr.json should be in the stdlib inventory");

        assert!(json.source.contains("from _sifr.json import"));
    }
}

mod compiled_identity;
#[doc(hidden)]
pub use compiled_identity::compiled_input_tokens;

/// Captured source identity excluding producer host/profile/toolchain configuration.
#[doc(hidden)]
pub fn portable_source_token() -> &'static str {
    env!("SIFR_PORTABLE_SOURCE_TOKEN")
}
