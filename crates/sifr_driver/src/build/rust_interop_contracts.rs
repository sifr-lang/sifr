use sifr_codegen::{
    RustBridgeParamConvention, RustBridgeSignatureContract, RustBridgeTypeContract,
};

pub(super) struct RustBridgeContractDiagnostic {
    pub(super) signature: RustBridgeSignatureContract,
    pub(super) args: Vec<(&'static str, String)>,
    pub(super) notes: Vec<String>,
}

pub(super) fn bridge_contract_diagnostics(
    signatures: &[RustBridgeSignatureContract],
) -> Vec<RustBridgeContractDiagnostic> {
    let mut diagnostics = Vec::new();
    for signature in signatures {
        for param in &signature.params {
            let selected = rust_param_type(param.convention, &param.ty);
            if let Some(reason) = unsupported_bridge_type_reason(&param.ty, selected.as_ref()) {
                diagnostics.push(RustBridgeContractDiagnostic {
                    signature: signature.clone(),
                    args: vec![
                        ("sifr_type", param.ty.sifr_type.clone()),
                        ("target", signature.canonical_target_path.clone()),
                        ("role", format!("parameter `{}`", param.name)),
                        ("reason", reason),
                    ],
                    notes: vec![format!(
                        "parameter `{}` is not bridge-compatible",
                        param.name
                    )],
                });
            }
        }
        if let Some(reason) = unsupported_bridge_type_reason(
            &signature.return_type,
            signature.return_type.rust_return_type.as_ref(),
        ) {
            diagnostics.push(RustBridgeContractDiagnostic {
                signature: signature.clone(),
                args: vec![
                    ("sifr_type", signature.return_type.sifr_type.clone()),
                    ("target", signature.canonical_target_path.clone()),
                    ("role", "return type".to_string()),
                    ("reason", reason),
                ],
                notes: vec!["return type is not bridge-compatible".to_string()],
            });
        }
    }
    diagnostics
}

fn rust_param_type(
    convention: RustBridgeParamConvention,
    ty: &RustBridgeTypeContract,
) -> Option<String> {
    match convention {
        RustBridgeParamConvention::Borrow => ty.rust_borrowed_type.clone(),
        RustBridgeParamConvention::MutableBorrow => {
            ty.rust_borrowed_type.as_deref().map(mutable_borrow_type)
        }
        RustBridgeParamConvention::Own => ty.rust_owned_type.clone(),
    }
}

fn mutable_borrow_type(rust_type: &str) -> String {
    rust_type.strip_prefix('&').map_or_else(
        || rust_type.to_string(),
        |inner| format!("&mut {}", inner.trim_start()),
    )
}

fn unsupported_bridge_type_reason(
    ty: &RustBridgeTypeContract,
    selected_rust_type: Option<&String>,
) -> Option<String> {
    ty.unsupported_reason.clone().or_else(|| {
        selected_rust_type
            .is_none()
            .then(|| "missing Rust bridge representation".to_string())
    })
}
