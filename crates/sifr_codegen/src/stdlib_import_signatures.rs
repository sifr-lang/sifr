use crate::{FuncSignature, RustEmitter, StdlibCode};
use sifr_ir::HirImport;
use std::collections::BTreeSet;

fn local_import_name(import: &HirImport, name: &str) -> String {
    import
        .aliases
        .iter()
        .find(|(original, _)| original == name)
        .map_or_else(|| name.to_string(), |(_, alias)| alias.clone())
}

fn transitive_stdlib_signature(
    stdlib_code: &StdlibCode,
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
    stdlib_code: &StdlibCode,
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
                .insert(name.to_string(), sig.clone());
        }
        return;
    }
    if let Some(sig) = transitive_stdlib_signature(stdlib_code, &import.module, name) {
        emitter
            .func_signatures
            .insert(local_name.clone(), sig.clone());
        if import.module.starts_with("_sifr.") && local_name != name {
            emitter.func_signatures.insert(name.to_string(), sig);
        }
    }
}
