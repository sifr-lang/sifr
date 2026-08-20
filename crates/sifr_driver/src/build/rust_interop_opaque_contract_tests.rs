use super::rust_interop::apply_package_rust_interop_metadata;
use super::rust_interop_contract_tests::{
    backend_with_manifest, base_project_with_contracts, bool_argument, interop_errors,
    opaque_class_declaration_entry, package_context, set_bridge_roots, symbol_argument,
    target_argument, temp_package_root, tokenizer_method_declaration_entry,
};
use sifr_ir::RustInteropDecoratorKind;
use sifr_package::TrustPolicy;
use std::path::PathBuf;

#[test]
#[doc = "sifr-evidence: executes-cargo-probe"]
fn package_rust_interop_opaque_probe_accepts_declared_send_sync_copy() {
    let backend_root = temp_package_root("rust_interop_opaque_send_sync_copy");
    std::fs::create_dir_all(backend_root.join("src")).expect("create backend src");
    std::fs::write(
        backend_root.join("Cargo.toml"),
        "[package]\nname = \"native\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write backend cargo toml");
    std::fs::write(
        backend_root.join("src/lib.rs"),
        "#[derive(Clone, Copy)]\npub struct Tokenizer(pub u64);\n",
    )
    .expect("write backend lib");

    let generated = base_project_with_contracts(
        vec![opaque_class_declaration_entry(vec![
            target_argument("type", "native.Tokenizer"),
            bool_argument("send", true),
            bool_argument("sync", true),
            symbol_argument("clone", "copy"),
        ])],
        Vec::new(),
    );
    let context = package_context(
        TrustPolicy::default(),
        vec![backend_with_manifest(
            "native",
            backend_root.join("Cargo.toml"),
        )],
    );

    let generated = apply_package_rust_interop_metadata(generated, Some(context))
        .expect("opaque Send + Sync + Copy type should pass probe");
    let probe = &generated.interop.rust.probe_plan.probes[0];
    assert_eq!(probe.kind, sifr_codegen::RustBridgeProbeKind::OpaqueHandle);
    assert!(probe.requires_send);
    assert!(probe.requires_sync);
}

#[test]
fn package_rust_interop_opaque_probe_rejects_unsatisfied_send_obligation() {
    let backend_root = temp_package_root("rust_interop_opaque_not_send");
    std::fs::create_dir_all(backend_root.join("src")).expect("create backend src");
    std::fs::write(
        backend_root.join("Cargo.toml"),
        "[package]\nname = \"native\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write backend cargo toml");
    std::fs::write(
        backend_root.join("src/lib.rs"),
        "pub struct Tokenizer(pub std::rc::Rc<()>);\n",
    )
    .expect("write backend lib");

    let generated = base_project_with_contracts(
        vec![opaque_class_declaration_entry(vec![
            target_argument("type", "native.Tokenizer"),
            bool_argument("send", true),
        ])],
        Vec::new(),
    );
    let context = package_context(
        TrustPolicy::default(),
        vec![backend_with_manifest(
            "native",
            backend_root.join("Cargo.toml"),
        )],
    );

    let diagnostics = interop_errors(generated, Some(context), "Send probe must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TYPE-0001");
    assert!(diagnostics[0].message.contains("Rust bridge probe failed"));
}

#[test]
fn package_rust_interop_opaque_probe_rejects_unsatisfied_copy_clone_policy() {
    let backend_root = temp_package_root("rust_interop_opaque_not_copy");
    std::fs::create_dir_all(backend_root.join("src")).expect("create backend src");
    std::fs::write(
        backend_root.join("Cargo.toml"),
        "[package]\nname = \"native\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write backend cargo toml");
    std::fs::write(
        backend_root.join("src/lib.rs"),
        "#[derive(Clone)]\npub struct Tokenizer(pub String);\n",
    )
    .expect("write backend lib");

    let generated = base_project_with_contracts(
        vec![opaque_class_declaration_entry(vec![
            target_argument("type", "native.Tokenizer"),
            symbol_argument("clone", "copy"),
        ])],
        Vec::new(),
    );
    let context = package_context(
        TrustPolicy::default(),
        vec![backend_with_manifest(
            "native",
            backend_root.join("Cargo.toml"),
        )],
    );

    let diagnostics = interop_errors(generated, Some(context), "Copy probe must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TYPE-0001");
    assert!(diagnostics[0].message.contains("Rust bridge probe failed"));
}

