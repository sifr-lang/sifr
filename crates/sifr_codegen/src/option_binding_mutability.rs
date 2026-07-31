use sifr_type_system::Type;
use std::collections::{HashMap, HashSet};

pub(crate) fn option_binding_requires_mut(
    option_var: &str,
    mutated_vars: &HashSet<String>,
    borrowed_params: &HashSet<String>,
    mut_borrowed_params: &HashSet<String>,
    local_binding_types: &HashMap<String, Type>,
    recursive_fields: &HashSet<(String, String)>,
) -> bool {
    if borrowed_params.contains(option_var) || mut_borrowed_params.contains(option_var) {
        return false;
    }
    if mutated_vars.contains(option_var) {
        return true;
    }
    let Some(option_ty) = local_binding_types.get(option_var) else {
        return false;
    };
    let Some(inner_ty) = option_inner_type(option_ty) else {
        return false;
    };
    let Type::Class { name, .. } = crate::resolve_alias_type_for_plain_call(inner_ty) else {
        return false;
    };
    recursive_fields
        .iter()
        .any(|(class_name, _)| class_name == name)
}

fn option_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Union(members) = crate::resolve_alias_type_for_plain_call(ty) else {
        return None;
    };
    let mut non_none = members
        .iter()
        .filter(|member| !matches!(crate::resolve_alias_type_for_plain_call(member), Type::None));
    let inner = non_none.next()?;
    if non_none.next().is_some()
        || !members
            .iter()
            .any(|member| matches!(crate::resolve_alias_type_for_plain_call(member), Type::None))
    {
        return None;
    }
    Some(inner)
}
