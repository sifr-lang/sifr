use super::super::rust_interop_probe_paths::{
    normalize_cargo_target_dir, probe_cargo_target_dir_with_env, RUST_BRIDGE_PROBE_TARGET_DIR,
};
use super::super::workspace::artifact_cache_root;
use super::{
    dependency_features, generated_bridge_type_stubs, prefixed_probe_source, probe_cargo_toml,
    probe_cargo_vendor_args, python_raw_callback_probe_source, signature_return_probe_type,
    structural_signature_probe_source, zero_copy_type_probe_source,
};
use ruff_text_size::TextRange;
use sifr_codegen::{
    RustBridgeSignatureContract, RustBridgeTypeContract, RustBridgeTypeKind, RustInteropOwner,
};
use sifr_ir::RustTargetPath;
use std::path::Path;

#[test]
fn zero_copy_type_probe_emits_each_declared_thread_obligation() {
    for (obligations, send, sync) in [
        ((false, false), false, false),
        ((true, false), true, false),
        ((false, true), false, true),
        ((true, true), true, true),
    ] {
        let source = zero_copy_type_probe_source(obligations, "bridge::views::View");

        assert!(source.contains("type __SifrView = bridge::views::View;"));
        assert_eq!(source.contains("__sifr_assert_send::<__SifrView>();"), send);
        assert_eq!(source.contains("__sifr_assert_sync::<__SifrView>();"), sync);
    }
}

#[test]
fn prefixed_probe_keeps_inner_attributes_before_bridge_imports() {
    let source = prefixed_probe_source(
        "use bridge_backend::bridges as bridge;",
        "#![allow(dead_code)]\nfn __sifr_probe() {}\n",
    );

    assert_eq!(
        source,
        "#![allow(dead_code)]\nuse bridge_backend::bridges as bridge;\nfn __sifr_probe() {}\n"
    );
}

#[test]
fn sysroot_probe_manifest_uses_sysroot_runtime_crate() {
    let manifest = probe_cargo_toml(
        "sifr_stdlib",
        "sifr_stdlib",
        Path::new("/opt/sifr/crates/sifr_stdlib"),
        Path::new("/opt/sifr/crates/sifr_runtime"),
        &[],
    );

    assert!(manifest.contains(
        "sifr_stdlib = { path = \"/opt/sifr/crates/sifr_stdlib\", default-features = false }"
    ));
    assert!(manifest.contains("sifr_runtime = { path = \"/opt/sifr/crates/sifr_runtime\" }"));
}

#[test]
fn sysroot_runtime_probe_manifest_does_not_duplicate_runtime_dependency() {
    let manifest = probe_cargo_toml(
        "sifr_runtime",
        "sifr_runtime",
        Path::new("/opt/sifr/crates/sifr_runtime"),
        Path::new("/opt/sifr/crates/sifr_runtime"),
        &[],
    );

    assert_eq!(manifest.matches("sifr_runtime =").count(), 1);
}

#[test]
fn sysroot_probe_manifest_enables_declared_stdlib_features() {
    let manifest = probe_cargo_toml(
        "sifr_stdlib",
        "sifr_stdlib",
        Path::new("/opt/sifr/crates/sifr_stdlib"),
        Path::new("/opt/sifr/crates/sifr_runtime"),
        &["platform".to_string()],
    );

    assert!(manifest.contains(
        "sifr_stdlib = { path = \"/opt/sifr/crates/sifr_stdlib\", default-features = false, features = [\"platform\"] }"
    ));
}

fn signature(return_type: RustBridgeTypeContract) -> RustBridgeSignatureContract {
    RustBridgeSignatureContract {
        canonical_target_path: "_sifr.python.py_local_callback".to_string(),
        module_name: Some("_sifr.python".to_string()),
        owner: RustInteropOwner::Function {
            name: "py_local_callback".to_string(),
        },
        params: Vec::new(),
        return_type,
        structural_type_param: None,
        static_program_type_param: false,
        panic_error: sifr_codegen::RustBridgePanicErrorContract::None,
        span: TextRange::default(),
    }
}

#[test]
fn python_raw_callback_probe_uses_concrete_stdlib_error_type() {
    let signature = signature(RustBridgeTypeContract {
        sifr_type: "Result[tuple[int, int, int, int, str], PythonError]".to_string(),
        rust_borrowed_type: None,
        rust_owned_type: None,
        rust_return_type: Some(
            "Result<(i64, i64, i64, i64, String), __SifrBridgeError>".to_string(),
        ),
        kind: RustBridgeTypeKind::Result,
        unsupported_reason: None,
    });

    let source =
        python_raw_callback_probe_source(&signature, "::sifr_stdlib::python::py_local_callback");

    assert!(source.contains("::sifr_stdlib::python::PythonError"));
    assert!(!source.contains("__SifrBridgeError"));
}

