use super::cargo_manifest::{
    generate_dependency_cargo_toml_with_interop, try_generate_sysroot_dependency_plan,
};
use super::project_codegen::GeneratedBinaryProject;
use super::rust_interop::{
    PackageRustInteropContext, RustInteropModuleSource, apply_package_rust_interop_metadata,
};
use super::sysroot_interop::attach_stdlib_rust_interop;
use crate::stdlib::{StdlibRustInterop, StdlibRustInteropModuleSource};
use sifr_codegen::{InteropBuildPlan, RustInteropResolvedRoot, RustInteropTrustRequirementKind};
use sifr_package::{
    CargoPackageId, PackageClassification, PackageSourceMap, PackageSourceRoot, RustInteropConfig,
    SifrEdition, SifrManifest, SifrPackageGraph, SifrPackageId, SifrPackageMetadata,
    SifrPackageName, TrustPolicy,
};
use sifr_stdlib_manifest::{CargoVendorMode, SysrootCrate};
use sifr_sysroot::{
    COMPILER_SIFR_VERSION, ResolvedSysroot, SUPPORTED_SYSROOT_SCHEMA_VERSION, SysrootManifest,
    SysrootPaths,
};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;

#[test]
fn private_stdlib_interop_resolves_sysroot_crate_target() {
    let interop = private_stdlib_interop(
        "@rust(sifr_stdlib.test_bridge.noop)\ndef noop() -> None:\n    pass\n",
    );

    let (generated, context) = attach_stdlib_rust_interop(base_project(), None, &interop);
    let generated =
        apply_package_rust_interop_metadata(generated, context).expect("sysroot interop resolves");

    assert_eq!(generated.interop.rust.resolved_targets.len(), 1);
    assert!(matches!(
        &generated.interop.rust.resolved_targets[0].root,
        RustInteropResolvedRoot::SysrootCrate {
            dependency_name,
            sysroot_root,
            ..
        } if dependency_name == "sifr_stdlib" && sysroot_root == "/opt/sifr"
    ));
    assert_eq!(generated.interop.rust.trust_requirements.len(), 1);
    assert!(generated.interop.rust.trust_requirements[0].trusted);
    assert_eq!(
        generated.interop.rust.trust_requirements[0].kind,
        RustInteropTrustRequirementKind::NoPanic
    );
    assert_eq!(
        generated.interop.rust.trust_requirements[0].required_entry,
        "sifr_stdlib.test_bridge.noop"
    );
    assert!(generated.interop.rust.cargo_inputs.is_some());
}

#[test]
fn private_stdlib_interop_rejects_omitted_policy_for_runtime_target() {
    let interop = private_stdlib_interop(
        "@rust(sifr_runtime.test_bridge.noop)\ndef noop() -> None:\n    pass\n",
    );

    let (generated, context) = attach_stdlib_rust_interop(base_project(), None, &interop);
    let diagnostics = match apply_package_rust_interop_metadata(generated, context) {
        Ok(_) => panic!("implicit sysroot no-panic policy must be limited to sifr_stdlib"),
        Err(diagnostics) => diagnostics,
    };

    assert_eq!(diagnostics[0].code, "SIFR-RUST-PANIC-0001");
    assert!(
        diagnostics[0]
            .message
            .contains("canonical private `sifr_stdlib.*` targets")
    );
}

#[test]
fn private_stdlib_interop_rejects_non_sysroot_target_root() {
    let interop = private_stdlib_interop(
        "@rust(user_backend.test_bridge.noop, panic=trusted_no_panic)\ndef noop() -> None:\n    pass\n",
    );

    let (generated, context) = attach_stdlib_rust_interop(base_project(), None, &interop);
    let diagnostics = match apply_package_rust_interop_metadata(generated, context) {
        Ok(_) => panic!("private stdlib target root must be canonical"),
        Err(diagnostics) => diagnostics,
    };

    assert_eq!(diagnostics[0].code, "SIFR-RUST-RESOLVE-0001");
    assert!(diagnostics[0].message.contains("canonical sysroot crate"));
    assert_eq!(
        diagnostics[0].spans[0].file.as_deref(),
        Some("/opt/sifr/stdlib/_sifr/test_bridge.sifr")
    );
}

