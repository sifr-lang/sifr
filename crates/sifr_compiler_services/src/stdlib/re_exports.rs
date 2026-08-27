use sifr_lowering::{CompilerIntrinsicId, ExternalDefs, HirExpr};
use sifr_type_system::{FunctionType, Type};
use std::collections::{HashMap, HashSet};

pub(super) struct ReExportMaps<'a> {
    pub(super) functions: &'a mut HashMap<String, FunctionType>,
    pub(super) compiler_intrinsics: &'a mut HashMap<String, CompilerIntrinsicId>,
    pub(super) classes: &'a mut HashMap<String, Type>,
    pub(super) error_types: &'a mut HashSet<String>,
    pub(super) class_type_params: &'a mut HashMap<String, Vec<String>>,
    pub(super) defaults: &'a mut HashMap<String, Vec<(usize, HirExpr)>>,
    pub(super) varargs: &'a mut HashMap<String, usize>,
    pub(super) workloads: &'a mut HashMap<String, String>,
    pub(super) constants: &'a mut HashMap<String, Type>,
}

pub(super) fn re_export_stdlib_imports(
    exports: &mut ReExportMaps<'_>,
    stdlib_defs: &ExternalDefs,
    exporting_module: &str,
    import_module: &str,
    import_names: &[String],
    import_aliases: &[(String, String)],
) {
    let aliases = import_aliases
        .iter()
        .map(|(original, local)| (original.as_str(), local.as_str()))
        .collect::<HashMap<_, _>>();
    let exportable_names = import_names
        .iter()
        .filter(|name| {
            if !import_module.starts_with("_sifr.") {
                return true;
            }
            let local_name = aliases.get(name.as_str()).copied().unwrap_or(name.as_str());
            crate::private_re_exports::approved_private_re_export(
                exporting_module,
                import_module,
                name,
                local_name,
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    let imported_names = exportable_names
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    copy_named_exports(
        exports,
        stdlib_defs,
        import_module,
        &exportable_names,
        &aliases,
    );
    copy_callable_metadata(
        exports,
        stdlib_defs,
        import_module,
        &imported_names,
        &aliases,
    );
}

fn copy_named_exports(
    exports: &mut ReExportMaps<'_>,
    stdlib_defs: &ExternalDefs,
    import_module: &str,
    import_names: &[String],
    aliases: &HashMap<&str, &str>,
) {
    for name in import_names {
        let export_name = aliases.get(name.as_str()).copied().unwrap_or(name.as_str());
        if let Some(ft) = stdlib_defs
            .functions
            .get(import_module)
            .and_then(|module_fns| module_fns.get(name))
        {
            exports
                .functions
                .entry(export_name.to_string())
                .or_insert_with(|| ft.clone());
            continue;
        }
        if let Some(class_ty) = stdlib_defs
            .classes
            .get(import_module)
            .and_then(|module_classes| module_classes.get(name))
        {
            if stdlib_defs.is_error_type(import_module, name) {
                exports.error_types.insert(export_name.to_string());
            }
            if !exports.classes.contains_key(export_name) {
                exports
                    .classes
                    .insert(export_name.to_string(), class_ty.clone());
                if let Some(type_params) = stdlib_defs
                    .class_type_params
                    .get(import_module)
                    .and_then(|module_type_params| module_type_params.get(name))
                {
                    exports
                        .class_type_params
                        .insert(export_name.to_string(), type_params.clone());
                }
            } else if let Some(type_params) = stdlib_defs
                .class_type_params
                .get(import_module)
                .and_then(|module_type_params| module_type_params.get(name))
            {
                exports
                    .class_type_params
                    .entry(export_name.to_string())
                    .or_insert_with(|| type_params.clone());
            }
            continue;
        }
        if let Some(const_ty) = stdlib_defs
            .constants
            .get(import_module)
            .and_then(|module_constants| module_constants.get(name))
        {
            exports
                .constants
                .entry(export_name.to_string())
                .or_insert_with(|| const_ty.clone());
        }
    }
}

fn copy_callable_metadata(
    exports: &mut ReExportMaps<'_>,
    stdlib_defs: &ExternalDefs,
    import_module: &str,
    imported_names: &HashSet<&str>,
    aliases: &HashMap<&str, &str>,
) {
    if let Some(module_intrinsics) = stdlib_defs.compiler_intrinsics.get(import_module) {
        for (callable_name, intrinsic) in module_intrinsics {
            if let Some(export_name) =
                exported_callable_name(callable_name, imported_names, aliases)
            {
                exports
                    .compiler_intrinsics
                    .entry(export_name)
                    .or_insert(*intrinsic);
            }
        }
    }
    if let Some(module_defaults) = stdlib_defs.function_defaults.get(import_module) {
        for (callable_name, defaults) in module_defaults {
            if let Some(export_name) =
                exported_callable_name(callable_name, imported_names, aliases)
            {
                exports
                    .defaults
                    .entry(export_name)
                    .or_insert_with(|| defaults.clone());
            }
        }
    }
    if let Some(module_varargs) = stdlib_defs.function_varargs.get(import_module) {
        for (callable_name, vararg_index) in module_varargs {
            if let Some(export_name) =
                exported_callable_name(callable_name, imported_names, aliases)
            {
                exports.varargs.entry(export_name).or_insert(*vararg_index);
            }
        }
    }
    if let Some(module_workloads) = stdlib_defs.function_workloads.get(import_module) {
        for (callable_name, label) in module_workloads {
            if let Some(export_name) =
                exported_callable_name(callable_name, imported_names, aliases)
            {
                exports
                    .workloads
                    .entry(export_name)
                    .or_insert_with(|| label.clone());
            }
        }
    }
}

fn exported_callable_name(
    callable_name: &str,
    imported_names: &HashSet<&str>,
    aliases: &HashMap<&str, &str>,
) -> Option<String> {
    if imported_names.contains(callable_name) {
        return Some(
            aliases
                .get(callable_name)
                .copied()
                .unwrap_or(callable_name)
                .to_string(),
        );
    }
    let (owner_name, member_name) = callable_name.split_once('.')?;
    if !imported_names.contains(owner_name) {
        return None;
    }
    let exported_owner = aliases.get(owner_name).copied().unwrap_or(owner_name);
    Some(format!("{exported_owner}.{member_name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_sysroot_re_export_preserves_compiler_identity() {
        let mut defs = ExternalDefs::default();
        defs.compiler_intrinsics
            .entry("sifr.origin".to_string())
            .or_default()
            .insert("verify".to_string(), CompilerIntrinsicId::TestAssertTrue);
        defs.functions
            .entry("sifr.origin".to_string())
            .or_default()
            .insert(
                "verify".to_string(),
                FunctionType {
                    receiver: None,
                    params: vec![],
                    return_type: Box::new(Type::None),
                },
            );
        defs.insert_error_type("sifr.origin", "Failure");
        defs.classes
            .entry("sifr.origin".to_string())
            .or_default()
            .insert(
                "Failure".to_string(),
                Type::Class {
                    identity: None,
                    type_args: Vec::new(),
                    name: "Failure".to_string(),
                    fields: Vec::new(),
                    methods: Vec::new(),
                    parent_class: Some("Error".to_string()),
                },
            );

        let mut functions = HashMap::new();
        let mut compiler_intrinsics = HashMap::new();
        let mut classes = HashMap::new();
        let mut error_types = HashSet::new();
        let mut class_type_params = HashMap::new();
        let mut defaults = HashMap::new();
        let mut varargs = HashMap::new();
        let mut workloads = HashMap::new();
        let mut constants = HashMap::new();
        re_export_stdlib_imports(
            &mut ReExportMaps {
                functions: &mut functions,
                compiler_intrinsics: &mut compiler_intrinsics,
                classes: &mut classes,
                error_types: &mut error_types,
                class_type_params: &mut class_type_params,
                defaults: &mut defaults,
                varargs: &mut varargs,
                workloads: &mut workloads,
                constants: &mut constants,
            },
            &defs,
            "sifr.facade",
            "sifr.origin",
            &["verify".to_string(), "Failure".to_string()],
            &[],
        );

        assert_eq!(
            compiler_intrinsics.get("verify"),
            Some(&CompilerIntrinsicId::TestAssertTrue)
        );
        assert!(error_types.contains("Failure"));
    }

    #[test]
    fn public_aliased_imports_use_the_local_export_name() {
        let mut defs = ExternalDefs::default();
        defs.functions
            .entry("sifr.origin".to_string())
            .or_default()
            .insert(
                "legacy".to_string(),
                FunctionType {
                    receiver: None,
                    params: vec![],
                    return_type: Box::new(Type::Int),
                },
            );

        let mut functions = HashMap::new();
        let mut compiler_intrinsics = HashMap::new();
        let mut classes = HashMap::new();
        let mut error_types = HashSet::new();
        let mut class_type_params = HashMap::new();
        let mut defaults = HashMap::new();
        let mut varargs = HashMap::new();
        let mut workloads = HashMap::new();
        let mut constants = HashMap::new();
        re_export_stdlib_imports(
            &mut ReExportMaps {
                functions: &mut functions,
                compiler_intrinsics: &mut compiler_intrinsics,
                classes: &mut classes,
                error_types: &mut error_types,
                class_type_params: &mut class_type_params,
                defaults: &mut defaults,
                varargs: &mut varargs,
                workloads: &mut workloads,
                constants: &mut constants,
            },
            &defs,
            "sifr.facade",
            "sifr.origin",
            &["legacy".to_string()],
            &[("legacy".to_string(), "_legacy_impl".to_string())],
        );

        assert!(!functions.contains_key("legacy"));
        assert!(functions.contains_key("_legacy_impl"));
    }

    #[test]
    fn unapproved_private_imports_do_not_enter_public_exports() {
        let mut defs = ExternalDefs::default();
        defs.functions
            .entry("_sifr.origin".to_string())
            .or_default()
            .insert(
                "implementation_name".to_string(),
                FunctionType {
                    receiver: None,
                    params: vec![],
                    return_type: Box::new(Type::Int),
                },
            );
        defs.classes
            .entry("_sifr.origin".to_string())
            .or_default()
            .insert(
                "ImplementationClass".to_string(),
                Type::Class {
                    identity: None,
                    type_args: Vec::new(),
                    name: "ImplementationClass".to_string(),
                    fields: Vec::new(),
                    methods: Vec::new(),
                    parent_class: None,
                },
            );
        defs.constants
            .entry("_sifr.origin".to_string())
            .or_default()
            .insert("IMPLEMENTATION_VALUE".to_string(), Type::Int);

        let mut functions = HashMap::new();
        let mut compiler_intrinsics = HashMap::new();
        let mut classes = HashMap::new();
        let mut error_types = HashSet::new();
        let mut class_type_params = HashMap::new();
        let mut defaults = HashMap::new();
        let mut varargs = HashMap::new();
        let mut workloads = HashMap::new();
        let mut constants = HashMap::new();
        re_export_stdlib_imports(
            &mut ReExportMaps {
                functions: &mut functions,
                compiler_intrinsics: &mut compiler_intrinsics,
                classes: &mut classes,
                error_types: &mut error_types,
                class_type_params: &mut class_type_params,
                defaults: &mut defaults,
                varargs: &mut varargs,
                workloads: &mut workloads,
                constants: &mut constants,
            },
            &defs,
            "sifr.facade",
            "_sifr.origin",
            &[
                "implementation_name".to_string(),
                "ImplementationClass".to_string(),
                "IMPLEMENTATION_VALUE".to_string(),
            ],
            &[
                (
                    "implementation_name".to_string(),
                    "public_alias".to_string(),
                ),
                (
                    "ImplementationClass".to_string(),
                    "PublicClassAlias".to_string(),
                ),
                (
                    "IMPLEMENTATION_VALUE".to_string(),
                    "PUBLIC_VALUE_ALIAS".to_string(),
                ),
            ],
        );

        assert!(functions.is_empty());
        assert!(classes.is_empty());
        assert!(constants.is_empty());
    }
}
