use sifr_type_system::Type;

use crate::rust_interop_bridge_contract::RustBridgePanicErrorContract;

pub fn rust_bridge_panic_error_contract(return_type: &Type) -> RustBridgePanicErrorContract {
    let Type::Result(_, error_type) = return_type.resolve_alias() else {
        return RustBridgePanicErrorContract::None;
    };
    match error_type.resolve_alias() {
        error if is_rust_panic_error(error) => RustBridgePanicErrorContract::WrapperOnly,
        Type::Union(members)
            if members.len() == 2
                && members
                    .iter()
                    .filter(|member| is_rust_panic_error(member))
                    .count()
                    == 1 =>
        {
            RustBridgePanicErrorContract::OrdinaryAndWrapper
        }
        _ => RustBridgePanicErrorContract::None,
    }
}

pub(crate) fn recoverable_panic_bridge_error(
    error_type: &Type,
) -> Result<Option<&Type>, &'static str> {
    let Type::Union(members) = error_type.resolve_alias() else {
        return Ok(None);
    };
    let panic_count = members
        .iter()
        .filter(|member| is_rust_panic_error(member))
        .count();
    if panic_count == 0 {
        return Ok(None);
    }
    if panic_count != 1 || members.len() != 2 {
        return Err(
            "recoverable Rust panic Result errors require exactly one declared error and RustPanicError",
        );
    }
    Ok(members
        .iter()
        .find(|member| !is_rust_panic_error(member))
        .map(Some)
        .unwrap_or(None))
}

fn is_rust_panic_error(ty: &Type) -> bool {
    matches!(
        ty.resolve_alias(),
        Type::Class { name, .. } if name == "RustPanicError"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error(name: &str) -> Type {
        Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: name.to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        }
    }

    #[test]
    fn selects_the_non_panic_rust_bridge_error() {
        let mapped = error("PanicMapped");
        let errors = Type::Union(vec![mapped.clone(), error("RustPanicError")]);

        assert_eq!(recoverable_panic_bridge_error(&errors), Ok(Some(&mapped)));
    }

    #[test]
    fn rejects_ambiguous_recoverable_panic_unions() {
        let errors = Type::Union(vec![
            error("FirstError"),
            error("SecondError"),
            error("RustPanicError"),
        ]);

        assert!(recoverable_panic_bridge_error(&errors).is_err());
    }

    #[test]
    fn classifies_only_the_nominal_panic_error() {
        let similarly_named =
            Type::Result(Box::new(Type::Str), Box::new(error("RustPanicErrorish")));
        let panic_only = Type::Result(Box::new(Type::Str), Box::new(error("RustPanicError")));

        assert_eq!(
            rust_bridge_panic_error_contract(&similarly_named),
            RustBridgePanicErrorContract::None
        );
        assert_eq!(
            rust_bridge_panic_error_contract(&panic_only),
            RustBridgePanicErrorContract::WrapperOnly
        );
    }

    #[test]
    fn classifies_an_alias_of_the_ordinary_and_wrapper_union() {
        let alias = Type::Alias {
            name: "PanicSurface".to_string(),
            type_args: Vec::new(),
            body: Box::new(Type::Union(vec![
                error("PanicMapped"),
                error("RustPanicError"),
            ])),
        };
        let result = Type::Result(Box::new(Type::Str), Box::new(alias));

        assert_eq!(
            rust_bridge_panic_error_contract(&result),
            RustBridgePanicErrorContract::OrdinaryAndWrapper
        );
    }
}
