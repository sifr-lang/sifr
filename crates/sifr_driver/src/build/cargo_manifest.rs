use sifr_codegen::{InteropBuildPlan, RustInteropResolvedRoot};
use sifr_stdlib_model::{
    try_sysroot_dependency_plan, CargoVendorMode, StdlibFeature, SysrootDependencyPlan,
};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use sifr_sysroot::SysrootError;

pub(crate) fn generate_dependency_cargo_toml_for_cache_key(
    project_name: &str,
    dependency_plan: &SysrootDependencyPlan,
    interop: &InteropBuildPlan,
) -> String {
    render_dependency_cargo_toml(project_name, dependency_plan, interop)
}

pub(crate) fn try_generate_sysroot_dependency_plan(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
    interop: &InteropBuildPlan,
    requested_vendor_mode: CargoVendorMode,
) -> Result<SysrootDependencyPlan, SysrootError> {
    let vendor_mode = if requested_vendor_mode == CargoVendorMode::PackageOwned
        || !rust_interop_path_dependencies(interop).is_empty()
    {
        CargoVendorMode::PackageOwned
    } else {
        CargoVendorMode::SysrootOnly
    };
    try_sysroot_dependency_plan(stdlib_modules, required_features, vendor_mode)
}

pub(crate) fn sysroot_cargo_config_args(dependency_plan: &SysrootDependencyPlan) -> Vec<String> {
    if dependency_plan.cargo_vendor_mode != CargoVendorMode::SysrootOnly {
        return Vec::new();
    }
    vec![
        "--config".to_string(),
        "source.crates-io.replace-with=\"sifr-vendor\"".to_string(),
        "--config".to_string(),
        format!(
            "source.sifr-vendor.directory={}",
            toml_quote_string(&dependency_plan.vendor_dir.display().to_string())
        ),
    ]
}

fn render_dependency_cargo_toml(
    project_name: &str,
    dependency_plan: &SysrootDependencyPlan,
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

    let interop_deps = rust_interop_path_dependencies(interop);
    let stdlib_deps = dependency_plan.cargo_dependency_lines();
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
    let path = toml_quote_string(&crate_root.display().to_string());
    if dependency_name == cargo_package_name {
        Some(format!("{dependency_name} = {{ path = {path} }}"))
    } else {
        Some(format!(
            "{dependency_name} = {{ package = \"{cargo_package_name}\", path = {path} }}"
        ))
    }
}

fn toml_quote_string(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            '\u{08}' => quoted.push_str("\\b"),
            '\u{0C}' => quoted.push_str("\\f"),
            ch if ch.is_control() => {
                push_unicode_escape(&mut quoted, u32::from(ch));
            }
            ch => quoted.push(ch),
        }
    }
    quoted.push('"');
    quoted
}

fn push_unicode_escape(output: &mut String, value: u32) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    output.push_str("\\u");
    for shift in [12, 8, 4, 0] {
        let index = ((value >> shift) & 0xF) as usize;
        output.push(char::from(HEX[index]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_stdlib_model::{SysrootCrate, SysrootCrateDependency};
    use std::path::PathBuf;

    #[test]
    fn sysroot_cargo_config_args_apply_vendor_for_sysroot_only_mode() {
        let dependency_plan = test_dependency_plan(
            CargoVendorMode::SysrootOnly,
            PathBuf::from("/opt/sifr sysroot/vendor"),
        );

        assert_eq!(
            sysroot_cargo_config_args(&dependency_plan),
            vec![
                "--config",
                "source.crates-io.replace-with=\"sifr-vendor\"",
                "--config",
                "source.sifr-vendor.directory=\"/opt/sifr sysroot/vendor\"",
            ]
        );
    }

    #[test]
    fn sysroot_cargo_config_args_quote_control_characters() {
        let dependency_plan = test_dependency_plan(
            CargoVendorMode::SysrootOnly,
            PathBuf::from("/opt/sifr\nsysroot/vendor"),
        );

        assert_eq!(
            sysroot_cargo_config_args(&dependency_plan)[3],
            "source.sifr-vendor.directory=\"/opt/sifr\\nsysroot/vendor\""
        );
    }

    #[test]
    fn sysroot_cargo_config_args_leave_package_owned_mode_alone() {
        let dependency_plan = test_dependency_plan(
            CargoVendorMode::PackageOwned,
            PathBuf::from("/opt/sifr/vendor"),
        );

        assert!(sysroot_cargo_config_args(&dependency_plan).is_empty());
    }

    #[test]
    fn dependency_plan_honors_sysroot_only_request() {
        let plan = try_generate_sysroot_dependency_plan(
            &HashSet::new(),
            &HashSet::new(),
            &InteropBuildPlan::default(),
            CargoVendorMode::SysrootOnly,
        )
        .expect("source-tree sysroot should resolve");

        assert_eq!(plan.cargo_vendor_mode, CargoVendorMode::SysrootOnly);
    }

    #[test]
    fn dependency_plan_honors_package_owned_request_without_interop_deps() {
        let plan = try_generate_sysroot_dependency_plan(
            &HashSet::new(),
            &HashSet::new(),
            &InteropBuildPlan::default(),
            CargoVendorMode::PackageOwned,
        )
        .expect("source-tree sysroot should resolve");

        assert_eq!(plan.cargo_vendor_mode, CargoVendorMode::PackageOwned);
    }

    #[test]
    fn generated_cargo_toml_renders_sysroot_crates_before_retained_deps() {
        let mut dependency_plan = test_dependency_plan(
            CargoVendorMode::SysrootOnly,
            PathBuf::from("/opt/sifr/vendor"),
        );
        dependency_plan.crates = vec![SysrootCrateDependency {
            krate: SysrootCrate::SifrStdlib,
            path: PathBuf::from("/opt/sifr/crates/sifr_stdlib"),
            features: ["json".to_string()].into_iter().collect(),
        }];
        dependency_plan.retained_direct_dependencies = vec![
            "serde_json = { version = \"1.0.149\", features = [\"preserve_order\"] }".to_string(),
        ];

        let cargo_toml = generate_dependency_cargo_toml_for_cache_key(
            "sifr_output",
            &dependency_plan,
            &InteropBuildPlan::default(),
        );

        assert_eq!(
            cargo_toml,
            r#"[package]
name = "sifr_output"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
sifr_stdlib = { path = "/opt/sifr/crates/sifr_stdlib", default-features = false, features = ["json"] }
serde_json = { version = "1.0.149", features = ["preserve_order"] }
"#
        );
    }

    fn test_dependency_plan(
        cargo_vendor_mode: CargoVendorMode,
        vendor_dir: PathBuf,
    ) -> SysrootDependencyPlan {
        SysrootDependencyPlan {
            sysroot_root: PathBuf::from("/opt/sifr"),
            toolchain_id: "0.0.0-test-x86_64-test".to_string(),
            sysroot_content_sha256: "content".to_string(),
            cargo_config: PathBuf::from("/opt/sifr/.cargo/config.toml"),
            vendor_dir,
            crates: Vec::new(),
            retained_direct_dependencies: Vec::new(),
            cargo_vendor_mode,
            cache_fingerprint: "fingerprint".to_string(),
        }
    }
}
