use super::artifacts::{
    compose_test_runner_lib, test_support_module_file_path, try_generate_test_runner_cargo_plan,
};
use super::orchestrator::GeneratedTestRunnerProject;
use crate::build::{
    ArtifactCacheReport, PreparedArtifactCache, cargo_lock_mode_diagnostic,
    configure_hermetic_build_environment, prepare_cached_artifact, prepare_cargo_resolution,
    record_cargo_invocation, sysroot_cargo_config_args, validate_test_native_link_evidence,
};
use crate::diagnostics::{RenderedDiagnostic, write_stderr, write_stderr_line};
use crate::project::namespace_module_files;
use sifr_diagnostics::DiagnosticCode;
use sifr_stdlib_manifest::SysrootDependencyPlan;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

pub(crate) struct TestRunnerExecutionOutcome {
    pub(crate) success: bool,
    #[cfg(test)]
    pub(crate) native_project_root: PathBuf,
    #[cfg(test)]
    pub(crate) final_workspace_root: PathBuf,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) cache_report: ArtifactCacheReport,
}

pub(crate) fn execute_test_runner_project(
    generated_project: &GeneratedTestRunnerProject,
) -> Result<TestRunnerExecutionOutcome, Vec<RenderedDiagnostic>> {
    let cargo_resolution = &generated_project.cargo_resolution;
    let native_toolchain = cargo_resolution
        .native_toolchain
        .as_ref()
        .map_err(|error| {
            vec![crate::diagnostics::diagnostic_with_code(
                error.clone(),
                DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE,
            )]
        })?;
    native_toolchain
        .validate_configuration()
        .map_err(test_io_error)?;
    let mut cargo_plan = try_generate_test_runner_cargo_plan(
        &generated_project.all_stdlib_modules,
        &generated_project.all_required_features,
        &generated_project.interop,
        cargo_resolution.cargo_vendor_mode,
    )
    .map_err(|error| {
        vec![crate::diagnostics::diagnostic_with_code(
            error.boundary_message(),
            DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
        )]
    })?;
    let test_lib = compose_test_runner_lib(
        &generated_project.support_module_names,
        &generated_project.all_rust_code,
    );
    let mut cache_key = test_runner_cache_key(
        generated_project,
        &cargo_plan.cargo_toml,
        &test_lib,
        &cargo_plan.dependency_plan,
    )?;
    cache_key.push_str(generated_project.application_profile.name());
    cache_key.push_str(generated_project.application_profile.policy_identity());
    cache_key.push_str("\n[native-toolchain]\n");
    cache_key.push_str(native_toolchain.identity());
    cache_key.push_str("\n[resolution-policy]\n");
    cache_key.push_str(cargo_resolution.lock_mode.as_str());
    if let Some(seed) = cargo_resolution.normal_seed_cache_fragment() {
        cache_key.push_str(&seed);
    }
    let family = crate::build::native_storage::NativeFamily::acquire(
        native_toolchain.identity(),
        &format!(
            "{:?}:{}:{:?}",
            cargo_plan.dependency_plan.cargo_vendor_mode,
            cargo_plan.dependency_plan.sysroot_root.display(),
            cargo_resolution.normal_seed_cache_fragment()
        ),
        "",
        &format!("{:?}", generated_project.interop.rust.trust_requirements),
    )
    .map_err(test_io_error)?;
    let project_dir = family
        .project(&generated_project.cache_scope, "sifr_tests")
        .map_err(test_io_error)?;
    let root_id = sifr_sysroot::sha256_hex(project_dir.as_os_str().as_encoded_bytes());
    let _ = write!(
        cargo_plan.cargo_toml,
        "\n[lib]\nname = \"sifr_tests_{}\"\n",
        &root_id[..16]
    );
    {
        let src_dir = project_dir.join("src");
        let mut current_files = BTreeSet::new();
        std::fs::create_dir_all(&src_dir).map_err(|error| {
            vec![crate::diagnostics::diagnostic_with_code(
                format!("failed to create test directory: {error}"),
                DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
            )]
        })?;

        crate::build::native_storage::write_changed(
            &project_dir.join("Cargo.toml"),
            cargo_plan.cargo_toml.as_bytes(),
        )
        .map_err(|error| {
            vec![crate::diagnostics::diagnostic_with_code(
                format!("failed to write Cargo.toml: {error}"),
                DiagnosticCode::BUILD_CARGO_MANIFEST_FAILURE,
            )]
        })?;

        let support_files = generated_project
            .support_module_names
            .iter()
            .filter_map(|name| {
                generated_project
                    .support_rust_files
                    .get(name)
                    .map(|code| (test_support_module_file_path(name), code))
            });
        let bridge_files = generated_project
            .bridge_rust_files
            .iter()
            .map(|(path, code)| (path.clone(), code));
        for (module_path, code) in support_files.chain(bridge_files) {
            let output_path = src_dir.join(&module_path);
            current_files.insert(output_path.clone());
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    vec![crate::diagnostics::diagnostic_with_code(
                        format!(
                            "failed to create test support module directory '{}': {error}",
                            parent.display()
                        ),
                        DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
                    )]
                })?;
            }
            crate::build::native_storage::write_changed(&output_path, code.as_bytes()).map_err(
                |error| {
                    vec![crate::diagnostics::diagnostic_with_code(
                        format!(
                            "failed to write test support module '{}': {error}",
                            output_path.display()
                        ),
                        DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
                    )]
                },
            )?;
        }

        for namespace_file in namespace_module_files(&generated_project.support_module_names) {
            let output_path = src_dir.join(&namespace_file.path);
            current_files.insert(output_path.clone());
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    vec![crate::diagnostics::diagnostic_with_code(
                        format!(
                            "failed to create test support namespace directory '{}': {error}",
                            parent.display()
                        ),
                        DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
                    )]
                })?;
            }
            let mut contents = String::new();
            for declaration in namespace_file.declarations {
                contents.push_str("pub mod ");
                contents.push_str(&declaration);
                contents.push_str(";\n");
            }
            crate::build::native_storage::write_changed(&output_path, contents.as_bytes())
                .map_err(|error| {
                    vec![crate::diagnostics::diagnostic_with_code(
                        format!(
                            "failed to write test support namespace '{}': {error}",
                            output_path.display()
                        ),
                        DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
                    )]
                })?;
        }

        current_files.insert(src_dir.join("lib.rs"));
        crate::build::native_storage::write_changed(&src_dir.join("lib.rs"), test_lib.as_bytes())
            .map_err(|error| {
            vec![crate::diagnostics::diagnostic_with_code(
                format!("failed to write lib.rs: {error}"),
                DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
            )]
        })?;
        crate::build::native_storage::remove_stale(&src_dir, &current_files)
            .map_err(test_io_error)?;
    }

    crate::build::native_storage::write_changed(
        &project_dir.join("build.rs"),
        crate::build::native_storage::loader_build_script(None).as_bytes(),
    )
    .map_err(test_io_error)?;
    native_toolchain
        .validate_application_profile(
            generated_project.application_profile.cargo_name(),
            &project_dir.join("Cargo.toml"),
        )
        .map_err(test_io_error)?;
    let cargo_prefix_args = sysroot_cargo_config_args(&cargo_plan.dependency_plan);
    let prepared_resolution =
        prepare_cargo_resolution(&project_dir, cargo_resolution, &cargo_prefix_args)?;
    write_stderr_line(&format!(
        "application profile: {} ({})",
        generated_project.application_profile.name(),
        generated_project.application_profile.policy_identity()
    ));
    let mut command = cargo_resolution.cargo_command()?;
    command
        .args(&cargo_prefix_args)
        .args([
            "test",
            "--no-run",
            "--message-format=json-render-diagnostics",
        ])
        .arg("--manifest-path")
        .arg(project_dir.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(family.target());
    generated_project
        .application_profile
        .configure(&mut command);
    if let Some(argument) = cargo_resolution.lock_mode.cargo_arg() {
        command.arg(argument);
    }
    configure_hermetic_build_environment(&mut command);
    record_cargo_invocation("final-test", cargo_resolution.lock_mode, &command);
    let output = crate::process_execution::output(&mut command).map_err(test_io_error)?;
    write_stderr(&String::from_utf8_lossy(&output.stderr));
    validate_test_native_link_evidence(
        &output.stdout,
        &generated_project.interop,
        &cargo_plan.dependency_plan,
    )?;
    if !output.status.success() {
        let mut stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if let Ok(event) = serde_json::from_str::<serde_json::Value>(line)
                && let Some(rendered) = event["message"]["rendered"].as_str()
            {
                stderr.push_str(rendered);
            }
        }
        if let Some(diagnostic) = cargo_lock_mode_diagnostic("cargo test", &stderr) {
            return Err(vec![diagnostic]);
        }
        return Err(test_io_error(format!(
            "cargo test preparation failed: {stderr}"
        )));
    }
    prepared_resolution.assert_unchanged()?;
    for event in String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
    {
        if event["reason"] == "compiler-artifact"
            && event["profile"]["test"] == true
            && event["executable"].is_string()
            && event["profile"]["overflow_checks"] != true
        {
            return Err(test_io_error(
                "Cargo disabled required test overflow checks",
            ));
        }
    }
    let cargo_executables = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|event| event["reason"] == "compiler-artifact" && event["profile"]["test"] == true)
        .filter_map(|event| event["executable"].as_str().map(PathBuf::from))
        .collect::<Vec<_>>();
    let runtime_libraries =
        crate::build::native_storage::runtime_libraries(&output.stdout, &family.target())
            .map_err(test_io_error)?;
    let mut executables = Vec::new();
    let mut snapshots = Vec::new();
    for (index, path) in cargo_executables.iter().enumerate() {
        let relative = PathBuf::from(format!("executables/test-{index}"));
        let snapshot = crate::build::native_storage::NativeSnapshot::inspect(path, &relative)
            .and_then(|snapshot| snapshot.with_runtime(&runtime_libraries, &relative))
            .map_err(test_io_error)?;
        cache_key.push_str(&snapshot.identity);
        snapshots.push(snapshot);
        executables.push(relative);
    }
    if executables.is_empty() {
        return Err(test_io_error("Cargo produced no test executable"));
    }
    let mut required_paths = vec![Path::new("test_executables.json")];
    required_paths.extend(
        snapshots
            .iter()
            .flat_map(crate::build::native_storage::NativeSnapshot::required),
    );
    let entry = match prepare_cached_artifact(
        native_toolchain.identity(),
        "test_runner",
        &generated_project.cache_scope,
        &cache_key,
        &required_paths,
    )? {
        PreparedArtifactCache::Hit(entry) => {
            for snapshot in &snapshots {
                snapshot
                    .verify(entry.workspace_root())
                    .map_err(test_io_error)?;
            }
            entry
        }
        PreparedArtifactCache::Miss(pending) => {
            let stage = pending.workspace_root();
            crate::cache_storage::directory(&stage.join("executables")).map_err(test_io_error)?;
            for snapshot in &snapshots {
                snapshot.capture(stage).map_err(test_io_error)?;
            }
            std::fs::write(
                stage.join("test_executables.json"),
                serde_json::to_vec(&executables).map_err(test_io_error)?,
            )
            .map_err(test_io_error)?;
            pending.commit(&required_paths)?
        }
    };
    drop(family);
    if executables.is_empty() {
        return Err(test_io_error("cached test entry contains no executable"));
    }
    let mut success = true;
    for executable in executables {
        let output = crate::process_execution::output(
            std::process::Command::new(entry.workspace_root().join(executable))
                .current_dir(&generated_project.cache_scope),
        )
        .map_err(test_io_error)?;
        write_stderr(&String::from_utf8_lossy(&output.stdout));
        write_stderr(&String::from_utf8_lossy(&output.stderr));
        success &= output.status.success();
    }
    let cache_report = entry.report().clone();
    write_stderr_line(&cache_report.status_line());

    Ok(TestRunnerExecutionOutcome {
        success,
        cache_report,
        #[cfg(test)]
        native_project_root: project_dir,
        #[cfg(test)]
        final_workspace_root: entry.workspace_root().to_path_buf(),
    })
}

