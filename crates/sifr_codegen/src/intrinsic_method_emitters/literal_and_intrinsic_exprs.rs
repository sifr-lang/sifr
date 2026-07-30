use super::{intrinsics, HirExpr, RustEmitter};
use sifr_ir::CompilerIntrinsicId;

impl RustEmitter {
    pub(crate) fn try_lower_registry_dict_literal_expr(
        &mut self,
        keys: &[HirExpr],
        values: &[HirExpr],
    ) -> Option<crate::RustExpr> {
        if keys.len() != values.len() {
            return None;
        }

        let map_ident = "__sifr_registry_dict_literal".to_string();
        let mut stmts = vec![crate::RustStmt::Let {
            mutable: true,
            name: map_ident.clone(),
            ty: None,
            value: crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec![
                    "std".to_string(),
                    "collections".to_string(),
                    "HashMap".to_string(),
                    "new".to_string(),
                ])),
                args: vec![],
            },
        }];

        for (key, value) in keys.iter().zip(values.iter()) {
            stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Ident(map_ident.clone())),
                method: "insert".to_string(),
                args: vec![
                    self.try_lower_registry_expr_strict(key)?,
                    self.try_lower_registry_expr_strict(value)?,
                ],
            }));
        }

        Some(crate::RustExpr::Block {
            stmts,
            expr: Some(Box::new(crate::RustExpr::Ident(map_ident))),
        })
    }

    pub(crate) fn try_lower_registry_set_literal_expr(
        &mut self,
        elements: &[HirExpr],
    ) -> Option<crate::RustExpr> {
        let set_ident = "__sifr_registry_set_literal".to_string();
        let mut stmts = vec![crate::RustStmt::Let {
            mutable: true,
            name: set_ident.clone(),
            ty: None,
            value: crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec![
                    "std".to_string(),
                    "collections".to_string(),
                    "HashSet".to_string(),
                    "new".to_string(),
                ])),
                args: vec![],
            },
        }];

        for element in elements {
            let lowered = self.try_lower_registry_expr_strict(element)?;
            stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Ident(set_ident.clone())),
                method: "insert".to_string(),
                args: vec![Self::clone_owned_append_arg_expr_for_ir(element, lowered)],
            }));
        }

        Some(crate::RustExpr::Block {
            stmts,
            expr: Some(Box::new(crate::RustExpr::Ident(set_ident))),
        })
    }

    pub(crate) fn try_lower_registry_intrinsic_call_expr(
        &mut self,
        intrinsic: CompilerIntrinsicId,
        args: &[HirExpr],
        result_type: &sifr_type_system::Type,
    ) -> Option<crate::RustExpr> {
        if let Some(lowered) = self.try_lower_python_raw_intrinsic(intrinsic, args, result_type) {
            return Some(lowered);
        }
        let mut ir_args = if let Some(lowered_args) = self.try_lower_registry_exprs_strict(args) {
            lowered_args
        } else {
            let mut lowered_args = Vec::with_capacity(args.len());
            for arg in args {
                let lowered = self.lower_stmt_expr_for_ir(arg).ok().flatten()?;
                lowered_args.push(self.rewrite_stdlib_constant_idents_in_expr(lowered));
            }
            lowered_args
        };
        if matches!(
            intrinsic,
            CompilerIntrinsicId::TestAssertEqual
                | CompilerIntrinsicId::TestAssertNotEqual
                | CompilerIntrinsicId::TestAssertGreaterThan
                | CompilerIntrinsicId::TestAssertLessThan
                | CompilerIntrinsicId::TestAssertAlmostEqual
        ) {
            for (idx, arg) in args.iter().enumerate() {
                let HirExpr::Name { name, ty, .. } = arg else {
                    continue;
                };
                if !(self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name))
                {
                    continue;
                }
                if ty.ownership() == sifr_type_system::OwnershipKind::Copy {
                    continue;
                }
                if let Some(lowered_arg) = ir_args.get(idx).cloned() {
                    ir_args[idx] = crate::RustExpr::Clone(Box::new(lowered_arg));
                }
            }
        }
        let lowered = intrinsics::lower_intrinsic(intrinsic, &ir_args)?;
        self.apply_intrinsic_registry_side_effects(intrinsic, &lowered);
        Some(lowered.expr)
    }
}
