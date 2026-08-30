use crate::{HashMap, HashSet, HirModule, RustEmitter, StdlibCode};
use sifr_type_system::Type;

pub(crate) fn register_imported_constants(
    emitter: &mut RustEmitter,
    module: &HirModule,
    project_code: &StdlibCode,
) {
    for import in &module.imports {
        let Some(constants) = project_code.module_constants.get(&import.module) else {
            continue;
        };
        for source_name in &import.names {
            let Some((ty, rust_reference)) = constants.get(source_name) else {
                continue;
            };
            let local_name = import
                .aliases
                .iter()
                .find(|(original, _)| original == source_name)
                .map_or(source_name.as_str(), |(_, alias)| alias.as_str());
            emitter
                .module_constants
                .insert(local_name.to_string(), (ty.clone(), rust_reference.clone()));
        }
    }
}

pub(crate) fn extend_project_constant_mappings(
    project_code: &mut StdlibCode,
    modules: &[(&str, &HirModule)],
    crate_root_modules: &HashSet<&str>,
) {
    for (module_name, module) in modules {
        let rust_module_path = if crate_root_modules.contains(module_name) {
            "crate".to_string()
        } else {
            format!("crate::{}", module_name.replace('.', "::"))
        };
        let mut mappings = HashMap::<String, (Type, String)>::new();
        for (name, ty, value) in &module.constants {
            let local_reference = crate::module_constant_rust_reference(name, ty, value)
                .unwrap_or_else(|error| {
                    panic!("invalid project module constant {module_name}.{name}: {error}")
                });
            let qualified_reference =
                if let Some(function_name) = local_reference.strip_suffix("()") {
                    format!("{rust_module_path}::{function_name}()")
                } else {
                    format!("{rust_module_path}::{local_reference}")
                };
            mappings.insert(name.clone(), (ty.clone(), qualified_reference));
        }
        if !mappings.is_empty() {
            project_code
                .module_constants
                .insert((*module_name).to_string(), mappings);
        }
    }

    // Project modules can re-export imported constants. Resolve those aliases to the
    // original generated item so consumers never depend on a synthetic Rust name.
    loop {
        let mut changed = false;
        for (module_name, module) in modules {
            for import in &module.imports {
                let Some(imported_constants) =
                    project_code.module_constants.get(&import.module).cloned()
                else {
                    continue;
                };
                let module_constants = project_code
                    .module_constants
                    .entry((*module_name).to_string())
                    .or_default();
                for source_name in &import.names {
                    let Some(mapping) = imported_constants.get(source_name) else {
                        continue;
                    };
                    let exported_name = import
                        .aliases
                        .iter()
                        .find(|(original, _)| original == source_name)
                        .map_or(source_name.as_str(), |(_, alias)| alias.as_str());
                    if !module_constants.contains_key(exported_name) {
                        module_constants.insert(exported_name.to_string(), mapping.clone());
                        changed = true;
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
}
