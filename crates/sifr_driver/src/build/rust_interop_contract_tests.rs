use super::project_codegen::GeneratedBinaryProject;
use super::rust_interop::{
    PackageRustInteropContext, RustInteropModuleSource, apply_package_rust_interop_metadata,
};
use crate::diagnostics::RenderedDiagnostic;
use ruff_text_size::{TextRange, TextSize};
use sifr_codegen::{
    InteropBuildPlan, RustBridgeContractPlan, RustBridgeParamContract, RustBridgeParamConvention,
    RustBridgeSignatureContract, RustBridgeTypeContract, RustBridgeTypeKind, RustInteropOwner,
    RustInteropPlan, RustInteropPlanDeclaration, RustInteropTrustRequirementKind,
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

#[test]
fn package_rust_interop_rejects_unsupported_bridge_type_contract() {
    let generated = base_project_with_contracts(
        vec![declaration_entry(
            "bridge.hash",
            RustInteropDecoratorKind::Function,
        )],
        vec![signature_contract(
            vec![param_contract(
                "items",
                RustBridgeParamConvention::Borrow,
                unsupported_contract(
                    "set[int]",
                    "set[T] is not a supported Rust bridge container",
                ),
            )],
            none_contract(),
        )],
    );
    let mut context = package_context(TrustPolicy::default(), Vec::new());
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "unsupported bridge type must fail before build",
    );

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TYPE-0001");
    assert!(diagnostics[0].message.contains("set[int]"));
    assert!(diagnostics[0].message.contains("app.hash"));
}

#[test]
fn package_rust_interop_direct_probe_checks_signature_shape() {
    let backend_root = temp_package_root("rust_interop_signature_probe");
    std::fs::create_dir_all(backend_root.join("src")).expect("create backend src");
    std::fs::write(
        backend_root.join("Cargo.toml"),
        "[package]\nname = \"native\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write backend cargo toml");
    std::fs::write(
        backend_root.join("src/lib.rs"),
        "pub fn hash(input: Vec<u8>) -> Vec<u8> { input }\n",
    )
    .expect("write backend lib");

    let generated = base_project_with_contracts(
        vec![trusted_no_panic_declaration_entry(
            "native.hash",
            RustInteropDecoratorKind::Function,
        )],
        vec![signature_contract(
            vec![param_contract(
                "input",
                RustBridgeParamConvention::Borrow,
                bytes_contract(),
            )],
            bytes_contract(),
        )],
    );
    let context = trusted_no_panic_context(vec![backend_with_manifest(
        "native",
        backend_root.join("Cargo.toml"),
    )]);

    let diagnostics = interop_errors(generated, Some(context), "signature probe must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TYPE-0001");
    assert!(diagnostics[0].message.contains("Rust bridge probe failed"));
}

#[test]
#[doc = "sifr-evidence: executes-cargo-probe"]
fn package_rust_interop_direct_probe_accepts_bridge_signature() {
    let backend_root = temp_package_root("rust_interop_signature_probe_ok");
    std::fs::create_dir_all(backend_root.join("src")).expect("create backend src");
    std::fs::write(
        backend_root.join("Cargo.toml"),
        "[package]\nname = \"native\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write backend cargo toml");
    std::fs::write(
        backend_root.join("src/lib.rs"),
        "pub fn hash(input: &[u8]) -> Vec<u8> { input.to_vec() }\n",
    )
    .expect("write backend lib");

    let generated = base_project_with_contracts(
        vec![trusted_no_panic_declaration_entry(
            "native.hash",
            RustInteropDecoratorKind::Function,
        )],
        vec![signature_contract(
            vec![param_contract(
                "input",
                RustBridgeParamConvention::Borrow,
                bytes_contract(),
            )],
            bytes_contract(),
        )],
    );
    let context = trusted_no_panic_context(vec![backend_with_manifest(
        "native",
        backend_root.join("Cargo.toml"),
    )]);

    apply_package_rust_interop_metadata(generated, Some(context))
        .expect("compatible signature should pass probe");
}

