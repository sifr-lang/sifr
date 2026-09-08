use sifr_codegen::{InteropBuildPlan, RustInteropResolvedRoot};
use sifr_stdlib_manifest::{
    CargoVendorMode, StdlibFeature, SysrootCrate, SysrootCrateDependency, SysrootDependencyPlan,
    try_sysroot_dependency_plan,
};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::Write as _;
use std::path::Path;

const SIFR_GIT_SOURCE: &str = "https://github.com/sifr-lang/sifr.git";

use sifr_sysroot::SysrootError;

pub fn generate_dependency_cargo_toml(
    project_name: &str,
    dependency_plan: &SysrootDependencyPlan,
) -> String {
    render_dependency_cargo_toml(project_name, dependency_plan, &InteropBuildPlan::default())
}

pub(crate) fn generate_dependency_cargo_toml_with_interop(
    project_name: &str,
    dependency_plan: &SysrootDependencyPlan,
    interop: &InteropBuildPlan,
) -> String {
    render_dependency_cargo_toml(project_name, dependency_plan, interop)
}

pub(crate) fn generate_portable_dependency_cargo_toml_with_interop(
    project_name: &str,
    dependency_plan: &SysrootDependencyPlan,
    interop: &InteropBuildPlan,
    sifr_revision: &str,
) -> Result<String, String> {
    render_portable_dependency_cargo_toml(project_name, dependency_plan, interop, sifr_revision)
}

pub fn try_generate_standalone_dependency_plan(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
    interop: &InteropBuildPlan,
) -> Result<SysrootDependencyPlan, SysrootError> {
    try_generate_sysroot_dependency_plan(
        stdlib_modules,
        required_features,
        interop,
        CargoVendorMode::SysrootOnly,
    )
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
    let mut plan = try_sysroot_dependency_plan(stdlib_modules, required_features, vendor_mode)?;
    add_sysroot_interop_crates(&mut plan, interop);
    Ok(plan)
}

pub fn sysroot_cargo_config_args(dependency_plan: &SysrootDependencyPlan) -> Vec<String> {
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
edition = "2024"

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

    if !interop_deps.is_empty() {
        if let Some(runtime) = dependency_plan
            .crates
            .iter()
            .find(|dependency| dependency.krate == SysrootCrate::SifrRuntime)
        {
            let source = toml_quote_string(SIFR_GIT_SOURCE);
            let path = toml_quote_string(&runtime.path.display().to_string());
            let _ = write!(
                cargo_toml,
                "\n[patch.{source}]\nsifr_runtime = {{ path = {path} }}\n"
            );
        }
    }

    cargo_toml
}

fn render_portable_dependency_cargo_toml(
    project_name: &str,
    dependency_plan: &SysrootDependencyPlan,
    interop: &InteropBuildPlan,
    sifr_revision: &str,
) -> Result<String, String> {
    if sifr_revision.len() != 40 || !sifr_revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(
            "portable generated Cargo projects require an exact 40-character Sifr revision"
                .to_string(),
        );
    }
    let mut cargo_toml = format!(
        r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2024"

[workspace]
"#
    );
    let mut dependencies = dependency_plan
        .crates
        .iter()
        .map(|dependency| portable_sysroot_dependency_line(dependency, sifr_revision))
        .collect::<Vec<_>>();
    dependencies.extend(dependency_plan.retained_direct_dependencies.iter().cloned());
    dependencies.extend(portable_rust_interop_dependencies(interop)?);
    if !dependencies.is_empty() {
        cargo_toml.push_str("\n[dependencies]\n");
        for dependency in dependencies {
            cargo_toml.push_str(&dependency);
            cargo_toml.push('\n');
        }
    }
    Ok(cargo_toml)
}