fn test_io_error(error: impl std::fmt::Display) -> Vec<RenderedDiagnostic> {
    vec![crate::diagnostics::diagnostic_with_code(
        format!("test subprocess/artifact failure: {error}"),
        DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE,
    )]
}

fn test_runner_cache_key(
    generated_project: &GeneratedTestRunnerProject,
    cargo_toml: &str,
    test_lib: &str,
    dependency_plan: &SysrootDependencyPlan,
) -> Result<String, Vec<RenderedDiagnostic>> {
    let mut support_modules: Vec<(&str, &str)> = generated_project
        .support_rust_files
        .iter()
        .map(|(name, code)| (name.as_str(), code.as_str()))
        .collect();
    support_modules.sort_unstable_by(|left, right| left.0.cmp(right.0));
    let mut identity = sifr_identity::IdentityEncoder::new("test-runner-project-v1");
    identity.field(
        "scope",
        generated_project.cache_scope.as_os_str().as_encoded_bytes(),
    );
    identity.field("manifest", cargo_toml.as_bytes());
    identity.field("lib", test_lib.as_bytes());
    for (name, code) in support_modules {
        identity.field("support-name", name.as_bytes());
        identity.field("support-source", code.as_bytes());
    }
    for (name, code) in &generated_project.bridge_rust_files {
        let name = name.to_str().ok_or_else(|| {
            vec![crate::diagnostics::diagnostic_with_code(
                "failed to serialize test bridge cache inputs: non-UTF8 path".to_owned(),
                DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
            )]
        })?;
        identity.field("bridge-name", name.as_bytes());
        identity.field("bridge-source", code.as_bytes());
    }
    identity.field(
        "dependency-inputs",
        dependency_plan.dependency_input_fingerprint().as_bytes(),
    );
    identity.field(
        "dependency-plan",
        dependency_plan.cache_fingerprint.as_bytes(),
    );
    identity.field(
        "interop",
        generated_project.interop.cache_key_fragment().as_bytes(),
    );
    Ok(identity.finish())
}

