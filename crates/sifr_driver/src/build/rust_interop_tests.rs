use super::project_codegen::GeneratedBinaryProject;
use super::rust_interop::{
    PackageRustInteropContext, RustInteropModuleSource, apply_package_rust_interop_metadata,
};
use super::rust_interop_test_support::span;
use crate::diagnostics::RenderedDiagnostic;
use sifr_codegen::{
    InteropBuildPlan, RustInteropOwner, RustInteropPlan, RustInteropPlanDeclaration,
};
use sifr_ir::{
    RustInteropAbiRequirements, RustInteropArgument, RustInteropDeclaration,
    RustInteropDecoratorKind, RustInteropEffect, RustInteropValue, RustTargetPath,
};
use sifr_package::{
    BackendCrateMetadata, CargoPackageId, PackageClassification, PackageSourceMap,
    PackageSourceRoot, RustInteropConfig, SifrEdition, SifrManifest, SifrPackageGraph,
    SifrPackageId, SifrPackageMetadata, SifrPackageName, TrustPolicy,
};
use sifr_stdlib_manifest::StdlibFeature;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;

#[path = "rust_interop_trust_tests.rs"]
mod trust_tests;

#[test]
fn package_rust_interop_requires_cargo_context() {
    let generated = base_project(vec![declaration_entry(
        "native.hash",
        RustInteropDecoratorKind::Function,
    )]);

    let diagnostics = interop_errors(generated, None, "missing package context must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-CARGO-0001");
}

#[test]
fn package_rust_interop_rejects_private_stdlib_impersonation() {
    let mut entry = declaration_entry("native.hash", RustInteropDecoratorKind::Function);
    entry.module_name = Some("_sifr.crypto".to_string());
    let generated = base_project(vec![entry]);
    let context = package_context(TrustPolicy::default(), vec![backend("native", false)]);

    let diagnostics = interop_errors(generated, Some(context), "private module must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-CARGO-0001");
    assert!(
        diagnostics[0]
            .message
            .contains("compiler-owned sysroot context")
    );
}

#[test]
fn package_rust_interop_rejects_unknown_target_root() {
    let generated = base_project(vec![declaration_entry(
        "missing.hash",
        RustInteropDecoratorKind::Function,
    )]);
    let context = package_context(TrustPolicy::default(), Vec::new());

    let diagnostics = interop_errors(generated, Some(context), "unknown root must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-RESOLVE-0001");
    assert!(diagnostics[0].message.contains("missing"));
    assert_eq!(
        diagnostics[0].spans[0].file.as_deref(),
        Some("/ws/app/sifr/app.sifr")
    );
}

#[test]
fn package_rust_interop_records_probe_plan() {
    let generated = base_project(vec![declaration_entry(
        "native.hash",
        RustInteropDecoratorKind::Function,
    )]);
    let context = package_context(
        TrustPolicy {
            rust_build_scripts: vec!["native".to_string()],
            ..TrustPolicy::default()
        },
        vec![BackendCrateMetadata {
            cargo_package_id: CargoPackageId("path+file:///ws/native#native@0.1.0".to_string()),
            dependency_name: "native".to_string(),
            dependency_kind: None,
            cargo_package_name: "native".to_string(),
            cargo_version: "0.1.0".to_string(),
            cargo_source: None,
            cargo_manifest_path: PathBuf::from("/ws/native/Cargo.toml"),
            links: None,
            has_build_script: true,
            has_proc_macro: false,
        }],
    );

    let generated = apply_package_rust_interop_metadata(generated, Some(context))
        .expect("trusted interop metadata should apply");

    assert_eq!(generated.interop.rust.resolved_targets.len(), 1);
    assert_eq!(generated.interop.rust.trust_requirements.len(), 1);
    assert!(generated.interop.rust.trust_requirements[0].trusted);
    assert_eq!(generated.interop.rust.probe_plan.probes.len(), 1);
    assert!(
        generated
            .interop
            .cache_key_fragment()
            .contains("rust.probes=1")
    );
    assert!(generated.interop.rust.cargo_inputs.is_some());
}

