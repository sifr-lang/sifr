use super::cargo_invocation_trace::record_cargo_invocation;
use super::cargo_manifest::{
    generate_dependency_cargo_toml_with_interop, sysroot_cargo_config_args,
    try_generate_sysroot_dependency_plan,
};
use super::cargo_resolution::{
    CargoResolutionPolicy, cargo_lock_mode_diagnostic, prepare_cargo_resolution,
};
use super::project_codegen::GeneratedBinaryProject;
use super::report::BuildSysrootReport;
use super::rust_interop_sqlx_offline::configure_hermetic_build_environment;
use super::{CachedArtifactEntry, PreparedArtifactCache, prepare_cached_artifact};
use crate::diagnostics::RenderedDiagnostic;
use crate::project::{namespace_module_files, rust_module_file_path};
use sifr_codegen::RustInteropTrustRequirementKind;
use sifr_diagnostics::DiagnosticCode;
use sifr_stdlib_manifest::{CargoVendorMode, SysrootCrate, SysrootDependencyPlan};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
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

pub(super) fn materialize_binary_project_sources(
    output_dir: &Path,
    project_name: &str,
    generated_project: GeneratedBinaryProject,
    requested_vendor_mode: CargoVendorMode,
    cargo_resolution: &CargoResolutionPolicy,
) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
    let project_path = output_dir.join(project_name);
    let dependency_plan = try_generate_sysroot_dependency_plan(
        &generated_project.used_stdlib_modules,
        &generated_project.required_features,
        &generated_project.interop,
        requested_vendor_mode,
    )
    .map_err(|error| vec![build_error(error.boundary_message())])?;
    let interop = generated_project.interop.clone();
    let local_project_path =
        super::portable_project::local_resolution_project_path(output_dir, project_name);
    let result = (|| {
        materialize_binary_project_files(
            &local_project_path,
            project_name,
            generated_project,
            &dependency_plan,
        )?;
        let cargo_prefix_args = sysroot_cargo_config_args(&dependency_plan);
        let prepared_resolution =
            prepare_cargo_resolution(&local_project_path, cargo_resolution, &cargo_prefix_args)?;
        prepared_resolution.assert_unchanged()?;
        super::portable_project::prepare_portable_project_metadata(
            &local_project_path,
            project_name,
            &dependency_plan,
            &interop,
            cargo_resolution,
        )?;
        super::portable_project::publish_portable_project(&local_project_path, &project_path)?;
        Ok(project_path)
    })();
    let cleanup = std::fs::remove_dir_all(&local_project_path);
    match (result, cleanup) {
        (Ok(path), Ok(())) => Ok(path),
        (Ok(_), Err(error)) => Err(vec![build_error(format!(
            "failed to remove ephemeral local Cargo resolution state: {error}"
        ))]),
        (Err(errors), _) => Err(errors),
    }
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
    let cache_key = binary_project_cache_key(project_name, &generated_project, &dependency_plan);
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
    let validate_native_links = should_validate_native_link_evidence(&generated_project);
    let trusted_native_links = trusted_native_links(&generated_project, dependency_plan);
    let materialize_start = std::time::Instant::now();
    materialize_binary_project_files(
        project_path,
        project_name,
        generated_project,
        dependency_plan,
    )?;
    let materialize_elapsed = materialize_start.elapsed();

    let cargo_start = std::time::Instant::now();
    let cargo_prefix_args = sysroot_cargo_config_args(dependency_plan);
    let prepared_resolution =
        prepare_cargo_resolution(project_path, cargo_resolution, &cargo_prefix_args)?;
    run_cargo_build(
        project_path,
        python_interpreter.as_deref(),
        validate_native_links,
        &trusted_native_links,
        dependency_plan,
        cargo_resolution,
    )?;
    prepared_resolution.assert_unchanged()?;
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

