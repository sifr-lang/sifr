use sifr_lowering::{
    canonicalize_user_export_type, substitute_type_vars, AdapterHandlerPlan,
    DeclarationMetadataTargetKind, ExternalDefs, HirClass, MethodKind, StructuralMethodExport,
};
use sifr_type_system::Type;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct ClassMethodExports {
    instance: HashMap<String, HashSet<String>>,
    consuming: HashMap<String, HashSet<String>>,
}

fn local_instance_methods(
    class: &HirClass,
    known: &HashMap<String, HashSet<String>>,
) -> HashSet<String> {
    let mut methods = class
        .parent_class
        .as_ref()
        .and_then(|parent| known.get(parent))
        .cloned()
        .unwrap_or_default();
    for method in &class.methods {
        if method.name == "new" {
            continue;
        }
        if method.method_kind == MethodKind::Regular {
            methods.insert(method.name.clone());
        } else {
            methods.remove(&method.name);
        }
    }
    methods
}

fn imported_instance_methods(
    external_defs: &ExternalDefs,
    module: &str,
    class_name: &str,
) -> HashSet<String> {
    external_defs
        .class_instance_methods
        .get(module)
        .and_then(|classes| classes.get(class_name))
        .cloned()
        .unwrap_or_default()
}

fn exported_structural_methods(
    class: &HirClass,
    local_classes: &HashMap<String, String>,
    declared_names: &[&str],
) -> Option<Vec<StructuralMethodExport>> {
    let methods = declared_names
        .iter()
        .filter_map(|declared_name| {
            let hir_name = if *declared_name == "__init__" {
                "new"
            } else {
                declared_name
            };
            let method = class
                .methods
                .iter()
                .chain(class.operator_impls.iter().map(|(_, method)| method))
                .find(|method| method.name == hir_name)?;
            Some(StructuralMethodExport {
                handler_target: None,
                name: declared_name.to_string(),
                params: method
                    .params
                    .iter()
                    .cloned()
                    .map(|mut param| {
                        param.ty = canonicalize_user_export_type(&param.ty, local_classes);
                        param
                    })
                    .collect(),
                return_type: canonicalize_user_export_type(&method.return_type, local_classes),
                is_async: method.is_async,
                method_kind: method.method_kind,
                receiver: method.receiver,
            })
        })
        .collect::<Vec<_>>();
    (!methods.is_empty()).then_some(methods)
}

fn adapted_handler_method(
    module_name: &str,
    owner: &HirClass,
    handler: &AdapterHandlerPlan,
    local_classes: &HashMap<String, String>,
    lowering: &sifr_lowering::LoweringResult,
    external_defs: &ExternalDefs,
) -> Option<StructuralMethodExport> {
    let callable_owner = handler.callable.owner.as_deref()?;
    let root_args = owner
        .type_params
        .iter()
        .cloned()
        .map(Type::TypeVar)
        .collect::<Vec<_>>();
    let bindings = crate::handler_ancestry::bindings_for_owner(
        module_name,
        owner,
        &root_args,
        callable_owner,
        lowering,
        external_defs,
    );
    if bindings.is_none() {
        return adapted_imported_parent_method(
            owner,
            &root_args,
            handler,
            local_classes,
            external_defs,
        );
    }
    let bindings = bindings.unwrap_or_default();
    let (source_module, source_name) = callable_owner.rsplit_once('.')?;
    let hir_name = if handler.callable.symbol == "__init__" {
        "new"
    } else {
        handler.callable.symbol.as_str()
    };
    if source_module == module_name {
        let source = lowering.module.classes.iter().find(|class| {
            class.identity.as_deref() == Some(callable_owner)
                || (class.identity.is_none() && class.name == source_name)
        })?;
        let method = source
            .methods
            .iter()
            .chain(source.operator_impls.iter().map(|(_, method)| method))
            .find(|method| method.name == hir_name)?;
        return Some(StructuralMethodExport {
            handler_target: Some(handler.callable.clone()),
            name: handler.callable.symbol.clone(),
            params: method
                .params
                .iter()
                .cloned()
                .map(|mut param| {
                    param.ty = canonicalize_user_export_type(
                        &substitute_type_vars(&param.ty, &bindings),
                        local_classes,
                    );
                    param
                })
                .collect(),
            return_type: canonicalize_user_export_type(
                &substitute_type_vars(&method.return_type, &bindings),
                local_classes,
            ),
            is_async: method.is_async,
            method_kind: method.method_kind,
            receiver: method.receiver,
        });
    }
    let method = external_defs
        .structural_methods_for(source_module)?
        .get(source_name)?
        .iter()
        .find(|method| {
            method.handler_target.as_ref() == Some(&handler.callable)
                || (method.handler_target.is_none() && method.name == handler.callable.symbol)
        })?;
    let mut method = method.clone();
    method.handler_target = Some(handler.callable.clone());
    for parameter in &mut method.params {
        parameter.ty = canonicalize_user_export_type(
            &substitute_type_vars(&parameter.ty, &bindings),
            local_classes,
        );
    }
    method.return_type = canonicalize_user_export_type(
        &substitute_type_vars(&method.return_type, &bindings),
        local_classes,
    );
    Some(method)
}

