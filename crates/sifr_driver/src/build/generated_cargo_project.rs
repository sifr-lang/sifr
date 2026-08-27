use super::cargo_invocation_trace::record_cargo_invocation;
use super::cargo_manifest::{
    generate_dependency_cargo_toml_with_interop, sysroot_cargo_config_args,
};
use super::cargo_resolution::{
    CargoResolutionPolicy, cargo_lock_mode_diagnostic, prepare_cargo_resolution,
};
use super::rust_interop_bridge_sources::generated_bridge_sources;
use super::rust_interop_sqlx_offline::configure_hermetic_build_environment;
use crate::diagnostics::RenderedDiagnostic;
use crate::project::{namespace_module_files, rust_module_file_path};
use sifr_codegen::{InteropBuildPlan, RustInteropTrustRequirementKind};
use sifr_diagnostics::DiagnosticCode;
use sifr_stdlib_manifest::{SysrootCrate, SysrootDependencyPlan};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub(crate) struct GeneratedCargoProject {
    pub(crate) name: String,
    pub(crate) crate_root_file: PathBuf,
    pub(crate) crate_root_source: String,
    pub(crate) support_modules: BTreeMap<String, String>,
    pub(crate) support_main_alias: Option<PathBuf>,
    pub(crate) interop: InteropBuildPlan,
}

#[derive(Clone, Copy)]
pub(crate) enum GeneratedCargoCommand {
    BuildRelease,
    Test,
}

#[derive(Clone, Copy)]
pub(crate) struct GeneratedCargoExecution<'a> {
    pub(crate) python_interpreter: Option<&'a Path>,
    pub(crate) target_directory: Option<&'a Path>,
    pub(crate) additional_trusted_native_links: &'a BTreeSet<String>,
}

impl GeneratedCargoCommand {
    fn phase(self) -> &'static str {
        match self {
            Self::BuildRelease => "final-build",
            Self::Test => "test-build",
        }
    }

    fn context(self) -> &'static str {
        match self {
            Self::BuildRelease => "cargo build",
            Self::Test => "cargo test",
        }
    }

    fn args(self) -> &'static [&'static str] {
        match self {
            Self::BuildRelease => &[
                "build",
                "--release",
                "--quiet",
                "--message-format=json-render-diagnostics",
            ],
            Self::Test => &["test", "--message-format=json-render-diagnostics"],
        }
    }
}

pub(crate) fn generated_cargo_manifest(
    project_name: &str,
    dependency_plan: &SysrootDependencyPlan,
    interop: &InteropBuildPlan,
) -> String {
    generate_dependency_cargo_toml_with_interop(project_name, dependency_plan, interop)
}

pub(crate) fn materialize_generated_cargo_project(
    project_path: &Path,
    project: GeneratedCargoProject,
    dependency_plan: &SysrootDependencyPlan,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let src_dir = project_path.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|error| {
        vec![materialization_error(format!(
            "failed to create output directory: {error}"
        ))]
    })?;

    let cargo_toml = generated_cargo_manifest(&project.name, dependency_plan, &project.interop);
    write_project_file(&project_path.join("Cargo.toml"), cargo_toml, "Cargo.toml")?;

    let bridge_sources =
        generated_bridge_sources(&project.interop.rust.bridge_contracts.generated_types);
    let crate_root_source = if bridge_sources.is_empty() {
        project.crate_root_source
    } else {
        declare_bridge_module(&project.crate_root_source)
    };
    write_project_file(
        &src_dir.join(&project.crate_root_file),
        crate_root_source,
        &project.crate_root_file.display().to_string(),
    )?;
    for (path, source) in bridge_sources {
        write_project_file(&src_dir.join(&path), source, &path.display().to_string())?;
    }

    materialize_support_modules(
        &src_dir,
        project.support_modules,
        project.support_main_alias.as_deref(),
    )
}

fn declare_bridge_module(source: &str) -> String {
    let mut insertion = 0;
    for line in source.split_inclusive('\n') {
        if !line.trim_start().starts_with("#![") {
            break;
        }
        insertion += line.len();
    }
    let mut result = String::with_capacity(source.len() + 24);
    result.push_str(&source[..insertion]);
    result.push_str("pub mod __sifr_bridge;\n");
    result.push_str(&source[insertion..]);
    result
}

