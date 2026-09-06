use super::{Expr, LowerCtx, StmtClassDef};
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;

pub(super) fn missing_method_param_annotation(
    ctx: &mut LowerCtx,
    class_name: &str,
    method_name: &str,
    param_name: &str,
    range: ruff_text_size::TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_MISSING_ANNOTATION,
        format!(
            "parameter '{param_name}' in {class_name}.{method_name} is missing a type annotation"
        ),
        range,
    );
}

pub(super) fn invalid_class_base(
    ctx: &mut LowerCtx,
    class_name: &str,
    reason: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::CLASS_INVALID_BASE,
        format!("invalid base class for '{class_name}': {reason}"),
        range,
    );
}

pub(super) fn unsupported_class_declaration(
    ctx: &mut LowerCtx,
    class_name: &str,
    detail: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::CLASS_UNSUPPORTED_DECLARATION,
        format!("unsupported class declaration in '{class_name}': {detail}"),
        range,
    );
}

pub(super) fn parent_class_range(class_def: &StmtClassDef, parent_name: &str) -> TextRange {
    class_def
        .bases()
        .iter()
        .find_map(|base| match base {
            Expr::Name(name) if name.id.as_str() == parent_name => Some(name.range()),
            _ => None,
        })
        .unwrap_or_else(|| class_def.name.range())
}
