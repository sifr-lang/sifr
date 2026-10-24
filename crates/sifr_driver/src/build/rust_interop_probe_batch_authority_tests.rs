use super::super::cargo_resolution::CargoResolutionPolicy;
use super::super::rust_interop_probe::{AsyncThreadAffinity, PendingRustBridgeProbe};
use super::super::rust_interop_probe_cache::ProbeInputSnapshot;
use super::{execute, grouping_key, plan};
use ruff_text_size::TextRange;
use sifr_codegen::{RustInteropOwner, RustInteropPlanDeclaration};
use sifr_ir::{
    RustInteropAbiRequirements, RustInteropDeclaration, RustInteropDecoratorKind,
    RustInteropEffect, RustTargetPath,
};
use sifr_package::{BackendCrateMetadata, CargoPackageId};
use std::path::Path;

fn fixture(root: &Path) {
    for (directory, name) in [
        ("backend", "probe_backend"),
        ("runtime", "sifr_runtime"),
        ("runtime/third_party/libsqlite3-sys", "libsqlite3-sys"),
    ] {
        let crate_root = root.join(directory);
        std::fs::create_dir_all(crate_root.join("src")).unwrap();
        std::fs::write(
            crate_root.join("Cargo.toml"),
            format!("[package]\nname={name:?}\nversion=\"0.0.0\"\nedition=\"2024\"\n[workspace]\n"),
        )
        .unwrap();
        std::fs::write(
            crate_root.join("src/lib.rs"),
            "pub fn first() {} pub fn second() {}",
        )
        .unwrap();
    }
}
fn probe(
    root: &Path,
    tools: &sifr_sysroot::NativeToolchain,
    function: &str,
) -> PendingRustBridgeProbe {
    let path = RustTargetPath {
        segments: vec!["probe_backend".into(), function.into()],
        span: TextRange::default(),
    };
    PendingRustBridgeProbe {
        declaration: RustInteropPlanDeclaration {
            module_name: Some("batch".into()),
            owner: RustInteropOwner::Function {
                name: function.into(),
            },
            declaration: RustInteropDeclaration {
                kind: RustInteropDecoratorKind::Function,
                target: Some(path.clone()),
                arguments: Vec::new(),
                span: TextRange::default(),
                effect: RustInteropEffect::Sync,
                abi_requirements: RustInteropAbiRequirements::default(),
                consumes_receiver: false,
            },
        },
        path,
        backend: BackendCrateMetadata {
            cargo_package_id: CargoPackageId("test-backend".into()),
            dependency_name: "probe_backend".into(),
            dependency_kind: None,
            cargo_package_name: "probe_backend".into(),
            cargo_version: "0.0.0".into(),
            cargo_source: None,
            cargo_manifest_path: root.join("backend/Cargo.toml"),
            links: None,
            has_build_script: false,
            has_proc_macro: false,
        },
        source_prefix: None,
        signature: None,
        async_thread_affinity: AsyncThreadAffinity::None,
        zero_copy_obligations: (false, false),
        trusted_sysroot: false,
        sysroot_runtime_crate: root.join("runtime"),
        sysroot_vendor_dir: None,
        cargo_resolution: CargoResolutionPolicy::normal_with_native(Ok(tools.clone())),
    }
}

#[test]
fn configurations_split_on_every_shared_authority_boundary() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    let tools = sifr_sysroot::NativeToolchain::resolve_at(temp.path()).unwrap();
    let base = probe(temp.path(), &tools, "first");
    let (key, planned) = plan(0, &base).unwrap_or_else(|e| panic!("{:?}", e.args));
    let second = probe(temp.path(), &tools, "second");
    assert_eq!(
        key,
        plan(1, &second)
            .unwrap_or_else(|e| panic!("{:?}", e.args))
            .0
    );
    assert_ne!(
        key,
        grouping_key(
            &base,
            &(planned.manifest.clone() + "\n[features]\nextra=[]\n")
        )
        .unwrap_or_else(|e| panic!("{:?}", e.args))
    );
    macro_rules! distinct {
        ($change:expr) => {{
            let mut changed = base.clone();
            $change(&mut changed);
            assert_ne!(
                key,
                grouping_key(&changed, &planned.manifest)
                    .unwrap_or_else(|e| panic!("{:?}", e.args))
            );
        }};
    }
    distinct!(|p: &mut PendingRustBridgeProbe| p.backend.cargo_package_id.0.push('x'));
    distinct!(|p: &mut PendingRustBridgeProbe| p.backend.cargo_version.push('x'));
    distinct!(|p: &mut PendingRustBridgeProbe| p.backend.cargo_source = Some("registry".into()));
    distinct!(
        |p: &mut PendingRustBridgeProbe| p.cargo_resolution.application_profile =
            crate::ApplicationProfile::Development
    );
    distinct!(
        |p: &mut PendingRustBridgeProbe| p.cargo_resolution.lock_mode =
            sifr_package::CargoLockMode::Frozen
    );
    distinct!(
        |p: &mut PendingRustBridgeProbe| p.cargo_resolution.cargo_vendor_mode =
            sifr_stdlib_manifest::CargoVendorMode::PackageOwned
    );
    distinct!(|p: &mut PendingRustBridgeProbe| p.sysroot_runtime_crate.push("other"));
    distinct!(
        |p: &mut PendingRustBridgeProbe| p.sysroot_vendor_dir = Some(temp.path().join("vendor"))
    );
    distinct!(|p: &mut PendingRustBridgeProbe| p
        .cargo_resolution
        .authoritative_locks
        .push(temp.path().join("Cargo.lock")));
    distinct!(|p: &mut PendingRustBridgeProbe| p
        .cargo_resolution
        .trusted_vendor_dirs
        .push(temp.path().join("vendor")));
    distinct!(|p: &mut PendingRustBridgeProbe| p.trusted_sysroot = true);
    distinct!(
        |p: &mut PendingRustBridgeProbe| p.cargo_resolution.native_toolchain = Ok(tools
            .clone()
            .with_declared_environment(vec!["SIFR_BATCH_IDENTITY_CONTROL".into()]))
    );
}

