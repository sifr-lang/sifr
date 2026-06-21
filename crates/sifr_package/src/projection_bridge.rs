use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use crate::manifest::sifr::SifrManifest;
use crate::projection_rust_keywords::is_rust_keyword;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const MANAGED_BEGIN: &str = "// sifr-managed: rust-interop bridge projection v1";
const MANAGED_END: &str = "// end sifr-managed";
const GENERATED_BRIDGE_FILE: &str = "src/__sifr_bridge.rs";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RustProjectionRepair {
    pub diagnostics: Vec<PackageDiagnostic>,
    pub wrote_files: Vec<PathBuf>,
}

#[must_use]
pub(crate) fn has_local_bridges(manifest: &SifrManifest) -> bool {
    !manifest.rust.bridges.is_empty()
}

#[must_use]
pub(crate) fn cargo_include_entries(manifest: &SifrManifest) -> Vec<String> {
    let mut entries = vec![
        "Cargo.toml".to_string(),
        "Cargo.lock".to_string(),
        "sifr.toml".to_string(),
        "src/**/*.sifr".to_string(),
        "src/lib.rs".to_string(),
    ];
    if has_local_bridges(manifest) {
        entries.push("src/**/*.rs".to_string());
    }
    entries.extend(["README.md".to_string(), "LICENSE".to_string()]);
    entries
}

pub(crate) fn diagnostics(
    package_root: &Path,
    manifest: &SifrManifest,
    cargo_package_id: &CargoPackageId,
) -> Vec<PackageDiagnostic> {
    let mut diagnostics = Vec::new();
    if !has_local_bridges(manifest) {
        return diagnostics;
    }

    require_managed_file(
        package_root,
        Path::new("src/lib.rs"),
        cargo_package_id,
        &mut diagnostics,
    );
    require_managed_file(
        package_root,
        Path::new("src/__sifr_bridge/mod.rs"),
        cargo_package_id,
        &mut diagnostics,
    );
    if package_root.join(GENERATED_BRIDGE_FILE).exists() {
        diagnostics.push(projection_conflict(
            cargo_package_id,
            package_root.join(GENERATED_BRIDGE_FILE),
            "crate::__sifr_bridge is reserved for generated Sifr bridge types",
        ));
    }
    for bridge_root in &manifest.rust.bridges {
        let Some(relative_root) = normalized_bridge_root(bridge_root) else {
            diagnostics.push(projection_conflict(
                cargo_package_id,
                package_root.join(bridge_root),
                "bridge roots must be relative paths inside src",
            ));
            continue;
        };
        if bridge_root_module_name(&relative_root).is_none() {
            diagnostics.push(projection_conflict(
                cargo_package_id,
                package_root.join(&relative_root),
                "bridge roots must be direct src children with Rust identifier names",
            ));
            continue;
        }
        require_managed_file(
            package_root,
            &relative_root.join("mod.rs"),
            cargo_package_id,
            &mut diagnostics,
        );
        diagnostics.extend(user_bridge_file_diagnostics(
            package_root,
            &relative_root,
            cargo_package_id,
        ));
    }
    diagnostics
}

pub(crate) fn repair(
    package_root: &Path,
    manifest: &SifrManifest,
    cargo_package_id: &CargoPackageId,
) -> RustProjectionRepair {
    let mut diagnostics = Vec::new();
    let mut wrote_files = Vec::new();
    if !has_local_bridges(manifest) {
        return RustProjectionRepair {
            diagnostics,
            wrote_files,
        };
    }

    write_managed_file(
        package_root,
        Path::new("src/lib.rs"),
        &render_lib_rs(manifest),
        cargo_package_id,
        &mut diagnostics,
        &mut wrote_files,
    );
    write_managed_file(
        package_root,
        Path::new("src/__sifr_bridge/mod.rs"),
        &render_generated_bridge_mod_rs(),
        cargo_package_id,
        &mut diagnostics,
        &mut wrote_files,
    );
    for bridge_root in &manifest.rust.bridges {
        let Some(relative_root) = normalized_bridge_root(bridge_root) else {
            diagnostics.push(projection_conflict(
                cargo_package_id,
                package_root.join(bridge_root),
                "bridge roots must be relative paths inside src",
            ));
            continue;
        };
        if bridge_root_module_name(&relative_root).is_none() {
            diagnostics.push(projection_conflict(
                cargo_package_id,
                package_root.join(&relative_root),
                "bridge roots must be direct src children with Rust identifier names",
            ));
            continue;
        }
        let bridge_mod = render_bridge_mod_rs(package_root, &relative_root, cargo_package_id);
        match bridge_mod {
            Ok(source) => write_managed_file(
                package_root,
                &relative_root.join("mod.rs"),
                &source,
                cargo_package_id,
                &mut diagnostics,
                &mut wrote_files,
            ),
            Err(error) => diagnostics.push(error),
        }
    }

    RustProjectionRepair {
        diagnostics,
        wrote_files,
    }
}

#[must_use]
pub(crate) fn required_archive_entries(
    package_root: &Path,
    manifest: &SifrManifest,
) -> BTreeSet<PathBuf> {
    let mut required = BTreeSet::new();
    if manifest.declares_rust_backend() {
        required.insert(PathBuf::from("Cargo.toml"));
    }
    if !has_local_bridges(manifest) {
        return required;
    }

    required.insert(PathBuf::from("src/lib.rs"));
    required.insert(PathBuf::from("src/__sifr_bridge/mod.rs"));
    for bridge_root in &manifest.rust.bridges {
        let Some(relative_root) = normalized_bridge_root(bridge_root) else {
            continue;
        };
        required.insert(relative_root.join("mod.rs"));
        required.extend(user_bridge_files(package_root, &relative_root).into_iter());
    }
    required
}

