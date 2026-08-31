use super::{HirExpr, RustEmitter, Type};

pub(super) fn lower_boolop_operand(
    emitter: &mut RustEmitter,
    operand: &HirExpr,
    result_ty: &Type,
) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
    if matches!(
        crate::resolve_alias_type_for_plain_call(result_ty),
        Type::Bool
    ) {
        emitter.lower_condition_expr_for_ir(operand)
    } else {
        emitter.lower_stmt_expr_for_ir(operand)
    }
}
