use super::cargo_manifest::{
    generate_dependency_cargo_toml_with_interop, try_generate_sysroot_dependency_plan,
};
use super::cargo_resolution::{CargoResolutionPolicy, cargo_resolution_cache_key_fragment};
use super::generated_cargo_project::{
    GeneratedCargoCommand, GeneratedCargoExecution, GeneratedCargoProject, cargo_execution_error,
    materialize_generated_cargo_project, run_generated_cargo_command,
};
use super::project_codegen::GeneratedBinaryProject;
use super::report::BuildSysrootReport;
use super::{CachedArtifactEntry, PreparedArtifactCache, prepare_cached_artifact};
use crate::diagnostics::RenderedDiagnostic;
use sifr_diagnostics::DiagnosticCode;
use sifr_stdlib_manifest::{CargoVendorMode, SysrootDependencyPlan};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub(super) struct MaterializedBinaryProject {
    pub(super) binary_path: PathBuf,
    pub(super) sysroot: BuildSysrootReport,
    pub(super) materialize_elapsed: Duration,
    pub(super) cargo_elapsed: Duration,
}

pub(super) fn materialize_binary_project_with_report(
    output_dir: &Path,
    project_name: &str,
    generated_project: GeneratedBinaryProject,
    requested_vendor_mode: CargoVendorMode,
    cargo_resolution: &CargoResolutionPolicy,
) -> Result<MaterializedBinaryProject, Vec<RenderedDiagnostic>> {
    let project_path = output_dir.join(project_name);
    let dependency_plan = try_generate_sysroot_dependency_plan(
        &generated_project.used_stdlib_modules,
        &generated_project.required_features,
        &generated_project.interop,
        requested_vendor_mode,
    )
    .map_err(|error| vec![build_error(error.boundary_message())])?;
    materialize_binary_project_at_path(
        &project_path,
        project_name,
        generated_project,
        &dependency_plan,
        cargo_resolution,
    )
    .map(|mut report| {
        report.binary_path = cached_binary_path(output_dir, project_name);
        report
    })
}

pub(super) fn materialize_cached_binary_project_with_report(
    cache_namespace: &str,
    cache_scope: &Path,
    project_name: &str,
    generated_project: GeneratedBinaryProject,
    requested_vendor_mode: CargoVendorMode,
    cargo_resolution: &CargoResolutionPolicy,
) -> Result<
    (
        CachedArtifactEntry,
        Option<MaterializedBinaryProject>,
        BuildSysrootReport,
    ),
    Vec<RenderedDiagnostic>,
> {
    let dependency_plan = try_generate_sysroot_dependency_plan(
        &generated_project.used_stdlib_modules,
        &generated_project.required_features,
        &generated_project.interop,
        requested_vendor_mode,
    )
    .map_err(|error| vec![build_error(error.boundary_message())])?;
    let sysroot = sysroot_report(&dependency_plan);
    let cache_key = binary_project_cache_key(
        project_name,
        &generated_project,
        &dependency_plan,
        cargo_resolution,
    )?;
    let required_paths = [
        Path::new(project_name).join("target"),
        binary_relative_path(project_name),
    ];
    let required_refs: Vec<&Path> = required_paths.iter().map(PathBuf::as_path).collect();
    let prepared =
        prepare_cached_artifact(cache_namespace, cache_scope, &cache_key, &required_refs)?;
    match prepared {
        PreparedArtifactCache::Hit(entry) => Ok((entry, None, sysroot)),
        PreparedArtifactCache::Miss(pending) => {
            let project_root = pending.workspace_root().join(project_name);
            let report = materialize_binary_project_at_path(
                &project_root,
                project_name,
                generated_project,
                &dependency_plan,
                cargo_resolution,
            )?;
            pending
                .commit(&required_refs)
                .map(|entry| (entry, Some(report), sysroot))
        }
    }
}

pub(super) fn cached_binary_path(workspace_root: &Path, project_name: &str) -> PathBuf {
    workspace_root.join(binary_relative_path(project_name))
}

fn binary_relative_path(project_name: &str) -> PathBuf {
    let binary_name = if cfg!(target_os = "windows") {
        format!("{project_name}.exe")
    } else {
        project_name.to_string()
    };
    PathBuf::from(project_name)
        .join("target")
        .join("release")
        .join(binary_name)
}

