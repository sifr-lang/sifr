use crate::{FuncSignature, RustEmitter};
use sifr_ir::HirImport;
use std::collections::BTreeSet;

pub(crate) fn register_imported_stdlib_metadata(
    emitter: &mut RustEmitter,
    module: &sifr_ir::HirModule,
    stdlib_code: &crate::StdlibEmissionCode,
) {
    // Register stdlib generic classes so user code skips explicit type annotations
    emitter
        .generic_classes
        .extend(stdlib_code.generic_classes.iter().cloned());
    emitter
        .generic_class_params
        .extend(stdlib_code.generic_class_params.clone());
    emitter
        .generic_class_templates
        .extend(stdlib_code.generic_class_templates.clone());

    // Pre-register imported constants and function signatures so user code can reference them correctly.
    crate::project_constants::register_imported_constants(emitter, module, stdlib_code);
    for import in &module.imports {
        if let Some(sig_map) = stdlib_code.func_signatures.get(&import.module) {
            for name in &import.names {
                register_imported_stdlib_signature(emitter, stdlib_code, import, name);
                // Also load class method signatures (ClassName::method entries)
                let prefix = format!("{name}::");
                for (key, sig) in sig_map {
                    if let Some(method) = key.strip_prefix(&prefix) {
                        let local_name = import
                            .aliases
                            .iter()
                            .find(|(original, _)| original == name)
                            .map_or(name.as_str(), |(_, alias)| alias.as_str());
                        emitter
                            .func_signatures
                            .insert(format!("{local_name}::{method}"), sig.clone());
                    }
                }
            }
            // Load class method signatures for classes returned by imported functions.
            // This handles cases like `compile_flags` returning `Pattern` - we need
            // `Pattern::search` etc. to be available for correct borrow prefix emission.
            for (key, sig) in sig_map {
                if key.contains("::") && !emitter.func_signatures.contains_key(key) {
                    emitter.func_signatures.insert(key.clone(), sig.clone());
                }
            }
        } else {
            for name in &import.names {
                register_imported_stdlib_signature(emitter, stdlib_code, import, name);
            }
        }
        if let Some(class_fields) = stdlib_code.module_class_fields.get(&import.module) {
            for name in &import.names {
                if let Some(fields) = class_fields.get(name) {
                    let local_name = import
                        .aliases
                        .iter()
                        .find(|(original, _)| original == name)
                        .map(|(_, alias)| alias.as_str())
                        .unwrap_or(name);
                    emitter.register_external_class_fields(local_name, name, fields);
                }
            }
        }
        // Pre-register stdlib generator functions so .collect() is emitted at call sites
        if let Some(gen_set) = stdlib_code.generator_functions.get(&import.module) {
            for name in &import.names {
                if gen_set.contains(name) {
                    emitter.generator_functions.insert(name.clone());
                }
            }
        }
    }
}

fn local_import_name(import: &HirImport, name: &str) -> String {
    import
        .aliases
        .iter()
        .find(|(original, _)| original == name)
        .map_or_else(|| name.to_string(), |(_, alias)| alias.clone())
}

fn transitive_stdlib_signature(
    stdlib_code: &crate::StdlibEmissionCode,
    module_name: &str,
    name: &str,
) -> Option<FuncSignature> {
    let deps = stdlib_code.transitive_deps.get(module_name)?;
    let mut found = None;
    for dep in deps.iter().collect::<BTreeSet<_>>() {
        let Some(sig) = stdlib_code
            .func_signatures
            .get(dep.as_str())
            .and_then(|sig_map| sig_map.get(name))
        else {
            continue;
        };
        if found.is_some() {
            return None;
        }
        found = Some(sig.clone());
    }
    found
}

pub(crate) fn register_imported_stdlib_signature(
    emitter: &mut RustEmitter,
    stdlib_code: &crate::StdlibEmissionCode,
    import: &HirImport,
    name: &str,
) {
    let local_name = local_import_name(import, name);
    if let Some(sig) = stdlib_code
        .func_signatures
        .get(&import.module)
        .and_then(|sig_map| sig_map.get(name))
    {
        emitter
            .func_signatures
            .insert(local_name.clone(), sig.clone());
        if import.module.starts_with("_sifr.") && local_name != name {
            emitter
                .func_signatures
                .entry(name.to_string())
                .or_insert_with(|| sig.clone());
        }
        return;
    }
    if let Some(sig) = transitive_stdlib_signature(stdlib_code, &import.module, name) {
        emitter
            .func_signatures
            .insert(local_name.clone(), sig.clone());
        if import.module.starts_with("_sifr.") && local_name != name {
            emitter
                .func_signatures
                .entry(name.to_string())
                .or_insert(sig);
        }
    }
}

#[cfg(test)]
#[path = "stdlib_import_signatures_tests.rs"]
mod tests;
