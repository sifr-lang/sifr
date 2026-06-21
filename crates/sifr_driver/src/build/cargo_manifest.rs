use sifr_codegen::{InteropBuildPlan, RustInteropResolvedRoot};
use sifr_stdlib::{generated_cargo_dependencies, StdlibFeature};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

pub(crate) fn generate_dependency_cargo_toml(
    project_name: &str,
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
    interop: &InteropBuildPlan,
) -> String {
    let mut cargo_toml = format!(
        r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2021"

[workspace]
"#
    );

    let stdlib_deps = generated_cargo_dependencies(stdlib_modules, required_features);
    let interop_deps = rust_interop_path_dependencies(interop);
    if !stdlib_deps.is_empty() || !interop_deps.is_empty() {
        cargo_toml.push_str("\n[dependencies]\n");
        for dep in &stdlib_deps {
            cargo_toml.push_str(dep);
            cargo_toml.push('\n');
        }
        for dep in interop_deps.values() {
            cargo_toml.push_str(dep);
            cargo_toml.push('\n');
        }
    }

    cargo_toml
}

fn rust_interop_path_dependencies(interop: &InteropBuildPlan) -> BTreeMap<String, String> {
    interop
        .rust
        .resolved_targets
        .iter()
        .filter_map(|target| match &target.root {
            RustInteropResolvedRoot::DirectCargoDependency {
                dependency_name,
                cargo_package_name,
                cargo_manifest_path,
                ..
            } => direct_dependency_line(dependency_name, cargo_package_name, cargo_manifest_path)
                .map(|line| (dependency_name.clone(), line)),
            RustInteropResolvedRoot::PackageBridge { .. }
            | RustInteropResolvedRoot::SelfMethod { .. } => None,
        })
        .collect()
}

fn direct_dependency_line(
    dependency_name: &str,
    cargo_package_name: &str,
    cargo_manifest_path: &str,
) -> Option<String> {
    let crate_root = Path::new(cargo_manifest_path).parent()?;
    let path = toml_escape(&crate_root.display().to_string());
    if dependency_name == cargo_package_name {
        Some(format!("{dependency_name} = {{ path = \"{path}\" }}"))
    } else {
        Some(format!(
            "{dependency_name} = {{ package = \"{cargo_package_name}\", path = \"{path}\" }}"
        ))
    }
}

fn toml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
