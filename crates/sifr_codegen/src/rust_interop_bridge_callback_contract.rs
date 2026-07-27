use crate::rust_interop_bridge_contract::generated_types::GeneratedTypeCollector;
use crate::rust_interop_bridge_contract::{
    bridge_type_contract, unsupported_type, BridgeTypePosition, CallbackParameterMode,
    ModuleCatalog, RustBridgeTypeContract, RustBridgeTypeKind,
};
use sifr_type_system::{ParamConvention, Type};
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
pub(crate) struct CallScopedCallbackSignature<'a> {
    pub(crate) callable: &'a Type,
    pub(crate) params: &'a [Type],
    pub(crate) conventions: &'a [ParamConvention],
    pub(crate) result: &'a Type,
}

pub(crate) fn bridge_call_scoped_callback_type(
    signature: CallScopedCallbackSignature<'_>,
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
    let Some(param_types) = signature
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
    else {
        return unsupported_type(
            signature.callable,
            "call-scoped callback parameter type is not Rust bridge-compatible",
        );
    };
    let result_type = if let Type::Result(ok, _) = signature.result.resolve_alias() {
        let ok_contract = bridge_type_contract(
            ok,
            module_name,
            module_catalogs,
            catalog,
            generated_types,
            BridgeTypePosition::Return,
        );
        let Some(ok_type) = ok_contract.rust_return_type else {
            return unsupported_type(
                signature.callable,
                "call-scoped callback success type is not Rust bridge-compatible",
            );
        };
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
        let Some(result_type) = result_contract.rust_return_type else {
            return unsupported_type(
                signature.callable,
                "call-scoped callback result type is not Rust bridge-compatible",
            );
        };
        result_type
    };
    let tuple = match param_types.as_slice() {
        [] => "()".to_string(),
        [only] => format!("({only},)"),
        many => format!("({})", many.join(", ")),
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
