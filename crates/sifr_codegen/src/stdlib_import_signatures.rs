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
    fn imported_metadata_preserves_aliases_methods_fields_generators_and_constants() {
        let parsed = sifr_python_parser::parse_module("class Container[T]:\n    value: T\n")
            .expect("parse metadata fixture");
        let mut module = sifr_lowering::lower_module(parsed.suite())
            .expect("lower metadata fixture")
            .module;
        let template = std::sync::Arc::new(module.classes.remove(0));
        module.imports.push(HirImport {
            module: "sifr.fixture".into(),
            names: vec![
                "consume".into(),
                "Container".into(),
                "items".into(),
                "LIMIT".into(),
            ],
            aliases: vec![
                ("consume".into(), "borrowed".into()),
                ("Container".into(), "Alias".into()),
                ("LIMIT".into(), "BOUND".into()),
            ],
        });
        let mut code = crate::StdlibCode::default();
        code.func_signatures.insert(
            "sifr.fixture".into(),
            [
                ("consume".into(), signature(Type::Bool)),
                ("Container::read".into(), signature(Type::Int)),
                ("Returned::read".into(), signature(Type::Str)),
            ]
            .into(),
        );
        code.module_class_fields.insert(
            "sifr.fixture".into(),
            [("Container".into(), vec![("value".into(), Type::Int)])].into(),
        );
        code.generic_classes.insert("Container".into());
        code.generic_class_params
            .insert("Container".into(), vec!["T".into()]);
        code.generic_class_templates
            .insert("Container".into(), template.clone());
        code.generator_functions
            .insert("sifr.fixture".into(), ["items".into()].into());
        code.module_constants.insert(
            "sifr.fixture".into(),
            [("LIMIT".into(), (Type::Int, "crate::LIMIT".into()))].into(),
        );

        let mut emitter = RustEmitter::new();
        register_imported_stdlib_metadata(&mut emitter, &module, &code);
        assert_eq!(emitter.func_signatures["borrowed"], signature(Type::Bool));
        assert_eq!(emitter.func_signatures["Alias::read"], signature(Type::Int));
        assert_eq!(
            emitter.func_signatures["Returned::read"],
            signature(Type::Str)
        );
        assert_eq!(emitter.class_field_order["Alias"], vec!["value"]);
        assert_eq!(
            emitter.class_field_types[&("Alias".into(), "value".into())],
            Type::Int
        );
        assert!(emitter.generic_classes.contains("Container"));
        assert_eq!(emitter.generic_class_params["Container"], vec!["T"]);
        assert!(std::sync::Arc::ptr_eq(
            &emitter.generic_class_templates["Container"],
            &template
        ));
        assert!(emitter.generator_functions.contains("items"));
        assert_eq!(
            emitter.module_constants["BOUND"],
            (Type::Int, "crate::LIMIT".into())
        );
    }

    #[test]
    fn private_alias_registration_preserves_local_signatures_without_overwriting_origin() {
        let mut code = crate::StdlibCode::default();
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