fn portable_sysroot_dependency_line(
    dependency: &SysrootCrateDependency,
    sifr_revision: &str,
) -> String {
    let package = dependency.krate.package_name();
    let mut fields = vec![
        format!("git = {}", toml_quote_string(SIFR_GIT_SOURCE)),
        format!("rev = {}", toml_quote_string(sifr_revision)),
        "default-features = false".to_string(),
    ];
    if !dependency.features.is_empty() {
        fields.push(format!(
            "features = [{}]",
            dependency
                .features
                .iter()
                .map(|feature| toml_quote_string(feature))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    format!("{package} = {{ {} }}", fields.join(", "))
}

fn portable_rust_interop_dependencies(interop: &InteropBuildPlan) -> Result<Vec<String>, String> {
    let mut dependencies = BTreeMap::new();
    for target in &interop.rust.resolved_targets {
        let fields = match &target.root {
            RustInteropResolvedRoot::DirectCargoDependency {
                dependency_name,
                cargo_package_name,
                cargo_version,
                cargo_source,
                ..
            }
            | RustInteropResolvedRoot::PackageBridge {
                dependency_name,
                cargo_package_name,
                cargo_version,
                cargo_source,
                ..
            } => Some((
                dependency_name,
                portable_dependency_line(
                    dependency_name,
                    cargo_package_name,
                    cargo_version,
                    cargo_source.as_deref(),
                )?,
            )),
            RustInteropResolvedRoot::SysrootCrate { .. }
            | RustInteropResolvedRoot::SelfMethod { .. } => None,
        };
        if let Some((name, line)) = fields {
            dependencies.insert(name.clone(), line);
        }
    }
    Ok(dependencies.into_values().collect())
}

fn portable_dependency_line(
    dependency_name: &str,
    cargo_package_name: &str,
    cargo_version: &str,
    cargo_source: Option<&str>,
) -> Result<String, String> {
    let package = if dependency_name == cargo_package_name {
        String::new()
    } else {
        format!("package = {}, ", toml_quote_string(cargo_package_name))
    };
    let Some(source) = cargo_source else {
        return Err(format!(
            "local Rust dependency `{dependency_name}` cannot be included in a portable generated project; publish it through an exact registry or Git source"
        ));
    };
    if source.starts_with("registry+") {
        return Ok(format!(
            "{dependency_name} = {{ {package}version = {} }}",
            toml_quote_string(&format!("={cargo_version}"))
        ));
    }
    let Some(git) = source.strip_prefix("git+") else {
        return Err(format!(
            "Rust dependency `{dependency_name}` uses unsupported Cargo source `{source}`"
        ));
    };
    let (location, revision) = git.rsplit_once('#').ok_or_else(|| {
        format!("Git dependency `{dependency_name}` has no exact locked revision")
    })?;
    let url = location.split('?').next().unwrap_or(location);
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!(
            "Git dependency `{dependency_name}` has no exact 40-character locked revision"
        ));
    }
    Ok(format!(
        "{dependency_name} = {{ {package}git = {}, rev = {} }}",
        toml_quote_string(url),
        toml_quote_string(revision)
    ))
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
            RustInteropResolvedRoot::PackageBridge {
                dependency_name,
                cargo_package_name,
                cargo_manifest_path,
                ..
            } => direct_dependency_line(dependency_name, cargo_package_name, cargo_manifest_path)
                .map(|line| (dependency_name.clone(), line)),
            RustInteropResolvedRoot::SysrootCrate { .. } => None,
            RustInteropResolvedRoot::SelfMethod { .. } => None,
        })
        .collect()
}

fn add_sysroot_interop_crates(
    dependency_plan: &mut SysrootDependencyPlan,
    interop: &InteropBuildPlan,
) {
    let mut added_crates = Vec::new();
    for krate in sysroot_interop_crates(interop) {
        if dependency_plan
            .crates
            .iter()
            .any(|dependency| dependency.krate == krate)
        {
            continue;
        }
        dependency_plan.crates.push(SysrootCrateDependency {
            krate,
            path: dependency_plan
                .sysroot_root
                .join("crates")
                .join(krate.package_name()),
            features: BTreeSet::new(),
        });
        added_crates.push(krate);
    }
    dependency_plan
        .crates
        .sort_by_key(|dependency| dependency.krate);
    if !added_crates.is_empty() {
        append_sysroot_interop_cache_fingerprint(dependency_plan, &added_crates);
    }
}