#[test]
fn package_rust_interop_resolves_bridge_root() {
    let generated = base_project(vec![declaration_entry(
        "bridge.hash",
        RustInteropDecoratorKind::Function,
    )]);
    let mut context = package_context(TrustPolicy::default(), Vec::new());
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let generated = apply_package_rust_interop_metadata(generated, Some(context))
        .expect("bridge root should resolve");

    assert!(matches!(
        generated.interop.rust.resolved_targets[0].root,
        sifr_codegen::RustInteropResolvedRoot::PackageBridge { .. }
    ));
    assert_eq!(generated.interop.rust.generated_bridge_modules.len(), 1);
    assert_eq!(
        generated.interop.rust.generated_bridge_modules[0].rust_module_path,
        ["__sifr_bridge".to_string(), "app".to_string()]
    );
    assert!(
        generated
            .interop
            .cache_key_fragment()
            .contains("__sifr_bridge_package_sifr_app")
    );
}

#[test]
fn package_rust_interop_injects_bridge_alias_into_declaring_module() {
    let mut generated = base_project(vec![declaration_entry(
        "bridge.hash",
        RustInteropDecoratorKind::Function,
    )]);
    generated
        .support_modules
        .insert("app".to_string(), "pub fn hash() {}\n".to_string());
    let mut context = package_context(TrustPolicy::default(), Vec::new());
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let generated = apply_package_rust_interop_metadata(generated, Some(context))
        .expect("bridge root should resolve");

    assert_eq!(
        generated.support_modules.get("app").map(String::as_str),
        Some("use __sifr_bridge_package_sifr_app::bridges as bridge;\npub fn hash() {}\n")
    );
}

#[test]
fn package_rust_interop_injects_one_bridge_alias_per_module() {
    let mut generated = base_project(vec![
        declaration_entry("bridge.hash", RustInteropDecoratorKind::Function),
        declaration_entry("bridge.codec", RustInteropDecoratorKind::Function),
    ]);
    generated
        .support_modules
        .insert("app".to_string(), "pub fn hash() {}\n".to_string());
    let mut context = package_context(TrustPolicy::default(), Vec::new());
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let generated = apply_package_rust_interop_metadata(generated, Some(context))
        .expect("bridge roots should resolve");

    let support_module = generated
        .support_modules
        .get("app")
        .expect("support module should remain");
    assert_eq!(
        support_module
            .matches("use __sifr_bridge_package_sifr_app::bridges as bridge;")
            .count(),
        1
    );
}

#[test]
fn package_rust_interop_rejects_removed_bridge_version_field() {
    let source = "[package]\nname = \"app\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n\n[rust]\nbridge-version = 1\nbridges = [\"src/bridges\"]\n";
    let diagnostic = SifrManifest::parse(
        &CargoPackageId("path+file:///ws/app#app@0.1.0".to_string()),
        &PathBuf::from("sifr.toml"),
        source,
    )
    .expect_err("the removed bridge-version field must fail");

    assert_eq!(
        diagnostic.code,
        sifr_diagnostics::DiagnosticCode::RUST_CARGO_METADATA
    );
    assert!(diagnostic.message.contains("bridge-version` was removed"));
}

#[test]
fn package_rust_interop_rejects_shared_bridge_crate_importing_generated_bridge_types() {
    let backend_root = temp_package_root("rust_interop_shared_bridge_boundary");
    std::fs::create_dir_all(backend_root.join("src")).expect("create backend src");
    std::fs::write(
        backend_root.join("Cargo.toml"),
        "[package]\nname = \"shared_bridge\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write backend cargo toml");
    std::fs::write(
        backend_root.join("src/lib.rs"),
        "use crate::__sifr_bridge::app::TokenBridge;\npub fn hash() {}\n",
    )
    .expect("write backend lib");
    let generated = base_project(vec![declaration_entry(
        "shared_bridge.hash",
        RustInteropDecoratorKind::Function,
    )]);
    let context = package_context(
        TrustPolicy::default(),
        vec![backend_with_manifest(
            "shared_bridge",
            false,
            backend_root.join("Cargo.toml"),
        )],
    );

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "shared bridge boundary violation must fail",
    );

    assert_eq!(diagnostics[0].code, "SIFR-RUST-RESOLVE-0001");
    assert!(diagnostics[0].message.contains("package-specific"));
}