#[test]
fn batch_snapshots_recheck_bytes_and_reject_missing_or_linked_inputs() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    let tools = sifr_sysroot::NativeToolchain::resolve_at(temp.path()).unwrap();
    let mut selected = probe(temp.path(), &tools, "first");
    let authority = temp.path().join("authority.lock");
    std::fs::write(&authority, b"first").unwrap();
    selected
        .cargo_resolution
        .authoritative_locks
        .push(authority.clone());
    let backend = temp.path().join("backend");
    let mut previous = ProbeInputSnapshot::capture(&selected, &backend).unwrap();
    for path in [
        backend.join("src/lib.rs"),
        temp.path().join("runtime/src/lib.rs"),
        authority,
    ] {
        std::fs::write(&path, b"changed").unwrap();
        let current = ProbeInputSnapshot::capture(&selected, &backend).unwrap();
        assert_ne!(previous, current);
        previous = current;
    }
    let source = temp.path().join("runtime/src/lib.rs");
    std::fs::remove_file(&source).unwrap();
    assert_ne!(
        previous,
        ProbeInputSnapshot::capture(&selected, &backend).unwrap()
    );
    std::fs::remove_dir_all(temp.path().join("runtime")).unwrap();
    assert!(ProbeInputSnapshot::capture(&selected, &backend).is_err());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&backend, temp.path().join("runtime")).unwrap();
        assert!(ProbeInputSnapshot::capture(&selected, &backend).is_err());
    }
}

#[test]
fn actual_batch_runs_every_probe_and_refreshes_after_a_source_edit() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    let tools = sifr_sysroot::NativeToolchain::resolve_at(temp.path()).unwrap();
    let selections = vec![
        probe(temp.path(), &tools, "first"),
        probe(temp.path(), &tools, "second"),
    ];
    let (result, calls) = crate::capture_cargo_invocations(|| execute(&selections));
    assert!(
        result.is_empty(),
        "{:?}",
        result.iter().map(|(_, f)| &f.args).collect::<Vec<_>>()
    );
    assert_eq!(owned_probe_calls(&calls, &temp.path().join("backend")), 1);
    std::fs::write(
        temp.path().join("backend/src/lib.rs"),
        "pub fn unrelated() {}",
    )
    .unwrap();
    let (result, calls) = crate::capture_cargo_invocations(|| execute(&selections));
    assert_eq!(
        result.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(owned_probe_calls(&calls, &temp.path().join("backend")), 1);
    assert!(result[0].1.args.iter().any(|(_, v)| v == "batch.first"));
    assert!(result[1].1.args.iter().any(|(_, v)| v == "batch.second"));
}

#[test]
fn cargo_cannot_publish_a_batch_after_mutating_its_input() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    let runtime = temp.path().join("runtime/src/lib.rs");
    std::fs::write(
        temp.path().join("backend/build.rs"),
        format!(
            "fn main() {{ std::fs::write({:?}, \"pub fn changed() {{}}\").unwrap(); }}",
            runtime.to_str().unwrap()
        ),
    )
    .unwrap();
    let tools = sifr_sysroot::NativeToolchain::resolve_at(temp.path()).unwrap();
    let selections = vec![
        probe(temp.path(), &tools, "first"),
        probe(temp.path(), &tools, "second"),
    ];
    let failures = execute(&selections);
    assert_eq!(failures.len(), 2);
    assert!(failures.iter().all(|(_, failure)| {
        failure
            .args
            .iter()
            .any(|(_, message)| message == "Rust probe inputs changed while Cargo was running")
    }));
}