#[test]
fn package_rust_interop_direct_non_result_requires_panic_policy() {
    let backend_root = temp_package_root("rust_interop_missing_panic_policy");
    std::fs::create_dir_all(backend_root.join("src")).expect("create backend src");
    std::fs::write(
        backend_root.join("Cargo.toml"),
        "[package]\nname = \"native\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write backend cargo toml");
    std::fs::write(
        backend_root.join("src/lib.rs"),
        "pub fn hash(input: &[u8]) -> Vec<u8> { input.to_vec() }\n",
    )
    .expect("write backend lib");

    let generated = base_project_with_contracts(
        vec![declaration_entry(
            "native.hash",
            RustInteropDecoratorKind::Function,
        )],
        vec![signature_contract(
            vec![param_contract(
                "input",
                RustBridgeParamConvention::Borrow,
                bytes_contract(),
            )],
            bytes_contract(),
        )],
    );
    let context = package_context(
        TrustPolicy::default(),
        vec![backend_with_manifest(
            "native",
            backend_root.join("Cargo.toml"),
        )],
    );

    let diagnostics = interop_errors(generated, Some(context), "missing panic policy must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-PANIC-0001");
    assert!(
        diagnostics[0]
            .message
            .contains("non-Result Rust interop declarations")
    );
}

#[test]
fn package_rust_interop_direct_probe_rejects_unsafe_fn() {
    let backend_root = temp_package_root("rust_interop_unsafe_signature_probe");
    std::fs::create_dir_all(backend_root.join("src")).expect("create backend src");
    std::fs::write(
        backend_root.join("Cargo.toml"),
        "[package]\nname = \"native\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write backend cargo toml");
    std::fs::write(
        backend_root.join("src/lib.rs"),
        "pub unsafe fn hash(input: &[u8]) -> Vec<u8> { input.to_vec() }\n",
    )
    .expect("write backend lib");

    let generated = base_project_with_contracts(
        vec![trusted_no_panic_declaration_entry(
            "native.hash",
            RustInteropDecoratorKind::Function,
        )],
        vec![signature_contract(
            vec![param_contract(
                "input",
                RustBridgeParamConvention::Borrow,
                bytes_contract(),
            )],
            bytes_contract(),
        )],
    );
    let context = trusted_no_panic_context(vec![backend_with_manifest(
        "native",
        backend_root.join("Cargo.toml"),
    )]);

    let diagnostics = interop_errors(generated, Some(context), "unsafe fn must fail probe");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TYPE-0001");
    assert!(diagnostics[0].message.contains("Rust bridge probe failed"));
}

#[test]
fn package_rust_interop_direct_probe_accepts_mutable_borrow_signature() {
    let backend_root = temp_package_root("rust_interop_mut_borrow_signature_probe");
    std::fs::create_dir_all(backend_root.join("src")).expect("create backend src");
    std::fs::write(
        backend_root.join("Cargo.toml"),
        "[package]\nname = \"native\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write backend cargo toml");
    std::fs::write(
        backend_root.join("src/lib.rs"),
        "pub fn hash(input: &mut [u8]) { input.reverse(); }\n",
    )
    .expect("write backend lib");

    let generated = base_project_with_contracts(
        vec![trusted_no_panic_declaration_entry(
            "native.hash",
            RustInteropDecoratorKind::Function,
        )],
        vec![signature_contract(
            vec![param_contract(
                "input",
                RustBridgeParamConvention::MutableBorrow,
                bytes_contract(),
            )],
            none_contract(),
        )],
    );
    let context = trusted_no_panic_context(vec![backend_with_manifest(
        "native",
        backend_root.join("Cargo.toml"),
    )]);

    apply_package_rust_interop_metadata(generated, Some(context))
        .expect("compatible mutable borrow signature should pass probe");
}

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
fn package_rust_interop_opaque_rejects_unknown_contract_key() {
    let generated = base_project_with_contracts(
        vec![opaque_class_declaration_entry(vec![
            target_argument("type", "native.Tokenizer"),
            symbol_argument("lifetime", "static"),
        ])],
        Vec::new(),
    );
    let context = package_context(TrustPolicy::default(), Vec::new());

    let diagnostics = interop_errors(generated, Some(context), "unknown opaque key must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-CONFIG-0001");
    assert!(diagnostics[0].message.contains("unsupported `@rust.opaque"));
}

