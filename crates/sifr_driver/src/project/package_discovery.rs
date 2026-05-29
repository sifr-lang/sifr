use super::discovery::{
    diagnostic_with_source_range, discovery_label, DiscoveryDiagnosticStyle, ParsedProjectModule,
};
use crate::diagnostics::RenderedDiagnostic;
use ruff_text_size::{Ranged as _, TextRange};
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode};
use sifr_package::{
    DottedModulePath, PackageImportOrigin, PackageSourceMap, SifrPackageGraph, SifrPackageId,
};
use sifr_python_ast::{Identifier, Stmt};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
struct PackageImportDependency {
    written_module_name: String,
    written_range: TextRange,
    resolved_import_path: DottedModulePath,
}

pub(crate) fn parse_package_import_closure_source_modules(
    graph: &SifrPackageGraph,
    source_map: &PackageSourceMap,
    entry_package_id: &SifrPackageId,
    entry_file: &Path,
    diagnostic_style: DiscoveryDiagnosticStyle,
) -> Result<(String, HashMap<String, ParsedProjectModule>), Vec<RenderedDiagnostic>> {
    let Some(entry_module) = source_map.module_for_file(entry_package_id, entry_file) else {
        return Err(vec![crate::diagnostics::diagnostic_with_code(
            format!(
                "package entrypoint '{}' is not part of the selected package source map",
                entry_file.display()
            ),
            DiagnosticCode::PACKAGE_UNDECLARED_DIRECT_IMPORT,
        )]);
    };
    let entry_module_name = entry_module.module_path.0.clone();
    let mut parsed_modules: HashMap<String, ParsedProjectModule> = HashMap::new();
    let mut parsed_names: BTreeSet<PackageDiscoveryItem> = BTreeSet::new();
    let mut pending = BTreeSet::from([PackageDiscoveryItem {
        package_id: entry_package_id.clone(),
        source_module_path: DottedModulePath(entry_module_name.clone()),
        compile_module_name: entry_module_name.clone(),
    }]);

    while let Some(item) = pending.pop_first() {
        if !parsed_names.insert(item.clone()) {
            continue;
        }
        let resolved = source_map
            .resolve_import(graph, &item.package_id, &item.source_module_path)
            .map_err(|error| vec![package_import_diagnostic(error)])?;
        let source = std::fs::read_to_string(&resolved.resolved_module.file_path).map_err(|e| {
            vec![crate::diagnostics::diagnostic_with_code(
                format!(
                    "failed to read '{}': {}",
                    resolved.resolved_module.file_path.display(),
                    e
                ),
                DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
            )]
        })?;
        let label = discovery_label(
            &item.compile_module_name,
            &resolved.resolved_module.file_path,
            diagnostic_style,
        );
        let mut suite = sifr_frontend::parse_source(&source, Some(&label))?;
        let is_namespace_module = resolved
            .resolved_module
            .file_path
            .file_name()
            .and_then(|name| name.to_str())
            == Some("__init__.sifr");
        for dependency in
            package_import_dependencies(&item.source_module_path.0, is_namespace_module, &suite)
                .into_values()
        {
            let dependency_resolution = source_map
                .resolve_import(
                    graph,
                    &resolved.resolved_module.package_id,
                    &dependency.resolved_import_path,
                )
                .map_err(|error| {
                    vec![package_import_source_diagnostic(
                        &error,
                        &dependency,
                        &resolved.resolved_module.file_path.display().to_string(),
                        &source,
                        &resolved.resolved_module.package_id,
                        graph,
                    )]
                })?;
            let compile_module_name = compile_module_name_for_dependency(
                entry_package_id,
                &item,
                &dependency_resolution.origin,
                &dependency_resolution.import_path,
                &dependency_resolution.resolved_module.module_path,
            );
            rewrite_import_from_dependency(
                &mut suite,
                &item.source_module_path.0,
                is_namespace_module,
                &dependency.resolved_import_path,
                &compile_module_name,
            );
            let dependency_item = PackageDiscoveryItem {
                package_id: dependency_resolution.resolved_module.package_id,
                source_module_path: dependency_resolution.resolved_module.module_path,
                compile_module_name,
            };
            if !parsed_names.contains(&dependency_item) {
                pending.insert(dependency_item);
            }
        }
        parsed_modules.insert(
            item.compile_module_name,
            ParsedProjectModule {
                suite,
                source,
                display_path: resolved.resolved_module.file_path.display().to_string(),
            },
        );
    }

    Ok((entry_module_name, parsed_modules))
}

fn package_import_source_diagnostic(
    error: &sifr_package::PackageDiagnostic,
    dependency: &PackageImportDependency,
    display_path: &str,
    source: &str,
    package_id: &SifrPackageId,
    graph: &SifrPackageGraph,
) -> RenderedDiagnostic {
    if error.code != DiagnosticCode::PACKAGE_UNDECLARED_DIRECT_IMPORT {
        return package_import_diagnostic(error.clone());
    }
    if package_import_targets_known_external_package(
        &dependency.resolved_import_path,
        package_id,
        graph,
    ) {
        return package_import_diagnostic(error.clone());
    }
    let resolved_path = dependency.resolved_import_path.0.clone();
    let args = [
        (
            "module",
            DiagnosticArg::String(dependency.written_module_name.clone()),
        ),
        (
            "resolution_scope",
            DiagnosticArg::String("package".to_string()),
        ),
        ("tried_paths", DiagnosticArg::String(resolved_path.clone())),
        (
            "written_module_path",
            DiagnosticArg::String(dependency.written_module_name.clone()),
        ),
        (
            "resolved_package_import_path",
            DiagnosticArg::String(resolved_path.clone()),
        ),
        (
            "package_import_origin",
            DiagnosticArg::String(format!("own_package:{}", package_id.0)),
        ),
    ];
    let notes = vec![
        format!("package import path: {resolved_path}"),
        format!("package origin: own package {}", package_id.0),
    ];
    diagnostic_with_source_range(
        DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE,
        display_path,
        source,
        dependency.written_range,
        "unknown import target: '{module}'",
        &args,
        &notes,
    )
}

