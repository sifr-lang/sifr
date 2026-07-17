use sifr_ir::HirExceptHandler;
use sifr_type_system::Type;

pub(crate) fn timeout_error_type() -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "TimeoutError".to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    }
}

pub(crate) fn exact_try_error_carrier(types: &[Type]) -> Option<Type> {
    let mut members = types.to_vec();
    members.sort_by_key(Type::union_variant_name);
    members.dedup_by(|left, right| left.union_variant_name() == right.union_variant_name());
    match members.len() {
        0 => None,
        1 => members.pop(),
        _ => Some(Type::Union(members)),
    }
}

pub(crate) fn try_error_carrier(
    body_error_types: &[Type],
    handlers: &[HirExceptHandler],
) -> Option<Type> {
    let mut types = body_error_types.to_vec();
    if let Some(catch_all) = handlers
        .iter()
        .find(|handler| {
            handler.error_type.is_none() || handler.error_type.as_deref() == Some("Error")
        })
        .and_then(|handler| handler.error_resolved_type.clone())
    {
        types.push(catch_all);
    }
    exact_try_error_carrier(&types)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error(identity: &str) -> Type {
        Type::Class {
            identity: Some(identity.to_string()),
            type_args: Vec::new(),
            name: "Error".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        }
    }

    #[test]
    fn carrier_is_deterministic_and_keeps_same_basename_identities_distinct() {
        let csv = error("sifr.csv.Error");
        let config = error("sifr.configparser.Error");
        let carrier = exact_try_error_carrier(&[csv.clone(), config.clone(), csv])
            .expect("carrier should exist");
        let Type::Union(members) = carrier else {
            panic!("distinct exact errors should form a union carrier");
        };
        assert_eq!(members.len(), 2);
        assert_ne!(members[0].rust_type(), members[1].rust_type());
    }

    #[test]
    fn carrier_collapses_structural_snapshots_of_one_nominal_identity() {
        let first = error("sifr.io.IOError");
        let mut second = first.clone();
        let Type::Class { fields, .. } = &mut second else {
            unreachable!();
        };
        fields.push(("kind".to_string(), Type::Str));

        assert_eq!(
            exact_try_error_carrier(&[first.clone(), second]),
            Some(first)
        );
    }

    #[test]
    fn catch_all_fallback_keeps_exact_error_variant() {
        let exact = error("sifr.tomllib.TOMLDecodeError");
        let fallback = error("builtins.Error");
        let handlers = vec![HirExceptHandler {
            error_type: Some("Error".to_string()),
            error_resolved_type: Some(fallback),
            name: None,
            body: Vec::new(),
        }];

        let carrier = try_error_carrier(&[exact], &handlers).expect("carrier should exist");
        let Type::Union(members) = carrier else {
            panic!("exact and fallback errors should remain distinct");
        };
        assert_eq!(members.len(), 2);
    }
}
