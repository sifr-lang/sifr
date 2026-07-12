use crate::diag::PackageDiagnostic;
use crate::graph::derive::SifrPackageMetadata;
use crate::manifest::sifr::{PackageSourceRoot, SifrManifest};
use crate::CargoPackageId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

mod filesystem;
mod import_classification;
mod imports;
use filesystem::{
    collect_python_paths, discover_source_paths, misplaced_root_diagnostics, module_name,
    path_string, sha256_hex,
};
use import_classification::{classify_imports, relative_import_escape};
use imports::{collect_imports, RawImport};

pub const PYTHON_BRIDGE_ROOT: &str = "src/python_bridges";
pub const PYTHON_BRIDGE_INVENTORY: &str = "src/python_bridges/__sifr_inventory__.json";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PythonBridgeInventory {
    pub schema_version: u32,
    pub source_root: String,
    pub digest_algorithm: String,
    pub inventory_digest: String,
    pub modules: Vec<PythonBridgeModule>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PythonBridgeModule {
    pub module: String,
    pub source_path: String,
    pub source_digest: String,
    pub is_package: bool,
    pub imports: Vec<PythonBridgeImport>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum PythonBridgeImport {
    SamePackage { module: String },
    ThirdParty { root: String },
}

pub fn discover_python_bridge_inventory(
    package: &SifrPackageMetadata,
) -> Result<PythonBridgeInventory, Vec<PackageDiagnostic>> {
    discover_python_bridge_inventory_at(
        &package.package_root,
        &package.cargo_package_id,
        &package.manifest.source_roots,
    )
}

fn discover_python_bridge_inventory_at(
    package_root: &Path,
    cargo_package_id: &CargoPackageId,
    source_roots: &[PackageSourceRoot],
) -> Result<PythonBridgeInventory, Vec<PackageDiagnostic>> {
    let root = package_root.join(PYTHON_BRIDGE_ROOT);
    let mut diagnostics =
        misplaced_root_diagnostics(package_root, cargo_package_id, source_roots, &root);
    let source_paths = discover_source_paths(cargo_package_id, &root, &mut diagnostics);
    let mut parsed_modules = Vec::new();
    let mut modules_by_name = BTreeMap::<String, PathBuf>::new();

    for source_path in source_paths {
        let (module, is_package) = match module_name(&root, &source_path) {
            Ok(module) => module,
            Err(error) => {
                diagnostics.push(PackageDiagnostic::invalid_python_bridge_source(
                    cargo_package_id,
                    &source_path,
                    error.reason(),
                ));
                continue;
            }
        };
        if let Some(previous) = modules_by_name.insert(module.clone(), source_path.clone()) {
            diagnostics.push(PackageDiagnostic::invalid_python_bridge_source(
                cargo_package_id,
                &source_path,
                format!(
                    "module '{module}' is defined by both '{}' and '{}'",
                    previous.display(),
                    source_path.display()
                ),
            ));
            continue;
        }
        match parse_bridge_source(
            package_root,
            cargo_package_id,
            &source_path,
            module,
            is_package,
        ) {
            Ok(parsed) => parsed_modules.push(parsed),
            Err(error) => diagnostics.push(error),
        }
    }

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let known_modules = modules_by_name.keys().cloned().collect::<BTreeSet<_>>();
    let mut modules = parsed_modules
        .into_iter()
        .map(|parsed| parsed.finish(&known_modules))
        .collect::<Vec<_>>();
    modules.sort_by(|left, right| left.module.cmp(&right.module));
    let digest_input = serde_json::to_vec(&(PYTHON_BRIDGE_ROOT, &modules)).map_err(|error| {
        vec![PackageDiagnostic::invalid_python_bridge_source(
            cargo_package_id,
            &root,
            format!("could not serialize canonical bridge inventory: {error}"),
        )]
    })?;

    Ok(PythonBridgeInventory {
        schema_version: 1,
        source_root: PYTHON_BRIDGE_ROOT.to_string(),
        digest_algorithm: "sha256".to_string(),
        inventory_digest: sha256_hex(&digest_input),
        modules,
    })
}

pub fn write_python_bridge_inventory(
    package: &SifrPackageMetadata,
    inventory: &PythonBridgeInventory,
) -> Result<Option<PathBuf>, PackageDiagnostic> {
    write_python_bridge_inventory_at(&package.package_root, &package.cargo_package_id, inventory)
}

fn write_python_bridge_inventory_at(
    package_root: &Path,
    cargo_package_id: &CargoPackageId,
    inventory: &PythonBridgeInventory,
) -> Result<Option<PathBuf>, PackageDiagnostic> {
    if inventory.modules.is_empty() {
        return Ok(None);
    }
    let path = package_root.join(PYTHON_BRIDGE_INVENTORY);
    let mut contents = serde_json::to_string_pretty(inventory).map_err(|error| {
        PackageDiagnostic::invalid_python_bridge_source(
            cargo_package_id,
            &path,
            format!("could not serialize generated inventory: {error}"),
        )
    })?;
    contents.push('\n');
    fs::write(&path, contents).map_err(|error| {
        PackageDiagnostic::invalid_python_bridge_source(
            cargo_package_id,
            &path,
            format!("could not write generated inventory: {error}"),
        )
    })?;
    Ok(Some(path))
}

pub fn validate_python_bridge_inventory_manifest(
    package: &SifrPackageMetadata,
    inventory: &PythonBridgeInventory,
) -> Result<(), PackageDiagnostic> {
    validate_python_bridge_inventory_manifest_at(
        &package.package_root,
        &package.cargo_package_id,
        inventory,
    )
}

fn validate_python_bridge_inventory_manifest_at(
    package_root: &Path,
    cargo_package_id: &CargoPackageId,
    inventory: &PythonBridgeInventory,
) -> Result<(), PackageDiagnostic> {
    let path = package_root.join(PYTHON_BRIDGE_INVENTORY);
    if inventory.modules.is_empty() {
        if path.exists() {
            return Err(PackageDiagnostic::invalid_python_bridge_source(
                cargo_package_id,
                &path,
                "generated inventory exists but the package has no bridge sources",
            ));
        }
        return Ok(());
    }
    let source = fs::read_to_string(&path).map_err(|error| {
        PackageDiagnostic::invalid_python_bridge_source(
            cargo_package_id,
            &path,
            format!("generated inventory is missing or unreadable: {error}"),
        )
    })?;
    let archived = serde_json::from_str::<PythonBridgeInventory>(&source).map_err(|error| {
        PackageDiagnostic::invalid_python_bridge_source(
            cargo_package_id,
            &path,
            format!("generated inventory JSON is invalid: {error}"),
        )
    })?;
    if &archived != inventory {
        return Err(PackageDiagnostic::invalid_python_bridge_source(
            cargo_package_id,
            &path,
            "generated inventory is stale; regenerate it before packaging",
        ));
    }
    Ok(())
}

pub(crate) fn python_bridge_projection_diagnostics(
    package_root: &Path,
    manifest: &SifrManifest,
    cargo_package_id: &CargoPackageId,
) -> Vec<PackageDiagnostic> {
    match discover_python_bridge_inventory_at(
        package_root,
        cargo_package_id,
        &manifest.source_roots,
    ) {
        Ok(inventory) => {
            validate_python_bridge_inventory_manifest_at(package_root, cargo_package_id, &inventory)
                .err()
                .into_iter()
                .collect()
        }
        Err(diagnostics) => diagnostics,
    }
}

pub(crate) fn repair_python_bridge_inventory(
    package_root: &Path,
    manifest: &SifrManifest,
    cargo_package_id: &CargoPackageId,
) -> Result<Option<PathBuf>, Vec<PackageDiagnostic>> {
    let inventory = discover_python_bridge_inventory_at(
        package_root,
        cargo_package_id,
        &manifest.source_roots,
    )?;
    write_python_bridge_inventory_at(package_root, cargo_package_id, &inventory)
        .map_err(|error| vec![error])
}

#[must_use]
pub fn required_python_bridge_archive_entries(package_root: &Path) -> BTreeSet<PathBuf> {
    let root = package_root.join(PYTHON_BRIDGE_ROOT);
    let mut paths = BTreeSet::new();
    collect_python_paths(&root, &mut paths, &mut Vec::new(), None);
    let mut required = paths
        .into_iter()
        .filter_map(|path| path.strip_prefix(package_root).ok().map(Path::to_path_buf))
        .collect::<BTreeSet<_>>();
    if !required.is_empty() {
        required.insert(PathBuf::from(PYTHON_BRIDGE_INVENTORY));
    }
    required
}

struct ParsedBridgeModule {
    module: String,
    source_path: String,
    source_digest: String,
    is_package: bool,
    raw_imports: Vec<RawImport>,
}

impl ParsedBridgeModule {
    fn finish(self, known_modules: &BTreeSet<String>) -> PythonBridgeModule {
        let imports = classify_imports(
            &self.module,
            self.is_package,
            &self.raw_imports,
            known_modules,
        );
        PythonBridgeModule {
            module: self.module,
            source_path: self.source_path,
            source_digest: self.source_digest,
            is_package: self.is_package,
            imports,
        }
    }
}

fn parse_bridge_source(
    package_root: &Path,
    cargo_package_id: &CargoPackageId,
    source_path: &Path,
    module: String,
    is_package: bool,
) -> Result<ParsedBridgeModule, PackageDiagnostic> {
    let source_bytes = fs::read(source_path).map_err(|error| {
        PackageDiagnostic::invalid_python_bridge_source(
            cargo_package_id,
            source_path,
            format!("could not read source: {error}"),
        )
    })?;
    let source = String::from_utf8(source_bytes).map_err(|_| {
        PackageDiagnostic::invalid_python_bridge_source(
            cargo_package_id,
            source_path,
            "bridge source must be UTF-8 encoded",
        )
    })?;
    let suite = sifr_syntax::parse_module_suite(&source, Some(&source_path.display().to_string()))
        .map_err(|errors| {
            let reason = errors
                .first()
                .map_or("invalid Python syntax", |error| error.message.as_str());
            PackageDiagnostic::invalid_python_bridge_source(
                cargo_package_id,
                source_path,
                format!("invalid Python syntax: {reason}"),
            )
        })?;
    let collected = collect_imports(&suite);
    if let Some(import) = relative_import_escape(&collected.raw_imports, &module, is_package) {
        return Err(PackageDiagnostic::invalid_python_bridge_source(
            cargo_package_id,
            source_path,
            format!("relative import '{import}' escapes the package bridge source root"),
        ));
    }
    if let Some(call) = collected.dynamic_calls.iter().next() {
        return Err(PackageDiagnostic::invalid_python_bridge_source(
            cargo_package_id,
            source_path,
            format!("dynamic import call '{call}' is not allowed"),
        ));
    }
    if let Some(reserved) = collected.reserved_imports.iter().next() {
        return Err(PackageDiagnostic::invalid_python_bridge_source(
            cargo_package_id,
            source_path,
            format!("reserved runtime namespace import '{reserved}' is not allowed"),
        ));
    }
    let relative_path = source_path
        .strip_prefix(package_root)
        .unwrap_or(source_path);
    Ok(ParsedBridgeModule {
        module,
        source_path: path_string(relative_path),
        source_digest: sha256_hex(source.as_bytes()),
        is_package,
        raw_imports: collected.raw_imports,
    })
}
