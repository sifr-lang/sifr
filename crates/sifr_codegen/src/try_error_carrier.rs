use sifr_ir::HirExceptHandler;
use sifr_type_system::{Type, make_union};

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
    if types.is_empty() {
        None
    } else {
        Some(make_union(types.to_vec()))
    }
}

pub(crate) fn try_error_carrier(
    body_error_types: &[Type],
    handlers: &[HirExceptHandler],
) -> Option<Type> {
    let mut types = body_error_types.to_vec();
    if let Some(catch_all) = handlers
        .iter()
        .find(|handler| handler_is_catch_all(handler))
        .and_then(|handler| handler.error_resolved_type.clone())
    {
        types.push(catch_all);
    }
    exact_try_error_carrier(&types)
}

pub(crate) fn handler_is_catch_all(handler: &HirExceptHandler) -> bool {
    handler.error_type.is_none()
        || handler
            .error_resolved_type
            .as_ref()
            .is_some_and(Type::is_builtin_error_base)
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
        assert_ne!(
            members[0].union_variant_name(),
            members[1].union_variant_name()
        );
    }

    #[test]
    fn carrier_uses_the_canonical_union_member_order() {
        let named_error = |name: &str| Type::Class {
            identity: Some(format!("a.{name}")),
            type_args: Vec::new(),
            name: name.to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        };
        let os_error = named_error("OSError");
        let zero_division = named_error("ZeroDivisionError");
        let ordinary = make_union(vec![os_error.clone(), zero_division.clone()]);
        let carrier = exact_try_error_carrier(&[zero_division, os_error])
            .expect("carrier should contain both errors");

        assert_eq!(carrier, ordinary);
    }

    #[test]
    fn carrier_collapses_structural_snapshots_of_one_nominal_identity() {
        let first = error("sifr.io.IOError");
        let mut second = first.clone();
        let Type::Class { fields, .. } = &mut second else {
            unreachable!();
        };
        fields.push(("kind".to_string(), Type::Str));

        let forward = exact_try_error_carrier(&[first.clone(), second.clone()]);
        let reverse = exact_try_error_carrier(&[second.clone(), first]);
        assert_eq!(forward, reverse);
        assert_eq!(forward, Some(second));
    }

    #[test]
    fn catch_all_fallback_keeps_exact_error_variant() {
        let exact = error("sifr.tomllib.TOMLDecodeError");
        let mut fallback = error("builtins.Error");
        let Type::Class {
            identity,
            parent_class,
            ..
        } = &mut fallback
        else {
            unreachable!();
        };
        *identity = None;
        *parent_class = None;
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

    #[test]
    fn same_basename_import_is_not_a_catch_all() {
        let handler = HirExceptHandler {
            error_type: Some("Error".to_string()),
            error_resolved_type: Some(error("sifr.csv.Error")),
            name: None,
            body: Vec::new(),
        };

        assert!(!handler_is_catch_all(&handler));
    }
}
