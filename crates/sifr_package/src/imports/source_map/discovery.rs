use super::{DottedModulePath, PackageModuleSource};
use crate::diag::PackageDiagnostic;
use crate::graph::derive::SifrPackageMetadata;
use crate::imports::namespace_api::{parse_init_sifr_reexports, NamespaceApi};
use crate::manifest::sifr::ImportRoot;
use sifr_frontend::SourceProvider;
use std::path::{Path, PathBuf};

pub(super) fn package_source_roots(package: &SifrPackageMetadata) -> Vec<PathBuf> {
    package
        .manifest
        .source_roots
        .iter()
        .map(|root| package.package_root.join(&root.0))
        .collect()
}

pub(super) fn root_for(package: &SifrPackageMetadata) -> ImportRoot {
    package
        .manifest
        .exports
        .first()
        .cloned()
        .unwrap_or_else(|| ImportRoot(package.sifr_name.0.clone()))
}

pub(super) fn discover_package_modules(
    package: &SifrPackageMetadata,
    source_root: &Path,
    provider: &mut impl SourceProvider,
) -> Result<Vec<PackageModuleSource>, String> {
    let mut modules = Vec::new();
    discover_modules_recursive(package, source_root, source_root, &mut modules, provider)?;
    modules.sort_by(|left, right| left.module_path.cmp(&right.module_path));
    Ok(modules)
}

fn discover_modules_recursive(
    package: &SifrPackageMetadata,
    source_root: &Path,
    directory: &Path,
    modules: &mut Vec<PackageModuleSource>,
    provider: &mut impl SourceProvider,
) -> Result<(), String> {
    let entries = provider.read_dir(directory).map_err(|error| {
        format!(
            "could not read source root '{}': {error}",
            directory.display()
        )
    })?;
    for entry in entries {
        let path = entry.path;
        if entry.is_dir {
            discover_modules_recursive(package, source_root, &path, modules, provider)?;
            continue;
        }
        if !entry.is_file || path.extension().and_then(|value| value.to_str()) != Some("sifr") {
            continue;
        }
        let Some(module_path) = module_path_from_file(package, source_root, &path) else {
            continue;
        };
        modules.push(PackageModuleSource {
            package_id: package.package_id.clone(),
            cargo_package_id: package.cargo_package_id.clone(),
            module_path,
            file_path: path,
            source_root: source_root.to_path_buf(),
        });
    }
    Ok(())
}

pub(super) fn discover_namespace_apis(
    package: &SifrPackageMetadata,
    source_root: &Path,
    provider: &mut impl SourceProvider,
) -> Result<Vec<NamespaceApi>, Vec<PackageDiagnostic>> {
    let modules = discover_package_modules(package, source_root, provider).map_err(|error| {
        vec![PackageDiagnostic::invalid_sifr_manifest(
            &package.cargo_package_id,
            package.sifr_manifest.clone(),
            if package.manifest.production_schema {
                "source.root"
            } else {
                "source.roots"
            },
            error,
        )]
    })?;
    let mut apis = Vec::new();
    let mut diagnostics = Vec::new();
    for module in modules {
        if module.file_path.file_name().and_then(|name| name.to_str()) != Some("__init__.sifr") {
            continue;
        }
        match parse_init_sifr_reexports(
            &package.cargo_package_id,
            &package.sifr_manifest,
            &module.module_path,
            &module.file_path,
            source_root,
            provider,
        ) {
            Ok(api) => apis.push(api),
            Err(errors) => diagnostics.extend(errors),
        }
    }
    if diagnostics.is_empty() {
        Ok(apis)
    } else {
        Err(diagnostics)
    }
}

fn module_path_from_file(
    package: &SifrPackageMetadata,
    source_root: &Path,
    file_path: &Path,
) -> Option<DottedModulePath> {
    let relative = file_path.strip_prefix(source_root).ok()?;
    let mut parts = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let last = parts.last_mut()?;
    if last == "__init__.sifr" {
        let _ = parts.pop();
    } else if let Some(stripped) = last.strip_suffix(".sifr") {
        *last = stripped.to_string();
    }
    if !parts.iter().all(|part| valid_identifier(part)) {
        return None;
    }
    if package.manifest.production_schema {
        parts.insert(0, package.sifr_name.0.clone());
    }
    if parts.is_empty() {
        return None;
    }
    Some(DottedModulePath(parts.join(".")))
}

fn valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}