#[test]
fn package_rust_interop_allows_shared_bridge_comments_about_generated_bridge_types() {
    let backend_root = temp_package_root("rust_interop_shared_bridge_comment");
    std::fs::create_dir_all(backend_root.join("src")).expect("create backend src");
    std::fs::write(
        backend_root.join("Cargo.toml"),
        "[package]\nname = \"shared_bridge\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write backend cargo toml");
    std::fs::write(
        backend_root.join("src/lib.rs"),
        "// crate::__sifr_bridge is package-local only\npub const NOTE: &str = \"__sifr_bridge::Token\";\npub fn hash() {}\n",
    )
    .expect("write backend lib");
    let generated = base_project(vec![declaration_entry(
        "shared_bridge.hash",
        RustInteropDecoratorKind::Function,
    )]);
    let context = package_context(
        TrustPolicy::default(),
        vec![backend_with_manifest(
            "shared_bridge",
            false,
            backend_root.join("Cargo.toml"),
        )],
    );

    apply_package_rust_interop_metadata(generated, Some(context))
        .expect("comments and strings must not violate shared bridge boundary");
}

#[test]
fn package_rust_interop_resolves_same_workspace_path_dependency() {
    let backend_root = temp_package_root("rust_interop_same_workspace_path");
    std::fs::create_dir_all(backend_root.join("src")).expect("create backend src");
    std::fs::write(
        backend_root.join("Cargo.toml"),
        "[package]\nname = \"workspace_backend\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write backend cargo toml");
    std::fs::write(backend_root.join("src/lib.rs"), "pub fn hash() {}\n")
        .expect("write backend lib");
    let generated = base_project(vec![declaration_entry(
        "workspace_backend.hash",
        RustInteropDecoratorKind::Function,
    )]);
    let context = package_context(
        TrustPolicy::default(),
        vec![backend_with_manifest(
            "workspace_backend",
            false,
            backend_root.join("Cargo.toml"),
        )],
    );

    apply_package_rust_interop_metadata(generated, Some(context))
        .expect("declared path dependency should resolve and probe");
}

#[test]
fn package_rust_interop_resolves_self_method_root() {
    let generated = base_project(vec![
        opaque_class_declaration_entry(),
        method_declaration_entry("Self.poll", RustInteropDecoratorKind::Function),
    ]);
    let context = package_context(TrustPolicy::default(), Vec::new());

    let generated = apply_package_rust_interop_metadata(generated, Some(context))
        .expect("Self method root should resolve");

    assert!(
        generated
            .interop
            .rust
            .resolved_targets
            .iter()
            .any(|target| matches!(
                target.root,
                sifr_codegen::RustInteropResolvedRoot::SelfMethod { .. }
            ))
    );
}

#[test]
fn package_rust_interop_rejects_self_method_root_without_opaque_class() {
    let generated = base_project(vec![method_declaration_entry(
        "Self.poll",
        RustInteropDecoratorKind::Function,
    )]);
    let context = package_context(TrustPolicy::default(), Vec::new());

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "Self root without opaque class should fail",
    );

    assert_eq!(diagnostics[0].code, "SIFR-RUST-RESOLVE-0001");
    assert!(
        diagnostics[0]
            .children
            .iter()
            .any(|child| child.message.contains("@rust.opaque"))
    );
}

#[test]
fn package_rust_interop_cache_fragment_is_deterministic() {
    let context = package_context(TrustPolicy::default(), vec![backend("native", false)]);

    let first = apply_package_rust_interop_metadata(
        base_project(vec![declaration_entry(
            "native.hash",
            RustInteropDecoratorKind::Function,
        )]),
        Some(context.clone()),
    )
    .expect("first metadata pass should apply")
    .interop
    .cache_key_fragment();
    let second = apply_package_rust_interop_metadata(
        base_project(vec![declaration_entry(
            "native.hash",
            RustInteropDecoratorKind::Function,
        )]),
        Some(context),
    )
    .expect("second metadata pass should apply")
    .interop
    .cache_key_fragment();

    assert_eq!(first, second);
}

#[test]
fn package_rust_interop_cache_changes_with_trust_policy() {
    let first = apply_package_rust_interop_metadata(
        base_project(vec![declaration_entry(
            "native.hash",
            RustInteropDecoratorKind::Function,
        )]),
        Some(package_context(
            TrustPolicy::default(),
            vec![backend("native", false)],
        )),
    )
    .expect("first metadata pass should apply")
    .interop
    .cache_key_fragment();
    let second = apply_package_rust_interop_metadata(
        base_project(vec![declaration_entry(
            "native.hash",
            RustInteropDecoratorKind::Function,
        )]),
        Some(package_context(
            TrustPolicy {
                build_env: vec!["OPENSSL_DIR".to_string()],
                ..TrustPolicy::default()
            },
            vec![backend("native", false)],
        )),
    )
    .expect("second metadata pass should apply")
    .interop
    .cache_key_fragment();

    assert_ne!(first, second);
}