fn materialize_binary_project_files(
    project_path: &Path,
    project_name: &str,
    generated_project: GeneratedBinaryProject,
    dependency_plan: &SysrootDependencyPlan,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let src_dir = project_path.join("src");
    if src_dir.exists() {
        std::fs::remove_dir_all(&src_dir).map_err(|error| {
            vec![build_error(format!(
                "failed to reset generated source directory: {error}"
            ))]
        })?;
    }
    std::fs::create_dir_all(&src_dir).map_err(|error| {
        vec![build_error(format!(
            "failed to create output directory: {error}"
        ))]
    })?;

    let cargo_toml = generate_dependency_cargo_toml_with_interop(
        project_name,
        dependency_plan,
        &generated_project.interop,
    );

    write_project_file(&project_path.join("Cargo.toml"), cargo_toml, "Cargo.toml")?;

    // Module declarations contain no fields. Keep the source-listing boundary,
    // but declare the already-canonicalized bridge family in the native crate.
    let main_rs = if generated_project.bridge_modules.is_empty() {
        generated_project.main_rs
    } else {
        format!(
            "pub mod {};\n{}",
            sifr_codegen::canonicalize_generated_rust_identifier("__sifr_bridge"),
            generated_project.main_rs
        )
    };
    write_project_file(&src_dir.join("main.rs"), main_rs, "main.rs")?;

    for (module, source) in generated_project.bridge_modules {
        let path = if module == "__sifr_bridge" {
            PathBuf::from("__sifr_bridge/mod.rs")
        } else {
            rust_module_file_path(&module.replace("::", "."))
        };
        let canonical_path = canonical_rust_module_path(&path)?;
        write_project_file(
            &src_dir.join(&canonical_path),
            source,
            &canonical_path.display().to_string(),
        )?;
    }

    let mut support_modules = generated_project.support_modules;
    let support_module_names: Vec<String> = support_modules.keys().cloned().collect();
    let mut namespace_contents: BTreeMap<PathBuf, String> = BTreeMap::new();
    for namespace_file in namespace_module_files(&support_module_names) {
        let mut contents = String::new();
        for module_name in &namespace_file.declarations {
            contents.push_str("pub mod ");
            contents.push_str(&sifr_codegen::canonicalize_generated_rust_identifier(
                module_name,
            ));
            contents.push_str(";\n");
        }
        namespace_contents.insert(namespace_file.path, contents);
    }

    for (module_name, code) in std::mem::take(&mut support_modules) {
        let namespace_path = namespace_module_file_path(&module_name);
        if let Some(contents) = namespace_contents.get_mut(&namespace_path) {
            if !contents.is_empty() && !contents.ends_with('\n') {
                contents.push('\n');
            }
            contents.push_str(&code);
            continue;
        }
        let file_name = canonical_rust_module_path(&rust_module_file_path(&module_name))?;
        write_project_file(
            &src_dir.join(&file_name),
            code,
            &file_name.display().to_string(),
        )?;
    }

    for (namespace_path, contents) in namespace_contents {
        let namespace_path = canonical_rust_module_path(&namespace_path)?;
        write_project_file(
            &src_dir.join(&namespace_path),
            contents,
            &namespace_path.display().to_string(),
        )?;
    }

    Ok(())
}

fn canonical_rust_module_path(path: &Path) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
    let mut canonical = PathBuf::new();
    for component in path.components() {
        let std::path::Component::Normal(component) = component else {
            return Err(vec![build_error(format!(
                "generated Rust module path must be relative and cannot escape its project: {}",
                path.display()
            ))]);
        };
        let component = Path::new(component);
        if component
            .extension()
            .is_some_and(|extension| extension == "rs")
        {
            let Some(stem) = component.file_stem().and_then(std::ffi::OsStr::to_str) else {
                return Err(vec![build_error(format!(
                    "generated Rust module filename is not valid UTF-8: {}",
                    component.display()
                ))]);
            };
            canonical.push(format!(
                "{}.rs",
                sifr_codegen::canonicalize_generated_rust_identifier(stem)
            ));
        } else {
            let Some(name) = component.as_os_str().to_str() else {
                return Err(vec![build_error(format!(
                    "generated Rust module path is not valid UTF-8: {}",
                    component.display()
                ))]);
            };
            canonical.push(sifr_codegen::canonicalize_generated_rust_identifier(name));
        }
    }
    Ok(canonical)
}

