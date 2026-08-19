use crate::ConstValue;
use sifr_lowering::{
    AdapterFieldDefault, AdapterFieldPlan, ClassAdapterSelection, ExternalDefs, HirClass,
    LoweringResult,
};
use sifr_type_system::{FunctionType, Type};
use std::collections::{BTreeMap, HashMap};

pub(super) fn declaration_value(
    module_name: &str,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
    declaration: &crate::class_declarations::ClassDeclaration,
    selection: &ClassAdapterSelection,
) -> Result<ConstValue, &'static str> {
    let mut value = declaration.to_const_value(result);
    let ConstValue::Record(fields) = &mut value else {
        return Err("class declaration is not structural");
    };
    let origin = fields
        .get("origin")
        .cloned()
        .ok_or("class declaration origin is unavailable")?;
    let Some(ConstValue::List(items)) = fields.get_mut("items") else {
        return Err("class declaration items are unavailable");
    };
    let Some(class) = result
        .module
        .classes
        .iter()
        .find(|class| class.name == selection.owner)
    else {
        return Ok(value);
    };
    if selection.data_parent.is_none() {
        return Ok(value);
    }
    let (parent_identity, mut parent_fields) = if let Some(Type::Class {
        identity,
        name,
        fields,
        type_args,
        ..
    }) = class.parent_type.as_ref()
    {
        let parent_identity = canonical_parent_identity(module_name, identity.as_deref(), name);
        let fields = if fields.is_empty() {
            local_parent_fields(module_name, result, &parent_identity, type_args)
        } else {
            fields.clone()
        };
        (parent_identity, fields)
    } else {
        return Err("adapted data parent shape is unavailable");
    };
    let parent_selection = parent_selection(module_name, result, external_defs, &parent_identity);
    if let Some(parent) = parent_selection.filter(|parent| !parent.field_plans.is_empty()) {
        let type_args = match class.parent_type.as_ref() {
            Some(Type::Class { type_args, .. }) => type_args.as_slice(),
            _ => &[],
        };
        let bindings = parent_bindings(
            module_name,
            result,
            external_defs,
            &parent_identity,
            type_args,
        );
        parent_fields = parent
            .field_plans
            .iter()
            .map(|field| {
                (
                    field.name.clone(),
                    sifr_lowering::substitute_type_vars(&field.declared_type, &bindings),
                )
            })
            .collect();
    }
    let mut inherited = Vec::with_capacity(parent_fields.len());
    for (index, (name, ty)) in parent_fields.iter().enumerate() {
        if let Some(local_index) = items.iter().position(|item| {
            matches!(
                item,
                ConstValue::Record(fields)
                    if matches!(fields.get("kind"), Some(ConstValue::String(kind)) if kind == "field")
                        && matches!(fields.get("name"), Some(ConstValue::String(local)) if local == name)
            )
        }) {
            inherited.push(items.remove(local_index));
        } else {
            inherited.push(inherited_field_item(
                index,
                name,
                ty,
                &parent_identity,
                parent_selection.and_then(|parent| parent.field_plans.get(index)),
                &origin,
            )?);
        }
    }
    let local_methods = items
        .iter()
        .filter_map(|item| match item {
            ConstValue::Record(fields)
                if matches!(fields.get("kind"), Some(ConstValue::String(kind)) if kind == "method") =>
            {
                match fields.get("name") {
                    Some(ConstValue::String(name)) => Some(name.clone()),
                    _ => None,
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    inherited.extend(
        parent_method_contracts(module_name, class, result, &parent_identity)
            .into_iter()
            .filter(|(name, _)| !local_methods.contains(name))
            .map(|(name, signature)| {
                inherited_method_item(&name, &signature, &parent_identity, &origin)
            }),
    );
    items.splice(0..0, inherited);
    for (order, item) in items.iter_mut().enumerate() {
        if let ConstValue::Record(fields) = item {
            fields.insert("order".to_string(), ConstValue::Integer(order.into()));
        }
    }
    Ok(value)
}

fn local_parent_fields(
    module_name: &str,
    result: &LoweringResult,
    parent_identity: &str,
    type_args: &[Type],
) -> Vec<(String, Type)> {
    let parent = result.module.classes.iter().find(|candidate| {
        canonical_parent_identity(module_name, candidate.identity.as_deref(), &candidate.name)
            == parent_identity
    });
    let bindings = parent
        .into_iter()
        .flat_map(|candidate| candidate.type_params.iter().cloned())
        .zip(type_args.iter().cloned())
        .collect::<HashMap<_, _>>();
    parent
        .into_iter()
        .flat_map(|parent| parent.fields.iter())
        .map(|(name, ty)| {
            (
                name.clone(),
                sifr_lowering::substitute_type_vars(ty, &bindings),
            )
        })
        .collect()
}

pub(super) fn parent_bindings(
    module_name: &str,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
    parent_identity: &str,
    type_args: &[Type],
) -> HashMap<String, Type> {
    let params = result
        .module
        .classes
        .iter()
        .find(|candidate| {
            canonical_parent_identity(module_name, candidate.identity.as_deref(), &candidate.name)
                == parent_identity
        })
        .into_iter()
        .flat_map(|candidate| candidate.type_params.iter())
        .cloned()
        .collect::<Vec<_>>();
    let params = (!params.is_empty())
        .then_some(params)
        .or_else(|| {
            let (module, owner) = parent_identity.rsplit_once('.')?;
            external_defs
                .class_type_params
                .get(module)
                .and_then(|classes| classes.get(owner))
                .cloned()
        })
        .unwrap_or_default();
    params.into_iter().zip(type_args.iter().cloned()).collect()
}

pub(super) fn canonical_parent_identity(
    module_name: &str,
    identity: Option<&str>,
    name: &str,
) -> String {
    identity
        .map(str::to_string)
        .unwrap_or_else(|| format!("{module_name}.{name}"))
}

fn parent_method_contracts(
    module_name: &str,
    class: &HirClass,
    result: &LoweringResult,
    parent_identity: &str,
) -> Vec<(String, FunctionType)> {
    if let Some(Type::Class { methods, .. }) = class.parent_type.as_ref() {
        if !methods.is_empty() {
            return methods.clone();
        }
    }
    result
        .module
        .classes
        .iter()
        .find(|parent| {
            canonical_parent_identity(module_name, parent.identity.as_deref(), &parent.name)
                == parent_identity
        })
        .into_iter()
        .flat_map(|parent| parent.methods.iter())
        .map(|method| {
            (
                method.name.clone(),
                FunctionType {
                    receiver: method.receiver,
                    params: method
                        .params
                        .iter()
                        .map(|parameter| {
                            (
                                parameter.name.clone(),
                                parameter.ty.clone(),
                                parameter.convention,
                            )
                        })
                        .collect(),
                    return_type: Box::new(method.return_type.clone()),
                },
            )
        })
        .collect()
}

fn inherited_method_item(
    name: &str,
    signature: &FunctionType,
    parent_identity: &str,
    origin: &ConstValue,
) -> ConstValue {
    ConstValue::Record(BTreeMap::from([
        ("order".to_string(), ConstValue::Integer(0.into())),
        ("kind".to_string(), ConstValue::String("method".to_string())),
        ("name".to_string(), ConstValue::String(name.to_string())),
        (
            "identity".to_string(),
            ConstValue::String(format!("{parent_identity}.{name}")),
        ),
        ("origin".to_string(), origin.clone()),
        ("annotation_origin".to_string(), ConstValue::None),
        ("value_origin".to_string(), ConstValue::None),
        ("return_origin".to_string(), origin.clone()),
        ("declared_type".to_string(), ConstValue::None),
        (
            "default_kind".to_string(),
            ConstValue::String("required".to_string()),
        ),
        ("default_value".to_string(), ConstValue::None),
        (
            "signature".to_string(),
            ConstValue::String(crate::canonical_types::function_identity(signature)),
        ),
        (
            "value_argument_origins".to_string(),
            ConstValue::List(Vec::new()),
        ),
        ("value_arguments".to_string(), ConstValue::List(Vec::new())),
        ("decorators".to_string(), ConstValue::List(Vec::new())),
        ("parameters".to_string(), ConstValue::List(Vec::new())),
    ]))
}

pub(super) fn parent_selection<'a>(
    module_name: &str,
    result: &'a LoweringResult,
    external_defs: &'a ExternalDefs,
    parent_identity: &str,
) -> Option<&'a ClassAdapterSelection> {
    if let Some(local_name) = result.module.classes.iter().find_map(|class| {
        let identity =
            canonical_parent_identity(module_name, class.identity.as_deref(), &class.name);
        (identity == parent_identity).then_some(class.name.as_str())
    }) {
        return result
            .class_adapter_selections
            .iter()
            .find(|selection| selection.owner == local_name);
    }
    let (module, owner) = parent_identity.rsplit_once('.')?;
    external_defs
        .class_adapter_selections
        .get(module)?
        .get(owner)
}

fn inherited_field_item(
    order: usize,
    name: &str,
    ty: &Type,
    parent_identity: &str,
    plan: Option<&AdapterFieldPlan>,
    origin: &ConstValue,
) -> Result<ConstValue, &'static str> {
    let (default_kind, default_value) = match plan.map(|plan| &plan.default) {
        Some(AdapterFieldDefault::Const(value)) => {
            ("const", crate::specialization_support::const_value(value)?)
        }
        Some(AdapterFieldDefault::Factory(_)) => ("factory", ConstValue::None),
        _ => ("required", ConstValue::None),
    };
    Ok(ConstValue::Record(BTreeMap::from([
        ("order".to_string(), ConstValue::Integer(order.into())),
        ("kind".to_string(), ConstValue::String("field".to_string())),
        ("name".to_string(), ConstValue::String(name.to_string())),
        (
            "identity".to_string(),
            ConstValue::String(plan.map_or_else(
                || format!("{parent_identity}.{name}"),
                |plan| plan.identity.clone(),
            )),
        ),
        ("origin".to_string(), origin.clone()),
        ("annotation_origin".to_string(), origin.clone()),
        ("value_origin".to_string(), ConstValue::None),
        ("return_origin".to_string(), ConstValue::None),
        (
            "declared_type".to_string(),
            ConstValue::String(crate::canonical_types::type_identity(ty)),
        ),
        (
            "default_kind".to_string(),
            ConstValue::String(default_kind.to_string()),
        ),
        ("default_value".to_string(), default_value),
        ("signature".to_string(), ConstValue::None),
        (
            "value_argument_origins".to_string(),
            ConstValue::List(Vec::new()),
        ),
        ("value_arguments".to_string(), ConstValue::List(Vec::new())),
        ("decorators".to_string(), ConstValue::List(Vec::new())),
        ("parameters".to_string(), ConstValue::List(Vec::new())),
    ])))
}

