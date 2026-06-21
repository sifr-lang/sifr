use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use crate::manifest::sifr::SifrManifest;
use crate::projection_bridge;
use crate::source::layout::canonical_pure_marker_source;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_VERSION: &str = "0.1.0";
const RUST_EDITION: &str = "2024";
const SIFR_EDITION: &str = "2026";
const SIFR_VERSION_REQ: &str = ">=0.3,<0.4";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitPackageKind {
    Lib,
    Bin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitPackageOptions {
    pub target_dir: PathBuf,
    pub sifr_name: String,
    pub kind: InitPackageKind,
    pub force: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectionCheck {
    pub diagnostics: Vec<PackageDiagnostic>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectionRepair {
    pub diagnostics: Vec<PackageDiagnostic>,
    pub wrote_files: Vec<PathBuf>,
}

pub fn init_package(options: &InitPackageOptions) -> Result<Vec<PathBuf>, PackageDiagnostic> {
    validate_target_dir(options)?;
    let mut written = Vec::new();
    fs::create_dir_all(options.target_dir.join("src")).map_err(|error| {
        projection_io_diagnostic(
            &options.target_dir,
            format!("could not create src: {error}"),
        )
    })?;

    write_new_file(
        &options.target_dir.join("sifr.toml"),
        &render_sifr_toml(&options.sifr_name),
        options.force,
        &mut written,
    )?;
    write_new_file(
        &options.target_dir.join("Cargo.toml"),
        &render_cargo_toml(&options.sifr_name),
        options.force,
        &mut written,
    )?;
    write_new_file(
        &options.target_dir.join("src/lib.rs"),
        &format!("{}\n", canonical_pure_marker_source()),
        options.force,
        &mut written,
    )?;
    match options.kind {
        InitPackageKind::Lib => write_new_file(
            &options.target_dir.join("src/__init__.sifr"),
            "",
            options.force,
            &mut written,
        )?,
        InitPackageKind::Bin => write_new_file(
            &options.target_dir.join("src/main.sifr"),
            "def main():\n    pass\n",
            options.force,
            &mut written,
        )?,
    }
    Ok(written)
}

pub fn check_projection(package_root: &Path) -> ProjectionCheck {
    ProjectionCheck {
        diagnostics: projection_diagnostics(package_root),
    }
}

pub fn repair_projection(package_root: &Path, check: bool) -> ProjectionRepair {
    let diagnostics = projection_diagnostics(package_root);
    if check || diagnostics.is_empty() {
        return ProjectionRepair {
            diagnostics,
            wrote_files: Vec::new(),
        };
    }

    let mut wrote_files = Vec::new();
    if let Ok(manifest) = load_manifest(package_root) {
        if fs::write(
            package_root.join("Cargo.toml"),
            render_cargo_toml_for_manifest(&manifest),
        )
        .is_ok()
        {
            wrote_files.push(package_root.join("Cargo.toml"));
        }
        let bridge_repair =
            projection_bridge::repair(package_root, &manifest, &projection_cargo_id(package_root));
        wrote_files.extend(bridge_repair.wrote_files);
    }
    if !package_root.join("src/lib.rs").exists()
        && fs::create_dir_all(package_root.join("src")).is_ok()
        && fs::write(
            package_root.join("src/lib.rs"),
            format!("{}\n", canonical_pure_marker_source()),
        )
        .is_ok()
    {
        wrote_files.push(package_root.join("src/lib.rs"));
    }

    ProjectionRepair {
        diagnostics: projection_diagnostics(package_root),
        wrote_files,
    }
}

fn validate_target_dir(options: &InitPackageOptions) -> Result<(), PackageDiagnostic> {
    if options.target_dir.exists()
        && !options.force
        && fs::read_dir(&options.target_dir)
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(false)
    {
        return Err(projection_io_diagnostic(
            &options.target_dir,
            "target directory is not empty; pass --force to create missing Sifr-owned files",
        ));
    }
    if options.target_dir.join("Cargo.toml").exists()
        || options.target_dir.join("sifr.toml").exists()
    {
        return Err(projection_io_diagnostic(
            &options.target_dir,
            "existing Cargo.toml or sifr.toml requires sifr repair --check or migration",
        ));
    }
    Ok(())
}

fn write_new_file(
    path: &Path,
    contents: &str,
    force: bool,
    written: &mut Vec<PathBuf>,
) -> Result<(), PackageDiagnostic> {
    if path.exists() {
        if force {
            return Ok(());
        }
        return Err(projection_io_diagnostic(
            path,
            "file already exists; pass --force to leave it untouched",
        ));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            projection_io_diagnostic(path, format!("could not create parent directory: {error}"))
        })?;
    }
    fs::write(path, contents).map_err(|error| {
        projection_io_diagnostic(path, format!("could not write file: {error}"))
    })?;
    written.push(path.to_path_buf());
    Ok(())
}

fn projection_diagnostics(package_root: &Path) -> Vec<PackageDiagnostic> {
    let mut diagnostics = Vec::new();
    let Ok(manifest) = load_manifest(package_root) else {
        diagnostics.push(PackageDiagnostic::projection_manifest_pointer_drift(
            &projection_cargo_id(package_root),
            package_root.join("sifr.toml"),
            "sifr.toml is missing or invalid",
        ));
        return diagnostics;
    };
    let cargo_toml = package_root.join("Cargo.toml");
    let cargo_source = fs::read_to_string(&cargo_toml).unwrap_or_default();
    if !cargo_source.contains("[package.metadata.sifr]")
        || !cargo_source.contains("manifest = \"sifr.toml\"")
    {
        diagnostics.push(PackageDiagnostic::projection_manifest_pointer_drift(
            &projection_cargo_id(package_root),
            cargo_toml.clone(),
            "missing [package.metadata.sifr] manifest pointer",
        ));
    }
    for required in ["sifr.toml", "src/**/*.sifr", "src/lib.rs"] {
        if !cargo_source.contains(required) {
            diagnostics.push(PackageDiagnostic::projection_include_drift(
                &projection_cargo_id(package_root),
                cargo_toml.clone(),
                required,
            ));
        }
    }
    for required in projection_bridge::cargo_include_entries(&manifest)
        .into_iter()
        .filter(|required| required == "src/**/*.rs")
    {
        if !cargo_source.contains(&required) {
            diagnostics.push(PackageDiagnostic::projection_include_drift(
                &projection_cargo_id(package_root),
                cargo_toml.clone(),
                required,
            ));
        }
    }
    if !package_root.join("src/lib.rs").exists() && !manifest.declares_rust_backend() {
        diagnostics.push(PackageDiagnostic::projection_pure_marker_missing(
            &projection_cargo_id(package_root),
            package_root.join("src/lib.rs"),
        ));
    }
    diagnostics.extend(projection_bridge::diagnostics(
        package_root,
        &manifest,
        &projection_cargo_id(package_root),
    ));
    diagnostics
}

fn load_manifest(package_root: &Path) -> Result<SifrManifest, PackageDiagnostic> {
    SifrManifest::load(
        &projection_cargo_id(package_root),
        &package_root.join("sifr.toml"),
    )
}

fn render_sifr_toml(sifr_name: &str) -> String {
    format!(
        "[package]\nname = \"{sifr_name}\"\nedition = \"{SIFR_EDITION}\"\nsifr-version = \"{SIFR_VERSION_REQ}\"\n\n[source]\nroot = \"src\"\n"
    )
}

fn render_cargo_toml(sifr_name: &str) -> String {
    render_cargo_toml_with_includes(sifr_name, &default_cargo_include_entries())
}

fn render_cargo_toml_for_manifest(manifest: &SifrManifest) -> String {
    render_cargo_toml_with_includes(
        &manifest.package_name.0,
        &projection_bridge::cargo_include_entries(manifest),
    )
}

fn default_cargo_include_entries() -> Vec<String> {
    vec![
        "Cargo.toml".to_string(),
        "Cargo.lock".to_string(),
        "sifr.toml".to_string(),
        "src/**/*.sifr".to_string(),
        "src/lib.rs".to_string(),
        "README.md".to_string(),
        "LICENSE".to_string(),
    ]
}

fn render_cargo_toml_with_includes(sifr_name: &str, include_entries: &[String]) -> String {
    let cargo_name = format!("sifr-{}", kebab_case(sifr_name));
    let include = include_entries
        .iter()
        .map(|entry| format!("\"{entry}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "[package]\nname = \"{cargo_name}\"\nversion = \"{DEFAULT_VERSION}\"\nedition = \"{RUST_EDITION}\"\ninclude = [{include}]\n\n# sifr-managed\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n# end sifr-managed\n"
    )
}

fn kebab_case(value: &str) -> String {
    value.replace('_', "-").to_ascii_lowercase()
}

fn projection_cargo_id(package_root: &Path) -> CargoPackageId {
    CargoPackageId(format!("path+file://{}#projection", package_root.display()))
}

fn projection_io_diagnostic(path: &Path, reason: impl Into<String>) -> PackageDiagnostic {
    PackageDiagnostic::projection_manifest_pointer_drift(
        &projection_cargo_id(path),
        path.to_path_buf(),
        reason,
    )
}
