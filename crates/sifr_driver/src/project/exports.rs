use crate::export_policy::should_export_callable;
use sifr_hir::{ExternalDefs, LoweringResult};
use sifr_type_system::{FunctionType, ParamConvention, Type};
use std::collections::HashMap;

pub(crate) fn collect_module_exports(
    module_name: &str,
    lowering_result: &LoweringResult,
    external_defs: &mut ExternalDefs,
) {
    let module = &lowering_result.module;
    let mut fn_exports = HashMap::new();
    let mut class_exports = HashMap::new();
    let mut class_type_param_exports = HashMap::new();
    let mut const_exports = HashMap::new();
    let mut default_exports = HashMap::new();

    for func in &module.functions {
        if should_export_callable(module_name, &func.name) {
            let params: Vec<(String, Type, ParamConvention)> = func
                .params
                .iter()
                .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                .collect();
            fn_exports.insert(
                func.name.clone(),
                FunctionType {
                    params,
                    return_type: Box::new(func.return_type.clone()),
                },
            );
        }
    }

    for (callable_name, defaults) in &lowering_result.function_defaults {
        if should_export_callable(module_name, callable_name) {
            default_exports.insert(callable_name.clone(), defaults.clone());
        }
    }

    for class in &module.classes {
        if !class.name.starts_with('_') {
            let mut methods: Vec<(String, FunctionType)> = class
                .methods
                .iter()
                .filter(|m| m.name != "new")
                .map(|m| {
                    let params: Vec<(String, Type, ParamConvention)> = m
                        .params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                        .collect();
                    (
                        m.name.clone(),
                        FunctionType {
                            params,
                            return_type: Box::new(m.return_type.clone()),
                        },
                    )
                })
                .collect();
            for (dunder_name, op_func) in &class.operator_impls {
                let params: Vec<(String, Type, ParamConvention)> = op_func
                    .params
                    .iter()
                    .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                    .collect();
                methods.push((
                    dunder_name.clone(),
                    FunctionType {
                        params,
                        return_type: Box::new(op_func.return_type.clone()),
                    },
                ));
            }
            let class_ty = Type::Class {
                name: class.name.clone(),
                fields: class.fields.clone(),
                methods,
                parent_class: None,
            };
            class_exports.insert(class.name.clone(), class_ty);
            if !class.type_params.is_empty() {
                class_type_param_exports.insert(class.name.clone(), class.type_params.clone());
            }
        }
    }

    for (name, ty, _) in &module.constants {
        if !name.starts_with('_') {
            const_exports.insert(name.clone(), ty.clone());
        }
    }

    external_defs
        .functions
        .insert(module_name.to_string(), fn_exports);
    external_defs
        .classes
        .insert(module_name.to_string(), class_exports);
    if !class_type_param_exports.is_empty() {
        external_defs
            .class_type_params
            .insert(module_name.to_string(), class_type_param_exports);
    }
    if !default_exports.is_empty() {
        external_defs
            .function_defaults
            .insert(module_name.to_string(), default_exports);
    }
    external_defs
        .constants
        .insert(module_name.to_string(), const_exports);
}
