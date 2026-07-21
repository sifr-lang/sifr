use super::{HirExpr, HirIteratorOp, Type};

pub(in crate::lower) fn lower_iterator_protocol_entry(
    iter_source_expr: HirExpr,
    elem_ty: Type,
) -> HirExpr {
    HirExpr::IteratorCall {
        op: HirIteratorOp::Iter,
        args: vec![iter_source_expr],
        ty: Type::Iterator(Box::new(elem_ty)),
    }
}
