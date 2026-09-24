use super::{
    binary_project_cache_key, canonical_rust_module_path, materialize_binary_project_files,
    should_validate_native_link_evidence, sysroot_trusted_native_links, trusted_native_links,
    validate_native_link_evidence,
};
use crate::build::project_codegen::GeneratedBinaryProject;
use crate::build::python_runtime::PackagePythonRuntime;
use crate::build::validate_test_native_link_evidence;
use sifr_codegen::{
    InteropBuildPlan, RustInteropOwner, RustInteropPlan, RustInteropPlanDeclaration,
    RustInteropTrustRequirement, RustInteropTrustRequirementKind,
};
use sifr_ir::{
    RustInteropAbiRequirements, RustInteropDeclaration, RustInteropDecoratorKind,
    RustInteropEffect, RustTargetPath,
};
use sifr_stdlib_manifest::{CargoVendorMode, StdlibFeature, SysrootDependencyPlan};
use std::collections::{BTreeMap, BTreeSet, HashSet};

#[test]
fn generated_module_paths_are_relative_and_cannot_escape() {
    let bridge = canonical_rust_module_path(std::path::Path::new("__sifr_bridge/_sifr_fs.rs"));
    assert!(matches!(
        bridge.as_deref(),
        Ok(path) if path == std::path::Path::new("sifr_generated_bridge/sifr_generated_fs.rs")
    ));
    let public = canonical_rust_module_path(std::path::Path::new("public/mod.rs"));
    assert!(matches!(
        public.as_deref(),
        Ok(path) if path == std::path::Path::new("public/mod.rs")
    ));
    assert!(canonical_rust_module_path(std::path::Path::new("../escape.rs")).is_err());
    assert!(canonical_rust_module_path(std::path::Path::new("/escape.rs")).is_err());
}

