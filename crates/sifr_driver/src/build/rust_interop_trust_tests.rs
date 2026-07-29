use super::*;

#[test]
fn package_rust_interop_rejects_untrusted_build_script() {
    let generated = base_project(vec![declaration_entry(
        "native.hash",
        RustInteropDecoratorKind::Function,
    )]);
    let context = package_context(
        TrustPolicy::default(),
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

    let diagnostics = interop_errors(generated, Some(context), "untrusted build script must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TRUST-0001");
    assert!(diagnostics[0].message.contains("app.hash"));
    assert!(diagnostics[0]
        .children
        .iter()
        .any(|child| child.message.contains("[trust].rust-build-scripts")));
}

#[test]
fn package_rust_interop_attributes_package_trust_to_matching_declaration() {
    let mut bridge_declaration =
        declaration_entry("bridge.hash", RustInteropDecoratorKind::Function);
    bridge_declaration.owner = RustInteropOwner::Function {
        name: "bridge_call".to_string(),
    };
    let generated = base_project(vec![
        bridge_declaration,
        declaration_entry("native.hash", RustInteropDecoratorKind::Function),
    ]);
    let mut context = package_context(
        TrustPolicy::default(),
        vec![backend_custom("native", true, false, None)],
    );
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "untrusted build script must fail");

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("app.hash"));
    assert!(!diagnostics[0].message.contains("app.bridge_call"));
}

#[test]
fn package_rust_interop_rejects_untrusted_proc_macro_for_direct_root() {
    let generated = base_project(vec![declaration_entry(
        "serde_derive.hash",
        RustInteropDecoratorKind::Function,
    )]);
    let context = package_context(
        TrustPolicy::default(),
        vec![backend_custom("serde_derive", false, true, None)],
    );

    let diagnostics = interop_errors(generated, Some(context), "untrusted proc-macro must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TRUST-0001");
    assert!(diagnostics[0].message.contains("app.hash"));
    assert!(diagnostics[0]
        .children
        .iter()
        .any(|child| child.message.contains("[trust].rust-proc-macros")));
}

#[test]
fn package_rust_interop_rejects_untrusted_proc_macro_for_local_bridge() {
    let generated = base_project(vec![declaration_entry(
        "bridge.hash",
        RustInteropDecoratorKind::Function,
    )]);
    let mut context = package_context(
        TrustPolicy::default(),
        vec![backend_custom("serde_derive", false, true, None)],
    );
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "untrusted proc-macro must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TRUST-0001");
    assert!(diagnostics[0]
        .message
        .contains("missing Rust interop trust declaration"));
    let rendered = format!("{diagnostics:#?}");
    assert!(
        rendered.contains("proc-macro target")
            && rendered.contains("serde_derive")
            && rendered.contains("[trust].rust-proc-macros"),
        "proc-macro trust diagnostic must be kind-specific: {diagnostics:#?}"
    );
}

#[test]
fn package_rust_interop_rejects_untrusted_native_links() {
    let generated = base_project(vec![declaration_entry(
        "native.hash",
        RustInteropDecoratorKind::Function,
    )]);
    let context = package_context(
        TrustPolicy::default(),
        vec![backend_custom(
            "native",
            false,
            false,
            Some("ssl".to_string()),
        )],
    );

    let diagnostics = interop_errors(generated, Some(context), "untrusted native link must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TRUST-0001");
}

#[test]
fn package_rust_interop_rejects_untrusted_unsafe_bridge_file() {
    let root = temp_package_root("rust_interop_unsafe_bridge");
    let bridge_dir = root.join("src/bridges");
    std::fs::create_dir_all(&bridge_dir).expect("create bridge dir");
    std::fs::write(bridge_dir.join("hash.rs"), "pub unsafe fn hash() {}\n")
        .expect("write unsafe bridge");
    let generated = base_project(vec![declaration_entry(
        "bridge.hash",
        RustInteropDecoratorKind::Function,
    )]);
    let mut context = package_context_with_root(TrustPolicy::default(), Vec::new(), root.clone());
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "untrusted unsafe bridge must fail",
    );

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TRUST-0001");
}

#[test]
fn package_rust_interop_cache_changes_with_proc_macro_trust_policy() {
    let cache_fragment = |rust_proc_macros: Vec<String>| {
        apply_package_rust_interop_metadata(
            base_project(vec![declaration_entry(
                "native.hash",
                RustInteropDecoratorKind::Function,
            )]),
            Some(package_context(
                TrustPolicy {
                    rust_proc_macros,
                    ..TrustPolicy::default()
                },
                vec![backend("native", false)],
            )),
        )
        .expect("interop metadata should apply")
        .interop
        .cache_key_fragment()
    };

    assert_ne!(
        cache_fragment(Vec::new()),
        cache_fragment(vec!["serde_derive".to_string()])
    );
}
