#![expect(
    clippy::print_stderr,
    reason = "explicit invocation-equivalence measurements"
)]
use super::e2e_entrypoints::read_dir_file_paths_sorted;
use super::*;
use sifr_driver::{CompileResultFull, CompilerContext, SingleFileCompilationSession};

fn equivalent(name: &str, left: &CompileResultFull, right: &CompileResultFull) {
    match (left, right) {
        (
            CompileResultFull::Success {
                rust_source: a,
                generated_source_map: am,
                used_stdlib_modules: au,
                required_features: af,
                interop: ai,
                lowering_stats: ast,
            },
            CompileResultFull::Success {
                rust_source: b,
                generated_source_map: bm,
                used_stdlib_modules: bu,
                required_features: bf,
                interop: bi,
                lowering_stats: bst,
            },
        ) => {
            assert!(a == b, "generated Rust differs: {name}");
            assert_eq!(am, bm, "source maps: {name}");
            assert_eq!(au, bu, "modules: {name}");
            assert_eq!(af, bf, "features: {name}");
            assert_eq!(ai, bi, "interop: {name}");
            assert_eq!(
                format!("{ast:?}"),
                format!("{bst:?}"),
                "lowering statistics: {name}"
            );
        }
        (CompileResultFull::Errors { errors: a }, CompileResultFull::Errors { errors: b }) => {
            assert_eq!(
                format!("{a:?}"),
                format!("{b:?}"),
                "complete diagnostics: {name}"
            )
        }
        _ => panic!("success/error contract differs: {name}"),
    }
}

#[test]
fn test_session_repetition_concurrency_and_changed_metadata_inputs() {
    let sources = [
        "def main():\n    assert 2 + 3 == 5\n",
        "def broken(:\n",
        "def main():\n    print(missing_name)\n",
    ];
    let context = CompilerContext::for_test();
    let session = SingleFileCompilationSession::new(context.clone()).unwrap();
    let expected = sources
        .iter()
        .map(|source| sifr_driver::compile_with_metadata(&CompilerContext::for_test(), source))
        .collect::<Vec<_>>();
    for workers in [1, 4, 2] {
        let outputs = run_in_parallel(&sources, workers, |source| {
            session.compile_with_metadata(source)
        });
        for (index, output) in outputs.iter().enumerate() {
            equivalent(sources[index], &expected[index], output);
        }
    }
    let temporary = tempfile::tempdir().unwrap();
    let changed = context
        .clone()
        .with_metadata_override(temporary.path().join("missing.sifrm"));
    let baseline = sifr_driver::compile_with_metadata(&changed, sources[0]);
    let errors = match SingleFileCompilationSession::new(changed) {
        Ok(_) => panic!("changed metadata reused an earlier provider"),
        Err(errors) => errors,
    };
    equivalent(
        "changed metadata",
        &baseline,
        &CompileResultFull::Errors { errors },
    );
    equivalent(
        "earlier owner remains valid",
        &expected[0],
        &session.compile_with_metadata(sources[0]),
    );
    // A newly selected root must not reuse the earlier metadata owner, even if
    // that root becomes unavailable between resolution and compilation.
    let mut selected = context.sysroot().unwrap().clone();
    selected.root = temporary.path().join("unavailable-selected-sysroot");
    let changed_root = CompilerContext::with_sysroot(context.identity().clone(), selected);
    let baseline = sifr_driver::compile_with_metadata(&changed_root, sources[0]);
    let errors = match SingleFileCompilationSession::new(changed_root) {
        Ok(_) => panic!("changed sysroot reused an earlier provider"),
        Err(errors) => errors,
    };
    equivalent(
        "changed sysroot",
        &baseline,
        &CompileResultFull::Errors { errors },
    );
    let refreshed = SingleFileCompilationSession::new(context.refreshed_toolchain()).unwrap();
    equivalent(
        "new toolchain generation",
        &expected[0],
        &refreshed.compile_with_metadata(sources[0]),
    );
    let development = CompilerContext::for_test()
        .with_application_profile(sifr_driver::ApplicationProfile::Development);
    let baseline = sifr_driver::compile_with_metadata(&development, sources[0]);
    let changed = SingleFileCompilationSession::new(development).unwrap();
    equivalent(
        "changed profile",
        &baseline,
        &changed.compile_with_metadata(sources[0]),
    );
}

