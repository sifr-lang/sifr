use sifr_ir::HirClass;
use sifr_type_system::Type;
use std::collections::HashMap;

pub(crate) struct StructuralRecordField<'a> {
    pub(crate) name: &'a str,
    pub(crate) ty: &'a Type,
    pub(crate) inherited: bool,
}

pub(crate) fn structural_record_fields(class: &HirClass) -> Vec<StructuralRecordField<'_>> {
    let mut fields = class
        .parent_type
        .as_ref()
        .and_then(|parent| match parent.resolve_alias() {
            Type::Class { fields, .. } => Some(fields.as_slice()),
            _ => None,
        })
        .unwrap_or_default()
        .iter()
        .map(|(name, ty)| StructuralRecordField {
            name,
            ty,
            inherited: true,
        })
        .collect::<Vec<_>>();
    fields.extend(class.fields.iter().map(|(name, ty)| StructuralRecordField {
        name,
        ty,
        inherited: false,
    }));
    fields
}

pub(crate) fn concrete_record_fields(class: &HirClass) -> Vec<(String, Type)> {
    structural_record_fields(class)
        .into_iter()
        .map(|field| (field.name.to_string(), field.ty.clone()))
        .collect()
}

pub(crate) fn substitute_structural_type(ty: &Type, bindings: &HashMap<String, Type>) -> Type {
    match ty {
        Type::TypeVar(name) => bindings.get(name).cloned().unwrap_or_else(|| ty.clone()),
        Type::List(value) => Type::List(Box::new(substitute_structural_type(value, bindings))),
        Type::Set(value) => Type::Set(Box::new(substitute_structural_type(value, bindings))),
        Type::Dict(key, value) => Type::Dict(
            Box::new(substitute_structural_type(key, bindings)),
            Box::new(substitute_structural_type(value, bindings)),
        ),
        Type::Tuple(values) => Type::Tuple(
            values
                .iter()
                .map(|value| substitute_structural_type(value, bindings))
                .collect(),
        ),
        Type::Union(values) => Type::Union(
            values
                .iter()
                .map(|value| substitute_structural_type(value, bindings))
                .collect(),
        ),
        Type::Class {
            identity,
            type_args,
            name,
            fields,
            methods,
            parent_class,
        } => Type::Class {
            identity: identity.clone(),
            type_args: type_args
                .iter()
                .map(|value| substitute_structural_type(value, bindings))
                .collect(),
            name: name.clone(),
            fields: fields
                .iter()
                .map(|(name, value)| (name.clone(), substitute_structural_type(value, bindings)))
                .collect(),
            methods: methods.clone(),
            parent_class: parent_class.clone(),
        },
        Type::Alias {
            name,
            type_args,
            body,
        } => Type::Alias {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(|value| substitute_structural_type(value, bindings))
                .collect(),
            body: Box::new(substitute_structural_type(body, bindings)),
        },
        other => other.clone(),
    }
}
