use super::{
    is_plain_i64_storage_type, is_proven_nonzero_integer_expr, is_result_plain_i64_storage_type,
    is_sifr_int_arithmetic_op, is_sifr_int_checked_floor_op, is_sifr_int_comparison_op,
    is_sifr_int_operand_coercion_op, promote_result_i64_ok_to_sifr_int, rust_expr_identifier_path,
};
use crate::helpers::needs_clone_for_type;
use crate::RustEmitter;
use sifr_ir::HirExpr;
use sifr_type_system::Type;

impl RustEmitter {
    pub(crate) fn method_call_needs_field_clone_suppression(
        &self,
        object: &HirExpr,
        method: &str,
    ) -> bool {
        let HirExpr::FieldAccess {
            object: parent,
            field,
            ..
        } = object
        else {
            return false;
        };

        if matches!(
            crate::resolve_alias_type_for_plain_call(object.ty()),
            Type::Class { .. }
        ) {
            return true;
        }

        if matches!(parent.as_ref(), HirExpr::Name { name, .. } if name == "self") {
            return true;
        }

        if !crate::helpers::MUTATING_METHODS.contains(&method) {
            return false;
        }

        let parent_class_name = match crate::resolve_alias_type_for_plain_call(parent.ty()) {
            Type::Class { name, .. } => Some(name.clone()),
            _ => None,
        };

        parent_class_name
            .is_some_and(|class_name| self.recursive_fields.contains(&(class_name, field.clone())))
    }

    pub(crate) fn lower_field_access_expr_with_lowered_object(
        &mut self,
        object: &HirExpr,
        field: &str,
        ty: &Type,
        lowered_object: crate::RustExpr,
    ) -> crate::RustExpr {
        if matches!(object.ty(), Type::Enum { .. }) {
            return crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Field {
                    expr: Box::new(lowered_object),
                    field: field.to_string(),
                }),
                args: vec![],
            };
        }

        let effective_object_ty = if let HirExpr::Name { name, ty } = object {
            if matches!(
                crate::resolve_alias_type_for_plain_call(ty),
                Type::Any | Type::Unknown
            ) {
                self.local_binding_types
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| ty.clone())
            } else {
                ty.clone()
            }
        } else {
            object.ty().clone()
        };
        let option_inner_object_ty = if let Type::Union(members) =
            crate::resolve_alias_type_for_plain_call(&effective_object_ty)
        {
            let mut inner = None;
            for member in members {
                if matches!(crate::resolve_alias_type_for_plain_call(member), Type::None) {
                    continue;
                }
                if inner.is_some() {
                    inner = None;
                    break;
                }
                inner = Some(member.clone());
            }
            inner
        } else {
            None
        };
        let effective_base_object_ty = option_inner_object_ty
            .clone()
            .unwrap_or_else(|| effective_object_ty.clone());
        let mut lowered_object = lowered_object;
        if option_inner_object_ty.is_some() {
            let option_expr = if matches!(object, HirExpr::Name { .. })
                && !crate::helpers::is_copy_type_for_codegen(&effective_object_ty)
            {
                crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_object))),
                    method: "clone".to_string(),
                    args: vec![],
                }
            } else {
                lowered_object
            };
            lowered_object = Self::force_unwrap_option_expr_for_ir(
                option_expr,
                "compiler-verified option field base should be Some",
            );
        }

        let is_self_access = matches!(object, HirExpr::Name { name, .. } if name == "self");
        let class_name_for_parent = if let Some(ref current_class_name) = self.current_class_name {
            if is_self_access {
                Some(current_class_name.clone())
            } else {
                None
            }
        } else {
            None
        }
        .or_else(|| {
            match crate::resolve_alias_type_for_plain_call(&effective_base_object_ty) {
                Type::Class { name, .. } => Some(name.clone()),
                _ => None,
            }
        });

        let is_recursive_field = class_name_for_parent.as_ref().is_some_and(|class_name| {
            self.recursive_fields
                .contains(&(class_name.clone(), field.to_owned()))
        });

        let suppress_self_clone = if self.pending_self_field_clone_suppression > 0
            && (is_self_access || is_recursive_field)
        {
            self.pending_self_field_clone_suppression -= 1;
            true
        } else {
            false
        };
        let needs_clone = is_self_access && needs_clone_for_type(ty) && !suppress_self_clone;

        let lowered_base = if let Some(ref class_name) = class_name_for_parent {
            if let Some((parent_name, parent_field_names)) = self.parent_fields.get(class_name) {
                if parent_field_names.contains(field) {
                    crate::RustExpr::Field {
                        expr: Box::new(lowered_object),
                        field: parent_name.to_lowercase(),
                    }
                } else {
                    lowered_object
                }
            } else {
                lowered_object
            }
        } else {
            lowered_object
        };

        let lowered_field = crate::RustExpr::Field {
            expr: Box::new(lowered_base),
            field: field.to_string(),
        };

        if is_recursive_field {
            if suppress_self_clone {
                return lowered_field;
            }
            if crate::helpers::is_option_type(ty) {
                if self.recursive_option_field_can_move(object) {
                    let moved_field = crate::RustExpr::MethodCall {
                        receiver: Box::new(lowered_field),
                        method: "take".to_string(),
                        args: vec![],
                    };
                    return crate::RustExpr::MethodCall {
                        receiver: Box::new(moved_field),
                        method: "map".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__sifr_boxed_recursive_value".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::Deref(Box::new(
                                crate::RustExpr::Ident("__sifr_boxed_recursive_value".to_string()),
                            ))),
                            is_move: false,
                        }],
                    };
                }
                return crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_field))),
                        method: "as_deref".to_string(),
                        args: vec![],
                    }),
                    method: "cloned".to_string(),
                    args: vec![],
                };
            }
            return crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_field))),
                    method: "as_ref".to_string(),
                    args: vec![],
                }),
                method: "clone".to_string(),
                args: vec![],
            };
        }

        if needs_clone {
            return crate::RustExpr::MethodCall {
                receiver: Box::new(lowered_field),
                method: "clone".to_string(),
                args: vec![],
            };
        }

        lowered_field
    }

    fn recursive_option_field_can_move(&self, object: &HirExpr) -> bool {
        let HirExpr::Name { name, .. } = object else {
            return false;
        };
        name != "self"
            && !self.borrowed_params.contains(name)
            && !self.mut_borrowed_params.contains(name)
    }
}

