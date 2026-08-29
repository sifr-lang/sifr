use super::{RustEmitter, RustStmt};
impl RustEmitter {
    pub(crate) fn emit_lowered_stmts(&mut self, lowered_stmts: &[RustStmt]) {
        for lowered_stmt in lowered_stmts {
            match lowered_stmt {
                RustStmt::Let {
                    mutable,
                    name,
                    ty,
                    value,
                } => self.push_captured_stmt(&crate::RustStmt::Let {
                    mutable: *mutable,
                    name: name.clone(),
                    ty: ty.clone(),
                    value: if let crate::RustExpr::Ident(value_name) = value {
                        if self.borrowed_params.contains(value_name)
                            || self.mut_borrowed_params.contains(value_name)
                        {
                            crate::RustExpr::Clone(Box::new(crate::RustExpr::Paren(Box::new(
                                crate::RustExpr::Ident(value_name.clone()),
                            ))))
                        } else {
                            value.clone()
                        }
                    } else {
                        value.clone()
                    },
                }),
                RustStmt::Expr(lowered_expr) => {
                    self.push_captured_stmt(&crate::RustStmt::Expr(lowered_expr.clone()));
                }
                _ => self.push_captured_stmt(lowered_stmt),
            }
        }
    }

    pub(crate) fn current_loop_has_else(&self) -> bool {
        self.loop_else_stack.last().copied().unwrap_or(false)
    }
}