#[test]
fn structural_probe_normalizes_backend_display_errors_without_erasing_ok_type() {
    let mut signature = signature(RustBridgeTypeContract {
        sifr_type: "Result[T, BridgeError]".to_string(),
        rust_borrowed_type: None,
        rust_owned_type: None,
        rust_return_type: Some(
            "Result<T, crate::__sifr_bridge::main::BridgeErrorBridge>".to_string(),
        ),
        kind: RustBridgeTypeKind::Result,
        unsupported_reason: None,
    });
    signature.structural_type_param = Some("T".to_string());
    signature.static_program_type_param = true;

    let source = structural_signature_probe_source(&signature, "bridge::roundtrip", "T");

    assert!(source.contains("let _: Result<T, String>"));
    assert!(source.contains("bridge::roundtrip::<T>()"));
    assert!(source.contains(".map_err(|error| error.to_string())"));
    assert!(source
        .contains("StructuralProject + ::sifr_runtime::interop::structural::StaticProgramType"));
}

#[test]
fn sysroot_stdlib_probe_features_follow_target_module_segment() {
    let stdlib_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("driver crate should have crates parent")
        .join("sifr_stdlib");
    let path = RustTargetPath {
        segments: ["sifr_stdlib", "platform", "platform_system"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        span: TextRange::default(),
    };

    assert_eq!(
        dependency_features("sifr_stdlib", &stdlib_root, &path),
        vec!["platform".to_string()]
    );
}

#[test]
fn sysroot_stdlib_probe_features_normalize_rust_module_separators() {
    let stdlib_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("driver crate should have crates parent")
        .join("sifr_stdlib");
    let path = RustTargetPath {
        segments: ["sifr_stdlib", "runtime_observability", "emit_diagnostic"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        span: TextRange::default(),
    };

    assert_eq!(
        dependency_features("sifr_stdlib", &stdlib_root, &path),
        vec!["runtime-observability".to_string()]
    );
}

#[test]
fn sysroot_stdlib_probe_features_ignore_undeclared_target_segment() {
    let stdlib_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("driver crate should have crates parent")
        .join("sifr_stdlib");
    let path = RustTargetPath {
        segments: ["sifr_stdlib", "not_a_feature", "leaf"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        span: TextRange::default(),
    };

    assert!(dependency_features("sifr_stdlib", &stdlib_root, &path).is_empty());
}

#[test]
fn generated_bridge_type_stubs_follow_sanitized_bridge_type_paths() {
    let signature = signature(RustBridgeTypeContract {
        sifr_type: "list[int]".to_string(),
        rust_borrowed_type: None,
        rust_owned_type: None,
        rust_return_type: Some("__sifr_bridge::_sifr_calendar::CalendarBridge".to_string()),
        kind: RustBridgeTypeKind::GeneratedRecord,
        unsupported_reason: None,
    });
    let source = generated_bridge_type_stubs(&signature);

    assert!(source.contains("pub mod _sifr_calendar {\n"));
    assert!(source.contains("pub struct CalendarBridge;"));
    assert!(!source.contains("pub mod _sifr.calendar"));
}

#[test]
fn result_error_bridge_return_probes_display_error_generic() {
    let return_type = RustBridgeTypeContract {
        sifr_type: "Result[str, ParseError]".to_string(),
        rust_borrowed_type: None,
        rust_owned_type: None,
        rust_return_type: Some(
            "Result<String, crate::__sifr_bridge::_sifr_crypto::ParseErrorBridge>".to_string(),
        ),
        kind: RustBridgeTypeKind::Result,
        unsupported_reason: None,
    };

    let probe_type = signature_return_probe_type(&return_type);

    assert_eq!(
        probe_type.ty,
        "Result<String, __SifrBridgeError>".to_string()
    );
    assert!(probe_type.display_error_generic);
}

#[test]
fn sysroot_probe_vendor_args_use_invocation_scoped_config() {
    let args = probe_cargo_vendor_args(Some(Path::new("/opt/sifr sysroot/vendor")));

    assert_eq!(
        args,
        vec![
            "--config",
            "source.crates-io.replace-with=\"sifr-vendor\"",
            "--config",
            "source.sifr-vendor.directory=\"/opt/sifr sysroot/vendor\"",
        ]
    );
}

#[test]
fn relative_probe_target_dir_is_anchored_to_invocation_cwd() {
    assert_eq!(
        normalize_cargo_target_dir(
            Path::new("/workspace/sifr"),
            Path::new("target/create-pr").to_path_buf()
        ),
        Path::new("/workspace/sifr/target/create-pr")
    );
}

#[test]
fn absolute_probe_target_dir_is_preserved() {
    assert_eq!(
        normalize_cargo_target_dir(
            Path::new("/workspace/sifr"),
            Path::new("/tmp/sifr-target").to_path_buf()
        ),
        Path::new("/tmp/sifr-target")
    );
}

#[test]
fn probe_target_dir_defaults_to_stable_artifact_cache_subdir() {
    assert_eq!(
        probe_cargo_target_dir_with_env(None, Path::new("/workspace/sifr")),
        artifact_cache_root().join(RUST_BRIDGE_PROBE_TARGET_DIR)
    );
}

#[test]
fn probe_target_dir_honors_relative_env_override() {
    assert_eq!(
        probe_cargo_target_dir_with_env(
            Some(std::ffi::OsString::from("target/create-pr")),
            Path::new("/workspace/sifr")
        ),
        Path::new("/workspace/sifr/target/create-pr")
    );
}