fn require_managed_file(
    package_root: &Path,
    relative_path: &Path,
    cargo_package_id: &CargoPackageId,
    diagnostics: &mut Vec<PackageDiagnostic>,
) {
    let path = package_root.join(relative_path);
    match fs::read_to_string(&path) {
        Ok(source) if is_managed(&source) => {}
        Ok(_) => diagnostics.push(projection_conflict(
            cargo_package_id,
            path,
            "Sifr-managed Rust bridge projection file is user-authored",
        )),
        Err(_) => diagnostics.push(projection_conflict(
            cargo_package_id,
            path,
            "Sifr-managed Rust bridge projection file is missing",
        )),
    }
}

fn write_managed_file(
    package_root: &Path,
    relative_path: &Path,
    contents: &str,
    cargo_package_id: &CargoPackageId,
    diagnostics: &mut Vec<PackageDiagnostic>,
    wrote_files: &mut Vec<PathBuf>,
) {
    let path = package_root.join(relative_path);
    if let Ok(existing) = fs::read_to_string(&path) {
        if !is_managed(&existing) {
            diagnostics.push(projection_conflict(
                cargo_package_id,
                path,
                "refusing to overwrite user-authored Rust bridge projection file",
            ));
            return;
        }
    }
    if let Some(parent) = path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            diagnostics.push(projection_conflict(
                cargo_package_id,
                path,
                format!("could not create projection directory: {error}"),
            ));
            return;
        }
    }
    if fs::write(&path, contents).is_ok() {
        wrote_files.push(path);
    }
}

fn render_lib_rs(manifest: &SifrManifest) -> String {
    let modules = manifest
        .rust
        .bridges
        .iter()
        .filter_map(|root| {
            normalized_bridge_root(root).and_then(|root| bridge_root_module_name(&root))
        })
        .collect::<BTreeSet<_>>();
    let mut source = format!("{MANAGED_BEGIN}\n");
    for module in modules {
        source.push_str(&format!("pub mod {module};\n"));
    }
    source.push_str("pub mod __sifr_bridge;\n");
    source.push_str(&format!("{MANAGED_END}\n"));
    source
}

fn render_generated_bridge_mod_rs() -> String {
    format!("{MANAGED_BEGIN}\n{MANAGED_END}\n")
}

fn render_bridge_mod_rs(
    package_root: &Path,
    bridge_root: &Path,
    cargo_package_id: &CargoPackageId,
) -> Result<String, PackageDiagnostic> {
    let mut modules = Vec::new();
    for entry in user_bridge_files(package_root, bridge_root) {
        let Some(stem) = entry.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if !is_rust_identifier(stem) || stem == "__sifr_bridge" {
            return Err(projection_conflict(
                cargo_package_id,
                package_root.join(entry),
                "bridge module filenames must be valid Rust identifiers",
            ));
        }
        modules.push(stem.to_string());
    }
    let mut source = format!("{MANAGED_BEGIN}\n");
    for module in modules {
        source.push_str(&format!("pub mod {module};\n"));
    }
    source.push_str(&format!("{MANAGED_END}\n"));
    Ok(source)
}

fn user_bridge_file_diagnostics(
    package_root: &Path,
    bridge_root: &Path,
    cargo_package_id: &CargoPackageId,
) -> Vec<PackageDiagnostic> {
    user_bridge_files(package_root, bridge_root)
        .into_iter()
        .filter_map(|entry| {
            let stem = entry.file_stem()?.to_str()?;
            if is_rust_identifier(stem) && stem != "__sifr_bridge" {
                return None;
            }
            Some(projection_conflict(
                cargo_package_id,
                package_root.join(entry),
                "bridge module filenames must be valid Rust identifiers",
            ))
        })
        .collect()
}

fn user_bridge_files(package_root: &Path, bridge_root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(package_root.join(bridge_root)) else {
        return Vec::new();
    };
    let mut files = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .filter(|path| path.file_name().is_some_and(|name| name != "mod.rs"))
        .filter_map(|path| path.strip_prefix(package_root).ok().map(Path::to_path_buf))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn normalized_bridge_root(path: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        return None;
    }
    let mut components = path.components();
    if components.next()?.as_os_str() != "src" {
        return None;
    }
    if path.components().any(|component| {
        !matches!(
            component,
            std::path::Component::Normal(_) | std::path::Component::CurDir
        )
    }) {
        return None;
    }
    Some(path.components().filter_map(normal_component).collect())
}

fn bridge_root_module_name(path: &Path) -> Option<String> {
    if path.components().count() != 2 {
        return None;
    }
    let name = path.file_name()?.to_str()?;
    is_rust_identifier(name).then(|| name.to_string())
}

fn normal_component(component: std::path::Component<'_>) -> Option<PathBuf> {
    match component {
        std::path::Component::Normal(value) => Some(PathBuf::from(value)),
        std::path::Component::CurDir => None,
        _ => None,
    }
}

fn is_managed(source: &str) -> bool {
    source.contains(MANAGED_BEGIN)
}

fn is_rust_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
        && !is_rust_keyword(value)
}

fn projection_conflict(
    cargo_package_id: &CargoPackageId,
    path: impl Into<PathBuf>,
    reason: impl Into<String>,
) -> PackageDiagnostic {
    PackageDiagnostic::projection_manifest_pointer_drift(cargo_package_id, path.into(), reason)
}