#[test]
fn package_rust_interop_mapped_value_rejects_resource_clone_policy() {
    let generated = base_project_with_contracts(
        vec![opaque_class_declaration_entry(vec![
            target_argument("type", "native.Tokenizer"),
            target_argument("structural", "native.TokenizerMapping"),
            symbol_argument("clone", "arc"),
        ])],
        Vec::new(),
    );
    let context = package_context(TrustPolicy::default(), Vec::new());

    let diagnostics = interop_errors(generated, Some(context), "mapped clone policy must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-CONFIG-0001");
    assert!(
        diagnostics[0]
            .message
            .contains("resource-sharing clone policies")
    );
}

#[test]
fn package_rust_interop_records_declared_transitive_bridge_native_links() {
    let declaration = declaration_entry("bridge.hash", RustInteropDecoratorKind::Function);
    let generated = base_project_with_contracts(vec![declaration], Vec::new());
    let trust = TrustPolicy {
        native_links: vec!["ring_core_0_17_14_".to_string()],
        ..TrustPolicy::default()
    };
    let mut context = package_context(trust, Vec::new());
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let generated = apply_package_rust_interop_metadata(generated, Some(context))
        .expect("declared transitive bridge native link should be recorded");

    assert!(
        generated
            .interop
            .rust
            .trust_requirements
            .iter()
            .any(|requirement| {
                requirement.kind == RustInteropTrustRequirementKind::NativeLinks
                    && requirement.required_entry == "ring_core_0_17_14_"
                    && requirement.trusted
            })
    );
}

#[test]
fn package_rust_interop_records_declared_native_links_for_direct_crate_bindings() {
    let declaration = declaration_entry("native.hash", RustInteropDecoratorKind::Function);
    let generated = base_project_with_contracts(vec![declaration], Vec::new());
    let trust = TrustPolicy {
        native_links: vec!["psm_s".to_string()],
        ..TrustPolicy::default()
    };
    let context = package_context(
        trust,
        vec![backend_with_manifest(
            "native",
            PathBuf::from("/ws/native/Cargo.toml"),
        )],
    );

    let generated = apply_package_rust_interop_metadata(generated, Some(context))
        .expect("package-scoped native-link trust should cover direct crate bindings");

    assert!(
        generated
            .interop
            .rust
            .trust_requirements
            .iter()
            .any(|requirement| {
                requirement.kind == RustInteropTrustRequirementKind::NativeLinks
                    && requirement.required_entry == "psm_s"
                    && requirement.trusted
            })
    );
}

pub(super) fn base_project_with_contracts(
    declarations: Vec<RustInteropPlanDeclaration>,
    signatures: Vec<RustBridgeSignatureContract>,
) -> GeneratedBinaryProject {
    GeneratedBinaryProject {
        main_rs: "fn main() {}\n".to_string(),
        support_modules: BTreeMap::new(),
        used_stdlib_modules: HashSet::new(),
        required_features: HashSet::<StdlibFeature>::new(),
        interop: InteropBuildPlan {
            rust: RustInteropPlan {
                declarations,
                bridge_contracts: RustBridgeContractPlan {
                    signatures,
                    generated_types: Vec::new(),
                },
                ..RustInteropPlan::default()
            },
            ..InteropBuildPlan::default()
        },
        cache_key_fragment: None,
        bridge_modules: Default::default(),
        python_runtime: None,
    }
}

pub(super) fn signature_contract(
    params: Vec<RustBridgeParamContract>,
    return_type: RustBridgeTypeContract,
) -> RustBridgeSignatureContract {
    RustBridgeSignatureContract {
        canonical_target_path: "app.hash".to_string(),
        module_name: Some("app".to_string()),
        owner: RustInteropOwner::Function {
            name: "hash".to_string(),
        },
        params,
        return_type,
        structural_type_params: Vec::new(),
        static_program_type_params: Vec::new(),
        method_slot_contract: None,
        panic_error: sifr_codegen::RustBridgePanicErrorContract::None,
        span: span(),
    }
}

pub(super) fn param_contract(
    name: &str,
    convention: RustBridgeParamConvention,
    ty: RustBridgeTypeContract,
) -> RustBridgeParamContract {
    RustBridgeParamContract {
        name: name.to_string(),
        convention,
        ty,
    }
}

fn bytes_contract() -> RustBridgeTypeContract {
    RustBridgeTypeContract {
        sifr_type: "bytes".to_string(),
        rust_borrowed_type: Some("&[u8]".to_string()),
        rust_owned_type: Some("Vec<u8>".to_string()),
        rust_return_type: Some("Vec<u8>".to_string()),
        kind: RustBridgeTypeKind::Bytes,
        unsupported_reason: None,
    }
}

