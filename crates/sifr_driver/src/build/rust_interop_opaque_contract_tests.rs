use super::rust_interop::apply_package_rust_interop_metadata;
use super::rust_interop_contract_tests::{
    base_project_with_contracts, interop_errors, opaque_class_declaration_entry, package_context,
    set_bridge_roots, symbol_argument, target_argument, tokenizer_method_declaration_entry,
};
use sifr_ir::RustInteropDecoratorKind;
use sifr_package::TrustPolicy;
use std::path::PathBuf;

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