#[cfg(test)]
mod parent_selection_tests {
    use super::{
        canonical_parent_identity, parent_bindings, parent_method_contracts, parent_selection,
    };
    use sifr_lowering::{lower_module, ClassAdapterSelection, ExternalDefs, LoweringResult};
    use sifr_syntax::parse_module_suite;

    fn selection(owner: &str, provider_module: &str) -> ClassAdapterSelection {
        ClassAdapterSelection {
            owner: owner.to_string(),
            provider_module: provider_module.to_string(),
            provider_function: "adapt".to_string(),
            descriptor_type: sifr_type_system::Type::None,
            marker_identities: Vec::new(),
            data_parent: None,
            field_plans: Vec::new(),
            handler_plans: Vec::new(),
            attached_api_set: None,
            adapter_invocation_identity: [0; 32],
            post_adapter_identity: [0; 32],
            range: ruff_text_size::TextRange::default(),
        }
    }

    fn local_result() -> LoweringResult {
        let parsed = parse_module_suite("class Parent:\n    pass\n", None).expect("fixture parses");
        let mut result = lower_module(&parsed).expect("fixture lowers");
        result
            .class_adapter_selections
            .push(selection("Parent", "local.provider"));
        result
    }

    #[test]
    fn canonical_imported_parent_wins_over_a_colliding_local_selection() {
        let result = local_result();
        let mut external_defs = ExternalDefs::default();
        external_defs
            .class_adapter_selections
            .entry("models".to_string())
            .or_default()
            .insert("Parent".to_string(), selection("Parent", "models.provider"));

        let selected = parent_selection("consumer", &result, &external_defs, "models.Parent")
            .expect("imported selection exists");
        assert_eq!(selected.provider_module, "models.provider");
    }

