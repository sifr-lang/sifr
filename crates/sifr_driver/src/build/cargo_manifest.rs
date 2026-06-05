use sifr_stdlib::{generated_cargo_dependencies, StdlibFeature};
use std::collections::HashSet;

pub(crate) fn generate_dependency_cargo_toml(
    project_name: &str,
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> String {
    let mut cargo_toml = format!(
        r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2021"

[workspace]
"#
    );

    let deps = generated_cargo_dependencies(stdlib_modules, required_features);
    if !deps.is_empty() {
        cargo_toml.push_str("\n[dependencies]\n");
        for dep in &deps {
            cargo_toml.push_str(dep);
            cargo_toml.push('\n');
        }
    }

    cargo_toml
}
