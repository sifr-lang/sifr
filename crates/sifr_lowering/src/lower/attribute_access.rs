use sifr_type_system::{make_union, Type};

pub(in crate::lower) fn optional_class_union_field_type(
    resolved_object_ty: &Type,
    field_name: &str,
) -> Option<Type> {
    let Type::Union(members) = resolved_object_ty else {
        return None;
    };
    let mut has_none = false;
    let mut field_candidates = Vec::new();
    for member in members {
        match member.resolve_alias() {
            Type::None => {
                has_none = true;
            }
            Type::Class { fields, .. } => {
                if let Some((_, field_ty)) = fields.iter().find(|(n, _)| n == field_name) {
                    field_candidates.push(field_ty.clone());
                } else {
                    return None;
                }
            }
            _ => {
                field_candidates.clear();
                break;
            }
        }
    }
    if field_candidates.is_empty() {
        return None;
    }
    if has_none {
        field_candidates.push(Type::None);
    }
    Some(make_union(field_candidates))
}