#[test]
fn sysroot_interop_dependency_plan_keeps_sysroot_vendor_mode() {
    let interop = private_stdlib_interop(
        "@rust(sifr_stdlib.test_bridge.noop)\ndef noop() -> None:\n    pass\n",
    );
    let (generated, context) = attach_stdlib_rust_interop(base_project(), None, &interop);
    let generated =
        apply_package_rust_interop_metadata(generated, context).expect("sysroot interop resolves");

    let plan = try_generate_sysroot_dependency_plan(
        &HashSet::new(),
        &HashSet::new(),
        &generated.interop,
        CargoVendorMode::SysrootOnly,
    )
    .expect("source-tree sysroot resolves");

    assert_eq!(plan.cargo_vendor_mode, CargoVendorMode::SysrootOnly);
    assert!(
        plan.crates
            .iter()
            .any(|dependency| dependency.krate == SysrootCrate::SifrStdlib)
    );
    assert!(
        plan.cache_fingerprint
            .contains("[sysroot-interop-crates]\nsifr_stdlib\n")
    );
    let trust = generated
        .interop
        .rust
        .trust_requirements
        .iter()
        .find(|requirement| requirement.kind == RustInteropTrustRequirementKind::NoPanic)
        .expect("implicit sysroot policy records no-panic trust");
    assert!(trust.trusted);
    assert_eq!(trust.required_entry, "sifr_stdlib.test_bridge.noop");

    let cargo_toml =
        generate_dependency_cargo_toml_with_interop("sifr_output", &plan, &generated.interop);
    assert!(cargo_toml.contains("sifr_stdlib = { path = "));
    assert!(cargo_toml.contains("default-features = false"));
}

#[test]
fn merged_user_and_private_stdlib_interop_both_resolve() {
    let stdlib_interop = private_stdlib_interop(
        "@rust(sifr_stdlib.test_bridge.noop, panic=trusted_no_panic)\ndef noop() -> None:\n    pass\n",
    );
    let mut generated = base_project();
    generated.interop = user_interop(
        "app",
        "@rust(bridge.user_noop, panic=trusted_no_panic)\ndef user_noop() -> None:\n    pass\n",
    );

    let (generated, context) =
        attach_stdlib_rust_interop(generated, Some(user_context()), &stdlib_interop);
    let generated =
        apply_package_rust_interop_metadata(generated, context).expect("merged interop resolves");

    assert_eq!(generated.interop.rust.resolved_targets.len(), 2);
    assert!(
        generated
            .interop
            .rust
            .resolved_targets
            .iter()
            .any(|target| target.written_path == "bridge.user_noop")
    );
    assert!(
        generated
            .interop
            .rust
            .resolved_targets
            .iter()
            .any(|target| matches!(
                &target.root,
                RustInteropResolvedRoot::SysrootCrate {
                    dependency_name, ..
                } if dependency_name == "sifr_stdlib"
            ))
    );
}

#[test]
fn merged_user_and_private_stdlib_interop_keeps_user_trust_separate() {
    let stdlib_interop = private_stdlib_interop(
        "@rust(sifr_stdlib.test_bridge.noop, panic=trusted_no_panic)\ndef noop() -> None:\n    pass\n",
    );
    let mut generated = base_project();
    generated.interop = user_interop(
        "app",
        "@rust(bridge.user_noop, panic=trusted_no_panic)\ndef user_noop() -> None:\n    pass\n",
    );
    let mut context = user_context();
    let app_package_id = context
        .module_packages
        .get("app")
        .expect("app module should have a package")
        .clone();
    context
        .graph
        .packages
        .get_mut(&app_package_id)
        .expect("app package should exist")
        .manifest
        .trust = TrustPolicy::default();

    let (generated, context) =
        attach_stdlib_rust_interop(generated, Some(context), &stdlib_interop);
    let diagnostics = match apply_package_rust_interop_metadata(generated, context) {
        Ok(_) => panic!("sysroot trust must not satisfy user package no-panic trust"),
        Err(diagnostics) => diagnostics,
    };

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TRUST-0001");
    assert!(diagnostics[0].message.contains("bridge.user_noop"));
}

