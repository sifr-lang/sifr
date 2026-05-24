use sifr_type_system::Type;

/// Shape compatibility used when generic inference leaves unresolved `TypeVar`s.
/// `TypeVar`s are treated as wildcards, but container/class structure must still match.
pub(in crate::lower) fn is_compatible_with_unresolved_typevars(
    source: &Type,
    target: &Type,
) -> bool {
    match target {
        Type::TypeVar(_) => true,
        Type::List(target_elem) => match source {
            Type::List(source_elem) => {
                is_compatible_with_unresolved_typevars(source_elem, target_elem)
            }
            _ => false,
        },
        Type::Set(target_elem) => match source {
            Type::Set(source_elem) => {
                is_compatible_with_unresolved_typevars(source_elem, target_elem)
            }
            _ => false,
        },
        Type::Dict(target_key, target_val) => match source {
            Type::Dict(source_key, source_val) => {
                is_compatible_with_unresolved_typevars(source_key, target_key)
                    && is_compatible_with_unresolved_typevars(source_val, target_val)
            }
            _ => false,
        },
        Type::Tuple(target_elems) => match source {
            Type::Tuple(source_elems) => {
                source_elems.len() == target_elems.len()
                    && source_elems
                        .iter()
                        .zip(target_elems.iter())
                        .all(|(src, dst)| is_compatible_with_unresolved_typevars(src, dst))
            }
            _ => false,
        },
        Type::Result(target_ok, target_err) => match source {
            Type::Result(source_ok, source_err) => {
                is_compatible_with_unresolved_typevars(source_ok, target_ok)
                    && is_compatible_with_unresolved_typevars(source_err, target_err)
            }
            _ => false,
        },
        Type::Class {
            name: target_name, ..
        } => match source {
            Type::Class {
                name: source_name, ..
            } => source_name == target_name,
            _ => false,
        },
        Type::Union(target_members) => target_members
            .iter()
            .any(|member| is_compatible_with_unresolved_typevars(source, member)),
        _ => source.is_assignable_to(target),
    }
}