#[test]
fn package_rust_interop_rejects_unrepresentable_probe_owner() {
    let mut entry = declaration_entry("native.hash", RustInteropDecoratorKind::Function);
    entry.owner = RustInteropOwner::Class {
        name: "Hash".to_string(),
    };
    let generated = base_project(vec![entry]);
    let context = package_context(TrustPolicy::default(), vec![backend("native", false)]);

    let diagnostics = interop_errors(generated, Some(context), "invalid probe owner must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TYPE-0001");
}

#[test]
fn package_rust_interop_maps_rustc_probe_resolution_failure() {
    let backend_root = temp_package_root("rust_interop_backend_probe");
    std::fs::create_dir_all(backend_root.join("src")).expect("create backend src");
    std::fs::write(
        backend_root.join("Cargo.toml"),
        "[package]\nname = \"native\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write backend cargo toml");
    std::fs::write(backend_root.join("src/lib.rs"), "pub fn hash() {}\n")
        .expect("write backend lib");

    let generated = base_project(vec![declaration_entry(
        "native.missing",
        RustInteropDecoratorKind::Function,
    )]);
    let context = package_context(
        TrustPolicy::default(),
        vec![backend_with_manifest(
            "native",
            false,
            backend_root.join("Cargo.toml"),
        )],
    );

    let diagnostics = interop_errors(generated, Some(context), "rustc probe must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-RESOLVE-0001");
    assert_eq!(
        diagnostics[0].spans[0].file.as_deref(),
        Some("/ws/app/sifr/app.sifr")
    );
}

#[test]
fn package_rust_interop_rejects_untrusted_no_panic_policy() {
    let generated = base_project(vec![declaration_entry_with_args(
        "native.hash",
        RustInteropDecoratorKind::Function,
        vec![symbol_arg("panic", "trusted_no_panic")],
    )]);
    let context = package_context(TrustPolicy::default(), vec![backend("native", false)]);

    let diagnostics = interop_errors(generated, Some(context), "untrusted no-panic must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TRUST-0001");
    assert!(diagnostics[0].message.contains("native.hash"));
}

#[test]
fn package_rust_interop_rejects_untrusted_panic_abort_policy() {
    let generated = base_project(vec![declaration_entry_with_args(
        "native.hash",
        RustInteropDecoratorKind::Function,
        vec![symbol_arg("panic", "abort")],
    )]);
    let context = package_context(TrustPolicy::default(), vec![backend("native", false)]);

    let diagnostics = interop_errors(generated, Some(context), "untrusted panic-abort must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TRUST-0001");
    assert!(
        diagnostics[0]
            .message
            .contains("missing Rust interop trust declaration")
    );
}

#[test]
fn package_rust_interop_rejects_untrusted_build_env() {
    let generated = base_project(vec![declaration_entry_with_args(
        "native.hash",
        RustInteropDecoratorKind::Function,
        vec![symbol_arg("build_env", "OPENSSL_DIR")],
    )]);
    let context = package_context(TrustPolicy::default(), vec![backend("native", false)]);

    let diagnostics = interop_errors(generated, Some(context), "untrusted build-env must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TRUST-0001");
    assert!(diagnostics[0].message.contains("native.hash"));
}

#[test]
fn package_rust_interop_records_lock_and_profile_cache_inputs() {
    let root = temp_package_root("rust_interop_cache_inputs");
    std::fs::create_dir_all(&root).expect("create package root");
    std::fs::write(root.join("Cargo.lock"), "version = 4\n").expect("write lock");
    std::fs::write(
        root.join("Cargo.toml"),
        "[profile.release]\nlto = true\ncodegen-units = 1\n",
    )
    .expect("write cargo toml");

    let generated = base_project(vec![declaration_entry(
        "native.hash",
        RustInteropDecoratorKind::Function,
    )]);
    let context = package_context_with_root(
        TrustPolicy::default(),
        vec![backend("native", false)],
        root.clone(),
    );

    let generated = apply_package_rust_interop_metadata(generated, Some(context))
        .expect("interop metadata should apply");
    let cargo_inputs = generated.interop.rust.cargo_inputs.expect("cargo inputs");

    assert!(cargo_inputs.cargo_lock_digest.is_some());
    assert_eq!(cargo_inputs.cargo_profile, "release");
    assert!(
        cargo_inputs
            .profile_codegen_settings
            .iter()
            .any(|(name, value)| name.ends_with(":lto") && value == "true")
    );
    assert!(cargo_inputs.target_triple.is_some());
}

