use sifr_ir::{RustInteropDeclaration, RustInteropValue};

pub(super) fn panic_policy(declaration: &RustInteropDeclaration) -> Option<String> {
    declaration.arguments.iter().find_map(|argument| {
        if argument.name.as_deref() != Some("panic") {
            return None;
        }
        match &argument.value {
            RustInteropValue::Symbol(policy) => Some(policy.clone()),
            _ => None,
        }
    })
}

pub(super) fn build_env_trust_entries(declaration: &RustInteropDeclaration) -> Vec<String> {
    declaration
        .arguments
        .iter()
        .filter_map(|argument| {
            if argument.name.as_deref() != Some("build_env") {
                return None;
            }
            match &argument.value {
                RustInteropValue::Symbol(name) => Some(name.clone()),
                _ => None,
            }
        })
        .collect()
}
