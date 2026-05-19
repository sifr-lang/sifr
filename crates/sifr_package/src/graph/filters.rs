use crate::diag::PackageDiagnostic;
use crate::graph::derive::{SifrPackageGraph, SifrPackageId};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageFilter {
    pub raw: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageFilterTerm {
    Package(String),
    DependencyClosure(String),
    DependentClosure(String),
    DependentsOnly(String),
    Negated(Box<PackageFilterTerm>),
}

pub fn parse_package_filter(raw: &str) -> Result<PackageFilterTerm, PackageDiagnostic> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(PackageDiagnostic::selector_ambiguous(raw, &[]));
    }
    if let Some(rest) = raw.strip_prefix('!') {
        return parse_package_filter(rest).map(|term| PackageFilterTerm::Negated(Box::new(term)));
    }
    if let Some(package) = raw.strip_prefix("...^") {
        return Ok(PackageFilterTerm::DependentsOnly(package.to_string()));
    }
    if let Some(package) = raw.strip_prefix("...") {
        return Ok(PackageFilterTerm::DependentClosure(package.to_string()));
    }
    if let Some(package) = raw.strip_suffix("...") {
        return Ok(PackageFilterTerm::DependencyClosure(package.to_string()));
    }
    Ok(PackageFilterTerm::Package(raw.to_string()))
}

pub fn apply_package_filters(
    graph: &SifrPackageGraph,
    filters: &[PackageFilterTerm],
) -> Result<BTreeSet<SifrPackageId>, Vec<PackageDiagnostic>> {
    let mut selected = BTreeSet::new();
    let mut diagnostics = Vec::new();

    for filter in filters {
        match apply_one_filter(graph, filter) {
            Ok((matches, negated)) => {
                if negated {
                    for package_id in matches {
                        selected.remove(&package_id);
                    }
                } else {
                    selected.extend(matches);
                }
            }
            Err(error) => diagnostics.push(error),
        }
    }

    if diagnostics.is_empty() {
        Ok(selected)
    } else {
        Err(diagnostics)
    }
}

fn apply_one_filter(
    graph: &SifrPackageGraph,
    filter: &PackageFilterTerm,
) -> Result<(BTreeSet<SifrPackageId>, bool), PackageDiagnostic> {
    match filter {
        PackageFilterTerm::Package(name) => resolve_package_name(graph, name).map(|id| {
            let mut set = BTreeSet::new();
            set.insert(id);
            (set, false)
        }),
        PackageFilterTerm::DependencyClosure(name) => {
            let root = resolve_package_name(graph, name)?;
            let mut set = BTreeSet::new();
            collect_dependency_closure(graph, &root, &mut set);
            Ok((set, false))
        }
        PackageFilterTerm::DependentClosure(name) => {
            let root = resolve_package_name(graph, name)?;
            let mut set = BTreeSet::new();
            collect_dependent_closure(&root, &reverse_edges(graph), &mut set);
            Ok((set, false))
        }
        PackageFilterTerm::DependentsOnly(name) => {
            let root = resolve_package_name(graph, name)?;
            let mut set = BTreeSet::new();
            collect_dependent_closure(&root, &reverse_edges(graph), &mut set);
            set.remove(&root);
            Ok((set, false))
        }
        PackageFilterTerm::Negated(term) => {
            let (matches, _) = apply_one_filter(graph, term)?;
            Ok((matches, true))
        }
    }
}

fn resolve_package_name(
    graph: &SifrPackageGraph,
    name: &str,
) -> Result<SifrPackageId, PackageDiagnostic> {
    let matches = graph
        .packages
        .values()
        .filter(|package| {
            package.cargo_package_name == name
                || package.sifr_name.0 == name
                || package.package_id.0 == name
        })
        .map(|package| package.package_id.clone())
        .collect::<Vec<_>>();

    if let [package_id] = matches.as_slice() {
        Ok(package_id.clone())
    } else {
        let candidates = matches
            .iter()
            .map(|package_id| package_id.0.clone())
            .collect::<Vec<_>>();
        Err(PackageDiagnostic::selector_ambiguous(name, &candidates))
    }
}

fn collect_dependency_closure(
    graph: &SifrPackageGraph,
    package_id: &SifrPackageId,
    selected: &mut BTreeSet<SifrPackageId>,
) {
    if !selected.insert(package_id.clone()) {
        return;
    }
    if let Some(dependencies) = graph.cargo_edges.get(package_id) {
        for dependency in dependencies {
            collect_dependency_closure(graph, dependency, selected);
        }
    }
}

fn collect_dependent_closure(
    package_id: &SifrPackageId,
    reverse: &BTreeMap<SifrPackageId, BTreeSet<SifrPackageId>>,
    selected: &mut BTreeSet<SifrPackageId>,
) {
    if !selected.insert(package_id.clone()) {
        return;
    }
    if let Some(dependents) = reverse.get(package_id) {
        for dependent in dependents {
            collect_dependent_closure(dependent, reverse, selected);
        }
    }
}

fn reverse_edges(graph: &SifrPackageGraph) -> BTreeMap<SifrPackageId, BTreeSet<SifrPackageId>> {
    let mut reverse = BTreeMap::new();
    for (from, dependencies) in &graph.cargo_edges {
        for dependency in dependencies {
            reverse
                .entry(dependency.clone())
                .or_insert_with(BTreeSet::new)
                .insert(from.clone());
        }
    }
    reverse
}