pub(crate) fn run_generated_cargo_command(
    project_path: &Path,
    command_kind: GeneratedCargoCommand,
    execution: GeneratedCargoExecution<'_>,
    interop: &InteropBuildPlan,
    dependency_plan: &SysrootDependencyPlan,
    cargo_resolution: &CargoResolutionPolicy,
) -> Result<Output, Vec<RenderedDiagnostic>> {
    let cargo_prefix_args = sysroot_cargo_config_args(dependency_plan);
    let prepared_resolution =
        prepare_cargo_resolution(project_path, cargo_resolution, &cargo_prefix_args)?;
    let mut command = Command::new("cargo");
    command
        .args(&cargo_prefix_args)
        .args(command_kind.args())
        .current_dir(project_path);
    if let Some(argument) = cargo_resolution.lock_mode.cargo_arg() {
        command.arg(argument);
    }
    if let Some(target_directory) = execution.target_directory {
        command.env("CARGO_TARGET_DIR", target_directory);
    } else {
        command.env_remove("CARGO_TARGET_DIR");
    }
    configure_hermetic_build_environment(&mut command);
    if let Some(python_interpreter) = execution.python_interpreter {
        command.env("PYO3_PYTHON", python_interpreter);
    }
    record_cargo_invocation(command_kind.phase(), cargo_resolution.lock_mode, &command);
    let output = command.output().map_err(|error| {
        vec![cargo_execution_error(format!(
            "failed to run {}: {error}",
            command_kind.context()
        ))]
    })?;

    if should_validate_native_link_evidence(interop) {
        let mut trusted_native_links = trusted_native_links(interop, dependency_plan);
        trusted_native_links.extend(execution.additional_trusted_native_links.iter().cloned());
        validate_native_link_evidence(&output.stdout, &trusted_native_links)?;
    }
    prepared_resolution.assert_unchanged()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if let Some(diagnostic) = cargo_lock_mode_diagnostic(command_kind.context(), &stderr) {
            return Err(vec![diagnostic]);
        }
    }
    Ok(output)
}

fn materialize_support_modules(
    src_dir: &Path,
    support_modules: BTreeMap<String, String>,
    support_main_alias: Option<&Path>,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let support_module_names = support_modules.keys().cloned().collect::<Vec<_>>();
    let mut namespace_contents = namespace_module_files(&support_module_names)
        .into_iter()
        .map(|namespace| {
            let mut contents = String::new();
            for declaration in namespace.declarations {
                contents.push_str("pub mod ");
                contents.push_str(&declaration);
                contents.push_str(";\n");
            }
            (
                remap_support_main_path(namespace.path, support_main_alias),
                contents,
            )
        })
        .collect::<BTreeMap<_, _>>();

    for (module_name, code) in support_modules {
        let namespace_path =
            remap_support_main_path(namespace_module_file_path(&module_name), support_main_alias);
        if let Some(contents) = namespace_contents.get_mut(&namespace_path) {
            if !contents.is_empty() && !contents.ends_with('\n') {
                contents.push('\n');
            }
            contents.push_str(&code);
            continue;
        }
        let file_name =
            remap_support_main_path(rust_module_file_path(&module_name), support_main_alias);
        write_project_file(
            &src_dir.join(&file_name),
            code,
            &file_name.display().to_string(),
        )?;
    }

    for (namespace_path, contents) in namespace_contents {
        write_project_file(
            &src_dir.join(&namespace_path),
            contents,
            &namespace_path.display().to_string(),
        )?;
    }
    Ok(())
}

fn remap_support_main_path(path: PathBuf, support_main_alias: Option<&Path>) -> PathBuf {
    let Some(alias) = support_main_alias else {
        return path;
    };
    if path == Path::new("main.rs") {
        return alias.to_path_buf();
    }
    let mut components = path.components();
    let Some(first) = components.next() else {
        return path;
    };
    if first.as_os_str() != "main" {
        return path;
    }
    let remainder = components.collect::<PathBuf>();
    if remainder.as_os_str().is_empty() || remainder == Path::new("mod.rs") {
        return alias.to_path_buf();
    }
    alias.with_extension("").join(remainder)
}

fn namespace_module_file_path(module_name: &str) -> PathBuf {
    let mut path = PathBuf::new();
    for component in module_name.split('.') {
        path.push(component);
    }
    path.push("mod.rs");
    path
}