fn base_project(declarations: Vec<RustInteropPlanDeclaration>) -> GeneratedBinaryProject {
    GeneratedBinaryProject {
        main_rs: "fn main() {}\n".to_string(),
        support_modules: BTreeMap::new(),
        used_stdlib_modules: HashSet::new(),
        required_features: HashSet::<StdlibFeature>::new(),
        interop: InteropBuildPlan {
            rust: RustInteropPlan {
                declarations,
                ..RustInteropPlan::default()
            },
            ..InteropBuildPlan::default()
        },
        cache_key_fragment: None,
        bridge_modules: Default::default(),
        python_runtime: None,
    }
}

fn interop_errors(
    generated: GeneratedBinaryProject,
    context: Option<PackageRustInteropContext>,
    message: &str,
) -> Vec<RenderedDiagnostic> {
    match apply_package_rust_interop_metadata(generated, context) {
        Ok(_) => panic!("{message}"),
        Err(diagnostics) => diagnostics,
    }
}

fn declaration_entry(target: &str, kind: RustInteropDecoratorKind) -> RustInteropPlanDeclaration {
    declaration_entry_with_args(
        target,
        kind,
        vec![RustInteropArgument {
            name: Some("trusted_no_panic".to_string()),
            value: RustInteropValue::Boolean(false),
            span: span(),
        }],
    )
}

fn method_declaration_entry(
    target: &str,
    kind: RustInteropDecoratorKind,
) -> RustInteropPlanDeclaration {
    let mut entry = declaration_entry(target, kind);
    entry.owner = RustInteropOwner::Method {
        class_name: "Consumer".to_string(),
        name: "poll".to_string(),
    };
    entry
}

fn opaque_class_declaration_entry() -> RustInteropPlanDeclaration {
    RustInteropPlanDeclaration {
        module_name: Some("app".to_string()),
        owner: RustInteropOwner::Class {
            name: "Consumer".to_string(),
        },
        declaration: RustInteropDeclaration {
            kind: RustInteropDecoratorKind::Opaque,
            target: None,
            arguments: vec![
                RustInteropArgument {
                    name: Some("type".to_string()),
                    value: RustInteropValue::TargetPath(target_path("bridge.consumer.Consumer")),
                    span: span(),
                },
                RustInteropArgument {
                    name: Some("send".to_string()),
                    value: RustInteropValue::Boolean(false),
                    span: span(),
                },
                RustInteropArgument {
                    name: Some("sync".to_string()),
                    value: RustInteropValue::Boolean(false),
                    span: span(),
                },
                symbol_arg("clone", "none"),
            ],
            span: span(),
            effect: RustInteropEffect::Sync,
            abi_requirements: RustInteropAbiRequirements::default(),
            consumes_receiver: false,
        },
    }
}

fn declaration_entry_with_args(
    target: &str,
    kind: RustInteropDecoratorKind,
    arguments: Vec<RustInteropArgument>,
) -> RustInteropPlanDeclaration {
    RustInteropPlanDeclaration {
        module_name: Some("app".to_string()),
        owner: RustInteropOwner::Function {
            name: "hash".to_string(),
        },
        declaration: RustInteropDeclaration {
            kind,
            target: Some(target_path(target)),
            arguments,
            span: span(),
            effect: RustInteropEffect::Sync,
            abi_requirements: RustInteropAbiRequirements::default(),
            consumes_receiver: false,
        },
    }
}

fn target_path(target: &str) -> RustTargetPath {
    RustTargetPath {
        segments: target.split('.').map(str::to_string).collect(),
        span: span(),
    }
}

fn symbol_arg(name: &str, value: &str) -> RustInteropArgument {
    RustInteropArgument {
        name: Some(name.to_string()),
        value: RustInteropValue::Symbol(value.to_string()),
        span: span(),
    }
}

fn package_context(
    trust: TrustPolicy,
    backend_crates: Vec<BackendCrateMetadata>,
) -> PackageRustInteropContext {
    package_context_with_root(trust, backend_crates, PathBuf::from("/ws/app"))
}