    #[test]
    fn canonical_local_parent_keeps_the_local_selection() {
        let result = local_result();
        let external_defs = ExternalDefs::default();
        let selected = parent_selection("consumer", &result, &external_defs, "consumer.Parent")
            .expect("local selection exists");
        assert_eq!(selected.provider_module, "local.provider");
    }

    #[test]
    fn imported_parent_bindings_ignore_a_colliding_local_class() {
        let parsed =
            parse_module_suite("class Parent[LocalT]:\n    pass\n", None).expect("fixture parses");
        let result = lower_module(&parsed).expect("fixture lowers");
        let mut external_defs = ExternalDefs::default();
        external_defs
            .class_type_params
            .entry("models".to_string())
            .or_default()
            .insert("Parent".to_string(), vec!["RemoteT".to_string()]);

        let bindings = parent_bindings(
            "consumer",
            &result,
            &external_defs,
            "models.Parent",
            &[sifr_type_system::Type::Str],
        );
        assert_eq!(bindings.get("RemoteT"), Some(&sifr_type_system::Type::Str));
        assert!(!bindings.contains_key("LocalT"));
    }

    #[test]
    fn missing_parent_identity_is_canonicalized_to_the_current_module() {
        assert_eq!(
            canonical_parent_identity("consumer", None, "Parent"),
            "consumer.Parent"
        );
    }

    #[test]
    fn empty_imported_parent_methods_ignore_a_colliding_local_class() {
        let parsed = parse_module_suite(
            "class Parent:\n    def local_method(self) -> int:\n        return 1\n\nclass Child(Parent):\n    pass\n",
            None,
        )
        .expect("fixture parses");
        let result = lower_module(&parsed).expect("fixture lowers");
        let mut child = result
            .module
            .classes
            .iter()
            .find(|class| class.name == "Child")
            .cloned()
            .expect("child exists");
        let Some(sifr_type_system::Type::Class {
            identity, methods, ..
        }) = child.parent_type.as_mut()
        else {
            panic!("parent class type exists");
        };
        *identity = Some("models.Parent".to_string());
        methods.clear();

        assert!(parent_method_contracts("consumer", &child, &result, "models.Parent").is_empty());
    }
}
