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
#[path = "../tests/unit/rust_interop_bridge_panic_contract/tests.rs"]
mod tests;
