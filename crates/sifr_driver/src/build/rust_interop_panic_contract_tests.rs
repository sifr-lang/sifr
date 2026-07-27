use super::rust_interop_contract_tests::{
    base_project_with_contracts, declaration_entry_with_arguments, interop_errors, package_context,
    result_contract, set_bridge_roots, signature_contract, string_contract, symbol_argument,
    temp_package_root,
};
use ruff_text_size::TextRange;
use sifr_codegen::{RustBridgeTypeContract, RustBridgeTypeKind};
use sifr_ir::{
    RustInteropArgument, RustInteropDecoratorKind, RustInteropEffect, RustInteropValue,
    RustTargetPath,
};
use sifr_package::TrustPolicy;
use sifr_type_system::Type;
use std::path::PathBuf;

#[test]
fn package_rust_interop_result_requires_panic_surface() {
    let generated = base_project_with_contracts(
        vec![declaration_entry_with_arguments(
            "bridge.hash.digest",
            RustInteropDecoratorKind::Function,
            Vec::new(),
        )],
        vec![panic_result_signature(
            "HashError",
            declared_error_type("HashError"),
        )],
    );
    let context = bridge_context(TrustPolicy::default());

    let diagnostics = interop_errors(generated, Some(context), "missing panic surface must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-PANIC-0001");
    assert!(diagnostics[0].message.contains("RustPanicError"));
}

#[test]
fn package_rust_interop_result_accepts_rust_panic_error_surface() {
    let generated = base_project_with_contracts(
        vec![declaration_entry_with_arguments(
            "bridge.hash.digest",
            RustInteropDecoratorKind::Function,
            Vec::new(),
        )],
        vec![panic_result_signature(
            "HashError | RustPanicError",
            declared_error_union("HashError"),
        )],
    );
    let context = bridge_context(TrustPolicy::default());

    super::rust_interop::apply_package_rust_interop_metadata(generated, Some(context))
        .expect("RustPanicError in Result error channel should satisfy panic surface");
}

#[test]
fn package_rust_interop_result_accepts_map_error_surface() {
    let generated = base_project_with_contracts(
        vec![declaration_entry_with_arguments(
            "bridge.hash.digest",
            RustInteropDecoratorKind::Function,
            vec![map_error_argument("bridge.hash.map_panic")],
        )],
        vec![panic_result_signature(
            "HashError | RustPanicError",
            declared_error_union("HashError"),
        )],
    );
    let context = bridge_context(TrustPolicy::default());

    super::rust_interop::apply_package_rust_interop_metadata(generated, Some(context))
        .expect("map_error adapter with a redacted fallback should satisfy Result panic surface");
}

#[test]
fn package_rust_interop_rejects_map_error_without_representable_fallback() {
    let generated = base_project_with_contracts(
        vec![declaration_entry_with_arguments(
            "bridge.hash.digest",
            RustInteropDecoratorKind::Function,
            vec![map_error_argument("bridge.hash.map_panic")],
        )],
        vec![panic_result_signature(
            "HashError",
            declared_error_type("HashError"),
        )],
    );
    let context = bridge_context(TrustPolicy::default());

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "map_error without a redacted fallback must fail",
    );

    assert_eq!(diagnostics[0].code, "SIFR-RUST-PANIC-0001");
    assert!(diagnostics[0]
        .message
        .contains("representable redacted fallback"));
}

#[test]
fn package_rust_interop_rejects_map_error_without_a_mapped_error_member() {
    let generated = base_project_with_contracts(
        vec![declaration_entry_with_arguments(
            "bridge.hash.digest",
            RustInteropDecoratorKind::Function,
            vec![map_error_argument("bridge.hash.map_panic")],
        )],
        vec![panic_result_signature(
            "RustPanicError",
            declared_error_type("RustPanicError"),
        )],
    );
    let context = bridge_context(TrustPolicy::default());

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "map_error without a mapped error member must fail",
    );

    assert_eq!(diagnostics[0].code, "SIFR-RUST-PANIC-0001");
    assert!(diagnostics[0]
        .message
        .contains("reserved for generated wrapper failures"));
}

#[test]
fn package_rust_interop_rejects_wrapper_only_error_channel() {
    let generated = base_project_with_contracts(
        vec![declaration_entry_with_arguments(
            "bridge.hash.digest",
            RustInteropDecoratorKind::Function,
            Vec::new(),
        )],
        vec![panic_result_signature(
            "RustPanicError",
            declared_error_type("RustPanicError"),
        )],
    );
    let context = bridge_context(TrustPolicy::default());

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "ordinary target errors require a distinct declared member",
    );

    assert_eq!(diagnostics[0].code, "SIFR-RUST-PANIC-0001");
    assert!(diagnostics[0]
        .message
        .contains("reserved for generated wrapper failures"));
}

