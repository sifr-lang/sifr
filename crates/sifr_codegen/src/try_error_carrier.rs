use sifr_ir::HirExceptHandler;
use sifr_type_system::{Type, make_union};

pub(crate) fn timeout_error_type() -> Type {
    Type::Class {
        identity: Some("sifr.builtin.TimeoutError".to_string()),
        type_args: Vec::new(),
        name: "TimeoutError".to_string(),
        fields: vec![("message".to_string(), Type::Str)].into(),
        methods: Vec::new().into(),
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
#[path = "../tests/unit/try_error_carrier/tests.rs"]
mod tests;
