use super::{
    ensure::{Stage, ensure_with_hook},
    production::Inputs,
    wire, *,
};
use sifr_identity::CompilerIdentity;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Instant,
};
#[path = "tests_process.rs"]
mod processes;
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn identity(configuration: &str) -> CompilerIdentity {
    CompilerIdentity::for_test(crate::compiled_input_tokens(), configuration)
}
const TARGET: &str = if cfg!(windows) {
    "x86_64-pc-windows-msvc"
} else if cfg!(target_os = "macos") {
    if cfg!(target_arch = "aarch64") {
        "aarch64-apple-darwin"
    } else {
        "x86_64-apple-darwin"
    }
} else if cfg!(target_arch = "aarch64") {
    "aarch64-unknown-linux-gnu"
} else {
    "x86_64-unknown-linux-gnu"
};
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "sifr-dx6-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        #[cfg(unix)]
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        #[cfg(windows)]
        fs::create_dir(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }
    fn source(&self) -> PathBuf {
        fn copy(from: &Path, to: &Path) {
            fs::create_dir_all(to).unwrap();
            for entry in fs::read_dir(from).unwrap() {
                let entry = entry.unwrap();
                let dest = to.join(entry.file_name());
                if entry.file_type().unwrap().is_dir() {
                    copy(&entry.path(), &dest);
                } else {
                    fs::copy(entry.path(), dest).unwrap();
                }
            }
        }
        let dest = self.0.join("source");
        fs::create_dir(&dest).unwrap();
        copy(&root().join("stdlib"), &dest.join("stdlib"));
        for relative in [
            "sysroot.toml",
            "Cargo.toml",
            "Cargo.lock",
            ".cargo/config.toml",
            "crates/sifr_runtime/Cargo.toml",
            "crates/sifr_stdlib/Cargo.toml",
            "crates/sifr_structural_identity/Cargo.toml",
        ] {
            let path = dest.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::copy(root().join(relative), path).unwrap();
        }
        fs::create_dir(dest.join("vendor")).unwrap();
        dest
    }
}
#[cfg(unix)]
fn permissions(path: &Path, readonly: bool) {
    if path.is_dir() {
        if !readonly {
            fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
        }
        for entry in fs::read_dir(path).unwrap() {
            permissions(&entry.unwrap().path(), readonly);
        }
    }
    fs::set_permissions(
        path,
        fs::Permissions::from_mode(if path.is_dir() {
            if readonly { 0o500 } else { 0o700 }
        } else if readonly {
            0o400
        } else {
            0o600
        }),
    )
    .unwrap();
}
impl Drop for Scratch {
    fn drop(&mut self) {
        #[cfg(unix)]
        permissions(&self.0, false);
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn assertion(metadata: &PreparedMetadata, identity: &CompilerIdentity, source: &Path) {
    let modules = metadata
        .store
        .record_refs::<wire::Module>()
        .collect::<Vec<_>>();
    assert!(modules.len() > 50);
    for reference in modules {
        let module = metadata.store.get(reference).unwrap();
        let name = &metadata.store.get(module.name).unwrap().value;
        assert!(name.starts_with("sifr.") || name.starts_with("_sifr."));
        metadata.store.get(module.semantic).unwrap();
    }
    let context = crate::CompilerContext::with_sysroot(
        identity.clone(),
        sifr_sysroot::resolve_sysroot(Some(source.to_owned())).unwrap(),
    )
    .with_metadata_override(metadata.path.clone());
    assert!(
        crate::type_check_source(
            &context,
            "from sifr.math import sqrt\n\ndef main() -> float:\n    return sqrt(4.0)\n"
        )
        .is_empty()
    );
    assert!(
        !crate::type_check_source(
            &context,
            "from sifr.math import sqrt\n\ndef main() -> int:\n    return \"wrong\"\n"
        )
        .is_empty()
    );
}
#[test]
fn dx6_canonical_inventory_producer_without_metadata() {
    let source = root();
    let identity = identity("dx6-inventory");
    let inputs = Inputs::capture(&identity, &source, TARGET).unwrap();
    let started = Instant::now();
    let bytes = inputs.produce().unwrap();
    let store = wire::MetadataStore::open(
        std::io::Cursor::new(bytes.clone()),
        inputs.compatibility,
        wire::Limits::default(),
    )
    .unwrap();
    store.validate_complete().unwrap();
    assert_eq!(
        store.record_refs::<wire::Module>().count(),
        inputs.sources.len()
    );
    let compiled =
        crate::stdlib::compile_stdlib_sources_with_sysroot(&inputs.sources, inputs.sysroot.clone())
            .unwrap();
    for reference in store.record_refs::<wire::Module>() {
        let module = store.get(reference).unwrap();
        let name = store.get(module.name).unwrap();
        let semantic = store.get(module.semantic).unwrap();
        let names = semantic
            .functions
            .keys()
            .map(|reference| store.get(*reference).unwrap().value.clone())
            .collect::<std::collections::BTreeSet<_>>();
        let expected = compiled
            .defs
            .functions
            .get(&name.value)
            .into_iter()
            .flat_map(|m| m.keys().cloned())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(names, expected, "{}", name.value);
        let hir = store.get(module.hir_inventory).unwrap();
        assert_eq!(
            hir.functions.len(),
            compiled.code.hir_modules[&name.value].functions.len()
        );
        assert_eq!(
            hir.classes.len(),
            compiled.code.hir_modules[&name.value].classes.len()
        );
        if let Some(rust) = module.rust {
            let payload = store.get(rust).unwrap();
            assert_eq!(
                store.get(payload.source).unwrap().value,
                compiled.code.module_rust_code[&name.value].rust
            );
        }
    }
    let second = inputs.produce().unwrap();
    if bytes != second {
        let debug = std::path::PathBuf::from(format!("/tmp/sifr-dx6-repro-{}", std::process::id()));
        fs::create_dir_all(&debug).unwrap();
        fs::write(debug.join("first.sifrmeta"), &bytes).unwrap();
        fs::write(debug.join("second.sifrmeta"), &second).unwrap();
        panic!(
            "metadata producer is nondeterministic; artifacts at {}",
            debug.display()
        );
    }
    writeln!(
        std::io::stderr(),
        "DX6 inventory modules={} bytes={} compiler={} producer_seconds={}",
        inputs.sources.len(),
        bytes.len(),
        identity.as_str(),
        started.elapsed().as_secs_f64()
    )
    .unwrap();
}
#[test]
fn failure_recovery_stale_override_and_missing_entry() {
    let scratch = Scratch::new();
    let cache = scratch.0.join("cache");
    let source = root();
    let id = identity("failure-recovery");
    let cancel = AtomicBool::new(false);
    let failure = ensure_with_hook(&id, &source, TARGET, &cache, &cancel, |stage| {
        if stage == Stage::Owned {
            Err(wire::MetadataError(
                "injected original producer failure".into(),
            ))
        } else {
            Ok(())
        }
    });
    assert!(
        failure
            .err()
            .unwrap()
            .0
            .contains("injected original producer failure")
    );
    let prepared = ensure_development_metadata(&id, &source, TARGET, &cache, &cancel).unwrap();
    assertion(&prepared, &id, &source);
    assert!(
        validate_development_metadata(
            &identity("incompatible-family"),
            &source,
            TARGET,
            &prepared.path
        )
        .is_err()
    );
    fs::remove_file(&prepared.path).unwrap();
    let rebuilt = ensure_development_metadata(&id, &source, TARGET, &cache, &cancel).unwrap();
    assert_eq!(prepared.metadata_id, rebuilt.metadata_id);
    assert!(rebuilt.path.is_file());
    fs::write(&rebuilt.path, b"incomplete metadata").unwrap();
    assert!(validate_development_metadata(&id, &source, TARGET, &rebuilt.path).is_err());
    assert_eq!(
        ensure_development_metadata(&id, &source, TARGET, &cache, &cancel)
            .unwrap()
            .metadata_id,
        prepared.metadata_id
    );
}
#[test]
fn source_mutation_cannot_publish_mislabeled_metadata() {
    let scratch = Scratch::new();
    let source = scratch.source();
    let cache = scratch.0.join("cache");
    let id = identity("source-mutation");
    let file = source.join("stdlib/sifr/math.sifr");
    let original = fs::read(&file).unwrap();
    let result = ensure_with_hook(
        &id,
        &source,
        TARGET,
        &cache,
        &AtomicBool::new(false),
        |stage| {
            if stage == Stage::Staged {
                let mut changed = original.clone();
                changed.extend_from_slice(b"\n# changed while staged\n");
                fs::write(&file, changed).unwrap();
            }
            Ok(())
        },
    );
    assert!(result.err().unwrap().0.contains("inputs changed"));
    assert!(!fs::read_dir(cache.join("metadata")).unwrap().any(|entry| {
        entry
            .unwrap()
            .path()
            .extension()
            .is_some_and(|v| v == "sifrmeta")
    }));
    fs::write(&file, original).unwrap();
    let result =
        ensure_development_metadata(&id, &source, TARGET, &cache, &AtomicBool::new(false)).unwrap();
    assertion(&result, &id, &source);
}
#[test]
fn threads_share_one_success_and_execute_independent_assertions() {
    let scratch = Scratch::new();
    let source = root();
    let cache = scratch.0.join("cache");
    let id = identity("dx6-threads");
    let productions = AtomicUsize::new(0);
    let assertions = AtomicUsize::new(0);
    let barrier = std::sync::Barrier::new(4);
    let started = Instant::now();
    let values = std::thread::scope(|scope| {
        let handles = (0..4)
            .map(|_| {
                scope.spawn(|| {
                    barrier.wait();
                    let value = ensure_with_hook(
                        &id,
                        &source,
                        TARGET,
                        &cache,
                        &AtomicBool::new(false),
                        |stage| {
                            if stage == Stage::Published {
                                productions.fetch_add(1, Ordering::SeqCst);
                            }
                            Ok(())
                        },
                    )
                    .unwrap();
                    assertion(&value, &id, &source);
                    assertions.fetch_add(1, Ordering::SeqCst);
                    value
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(productions.load(Ordering::SeqCst), 1);
    assert_eq!(assertions.load(Ordering::SeqCst), 4);
    for value in &values[1..] {
        assert!(Arc::ptr_eq(&values[0], value));
    }
    writeln!(
        std::io::stderr(),
        "DX6 threads=4 productions=1 assertions=4 seconds={}",
        started.elapsed().as_secs_f64()
    )
    .unwrap();
}
#[cfg(unix)]
#[test]
fn dx6_readonly_source_writable_cache_and_output_diagnostics() {
    let scratch = Scratch::new();
    let source = scratch.source();
    let cache = scratch.0.join("cache");
    permissions(&source, true);
    let id = identity("dx6-readonly");
    let prepared =
        ensure_development_metadata(&id, &source, TARGET, &cache, &AtomicBool::new(false)).unwrap();
    assertion(&prepared, &id, &source);
    let output = scratch.0.join("stdlib.sifrmeta");
    prepared.publish_output(&output).unwrap();
    assert_eq!(
        fs::read(&output).unwrap(),
        fs::read(&prepared.path).unwrap()
    );
    assert!(
        prepared
            .publish_output(&scratch.0.join("missing/output.sifrmeta"))
            .err()
            .unwrap()
            .0
            .contains("output parent")
    );
    let readonly = scratch.0.join("readonly");
    fs::create_dir(&readonly).unwrap();
    permissions(&readonly, true);
    assert!(
        prepared
            .publish_output(&readonly.join("output.sifrmeta"))
            .err()
            .unwrap()
            .0
            .contains("writable output directory")
    );
    assert!(
        ensure_development_metadata(
            &id,
            &source,
            TARGET,
            &readonly.join("cache"),
            &AtomicBool::new(false)
        )
        .is_err()
    );
}
#[test]
fn dx6_r10_linked_real_stdlib_test_adapter() {
    let context = crate::CompilerContext::for_test();
    context.ensure_development_metadata().unwrap();
    assert!(
        crate::type_check_source(
            &context,
            "from sifr.math import sqrt\n\ndef main() -> float:\n    return sqrt(9.0)\n"
        )
        .is_empty()
    );
}

#[test]
fn dx6_prepare_test_metadata() {
    let context = crate::CompilerContext::for_test();
    let started = Instant::now();
    let metadata = ensure_development_metadata(
        context.identity(),
        &root(),
        TARGET,
        &crate::cache_storage::root(),
        &AtomicBool::new(false),
    )
    .unwrap();
    metadata.store.validate_complete().unwrap();
    writeln!(
        std::io::stderr(),
        "DX6 prepared {}",
        serde_json::json!({"compiler_identity":context.identity().as_str(),
        "metadata_id":metadata.metadata_id,"path":metadata.path,
        "production_seconds":metadata.production_seconds,"elapsed_seconds":started.elapsed().as_secs_f64()})
    )
    .unwrap();
}

#[test]
fn portable_records_ignore_only_the_producer_envelope() {
    let a = Inputs::capture(&identity("dx8-envelope-a"), &root(), TARGET).unwrap();
    let b = Inputs::capture(&identity("dx8-envelope-b"), &root(), TARGET).unwrap();
    assert_eq!(a.compatibility.stdlib_inputs, b.compatibility.stdlib_inputs);
    assert_ne!(a.compatibility.compiler, b.compatibility.compiler);
    let a_bytes = a.produce().unwrap();
    let b_bytes = b.produce().unwrap();
    assert_ne!(a_bytes, b_bytes);
    let a_store = wire::MetadataStore::open(
        std::io::Cursor::new(a_bytes),
        a.compatibility,
        wire::Limits::default(),
    )
    .unwrap();
    let b_store = wire::MetadataStore::open(
        std::io::Cursor::new(b_bytes),
        b.compatibility,
        wire::Limits::default(),
    )
    .unwrap();
    assert_eq!(
        a_store.portable_payload_digest().unwrap(),
        b_store.portable_payload_digest().unwrap()
    );
    // Different semantic targets are deliberately not comparable.
    let other = if TARGET == "aarch64-apple-darwin" {
        "x86_64-unknown-linux-gnu"
    } else {
        "aarch64-apple-darwin"
    };
    let changed = Inputs::capture(&identity("dx8-envelope-a"), &root(), other).unwrap();
    assert_ne!(
        a.compatibility.semantic_target,
        changed.compatibility.semantic_target
    );
    assert_ne!(
        a.compatibility.stdlib_inputs,
        changed.compatibility.stdlib_inputs
    );
}

#[cfg(windows)]
#[test]
fn windows_portability_metadata_staged_failure_and_output_acl() {
    let scratch = Scratch::new();
    let cache = scratch.0.join("cache");
    let source = root();
    let id = identity("windows-stage-recovery");
    let cancel = AtomicBool::new(false);
    let failure = ensure_with_hook(&id, &source, TARGET, &cache, &cancel, |stage| {
        if stage == Stage::Staged {
            Err(wire::MetadataError("injected staged failure".into()))
        } else {
            Ok(())
        }
    });
    assert!(failure.err().unwrap().0.contains("injected staged failure"));
    let staging = fs::read_dir(cache.join("metadata"))
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "stage")
        })
        .unwrap();
    crate::cache_storage::check_owned(&staging).unwrap();
    let prepared = ensure_development_metadata(&id, &source, TARGET, &cache, &cancel).unwrap();
    assert!(!staging.exists());
    let output = scratch.0.join("output.sifrmeta");
    prepared.publish_output(&output).unwrap();
    crate::cache_storage::check_owned(&output).unwrap();
    let intact = fs::read(&output).unwrap();
    crate::windows_storage_security::test_grant_world(&output).unwrap();
    assert!(prepared.publish_output(&output).is_err());
    assert_eq!(fs::read(&output).unwrap(), intact);
    crate::windows_storage_security::seal(&output).unwrap();
}
