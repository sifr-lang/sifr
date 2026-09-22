use super::cargo_resolution::CargoResolutionPolicy;
use super::materialize::tests::{base_project, test_dependency_plan};
use super::materialize::{materialize_binary_project_at_path, materialize_binary_project_files};
use sifr_package::CargoLockMode;
use sifr_stdlib_manifest::CargoVendorMode;
use std::path::{Path, PathBuf};

fn root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "sifr-dx9-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos(),
    ));
    std::fs::create_dir_all(&root).expect("owned fixture");
    root
}

fn build(
    root: &Path,
    source: &str,
) -> Result<PathBuf, Vec<crate::diagnostics::RenderedDiagnostic>> {
    let mut plan = test_dependency_plan("dx9");
    plan.cargo_vendor_mode = CargoVendorMode::PackageOwned;
    let mut project = base_project();
    project.main_rs = source.to_owned();
    materialize_binary_project_at_path(
        root,
        "same_name",
        project,
        &plan,
        &CargoResolutionPolicy::normal(),
    )
    .map(|report| report.binary_path)
}

#[test]
fn dx9_unchanged_materialization_preserves_input_mtimes() {
    let root = root("mtime");
    let plan = test_dependency_plan("dx9");
    materialize_binary_project_files(&root, "same_name", base_project(), &plan).expect("first");
    let paths = [root.join("Cargo.toml"), root.join("src/main.rs")];
    let before: Vec<_> = paths
        .iter()
        .map(|p| {
            std::fs::metadata(p)
                .expect("file")
                .modified()
                .expect("mtime")
        })
        .collect();
    materialize_binary_project_files(&root, "same_name", base_project(), &plan).expect("second");
    let after: Vec<_> = paths
        .iter()
        .map(|p| {
            std::fs::metadata(p)
                .expect("file")
                .modified()
                .expect("mtime")
        })
        .collect();
    assert_eq!(before, after);
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn dx9_f7_locked_root_reconciles_alternating_runtime_dependency() {
    let root = root("locked-manifest-drift");
    let dependency = root.join("sifr_runtime");
    std::fs::create_dir_all(dependency.join("src")).expect("runtime source");
    std::fs::write(
        dependency.join("Cargo.toml"),
        "[package]\nname = \"sifr_runtime\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("runtime manifest");
    std::fs::write(
        dependency.join("src/lib.rs"),
        "pub fn value() -> &'static str { \"runtime\" }\n",
    )
    .expect("runtime source");
    let authority = root.join("authority.lock");
    std::fs::write(&authority, "version = 4\n").expect("lock authority");
    let policy = CargoResolutionPolicy {
        application_profile: crate::ApplicationProfile::Release,
        native_toolchain: CargoResolutionPolicy::resolve_native_toolchain(),
        lock_mode: CargoLockMode::Locked,
        cargo_vendor_mode: CargoVendorMode::PackageOwned,
        authoritative_locks: vec![authority],
        trusted_vendor_dirs: Vec::new(),
    };
    let tools = policy.native_toolchain.as_ref().expect("native toolchain");
    let family = super::native_storage::NativeFamily::acquire(
        tools.identity(),
        &root.display().to_string(),
        "",
        "",
    )
    .expect("family lease");
    let project_root = family.project(&root, "same_name").expect("editable root");
    let target = family.target();
    for needs_runtime in [false, true, false, true] {
        let mut plan = test_dependency_plan("dx9-f7");
        plan.cargo_vendor_mode = CargoVendorMode::PackageOwned;
        if needs_runtime {
            plan.retained_direct_dependencies
                .push(format!("sifr_runtime = {{ path = {dependency:?} }}"));
        }
        let mut project = base_project();
        project.main_rs = if needs_runtime {
            "fn main() { println!(\"{}\", sifr_runtime::value()); }".to_owned()
        } else {
            "fn main() { println!(\"plain\"); }".to_owned()
        };
        let report = super::materialize::materialize_binary_project_at_path_with_target(
            &project_root,
            "same_name",
            project,
            &plan,
            &policy,
            &target,
        )
        .expect("locked build must reconcile the generated manifest and lock");
        let output = std::process::Command::new(report.binary_path)
            .output()
            .expect("run locked executable");
        assert!(output.status.success());
        let expected = if needs_runtime {
            "runtime\n"
        } else {
            "plain\n"
        };
        assert_eq!(output.stdout, expected.as_bytes());
        let lock = std::fs::read_to_string(project_root.join("Cargo.lock"))
            .expect("prepared generated lock");
        assert_eq!(lock.contains("name = \"sifr_runtime\""), needs_runtime);
        assert!(target.is_dir(), "the shared Cargo target must remain warm");
    }
    let family_root = family.root.clone();
    drop(family);
    std::fs::remove_dir_all(family_root).expect("family cleanup");
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn dx9_real_same_name_root_error_lint_and_native_stage_failures() {
    let root = root("stages");
    let good = build(&root.join("one"), "fn main() { println!(\"first\"); }").expect("valid root");
    assert_eq!(
        std::process::Command::new(&good)
            .output()
            .expect("run")
            .stdout,
        b"first\n"
    );
    let error = build(&root.join("two"), "fn main() { let _: u32 = \"wrong\"; }")
        .expect_err("invalid same-name root");
    assert!(error.iter().any(|e| e.message.contains("mismatched types")));
    let lint = build(
        &root.join("three"),
        "#![deny(unused_variables)]\nfn main() { let unused = 1; }",
    )
    .expect_err("lint");
    assert!(lint.iter().any(|e| e.message.contains("unused variable")));
    let link = build(&root.join("four"), "#[link(name = \"sifr_dx9_nonexistent_native_library\")] unsafe extern \"C\" {}\nfn main() {}").expect_err("link");
    assert!(
        link.iter()
            .any(|e| e.message.contains("linking") || e.message.contains("linker"))
    );
    let failing =
        build(&root.join("five"), "fn main() { std::process::exit(7); }").expect("link success");
    assert_eq!(
        std::process::Command::new(failing)
            .status()
            .expect("run")
            .code(),
        Some(7)
    );
    // An unsuccessful replacement does not return the previous executable.
    build(&root.join("one"), "fn main() { absent(); }").expect_err("failed rebuild");
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn dx9_real_concurrent_same_root_capture_is_serialized() {
    let root = root("concurrent");
    let threads: Vec<_> = (0..4)
        .map(|value| {
            let path = root.join("app");
            std::thread::spawn(move || {
                // The publication path is deliberately caller-owned for this API.
                // Different native contexts still serialize the caller-owned root.
                let publication =
                    super::native_storage::publication_lock(&path).expect("publication lease");
                let tools = CargoResolutionPolicy::normal()
                    .native_toolchain
                    .expect("tools");
                let family = super::native_storage::NativeFamily::acquire(
                    tools.identity(),
                    "dx9-concurrent",
                    &value.to_string(),
                    "",
                )
                .expect("family");
                let mut project = base_project();
                project.main_rs = format!("fn main() {{ println!(\"{value}\"); }}");
                let mut plan = test_dependency_plan("dx9");
                plan.cargo_vendor_mode = CargoVendorMode::PackageOwned;
                let report = super::materialize::materialize_binary_project_at_path_with_target(
                    &path,
                    "same_name",
                    project,
                    &plan,
                    &CargoResolutionPolicy::normal(),
                    &family.target(),
                )
                .expect("compile");
                let captured = family.root.join(format!("captured-{value}"));
                std::fs::copy(report.binary_path, &captured).expect("capture");
                drop(family);
                drop(publication);
                assert_eq!(
                    std::process::Command::new(captured)
                        .output()
                        .expect("run")
                        .stdout,
                    format!("{value}\n").as_bytes()
                );
            })
        })
        .collect();
    for thread in threads {
        thread.join().expect("worker");
    }
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn dx9_build_script_inputs_rebuild_unchanged_generated_root() {
    let root = root("build-script");
    let dependency = root.join("dependency");
    std::fs::create_dir_all(dependency.join("src")).expect("dependency");
    std::fs::write(
        dependency.join("Cargo.toml"),
        r#"[package]
name = "dx9_dependency"
version = "0.0.0"
edition = "2024"
"#,
    )
    .expect("manifest");
    std::fs::write(dependency.join("build.rs"),
        r#"fn main() { println!("cargo:rerun-if-changed=value.txt"); println!("cargo:rustc-env=VALUE={}", std::fs::read_to_string("value.txt").unwrap().trim()); }"#).expect("build script");
    std::fs::write(
        dependency.join("src/lib.rs"),
        r#"pub fn value() -> &'static str { env!("VALUE") }"#,
    )
    .expect("library");
    let mut plan = test_dependency_plan("dx9");
    plan.cargo_vendor_mode = CargoVendorMode::PackageOwned;
    plan.retained_direct_dependencies
        .push(format!("dx9_dependency = {{ path = {dependency:?} }}"));
    for value in ["first", "second"] {
        std::fs::write(dependency.join("value.txt"), value).expect("changed input");
        let mut project = base_project();
        project.main_rs = r#"fn main() { println!("{}", dx9_dependency::value()); }"#.to_owned();
        let report = materialize_binary_project_at_path(
            &root.join("app"),
            "same_name",
            project,
            &plan,
            &CargoResolutionPolicy::normal(),
        )
        .expect("Cargo must inspect dependency build-script freshness");
        assert_eq!(
            std::process::Command::new(report.binary_path)
                .output()
                .expect("run")
                .stdout,
            format!(
                "{value}
"
            )
            .as_bytes()
        );
    }
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn dx9_finalized_artifact_survives_same_root_edit_and_runs_again() {
    let root = root("finalized");
    let compile = |source: &str| {
        let mut project = base_project();
        project.main_rs = source.to_owned();
        super::materialize::materialize_cached_binary_project_with_report(
            "dx9-tests",
            &root,
            "same_name",
            project,
            CargoVendorMode::SysrootOnly,
            &CargoResolutionPolicy::normal(),
        )
        .expect("finalized build")
        .0
    };
    let first = compile(r#"fn main() { println!("first"); }"#);
    let (repeat, invocations) = super::cargo_invocation_trace::capture_cargo_invocations(|| {
        compile(r#"fn main() { println!("first"); }"#)
    });
    assert!(
        invocations
            .iter()
            .any(|invocation| invocation.phase == "final-build")
    );
    assert!(repeat.report().cache_hit());
    assert_eq!(first.workspace_root(), repeat.workspace_root());
    let second = compile(r#"fn main() { println!("second"); }"#);
    assert_ne!(first.workspace_root(), second.workspace_root());
    for (entry, expected) in [
        (
            &first, "first
",
        ),
        (
            &second, "second
",
        ),
    ] {
        let binary = super::materialize::cached_binary_path(entry.workspace_root(), "same_name");
        assert!(!binary.starts_with(crate::cache_storage::root().join("native/families")));
        assert_eq!(
            std::process::Command::new(binary)
                .output()
                .expect("run finalized")
                .stdout,
            expected.as_bytes()
        );
    }
    std::fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(target_os = "linux")]
#[test]
fn dx9_runtime_library_bundle_survives_mutable_cargo_output_removal() {
    let root = root("dylib");
    let dependency = root.join("dependency");
    std::fs::create_dir_all(dependency.join("src")).expect("dependency");
    std::fs::write(
        dependency.join("Cargo.toml"),
        "[package]\nname = \"dx9_dylib\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("manifest");
    std::fs::write(
        dependency.join("value.c"),
        "int dx9_value(void) { return 42; }\n",
    )
    .expect("C source");
    std::fs::write(dependency.join("build.rs"), r#"fn main() {
        let out = std::env::var("OUT_DIR").unwrap();
        assert!(std::process::Command::new("cc").args(["-shared", "-fPIC", "value.c", "-o"]).arg(format!("{out}/libdx9_value.so")).status().unwrap().success());
        println!("cargo:rerun-if-changed=value.c");
        println!("cargo:rustc-link-search=native={out}");
        println!("cargo:rustc-link-lib=dylib=dx9_value");
    }"#).expect("build script");
    std::fs::write(
        dependency.join("src/lib.rs"),
        r#"unsafe extern "C" { fn dx9_value() -> i32; }
pub fn value() -> i32 { unsafe { dx9_value() } }"#,
    )
    .expect("library");
    let mut plan = test_dependency_plan("dx9");
    plan.cargo_vendor_mode = CargoVendorMode::PackageOwned;
    plan.retained_direct_dependencies
        .push(format!("dx9_dylib = {{ path = {dependency:?} }}"));
    let mut project = base_project();
    project.main_rs = r#"fn main() { println!("{}", dx9_dylib::value()); }"#.to_owned();
    let policy = CargoResolutionPolicy::normal();
    let tools = policy.native_toolchain.as_ref().expect("tools");
    let family = super::native_storage::NativeFamily::acquire(
        tools.identity(),
        &root.display().to_string(),
        "",
        "dx9-dylib",
    )
    .expect("family");
    let report = super::materialize::materialize_binary_project_at_path_with_target(
        &root.join("app"),
        "same_name",
        project,
        &plan,
        &policy,
        &family.target(),
    )
    .expect("native dylib build");
    assert!(!report.native_libraries.is_empty());
    let snapshot = super::native_storage::NativeSnapshot::inspect(
        &report.native_executable,
        Path::new("program"),
    )
    .and_then(|snapshot| snapshot.with_runtime(&report.native_libraries, Path::new("program")))
    .expect("bundle inventory");
    let final_root = root.join("final");
    snapshot.capture(&final_root).expect("independent capture");
    for library in report.native_libraries {
        std::fs::remove_file(library).expect("remove mutable output");
    }
    drop(family);
    let output = std::process::Command::new(final_root.join("program"))
        .env_remove("LD_LIBRARY_PATH")
        .output()
        .expect("run captured bundle");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"42\n");
    std::fs::remove_dir_all(root).expect("cleanup");
}