fn run_cargo_build(
    project_path: &Path,
    python_interpreter: Option<&Path>,
    validate_native_links: bool,
    trusted_native_links: &BTreeSet<String>,
    dependency_plan: &SysrootDependencyPlan,
    cargo_resolution: &CargoResolutionPolicy,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let mut command = Command::new("cargo");
    command.args(sysroot_cargo_config_args(dependency_plan));
    command
        .args([
            "build",
            "--release",
            "--quiet",
            "--message-format=json-render-diagnostics",
        ])
        .current_dir(project_path);
    if let Some(argument) = cargo_resolution.lock_mode.cargo_arg() {
        command.arg(argument);
    }
    // Generated projects are materialized and cached with their own `target/`
    // directory. Inheriting an outer CARGO_TARGET_DIR moves binaries away from
    // the reported artifact paths and breaks cache completeness checks.
    command.env_remove("CARGO_TARGET_DIR");
    configure_hermetic_build_environment(&mut command);
    if let Some(python_interpreter) = python_interpreter {
        command.env("PYO3_PYTHON", python_interpreter);
    }
    record_cargo_invocation("final-build", cargo_resolution.lock_mode, &command);
    let output = command.output().map_err(|error| {
        vec![cargo_build_error(format!(
            "failed to run cargo build: {error}"
        ))]
    })?;

    if validate_native_links {
        validate_native_link_evidence(&output.stdout, trusted_native_links)?;
    }

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if let Some(diagnostic) = cargo_lock_mode_diagnostic("cargo build", &stderr) {
            return Err(vec![diagnostic]);
        }
        return Err(vec![cargo_build_error(format!(
            "cargo build failed:\n{stderr}"
        ))]);
    }
    Ok(())
}

fn trusted_native_links(
    generated_project: &GeneratedBinaryProject,
    dependency_plan: &SysrootDependencyPlan,
) -> BTreeSet<String> {
    let mut trusted = generated_project
        .interop
        .rust
        .trust_requirements
        .iter()
        .filter(|requirement| {
            requirement.trusted && requirement.kind == RustInteropTrustRequirementKind::NativeLinks
        })
        .map(|requirement| requirement.required_entry.clone())
        .collect::<BTreeSet<_>>();
    if let Some(python_runtime) = &generated_project.python_runtime {
        trusted.extend(python_runtime.trusted_native_link_names());
    }
    trusted.extend(sysroot_trusted_native_links(dependency_plan));
    trusted
}

fn sysroot_trusted_native_links(dependency_plan: &SysrootDependencyPlan) -> BTreeSet<String> {
    let tls_selected = dependency_plan.crates.iter().any(|dependency| {
        matches!(
            dependency.krate,
            SysrootCrate::SifrRuntime | SysrootCrate::SifrStdlib
        ) && (dependency.features.contains("tls") || dependency.features.contains("http"))
    });
    if tls_selected {
        return BTreeSet::from(["aws_lc_0_44_0_crypto".to_string()]);
    }
    BTreeSet::new()
}

fn should_validate_native_link_evidence(generated_project: &GeneratedBinaryProject) -> bool {
    let rust = &generated_project.interop.rust;
    !rust.declarations.is_empty()
        || !rust.resolved_targets.is_empty()
        || !rust.trust_requirements.is_empty()
        || !rust.probe_plan.probes.is_empty()
        || !rust.bridge_sources.is_empty()
        || rust.cargo_inputs.is_some()
}

fn validate_native_link_evidence(
    stdout: &[u8],
    trusted_native_links: &BTreeSet<String>,
) -> Result<(), Vec<RenderedDiagnostic>> {
    for line in String::from_utf8_lossy(stdout).lines() {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if value.get("reason").and_then(serde_json::Value::as_str) != Some("build-script-executed")
        {
            continue;
        }
        let Some(linked_libs) = value
            .get("linked_libs")
            .and_then(serde_json::Value::as_array)
        else {
            continue;
        };
        for linked_lib in linked_libs {
            let Some(linked_lib) = linked_lib.as_str() else {
                continue;
            };
            let link_name = normalized_link_name(linked_lib);
            if !trusted_native_links.contains(&link_name) {
                return Err(vec![crate::diagnostics::diagnostic_with_code(
                    format!(
                        "untrusted native link evidence `{link_name}` emitted by Rust build script"
                    ),
                    DiagnosticCode::RUST_TRUST_MISSING,
                )]);
            }
        }
    }
    Ok(())
}

fn normalized_link_name(linked_lib: &str) -> String {
    linked_lib
        .rsplit_once('=')
        .map_or(linked_lib, |(_, name)| name)
        .to_string()
}

