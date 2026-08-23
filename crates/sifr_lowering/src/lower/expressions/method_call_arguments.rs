//! Signature-aware argument lowering for method calls.

use super::{
    ExprCall, FunctionType, HirExpr, LowerCtx, Type, lower_method_call_args,
    lower_signature_call_args,
};

pub(super) fn lower(
    object_type: &Type,
    canonical_type: &Type,
    method: &str,
    call: &ExprCall,
    raw_python_method: Option<&FunctionType>,
    ctx: &mut LowerCtx,
) -> Option<Vec<HirExpr>> {
    if let Some(function_type) = raw_python_method {
        return lower_signature_call_args(
            call,
            &format!("Object.{method}"),
            function_type,
            None,
            ctx,
        );
    }
    match canonical_type {
        Type::Class { name, methods, .. } => {
            if let Some((_, function_type)) =
                methods.iter().find(|(candidate, _)| candidate == method)
            {
                let defaults_key = format!("{name}.{method}");
                let method_defaults = ctx.function_defaults.get(&defaults_key).cloned();
                lower_signature_call_args(
                    call,
                    &defaults_key,
                    function_type,
                    method_defaults.as_deref(),
                    ctx,
                )
            } else {
                lower_method_call_args(object_type, method, call, ctx)
            }
        }
        Type::Protocol { name, methods, .. } => {
            if let Some((_, function_type)) =
                methods.iter().find(|(candidate, _)| candidate == method)
            {
                lower_signature_call_args(
                    call,
                    &format!("{name}.{method}"),
                    function_type,
                    None,
                    ctx,
                )
            } else {
                lower_method_call_args(object_type, method, call, ctx)
            }
        }
        _ => lower_method_call_args(object_type, method, call, ctx),
    }
}
