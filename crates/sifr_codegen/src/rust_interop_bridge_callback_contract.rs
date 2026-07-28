use crate::rust_interop_bridge_contract::generated_types::GeneratedTypeCollector;
use crate::rust_interop_bridge_contract::{
    bridge_type_contract, unsupported_type, BridgeTypePosition, CallbackParameterMode,
    ModuleCatalog, RustBridgeTypeContract, RustBridgeTypeKind,
};
use sifr_type_system::{ParamConvention, Type};
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
pub(crate) struct CallbackSignature<'a> {
    pub(crate) callable: &'a Type,
    pub(crate) params: &'a [Type],
    pub(crate) conventions: &'a [ParamConvention],
    pub(crate) result: &'a Type,
}

pub(crate) fn bridge_call_scoped_callback_type(
    signature: CallbackSignature<'_>,
    module_name: Option<&String>,
    module_catalogs: &BTreeMap<Option<String>, ModuleCatalog>,
    catalog: Option<&ModuleCatalog>,
    generated_types: &mut GeneratedTypeCollector,
) -> RustBridgeTypeContract {
    if signature
        .conventions
        .iter()
        .any(|convention| convention.is_mut_borrow())
    {
        return RustBridgeTypeContract {
            sifr_type: signature.callable.display_name(),
            rust_borrowed_type: None,
            rust_owned_type: None,
            rust_return_type: None,
            kind: RustBridgeTypeKind::CallScopedCallback,
            unsupported_reason: Some(
                "call-scoped callback arguments do not support mutable-borrow conventions"
                    .to_string(),
            ),
        };
    }
    let Ok((tuple, result_type)) = callback_rust_types(
        signature,
        module_name,
        module_catalogs,
        catalog,
        generated_types,
    ) else {
        return unsupported_type(
            signature.callable,
            "call-scoped callback parameter or result type is not Rust bridge-compatible",
        );
    };
    let rust_type =
        format!("::sifr_runtime::interop::CallScopedCallbackBridge<'_, {tuple}, {result_type}>");
    RustBridgeTypeContract {
        sifr_type: signature.callable.display_name(),
        rust_borrowed_type: Some(rust_type.clone()),
        rust_owned_type: Some(rust_type),
        rust_return_type: None,
        kind: RustBridgeTypeKind::CallScopedCallback,
        unsupported_reason: None,
    }
}

pub(crate) fn bridge_threadsafe_callback_type(
    signature: CallbackSignature<'_>,
    module_name: Option<&String>,
    module_catalogs: &BTreeMap<Option<String>, ModuleCatalog>,
    catalog: Option<&ModuleCatalog>,
    generated_types: &mut GeneratedTypeCollector,
) -> RustBridgeTypeContract {
    if signature
        .conventions
        .iter()
        .any(|convention| convention.is_mut_borrow())
    {
        return unsupported_type(
            signature.callable,
            "thread-safe callback arguments do not support mutable-borrow conventions",
        );
    }
    let Ok((tuple, result_type)) = callback_rust_types(
        signature,
        module_name,
        module_catalogs,
        catalog,
        generated_types,
    ) else {
        return unsupported_type(
            signature.callable,
            "thread-safe callback parameter or result type is not Rust bridge-compatible",
        );
    };
    let rust_type =
        format!("::sifr_runtime::interop::ThreadsafeCallbackBridge<{tuple}, {result_type}>");
    RustBridgeTypeContract {
        sifr_type: signature.callable.display_name(),
        rust_borrowed_type: Some(format!("&{rust_type}")),
        rust_owned_type: Some(rust_type),
        rust_return_type: None,
        kind: RustBridgeTypeKind::Callback,
        unsupported_reason: None,
    }
}

fn callback_rust_types(
    signature: CallbackSignature<'_>,
    module_name: Option<&String>,
    module_catalogs: &BTreeMap<Option<String>, ModuleCatalog>,
    catalog: Option<&ModuleCatalog>,
    generated_types: &mut GeneratedTypeCollector,
) -> Result<(String, String), ()> {
    let param_types = signature
        .params
        .iter()
        .map(|param| {
            bridge_type_contract(
                param,
                module_name,
                module_catalogs,
                catalog,
                generated_types,
                BridgeTypePosition::Parameter(CallbackParameterMode::Nested),
            )
            .rust_owned_type
        })
        .collect::<Option<Vec<_>>>()
        .ok_or(())?;
    let result_type = if let Type::Result(ok, _) = signature.result.resolve_alias() {
        let ok_contract = bridge_type_contract(
            ok,
            module_name,
            module_catalogs,
            catalog,
            generated_types,
            BridgeTypePosition::Return,
        );
        let ok_type = ok_contract.rust_return_type.ok_or(())?;
        format!("Result<{ok_type}, String>")
    } else {
        let result_contract = bridge_type_contract(
            signature.result,
            module_name,
            module_catalogs,
            catalog,
            generated_types,
            BridgeTypePosition::Return,
        );
        result_contract.rust_return_type.ok_or(())?
    };
    let tuple = match param_types.as_slice() {
        [] => "()".to_string(),
        [only] => format!("({only},)"),
        many => format!("({})", many.join(", ")),
    };
    Ok((tuple, result_type))
}
