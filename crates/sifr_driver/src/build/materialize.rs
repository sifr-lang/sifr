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
    let mut cache_key =
        binary_project_cache_key(project_name, &generated_project, &dependency_plan);
    let tools = cargo_resolution
        .native_toolchain
        .as_ref()
        .map_err(|error| vec![cargo_build_error(error.clone())])?;
    tools
        .validate_configuration()
        .map_err(|error| vec![cargo_build_error(error)])?;
    let native_context = sifr_sysroot::NativeBuildContext {
        toolchain: tools.clone(),
        target: tools.target().to_owned(),
        profile: "release".to_owned(),
        flags_id: tools.identity().to_owned(),
        features_id: dependency_plan.cache_fingerprint.clone(),
        resolution_id: dependency_plan.dependency_input_fingerprint(),
        python_loader_id: generated_project.cache_key_fragment.clone(),
        trust_policy_id: sifr_sysroot::sha256_hex(
            generated_project.interop.cache_key_fragment().as_bytes(),
        ),
        destination: cache_scope.to_path_buf(),
    };
    cache_key.push_str("\n[native-build-context]\n");
    cache_key.push_str(native_context.identity().as_str());
    if let Some(seed) = cargo_resolution.normal_seed_cache_fragment() {
        cache_key.push_str("\n[normal-authority-seed]\n");
        cache_key.push_str(&seed);
    }
    let required_paths = [
        Path::new(project_name).join("target"),
        binary_relative_path(project_name),
    ];
    let required_refs: Vec<&Path> = required_paths.iter().map(PathBuf::as_path).collect();
    let prepared = prepare_cached_artifact(
        native_context.identity().as_str(),
        cache_namespace,
        cache_scope,
        &cache_key,
        &required_refs,
    )?;
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
    let executable = run_cargo_build(
        project_path,
        python_interpreter.as_deref(),
        validate_native_links,
        &trusted_native_links,
        dependency_plan,
        cargo_resolution,
    )?;
    let destination = cached_binary_path(
        project_path.parent().unwrap_or(Path::new(".")),
        project_name,
    );
    if executable != destination {
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| vec![cargo_build_error(error.to_string())])?;
        }
        std::fs::copy(&executable, &destination).map_err(|error| {
            vec![cargo_build_error(format!(
                "failed to publish native executable: {error}"
            ))]
        })?;
    }
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

    let loader_script = generated_project
        .python_runtime
        .as_ref()
        .map(super::python_runtime::PackagePythonRuntime::native_loader_build_script)
        .transpose()
        .map_err(|message| vec![build_error(message)])?
        .flatten();
    let build_script = project_path.join("build.rs");
    if let Some(source) = loader_script {
        write_project_file(&build_script, source, "Python loader build script")?;
    } else if build_script.exists() {
        std::fs::remove_file(&build_script).map_err(|error| {
            vec![build_error(format!(
                "failed to remove obsolete Python loader build script: {error}"
            ))]
        })?;
    }

    let main_rs = format!(
        "{}{}",
        generated_project.bridge_root_declaration(),
        generated_project.main_rs
    );
    write_project_file(&src_dir.join("main.rs"), main_rs, "main.rs")?;

    for (module, source) in generated_project.bridge_modules {
        let path = if module.contains("::") {
            rust_module_file_path(&module.replace("::", "."))
        } else {
            PathBuf::from(&module).join("mod.rs")
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

pub(super) fn canonical_rust_module_path(path: &Path) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
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
) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
    let mut command = cargo_resolution.cargo_command()?;
    command.args(sysroot_cargo_config_args(dependency_plan));
    command
        .args([
            "build",
            "--release",
            "--quiet",
            "--message-format=json-render-diagnostics",
        ])
        .arg("--manifest-path")
        .arg(project_path.join("Cargo.toml"));
    if let Some(argument) = cargo_resolution.lock_mode.cargo_arg() {
        command.arg(argument);
    }
    // Generated projects are materialized and cached with their own `target/`
    // directory. Inheriting an outer CARGO_TARGET_DIR moves binaries away from
    // the reported artifact paths and breaks cache completeness checks.
    command.arg("--target-dir").arg(project_path.join("target"));
    configure_hermetic_build_environment(&mut command);
    if let Some(python_interpreter) = python_interpreter {
        command.env("PYO3_PYTHON", python_interpreter);
    }
    record_cargo_invocation("final-build", cargo_resolution.lock_mode, &command);
    let output = crate::process_execution::output(&mut command).map_err(|error| {
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
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|message| {
            message.get("reason").and_then(serde_json::Value::as_str) == Some("compiler-artifact")
        })
        .find_map(|message| {
            message
                .get("executable")
                .and_then(serde_json::Value::as_str)
                .map(PathBuf::from)
        })
        .ok_or_else(|| {
            vec![cargo_build_error(
                "cargo did not report a native executable".to_owned(),
            )]
        })
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
    let mut identity = sifr_identity::IdentityEncoder::new("native-project-source-v2");
    identity.field("project", project_name.as_bytes());
    identity.field(
        "manifest",
        generate_dependency_cargo_toml_with_interop(
            project_name,
            dependency_plan,
            &generated_project.interop,
        )
        .as_bytes(),
    );
    identity.field("main", generated_project.main_rs.as_bytes());
    identity.field(
        "python-selected-library",
        generated_project
            .python_runtime
            .as_ref()
            .and_then(super::python_runtime::PackagePythonRuntime::selected_library)
            .unwrap_or("")
            .as_bytes(),
    );
    for (name, code) in &generated_project.support_modules {
        identity.field("support-name", name.as_bytes());
        identity.field("support-source", code.as_bytes());
    }
    for (name, code) in &generated_project.bridge_modules {
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
    identity.field(
        "context",
        generated_project
            .cache_key_fragment
            .as_deref()
            .unwrap_or("")
            .as_bytes(),
    );
    identity.finish()
}

#[cfg(test)]
#[path = "materialize_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "materialize_field_identity_tests.rs"]
mod field_identity_tests;