fn adapted_imported_parent_method(
    owner: &HirClass,
    root_args: &[Type],
    handler: &AdapterHandlerPlan,
    local_classes: &HashMap<String, String>,
    external_defs: &ExternalDefs,
) -> Option<StructuralMethodExport> {
    let root_bindings = owner
        .type_params
        .iter()
        .cloned()
        .zip(root_args.iter().cloned())
        .collect::<HashMap<_, _>>();
    let Type::Class {
        identity,
        name,
        type_args,
        ..
    } = owner.parent_type.as_ref()?.resolve_alias()
    else {
        return None;
    };
    let parent_identity = identity.as_deref().unwrap_or(name);
    let (parent_module, parent_name) = parent_identity.rsplit_once('.')?;
    let parent_params = external_defs
        .class_type_params
        .get(parent_module)
        .and_then(|classes| classes.get(parent_name))?;
    let parent_bindings = parent_params
        .iter()
        .cloned()
        .zip(
            type_args
                .iter()
                .map(|argument| substitute_type_vars(argument, &root_bindings)),
        )
        .collect::<HashMap<_, _>>();
    let mut method = external_defs
        .structural_methods_for(parent_module)?
        .get(parent_name)?
        .iter()
        .find(|method| method.handler_target.as_ref() == Some(&handler.callable))?
        .clone();
    for parameter in &mut method.params {
        parameter.ty = canonicalize_user_export_type(
            &substitute_type_vars(&parameter.ty, &parent_bindings),
            local_classes,
        );
    }
    method.return_type = canonicalize_user_export_type(
        &substitute_type_vars(&method.return_type, &parent_bindings),
        local_classes,
    );
    Some(method)
}

pub(crate) fn structural_method_map(
    module_name: &str,
    module: &sifr_lowering::HirModule,
    local_classes: &HashMap<String, String>,
    lowering: &sifr_lowering::LoweringResult,
    external_defs: &ExternalDefs,
) -> HashMap<String, Vec<StructuralMethodExport>> {
    if module.classes.is_empty() {
        return HashMap::new();
    }
    let mut names_by_class: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut seen: HashSet<String> = HashSet::new();
    for entry in &lowering.declaration_metadata {
        if entry.target_kind != DeclarationMetadataTargetKind::Method
            || !seen.insert(entry.owner.clone())
        {
            continue;
        }
        let Some((class_name, method_name)) = entry.owner.rsplit_once('.') else {
            continue;
        };
        names_by_class
            .entry(class_name)
            .or_default()
            .push(method_name);
    }
    for selection in &lowering.class_adapter_selections {
        for handler in &selection.handler_plans {
            let Some(owner) = handler.callable.owner.as_deref() else {
                continue;
            };
            let Some(class_name) = owner.rsplit_once('.').map(|(_, name)| name) else {
                continue;
            };
            if local_classes.get(class_name).map(String::as_str) != Some(owner) {
                continue;
            }
            let key = format!("{class_name}.{}", handler.callable.symbol);
            if seen.insert(key) {
                names_by_class
                    .entry(class_name)
                    .or_default()
                    .push(handler.callable.symbol.as_str());
            }
        }
    }
    let mut methods = module
        .classes
        .iter()
        .filter_map(|class| {
            let declared_names = names_by_class.get(class.name.as_str())?;
            exported_structural_methods(class, local_classes, declared_names)
                .map(|methods| (class.name.clone(), methods))
        })
        .collect::<HashMap<_, _>>();
    for selection in &lowering.class_adapter_selections {
        let Some(owner) = module
            .classes
            .iter()
            .find(|class| class.name == selection.owner)
        else {
            continue;
        };
        let exports = selection
            .handler_plans
            .iter()
            .filter_map(|handler| {
                adapted_handler_method(
                    module_name,
                    owner,
                    handler,
                    local_classes,
                    lowering,
                    external_defs,
                )
            })
            .collect::<Vec<_>>();
        if !exports.is_empty() {
            methods
                .entry(selection.owner.clone())
                .or_default()
                .extend(exports);
        }
    }
    methods
}

impl ClassMethodExports {
    pub(crate) fn record_local(&mut self, class: &HirClass) {
        let methods = local_instance_methods(class, &self.instance);
        self.instance.insert(class.name.clone(), methods);
        let Some(selected) = sifr_lowering::rust_opaque_close_method(&class.rust_interop) else {
            return;
        };
        let consuming = class
            .methods
            .iter()
            .filter(|method| {
                method.name == selected
                    && method
                        .rust_interop
                        .iter()
                        .any(|declaration| declaration.consumes_receiver)
            })
            .map(|method| method.name.clone())
            .collect::<HashSet<_>>();
        if !consuming.is_empty() {
            self.consuming.insert(class.name.clone(), consuming);
        }
    }

    pub(crate) fn record_imported(
        &mut self,
        external_defs: &ExternalDefs,
        module: &str,
        source_name: &str,
        local_name: &str,
    ) {
        let methods = imported_instance_methods(external_defs, module, source_name);
        self.instance.insert(local_name.to_string(), methods);
        if let Some(consuming) = external_defs
            .rust_consuming_methods
            .get(module)
            .and_then(|classes| classes.get(source_name))
            .cloned()
        {
            self.consuming.insert(local_name.to_string(), consuming);
        }
    }

    pub(crate) fn store(self, external_defs: &mut ExternalDefs, module_name: &str) {
        external_defs
            .class_instance_methods
            .insert(module_name.to_string(), self.instance);
        if !self.consuming.is_empty() {
            external_defs
                .rust_consuming_methods
                .insert(module_name.to_string(), self.consuming);
        }
    }
}
