use crate::rust_interop_bridge_contract::{
    RustBridgeContractPlan, RustBridgeParamConvention, RustBridgeSignatureContract,
    RustBridgeTypeContract, RustBridgeTypeKind, RustGeneratedBridgeType,
    RustGeneratedBridgeTypeKind,
};

pub(crate) fn push_bridge_contract_plan(out: &mut String, plan: &RustBridgeContractPlan) {
    out.push_str("rust.bridge_signatures=");
    out.push_str(&plan.signatures.len().to_string());
    out.push('\n');
    for signature in &plan.signatures {
        push_signature(out, signature);
        out.push('\n');
    }
    out.push_str("rust.generated_bridge_types=");
    out.push_str(&plan.generated_types.len().to_string());
    out.push('\n');
    for bridge_type in &plan.generated_types {
        push_generated_type(out, bridge_type);
        out.push('\n');
    }
}

fn push_signature(out: &mut String, signature: &RustBridgeSignatureContract) {
    out.push_str("bridge-signature=");
    out.push_str(&signature.canonical_target_path);
    out.push_str("|module=");
    out.push_str(signature.module_name.as_deref().unwrap_or("<single>"));
    out.push_str("|params=");
    out.push_str(&signature.params.len().to_string());
    for param in &signature.params {
        out.push('|');
        out.push_str(&param.name);
        out.push(':');
        out.push_str(match param.convention {
            RustBridgeParamConvention::Borrow => "borrow",
            RustBridgeParamConvention::MutableBorrow => "mut-borrow",
            RustBridgeParamConvention::Own => "own",
            RustBridgeParamConvention::OwnMutable => "own-mut",
        });
        out.push(':');
        push_type_contract(out, &param.ty);
    }
    out.push_str("|return=");
    push_type_contract(out, &signature.return_type);
    out.push_str("|structural=");
    out.push_str(
        signature
            .structural_type_param
            .as_deref()
            .unwrap_or("<none>"),
    );
    out.push_str("|static-program=");
    out.push_str(if signature.static_program_type_param {
        "required"
    } else {
        "absent"
    });
    out.push_str("|panic-error=");
    out.push_str(match signature.panic_error {
        crate::rust_interop_bridge_contract::RustBridgePanicErrorContract::None => "none",
        crate::rust_interop_bridge_contract::RustBridgePanicErrorContract::WrapperOnly => {
            "wrapper-only"
        }
        crate::rust_interop_bridge_contract::RustBridgePanicErrorContract::OrdinaryAndWrapper => {
            "ordinary-and-wrapper"
        }
    });
}

fn push_type_contract(out: &mut String, ty: &RustBridgeTypeContract) {
    out.push_str(&ty.sifr_type);
    out.push_str("=>");
    out.push_str(ty.rust_return_type.as_deref().unwrap_or("<unsupported>"));
    out.push(':');
    out.push_str(match ty.kind {
        RustBridgeTypeKind::Bool => "bool",
        RustBridgeTypeKind::FixedInt => "fixed-int",
        RustBridgeTypeKind::ExactInt => "exact-int",
        RustBridgeTypeKind::Float64 => "float64",
        RustBridgeTypeKind::String => "string",
        RustBridgeTypeKind::Bytes => "bytes",
        RustBridgeTypeKind::List => "list",
        RustBridgeTypeKind::Dict => "dict",
        RustBridgeTypeKind::Option => "option",
        RustBridgeTypeKind::Tuple => "tuple",
        RustBridgeTypeKind::Result => "result",
        RustBridgeTypeKind::GeneratedRecord => "record",
        RustBridgeTypeKind::GeneratedEnum => "enum",
        RustBridgeTypeKind::GeneratedError => "error",
        RustBridgeTypeKind::OpaqueHandle => "handle",
        RustBridgeTypeKind::Callback => "callback",
        RustBridgeTypeKind::CallScopedCallback => "call-scoped-callback",
        RustBridgeTypeKind::StructuralTypeParam => "structural-type-param",
        RustBridgeTypeKind::None => "none",
        RustBridgeTypeKind::Unsupported => "unsupported",
    });
    if let Some(reason) = &ty.unsupported_reason {
        out.push_str(":unsupported=");
        out.push_str(reason);
    }
}

fn push_generated_type(out: &mut String, bridge_type: &RustGeneratedBridgeType) {
    out.push_str("bridge-type=");
    out.push_str(bridge_type.module_name.as_deref().unwrap_or("<single>"));
    out.push('|');
    out.push_str(&bridge_type.name);
    out.push('|');
    out.push_str(&bridge_type.rust_type_path);
    out.push('|');
    out.push_str(match bridge_type.kind {
        RustGeneratedBridgeTypeKind::Record => "record",
        RustGeneratedBridgeTypeKind::ClosedEnum => "enum",
        RustGeneratedBridgeTypeKind::Error => "error",
    });
    for field in &bridge_type.fields {
        out.push_str("|field=");
        out.push_str(&field.name);
        out.push(':');
        out.push_str(&field.sifr_type);
        out.push(':');
        out.push_str(&field.rust_type);
    }
    for variant in &bridge_type.variants {
        out.push_str("|variant=");
        out.push_str(&variant.name);
        out.push(':');
        out.push_str(&variant.discriminant.to_string());
    }
}