fn sysroot_interop_crates(interop: &InteropBuildPlan) -> BTreeSet<SysrootCrate> {
    let mut crates: BTreeSet<SysrootCrate> = interop
        .rust
        .resolved_targets
        .iter()
        .filter_map(|target| match &target.root {
            RustInteropResolvedRoot::SysrootCrate {
                dependency_name, ..
            } => match dependency_name.as_str() {
                "sifr_runtime" => Some(SysrootCrate::SifrRuntime),
                "sifr_stdlib" => Some(SysrootCrate::SifrStdlib),
                _ => None,
            },
            RustInteropResolvedRoot::DirectCargoDependency { .. }
            | RustInteropResolvedRoot::SelfMethod { .. } => None,
            RustInteropResolvedRoot::PackageBridge { .. } => None,
        })
        .collect();

    for signature in &interop.rust.bridge_contracts.signatures {
        for param in &signature.params {
            collect_sysroot_crates_from_bridge_type(&mut crates, &param.ty);
        }
        collect_sysroot_crates_from_bridge_type(&mut crates, &signature.return_type);
    }
    for bridge_type in &interop.rust.bridge_contracts.generated_types {
        collect_sysroot_crates_from_rust_path(&mut crates, &bridge_type.rust_type_path);
        for field in &bridge_type.fields {
            collect_sysroot_crates_from_rust_path(&mut crates, &field.rust_type);
        }
    }

    crates
}

fn collect_sysroot_crates_from_bridge_type(
    crates: &mut BTreeSet<SysrootCrate>,
    ty: &sifr_codegen::RustBridgeTypeContract,
) {
    for rust_type in [
        ty.rust_borrowed_type.as_deref(),
        ty.rust_owned_type.as_deref(),
        ty.rust_return_type.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        collect_sysroot_crates_from_rust_path(crates, rust_type);
    }
}

fn collect_sysroot_crates_from_rust_path(crates: &mut BTreeSet<SysrootCrate>, rust_type: &str) {
    if rust_type.contains("sifr_runtime::") {
        crates.insert(SysrootCrate::SifrRuntime);
    }
    if rust_type.contains("sifr_stdlib::") {
        crates.insert(SysrootCrate::SifrStdlib);
    }
}

