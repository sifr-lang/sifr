use super::{HirStmt, RustEmitter, RustStmt};
impl RustEmitter {
    /// Emit a generator initialization statement (always mutable for closure capture)
    pub(crate) fn emit_generator_init_stmt(&mut self, stmt: &HirStmt) {
        if let HirStmt::Let {
            name, ty, value, ..
        } = stmt
        {
            let lowered_value = match self.lower_stmt_expr_for_ir(value) {
                Ok(Some(lowered_value)) => lowered_value,
                Ok(None) => {
                    self.record_codegen_error(crate::CodegenError::new(format!(
                        "structured generator-init expression emission missing for production path: {value:?}"
                    )));
                    return;
                }
                Err(error) => {
                    self.record_codegen_error(error.in_context(format!(
                        "structured generator-init expression emission failed for production path: {value:?}"
                    )));
                    return;
                }
            };
            self.push_captured_stmt(&crate::RustStmt::Let {
                mutable: true,
                name: name.clone(),
                ty: Some(self.rust_ir_type_with_generics(ty)),
                value: lowered_value,
            });
            return;
        }

        match self.try_lower_structured_stmt(stmt) {
            Ok(true) => {}
            Ok(false) => self.record_codegen_error(crate::CodegenError::new(format!(
                "structured generator-init statement emission missing for production path: {stmt:?}"
            ))),
            Err(error) => self.record_codegen_error(error.in_context(format!(
                "structured generator-init statement emission failed for production path: {stmt:?}"
            ))),
        }
    }

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