fn package_context_with_root(
    trust: TrustPolicy,
    backend_crates: Vec<BackendCrateMetadata>,
    package_root: PathBuf,
) -> PackageRustInteropContext {
    let package_id = SifrPackageId("sifr-app@0.1.0#path".to_string());
    let cargo_package_id = CargoPackageId("path+file:///ws/app#sifr-app@0.1.0".to_string());
    let sifr_manifest = package_root.join("sifr.toml");
    let package = SifrPackageMetadata {
        package_id: package_id.clone(),
        cargo_package_id: cargo_package_id.clone(),
        cargo_package_name: "sifr-app".to_string(),
        cargo_version: "0.1.0".to_string(),
        cargo_source: None,
        package_root,
        sifr_manifest,
        sifr_name: SifrPackageName("app".to_string()),
        manifest: SifrManifest {
            package_name: SifrPackageName("app".to_string()),
            edition: SifrEdition("2026".to_string()),
            compiler_requirement: sifr_package::CompilerRequirement(">=0.3,<0.4".to_string()),
            default_run: None,
            source_root: PackageSourceRoot(PathBuf::from("src")),
            source_features: BTreeMap::new(),
            scripts: BTreeMap::new(),
            dependencies: BTreeMap::new(),
            dev_dependencies: BTreeMap::new(),
            compiler_components: BTreeMap::new(),
            sql: sifr_package::SqlConfig::default(),
            trust,
            python: sifr_package::PythonConfig::default(),
            rust: RustInteropConfig {
                bridges: Vec::new(),
                direct_crate_bindings: true,
            },
        },
        aliases: BTreeMap::new(),
    };
    PackageRustInteropContext {
        package_id: package_id.clone(),
        graph: SifrPackageGraph {
            packages: BTreeMap::from([(package_id.clone(), package)]),
            cargo_edges: BTreeMap::new(),
            direct_dependency_scopes: BTreeMap::new(),
            backend_crates: BTreeMap::from([(package_id.clone(), backend_crates)]),
            classifications: BTreeMap::from([(
                cargo_package_id,
                PackageClassification::SifrSource(package_id.clone()),
            )]),
        },
        source_map: PackageSourceMap::default(),
        module_packages: HashMap::from([("app".to_string(), package_id)]),
        module_sources: HashMap::from([(
            "app".to_string(),
            RustInteropModuleSource {
                source: "@rust(native.hash)\ndef hash() -> None:\n    pass\n".to_string(),
                display_path: "/ws/app/sifr/app.sifr".to_string(),
            },
        )]),
        sysroot_runtime_crate: Some(test_runtime_crate()),
        sysroot_trust: None,
    }
}

fn test_runtime_crate() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("driver crate should have crates parent")
        .join("sifr_runtime")
}

fn backend(name: &str, has_build_script: bool) -> BackendCrateMetadata {
    backend_with_manifest(
        name,
        has_build_script,
        PathBuf::from(format!("/ws/{name}/Cargo.toml")),
    )
}

fn backend_with_manifest(
    name: &str,
    has_build_script: bool,
    cargo_manifest_path: PathBuf,
) -> BackendCrateMetadata {
    BackendCrateMetadata {
        cargo_package_id: CargoPackageId(format!("path+file:///ws/{name}#{name}@0.1.0")),
        dependency_name: name.to_string(),
        dependency_kind: None,
        cargo_package_name: name.to_string(),
        cargo_version: "0.1.0".to_string(),
        cargo_source: None,
        cargo_manifest_path,
        links: None,
        has_build_script,
        has_proc_macro: false,
    }
}

fn backend_custom(
    name: &str,
    has_build_script: bool,
    has_proc_macro: bool,
    links: Option<String>,
) -> BackendCrateMetadata {
    BackendCrateMetadata {
        has_proc_macro,
        links,
        ..backend(name, has_build_script)
    }
}

fn set_bridge_roots(context: &mut PackageRustInteropContext, bridges: Vec<PathBuf>) {
    let package = context
        .graph
        .packages
        .get_mut(&context.package_id)
        .expect("package exists");
    package.manifest.rust.bridges = bridges;
}

fn temp_package_root(name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let root = std::env::temp_dir().join(format!("sifr_{name}_{}_{nonce}", std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).expect("remove stale temp root");
    }
    root
}