fn append_sysroot_interop_cache_fingerprint(
    dependency_plan: &mut SysrootDependencyPlan,
    added_crates: &[SysrootCrate],
) {
    dependency_plan
        .cache_fingerprint
        .push_str("[sysroot-interop-crates]\n");
    for krate in added_crates {
        let path = dependency_plan
            .sysroot_root
            .join("crates")
            .join(krate.package_name());
        dependency_plan
            .cache_fingerprint
            .push_str(krate.fingerprint_key());
        dependency_plan.cache_fingerprint.push('\n');
        dependency_plan.cache_fingerprint.push_str("path=");
        dependency_plan
            .cache_fingerprint
            .push_str(&path.display().to_string());
        dependency_plan.cache_fingerprint.push('\n');
        dependency_plan.cache_fingerprint.push_str("features=\n");
    }
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
    use sifr_codegen::{
        RustBridgeContractPlan, RustGeneratedBridgeField, RustGeneratedBridgeType,
        RustGeneratedBridgeTypeKind, RustInteropOwner, RustInteropResolvedRoot,
        RustInteropResolvedTarget,
    };
    use sifr_stdlib_manifest::{SysrootCrate, SysrootCrateDependency};
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
    fn dependency_plan_includes_runtime_for_generated_bridge_int_fields() {
        let mut dependency_plan = test_dependency_plan(
            CargoVendorMode::SysrootOnly,
            PathBuf::from("/opt/sifr/vendor"),
        );
        let mut interop = InteropBuildPlan::default();
        interop.rust.bridge_contracts = RustBridgeContractPlan {
            signatures: Vec::new(),
            generated_types: vec![RustGeneratedBridgeType {
                module_name: Some("_sifr.json".to_string()),
                name: "JSONDecodeErrorBridge".to_string(),
                rust_type_path: "crate::__sifr_bridge::_sifr_json::JSONDecodeErrorBridge"
                    .to_string(),
                kind: RustGeneratedBridgeTypeKind::Error,
                supports_eq: true,
                fields: vec![RustGeneratedBridgeField {
                    name: "line".to_string(),
                    sifr_type: "int".to_string(),
                    rust_type: "::sifr_runtime::interop::SifrIntBridge".to_string(),
                }],
                variants: Vec::new(),
            }],
        };

        add_sysroot_interop_crates(&mut dependency_plan, &interop);

        assert!(
            dependency_plan
                .crates
                .iter()
                .any(|dependency| dependency.krate == SysrootCrate::SifrRuntime)
        );
    }

    #[test]
    fn generated_cargo_toml_includes_package_bridge_dependency_alias() {
        let mut dependency_plan = test_dependency_plan(
            CargoVendorMode::PackageOwned,
            PathBuf::from("/opt/sifr/vendor"),
        );
        dependency_plan.crates = vec![SysrootCrateDependency {
            krate: SysrootCrate::SifrRuntime,
            path: PathBuf::from("/opt/sifr/crates/sifr_runtime"),
            features: ["structural".to_string()].into_iter().collect(),
        }];
        let mut interop = InteropBuildPlan::default();
        interop.rust.resolved_targets = vec![RustInteropResolvedTarget {
            module_name: Some("main".to_string()),
            owner: RustInteropOwner::Function {
                name: "hash_bytes".to_string(),
            },
            written_path: "bridge.blake3.hash_bytes".to_string(),
            canonical_target_path: "main::hash_bytes".to_string(),
            root: RustInteropResolvedRoot::PackageBridge {
                package_id: "local-blake3-bridge@0.1.0#path".to_string(),
                dependency_name: "__sifr_bridge_package_local_blake3_bridge".to_string(),
                cargo_package_name: "local-blake3-bridge".to_string(),
                cargo_version: "0.1.0".to_string(),
                cargo_source: None,
                cargo_manifest_path: "/ws/local_blake3_bridge/Cargo.toml".to_string(),
                bridge_roots: vec!["src/bridges".to_string()],
            },
            span: Default::default(),
        }];

        let cargo_toml =
            generate_dependency_cargo_toml_with_interop("sifr_output", &dependency_plan, &interop);

        assert!(cargo_toml.contains(
            "__sifr_bridge_package_local_blake3_bridge = { package = \"local-blake3-bridge\", path = \"/ws/local_blake3_bridge\" }"
        ));
        assert!(cargo_toml.contains(
            "[patch.\"https://github.com/sifr-lang/sifr.git\"]\nsifr_runtime = { path = \"/opt/sifr/crates/sifr_runtime\" }"
        ));
    }

    #[test]
    fn generated_cargo_toml_renders_sysroot_crates_before_retained_glue_deps() {
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
            "serde_json = { version = \"1.0.151\", features = [\"preserve_order\"] }".to_string(),
        ];

        let cargo_toml = generate_dependency_cargo_toml_with_interop(
            "sifr_output",
            &dependency_plan,
            &InteropBuildPlan::default(),
        );

        assert_eq!(
            cargo_toml,
            r#"[package]
name = "sifr_output"
version = "0.1.0"
edition = "2024"

[workspace]

[dependencies]
sifr_stdlib = { path = "/opt/sifr/crates/sifr_stdlib", default-features = false, features = ["json"] }
serde_json = { version = "1.0.151", features = ["preserve_order"] }
"#
        );
    }

    #[test]
    fn portable_manifest_replaces_host_paths_with_exact_sources() {
        let mut dependency_plan = test_dependency_plan(
            CargoVendorMode::SysrootOnly,
            PathBuf::from("/host/sysroot/vendor"),
        );
        dependency_plan.crates = vec![SysrootCrateDependency {
            krate: SysrootCrate::SifrRuntime,
            path: PathBuf::from("/host/sysroot/crates/sifr_runtime"),
            features: BTreeSet::new(),
        }];
        let revision = "0123456789abcdef0123456789abcdef01234567";

        let manifest = generate_portable_dependency_cargo_toml_with_interop(
            "sifr_output",
            &dependency_plan,
            &InteropBuildPlan::default(),
            revision,
        )
        .expect("portable manifest should render");

        assert!(!manifest.contains("/host/"), "{manifest}");
        assert!(manifest.contains("git = \"https://github.com/sifr-lang/sifr.git\""));
        assert!(manifest.contains(&format!("rev = \"{revision}\"")));
    }

    fn test_dependency_plan(
        cargo_vendor_mode: CargoVendorMode,
        vendor_dir: PathBuf,
    ) -> SysrootDependencyPlan {
        SysrootDependencyPlan {
            stdlib_modules: BTreeSet::new(),
            required_features: BTreeSet::new(),
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
