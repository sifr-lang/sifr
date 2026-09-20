use super::artifacts::{
    compose_test_runner_lib, test_support_module_file_path, try_generate_test_runner_cargo_plan,
};
use super::orchestrator::GeneratedTestRunnerProject;
use crate::build::{
    ArtifactCacheReport, PreparedArtifactCache, prepare_cached_artifact, sysroot_cargo_config_args,
};
use crate::diagnostics::{RenderedDiagnostic, write_stderr, write_stderr_line};
use crate::project::namespace_module_files;
use sifr_diagnostics::DiagnosticCode;
use sifr_stdlib_manifest::SysrootDependencyPlan;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

pub(crate) struct TestRunnerExecutionOutcome {
    pub(crate) success: bool,
    #[cfg(test)]
    pub(crate) native_project_root: PathBuf,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) cache_report: ArtifactCacheReport,
}

pub(crate) fn execute_test_runner_project(
    generated_project: &GeneratedTestRunnerProject,
) -> Result<TestRunnerExecutionOutcome, Vec<RenderedDiagnostic>> {
    let native_toolchain = std::env::current_dir()
        .map_err(|_| "cannot resolve invocation directory".to_owned())
        .and_then(|cwd| sifr_sysroot::NativeToolchain::resolve_at(&cwd))
        .map_err(|error| {
            vec![crate::diagnostics::diagnostic_with_code(
                error,
                DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE,
            )]
        })?;
    let mut cargo_plan = try_generate_test_runner_cargo_plan(
        &generated_project.all_stdlib_modules,
        &generated_project.all_required_features,
        &generated_project.interop,
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
    let family = crate::build::native_storage::NativeFamily::acquire(
        native_toolchain.identity(),
        &format!(
            "{:?}:{}",
            cargo_plan.dependency_plan.cargo_vendor_mode,
            cargo_plan.dependency_plan.sysroot_root.display()
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

        crate::build::native_storage::write_changed(&src_dir.join("lib.rs"), test_lib.as_bytes())
            .map_err(|error| {
            vec![crate::diagnostics::diagnostic_with_code(
                format!("failed to write lib.rs: {error}"),
                DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
            )]
        })?;
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
    write_stderr_line(&format!(
        "application profile: {} ({})",
        generated_project.application_profile.name(),
        generated_project.application_profile.policy_identity()
    ));
    let mut command = native_toolchain.cargo_command().map_err(test_io_error)?;
    command
        .args(sysroot_cargo_config_args(&cargo_plan.dependency_plan))
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
    let output = crate::process_execution::output(&mut command).map_err(test_io_error)?;
    write_stderr(&String::from_utf8_lossy(&output.stderr));
    if !output.status.success() {
        return Err(test_io_error(format!(
            "cargo test preparation failed: {}",
            String::from_utf8_lossy(&output.stdout)
        )));
    }
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
    fn test_runner_cache_key_uses_sysroot_dependency_plan_inputs() {
        let mut generated_project = GeneratedTestRunnerProject {
            application_profile: crate::ApplicationProfile::Test,
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