#[test]
#[doc = "sifr-evidence: executes-cargo-probe"]
fn package_rust_interop_same_path_mapping_failure_is_structured() {
    let backend_root = temp_package_root("rust_interop_same_path_mapping");
    std::fs::create_dir_all(backend_root.join("src")).expect("create backend src");
    std::fs::write(
        backend_root.join("Cargo.toml"),
        "[package]\nname = \"native\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write backend cargo toml");
    std::fs::write(backend_root.join("src/lib.rs"), "pub struct Tokenizer;\n")
        .expect("write backend lib");

    let generated = base_project_with_contracts(
        vec![opaque_class_declaration_entry(vec![
            target_argument("type", "native.Tokenizer"),
            target_argument("structural", "native.Tokenizer"),
        ])],
        Vec::new(),
    );
    let context = package_context(
        TrustPolicy::default(),
        vec![backend_with_manifest(
            "native",
            backend_root.join("Cargo.toml"),
        )],
    );

    let diagnostics = interop_errors(generated, Some(context), "mapping probe must fail");

    assert_eq!(
        diagnostics[0].code, "SIFR-RUST-TYPE-0001",
        "{diagnostics:#?}"
    );
    assert!(diagnostics[0].message.contains("Rust bridge probe failed"));
    assert!(diagnostics[0]
        .children
        .iter()
        .any(|child| child.message.contains("StructuralMapping")));
}

#[test]
fn package_rust_interop_opaque_close_policy_requires_close_method_contract() {
    let generated = base_project_with_contracts(
        vec![opaque_class_declaration_entry(vec![
            target_argument("type", "bridge.resources.Connection"),
            symbol_argument("close", "close"),
        ])],
        Vec::new(),
    );
    let mut context = package_context(TrustPolicy::default(), Vec::new());
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "missing close method must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-HANDLE-0001");
    assert!(diagnostics[0]
        .message
        .contains("requires `close` cleanup method"));
}

#[test]
fn package_rust_interop_opaque_async_close_policy_accepts_async_aclose_contract() {
    let generated = base_project_with_contracts(
        vec![
            opaque_class_declaration_entry(vec![
                target_argument("type", "bridge.resources.Connection"),
                symbol_argument("close", "async_close"),
            ]),
            tokenizer_method_declaration_entry("aclose", RustInteropDecoratorKind::Async),
        ],
        Vec::new(),
    );
    let mut context = package_context(TrustPolicy::default(), Vec::new());
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    apply_package_rust_interop_metadata(generated, Some(context))
        .expect("async aclose should satisfy async_close policy");
}

#[test]
fn package_rust_interop_opaque_async_close_policy_requires_owned_receiver() {
    let mut aclose = tokenizer_method_declaration_entry("aclose", RustInteropDecoratorKind::Async);
    aclose.declaration.consumes_receiver = false;
    let generated = base_project_with_contracts(
        vec![
            opaque_class_declaration_entry(vec![
                target_argument("type", "bridge.resources.Connection"),
                symbol_argument("close", "async_close"),
            ]),
            aclose,
        ],
        Vec::new(),
    );
    let mut context = package_context(TrustPolicy::default(), Vec::new());
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "borrowed aclose receiver must fail",
    );

    assert_eq!(diagnostics[0].code, "SIFR-RUST-HANDLE-0001");
    assert!(diagnostics[0]
        .message
        .contains("requires `aclose` cleanup method"));
}

#[test]
fn package_rust_interop_opaque_async_close_policy_requires_async_aclose_contract() {
    let generated = base_project_with_contracts(
        vec![
            opaque_class_declaration_entry(vec![
                target_argument("type", "bridge.resources.Connection"),
                symbol_argument("close", "async_close"),
            ]),
            tokenizer_method_declaration_entry("aclose", RustInteropDecoratorKind::Function),
        ],
        Vec::new(),
    );
    let mut context = package_context(TrustPolicy::default(), Vec::new());
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "sync aclose must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-HANDLE-0001");
    assert!(diagnostics[0]
        .message
        .contains("requires `aclose` cleanup method"));
}

#[test]
fn package_rust_interop_opaque_async_close_policy_rejects_sync_close_only_contract() {
    let generated = base_project_with_contracts(
        vec![
            opaque_class_declaration_entry(vec![
                target_argument("type", "bridge.resources.Connection"),
                symbol_argument("close", "async_close"),
            ]),
            tokenizer_method_declaration_entry("close", RustInteropDecoratorKind::Function),
        ],
        Vec::new(),
    );
    let mut context = package_context(TrustPolicy::default(), Vec::new());
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "sync close must not satisfy aclose",
    );

    assert_eq!(diagnostics[0].code, "SIFR-RUST-HANDLE-0001");
    assert!(diagnostics[0]
        .message
        .contains("requires `aclose` cleanup method"));
}
