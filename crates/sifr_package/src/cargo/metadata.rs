use crate::diag::PackageDiagnostic;
use crate::manifest::metadata::CargoSifrMetadata;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CargoPackageId(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CargoDependency {
    pub name: String,
    pub package: Option<String>,
    pub req: String,
    pub kind: Option<String>,
    pub target: Option<String>,
    pub uses_workspace: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CargoResolveEdge {
    pub from: CargoPackageId,
    pub dependency_name: String,
    pub to: CargoPackageId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CargoTarget {
    pub name: String,
    pub kind: BTreeSet<String>,
    pub crate_types: BTreeSet<String>,
    pub src_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CargoPackage {
    pub id: CargoPackageId,
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub links: Option<String>,
    pub manifest_path: PathBuf,
    pub dependencies: Vec<CargoDependency>,
    pub targets: Vec<CargoTarget>,
    pub features: BTreeMap<String, Vec<String>>,
    pub sifr_metadata: Option<CargoSifrMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CargoMetadata {
    pub packages: Vec<CargoPackage>,
    pub resolve_edges: Vec<CargoResolveEdge>,
    pub workspace_members: BTreeSet<CargoPackageId>,
    pub workspace_default_members: BTreeSet<CargoPackageId>,
    pub target_directory: PathBuf,
    pub workspace_root: PathBuf,
    pub workspace_sifr: CargoWorkspaceSifrMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizedCargoMetadata {
    pub packages: BTreeMap<CargoPackageId, CargoPackage>,
    pub resolve_edges: Vec<CargoResolveEdge>,
    pub workspace_members: BTreeSet<CargoPackageId>,
    pub workspace_default_members: BTreeSet<CargoPackageId>,
    pub target_directory: PathBuf,
    pub workspace_root: PathBuf,
    pub workspace_sifr: CargoWorkspaceSifrMetadata,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CargoWorkspaceSifrMetadata {
    pub tools_package: Option<String>,
}

impl CargoMetadata {
    #[must_use]
    pub fn normalize(self) -> NormalizedCargoMetadata {
        let packages = self
            .packages
            .into_iter()
            .map(|mut package| {
                package.dependencies.sort_by(|left, right| {
                    (
                        &left.name,
                        left.package.as_deref().unwrap_or_default(),
                        &left.req,
                        left.kind.as_deref().unwrap_or_default(),
                        left.target.as_deref().unwrap_or_default(),
                    )
                        .cmp(&(
                            &right.name,
                            right.package.as_deref().unwrap_or_default(),
                            &right.req,
                            right.kind.as_deref().unwrap_or_default(),
                            right.target.as_deref().unwrap_or_default(),
                        ))
                });
                package.targets.sort_by(|left, right| {
                    (&left.name, &left.src_path).cmp(&(&right.name, &right.src_path))
                });
                (package.id.clone(), package)
            })
            .collect();
        let mut resolve_edges = self.resolve_edges;
        resolve_edges.sort_by(|left, right| {
            (&left.from, &left.dependency_name, &left.to).cmp(&(
                &right.from,
                &right.dependency_name,
                &right.to,
            ))
        });

        NormalizedCargoMetadata {
            packages,
            resolve_edges,
            workspace_members: self.workspace_members,
            workspace_default_members: self.workspace_default_members,
            target_directory: self.target_directory,
            workspace_root: self.workspace_root,
            workspace_sifr: self.workspace_sifr,
        }
    }
}

pub fn parse_metadata_json(source: &str) -> Result<CargoMetadata, PackageDiagnostic> {
    let raw: RawCargoMetadata = serde_json::from_str(source)
        .map_err(|error| PackageDiagnostic::cargo_metadata_parse(&error.to_string()))?;

    raw.try_into()
}

#[derive(Debug, Deserialize)]
struct RawCargoMetadata {
    packages: Vec<RawCargoPackage>,
    resolve: Option<RawCargoResolve>,
    workspace_members: Vec<String>,
    #[serde(default)]
    workspace_default_members: Vec<String>,
    target_directory: PathBuf,
    workspace_root: PathBuf,
    #[serde(default, rename = "metadata")]
    workspace_metadata: Value,
}

#[derive(Debug, Deserialize)]
struct RawCargoPackage {
    id: String,
    name: String,
    version: String,
    source: Option<String>,
    links: Option<String>,
    manifest_path: PathBuf,
    dependencies: Vec<RawCargoDependency>,
    targets: Vec<RawCargoTarget>,
    features: BTreeMap<String, Vec<String>>,
    metadata: Value,
}

#[derive(Debug, Deserialize)]
struct RawCargoDependency {
    name: String,
    package: Option<String>,
    req: String,
    kind: Option<String>,
    target: Option<String>,
    #[serde(default)]
    uses_workspace: bool,
}

#[derive(Debug, Deserialize)]
struct RawCargoResolve {
    nodes: Vec<RawCargoResolveNode>,
}

#[derive(Debug, Deserialize)]
struct RawCargoResolveNode {
    id: String,
    #[serde(default)]
    deps: Vec<RawCargoResolveDep>,
}

#[derive(Debug, Deserialize)]
struct RawCargoResolveDep {
    name: String,
    pkg: String,
}

#[derive(Debug, Deserialize)]
struct RawCargoTarget {
    name: String,
    kind: Vec<String>,
    crate_types: Vec<String>,
    src_path: PathBuf,
}

impl TryFrom<RawCargoMetadata> for CargoMetadata {
    type Error = PackageDiagnostic;

    fn try_from(raw: RawCargoMetadata) -> Result<Self, Self::Error> {
        let packages = raw
            .packages
            .into_iter()
            .map(CargoPackage::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let resolve_edges = raw
            .resolve
            .map(|resolve| {
                resolve
                    .nodes
                    .into_iter()
                    .flat_map(|node| {
                        let from = CargoPackageId(node.id);
                        node.deps
                            .into_iter()
                            .map(move |dependency| CargoResolveEdge {
                                from: from.clone(),
                                dependency_name: dependency.name,
                                to: CargoPackageId(dependency.pkg),
                            })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let workspace_members = raw
            .workspace_members
            .into_iter()
            .map(CargoPackageId)
            .collect();
        let workspace_default_members = raw
            .workspace_default_members
            .into_iter()
            .map(CargoPackageId)
            .collect();

        Ok(Self {
            packages,
            resolve_edges,
            workspace_members,
            workspace_default_members,
            target_directory: raw.target_directory,
            workspace_root: raw.workspace_root,
            workspace_sifr: parse_workspace_sifr_metadata(&raw.workspace_metadata)?,
        })
    }
}

fn parse_workspace_sifr_metadata(
    metadata: &Value,
) -> Result<CargoWorkspaceSifrMetadata, PackageDiagnostic> {
    let Some(sifr) = metadata.get("sifr") else {
        return Ok(CargoWorkspaceSifrMetadata::default());
    };
    let Some(table) = sifr.as_object() else {
        return Err(PackageDiagnostic::cargo_metadata_parse(
            "[workspace.metadata.sifr] must be a table",
        ));
    };
    for key in table.keys() {
        if key != "tools-package" {
            return Err(PackageDiagnostic::cargo_metadata_parse(&format!(
                "unsupported [workspace.metadata.sifr] key '{key}'"
            )));
        }
    }
    let tools_package = table
        .get("tools-package")
        .map(|value| {
            value
                .as_str()
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .ok_or_else(|| {
                    PackageDiagnostic::cargo_metadata_parse(
                        "[workspace.metadata.sifr].tools-package must be a non-empty string",
                    )
                })
        })
        .transpose()?;
    Ok(CargoWorkspaceSifrMetadata { tools_package })
}

impl TryFrom<RawCargoPackage> for CargoPackage {
    type Error = PackageDiagnostic;

    fn try_from(raw: RawCargoPackage) -> Result<Self, Self::Error> {
        let sifr_metadata = CargoSifrMetadata::from_cargo_metadata_value(
            &CargoPackageId(raw.id.clone()),
            &raw.name,
            &raw.metadata,
        )?;
        let dependencies = raw.dependencies.into_iter().map(Into::into).collect();
        let targets = raw.targets.into_iter().map(Into::into).collect();

        Ok(Self {
            id: CargoPackageId(raw.id),
            name: raw.name,
            version: raw.version,
            source: raw.source,
            links: raw.links,
            manifest_path: raw.manifest_path,
            dependencies,
            targets,
            features: raw.features,
            sifr_metadata,
        })
    }
}

impl From<RawCargoDependency> for CargoDependency {
    fn from(raw: RawCargoDependency) -> Self {
        Self {
            name: raw.name,
            package: raw.package,
            req: raw.req,
            kind: raw.kind,
            target: raw.target,
            uses_workspace: raw.uses_workspace,
        }
    }
}

impl From<RawCargoTarget> for CargoTarget {
    fn from(raw: RawCargoTarget) -> Self {
        Self {
            name: raw.name,
            kind: raw.kind.into_iter().collect(),
            crate_types: raw.crate_types.into_iter().collect(),
            src_path: raw.src_path,
        }
    }
}
