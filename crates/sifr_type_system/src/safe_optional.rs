use crate::{make_union, Type};

/// Return the result type of an operation that can produce no value.
///
/// A multi-member payload remains inside a representation-significant outer
/// optional wrapper. A simple optional payload is flattened because both
/// absence layers have the same observable Sifr value.
#[must_use]
pub fn safe_optional_result(payload: Type) -> Type {
    match payload.resolve_alias() {
        Type::Union(members) if members.len() > 1 && payload.optional_member_type().is_none() => {
            Type::Union(vec![payload, Type::None])
        }
        _ => make_union(vec![payload, Type::None]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_multi_member_payload_inside_outer_optional() {
        let payload = make_union(vec![Type::Int, Type::Str]);
        assert_eq!(
            safe_optional_result(payload.clone()),
            Type::Union(vec![payload, Type::None])
        );
    }

    #[test]
    fn preserves_nullable_multi_member_payload_inside_outer_optional() {
        let payload = make_union(vec![Type::Int, Type::Str, Type::None]);
        assert_eq!(
            safe_optional_result(payload.clone()),
            Type::Union(vec![payload, Type::None])
        );
    }

    #[test]
    fn flattens_simple_optional_payload() {
        let payload = make_union(vec![Type::Str, Type::None]);
        assert_eq!(safe_optional_result(payload.clone()), payload);
    }

    #[test]
    fn canonicalizes_non_union_payload() {
        assert_eq!(
            safe_optional_result(Type::Int),
            make_union(vec![Type::Int, Type::None])
        );
    }
}