#[test]
#[ignore = "explicit full-corpus setup-equivalence qualification; run for session changes"]
fn test_session_full_corpus_equivalence() {
    for directory in ["tests/e2e/pass", "tests/e2e/fail"] {
        let paths = read_dir_file_paths_sorted(Path::new(directory))
            .into_iter()
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sifr"))
            .collect::<Vec<_>>();
        let sources = paths
            .iter()
            .map(|path| std::fs::read_to_string(path).unwrap())
            .collect::<Vec<_>>();
        let started = Instant::now();
        let baseline = run_in_parallel(&sources, 4, |source| {
            sifr_driver::compile_with_metadata(&CompilerContext::for_test(), source)
        });
        let baseline_ms = started.elapsed().as_millis();
        let started = Instant::now();
        let session = SingleFileCompilationSession::new(CompilerContext::for_test()).unwrap();
        let shared = run_in_parallel(&sources, 4, |source| session.compile_with_metadata(source));
        let shared_ms = started.elapsed().as_millis();
        assert_eq!(baseline.len(), paths.len());
        assert_eq!(shared.len(), paths.len());
        for (index, (left, right)) in baseline.iter().zip(&shared).enumerate() {
            equivalent(&paths[index].display().to_string(), left, right);
        }
        // Reverse source order on the same owner to expose per-fixture leakage.
        let repeated = run_in_parallel(&(0..sources.len()).rev().collect::<Vec<_>>(), 2, |index| {
            (*index, session.compile_with_metadata(&sources[*index]))
        });
        for (index, result) in repeated {
            equivalent(
                &paths[index].display().to_string(),
                &baseline[index],
                &result,
            );
        }
        eprintln!(
            "[sifr-e4-equivalence] corpus={directory} fixtures={} baseline_ms={baseline_ms} session_ms={shared_ms} repeated_order=reverse fields=rust,maps,modules,features,interop,stats,diagnostics",
            paths.len()
        );
    }
}

#[cfg(unix)]
#[test]
fn test_session_native_selection_is_once_per_invocation_and_check_is_frontend_only() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap().keep();
    let tool = |name| {
        let output = Command::new("rustup")
            .args(["which", name])
            .output()
            .unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    };
    let rustc = tool("rustc");
    let cargo = tool("cargo");
    let quote = |value: &str| format!("'{}'", value.replace('\'', "'\"'\"'"));
    for configuration in ["first", "changed"] {
        let wrapper = temp.join(format!("{configuration}-rustc"));
        let log = temp.join(format!("{configuration}.log"));
        std::fs::write(
            &wrapper,
            format!(
                "#!/bin/sh\nprintf 'probe\\n' >> {}\nexec {} \"$@\"\n",
                quote(&log.display().to_string()),
                quote(&rustc)
            ),
        )
        .unwrap();
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut child = Command::new(std::env::current_exe().unwrap());
        child
            .args([
                "--ignored",
                "--exact",
                "e2e_support::session_tests::test_session_native_probe_child",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("SIFR_CARGO", &cargo)
            .env("SIFR_RUSTC", &wrapper)
            .env("SIFR_SESSION_PROBE_LOG", &log);
        if configuration == "changed" {
            child.env("RUSTFLAGS", "--cfg sifr_session_control");
        }
        let output = sifr_driver::process_execution::output_with_deadline(
            &mut child,
            std::time::Duration::from_secs(60),
        )
        .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    std::fs::remove_dir_all(temp).unwrap();
}

#[cfg(unix)]
#[test]
#[ignore = "owned child of the native-selection control"]
fn test_session_native_probe_child() {
    let log =
        PathBuf::from(std::env::var("SIFR_SESSION_PROBE_LOG").expect("owned child configuration"));
    let probes = || {
        std::fs::read_to_string(&log)
            .unwrap_or_default()
            .lines()
            .count()
    };
    let source = "def main():\n    assert 1 == 1\n";
    let context = CompilerContext::for_test();
    assert!(sifr_driver::check(&context, source).is_empty());
    assert_eq!(probes(), 0, "check performed native preparation");
    assert!(matches!(
        sifr_driver::compile_with_metadata(&context, source),
        CompileResultFull::Success { .. }
    ));
    for invalid in ["def broken(:\n", "def main():\n    print(missing_name)\n"] {
        assert!(matches!(
            sifr_driver::compile_with_metadata(&context, invalid),
            CompileResultFull::Errors { .. }
        ));
    }
    assert_eq!(
        probes(),
        0,
        "standalone non-interop emission prepared native tools"
    );

    let native_source =
        "from sifr.gzip import compress\ndef main():\n    _ = compress(\"sample\")\n";
    let CompileResultFull::Success { interop, .. } =
        sifr_driver::compile_with_metadata(&context, native_source)
    else {
        panic!("demanded stdlib interop failed to emit");
    };
    assert!(!interop.rust.declarations.is_empty());
    assert!(interop.rust.cargo_inputs.is_some());
    assert_eq!(
        probes(),
        1,
        "standalone interop must resolve native tools exactly once"
    );

    // A standalone call owns a new input snapshot even in the same process.
    // The prior invocation's resolved interop must not bypass native selection.
    assert!(matches!(
        sifr_driver::compile_with_metadata(&context, native_source),
        CompileResultFull::Success { .. }
    ));
    assert_eq!(
        probes(),
        2,
        "standalone reused process-wide native metadata"
    );

    let first = SingleFileCompilationSession::new(context.clone()).unwrap();
    let after_first = probes();
    assert_eq!(
        after_first, 3,
        "session must capture its own native selection"
    );
    for _ in 0..3 {
        for input in [source, native_source] {
            assert!(matches!(
                first.compile_with_metadata(input),
                CompileResultFull::Success { .. }
            ));
        }
    }
    assert_eq!(
        probes(),
        after_first,
        "source compilation repeated native resolution"
    );
    let second = SingleFileCompilationSession::new(context).unwrap();
    assert!(
        probes() > after_first,
        "new invocation reused an old native selection"
    );
    assert!(matches!(
        second.compile_with_metadata(source),
        CompileResultFull::Success { .. }
    ));
}
