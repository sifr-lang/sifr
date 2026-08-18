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
mod tests {
    use super::*;
    use sifr_type_system::{ParamConvention, Type};

    fn signature(return_type: Type) -> FuncSignature {
        (vec![(Type::Int, ParamConvention::borrow())], return_type)
    }

    fn private_import(module: &str, alias: &str) -> HirImport {
        HirImport {
            module: module.to_string(),
            names: vec!["shared_name".to_string()],
            aliases: vec![("shared_name".to_string(), alias.to_string())],
        }
    }

    #[test]
    fn private_alias_registration_preserves_local_signatures_without_overwriting_origin() {
        let mut code = StdlibCode::default();
        code.func_signatures.insert(
            "_sifr.first".to_string(),
            [("shared_name".to_string(), signature(Type::Int))].into(),
        );
        code.func_signatures.insert(
            "_sifr.second".to_string(),
            [("shared_name".to_string(), signature(Type::Str))].into(),
        );
        let mut emitter = RustEmitter::new();

        register_imported_stdlib_signature(
            &mut emitter,
            &code,
            &private_import("_sifr.first", "_first_impl"),
            "shared_name",
        );
        register_imported_stdlib_signature(
            &mut emitter,
            &code,
            &private_import("_sifr.second", "_second_impl"),
            "shared_name",
        );

        assert_eq!(emitter.func_signatures["_first_impl"].1, Type::Int);
        assert_eq!(emitter.func_signatures["_second_impl"].1, Type::Str);
        assert_eq!(emitter.func_signatures["shared_name"].1, Type::Int);
    }
}
