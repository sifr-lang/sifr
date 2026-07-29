use super::Type;

pub(super) fn option_member_type(ty: &Type) -> Option<Type> {
    let Type::Union(members) = ty.resolve_alias() else {
        return None;
    };
    let has_none = members
        .iter()
        .any(|member| matches!(member.resolve_alias(), Type::None));
    let non_none: Vec<Type> = members
        .iter()
        .filter(|member| !matches!(member.resolve_alias(), Type::None))
        .cloned()
        .collect();
    if has_none && non_none.len() == 1 {
        non_none.first().cloned()
    } else {
        None
    }
}