fn private_stdlib_interop(source: &str) -> StdlibRustInterop {
    let parsed =
        sifr_syntax::parse_module_raw(source, Some("/opt/sifr/stdlib/_sifr/test_bridge.sifr"))
            .expect("private declaration parses");
    assert!(parsed.has_valid_syntax());
    let lowered = sifr_lowering::lower_module_sysroot_private_declaration_with_externals(
        parsed.suite(),
        &sifr_lowering::ExternalDefs::default(),
    )
    .expect("private declaration lowers");
    let plan = sifr_codegen::interop_build_plan_for_named_modules([(
        Some("_sifr.test_bridge"),
        &lowered.module,
    )]);
    assert_eq!(plan.rust.declarations.len(), 1);
    StdlibRustInterop {
        plan,
        module_sources: HashMap::from([(
            "_sifr.test_bridge".to_string(),
            StdlibRustInteropModuleSource {
                source: source.to_string(),
                display_path: "/opt/sifr/stdlib/_sifr/test_bridge.sifr".to_string(),
            },
        )]),
        sysroot: Some(fake_sysroot()),
    }
}

fn user_interop(module_name: &str, source: &str) -> InteropBuildPlan {
    let parsed = sifr_syntax::parse_module_raw(source, Some("/ws/app/sifr/app.sifr"))
        .expect("user declaration parses");
    assert!(parsed.has_valid_syntax());
    let lowered = sifr_lowering::lower_module_sysroot_private_declaration_with_externals(
        parsed.suite(),
        &sifr_lowering::ExternalDefs::default(),
    )
    .expect("user declaration lowers");
    sifr_codegen::interop_build_plan_for_named_modules([(Some(module_name), &lowered.module)])
}

fn user_context() -> PackageRustInteropContext {
    let package_id = SifrPackageId("sifr-app@0.1.0#path".to_string());
    let cargo_package_id = CargoPackageId("path+file:///ws/app#sifr-app@0.1.0".to_string());
    let package = SifrPackageMetadata {
        package_id: package_id.clone(),
        cargo_package_id: cargo_package_id.clone(),
        cargo_package_name: "sifr-app".to_string(),
        cargo_version: "0.1.0".to_string(),
        cargo_source: None,
        package_root: PathBuf::from("/ws/app"),
        sifr_manifest: PathBuf::from("/ws/app/sifr.toml"),
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
            trust: TrustPolicy {
                rust_no_panic: vec!["bridge.user_noop".to_string()],
                ..TrustPolicy::default()
            },
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
            backend_crates: BTreeMap::from([(package_id.clone(), Vec::new())]),
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
                source:
                    "@rust(bridge.user_noop, panic=trusted_no_panic)\ndef user_noop() -> None:\n    pass\n"
                        .to_string(),
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

fn base_project() -> GeneratedBinaryProject {
    GeneratedBinaryProject {
        main_rs: "fn main() {}\n".to_string(),
        support_modules: BTreeMap::new(),
        used_stdlib_modules: HashSet::new(),
        required_features: HashSet::new(),
        interop: InteropBuildPlan::default(),
        cache_key_fragment: None,
        bridge_modules: Default::default(),
        python_runtime: None,
    }
}

fn fake_sysroot() -> ResolvedSysroot {
    let root = PathBuf::from("/opt/sifr");
    ResolvedSysroot {
        root: root.clone(),
        manifest: SysrootManifest {
            schema_version: SUPPORTED_SYSROOT_SCHEMA_VERSION,
            sifr_version: COMPILER_SIFR_VERSION.to_string(),
            target_triple: "test-target".to_string(),
            built_by_compiler_commit: "abcdef0".to_string(),
            sysroot_content_sha256: "0".repeat(64),
            cargo_lock_sha256: "1".repeat(64),
        },
        paths: SysrootPaths::from_root(&root),
        cargo_lock_content_sha256: "2".repeat(64),
    }
}