#[test]
fn package_rust_interop_rejects_similarly_named_panic_error() {
    let generated = base_project_with_contracts(
        vec![declaration_entry_with_arguments(
            "bridge.hash.digest",
            RustInteropDecoratorKind::Function,
            Vec::new(),
        )],
        vec![panic_result_signature(
            "RustPanicErrorish",
            declared_error_type("RustPanicErrorish"),
        )],
    );
    let context = bridge_context(TrustPolicy::default());

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "a similarly named error must not satisfy the nominal panic contract",
    );

    assert_eq!(diagnostics[0].code, "SIFR-RUST-PANIC-0001");
    assert!(diagnostics[0]
        .message
        .contains("must expose `RustPanicError`"));
}

#[test]
fn package_rust_interop_rejects_async_map_error_until_async_wrapper_certification() {
    let mut declaration = declaration_entry_with_arguments(
        "bridge.hash.digest",
        RustInteropDecoratorKind::Function,
        vec![map_error_argument("bridge.hash.map_panic")],
    );
    declaration.declaration.effect = RustInteropEffect::Async;
    declaration.declaration.abi_requirements.async_boundary = true;
    let generated = base_project_with_contracts(
        vec![declaration],
        vec![panic_result_signature(
            "HashError | RustPanicError",
            declared_error_union("HashError"),
        )],
    );
    let context = bridge_context(TrustPolicy::default());

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "async map_error must wait for async wrapper certification",
    );

    assert_eq!(diagnostics[0].code, "SIFR-RUST-PANIC-0001");
    assert!(diagnostics[0]
        .message
        .contains("async panic-wrapper certification"));
}

#[test]
fn package_rust_interop_rejects_invalid_map_error_shape() {
    let generated = base_project_with_contracts(
        vec![declaration_entry_with_arguments(
            "bridge.hash.digest",
            RustInteropDecoratorKind::Function,
            vec![RustInteropArgument {
                name: Some("panic".to_string()),
                value: RustInteropValue::PolicyCall {
                    name: "map_error".to_string(),
                    argument: Box::new(RustInteropValue::Integer(1)),
                    span: TextRange::default(),
                },
                span: TextRange::default(),
            }],
        )],
        vec![panic_result_signature(
            "HashError",
            declared_error_type("HashError"),
        )],
    );
    let context = bridge_context(TrustPolicy::default());

    let diagnostics = interop_errors(generated, Some(context), "invalid map_error must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-PANIC-0001");
    assert!(diagnostics[0].message.contains("unsupported panic policy"));
}

#[test]
fn package_rust_interop_rejects_unknown_panic_policy() {
    let generated = base_project_with_contracts(
        vec![declaration_entry_with_arguments(
            "bridge.hash.digest",
            RustInteropDecoratorKind::Function,
            vec![symbol_argument("panic", "resume_unwind")],
        )],
        vec![panic_result_signature(
            "HashError",
            declared_error_type("HashError"),
        )],
    );
    let context = bridge_context(TrustPolicy::default());

    let diagnostics = interop_errors(generated, Some(context), "unknown panic policy must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-PANIC-0001");
    assert!(diagnostics[0].message.contains("unsupported panic policy"));
}

#[test]
fn package_rust_interop_rejects_map_error_on_non_result() {
    let generated = base_project_with_contracts(
        vec![declaration_entry_with_arguments(
            "bridge.hash.digest",
            RustInteropDecoratorKind::Function,
            vec![map_error_argument("bridge.hash.map_panic")],
        )],
        vec![signature_contract(Vec::new(), string_contract())],
    );
    let context = bridge_context(TrustPolicy::default());

    let diagnostics = interop_errors(generated, Some(context), "map_error needs Result");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-PANIC-0001");
    assert!(diagnostics[0].message.contains("Result-returning"));
}

#[test]
fn package_rust_interop_non_result_requires_explicit_panic_policy() {
    let generated = base_project_with_contracts(
        vec![declaration_entry_with_arguments(
            "bridge.hash.digest",
            RustInteropDecoratorKind::Function,
            Vec::new(),
        )],
        vec![signature_contract(Vec::new(), string_contract())],
    );
    let context = bridge_context(TrustPolicy::default());

    let diagnostics = interop_errors(generated, Some(context), "non-Result needs policy");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-PANIC-0001");
    assert!(diagnostics[0].message.contains("non-Result"));
}

#[test]
fn package_rust_interop_sifr_stdlib_named_root_still_requires_explicit_policy() {
    let generated = base_project_with_contracts(
        vec![declaration_entry_with_arguments(
            "sifr_stdlib.hash.digest",
            RustInteropDecoratorKind::Function,
            Vec::new(),
        )],
        vec![signature_contract(Vec::new(), string_contract())],
    );
    let context = bridge_context(TrustPolicy::default());

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "package-authored sifr_stdlib root still needs policy",
    );

    assert_eq!(diagnostics[0].code, "SIFR-RUST-PANIC-0001");
    assert!(diagnostics[0].message.contains("non-Result"));
}