fn owned_probe_calls(calls: &[crate::CargoInvocation], backend: &Path) -> usize {
    calls
        .iter()
        .filter(|call| {
            if call.phase != "rust-probe" {
                return false;
            }
            let Some(argument) = call
                .args
                .windows(2)
                .find(|pair| pair[0] == "--manifest-path")
            else {
                return false;
            };
            let manifest = std::path::PathBuf::from(&argument[1]);
            let source = std::fs::read_to_string(&manifest).unwrap();
            let root: toml::Table = source.parse().unwrap();
            let Some(member) = root
                .get("workspace")
                .and_then(|v| v.get("members"))
                .and_then(toml::Value::as_array)
                .and_then(|members| members.first())
                .and_then(toml::Value::as_str)
            else {
                return false;
            };
            let member = manifest.parent().unwrap().join(member).join("Cargo.toml");
            let source = std::fs::read_to_string(member).unwrap();
            let package: toml::Table = source.parse().unwrap();
            package
                .get("dependencies")
                .and_then(|v| v.get("probe_backend"))
                .and_then(|v| v.get("path"))
                .and_then(toml::Value::as_str)
                .is_some_and(|path| Path::new(path) == backend)
        })
        .count()
}

#[test]
fn build_script_failure_keeps_original_diagnostics_for_every_probe() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    std::fs::write(
        temp.path().join("backend/build.rs"),
        "fn main() { panic!(\"intentional bridge build failure\"); }",
    )
    .unwrap();
    let tools = sifr_sysroot::NativeToolchain::resolve_at(temp.path()).unwrap();
    let selections = vec![
        probe(temp.path(), &tools, "first"),
        probe(temp.path(), &tools, "second"),
    ];
    let failures = execute(&selections);
    assert_eq!(failures.len(), 2);
    for ((index, failure), target) in failures.iter().zip(["batch.first", "batch.second"]) {
        assert!(*index < selections.len());
        assert!(failure.code == sifr_diagnostics::DiagnosticCode::RUST_TYPE_PROBE_FAILURE);
        assert!(failure.args.iter().any(|(_, value)| value == target));
        assert!(
            failure
                .notes
                .iter()
                .any(|note| note.contains("intentional bridge build failure"))
        );
    }
}

#[test]
fn missing_backend_manifest_keeps_the_resolution_contract() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    let tools = sifr_sysroot::NativeToolchain::resolve_at(temp.path()).unwrap();
    let selected = probe(temp.path(), &tools, "first");
    std::fs::remove_file(temp.path().join("backend/Cargo.toml")).unwrap();
    let failure = plan(0, &selected)
        .err()
        .expect("missing manifest must fail");
    assert!(failure.code == sifr_diagnostics::DiagnosticCode::RUST_CARGO_METADATA);
    assert!(
        failure
            .args
            .iter()
            .any(|(_, value)| value.contains("missing or unreadable in normal mode"))
    );
    assert!(
        failure
            .notes
            .iter()
            .any(|note| note.contains("prepare the package lockfile"))
    );
}

#[test]
fn procedural_macro_stdout_keeps_all_real_probes_valid() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    let backend = temp.path().join("backend");
    let macros = backend.join("macros");
    std::fs::create_dir_all(macros.join("src")).unwrap();
    std::fs::write(
        macros.join("Cargo.toml"),
        "[package]\nname=\"probe_output_macro\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[lib]\nproc-macro=true\n",
    )
    .unwrap();
    std::fs::write(
        macros.join("src/lib.rs"),
        r#"use proc_macro::TokenStream;
#[proc_macro]
pub fn make(_: TokenStream) -> TokenStream {
    println!("ordinary procedural macro output");
    "pub fn first() {} pub fn second() {}".parse().unwrap()
}
"#,
    )
    .unwrap();
    let manifest = backend.join("Cargo.toml");
    let mut contents = std::fs::read_to_string(&manifest).unwrap();
    contents.push_str("\n[dependencies]\nprobe_output_macro={path=\"macros\"}\n");
    std::fs::write(manifest, contents).unwrap();
    std::fs::write(backend.join("src/lib.rs"), "probe_output_macro::make!();").unwrap();
    let tools = sifr_sysroot::NativeToolchain::resolve_at(temp.path()).unwrap();
    let failures = execute(&[
        probe(temp.path(), &tools, "first"),
        probe(temp.path(), &tools, "second"),
    ]);
    assert!(
        failures.is_empty(),
        "{:?}",
        failures.iter().map(|(_, f)| &f.args).collect::<Vec<_>>()
    );
}
