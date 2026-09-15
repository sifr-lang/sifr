use super::{
    HirExpr, RustEmitter, Type, methods, registry_defaultdict_alias_parts,
    registry_defaultdict_default_expr, registry_defaultdict_key_arg,
};
use crate::place_emitter::MethodCallPlaces;
use sifr_ir::MutableReceiverTarget;

impl RustEmitter {
    pub(crate) fn try_lower_defaultdict_index_method_call_expr(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
        places: MethodCallPlaces<'_>,
        method_return_ty: &Type,
    ) -> Option<crate::RustExpr> {
        let HirExpr::Index {
            object: base_object,
            index,
            ..
        } = object
        else {
            return None;
        };
        let (alias_name, key_ty, value_ty) = registry_defaultdict_alias_parts(base_object.ty())?;
        if !methods::is_in_place_collection_method(value_ty, method) {
            return None;
        }
        let MutableReceiverTarget::SpecializedIndexedStorage(base_place) = places.receiver_target?
        else {
            return None;
        };
        let lowered_object = self.emit_checked_place(base_object, base_place)?;
        let lowered_index = self.try_lower_registry_expr_strict(index)?;
        let lowered_key_arg = registry_defaultdict_key_arg(index, lowered_index, key_ty);
        let entry_key = crate::RustExpr::Ident("__sifr_defaultdict_key".to_string());
        let build_entry_expr =
            |receiver: crate::RustExpr, key: crate::RustExpr| crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(receiver),
                    method: "entry".to_string(),
                    args: vec![key],
                }),
                method: "or_insert".to_string(),
                args: vec![registry_defaultdict_default_expr(alias_name)],
            };
        let preinsert_entry_expr = build_entry_expr(
            lowered_object.clone(),
            crate::RustExpr::Clone(Box::new(entry_key.clone())),
        );
        let entry_expr = build_entry_expr(lowered_object, entry_key);

        if alias_name == "__sifr_defaultdict_list" && method == "extend" {
            let [iterable] = args else {
                return None;
            };
            return self.try_lower_defaultdict_list_extend_expr(
                lowered_key_arg,
                preinsert_entry_expr,
                entry_expr,
                iterable,
                value_ty,
            );
        }

        if alias_name == "__sifr_defaultdict_set"
            && matches!(
                method,
                "update"
                    | "intersection_update"
                    | "difference_update"
                    | "symmetric_difference_update"
            )
        {
            return self.try_lower_defaultdict_set_update_expr(
                lowered_key_arg,
                preinsert_entry_expr,
                entry_expr,
                method,
                args,
                value_ty,
            );
        }

        // Evaluate the key once, insert the default before argument effects, and
        // only activate the bucket borrow after all arguments are materialized.
        // Lowerers may use their receiver repeatedly (insert, pop, remove).
        let mut stmts = vec![crate::RustStmt::Let {
            mutable: false,
            name: "__sifr_defaultdict_key".to_string(),
            ty: None,
            value: lowered_key_arg,
        }];
        if !args.is_empty() {
            stmts.push(crate::RustStmt::Expr(preinsert_entry_expr));
        }
        let mut lowered_args = Vec::with_capacity(args.len());
        for (index, arg) in args.iter().enumerate() {
            let lowered = self
                .try_lower_registry_expr_strict(arg)
                .or_else(|| self.lower_stmt_expr_for_ir(arg).ok().flatten())?;
            let name = format!("__sifr_defaultdict_arg_{index}");
            stmts.push(crate::RustStmt::Let {
                mutable: false,
                name: name.clone(),
                ty: None,
                value: self.materialize_reusable_value_for_ir(arg, lowered),
            });
            lowered_args.push(crate::RustExpr::Ident(name));
        }
        let bucket = crate::RustExpr::Ident("__sifr_defaultdict_bucket".to_string());
        stmts.push(crate::RustStmt::Let {
            mutable: false,
            name: "__sifr_defaultdict_bucket".to_string(),
            ty: None,
            value: entry_expr,
        });
        let lowered = methods::lower_method(value_ty, method, &bucket, &lowered_args)?;
        let result = Self::unwrap_compiler_verified_nonempty_pop_result(
            value_ty,
            method,
            args,
            method_return_ty,
            bucket,
            false,
            lowered.expr,
        );
        let expr = if matches!(method, "append" | "add") {
            stmts.push(crate::RustStmt::Expr(result));
            crate::RustExpr::Literal(crate::RustLiteral::Unit)
        } else {
            result
        };
        Some(crate::RustExpr::Block {
            stmts,
            expr: Some(Box::new(expr)),
        })
    }

    pub(crate) fn try_lower_defaultdict_index_contains_expr(
        &mut self,
        element: &HirExpr,
        collection: &HirExpr,
    ) -> Option<crate::RustExpr> {
        let HirExpr::Index {
            object: base_object,
            index,
            ..
        } = collection
        else {
            return None;
        };
        let (alias_name, _key_ty, _) = registry_defaultdict_alias_parts(base_object.ty())?;
        if !crate::intrinsics::is_collection_defaultdict_storage_alias(alias_name) {
            return None;
        }
        let lowered_object = self.try_lower_registry_expr_strict(base_object)?;
        let lowered_index = self.try_lower_registry_expr_strict(index)?;
        let lowered_element = self.lower_stmt_expr_for_ir(element).ok()??;
        let element_arg = if matches!(
            element,
            HirExpr::Name { name, .. }
                if self.borrowed_params.contains(name)
                    || self.mut_borrowed_params.contains(name)
        ) {
            lowered_element
        } else {
            crate::RustExpr::Ref {
                mutable: false,
                expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_element))),
            }
        };
        let lookup = crate::RustExpr::MethodCall {
            receiver: Box::new(lowered_object),
            method: "get".to_string(),
            args: vec![self.checked_dict_key_arg_for_ir(index, lowered_index)],
        };
        Some(crate::RustExpr::MethodCall {
            receiver: Box::new(lookup),
            method: "is_some_and".to_string(),
            args: vec![crate::RustExpr::Closure {
                params: vec![crate::RustParam::Named {
                    name: "__sifr_defaultdict_bucket".to_string(),
                    ty: crate::RustType::Named("_".to_string()),
                }],
                body: Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Ident(
                        "__sifr_defaultdict_bucket".to_string(),
                    )),
                    method: "contains".to_string(),
                    args: vec![element_arg],
                }),
                is_move: false,
            }],
        })
    }
}
