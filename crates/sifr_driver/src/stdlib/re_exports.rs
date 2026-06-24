use sifr_lowering::{ExternalDefs, HirExpr};
use sifr_type_system::{FunctionType, Type};
use std::collections::{HashMap, HashSet};

pub(super) struct ReExportMaps<'a> {
    pub(super) functions: &'a mut HashMap<String, FunctionType>,
    pub(super) classes: &'a mut HashMap<String, Type>,
    pub(super) class_type_params: &'a mut HashMap<String, Vec<String>>,
    pub(super) defaults: &'a mut HashMap<String, Vec<(usize, HirExpr)>>,
    pub(super) varargs: &'a mut HashMap<String, usize>,
    pub(super) workloads: &'a mut HashMap<String, String>,
    pub(super) constants: &'a mut HashMap<String, Type>,
}

pub(super) fn re_export_stdlib_imports(
    exports: &mut ReExportMaps<'_>,
    stdlib_defs: &ExternalDefs,
    import_module: &str,
    import_names: &[String],
) {
    let imported_names = import_names
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    copy_named_exports(exports, stdlib_defs, import_module, import_names);
    copy_callable_metadata(exports, stdlib_defs, import_module, &imported_names);
}

fn copy_named_exports(
    exports: &mut ReExportMaps<'_>,
    stdlib_defs: &ExternalDefs,
    import_module: &str,
    import_names: &[String],
) {
    for name in import_names {
        if let Some(ft) = stdlib_defs
            .functions
            .get(import_module)
            .and_then(|module_fns| module_fns.get(name))
        {
            exports
                .functions
                .entry(name.clone())
                .or_insert_with(|| ft.clone());
            continue;
        }
        if let Some(class_ty) = stdlib_defs
            .classes
            .get(import_module)
            .and_then(|module_classes| module_classes.get(name))
        {
            if !exports.classes.contains_key(name) {
                exports.classes.insert(name.clone(), class_ty.clone());
                if let Some(type_params) = stdlib_defs
                    .class_type_params
                    .get(import_module)
                    .and_then(|module_type_params| module_type_params.get(name))
                {
                    exports
                        .class_type_params
                        .insert(name.clone(), type_params.clone());
                }
            } else if let Some(type_params) = stdlib_defs
                .class_type_params
                .get(import_module)
                .and_then(|module_type_params| module_type_params.get(name))
            {
                exports
                    .class_type_params
                    .entry(name.clone())
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
                .entry(name.clone())
                .or_insert_with(|| const_ty.clone());
        }
    }
}

fn copy_callable_metadata(
    exports: &mut ReExportMaps<'_>,
    stdlib_defs: &ExternalDefs,
    import_module: &str,
    imported_names: &HashSet<&str>,
) {
    if let Some(module_defaults) = stdlib_defs.function_defaults.get(import_module) {
        for (callable_name, defaults) in module_defaults {
            if is_imported_callable(callable_name, imported_names) {
                exports
                    .defaults
                    .entry(callable_name.clone())
                    .or_insert_with(|| defaults.clone());
            }
        }
    }
    if let Some(module_varargs) = stdlib_defs.function_varargs.get(import_module) {
        for (callable_name, vararg_index) in module_varargs {
            if is_imported_callable(callable_name, imported_names) {
                exports
                    .varargs
                    .entry(callable_name.clone())
                    .or_insert(*vararg_index);
            }
        }
    }
    if let Some(module_workloads) = stdlib_defs.function_workloads.get(import_module) {
        for (callable_name, label) in module_workloads {
            if is_imported_callable(callable_name, imported_names) {
                exports
                    .workloads
                    .entry(callable_name.clone())
                    .or_insert_with(|| label.clone());
            }
        }
    }
}

fn is_imported_callable(callable_name: &str, imported_names: &HashSet<&str>) -> bool {
    imported_names.contains(callable_name)
        || callable_name
            .split_once('.')
            .is_some_and(|(owner_name, _)| imported_names.contains(owner_name))
}