pub(super) fn string_contract() -> RustBridgeTypeContract {
    RustBridgeTypeContract {
        sifr_type: "str".to_string(),
        rust_borrowed_type: Some("&str".to_string()),
        rust_owned_type: Some("String".to_string()),
        rust_return_type: Some("String".to_string()),
        kind: RustBridgeTypeKind::String,
        unsupported_reason: None,
    }
}

pub(super) fn result_contract(
    ok: RustBridgeTypeContract,
    err: RustBridgeTypeContract,
) -> RustBridgeTypeContract {
    RustBridgeTypeContract {
        sifr_type: format!("Result[{}, {}]", ok.sifr_type, err.sifr_type),
        rust_borrowed_type: None,
        rust_owned_type: None,
        rust_return_type: Some(format!(
            "Result<{}, {}>",
            ok.rust_return_type.expect("ok result type"),
            err.rust_return_type.expect("err result type")
        )),
        kind: RustBridgeTypeKind::Result,
        unsupported_reason: None,
    }
}

pub(super) fn none_contract() -> RustBridgeTypeContract {
    RustBridgeTypeContract {
        sifr_type: "None".to_string(),
        rust_borrowed_type: Some("()".to_string()),
        rust_owned_type: Some("()".to_string()),
        rust_return_type: Some("()".to_string()),
        kind: RustBridgeTypeKind::None,
        unsupported_reason: None,
    }
}

pub(super) fn unsupported_contract(sifr_type: &str, reason: &str) -> RustBridgeTypeContract {
    RustBridgeTypeContract {
        sifr_type: sifr_type.to_string(),
        rust_borrowed_type: None,
        rust_owned_type: None,
        rust_return_type: None,
        kind: RustBridgeTypeKind::Unsupported,
        unsupported_reason: Some(reason.to_string()),
    }
}

pub(super) fn interop_errors(
    generated: GeneratedBinaryProject,
    context: Option<PackageRustInteropContext>,
    message: &str,
) -> Vec<RenderedDiagnostic> {
    match apply_package_rust_interop_metadata(generated, context) {
        Ok(_) => panic!("{message}"),
        Err(diagnostics) => diagnostics,
    }
}

pub(super) fn declaration_entry(
    target: &str,
    kind: RustInteropDecoratorKind,
) -> RustInteropPlanDeclaration {
    declaration_entry_with_arguments(target, kind, Vec::new())
}

pub(super) fn trusted_no_panic_declaration_entry(
    target: &str,
    kind: RustInteropDecoratorKind,
) -> RustInteropPlanDeclaration {
    declaration_entry_with_arguments(
        target,
        kind,
        vec![RustInteropArgument {
            name: Some("panic".to_string()),
            value: RustInteropValue::Symbol("trusted_no_panic".to_string()),
            span: span(),
        }],
    )
}

pub(super) fn opaque_class_declaration_entry(
    arguments: Vec<RustInteropArgument>,
) -> RustInteropPlanDeclaration {
    RustInteropPlanDeclaration {
        module_name: Some("app".to_string()),
        owner: RustInteropOwner::Class {
            name: "Tokenizer".to_string(),
        },
        declaration: RustInteropDeclaration {
            kind: RustInteropDecoratorKind::Opaque,
            target: None,
            arguments,
            span: span(),
            effect: RustInteropEffect::Sync,
            abi_requirements: RustInteropAbiRequirements {
                opaque_handle: true,
                ..RustInteropAbiRequirements::default()
            },
            consumes_receiver: false,
        },
    }
}

pub(super) fn tokenizer_method_declaration_entry(
    name: &str,
    kind: RustInteropDecoratorKind,
) -> RustInteropPlanDeclaration {
    RustInteropPlanDeclaration {
        module_name: Some("app".to_string()),
        owner: RustInteropOwner::Method {
            class_name: "Tokenizer".to_string(),
            name: name.to_string(),
        },
        declaration: RustInteropDeclaration {
            kind,
            target: Some(RustTargetPath {
                segments: ["bridge", "resources", name]
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
                span: span(),
            }),
            arguments: Vec::new(),
            span: span(),
            effect: effect_for_kind(kind),
            abi_requirements: abi_for_kind(kind),
            consumes_receiver: true,
        },
    }
}