fn package_import_targets_known_external_package(
    import_path: &DottedModulePath,
    importing_package_id: &SifrPackageId,
    graph: &SifrPackageGraph,
) -> bool {
    graph.packages.values().any(|package| {
        &package.package_id != importing_package_id
            && package.manifest.exports.iter().any(|export| {
                import_path.0 == export.0
                    || import_path
                        .0
                        .strip_prefix(&format!("{}.", export.0))
                        .is_some()
            })
    })
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PackageDiscoveryItem {
    package_id: SifrPackageId,
    source_module_path: DottedModulePath,
    compile_module_name: String,
}

fn compile_module_name_for_dependency(
    entry_package_id: &SifrPackageId,
    current: &PackageDiscoveryItem,
    origin: &PackageImportOrigin,
    import_path: &DottedModulePath,
    resolved_module_path: &DottedModulePath,
) -> String {
    match origin {
        PackageImportOrigin::DirectDependency {
            dependency_package_id,
            ..
        } => {
            if &current.package_id == entry_package_id {
                import_path.0.clone()
            } else {
                scoped_dependency_compile_name(
                    &current.compile_module_name,
                    dependency_package_id,
                    &import_path.0,
                )
            }
        }
        PackageImportOrigin::OwnPackage => remap_own_package_compile_name(
            &current.source_module_path.0,
            &current.compile_module_name,
            &resolved_module_path.0,
        ),
    }
}

fn scoped_dependency_compile_name(
    current_compile_module: &str,
    dependency_package_id: &SifrPackageId,
    import_path: &str,
) -> String {
    let current_root = current_compile_module
        .split('.')
        .next()
        .filter(|root| !root.is_empty())
        .unwrap_or("pkg");
    format!(
        "{current_root}.__pkg_{}.{}",
        package_instance_hash(dependency_package_id),
        import_path
    )
}

fn package_instance_hash(package_id: &SifrPackageId) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in package_id.0.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

fn remap_own_package_compile_name(
    current_source_module: &str,
    current_compile_module: &str,
    resolved_source_module: &str,
) -> String {
    let Some(source_root) = current_source_module.split('.').next() else {
        return resolved_source_module.to_string();
    };
    let Some(compile_root) = current_compile_module.split('.').next() else {
        return resolved_source_module.to_string();
    };
    if source_root == compile_root {
        return resolved_source_module.to_string();
    }
    if resolved_source_module == source_root {
        return compile_root.to_string();
    }
    resolved_source_module
        .strip_prefix(source_root)
        .and_then(|suffix| suffix.strip_prefix('.'))
        .map_or_else(
            || resolved_source_module.to_string(),
            |suffix| format!("{compile_root}.{suffix}"),
        )
}

fn package_import_dependencies(
    current_module: &str,
    is_namespace_module: bool,
    stmts: &[Stmt],
) -> BTreeMap<DottedModulePath, PackageImportDependency> {
    let mut deps = BTreeMap::new();
    for stmt in stmts {
        let Stmt::ImportFrom(import_from) = stmt else {
            continue;
        };
        if import_from.level > 1 {
            continue;
        }
        let Some(module) = &import_from.module else {
            continue;
        };
        let module_name = module.to_string();
        if module_name == "typing"
            || module_name == "enum"
            || module_name.starts_with("sifr.")
            || module_name.starts_with("_sifr.")
        {
            continue;
        }
        let resolved = if import_from.level == 1 {
            package_relative_import_module(current_module, is_namespace_module, &module_name)
        } else {
            module_name
        };
        let resolved_import_path = DottedModulePath(resolved);
        deps.entry(resolved_import_path.clone())
            .or_insert_with(|| PackageImportDependency {
                written_module_name: module.to_string(),
                written_range: module.range(),
                resolved_import_path,
            });
    }
    deps
}

fn rewrite_import_from_dependency(
    stmts: &mut [Stmt],
    current_module: &str,
    is_namespace_module: bool,
    dependency: &DottedModulePath,
    compile_module_name: &str,
) {
    for stmt in stmts {
        let Stmt::ImportFrom(import_from) = stmt else {
            continue;
        };
        if import_from.level > 1 {
            continue;
        }
        let Some(module) = &import_from.module else {
            continue;
        };
        let module_name = module.to_string();
        if module_name == "typing"
            || module_name == "enum"
            || module_name.starts_with("sifr.")
            || module_name.starts_with("_sifr.")
        {
            continue;
        }
        let resolved = if import_from.level == 1 {
            package_relative_import_module(current_module, is_namespace_module, &module_name)
        } else {
            module_name
        };
        if resolved != dependency.0 {
            continue;
        }
        import_from.module = Some(Identifier::new(
            compile_module_name.to_string(),
            import_from
                .module
                .as_ref()
                .map_or(TextRange::default(), |module| module.range),
        ));
        import_from.level = 0;
    }
}

fn package_relative_import_module(
    current_module: &str,
    is_namespace_module: bool,
    module_name: &str,
) -> String {
    let base = if is_namespace_module {
        current_module
    } else {
        current_module
            .rsplit_once('.')
            .map_or(current_module, |(parent, _)| parent)
    };
    if base.is_empty() {
        module_name.to_string()
    } else {
        format!("{base}.{module_name}")
    }
}

fn package_import_diagnostic(error: sifr_package::PackageDiagnostic) -> RenderedDiagnostic {
    crate::diagnostics::render_package_diagnostic(error)
}
