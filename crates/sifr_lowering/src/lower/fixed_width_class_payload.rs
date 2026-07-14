use sifr_type_system::Type;

fn type_mentions_class(ty: &Type, class_name: &str) -> bool {
    match ty.resolve_alias() {
        Type::Class { name, fields, .. } => {
            name == class_name
                || fields
                    .iter()
                    .any(|(_, field_ty)| type_mentions_class(field_ty, class_name))
        }
        Type::List(elem)
        | Type::Set(elem)
        | Type::Iterable(elem)
        | Type::Iterator(elem)
        | Type::Alias { body: elem, .. }
        | Type::Newtype { inner: elem, .. } => type_mentions_class(elem, class_name),
        Type::Dict(key, value) | Type::Result(key, value) => {
            type_mentions_class(key, class_name) || type_mentions_class(value, class_name)
        }
        Type::Tuple(elems) | Type::Union(elems) | Type::Intersection(elems) => elems
            .iter()
            .any(|elem| type_mentions_class(elem, class_name)),
        Type::Callable(params, _, ret) | Type::AsyncCallable(params, _, ret) => {
            params
                .iter()
                .any(|param| type_mentions_class(param, class_name))
                || type_mentions_class(ret, class_name)
        }
        Type::Function(ft) => {
            ft.params
                .iter()
                .any(|(_, param_ty, _)| type_mentions_class(param_ty, class_name))
                || type_mentions_class(&ft.return_type, class_name)
        }
        Type::Protocol { methods, .. } => methods.iter().any(|(_, ft)| {
            ft.params
                .iter()
                .any(|(_, param_ty, _)| type_mentions_class(param_ty, class_name))
                || type_mentions_class(&ft.return_type, class_name)
        }),
        _ => false,
    }
}

fn type_contains_fixed_width(ty: &Type) -> bool {
    match ty.resolve_alias() {
        Type::FixedInt(_) => true,
        Type::Class {
            fields, methods, ..
        } => {
            fields
                .iter()
                .any(|(_, field_ty)| type_contains_fixed_width(field_ty))
                || methods.iter().any(|(_, ft)| {
                    ft.params
                        .iter()
                        .any(|(_, param_ty, _)| type_contains_fixed_width(param_ty))
                        || type_contains_fixed_width(&ft.return_type)
                })
        }
        Type::List(elem)
        | Type::Set(elem)
        | Type::Iterable(elem)
        | Type::Iterator(elem)
        | Type::Alias { body: elem, .. }
        | Type::Newtype { inner: elem, .. } => type_contains_fixed_width(elem),
        Type::Dict(key, value) | Type::Result(key, value) => {
            type_contains_fixed_width(key) || type_contains_fixed_width(value)
        }
        Type::Tuple(elems) | Type::Union(elems) | Type::Intersection(elems) => {
            elems.iter().any(type_contains_fixed_width)
        }
        Type::Callable(params, _, ret) | Type::AsyncCallable(params, _, ret) => {
            params.iter().any(type_contains_fixed_width) || type_contains_fixed_width(ret)
        }
        Type::Function(ft) => {
            ft.params
                .iter()
                .any(|(_, param_ty, _)| type_contains_fixed_width(param_ty))
                || type_contains_fixed_width(&ft.return_type)
        }
        Type::Protocol { methods, .. } => methods.iter().any(|(_, ft)| {
            ft.params
                .iter()
                .any(|(_, param_ty, _)| type_contains_fixed_width(param_ty))
                || type_contains_fixed_width(&ft.return_type)
        }),
        _ => false,
    }
}

fn class_payload_contains_fixed_width(fields: &[(String, Type)]) -> bool {
    fields
        .iter()
        .any(|(_, field_ty)| type_contains_fixed_width(field_ty))
}

pub(in crate::lower) fn class_specialization_payload_conflicts(
    source: &Type,
    target: &Type,
) -> bool {
    let (
        Type::Class {
            name: source_name,
            fields: source_fields,
            ..
        },
        Type::Class {
            name: target_name,
            fields: target_fields,
            ..
        },
    ) = (source.resolve_alias(), target.resolve_alias())
    else {
        return false;
    };
    if source_name != target_name || source_fields.len() != target_fields.len() {
        return false;
    }
    if source_fields
        .iter()
        .any(|(_, field_ty)| type_mentions_class(field_ty, source_name))
        || target_fields
            .iter()
            .any(|(_, field_ty)| type_mentions_class(field_ty, target_name))
    {
        return false;
    }
    if !class_payload_contains_fixed_width(source_fields)
        && !class_payload_contains_fixed_width(target_fields)
    {
        return false;
    }
    source_fields.iter().any(|(source_field, source_ty)| {
        target_fields
            .iter()
            .find(|(target_field, _)| target_field == source_field)
            .is_some_and(|(_, target_ty)| !source_ty.is_assignable_to(target_ty))
    })
}