pub(super) fn declaration_entry_with_arguments(
    target: &str,
    kind: RustInteropDecoratorKind,
    arguments: Vec<RustInteropArgument>,
) -> RustInteropPlanDeclaration {
    declaration_entry_with_arguments_and_effect(target, kind, arguments, effect_for_kind(kind))
}

fn declaration_entry_with_arguments_and_effect(
    target: &str,
    kind: RustInteropDecoratorKind,
    arguments: Vec<RustInteropArgument>,
    effect: RustInteropEffect,
) -> RustInteropPlanDeclaration {
    RustInteropPlanDeclaration {
        module_name: Some("app".to_string()),
        owner: RustInteropOwner::Function {
            name: "hash".to_string(),
        },
        declaration: RustInteropDeclaration {
            kind,
            target: Some(RustTargetPath {
                segments: target.split('.').map(str::to_string).collect(),
                span: span(),
            }),
            arguments,
            span: span(),
            effect,
            abi_requirements: abi_for_kind(kind),
            consumes_receiver: false,
        },
    }
}

fn effect_for_kind(kind: RustInteropDecoratorKind) -> RustInteropEffect {
    if kind == RustInteropDecoratorKind::Async {
        RustInteropEffect::Async
    } else {
        RustInteropEffect::Sync
    }
}

fn abi_for_kind(kind: RustInteropDecoratorKind) -> RustInteropAbiRequirements {
    RustInteropAbiRequirements {
        async_boundary: kind == RustInteropDecoratorKind::Async,
        opaque_handle: kind == RustInteropDecoratorKind::Opaque,
        zero_copy: kind == RustInteropDecoratorKind::ZeroCopy,
        view: kind == RustInteropDecoratorKind::View,
    }
}

pub(super) fn target_argument(name: &str, target: &str) -> RustInteropArgument {
    RustInteropArgument {
        name: Some(name.to_string()),
        value: RustInteropValue::TargetPath(RustTargetPath {
            segments: target.split('.').map(str::to_string).collect(),
            span: span(),
        }),
        span: span(),
    }
}

fn bool_argument(name: &str, value: bool) -> RustInteropArgument {
    RustInteropArgument {
        name: Some(name.to_string()),
        value: RustInteropValue::Boolean(value),
        span: span(),
    }
}

pub(super) fn symbol_argument(name: &str, value: &str) -> RustInteropArgument {
    RustInteropArgument {
        name: Some(name.to_string()),
        value: RustInteropValue::Symbol(value.to_string()),
        span: span(),
    }
}

pub(super) fn trusted_no_panic_context(
    backend_crates: Vec<BackendCrateMetadata>,
) -> PackageRustInteropContext {
    let trust = TrustPolicy {
        rust_no_panic: vec!["native.hash".to_string()],
        ..TrustPolicy::default()
    };
    package_context(trust, backend_crates)
}

pub(super) fn package_context(
    trust: TrustPolicy,
    backend_crates: Vec<BackendCrateMetadata>,
) -> PackageRustInteropContext {
    let package_root = PathBuf::from("/ws/app");
    let package_id = SifrPackageId("sifr-app@0.1.0#path".to_string());
    let cargo_package_id = CargoPackageId("path+file:///ws/app#sifr-app@0.1.0".to_string());
    let package = SifrPackageMetadata {
        package_id: package_id.clone(),
        cargo_package_id: cargo_package_id.clone(),
        cargo_package_name: "sifr-app".to_string(),
        cargo_version: "0.1.0".to_string(),
        cargo_source: None,
        package_root: package_root.clone(),
        sifr_manifest: package_root.join("sifr.toml"),
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

pub(super) fn backend_with_manifest(
    name: &str,
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
        has_build_script: false,
        has_proc_macro: false,
    }
}

pub(super) fn set_bridge_roots(context: &mut PackageRustInteropContext, bridges: Vec<PathBuf>) {
    let package = context
        .graph
        .packages
        .get_mut(&context.package_id)
        .expect("package exists");
    package.manifest.rust.bridges = bridges;
}

pub(super) fn temp_package_root(name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let root = std::env::temp_dir().join(format!("sifr_{name}_{}_{nonce}", std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).expect("remove stale temp root");
    }
    root
}

fn span() -> TextRange {
    TextRange::new(TextSize::from(0), TextSize::from(18))
}
