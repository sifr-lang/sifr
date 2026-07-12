use crate::graph::derive::{SifrPackageGraph, SifrPackageId};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PythonRequirementKind {
    Manifest,
    Declaration,
    BridgeImport,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PythonRequirementContribution {
    pub root: String,
    pub package_id: SifrPackageId,
    pub kind: PythonRequirementKind,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalPythonRequirement {
    pub root: String,
    pub contributions: Vec<PythonRequirementContribution>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CanonicalPythonRequirements {
    pub roots: Vec<CanonicalPythonRequirement>,
}

impl CanonicalPythonRequirements {
    #[must_use]
    pub fn import_roots(&self) -> Vec<String> {
        self.roots.iter().map(|entry| entry.root.clone()).collect()
    }
}

#[must_use]
pub fn canonical_python_requirements(
    graph: &SifrPackageGraph,
    derived: &[PythonRequirementContribution],
) -> CanonicalPythonRequirements {
    let mut by_root = BTreeMap::<String, Vec<PythonRequirementContribution>>::new();
    for package in graph.packages.values() {
        for root in &package.manifest.python.requires_imports {
            by_root
                .entry(root.clone())
                .or_default()
                .push(PythonRequirementContribution {
                    root: root.clone(),
                    package_id: package.package_id.clone(),
                    kind: PythonRequirementKind::Manifest,
                    source: format!(
                        "{}:[python].requires-imports",
                        package.sifr_manifest.display()
                    ),
                });
        }
    }
    for contribution in derived {
        by_root
            .entry(contribution.root.clone())
            .or_default()
            .push(contribution.clone());
    }
    CanonicalPythonRequirements {
        roots: by_root
            .into_iter()
            .map(|(root, mut contributions)| {
                contributions.sort();
                contributions.dedup();
                CanonicalPythonRequirement {
                    root,
                    contributions,
                }
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::sifr::{PythonConfig, TrustPolicy};
    use crate::python::test_support::{graph, package, package_id};

    #[test]
    fn canonical_requirements_preserve_duplicate_root_provenance() {
        let graph = graph(vec![
            package(
                "app",
                PythonConfig {
                    requires_imports: vec!["numpy".to_string()],
                    ..PythonConfig::default()
                },
                TrustPolicy::default(),
            ),
            package(
                "lib",
                PythonConfig {
                    requires_imports: vec!["numpy".to_string()],
                    ..PythonConfig::default()
                },
                TrustPolicy::default(),
            ),
        ]);
        let derived = [PythonRequirementContribution {
            root: "numpy".to_string(),
            package_id: package_id("lib"),
            kind: PythonRequirementKind::Declaration,
            source: "lib.sifr:4:1".to_string(),
        }];

        let requirements = canonical_python_requirements(&graph, &derived);

        assert_eq!(requirements.import_roots(), ["numpy".to_string()]);
        assert_eq!(requirements.roots[0].contributions.len(), 3);
    }
}