impl RustEmitter {
    pub(crate) fn lower_proven_index_option_expr_for_ir(
        option_expr: crate::RustExpr,
        binding_name: &str,
        message: &str,
    ) -> crate::RustExpr {
        crate::RustExpr::Block {
            stmts: vec![crate::RustStmt::LetElse {
                pattern: format!("Some({binding_name})"),
                value: option_expr,
                else_body: vec![crate::RustStmt::Expr(crate::RustExpr::MacroCall {
                    name: "unreachable".to_string(),
                    args: vec![crate::RustExpr::Literal(crate::RustLiteral::Str(
                        message.to_string(),
                    ))],
                })],
            }],
            expr: Some(Box::new(crate::RustExpr::Ident(binding_name.to_string()))),
        }
    }

    pub(crate) fn try_lower_registry_expr_result(
        &self,
        expr: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        Ok(crate::try_lower_leaf_expr_result(expr)?
            .map(|lowered| self.rewrite_stdlib_constant_idents_in_expr(lowered)))
    }

    pub(crate) fn rewrite_stdlib_constant_idents_in_expr(
        &self,
        expr: crate::RustExpr,
    ) -> crate::RustExpr {
        match expr {
            crate::RustExpr::Ident(name) => self.rewrite_special_ident(name),
            crate::RustExpr::MethodCall {
                receiver,
                method,
                args,
            } => {
                let receiver = match receiver.as_ref() {
                    crate::RustExpr::Ident(name) if self.is_stdlib_constant(name) => {
                        crate::RustExpr::Ident(name.clone())
                    }
                    _ => self.rewrite_stdlib_constant_idents_in_expr(*receiver),
                };
                let receiver_class = self.rust_expr_class_name(&receiver);
                let args = args
                    .into_iter()
                    .enumerate()
                    .map(|(idx, arg)| {
                        let arg = self.rewrite_stdlib_constant_idents_in_expr(arg);
                        if receiver_class.as_ref().is_some_and(|class_name| {
                            self.method_param_lowers_to_sifr_int_result(class_name, &method, idx)
                        }) {
                            self.coerce_result_int_expr_to_sifr_int_value(arg)
                        } else {
                            arg
                        }
                    })
                    .collect();
                crate::RustExpr::MethodCall {
                    receiver: Box::new(receiver),
                    method,
                    args,
                }
            }
            crate::RustExpr::FnCall { func, args } => {
                let func = self.rewrite_stdlib_constant_idents_in_expr(*func);
                let args = if let Some(func_name) = rust_expr_identifier_path(&func) {
                    args.into_iter()
                        .enumerate()
                        .map(|(idx, arg)| {
                            let arg = self.rewrite_stdlib_constant_idents_in_expr(arg);
                            if self.function_param_lowers_to_sifr_int(&func_name, idx) {
                                self.coerce_expr_to_sifr_int_value(arg)
                            } else if self.function_param_lowers_to_sifr_int_result(&func_name, idx)
                            {
                                self.coerce_result_int_expr_to_sifr_int_value(arg)
                            } else {
                                arg
                            }
                        })
                        .collect()
                } else {
                    args.into_iter()
                        .map(|arg| self.rewrite_stdlib_constant_idents_in_expr(arg))
                        .collect()
                };
                crate::RustExpr::FnCall {
                    func: Box::new(func),
                    args,
                }
            }
            crate::RustExpr::MacroCall { name, args } => crate::RustExpr::MacroCall {
                name,
                args: args
                    .into_iter()
                    .map(|arg| self.rewrite_stdlib_constant_idents_in_expr(arg))
                    .collect(),
            },
            crate::RustExpr::FormatMacro {
                name,
                format_str,
                args,
            } => crate::RustExpr::FormatMacro {
                name,
                format_str,
                args: args
                    .into_iter()
                    .map(|arg| self.rewrite_stdlib_constant_idents_in_expr(arg))
                    .collect(),
            },
            crate::RustExpr::BinOp { left, op, right } => {
                let left = self.rewrite_stdlib_constant_idents_in_expr(*left);
                let right = self.rewrite_stdlib_constant_idents_in_expr(*right);
                if is_sifr_int_checked_floor_op(&op)
                    && self.is_sifr_int_expr(&left)
                    && is_proven_nonzero_integer_expr(&right)
                {
                    return self.sifr_int_known_nonzero_floor_expr(op.as_str(), left, right);
                }
                if is_sifr_int_operand_coercion_op(&op)
                    && (self.is_sifr_int_expr(&left) || self.is_sifr_int_expr(&right))
                {
                    let (left, right) = if is_sifr_int_comparison_op(&op) {
                        (
                            self.coerce_expr_to_sifr_int_comparison_operand(left),
                            self.coerce_expr_to_sifr_int_comparison_operand(right),
                        )
                    } else {
                        (
                            self.coerce_expr_to_sifr_int(left),
                            self.coerce_expr_to_sifr_int(right),
                        )
                    };
                    return crate::RustExpr::BinOp {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    };
                }
                crate::RustExpr::BinOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                }
            }
            crate::RustExpr::UnaryOp { op, operand } => {
                let operand = self.rewrite_stdlib_constant_idents_in_expr(*operand);
                let operand = if op == "-" && self.is_sifr_int_expr(&operand) {
                    self.coerce_expr_to_sifr_int(operand)
                } else {
                    operand
                };
                crate::RustExpr::UnaryOp {
                    op,
                    operand: Box::new(operand),
                }
            }
            crate::RustExpr::Field { expr, field } => {
                let rewritten_expr = match expr.as_ref() {
                    crate::RustExpr::Ident(name) if self.is_stdlib_constant(name) => {
                        crate::RustExpr::Ident(name.clone())
                    }
                    _ => self.rewrite_stdlib_constant_idents_in_expr(*expr),
                };
                crate::RustExpr::Field {
                    expr: Box::new(rewritten_expr),
                    field,
                }
            }
            crate::RustExpr::Index { expr, index } => crate::RustExpr::Index {
                expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)),
                index: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*index)),
            },
            crate::RustExpr::Slice { expr, start, stop } => crate::RustExpr::Slice {
                expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)),
                start: start
                    .map(|part| Box::new(self.rewrite_stdlib_constant_idents_in_expr(*part))),
                stop: stop.map(|part| Box::new(self.rewrite_stdlib_constant_idents_in_expr(*part))),
            },
            crate::RustExpr::Ref { mutable, expr } => crate::RustExpr::Ref {
                mutable,
                expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)),
            },
            crate::RustExpr::Deref(expr) => {
                crate::RustExpr::Deref(Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)))
            }
            crate::RustExpr::Clone(expr) => {
                crate::RustExpr::Clone(Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)))
            }
            crate::RustExpr::Cast { expr, ty } => crate::RustExpr::Cast {
                expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)),
                ty,
            },
            crate::RustExpr::Block { stmts, expr } => crate::RustExpr::Block {
                stmts: stmts
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
                expr: expr
                    .map(|inner| Box::new(self.rewrite_stdlib_constant_idents_in_expr(*inner))),
            },
            crate::RustExpr::If {
                cond,
                then_expr,
                else_expr,
            } => crate::RustExpr::If {
                cond: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*cond)),
                then_expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*then_expr)),
                else_expr: else_expr
                    .map(|inner| Box::new(self.rewrite_stdlib_constant_idents_in_expr(*inner))),
            },
            crate::RustExpr::Match { expr, arms } => crate::RustExpr::Match {
                expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)),
                arms: arms
                    .into_iter()
                    .map(|arm| crate::RustMatchArm {
                        pattern: arm.pattern,
                        bindings: arm.bindings,
                        guard: arm
                            .guard
                            .map(|guard| self.rewrite_stdlib_constant_idents_in_expr(guard)),
                        body: arm
                            .body
                            .into_iter()
                            .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                            .collect(),
                    })
                    .collect(),
            },
            crate::RustExpr::Closure {
                params,
                body,
                is_move,
            } => {
                let saved_current_sifr_int_return = self.current_sifr_int_return.get();
                self.current_sifr_int_return.set(false);
                let body = Box::new(self.rewrite_stdlib_constant_idents_in_expr(*body));
                self.current_sifr_int_return
                    .set(saved_current_sifr_int_return);
                crate::RustExpr::Closure {
                    params,
                    body,
                    is_move,
                }
            }
            crate::RustExpr::ClosureBlock {
                params,
                body,
                is_move,
                is_async,
            } => {
                let saved_current_sifr_int_return = self.current_sifr_int_return.get();
                self.current_sifr_int_return.set(false);
                let body = body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect();
                self.current_sifr_int_return
                    .set(saved_current_sifr_int_return);
                crate::RustExpr::ClosureBlock {
                    params,
                    body,
                    is_move,
                    is_async,
                }
            }
            crate::RustExpr::StructInit { name, fields } => crate::RustExpr::StructInit {
                name,
                fields: fields
                    .into_iter()
                    .map(|(field, value)| {
                        (field, self.rewrite_stdlib_constant_idents_in_expr(value))
                    })
                    .collect(),
            },
            crate::RustExpr::Tuple(items) => crate::RustExpr::Tuple(
                items
                    .into_iter()
                    .map(|item| self.rewrite_stdlib_constant_idents_in_expr(item))
                    .collect(),
            ),
            crate::RustExpr::Array(items) => crate::RustExpr::Array(
                items
                    .into_iter()
                    .map(|item| self.rewrite_stdlib_constant_idents_in_expr(item))
                    .collect(),
            ),
            crate::RustExpr::Vec(items) => crate::RustExpr::Vec(
                items
                    .into_iter()
                    .map(|item| self.rewrite_stdlib_constant_idents_in_expr(item))
                    .collect(),
            ),
            crate::RustExpr::TimeoutAwait { duration, future } => crate::RustExpr::TimeoutAwait {
                duration: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*duration)),
                future: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*future)),
            },
            crate::RustExpr::Try(expr) => {
                crate::RustExpr::Try(Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)))
            }
            crate::RustExpr::Await(expr) => {
                crate::RustExpr::Await(Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)))
            }
            crate::RustExpr::Paren(expr) => {
                crate::RustExpr::Paren(Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)))
            }
            crate::RustExpr::Range { start, end } => crate::RustExpr::Range {
                start: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*start)),
                end: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*end)),
            },
            crate::RustExpr::Literal(lit) => crate::RustExpr::Literal(lit),
            crate::RustExpr::Path(path) => crate::RustExpr::Path(path),
        }
    }
}