#[test]
fn source_materialization_writes_a_complete_uncompiled_cargo_project() {
    let root = std::env::temp_dir().join(format!(
        "sifr_source_materialization_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    ));
    let project_path = root.join("sifr_output");

    materialize_binary_project_files(
        &project_path,
        "sifr_output",
        base_project(),
        &test_dependency_plan("fingerprint-a"),
    )
    .expect("source-only materialization should succeed");

    assert!(project_path.join("Cargo.toml").is_file());
    let build_script = std::fs::read_to_string(project_path.join("build.rs"))
        .expect("formatted native loader build script should be readable");
    assert!(
        build_script
            .replace("\r\n", "\n")
            .starts_with("fn main() {\n    println!(\"cargo:rerun-if-changed=build.rs\");\n"),
        "{build_script}"
    );
    assert!(build_script.contains("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN"));
    syn::parse_file(&build_script).expect("formatted native loader build script should parse");
    let main_rs = std::fs::read_to_string(project_path.join("src/main.rs"))
        .expect("generated main should be readable");
    assert!(main_rs.contains("fn main()"), "{main_rs}");
    assert!(!project_path.join("target").exists());

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn rematerialization_removes_stale_generated_sources_but_preserves_target() {
    let root = std::env::temp_dir().join(format!(
        "sifr_source_rematerialization_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    ));
    let project_path = root.join("sifr_output");
    let stale_source = project_path.join("src/obsolete/generated.rs");
    std::fs::create_dir_all(stale_source.parent().expect("stale source has a parent"))
        .expect("stale source directory should be writable");
    std::fs::write(&stale_source, "fn obsolete() {}\n").expect("stale source should be writable");
    let target_marker = project_path.join("target/cache-marker");
    std::fs::create_dir_all(target_marker.parent().expect("target marker has a parent"))
        .expect("target directory should be writable");
    std::fs::write(&target_marker, "preserve").expect("target marker should be writable");

    materialize_binary_project_files(
        &project_path,
        "sifr_output",
        base_project(),
        &test_dependency_plan("fingerprint-a"),
    )
    .expect("rematerialization should succeed");

    assert!(!stale_source.exists());
    assert!(target_marker.is_file());
    assert!(project_path.join("src/main.rs").is_file());
    let _ = std::fs::remove_dir_all(root);
}

/// This test runs in a child so SIFR_CACHE_DIR and a permissive umask cannot
/// affect other crate tests. Windows also verifies the native default owner.
#[test]
fn cache_owned_nested_generated_roots_survive_stale_cleanup_and_test_runner() {
    const CHILD: &str = "SIFR_PRIVATE_NESTED_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let scope = std::env::temp_dir().join(format!(
            "sifr-private-nested-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&scope).expect("test scope");
        let status = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "build::materialize::tests::cache_owned_nested_generated_roots_survive_stale_cleanup_and_test_runner",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .env("SIFR_CACHE_DIR", scope.join("cache"))
            .status()
            .expect("nested generated child");
        assert!(status.success(), "nested generated child failed: {status}");
        std::fs::remove_dir_all(scope).expect("test scope cleanup");
        return;
    }

    #[cfg(unix)]
    #[allow(unsafe_code)]
    unsafe {
        libc::umask(0o002);
    }
    #[cfg(windows)]
    assert!(
        crate::windows_storage_security::token_default_owner_is_administrators()
            .expect("TokenOwner"),
        "native Windows case requires Administrators TokenOwner"
    );

    let cache = crate::cache_storage::root();
    let scope = cache.join("nested-scope");
    crate::cache_storage::directory(&scope).expect("private scope");
    let family = crate::build::native_storage::NativeFamily::acquire(
        "nested-toolchain",
        "nested-sources",
        "",
        "nested-trust",
    )
    .expect("native family");
    let project_root = family
        .project(&scope, "nested-materialize")
        .expect("editable root");
    let mut first_project = base_project();
    first_project
        .support_modules
        .insert("nested.old".into(), "pub fn value() -> u32 { 41 }".into());
    materialize_binary_project_files(
        &project_root,
        "nested_materialize",
        first_project,
        &test_dependency_plan("nested"),
    )
    .expect("first cache materialization");
    let old = project_root.join("src/nested/old.rs");
    assert!(old.is_file());
    crate::cache_storage::check_owned(old.parent().expect("nested parent"))
        .expect("private generated parent");
    let mut second_project = base_project();
    second_project
        .support_modules
        .insert("nested.new".into(), "pub fn value() -> u32 { 42 }".into());
    materialize_binary_project_files(
        &project_root,
        "nested_materialize",
        second_project,
        &test_dependency_plan("nested"),
    )
    .expect("second cache materialization and stale cleanup");
    assert!(!old.exists());
    assert!(project_root.join("src/nested/new.rs").is_file());

    use crate::test_runner::GeneratedTestRunnerProject;
    let mut runner = GeneratedTestRunnerProject {
        application_profile: crate::ApplicationProfile::Test,
        cargo_resolution: crate::build::CargoResolutionPolicy::normal(),
        interop: sifr_codegen::InteropBuildPlan::default(),
        cache_scope: scope,
        support_module_names: vec!["nested.old".into()],
        support_rust_files: std::collections::HashMap::from([(
            "nested.old".into(),
            "pub fn value() -> u32 { 41 }".into(),
        )]),
        bridge_rust_files: Default::default(),
        all_rust_code: "#[test] fn nested_case() { assert_eq!(nested::old::value(), 41); }".into(),
        all_stdlib_modules: HashSet::new(),
        all_required_features: HashSet::new(),
    };
    let first =
        crate::test_runner::execute_test_runner_project(&runner).expect("first nested test runner");
    assert!(first.success);
    let runner_old = first.native_project_root.join("src/nested/old.rs");
    assert!(runner_old.is_file());
    crate::cache_storage::check_owned(runner_old.parent().expect("runner nested parent"))
        .expect("private runner parent");
    runner.support_module_names = vec!["nested.new".into()];
    runner.support_rust_files = std::collections::HashMap::from([(
        "nested.new".into(),
        "pub fn value() -> u32 { 42 }".into(),
    )]);
    runner.all_rust_code =
        "#[test] fn nested_case() { assert_eq!(nested::new::value(), 42); }".into();
    let second = crate::test_runner::execute_test_runner_project(&runner)
        .expect("second nested test runner and stale cleanup");
    assert!(second.success);
    assert_eq!(first.native_project_root, second.native_project_root);
    assert!(!runner_old.exists());
    assert!(
        second
            .native_project_root
            .join("src/nested/new.rs")
            .is_file()
    );
}

#[test]
fn binary_project_cache_key_includes_package_cache_fragment() {
    let base = base_project();
    let mut with_python_probe = GeneratedBinaryProject {
        cache_key_fragment: Some("python-probe-a".to_string()),
        ..base
    };
    let dependency_plan = test_dependency_plan("fingerprint-a");
    let first = binary_project_cache_key("sifr_output", &with_python_probe, &dependency_plan);
    with_python_probe.cache_key_fragment = Some("python-probe-b".to_string());
    let second = binary_project_cache_key("sifr_output", &with_python_probe, &dependency_plan);

    assert_ne!(first, second);
}

#[test]
fn binary_project_cache_key_includes_interop_build_plan() {
    let base = base_project();
    let mut with_interop = base_project();
    with_interop.interop = InteropBuildPlan {
        rust: RustInteropPlan {
            declarations: vec![RustInteropPlanDeclaration {
                module_name: Some("main".to_string()),
                owner: RustInteropOwner::Function {
                    name: "digest".to_string(),
                },
                declaration: RustInteropDeclaration {
                    kind: RustInteropDecoratorKind::Function,
                    target: Some(RustTargetPath {
                        segments: vec![
                            "bridge".to_string(),
                            "hash".to_string(),
                            "digest".to_string(),
                        ],
                        span: Default::default(),
                    }),
                    arguments: Vec::new(),
                    span: Default::default(),
                    effect: RustInteropEffect::Sync,
                    abi_requirements: RustInteropAbiRequirements::default(),
                    consumes_receiver: false,
                },
            }],
            ..RustInteropPlan::default()
        },
        ..InteropBuildPlan::default()
    };

    assert_ne!(
        binary_project_cache_key("sifr_output", &base, &test_dependency_plan("fingerprint-a")),
        binary_project_cache_key(
            "sifr_output",
            &with_interop,
            &test_dependency_plan("fingerprint-a")
        )
    );
}

#[test]
fn binary_project_cache_key_includes_sysroot_dependency_plan() {
    let base = base_project();

    assert_ne!(
        binary_project_cache_key("sifr_output", &base, &test_dependency_plan("fingerprint-a")),
        binary_project_cache_key("sifr_output", &base, &test_dependency_plan("fingerprint-b"))
    );
}

#[test]
fn binary_project_cache_key_uses_sysroot_dependency_plan_inputs() {
    let base = base_project();
    let mut dependency_plan = test_dependency_plan("fingerprint-a");
    dependency_plan.stdlib_modules = BTreeSet::from(["sifr.json".to_string()]);
    dependency_plan.required_features = BTreeSet::from([StdlibFeature::SerdeJson]);

    let cache_key = binary_project_cache_key("sifr_output", &base, &dependency_plan);

    assert_eq!(cache_key.len(), 64);
    dependency_plan.stdlib_modules.insert("sifr.math".into());
    assert_ne!(
        cache_key,
        binary_project_cache_key("sifr_output", &base, &dependency_plan)
    );
}

#[test]
fn native_link_evidence_rejects_untrusted_build_script_output() {
    let stdout = br#"{"reason":"build-script-executed","linked_libs":["dylib=ssl"]}"#;
    let diagnostics = validate_native_link_evidence(stdout, &BTreeSet::new())
        .expect_err("untrusted link evidence should fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TRUST-0001");

    let trusted = BTreeSet::from(["ssl".to_string()]);
    validate_native_link_evidence(stdout, &trusted).expect("trusted link should pass");
}

#[test]
fn native_link_evidence_policy_skips_non_rust_interop_projects() {
    let mut project = base_project();
    assert!(!should_validate_native_link_evidence(&project));

    project
        .interop
        .rust
        .trust_requirements
        .push(RustInteropTrustRequirement {
            canonical_target_path: "openssl::ssl".to_string(),
            kind: RustInteropTrustRequirementKind::NativeLinks,
            trusted: true,
            required_entry: "ssl".to_string(),
            evidence: "links=ssl".to_string(),
        });
    assert!(should_validate_native_link_evidence(&project));
}

#[test]
fn test_native_link_policy_uses_the_same_trust_contract_as_build() {
    let plan = test_dependency_plan("test-native-links");
    let mut interop = InteropBuildPlan::default();
    let untrusted = br#"{"reason":"build-script-executed","linked_libs":["dylib=ssl"]}"#;
    validate_test_native_link_evidence(untrusted, &interop, &plan)
        .expect("non-interop tests do not validate build-script links");
    interop
        .rust
        .trust_requirements
        .push(RustInteropTrustRequirement {
            canonical_target_path: "openssl::ssl".to_string(),
            kind: RustInteropTrustRequirementKind::NativeLinks,
            trusted: true,
            required_entry: "ssl".to_string(),
            evidence: "links=ssl".to_string(),
        });
    validate_test_native_link_evidence(untrusted, &interop, &plan)
        .expect("declared native link is trusted in tests");
    let unexpected = br#"{"reason":"build-script-executed","linked_libs":["dylib=zlib"]}"#;
    let diagnostics = validate_test_native_link_evidence(unexpected, &interop, &plan)
        .expect_err("undeclared native link must be rejected in tests");
    assert_eq!(diagnostics[0].code, "SIFR-RUST-TRUST-0001");
}

#[test]
fn python_runtime_libpython_link_is_trusted_when_interop_validation_runs() {
    let mut project = base_project();
    let mut python_runtime = PackagePythonRuntime::for_tests("/tmp/sifr-py/bin/python", "digest-a");
    python_runtime.set_libpython_for_tests("/opt/python/lib/libpython3.14.dylib");
    project.python_runtime = Some(python_runtime);
    project
        .interop
        .rust
        .trust_requirements
        .push(RustInteropTrustRequirement {
            canonical_target_path: "::sifr_stdlib::html::html_escape".to_string(),
            kind: RustInteropTrustRequirementKind::NativeLinks,
            trusted: true,
            required_entry: "ssl".to_string(),
            evidence: "links=ssl".to_string(),
        });

    let stdout = br#"{"reason":"build-script-executed","linked_libs":["dylib=python3.14"]}"#;
    validate_native_link_evidence(
        stdout,
        &trusted_native_links(&project, &test_dependency_plan("fingerprint-a")),
    )
    .expect("selected Python runtime link should be trusted");
}

#[test]
fn sysroot_tls_native_link_evidence_is_explicitly_trusted() {
    let mut dependency_plan = test_dependency_plan("fingerprint-a");
    dependency_plan
        .crates
        .push(sifr_stdlib_manifest::SysrootCrateDependency {
            krate: sifr_stdlib_manifest::SysrootCrate::SifrStdlib,
            path: "/sysroot/crates/sifr_stdlib".into(),
            features: BTreeSet::from(["tls".to_string()]),
        });

    let trusted = sysroot_trusted_native_links(&dependency_plan);
    assert_eq!(
        trusted,
        BTreeSet::from(["aws_lc_0_44_0_crypto".to_string()])
    );

    let stdout =
        br#"{"reason":"build-script-executed","linked_libs":["static=aws_lc_0_44_0_crypto"]}"#;
    validate_native_link_evidence(stdout, &trusted)
        .expect("sysroot-selected TLS provider link should pass");

    let untrusted = br#"{"reason":"build-script-executed","linked_libs":["static=crypto"]}"#;
    validate_native_link_evidence(untrusted, &trusted)
        .expect_err("unrelated native links must still fail");
}

#[test]
fn sysroot_http_native_link_evidence_inherits_tls_provider_trust() {
    let mut dependency_plan = test_dependency_plan("fingerprint-a");
    dependency_plan
        .crates
        .push(sifr_stdlib_manifest::SysrootCrateDependency {
            krate: sifr_stdlib_manifest::SysrootCrate::SifrStdlib,
            path: "/sysroot/crates/sifr_stdlib".into(),
            features: BTreeSet::from(["http".to_string()]),
        });

    let trusted = sysroot_trusted_native_links(&dependency_plan);
    assert_eq!(
        trusted,
        BTreeSet::from(["aws_lc_0_44_0_crypto".to_string()])
    );
}

pub(crate) fn base_project() -> GeneratedBinaryProject {
    GeneratedBinaryProject {
        main_rs: "fn main() {}\n".to_string(),
        support_modules: BTreeMap::new(),
        used_stdlib_modules: HashSet::new(),
        required_features: HashSet::new(),
        interop: InteropBuildPlan::default(),
        cache_key_fragment: None,
        bridge_modules: BTreeMap::new(),
        python_runtime: None,
    }
}

pub(crate) fn test_dependency_plan(cache_fingerprint: &str) -> SysrootDependencyPlan {
    SysrootDependencyPlan {
        stdlib_modules: BTreeSet::new(),
        required_features: BTreeSet::new(),
        sysroot_root: "/sysroot".into(),
        toolchain_id: "0.1.0-test-aarch64-test".to_string(),
        sysroot_content_sha256: "0".repeat(64),
        cargo_config: "/sysroot/.cargo/config.toml".into(),
        vendor_dir: "/sysroot/vendor".into(),
        crates: Vec::new(),
        retained_direct_dependencies: Vec::new(),
        cargo_vendor_mode: CargoVendorMode::SysrootOnly,
        cache_fingerprint: cache_fingerprint.to_string(),
    }
}

#[test]
fn native_source_identity_distinguishes_support_and_bridge_roles() {
    let mut support = base_project();
    support
        .support_modules
        .insert("same".into(), "pub fn same() {}".into());
    let mut bridge = base_project();
    bridge
        .bridge_modules
        .insert("same".into(), "pub fn same() {}".into());
    let plan = test_dependency_plan("fingerprint-a");
    assert_ne!(
        binary_project_cache_key("app", &support, &plan),
        binary_project_cache_key("app", &bridge, &plan)
    );
}

#[test]
fn python_native_loader_materialization_preserves_selection_and_removes_stale_path() {
    let root = std::env::temp_dir().join(format!(
        "sifr_python_native_loader_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let local = root.join("local");
    let portable = root.join("portable");
    let plan = test_dependency_plan("python-loader");
    let mut project = base_project();
    let mut python = PackagePythonRuntime::for_tests("/opt/python/bin/python", "probe");
    python.set_libpython_for_tests("/opt/a python/lib/libpython3.14.so");
    project.python_runtime = Some(python.clone());
    let first_key = binary_project_cache_key("sifr_output", &project, &plan);
    python.set_libpython_for_tests("/opt/another python/lib/libpython3.14.so");
    project.python_runtime = Some(python);
    assert_ne!(
        first_key,
        binary_project_cache_key("sifr_output", &project, &plan)
    );
    materialize_binary_project_files(&local, "sifr_output", project, &plan)
        .expect("Python project should materialize");
    let script = std::fs::read_to_string(local.join("build.rs")).expect("loader script");
    assert!(script.contains("\"/opt/another python/lib\""));
    assert!(script.contains("\"-Xlinker\", \"-rpath\", \"-Xlinker\""));
    std::fs::write(local.join("Cargo.lock"), "version = 4\n").expect("lock");
    crate::build::portable_project::publish_portable_project(&local, &portable)
        .expect("portable publication should preserve loader script");
    assert_eq!(
        std::fs::read_to_string(portable.join("build.rs")).expect("published loader script"),
        script
    );

    materialize_binary_project_files(&local, "sifr_output", base_project(), &plan)
        .expect("non-Python project should rematerialize");
    let loader = std::fs::read_to_string(local.join("build.rs")).expect("generic runtime loader");
    assert!(loader.contains("$ORIGIN"));
    assert!(!loader.contains("/opt/a python"));
    assert!(!loader.contains("/opt/another python"));
    crate::build::portable_project::publish_portable_project(&local, &portable)
        .expect("portable publication should replace the obsolete Python loader path");
    assert_eq!(
        std::fs::read_to_string(portable.join("build.rs")).expect("generic published loader"),
        loader
    );
    std::fs::remove_dir_all(root).expect("remove owned test fixture");
}
