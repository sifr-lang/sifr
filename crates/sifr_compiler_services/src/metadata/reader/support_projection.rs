use crate::{
    export_policy::should_export_callable,
    stdlib::{signature_params, stdlib_class_template},
};
use sifr_codegen::StdlibCode;
use sifr_ir::HirModule;
use sifr_type_system::{ParamConvention, Type};
use std::collections::HashMap;
pub(super) fn project_signatures(
    module_name: &str,
    module: &HirModule,
    stdlib_code: &mut StdlibCode,
    private_declaration: bool,
) {
    let mut sig_map = HashMap::new();
    for func in &module.functions {
        if private_declaration || should_export_callable(module_name, &func.name) {
            let param_info = signature_params(&func.params, None);
            sig_map.insert(func.name.clone(), (param_info, func.return_type.clone()));
        }
    }
    for class in &module.classes {
        let mut has_constructor = false;
        for method in &class.methods {
            let param_info = signature_params(
                &method.params,
                (method.name == "new").then_some(ParamConvention::own()),
            );
            sig_map.insert(
                format!("{}::{}", class.name, method.name),
                (param_info, method.return_type.clone()),
            );
            if method.name == "new" {
                has_constructor = true;
            }
        }
        if !has_constructor {
            let ctor_params = class
                .fields
                .iter()
                .map(|(_, ty)| (ty.clone(), ParamConvention::own()))
                .collect::<Vec<_>>();
            sig_map.insert(
                format!("{}::new", class.name),
                (
                    ctor_params,
                    Type::Class {
                        identity: None,
                        type_args: class
                            .type_params
                            .iter()
                            .cloned()
                            .map(Type::TypeVar)
                            .collect(),
                        name: class.name.clone(),
                        fields: class.fields.clone().into(),
                        methods: Vec::new().into(),
                        parent_class: class.semantic_parent_chain(),
                    },
                ),
            );
        }
    }
    if !sig_map.is_empty() {
        stdlib_code
            .func_signatures
            .insert(module_name.to_string(), sig_map);
    }

    for class in &module.classes {
        if !class.type_params.is_empty() {
            stdlib_code.generic_classes.insert(class.name.clone());
            stdlib_code
                .generic_class_params
                .insert(class.name.clone(), class.type_params.clone());
            stdlib_code
                .generic_class_templates
                .insert(class.name.clone(), std::sync::Arc::new(class.clone()));
        }
    }
    let class_fields = module
        .classes
        .iter()
        .map(|class| (class.name.clone(), class.fields.clone()))
        .collect();
    stdlib_code
        .module_class_fields
        .insert(module_name.to_string(), class_fields);
    let class_templates = module
        .classes
        .iter()
        .map(|class| {
            (
                class.name.clone(),
                stdlib_class_template(module_name, class),
            )
        })
        .collect();
    stdlib_code
        .module_class_templates
        .insert(module_name.to_string(), class_templates);
}
