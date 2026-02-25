use crate::RustEmitter;
use sifr_hir::{HirExpr, HirStmt};

impl RustEmitter {
    /// Explicit legacy bridge for expression shapes that are not yet fully lowered.
    /// This keeps production routing structured-first while making fallback use auditable.
    pub(crate) fn try_emit_expr_legacy_bridge(&mut self, expr: &HirExpr) -> bool {
        match expr {
            HirExpr::IntLiteral(_)
            | HirExpr::FloatLiteral(_)
            | HirExpr::StringLiteral(_)
            | HirExpr::BoolLiteral(_)
            | HirExpr::NoneLiteral
            | HirExpr::Name { .. }
            | HirExpr::UnaryOp { .. }
            | HirExpr::BinOp { .. }
            | HirExpr::Compare { .. }
            | HirExpr::BoolOp { .. }
            | HirExpr::Call { .. }
            | HirExpr::MethodCall { .. }
            | HirExpr::ConstructorCall { .. }
            | HirExpr::IfExpr { .. }
            | HirExpr::Index { .. }
            | HirExpr::Slice { .. }
            | HirExpr::ListLiteral { .. }
            | HirExpr::TupleLiteral { .. }
            | HirExpr::DictLiteral { .. }
            | HirExpr::SetLiteral { .. }
            | HirExpr::ListComp { .. }
            | HirExpr::DictComp { .. }
            | HirExpr::SetComp { .. }
            | HirExpr::GeneratorExpr { .. }
            | HirExpr::FieldAccess { .. }
            | HirExpr::EnumVariant { .. }
            | HirExpr::ContainsOp { .. }
            | HirExpr::QuestionMark { .. }
            | HirExpr::OkWrap { .. }
            | HirExpr::ErrWrap { .. }
            | HirExpr::WalrusExpr { .. }
            | HirExpr::RangeLiteral { .. }
            | HirExpr::SuperCall { .. }
            | HirExpr::FString { .. }
            | HirExpr::Lambda { .. } => {
                self.emit_expr_fallback(expr);
                true
            }
        }
    }

    /// Explicit legacy bridge for statement shapes that are not yet fully lowered.
    /// This keeps production routing structured-first while making fallback use auditable.
    pub(crate) fn try_emit_stmt_legacy_bridge(&mut self, stmt: &HirStmt) -> bool {
        match stmt {
            HirStmt::Expr { .. }
            | HirStmt::Let { .. }
            | HirStmt::Assign { .. }
            | HirStmt::AugAssign { .. }
            | HirStmt::AttributeAugAssign { .. }
            | HirStmt::FieldAssign { .. }
            | HirStmt::Return { .. }
            | HirStmt::Assert { .. }
            | HirStmt::Raise { .. }
            | HirStmt::If { .. }
            | HirStmt::While { .. }
            | HirStmt::For { .. }
            | HirStmt::Pass
            | HirStmt::Continue
            | HirStmt::Break
            | HirStmt::TupleUnpack { .. }
            | HirStmt::StarUnpack { .. }
            | HirStmt::SubscriptAssign { .. }
            | HirStmt::NestedSubscriptAssign { .. }
            | HirStmt::SubscriptAugAssign { .. }
            | HirStmt::AttributeSubscriptAssign { .. }
            | HirStmt::Delete { .. }
            | HirStmt::Yield { .. }
            | HirStmt::With { .. }
            | HirStmt::Match { .. }
            | HirStmt::NestedFunction { .. }
            | HirStmt::TryExcept { .. } => {
                self.emit_stmt_fallback(stmt);
                true
            }
        }
    }
}