#[cfg(test)]
mod tests {
    use super::test_runner_cache_key;
    use crate::test_runner::orchestrator::GeneratedTestRunnerProject;
    use sifr_stdlib_manifest::{
        CargoVendorMode, StdlibFeature, SysrootCrate, SysrootCrateDependency, SysrootDependencyPlan,
    };
    use std::collections::{BTreeSet, HashMap, HashSet};
    use std::path::PathBuf;

    #[test]
    fn test_runner_cargo_modes_use_build_resolution_and_trace_policy() {
        use sifr_package::CargoLockMode;
        use sifr_stdlib_manifest::CargoVendorMode;

        let scope = std::env::temp_dir().join(format!(
            "sifr-test-cargo-policy-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&scope).expect("owned scope");
        let mut project = GeneratedTestRunnerProject {
            application_profile: crate::ApplicationProfile::Test,
            cargo_resolution: crate::build::CargoResolutionPolicy::normal(),
            interop: sifr_codegen::InteropBuildPlan::default(),
            cache_scope: scope.clone(),
            support_module_names: Vec::new(),
            support_rust_files: HashMap::new(),
            bridge_rust_files: Default::default(),
            all_rust_code: "#[test] fn policy_case() { assert_eq!(2 + 2, 4); }".into(),
            all_stdlib_modules: HashSet::new(),
            all_required_features: HashSet::new(),
        };
        let (normal, invocations) = crate::build::capture_cargo_invocations(|| {
            super::execute_test_runner_project(&project)
        });
        let normal = normal.expect("normal test preparation");
        assert!(normal.success);
        assert!(
            invocations
                .iter()
                .any(|item| item.phase == "final-test" && item.lock_mode == CargoLockMode::Normal)
        );
        let authority = scope.join("authority.lock");
        std::fs::copy(normal.native_project_root.join("Cargo.lock"), &authority)
            .expect("normal Cargo lock authority");
        for mode in [
            CargoLockMode::Locked,
            CargoLockMode::Offline,
            CargoLockMode::Frozen,
        ] {
            project.cargo_resolution.lock_mode = mode;
            project.cargo_resolution.cargo_vendor_mode = CargoVendorMode::SysrootOnly;
            project.cargo_resolution.authoritative_locks = vec![authority.clone()];
            let (result, invocations) = crate::build::capture_cargo_invocations(|| {
                super::execute_test_runner_project(&project)
            });
            let outcome = result.expect("constrained test preparation");
            assert!(outcome.success);
            assert!(invocations.iter().any(|item| {
                item.phase == "final-test"
                    && item.lock_mode == mode
                    && item
                        .args
                        .iter()
                        .any(|arg| arg == mode.cargo_arg().expect("mode flag"))
            }));
            assert_eq!(
                std::fs::read(&authority).expect("authority"),
                std::fs::read(outcome.native_project_root.join("Cargo.lock"))
                    .expect("prepared lock")
            );
        }
        std::fs::remove_dir_all(scope).expect("owned scope cleanup");
    }

    #[test]
    fn test_runner_mutable_root_and_final_snapshot_stay_distinct() {
        let scope = std::env::temp_dir().join(format!(
            "sifr-test-native-storage-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&scope).expect("owned scope");
        let mut project = GeneratedTestRunnerProject {
            application_profile: crate::ApplicationProfile::Test,
            cargo_resolution: crate::build::CargoResolutionPolicy::normal(),
            interop: sifr_codegen::InteropBuildPlan::default(),
            cache_scope: scope.clone(),
            support_module_names: vec!["retired".into()],
            support_rust_files: HashMap::from([(
                "retired".into(),
                "pub fn value() -> u32 { 42 }".into(),
            )]),
            bridge_rust_files: Default::default(),
            all_rust_code: "#[test] fn storage_case() { assert_eq!(retired::value(), 42); }".into(),
            all_stdlib_modules: HashSet::new(),
            all_required_features: HashSet::new(),
        };
        let first = super::execute_test_runner_project(&project).expect("first test build");
        assert!(first.success);
        let first_entry = first.final_workspace_root.clone();
        let first_manifest = std::fs::read(first_entry.join("test_executables.json"))
            .expect("final executable manifest");
        let first_executables: Vec<PathBuf> =
            serde_json::from_slice(&first_manifest).expect("manifest paths");
        let first_hashes: Vec<_> = first_executables
            .iter()
            .map(|path| sifr_sysroot::sha256_file(&first_entry.join(path)).expect("executable"))
            .collect();
        assert!(first.native_project_root.join("Cargo.lock").is_file());
        assert!(first.native_project_root.join("src/retired.rs").is_file());
        assert!(!first_entry.join("Cargo.lock").exists());
        assert!(!first_entry.join("target").exists());
        assert!(!first_entry.join("src").exists());

        project.support_module_names.clear();
        project.support_rust_files.clear();
        project
            .all_required_features
            .insert(StdlibFeature::SerdeJson);
        project.all_rust_code = "#[test] fn storage_case() { assert_eq!(6 * 7, 42); }".into();
        let second = super::execute_test_runner_project(&project).expect("changed test build");
        assert!(second.success);
        assert_eq!(first.native_project_root, second.native_project_root);
        assert!(!second.native_project_root.join("src/retired.rs").exists());
        assert_ne!(first_entry, second.final_workspace_root);
        assert!(!second.final_workspace_root.join("Cargo.lock").exists());
        assert!(!second.final_workspace_root.join("target").exists());
        assert!(!second.final_workspace_root.join("src").exists());
        assert_eq!(
            std::fs::read(first_entry.join("test_executables.json")).expect("first manifest"),
            first_manifest
        );
        for (path, hash) in first_executables.iter().zip(first_hashes) {
            let executable = first_entry.join(path);
            assert_eq!(
                sifr_sysroot::sha256_file(&executable).expect("first executable"),
                hash
            );
            assert!(
                std::process::Command::new(executable)
                    .current_dir(&scope)
                    .status()
                    .expect("run finalized first test")
                    .success()
            );
        }
        std::fs::remove_dir_all(scope).expect("owned scope cleanup");
    }

    #[test]
    fn test_runner_same_key_concurrency_publishes_one_snapshot() {
        let scope = std::env::temp_dir().join(format!(
            "sifr-test-same-key-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&scope).expect("owned scope");
        let project = GeneratedTestRunnerProject {
            application_profile: crate::ApplicationProfile::Test,
            cargo_resolution: crate::build::CargoResolutionPolicy::normal(),
            interop: sifr_codegen::InteropBuildPlan::default(),
            cache_scope: scope.clone(),
            support_module_names: Vec::new(),
            support_rust_files: HashMap::new(),
            bridge_rust_files: Default::default(),
            all_rust_code: "#[test] fn concurrent_case() { assert_eq!(2 + 2, 4); }".into(),
            all_stdlib_modules: HashSet::new(),
            all_required_features: HashSet::new(),
        };
        let barrier = std::sync::Barrier::new(3);
        let outcomes = std::thread::scope(|threads| {
            let handles: Vec<_> = (0..3)
                .map(|_| {
                    let barrier = &barrier;
                    let project = &project;
                    threads.spawn(move || {
                        barrier.wait();
                        super::execute_test_runner_project(project).expect("concurrent test build")
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().expect("test thread"))
                .collect::<Vec<_>>()
        });
        assert!(outcomes.iter().all(|outcome| outcome.success));
        assert!(outcomes.iter().all(|outcome| {
            outcome.final_workspace_root == outcomes[0].final_workspace_root
                && outcome.native_project_root == outcomes[0].native_project_root
        }));
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| !outcome.cache_report.cache_hit())
                .count(),
            1
        );
        std::fs::remove_dir_all(scope).expect("owned scope cleanup");
    }

    #[test]
    fn test_runner_cache_key_uses_sysroot_dependency_plan_inputs() {
        let mut generated_project = GeneratedTestRunnerProject {
            application_profile: crate::ApplicationProfile::Test,
            cargo_resolution: crate::build::CargoResolutionPolicy::normal(),
            interop: sifr_codegen::InteropBuildPlan::default(),
            cache_scope: PathBuf::from("/tmp/sifr-tests"),
            support_module_names: Vec::new(),
            support_rust_files: HashMap::new(),
            bridge_rust_files: Default::default(),
            all_rust_code: "#[test]\nfn test_case() {}\n".to_string(),
            all_stdlib_modules: HashSet::from(["sifr.json".to_string()]),
            all_required_features: HashSet::from([StdlibFeature::SerdeJson]),
        };
        let dependency_plan = SysrootDependencyPlan {
            stdlib_modules: BTreeSet::from(["sifr.json".to_string()]),
            required_features: BTreeSet::from([StdlibFeature::SerdeJson]),
            sysroot_root: "/sysroot".into(),
            toolchain_id: "0.1.0-test-aarch64-test".to_string(),
            sysroot_content_sha256: "0".repeat(64),
            cargo_config: "/sysroot/.cargo/config.toml".into(),
            vendor_dir: "/sysroot/vendor".into(),
            crates: vec![SysrootCrateDependency {
                krate: SysrootCrate::SifrStdlib,
                path: "/sysroot/crates/sifr_stdlib".into(),
                features: BTreeSet::from(["json".to_string()]),
            }],
            retained_direct_dependencies: Vec::new(),
            cargo_vendor_mode: CargoVendorMode::SysrootOnly,
            cache_fingerprint: "fingerprint-a".to_string(),
        };

        let cache_key = test_runner_cache_key(
            &generated_project,
            "[package]\nname = \"sifr_tests\"\n",
            "#[test]\nfn test_case() {}\n",
            &dependency_plan,
        )
        .expect("valid cache inputs");

        assert_eq!(cache_key.len(), 64);
        let mut changed_plan = dependency_plan.clone();
        changed_plan.stdlib_modules.insert("sifr.math".into());
        assert_ne!(
            cache_key,
            test_runner_cache_key(
                &generated_project,
                "[package]\nname = \"sifr_tests\"\n",
                "#[test]\nfn test_case() {}\n",
                &changed_plan
            )
            .expect("valid cache inputs")
        );
        let identity = |project: &GeneratedTestRunnerProject| {
            test_runner_cache_key(
                project,
                "[package]\nname = \"sifr_tests\"\n",
                "#[test]\nfn test_case() {}\n",
                &dependency_plan,
            )
            .expect("valid cache inputs")
        };
        let path = PathBuf::from("sifr_generated_bridge/contract.rs");
        generated_project
            .bridge_rust_files
            .insert(path.clone(), "pub struct First;".into());
        let with_bridge = identity(&generated_project);
        assert_ne!(with_bridge, cache_key);
        generated_project
            .bridge_rust_files
            .insert(path.clone(), "pub struct Second;".into());
        let changed_content = identity(&generated_project);
        assert_ne!(changed_content, with_bridge);
        let content = generated_project
            .bridge_rust_files
            .remove(&path)
            .expect("bridge exists");
        generated_project
            .bridge_rust_files
            .insert(PathBuf::from("sifr_generated_bridge/renamed.rs"), content);
        assert_ne!(identity(&generated_project), changed_content);

        #[cfg(unix)]
        {
            use std::ffi::OsString;
            use std::os::unix::ffi::OsStringExt;

            generated_project.bridge_rust_files.clear();
            generated_project.bridge_rust_files.insert(
                PathBuf::from(OsString::from_vec(vec![0xff])),
                "pub struct InvalidPath;".to_string(),
            );
            let errors = test_runner_cache_key(
                &generated_project,
                "[package]\nname = \"sifr_tests\"\n",
                "#[test]\nfn test_case() {}\n",
                &dependency_plan,
            )
            .expect_err("non-UTF8 paths must fail before cache operations");
            assert_eq!(errors.len(), 1);
            assert_eq!(
                errors[0].code,
                sifr_diagnostics::DiagnosticCode::BUILD_MATERIALIZATION_FAILURE.code()
            );
            assert!(
                errors[0]
                    .message
                    .contains("failed to serialize test bridge cache inputs")
            );
        }
    }
}