fn namespace_module_file_path(module_name: &str) -> PathBuf {
    let mut path = PathBuf::new();
    for component in module_name.split('.') {
        path.push(component);
    }
    path.push("mod.rs");
    path
}

fn write_project_file(
    path: &Path,
    contents: impl AsRef<[u8]>,
    label: &str,
) -> Result<(), Vec<RenderedDiagnostic>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| vec![build_error(format!("failed to create {label}: {error}"))])?;
    }
    let contents = contents.as_ref();
    let formatted;
    let contents = if path.extension().is_some_and(|extension| extension == "rs") {
        let source = std::str::from_utf8(contents).map_err(|error| {
            vec![build_error(format!(
                "generated {label} is not valid UTF-8 before Rust formatting: {error}"
            ))]
        })?;
        formatted = super::rust_formatter::format_canonical_generated_rust(source, label)?;
        formatted.as_bytes()
    } else {
        contents
    };
    std::fs::write(path, contents)
        .map_err(|error| vec![build_error(format!("failed to write {label}: {error}"))])
}

fn build_error(message: String) -> RenderedDiagnostic {
    crate::diagnostics::diagnostic_with_code(message, DiagnosticCode::BUILD_MATERIALIZATION_FAILURE)
}

fn cargo_build_error(message: String) -> RenderedDiagnostic {
    crate::diagnostics::diagnostic_with_code(message, DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE)
}

fn binary_project_cache_key(
    project_name: &str,
    generated_project: &GeneratedBinaryProject,
    dependency_plan: &SysrootDependencyPlan,
) -> String {
    let support_modules = generated_project
        .support_modules
        .iter()
        .chain(generated_project.bridge_modules.iter())
        .map(|(name, code)| format!("{name}\n{code}"))
        .collect::<Vec<_>>()
        .join("\n===\n");
    format!(
        "project_name={project_name}\n[Cargo.toml]\n{}\n[main.rs]\n{}\n[support]\n{}\n[sysroot-dependency-inputs]\n{}[interop]\n{}\n[cache-key-fragment]\n{}\n[sysroot-dependency-plan]\n{}",
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
        dependency_plan.cache_fingerprint
    )
}

#[cfg(test)]
mod tests {
    use super::{
        binary_project_cache_key, canonical_rust_module_path, materialize_binary_project_files,
        should_validate_native_link_evidence, sysroot_trusted_native_links, trusted_native_links,
        validate_native_link_evidence,
    };
    use crate::build::project_codegen::GeneratedBinaryProject;
    use crate::build::python_runtime::PackagePythonRuntime;
    use sifr_codegen::{
        InteropBuildPlan, RustInteropOwner, RustInteropPlan, RustInteropPlanDeclaration,
        RustInteropTrustRequirement, RustInteropTrustRequirementKind,
    };
    use sifr_ir::{
        RustInteropAbiRequirements, RustInteropDeclaration, RustInteropDecoratorKind,
        RustInteropEffect, RustTargetPath,
    };
    use sifr_stdlib_manifest::{CargoVendorMode, StdlibFeature, SysrootDependencyPlan};
    use std::collections::{BTreeMap, BTreeSet, HashSet};

    #[test]
    fn generated_module_paths_are_relative_and_cannot_escape() {
        let bridge = canonical_rust_module_path(std::path::Path::new("__sifr_bridge/_sifr_fs.rs"));
        assert!(matches!(
            bridge.as_deref(),
            Ok(path) if path == std::path::Path::new("sifr_generated_bridge/sifr_generated_fs.rs")
        ));
        let public = canonical_rust_module_path(std::path::Path::new("public/mod.rs"));
        assert!(matches!(
            public.as_deref(),
            Ok(path) if path == std::path::Path::new("public/mod.rs")
        ));
        assert!(canonical_rust_module_path(std::path::Path::new("../escape.rs")).is_err());
        assert!(canonical_rust_module_path(std::path::Path::new("/escape.rs")).is_err());
    }