pub(super) fn trusted_native_links(
    interop: &InteropBuildPlan,
    dependency_plan: &SysrootDependencyPlan,
) -> BTreeSet<String> {
    let mut trusted = interop
        .rust
        .trust_requirements
        .iter()
        .filter(|requirement| {
            requirement.trusted && requirement.kind == RustInteropTrustRequirementKind::NativeLinks
        })
        .map(|requirement| requirement.required_entry.clone())
        .collect::<BTreeSet<_>>();
    trusted.extend(sysroot_trusted_native_links(dependency_plan));
    trusted
}

pub(super) fn should_validate_native_link_evidence(interop: &InteropBuildPlan) -> bool {
    let rust = &interop.rust;
    !rust.declarations.is_empty()
        || !rust.resolved_targets.is_empty()
        || !rust.trust_requirements.is_empty()
        || !rust.probe_plan.probes.is_empty()
        || !rust.bridge_sources.is_empty()
        || rust.cargo_inputs.is_some()
}

pub(super) fn sysroot_trusted_native_links(
    dependency_plan: &SysrootDependencyPlan,
) -> BTreeSet<String> {
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

pub(super) fn validate_native_link_evidence(
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
            let link_name = linked_lib
                .rsplit_once('=')
                .map_or(linked_lib, |(_, name)| name);
            if !trusted_native_links.contains(link_name) {
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

fn write_project_file(
    path: &Path,
    contents: impl AsRef<[u8]>,
    label: &str,
) -> Result<(), Vec<RenderedDiagnostic>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            vec![materialization_error(format!(
                "failed to create {label}: {error}"
            ))]
        })?;
    }
    std::fs::write(path, contents).map_err(|error| {
        vec![materialization_error(format!(
            "failed to write {label}: {error}"
        ))]
    })
}

fn materialization_error(message: String) -> RenderedDiagnostic {
    crate::diagnostics::diagnostic_with_code(message, DiagnosticCode::BUILD_MATERIALIZATION_FAILURE)
}

pub(super) fn cargo_execution_error(message: String) -> RenderedDiagnostic {
    crate::diagnostics::diagnostic_with_code(message, DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE)
}

#[cfg(test)]
mod tests {
    use super::{materialize_support_modules, validate_native_link_evidence};
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};

    #[test]
    fn shared_support_materializer_combines_namespace_source_and_children() {
        let root = temp_dir("support_namespace");
        materialize_support_modules(
            &root,
            BTreeMap::from([
                ("helpers".to_string(), "pub fn root() {}\n".to_string()),
                (
                    "helpers.nodes".to_string(),
                    "pub fn child() {}\n".to_string(),
                ),
            ]),
            None,
        )
        .expect("support modules should materialize");

        let namespace = std::fs::read_to_string(root.join("helpers/mod.rs"))
            .expect("namespace module should exist");
        assert_eq!(namespace, "pub mod nodes;\npub fn root() {}\n");
        assert!(root.join("helpers/nodes.rs").is_file());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn shared_support_materializer_remaps_test_main_namespace_to_alias() {
        let root = temp_dir("support_main_alias");
        materialize_support_modules(
            &root,
            BTreeMap::from([
                ("main".to_string(), "pub fn root() {}\n".to_string()),
                ("main.child".to_string(), "pub fn child() {}\n".to_string()),
            ]),
            Some(Path::new("__sifr_support_main.rs")),
        )
        .expect("support main modules should materialize");

        let main = std::fs::read_to_string(root.join("__sifr_support_main.rs"))
            .expect("support main alias should exist");
        assert_eq!(main, "pub mod child;\npub fn root() {}\n");
        assert!(root.join("__sifr_support_main/child.rs").is_file());
        assert!(!root.join("main").exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn native_link_evidence_rejects_untrusted_build_script_output() {
        let stdout = br#"{"reason":"build-script-executed","linked_libs":["dylib=ssl"]}"#;
        let diagnostics = validate_native_link_evidence(stdout, &BTreeSet::new())
            .expect_err("untrusted link evidence should fail");
        assert_eq!(diagnostics[0].code, "SIFR-RUST-TRUST-0001");
        validate_native_link_evidence(stdout, &BTreeSet::from(["ssl".to_string()]))
            .expect("trusted link should pass");
    }

    fn temp_dir(label: &str) -> PathBuf {
        let unique = format!(
            "sifr_generated_project_{label}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&root).expect("temporary directory should be created");
        root
    }
}
