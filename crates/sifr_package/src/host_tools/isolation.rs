//! Keep host-only tool dependency closures outside application graphs.

use super::declarations::parse_tool_declarations;
use super::{host_tool_diagnostic, package_root};
use crate::{
    CargoPackage, CargoPackageId, NormalizedCargoMetadata, PackageClassification,
    PackageDiagnostic, PackageGraphSnapshot,
};
use sifr_frontend::SourceProvider;
use std::collections::{BTreeSet, VecDeque};

pub fn validate_host_tool_application_isolation(
    metadata: &NormalizedCargoMetadata,
    provider: &mut impl SourceProvider,
) -> Result<(), Vec<PackageDiagnostic>> {
    let Some(tools_package_name) = metadata.workspace_sifr.tools_package.as_deref() else {
        return Ok(());
    };
    let tools_matches = metadata
        .workspace_members
        .iter()
        .filter_map(|id| metadata.packages.get(id))
        .filter(|package| package.name == tools_package_name)
        .collect::<Vec<_>>();
    if tools_matches.len() != 1 {
        return Err(vec![host_tool_diagnostic(format!(
            "tools package '{tools_package_name}' must name exactly one workspace member; found {}",
            tools_matches.len()
        ))]);
    }
    let tools_package = tools_matches[0];
    let Some(discovery) = tools_package.sifr_metadata.as_ref() else {
        return Err(vec![host_tool_diagnostic(format!(
            "tools package '{}' requires [package.metadata.sifr].manifest",
            tools_package.name
        ))]);
    };
    let manifest_path = package_root(tools_package).join(&discovery.manifest);
    let source = provider.read_file(&manifest_path).map_err(|error| {
        vec![host_tool_diagnostic(format!(
            "cannot read tools manifest '{}': {error}",
            manifest_path.display()
        ))]
    })?;
    let declarations = parse_tool_declarations(&manifest_path, source.as_str())?;
    let direct = direct_normal_dependencies(metadata, tools_package);
    let mut diagnostics = Vec::new();
    let mut tool_roots = BTreeSet::from([tools_package.id.clone()]);
    for (namespace, declaration) in declarations {
        let matches = direct
            .iter()
            .filter_map(|id| metadata.packages.get(*id))
            .filter(|package| package.name == declaration.package)
            .filter(|package| metadata.workspace_members.contains(&package.id))
            .collect::<Vec<_>>();
        if matches.len() == 1 {
            tool_roots.insert(matches[0].id.clone());
        } else {
            diagnostics.push(host_tool_diagnostic(format!(
                "tool namespace '{namespace}' requires one exact normal workspace-member package '{}'; found {}",
                declaration.package,
                matches.len()
            )));
        }
    }
    for application in metadata.workspace_members.iter().filter(|id| {
        !tool_roots.contains(*id)
            && metadata
                .packages
                .get(*id)
                .is_some_and(|package| package.sifr_metadata.is_some())
    }) {
        let closure = dependency_closure(metadata, application);
        for contaminated in closure.intersection(&tool_roots) {
            diagnostics.push(host_tool_diagnostic(format!(
                "application package '{}' reaches host-only tool package '{}'",
                application.0, contaminated.0
            )));
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

pub(super) fn direct_normal_dependencies<'a>(
    metadata: &'a NormalizedCargoMetadata,
    from: &CargoPackage,
) -> Vec<&'a CargoPackageId> {
    metadata
        .resolve_edges
        .iter()
        .filter(|edge| edge.from == from.id)
        .filter(|edge| {
            let Some(target) = metadata.packages.get(&edge.to) else {
                return false;
            };
            from.dependencies.iter().any(|dependency| {
                dependency.kind.is_none()
                    && dependency
                        .package
                        .as_deref()
                        .unwrap_or(dependency.name.as_str())
                        == target.name
            })
        })
        .map(|edge| &edge.to)
        .collect()
}

pub(super) fn target_contamination_diagnostics<'a>(
    metadata: &NormalizedCargoMetadata,
    snapshot: &PackageGraphSnapshot,
    tools_package: &CargoPackageId,
    entry_packages: impl Iterator<Item = &'a CargoPackageId>,
) -> Vec<PackageDiagnostic> {
    let tool_roots =
        BTreeSet::from_iter(std::iter::once(tools_package.clone()).chain(entry_packages.cloned()));
    let application_roots = snapshot
        .graph
        .classifications
        .iter()
        .filter_map(|(id, classification)| {
            matches!(
                classification,
                PackageClassification::SifrSource(_) | PackageClassification::RustBackedSifr(_)
            )
            .then_some(id.clone())
        })
        .collect::<Vec<_>>();
    let mut diagnostics = Vec::new();
    for application in application_roots {
        let closure = dependency_closure(metadata, &application);
        for contaminated in closure.intersection(&tool_roots) {
            diagnostics.push(host_tool_diagnostic(format!(
                "application package '{}' reaches host-only tool package '{}'",
                application.0, contaminated.0
            )));
        }
    }
    diagnostics
}

fn dependency_closure(
    metadata: &NormalizedCargoMetadata,
    root: &CargoPackageId,
) -> BTreeSet<CargoPackageId> {
    let mut closure = BTreeSet::new();
    let mut pending = VecDeque::from([root.clone()]);
    while let Some(current) = pending.pop_front() {
        if !closure.insert(current.clone()) {
            continue;
        }
        pending.extend(
            metadata
                .resolve_edges
                .iter()
                .filter(|edge| edge.from == current)
                .map(|edge| edge.to.clone()),
        );
    }
    closure
}