#[test]
fn package_rust_interop_abort_policy_requires_trust() {
    let generated = base_project_with_contracts(
        vec![declaration_entry_with_arguments(
            "bridge.hash.digest",
            RustInteropDecoratorKind::Function,
            vec![symbol_argument("panic", "abort")],
        )],
        vec![signature_contract(Vec::new(), string_contract())],
    );
    let context = bridge_context(TrustPolicy::default());

    let diagnostics = interop_errors(generated, Some(context), "abort trust required");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TRUST-0001");
    assert!(diagnostics[0].message.contains("bridge.hash.digest"));
}

#[test]
fn package_rust_interop_abort_policy_requires_abort_strategy_after_trust() {
    let generated = base_project_with_contracts(
        vec![declaration_entry_with_arguments(
            "bridge.hash.digest",
            RustInteropDecoratorKind::Function,
            vec![symbol_argument("panic", "abort")],
        )],
        vec![signature_contract(Vec::new(), string_contract())],
    );
    let mut trust = TrustPolicy::default();
    trust.rust_panic_abort = vec!["bridge.hash.digest".to_string()];
    let mut context = bridge_context(trust);
    let package_root = temp_package_root("rust_interop_panic_unwind_profile");
    std::fs::create_dir_all(&package_root).expect("create package root");
    std::fs::write(
        package_root.join("Cargo.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[profile.release]\npanic = \"unwind\"\n",
    )
    .expect("write package cargo profile");
    context
        .graph
        .packages
        .get_mut(&context.package_id)
        .expect("package exists")
        .package_root = package_root;

    let diagnostics = interop_errors(generated, Some(context), "abort strategy required");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-PANIC-0001");
    assert!(diagnostics[0].message.contains("panic strategy"));
}

#[test]
fn package_rust_interop_abort_policy_accepts_trust_and_abort_strategy() {
    let generated = base_project_with_contracts(
        vec![declaration_entry_with_arguments(
            "bridge.hash.digest",
            RustInteropDecoratorKind::Function,
            vec![symbol_argument("panic", "abort")],
        )],
        vec![signature_contract(Vec::new(), string_contract())],
    );
    let mut trust = TrustPolicy::default();
    trust.rust_panic_abort = vec!["bridge.hash.digest".to_string()];
    let mut context = bridge_context(trust);
    let package_root = temp_package_root("rust_interop_panic_abort_profile");
    std::fs::create_dir_all(package_root.join("src")).expect("create package source root");
    std::fs::write(
        package_root.join("Cargo.toml"),
        "[package]\nname = \"sifr-app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[profile.release]\npanic = \"abort\"\n",
    )
    .expect("write package cargo profile");
    std::fs::write(
        package_root.join("src/lib.rs"),
        "pub mod bridges { pub mod hash { pub fn digest() -> String { String::new() } } }\n",
    )
    .expect("write package-local bridge target");
    context
        .graph
        .packages
        .get_mut(&context.package_id)
        .expect("package exists")
        .package_root = package_root;

    super::rust_interop::apply_package_rust_interop_metadata(generated, Some(context))
        .expect("trusted abort policy with abort profile should pass panic contract validation");
}

fn bridge_context(trust: TrustPolicy) -> super::rust_interop::PackageRustInteropContext {
    let mut context = package_context(trust, Vec::new());
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);
    context
}

fn map_error_argument(target: &str) -> RustInteropArgument {
    RustInteropArgument {
        name: Some("panic".to_string()),
        value: RustInteropValue::PolicyCall {
            name: "map_error".to_string(),
            argument: Box::new(RustInteropValue::TargetPath(target_path(target))),
            span: TextRange::default(),
        },
        span: TextRange::default(),
    }
}

fn target_path(target: &str) -> RustTargetPath {
    RustTargetPath {
        segments: target.split('.').map(str::to_string).collect(),
        span: TextRange::default(),
    }
}

fn panic_result_signature(
    error_contract_name: &str,
    declared_error: Type,
) -> sifr_codegen::RustBridgeSignatureContract {
    let return_type = Type::Result(Box::new(Type::Str), Box::new(declared_error));
    let mut signature = signature_contract(
        Vec::new(),
        result_contract(string_contract(), error_contract(error_contract_name)),
    );
    signature.panic_error = sifr_codegen::rust_bridge_panic_error_contract(&return_type);
    signature
}

fn declared_error_union(ordinary_name: &str) -> Type {
    Type::Union(vec![
        declared_error_type(ordinary_name),
        declared_error_type("RustPanicError"),
    ])
}

fn declared_error_type(name: &str) -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: name.to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    }
}

fn error_contract(name: &str) -> RustBridgeTypeContract {
    RustBridgeTypeContract {
        sifr_type: name.to_string(),
        rust_borrowed_type: None,
        rust_owned_type: None,
        rust_return_type: Some(format!("crate::__sifr_bridge::main::{name}Bridge")),
        kind: RustBridgeTypeKind::GeneratedError,
        unsupported_reason: None,
    }
}
