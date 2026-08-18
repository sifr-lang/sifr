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
    let Some(parent_name) = selection.data_parent.as_deref() else {
        return Ok(value);
    };
    let local_field_count = items
        .iter()
        .filter(|item| {
            matches!(
                item,
                ConstValue::Record(fields)
                    if matches!(fields.get("kind"), Some(ConstValue::String(kind)) if kind == "field")
            )
        })
        .count();
    let (parent_identity, mut parent_fields) = if let Some(Type::Class {
        identity,
        name,
        fields,
        type_args,
        ..
    }) = class.parent_type.as_ref()
    {
        let fields = if fields.is_empty() {
            local_parent_fields(result, parent_name, type_args)
        } else {
            fields.clone()
        };
        (identity.as_deref().unwrap_or(name).to_string(), fields)
    } else {
        let inherited_count = class.fields.len().saturating_sub(local_field_count);
        (
            format!("{module_name}.{parent_name}"),
            class.fields.iter().take(inherited_count).cloned().collect(),
        )
    };
    let parent_selection = parent_selection(result, external_defs, &parent_identity, parent_name);
    if let Some(parent) = parent_selection.filter(|parent| !parent.field_plans.is_empty()) {
        let type_args = match class.parent_type.as_ref() {
            Some(Type::Class { type_args, .. }) => type_args.as_slice(),
            _ => &[],
        };
        let bindings = parent_bindings(
            result,
            external_defs,
            &parent_identity,
            parent_name,
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
        parent_method_contracts(class, result, parent_name)
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
    result: &LoweringResult,
    parent_name: &str,
    type_args: &[Type],
) -> Vec<(String, Type)> {
    let bindings = result
        .module
        .classes
        .iter()
        .find(|candidate| candidate.name == parent_name)
        .into_iter()
        .flat_map(|candidate| candidate.type_params.iter())
        .cloned()
        .zip(type_args.iter().cloned())
        .collect::<HashMap<_, _>>();
    result
        .module
        .classes
        .iter()
        .find(|parent| parent.name == parent_name)
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
    result: &LoweringResult,
    external_defs: &ExternalDefs,
    parent_identity: &str,
    parent_name: &str,
    type_args: &[Type],
) -> HashMap<String, Type> {
    let mut params = result
        .module
        .classes
        .iter()
        .find(|candidate| candidate.name == parent_name)
        .into_iter()
        .flat_map(|candidate| candidate.type_params.iter())
        .cloned()
        .collect::<Vec<_>>();
    if params.is_empty() {
        if let Some((module, owner)) = parent_identity.rsplit_once('.') {
            params = external_defs
                .class_type_params
                .get(module)
                .and_then(|classes| classes.get(owner))
                .cloned()
                .unwrap_or_default();
        }
    }
    params.into_iter().zip(type_args.iter().cloned()).collect()
}

fn parent_method_contracts(
    class: &HirClass,
    result: &LoweringResult,
    parent_name: &str,
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
        .find(|parent| parent.name == parent_name)
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
    result: &'a LoweringResult,
    external_defs: &'a ExternalDefs,
    parent_identity: &str,
    parent_name: &str,
) -> Option<&'a ClassAdapterSelection> {
    if let Some(parent) = result
        .class_adapter_selections
        .iter()
        .find(|selection| selection.owner == parent_name)
    {
        return Some(parent);
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