    #[test]
    fn source_materialization_writes_a_complete_uncompiled_cargo_project() {
        let root = std::env::temp_dir().join(format!(
            "sifr_source_materialization_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        ));
        let project_path = root.join("sifr_output");

        materialize_binary_project_files(
            &project_path,
            "sifr_output",
            base_project(),
            &test_dependency_plan("fingerprint-a"),
        )
        .expect("source-only materialization should succeed");

        assert!(project_path.join("Cargo.toml").is_file());
        let main_rs = std::fs::read_to_string(project_path.join("src/main.rs"))
            .expect("generated main should be readable");
        assert!(main_rs.contains("fn main()"), "{main_rs}");
        assert!(!project_path.join("target").exists());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn rematerialization_removes_stale_generated_sources_but_preserves_target() {
        let root = std::env::temp_dir().join(format!(
            "sifr_source_rematerialization_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        ));
        let project_path = root.join("sifr_output");
        let stale_source = project_path.join("src/obsolete/generated.rs");
        std::fs::create_dir_all(stale_source.parent().expect("stale source has a parent"))
            .expect("stale source directory should be writable");
        std::fs::write(&stale_source, "fn obsolete() {}\n")
            .expect("stale source should be writable");
        let target_marker = project_path.join("target/cache-marker");
        std::fs::create_dir_all(target_marker.parent().expect("target marker has a parent"))
            .expect("target directory should be writable");
        std::fs::write(&target_marker, "preserve").expect("target marker should be writable");

        materialize_binary_project_files(
            &project_path,
            "sifr_output",
            base_project(),
            &test_dependency_plan("fingerprint-a"),
        )
        .expect("rematerialization should succeed");

        assert!(!stale_source.exists());
        assert!(target_marker.is_file());
        assert!(project_path.join("src/main.rs").is_file());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn binary_project_cache_key_includes_package_cache_fragment() {
        let base = base_project();
        let mut with_python_probe = GeneratedBinaryProject {
            cache_key_fragment: Some("python-probe-a".to_string()),
            ..base
        };
        let dependency_plan = test_dependency_plan("fingerprint-a");
        let first = binary_project_cache_key("sifr_output", &with_python_probe, &dependency_plan);
        with_python_probe.cache_key_fragment = Some("python-probe-b".to_string());
        let second = binary_project_cache_key("sifr_output", &with_python_probe, &dependency_plan);

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
            binary_project_cache_key("sifr_output", &base, &test_dependency_plan("fingerprint-a")),
            binary_project_cache_key(
                "sifr_output",
                &with_interop,
                &test_dependency_plan("fingerprint-a")
            )
        );
    }

    #[test]
    fn binary_project_cache_key_includes_sysroot_dependency_plan() {
        let base = base_project();

        assert_ne!(
            binary_project_cache_key("sifr_output", &base, &test_dependency_plan("fingerprint-a")),
            binary_project_cache_key("sifr_output", &base, &test_dependency_plan("fingerprint-b"))
        );
    }

    #[test]
    fn binary_project_cache_key_uses_sysroot_dependency_plan_inputs() {
        let base = base_project();
        let mut dependency_plan = test_dependency_plan("fingerprint-a");
        dependency_plan.stdlib_modules = BTreeSet::from(["sifr.json".to_string()]);
        dependency_plan.required_features = BTreeSet::from([StdlibFeature::SerdeJson]);

        let cache_key = binary_project_cache_key("sifr_output", &base, &dependency_plan);

        assert!(cache_key.contains(
            "[sysroot-dependency-inputs]\n[stdlib]\nsifr.json\n[features]\nserde_json\n"
        ));
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
        assert!(!should_validate_native_link_evidence(&project));

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
        assert!(should_validate_native_link_evidence(&project));
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
        validate_native_link_evidence(
            stdout,
            &trusted_native_links(&project, &test_dependency_plan("fingerprint-a")),
        )
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

    pub(super) fn base_project() -> GeneratedBinaryProject {
        GeneratedBinaryProject {
            main_rs: "fn main() {}\n".to_string(),
            support_modules: BTreeMap::new(),
            used_stdlib_modules: HashSet::new(),
            required_features: HashSet::new(),
            interop: InteropBuildPlan::default(),
            cache_key_fragment: None,
            bridge_modules: BTreeMap::new(),
            python_runtime: None,
        }
    }

    pub(super) fn test_dependency_plan(cache_fingerprint: &str) -> SysrootDependencyPlan {
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

#[cfg(test)]
#[path = "materialize_field_identity_tests.rs"]
mod field_identity_tests;
