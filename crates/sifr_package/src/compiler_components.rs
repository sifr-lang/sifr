use crate::{SifrPackageGraph, SifrPackageId};
use sifr_compiler_component::{
    ComponentError, ComponentErrorKind, ComponentRegistration, ComponentRequirement,
    ResolvedComponent, resolve_component,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageCompilerComponent {
    pub package_id: SifrPackageId,
    pub package_root: PathBuf,
    pub artifact_path: PathBuf,
    pub registration: ComponentRegistration,
}

pub fn compiler_component_registrations(
    graph: &SifrPackageGraph,
) -> Result<BTreeMap<String, PackageCompilerComponent>, ComponentError> {
    let mut registrations = BTreeMap::new();
    for package in graph.packages.values() {
        for component in package.manifest.compiler_components.values() {
            for registration in component.registrations(&package.package_id.0) {
                let processor = registration.identity.processor.clone();
                let resolved = PackageCompilerComponent {
                    package_id: package.package_id.clone(),
                    package_root: package.package_root.clone(),
                    artifact_path: package.package_root.join(&component.artifact),
                    registration,
                };
                if registrations.insert(processor.clone(), resolved).is_some() {
                    return Err(ComponentError::new(
                        ComponentErrorKind::Registration,
                        format!(
                            "processor identity '{processor}' is registered by more than one package"
                        ),
                    ));
                }
            }
        }
    }
    Ok(registrations)
}

pub fn resolve_package_component(
    graph: &SifrPackageGraph,
    requirement: &ComponentRequirement,
) -> Result<ResolvedComponent, ComponentError> {
    let registrations = compiler_component_registrations(graph)?;
    let package_component = registrations
        .get(&requirement.identity.processor)
        .ok_or_else(|| {
            ComponentError::new(
                ComponentErrorKind::Registration,
                format!(
                    "processor identity '{}' is not registered",
                    requirement.identity.processor
                ),
            )
        })?;
    let package_root = package_component
        .package_root
        .canonicalize()
        .map_err(|error| {
            ComponentError::new(
                ComponentErrorKind::Integrity,
                format!("cannot resolve component package root: {error}"),
            )
        })?;
    let artifact_path = package_component
        .artifact_path
        .canonicalize()
        .map_err(|error| {
            ComponentError::new(
                ComponentErrorKind::Integrity,
                format!("cannot resolve component artifact: {error}"),
            )
        })?;
    if !artifact_path.starts_with(&package_root) {
        return Err(ComponentError::new(
            ComponentErrorKind::Integrity,
            "component artifact resolves outside its package root",
        ));
    }
    let bytes = std::fs::read(&artifact_path).map_err(|error| {
        ComponentError::new(
            ComponentErrorKind::Integrity,
            format!("cannot read component artifact: {error}"),
        )
    })?;
    resolve_component(
        requirement,
        [ResolvedComponent {
            registration: package_component.registration.clone(),
            bytes,
        }],
    )
}