fn materialize_binary_project_at_path(
    project_path: &Path,
    project_name: &str,
    generated_project: GeneratedBinaryProject,
    dependency_plan: &SysrootDependencyPlan,
    cargo_resolution: &CargoResolutionPolicy,
) -> Result<MaterializedBinaryProject, Vec<RenderedDiagnostic>> {
    let python_interpreter = generated_project
        .python_runtime
        .as_ref()
        .map(|runtime| runtime.interpreter().to_path_buf());
    let sysroot = sysroot_report(dependency_plan);
    let additional_trusted_native_links = generated_project
        .python_runtime
        .as_ref()
        .map_or_else(BTreeSet::new, |runtime| {
            runtime.trusted_native_link_names().into_iter().collect()
        });
    let interop = generated_project.interop.clone();
    let materialize_start = std::time::Instant::now();
    materialize_generated_cargo_project(
        project_path,
        GeneratedCargoProject {
            name: project_name.to_string(),
            crate_root_file: PathBuf::from("main.rs"),
            crate_root_source: generated_project.main_rs,
            support_modules: generated_project.support_modules,
            support_main_alias: None,
            interop: generated_project.interop,
        },
        dependency_plan,
    )?;
    let materialize_elapsed = materialize_start.elapsed();

    let cargo_start = std::time::Instant::now();
    let output = run_generated_cargo_command(
        project_path,
        GeneratedCargoCommand::BuildRelease,
        GeneratedCargoExecution {
            python_interpreter: python_interpreter.as_deref(),
            target_directory: None,
            additional_trusted_native_links: &additional_trusted_native_links,
        },
        &interop,
        dependency_plan,
        cargo_resolution,
    )?;
    if !output.status.success() {
        return Err(vec![cargo_execution_error(format!(
            "cargo build failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ))]);
    }
    let cargo_elapsed = cargo_start.elapsed();

    Ok(MaterializedBinaryProject {
        binary_path: cached_binary_path(
            project_path.parent().unwrap_or(Path::new(".")),
            project_name,
        ),
        sysroot,
        materialize_elapsed,
        cargo_elapsed,
    })
}

fn sysroot_report(dependency_plan: &SysrootDependencyPlan) -> BuildSysrootReport {
    BuildSysrootReport::from_dependency_plan(dependency_plan)
}

fn build_error(message: String) -> RenderedDiagnostic {
    crate::diagnostics::diagnostic_with_code(message, DiagnosticCode::BUILD_MATERIALIZATION_FAILURE)
}

fn binary_project_cache_key(
    project_name: &str,
    generated_project: &GeneratedBinaryProject,
    dependency_plan: &SysrootDependencyPlan,
    cargo_resolution: &CargoResolutionPolicy,
) -> Result<String, Vec<RenderedDiagnostic>> {
    let support_modules = generated_project
        .support_modules
        .iter()
        .map(|(name, code)| format!("{name}\n{code}"))
        .collect::<Vec<_>>()
        .join("\n===\n");
    let cargo_resolution_fragment = cargo_resolution_cache_key_fragment(cargo_resolution)?;
    Ok(format!(
        "project_name={project_name}\n[Cargo.toml]\n{}\n[main.rs]\n{}\n[support]\n{}\n[sysroot-dependency-inputs]\n{}[interop]\n{}\n[cache-key-fragment]\n{}\n[sysroot-dependency-plan]\n{}\n[cargo-resolution]\n{}",
        generate_dependency_cargo_toml_with_interop(
            project_name,
            dependency_plan,
            &generated_project.interop
        ),
        generated_project.main_rs,
        support_modules,
        dependency_plan.dependency_input_fingerprint(),
        generated_project.interop.cache_key_fragment(),
        generated_project
            .cache_key_fragment
            .as_deref()
            .unwrap_or(""),
        dependency_plan.cache_fingerprint,
        cargo_resolution_fragment
    ))
}

#[cfg(test)]
mod tests {
    use super::{CargoResolutionPolicy, binary_project_cache_key};
    use crate::build::generated_cargo_project::{
        should_validate_native_link_evidence, sysroot_trusted_native_links, trusted_native_links,
        validate_native_link_evidence,
    };
    use crate::build::project_codegen::GeneratedBinaryProject;
    use sifr_codegen::{
        InteropBuildPlan, RustInteropOwner, RustInteropPlan, RustInteropPlanDeclaration,
        RustInteropTrustRequirement, RustInteropTrustRequirementKind,
    };
    use sifr_compiler_services::PackagePythonRuntime;
    use sifr_ir::{
        RustInteropAbiRequirements, RustInteropDeclaration, RustInteropDecoratorKind,
        RustInteropEffect, RustTargetPath,
    };
    use sifr_stdlib_manifest::{CargoVendorMode, StdlibFeature, SysrootDependencyPlan};
    use std::collections::{BTreeMap, BTreeSet, HashSet};

    #[test]
    fn binary_project_cache_key_includes_package_cache_fragment() {
        let base = base_project();
        let mut with_python_probe = GeneratedBinaryProject {
            cache_key_fragment: Some("python-probe-a".to_string()),
            ..base
        };
        let dependency_plan = test_dependency_plan("fingerprint-a");
        let first = test_binary_project_cache_key(&with_python_probe, &dependency_plan);
        with_python_probe.cache_key_fragment = Some("python-probe-b".to_string());
        let second = test_binary_project_cache_key(&with_python_probe, &dependency_plan);

        assert_ne!(first, second);
    }

    #[test]
    fn binary_project_cache_key_includes_interop_build_plan() {
        let base = base_project();
        let mut with_interop = base_project();
        with_interop.interop = InteropBuildPlan {
            rust: RustInteropPlan {
                declarations: vec![RustInteropPlanDeclaration {
                    module_name: Some("main".to_string()),
                    owner: RustInteropOwner::Function {
                        name: "digest".to_string(),
                    },
                    declaration: RustInteropDeclaration {
                        kind: RustInteropDecoratorKind::Function,
                        target: Some(RustTargetPath {
                            segments: vec![
                                "bridge".to_string(),
                                "hash".to_string(),
                                "digest".to_string(),
                            ],
                            span: Default::default(),
                        }),
                        arguments: Vec::new(),
                        span: Default::default(),
                        effect: RustInteropEffect::Sync,
                        abi_requirements: RustInteropAbiRequirements::default(),
                        consumes_receiver: false,
                    },
                }],
                ..RustInteropPlan::default()
            },
            ..InteropBuildPlan::default()
        };

        assert_ne!(
            test_binary_project_cache_key(&base, &test_dependency_plan("fingerprint-a")),
            test_binary_project_cache_key(&with_interop, &test_dependency_plan("fingerprint-a"))
        );
    }

    #[test]
    fn binary_project_cache_key_includes_sysroot_dependency_plan() {
        let base = base_project();

        assert_ne!(
            test_binary_project_cache_key(&base, &test_dependency_plan("fingerprint-a")),
            test_binary_project_cache_key(&base, &test_dependency_plan("fingerprint-b"))
        );
    }

    #[test]
    fn binary_project_cache_key_uses_sysroot_dependency_plan_inputs() {
        let base = base_project();
        let mut dependency_plan = test_dependency_plan("fingerprint-a");
        dependency_plan.stdlib_modules = BTreeSet::from(["sifr.json".to_string()]);
        dependency_plan.required_features = BTreeSet::from([StdlibFeature::SerdeJson]);

        let cache_key = test_binary_project_cache_key(&base, &dependency_plan);

        assert!(cache_key.contains(
            "[sysroot-dependency-inputs]\n[stdlib]\nsifr.json\n[features]\nserde_json\n"
        ));
    }

    fn test_binary_project_cache_key(
        project: &GeneratedBinaryProject,
        dependency_plan: &SysrootDependencyPlan,
    ) -> String {
        binary_project_cache_key(
            "sifr_output",
            project,
            dependency_plan,
            &CargoResolutionPolicy::normal(),
        )
        .expect("normal Cargo resolution policy should have a cache identity")
    }

    #[test]
    fn native_link_evidence_rejects_untrusted_build_script_output() {
        let stdout = br#"{"reason":"build-script-executed","linked_libs":["dylib=ssl"]}"#;
        let diagnostics = validate_native_link_evidence(stdout, &BTreeSet::new())
            .expect_err("untrusted link evidence should fail");

        assert_eq!(diagnostics[0].code, "SIFR-RUST-TRUST-0001");

        let trusted = BTreeSet::from(["ssl".to_string()]);
        validate_native_link_evidence(stdout, &trusted).expect("trusted link should pass");
    }

    #[test]
    fn native_link_evidence_policy_skips_non_rust_interop_projects() {
        let mut project = base_project();
        assert!(!should_validate_native_link_evidence(&project.interop));

        project
            .interop
            .rust
            .trust_requirements
            .push(RustInteropTrustRequirement {
                canonical_target_path: "openssl::ssl".to_string(),
                kind: RustInteropTrustRequirementKind::NativeLinks,
                trusted: true,
                required_entry: "ssl".to_string(),
                evidence: "links=ssl".to_string(),
            });
        assert!(should_validate_native_link_evidence(&project.interop));
    }

    #[test]
    fn python_runtime_libpython_link_is_trusted_when_interop_validation_runs() {
        let mut project = base_project();
        let mut python_runtime =
            PackagePythonRuntime::for_tests("/tmp/sifr-py/bin/python", "digest-a");
        python_runtime.set_libpython_for_tests("/opt/python/lib/libpython3.14.dylib");
        project.python_runtime = Some(python_runtime);
        project
            .interop
            .rust
            .trust_requirements
            .push(RustInteropTrustRequirement {
                canonical_target_path: "::sifr_stdlib::html::html_escape".to_string(),
                kind: RustInteropTrustRequirementKind::NativeLinks,
                trusted: true,
                required_entry: "ssl".to_string(),
                evidence: "links=ssl".to_string(),
            });

        let stdout = br#"{"reason":"build-script-executed","linked_libs":["dylib=python3.14"]}"#;
        let mut trusted =
            trusted_native_links(&project.interop, &test_dependency_plan("fingerprint-a"));
        trusted.extend(
            project
                .python_runtime
                .as_ref()
                .expect("Python runtime should be configured")
                .trusted_native_link_names(),
        );
        validate_native_link_evidence(stdout, &trusted)
            .expect("selected Python runtime link should be trusted");
    }

    #[test]
    fn sysroot_tls_native_link_evidence_is_explicitly_trusted() {
        let mut dependency_plan = test_dependency_plan("fingerprint-a");
        dependency_plan
            .crates
            .push(sifr_stdlib_manifest::SysrootCrateDependency {
                krate: sifr_stdlib_manifest::SysrootCrate::SifrStdlib,
                path: "/sysroot/crates/sifr_stdlib".into(),
                features: BTreeSet::from(["tls".to_string()]),
            });

        let trusted = sysroot_trusted_native_links(&dependency_plan);
        assert_eq!(
            trusted,
            BTreeSet::from(["aws_lc_0_44_0_crypto".to_string()])
        );

        let stdout =
            br#"{"reason":"build-script-executed","linked_libs":["static=aws_lc_0_44_0_crypto"]}"#;
        validate_native_link_evidence(stdout, &trusted)
            .expect("sysroot-selected TLS provider link should pass");

        let untrusted = br#"{"reason":"build-script-executed","linked_libs":["static=crypto"]}"#;
        validate_native_link_evidence(untrusted, &trusted)
            .expect_err("unrelated native links must still fail");
    }

    #[test]
    fn sysroot_http_native_link_evidence_inherits_tls_provider_trust() {
        let mut dependency_plan = test_dependency_plan("fingerprint-a");
        dependency_plan
            .crates
            .push(sifr_stdlib_manifest::SysrootCrateDependency {
                krate: sifr_stdlib_manifest::SysrootCrate::SifrStdlib,
                path: "/sysroot/crates/sifr_stdlib".into(),
                features: BTreeSet::from(["http".to_string()]),
            });

        let trusted = sysroot_trusted_native_links(&dependency_plan);
        assert_eq!(
            trusted,
            BTreeSet::from(["aws_lc_0_44_0_crypto".to_string()])
        );
    }

    fn base_project() -> GeneratedBinaryProject {
        GeneratedBinaryProject {
            main_rs: "fn main() {}\n".to_string(),
            support_modules: BTreeMap::new(),
            used_stdlib_modules: HashSet::new(),
            required_features: HashSet::new(),
            interop: InteropBuildPlan::default(),
            cache_key_fragment: None,
            python_runtime: None,
        }
    }

    fn test_dependency_plan(cache_fingerprint: &str) -> SysrootDependencyPlan {
        SysrootDependencyPlan {
            stdlib_modules: BTreeSet::new(),
            required_features: BTreeSet::new(),
            sysroot_root: "/sysroot".into(),
            toolchain_id: "0.1.0-test-aarch64-test".to_string(),
            sysroot_content_sha256: "0".repeat(64),
            cargo_config: "/sysroot/.cargo/config.toml".into(),
            vendor_dir: "/sysroot/vendor".into(),
            crates: Vec::new(),
            retained_direct_dependencies: Vec::new(),
            cargo_vendor_mode: CargoVendorMode::SysrootOnly,
            cache_fingerprint: cache_fingerprint.to_string(),
        }
    }
}
