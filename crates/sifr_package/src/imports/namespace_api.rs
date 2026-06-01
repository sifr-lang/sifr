use crate::diag::PackageDiagnostic;
use crate::imports::source_map::DottedModulePath;
use crate::CargoPackageId;
use sifr_frontend::SourceProvider;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NamespaceApi {
    pub namespace: DottedModulePath,
    pub public_symbols: BTreeMap<String, PublicSymbolOrigin>,
    pub public_child_namespaces: BTreeMap<String, DottedModulePath>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PublicSymbolOrigin {
    DirectDefinition {
        file_path: PathBuf,
    },
    ReExport {
        module_path: DottedModulePath,
        symbol: String,
    },
}

pub fn parse_init_sifr_reexports(
    cargo_package_id: &CargoPackageId,
    sifr_manifest: &Path,
    namespace_path: &DottedModulePath,
    init_path: &Path,
    package_source_root: &Path,
    provider: &mut impl SourceProvider,
) -> Result<NamespaceApi, Vec<PackageDiagnostic>> {
    let source = match provider.read_file(init_path) {
        Ok(source) => source,
        Err(error) => {
            return Err(vec![PackageDiagnostic::invalid_sifr_manifest(
                cargo_package_id,
                sifr_manifest.to_path_buf(),
                "__init__.sifr",
                format!("could not read '{}': {error}", init_path.display()),
            )]);
        }
    };
    let mut api = NamespaceApi {
        namespace: namespace_path.clone(),
        ..NamespaceApi::default()
    };
    let mut diagnostics = Vec::new();

    for raw_line in source.as_str().lines() {
        if raw_line.starts_with(char::is_whitespace) {
            continue;
        }
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.contains("__all__") || line.contains(" import *") {
            diagnostics.push(unsupported_api_form(
                cargo_package_id,
                sifr_manifest,
                "__init__.sifr uses dynamic or wildcard public API construction",
            ));
            continue;
        }
        if let Some(rest) = line.strip_prefix("from .") {
            if let Err(diagnostic) = parse_relative_from(
                cargo_package_id,
                sifr_manifest,
                namespace_path,
                package_source_root,
                provider,
                &mut api,
                rest,
            ) {
                diagnostics.push(diagnostic);
            }
            continue;
        }
        if let Some(name) = top_level_definition_name(line) {
            insert_symbol(
                cargo_package_id,
                sifr_manifest,
                &mut api,
                name.to_string(),
                PublicSymbolOrigin::DirectDefinition {
                    file_path: init_path.to_path_buf(),
                },
                &mut diagnostics,
            );
            continue;
        }
        if looks_like_public_assignment(line) {
            diagnostics.push(unsupported_api_form(
                cargo_package_id,
                sifr_manifest,
                "__init__.sifr assignment-based exports are not supported",
            ));
        }
    }

    if diagnostics.is_empty() {
        Ok(api)
    } else {
        Err(diagnostics)
    }
}

fn parse_relative_from(
    cargo_package_id: &CargoPackageId,
    sifr_manifest: &Path,
    namespace_path: &DottedModulePath,
    package_source_root: &Path,
    provider: &mut impl SourceProvider,
    api: &mut NamespaceApi,
    rest: &str,
) -> Result<(), PackageDiagnostic> {
    let Some((module, imported)) = rest.split_once(" import ") else {
        return Err(unsupported_api_form(
            cargo_package_id,
            sifr_manifest,
            "__init__.sifr relative export is malformed",
        ));
    };
    if module.is_empty() {
        parse_child_namespace_exports(
            cargo_package_id,
            sifr_manifest,
            namespace_path,
            package_source_root,
            provider,
            api,
            imported,
        )
    } else {
        parse_symbol_reexports(
            cargo_package_id,
            sifr_manifest,
            namespace_path,
            api,
            module,
            imported,
        )
    }
}

fn parse_child_namespace_exports(
    cargo_package_id: &CargoPackageId,
    sifr_manifest: &Path,
    namespace_path: &DottedModulePath,
    package_source_root: &Path,
    provider: &mut impl SourceProvider,
    api: &mut NamespaceApi,
    imported: &str,
) -> Result<(), PackageDiagnostic> {
    for child in imported
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
    {
        let child_name = child
            .split_once(" as ")
            .map_or(child, |(_, alias)| alias)
            .trim();
        if !valid_identifier(child_name)
            || !public_child_namespace_exists(package_source_root, child_name, provider)
        {
            return Err(unsupported_api_form(
                cargo_package_id,
                sifr_manifest,
                "from . import child must name a public child namespace",
            ));
        }
        api.public_child_namespaces
            .insert(child_name.to_string(), namespace_path.join(child_name));
    }
    Ok(())
}

fn parse_symbol_reexports(
    cargo_package_id: &CargoPackageId,
    sifr_manifest: &Path,
    namespace_path: &DottedModulePath,
    api: &mut NamespaceApi,
    module: &str,
    imported: &str,
) -> Result<(), PackageDiagnostic> {
    if !module.split('.').all(valid_identifier) {
        return Err(unsupported_api_form(
            cargo_package_id,
            sifr_manifest,
            "__init__.sifr relative export module path is invalid",
        ));
    }
    let module_path = namespace_path.join(module);
    let mut diagnostics = Vec::new();
    for item in imported
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
    {
        let (symbol, public_name) = item
            .split_once(" as ")
            .map_or((item, item), |(symbol, alias)| {
                (symbol.trim(), alias.trim())
            });
        if !valid_identifier(symbol) || !valid_identifier(public_name) {
            return Err(unsupported_api_form(
                cargo_package_id,
                sifr_manifest,
                "__init__.sifr relative export name is invalid",
            ));
        }
        insert_symbol(
            cargo_package_id,
            sifr_manifest,
            api,
            public_name.to_string(),
            PublicSymbolOrigin::ReExport {
                module_path: module_path.clone(),
                symbol: symbol.to_string(),
            },
            &mut diagnostics,
        );
    }
    diagnostics.into_iter().next().map_or(Ok(()), Err)
}

fn insert_symbol(
    cargo_package_id: &CargoPackageId,
    sifr_manifest: &Path,
    api: &mut NamespaceApi,
    name: String,
    origin: PublicSymbolOrigin,
    diagnostics: &mut Vec<PackageDiagnostic>,
) {
    if name.starts_with('_') {
        return;
    }
    if let Some(existing) = api.public_symbols.get(&name) {
        if existing != &origin {
            diagnostics.push(PackageDiagnostic::duplicate_public_api_symbol(
                cargo_package_id,
                sifr_manifest,
                name,
            ));
        }
        return;
    }
    api.public_symbols.insert(name, origin);
}

fn top_level_definition_name(line: &str) -> Option<&str> {
    let rest = line
        .strip_prefix("class ")
        .or_else(|| line.strip_prefix("def "))
        .or_else(|| line.strip_prefix("type "))?;
    let name_end = rest
        .find(|ch: char| !(ch == '_' || ch.is_ascii_alphanumeric()))
        .unwrap_or(rest.len());
    let name = &rest[..name_end];
    valid_identifier(name).then_some(name)
}

fn public_child_namespace_exists(
    package_source_root: &Path,
    child: &str,
    provider: &mut impl SourceProvider,
) -> bool {
    provider.is_file(&package_source_root.join(child).join("__init__.sifr"))
}

fn looks_like_public_assignment(line: &str) -> bool {
    let Some((left, _)) = line.split_once('=') else {
        return false;
    };
    let name = left.trim();
    valid_identifier(name) && !name.starts_with('_')
}

fn unsupported_api_form(
    cargo_package_id: &CargoPackageId,
    sifr_manifest: &Path,
    reason: impl Into<String>,
) -> PackageDiagnostic {
    PackageDiagnostic::invalid_sifr_manifest(
        cargo_package_id,
        sifr_manifest.to_path_buf(),
        "__init__.sifr",
        reason,
    )
}

fn valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

impl DottedModulePath {
    #[must_use]
    pub fn join(&self, child: &str) -> Self {
        if self.0.is_empty() {
            Self(child.to_string())
        } else {
            Self(format!("{}.{}", self.0, child))
        }
    }
}