impl RustEmitter {
    pub(crate) fn rewrite_stdlib_constant_idents_in_stmt(
        &self,
        stmt: crate::RustStmt,
    ) -> crate::RustStmt {
        match stmt {
            crate::RustStmt::Let {
                mutable,
                name,
                ty,
                value,
            } => {
                let value = self.rewrite_stdlib_constant_idents_in_expr(value);
                let force_sifr_int = self.is_forced_sifr_int_local(&name);
                let value_is_sifr_int = self.is_sifr_int_expr(&value);
                let value_is_sifr_int_result = self.is_sifr_int_result_expr(&value);
                let (ty, value) = if is_plain_i64_storage_type(ty.as_ref())
                    && (value_is_sifr_int || force_sifr_int)
                {
                    let value = self.coerce_expr_to_sifr_int_value(value);
                    self.sifr_int_local_bindings
                        .borrow_mut()
                        .insert(name.clone());
                    (Some(crate::RustType::Named("SifrInt".to_string())), value)
                } else if is_result_plain_i64_storage_type(ty.as_ref()) && value_is_sifr_int_result
                {
                    self.sifr_int_result_local_bindings
                        .borrow_mut()
                        .insert(name.clone());
                    (ty.map(promote_result_i64_ok_to_sifr_int), value)
                } else {
                    if !force_sifr_int {
                        self.sifr_int_local_bindings.borrow_mut().remove(&name);
                    }
                    self.sifr_int_result_local_bindings
                        .borrow_mut()
                        .remove(&name);
                    (ty, value)
                };
                crate::RustStmt::Let {
                    mutable,
                    name,
                    ty,
                    value,
                }
            }
            crate::RustStmt::LetPattern { pattern, value } => crate::RustStmt::LetPattern {
                pattern,
                value: self.rewrite_stdlib_constant_idents_in_expr(value),
            },
            crate::RustStmt::LetElse {
                pattern,
                value,
                else_body,
            } => crate::RustStmt::LetElse {
                pattern,
                value: self.rewrite_stdlib_constant_idents_in_expr(value),
                else_body: else_body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
            },
            crate::RustStmt::Assign { target, value } => {
                let target = self.rewrite_stdlib_constant_idents_in_expr(target);
                let value = self.rewrite_stdlib_constant_idents_in_expr(value);
                let value = match &target {
                    crate::RustExpr::Ident(name)
                        if self.is_registered_sifr_int_local(name)
                            || self.is_forced_sifr_int_local(name) =>
                    {
                        self.sifr_int_local_bindings
                            .borrow_mut()
                            .insert(name.clone());
                        self.coerce_expr_to_sifr_int_value(value)
                    }
                    crate::RustExpr::Ident(name)
                        if self.is_registered_sifr_int_result_local(name) =>
                    {
                        self.coerce_result_int_expr_to_sifr_int_value(value)
                    }
                    _ => value,
                };
                crate::RustStmt::Assign { target, value }
            }
            crate::RustStmt::AugAssign { target, op, value } => {
                let target = self.rewrite_stdlib_constant_idents_in_expr(target);
                let value = self.rewrite_stdlib_constant_idents_in_expr(value);
                match &target {
                    crate::RustExpr::Ident(name)
                        if is_sifr_int_checked_floor_op(&op)
                            && is_proven_nonzero_integer_expr(&value)
                            && (self.is_registered_sifr_int_local(name)
                                || self.is_forced_sifr_int_local(name)) =>
                    {
                        self.sifr_int_local_bindings
                            .borrow_mut()
                            .insert(name.clone());
                        crate::RustStmt::Assign {
                            target: target.clone(),
                            value: self.sifr_int_known_nonzero_floor_expr(
                                op.as_str(),
                                target,
                                value,
                            ),
                        }
                    }
                    crate::RustExpr::Ident(name)
                        if is_sifr_int_arithmetic_op(&op)
                            && (self.is_registered_sifr_int_local(name)
                                || self.is_forced_sifr_int_local(name)) =>
                    {
                        self.sifr_int_local_bindings
                            .borrow_mut()
                            .insert(name.clone());
                        crate::RustStmt::Assign {
                            target: target.clone(),
                            value: crate::RustExpr::BinOp {
                                left: Box::new(crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(target),
                                }),
                                op,
                                right: Box::new(self.coerce_expr_to_sifr_int(value)),
                            },
                        }
                    }
                    _ => crate::RustStmt::AugAssign { target, op, value },
                }
            }
            crate::RustStmt::Expr(expr) => {
                crate::RustStmt::Expr(self.rewrite_stdlib_constant_idents_in_expr(expr))
            }
            crate::RustStmt::Assert { cond, msg } => crate::RustStmt::Assert {
                cond: self.rewrite_stdlib_constant_idents_in_expr(cond),
                msg: msg.map(|msg| self.rewrite_stdlib_constant_idents_in_expr(msg)),
            },
            crate::RustStmt::Return(expr) => {
                let value = expr.map(|ret| {
                    let ret = self.rewrite_stdlib_constant_idents_in_expr(ret);
                    if self.current_sifr_int_return.get() {
                        self.coerce_expr_to_sifr_int_value(ret)
                    } else if self.current_sifr_int_result_return.get() {
                        self.coerce_result_int_expr_to_sifr_int_value(ret)
                    } else {
                        ret
                    }
                });
                crate::RustStmt::Return(value)
            }
            crate::RustStmt::If {
                cond,
                then_body,
                else_body,
            } => crate::RustStmt::If {
                cond: self.rewrite_stdlib_constant_idents_in_expr(cond),
                then_body: then_body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
                else_body: else_body.map(|body| {
                    body.into_iter()
                        .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                        .collect()
                }),
            },
            crate::RustStmt::IfLet {
                pattern,
                expr,
                then_body,
                else_body,
            } => crate::RustStmt::IfLet {
                pattern,
                expr: self.rewrite_stdlib_constant_idents_in_expr(expr),
                then_body: then_body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
                else_body: else_body.map(|body| {
                    body.into_iter()
                        .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                        .collect()
                }),
            },
            crate::RustStmt::Match { expr, arms } => crate::RustStmt::Match {
                expr: self.rewrite_stdlib_constant_idents_in_expr(expr),
                arms: arms
                    .into_iter()
                    .map(|arm| crate::RustMatchArm {
                        pattern: arm.pattern,
                        bindings: arm.bindings,
                        guard: arm
                            .guard
                            .map(|guard| self.rewrite_stdlib_constant_idents_in_expr(guard)),
                        body: arm
                            .body
                            .into_iter()
                            .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                            .collect(),
                    })
                    .collect(),
            },
            crate::RustStmt::For { var, iter, body } => crate::RustStmt::For {
                var,
                iter: self.rewrite_stdlib_constant_idents_in_expr(iter),
                body: body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
            },
            crate::RustStmt::With { items, body } => crate::RustStmt::With {
                items: items
                    .into_iter()
                    .map(|item| crate::RustWithItem {
                        binding: item.binding,
                        value: self.rewrite_stdlib_constant_idents_in_expr(item.value),
                        has_cm: item.has_cm,
                        class_name: item.class_name,
                    })
                    .collect(),
                body: body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
            },
            crate::RustStmt::While { cond, body } => crate::RustStmt::While {
                cond: self.rewrite_stdlib_constant_idents_in_expr(cond),
                body: body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
            },
            crate::RustStmt::Loop { body } => crate::RustStmt::Loop {
                body: body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
            },
            crate::RustStmt::LocalFn {
                name,
                params,
                ret,
                body,
            } => crate::RustStmt::LocalFn {
                name,
                params,
                ret,
                body: body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
            },
            crate::RustStmt::Block(stmts) => crate::RustStmt::Block(
                stmts
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
            ),
            crate::RustStmt::Break | crate::RustStmt::Continue => stmt,
        }
    }
}
