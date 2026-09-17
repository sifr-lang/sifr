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
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub(super) struct MaterializedBinaryProject {
    pub(super) native_executable: PathBuf,
    pub(super) native_libraries: Vec<PathBuf>,
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
    let _publication = super::native_storage::publication_lock(&project_path)
        .map_err(|error| vec![build_error(error.to_string())])?;
    let runtime_contract = if let Some(runtime) = &generated_project.python_runtime {
        let library_sha256 = runtime
            .selected_library()
            .map(|path| sifr_sysroot::sha256_file(Path::new(path)))
            .transpose()
            .map_err(|error| {
                vec![build_error(format!(
                    "cannot identify exported Python library: {error}"
                ))]
            })?;
        Some(serde_json::json!({
            "schema": 1,
            "mode": "external-runtime",
            "contract": "Deploy the selected CPython environment at the declared paths. The executable validates the loaded shared-library path and content before initialization.",
            "interpreter": runtime.interpreter(),
            "shared_library": runtime.selected_library(),
            "shared_library_sha256": library_sha256,
            "relocatable": false,
        }))
    } else {
        None
    };
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
        if let Some(contract) = &runtime_contract {
            std::fs::write(
                local_project_path.join("sifr-python-runtime.json"),
                serde_json::to_vec_pretty(contract)
                    .map_err(|error| vec![build_error(error.to_string())])?,
            )
            .map_err(|error| vec![build_error(error.to_string())])?;
        }
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
    // Always run Cargo before consulting finalized output: build scripts and
    // local dependencies can change without changing generated Rust.
    let family = super::native_storage::NativeFamily::acquire(
        tools.identity(),
        &format!(
            "{:?}:{}",
            dependency_plan.cargo_vendor_mode,
            dependency_plan.sysroot_root.display()
        ),
        &python_environment(&generated_project),
        &format!("{:?}", generated_project.interop.rust.trust_requirements),
    )
    .map_err(|error| vec![build_error(error.to_string())])?;
    let scope = cache_scope
        .canonicalize()
        .unwrap_or_else(|_| cache_scope.to_path_buf());
    let project_root = family
        .project(&scope, project_name)
        .map_err(|error| vec![build_error(error.to_string())])?;
    let report = materialize_binary_project_at_path_with_target(
        &project_root,
        project_name,
        generated_project,
        &dependency_plan,
        cargo_resolution,
        &family.target(),
    )?;
    let snapshot = super::native_storage::NativeSnapshot::inspect(
        &report.native_executable,
        &binary_relative_path(project_name),
    )
    .and_then(|snapshot| {
        snapshot.with_runtime(
            &report.native_libraries,
            &binary_relative_path(project_name),
        )
    })
    .map_err(|error| vec![build_error(error.to_string())])?;
    cache_key.push_str("\n[final-native-bundle]\n");
    cache_key.push_str(&snapshot.identity);
    let required_refs = snapshot.required();
    match prepare_cached_artifact(
        native_context.identity().as_str(),
        cache_namespace,
        cache_scope,
        &cache_key,
        &required_refs,
    )? {
        PreparedArtifactCache::Hit(entry) => {
            snapshot
                .verify(entry.workspace_root())
                .map_err(|error| vec![build_error(error.to_string())])?;
            Ok((entry, Some(report), sysroot))
        }
        PreparedArtifactCache::Miss(pending) => {
            snapshot
                .capture(pending.workspace_root())
                .map_err(|error| vec![build_error(error.to_string())])?;
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

pub(super) fn materialize_binary_project_at_path(
    project_path: &Path,
    project_name: &str,
    generated_project: GeneratedBinaryProject,
    dependency_plan: &SysrootDependencyPlan,
    cargo_resolution: &CargoResolutionPolicy,
) -> Result<MaterializedBinaryProject, Vec<RenderedDiagnostic>> {
    let _publication = super::native_storage::publication_lock(project_path)
        .map_err(|error| vec![build_error(error.to_string())])?;
    let tools = cargo_resolution
        .native_toolchain
        .as_ref()
        .map_err(|error| vec![cargo_build_error(error.clone())])?;
    let family = super::native_storage::NativeFamily::acquire(
        tools.identity(),
        &format!(
            "{:?}:{}",
            dependency_plan.cargo_vendor_mode,
            dependency_plan.sysroot_root.display()
        ),
        &python_environment(&generated_project),
        &format!("{:?}", generated_project.interop.rust.trust_requirements),
    )
    .map_err(|error| vec![build_error(error.to_string())])?;
    materialize_binary_project_at_path_with_target(
        project_path,
        project_name,
        generated_project,
        dependency_plan,
        cargo_resolution,
        &family.target(),
    )
}

pub(super) fn materialize_binary_project_at_path_with_target(
    project_path: &Path,
    project_name: &str,
    generated_project: GeneratedBinaryProject,
    dependency_plan: &SysrootDependencyPlan,
    cargo_resolution: &CargoResolutionPolicy,
    target: &Path,
) -> Result<MaterializedBinaryProject, Vec<RenderedDiagnostic>> {
    let python_interpreter = generated_project
        .python_runtime
        .as_ref()
        .map(|runtime| runtime.interpreter().to_path_buf());
    let sysroot = sysroot_report(dependency_plan);
    let validate_native_links = should_validate_native_link_evidence(&generated_project);
    let trusted_native_links = trusted_native_links(&generated_project, dependency_plan);
    let materialize_start = std::time::Instant::now();
    let root_id = sifr_sysroot::sha256_hex(project_path.as_os_str().as_encoded_bytes());
    let target_name = format!("{project_name}_{}", &root_id[..16]);
    materialize_binary_project_files_with_target(
        project_path,
        project_name,
        generated_project,
        dependency_plan,
        Some(&target_name),
    )?;
    let materialize_elapsed = materialize_start.elapsed();

    let cargo_start = std::time::Instant::now();
    let cargo_prefix_args = sysroot_cargo_config_args(dependency_plan);
    let prepared_resolution =
        prepare_cargo_resolution(project_path, cargo_resolution, &cargo_prefix_args)?;
    let (executable, native_libraries) = run_cargo_build(
        project_path,
        python_interpreter.as_deref(),
        validate_native_links,
        &trusted_native_links,
        dependency_plan,
        cargo_resolution,
        target,
    )?;
    let destination = cached_binary_path(
        project_path.parent().unwrap_or(Path::new(".")),
        project_name,
    );
    let output_name = destination
        .file_name()
        .ok_or_else(|| vec![build_error("native output has no filename".to_owned())])?;
    let output_parent = destination
        .parent()
        .ok_or_else(|| vec![build_error("native output has no directory".to_owned())])?;
    super::native_storage::NativeSnapshot::inspect(&executable, Path::new(output_name))
        .and_then(|snapshot| snapshot.with_runtime(&native_libraries, Path::new(output_name)))
        .and_then(|snapshot| snapshot.capture(output_parent))
        .map_err(|error| {
            vec![cargo_build_error(format!(
                "failed to publish native output bundle: {error}"
            ))]
        })?;
    prepared_resolution.assert_unchanged()?;
    let cargo_elapsed = cargo_start.elapsed();

    Ok(MaterializedBinaryProject {
        native_executable: executable,
        native_libraries,
        binary_path: cached_binary_path(
            project_path.parent().unwrap_or(Path::new(".")),
            project_name,
        ),
        sysroot,
        materialize_elapsed,
        cargo_elapsed,
    })
}

fn python_environment(project: &GeneratedBinaryProject) -> String {
    project
        .python_runtime
        .as_ref()
        .map_or_else(String::new, |runtime| {
            format!(
                "{}:{}",
                runtime.probe_digest(),
                runtime.selected_library().unwrap_or("")
            )
        })
}

fn sysroot_report(dependency_plan: &SysrootDependencyPlan) -> BuildSysrootReport {
    BuildSysrootReport::from_dependency_plan(dependency_plan)
}

pub(super) fn materialize_binary_project_files(
    project_path: &Path,
    project_name: &str,
    generated_project: GeneratedBinaryProject,
    dependency_plan: &SysrootDependencyPlan,
) -> Result<(), Vec<RenderedDiagnostic>> {
    materialize_binary_project_files_with_target(
        project_path,
        project_name,
        generated_project,
        dependency_plan,
        None,
    )
}

fn materialize_binary_project_files_with_target(
    project_path: &Path,
    project_name: &str,
    generated_project: GeneratedBinaryProject,
    dependency_plan: &SysrootDependencyPlan,
    target_name: Option<&str>,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let src_dir = project_path.join("src");
    let mut current_files = BTreeSet::new();
    std::fs::create_dir_all(&src_dir).map_err(|error| {
        vec![build_error(format!(
            "failed to create output directory: {error}"
        ))]
    })?;

    let mut cargo_toml = generate_dependency_cargo_toml_with_interop(
        project_name,
        dependency_plan,
        &generated_project.interop,
    );

    if let Some(target_name) = target_name {
        let _ = write!(
            cargo_toml,
            "\n[[bin]]\nname = {target_name:?}\npath = \"src/main.rs\"\n"
        );
    }
    write_project_file(&project_path.join("Cargo.toml"), cargo_toml, "Cargo.toml")?;

    let loader_script = generated_project
        .python_runtime
        .as_ref()
        .map(super::python_runtime::PackagePythonRuntime::native_loader_build_script)
        .transpose()
        .map_err(|message| vec![build_error(message)])?
        .flatten();
    write_project_file(
        &project_path.join("build.rs"),
        super::native_storage::loader_build_script(loader_script),
        "native loader build script",
    )?;

    let main_rs = format!(
        "{}{}",
        generated_project.bridge_root_declaration(),
        generated_project.main_rs
    );
    current_files.insert(src_dir.join("main.rs"));
    write_project_file(&src_dir.join("main.rs"), main_rs, "main.rs")?;

    for (module, source) in generated_project.bridge_modules {
        let path = if module.contains("::") {
            rust_module_file_path(&module.replace("::", "."))
        } else {
            PathBuf::from(&module).join("mod.rs")
        };
        let canonical_path = canonical_rust_module_path(&path)?;
        current_files.insert(src_dir.join(&canonical_path));
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
        current_files.insert(src_dir.join(&file_name));
        write_project_file(
            &src_dir.join(&file_name),
            code,
            &file_name.display().to_string(),
        )?;
    }

    for (namespace_path, contents) in namespace_contents {
        let namespace_path = canonical_rust_module_path(&namespace_path)?;
        current_files.insert(src_dir.join(&namespace_path));
        write_project_file(
            &src_dir.join(&namespace_path),
            contents,
            &namespace_path.display().to_string(),
        )?;
    }

    super::native_storage::remove_stale(&src_dir, &current_files)
        .map_err(|error| vec![build_error(error.to_string())])?;
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
    target: &Path,
) -> Result<(PathBuf, Vec<PathBuf>), Vec<RenderedDiagnostic>> {
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
    // Cargo's mutable family storage is explicit; finalized results are
    // independent bundles captured while the family lease remains held.
    command.arg("--target-dir").arg(target);
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
        let mut stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if let Ok(event) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(rendered) = event["message"]["rendered"].as_str() {
                    stderr.push_str(rendered);
                }
            }
        }
        if let Some(diagnostic) = cargo_lock_mode_diagnostic("cargo build", &stderr) {
            return Err(vec![diagnostic]);
        }
        return Err(vec![cargo_build_error(format!(
            "cargo build failed:\n{stderr}"
        ))]);
    }
    let executable = String::from_utf8_lossy(&output.stdout)
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
        })?;
    let libraries = super::native_storage::runtime_libraries(&output.stdout, target)
        .map_err(|error| vec![cargo_build_error(error.to_string())])?;
    Ok((executable, libraries))
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
    super::native_storage::write_changed(path, contents)
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
pub(super) mod tests;

#[cfg(test)]
#[path = "materialize_field_identity_tests.rs"]
mod field_identity_tests;
