use super::{HirExpr, LowerCtx, TextRange, Type, reject_no_method_args};
use sifr_type_system::PythonArrowKind;

pub(super) fn resolve_python_arrow_method_type(
    kind: PythonArrowKind,
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    if !args.is_empty() {
        reject_no_method_args(
            ctx,
            &format!("python.{}.{method}", kind.source_name()),
            arg_ranges,
            method_range,
        );
        return None;
    }
    match method {
        "capsule_names" => python_result(Type::List(Box::new(Type::Str)), method_range, ctx),
        "release" => python_result(Type::None, method_range, ctx),
        "producer_module" | "producer_type" => Some(Type::Str),
        _ => {
            ctx.error_with_code_at(
                sifr_diagnostics::DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                format!(
                    "type 'python.{}' has no method '{method}'",
                    kind.source_name()
                ),
                method_range,
            );
            None
        }
    }
}

pub(super) fn consume_python_arrow_release_receiver(
    object: &HirExpr,
    range: TextRange,
    ctx: &mut LowerCtx,
) -> bool {
    let HirExpr::Name { name, .. } = object else {
        ctx.error_with_code_at(
            sifr_diagnostics::DiagnosticCode::PYZC_INVALID_DECLARATION,
            "explicit Python Arrow release requires an owning local binding".to_string(),
            range,
        );
        return false;
    };
    if ctx.borrowed_params.contains(name) {
        ctx.error_with_code_at(
            sifr_diagnostics::DiagnosticCode::PYZC_INVALID_DECLARATION,
            format!(
                "cannot release borrowed Python Arrow resource `{name}`; accept it with `own` or release its owning local binding"
            ),
            range,
        );
        return false;
    }
    ctx.mark_moved_with_flow(name);
    true
}

fn python_result(ok: Type, range: TextRange, ctx: &mut LowerCtx) -> Option<Type> {
    let Some(python_error) = ctx
        .class_types
        .get("PythonError")
        .filter(|ty| ty.is_python_error_contract())
        .cloned()
    else {
        ctx.error_with_code_at(
            sifr_diagnostics::DiagnosticCode::PYZC_INVALID_DECLARATION,
            "python.Arrow methods require the canonical `PythonError` field contract; import `PythonError` from `sifr.python`".to_string(),
            range,
        );
        return None;
    };
    Some(Type::Result(Box::new(ok), Box::new(python_error)))
}
