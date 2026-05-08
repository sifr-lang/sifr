use crate::helpers::needs_clone_for_type;
use crate::RustEmitter;
use sifr_hir::HirExpr;
use sifr_type_system::Type;

impl RustEmitter {
    pub(super) fn method_call_needs_field_clone_suppression(
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

        if !crate::helpers::MUTATING_METHODS.contains(&method) {
            return false;
        }

        if matches!(parent.as_ref(), HirExpr::Name { name, .. } if name == "self") {
            return true;
        }

        let parent_class_name = match crate::resolve_alias_type_for_plain_call(parent.ty()) {
            Type::Class { name, .. } => Some(name.clone()),
            _ => None,
        };

        parent_class_name
            .is_some_and(|class_name| self.recursive_fields.contains(&(class_name, field.clone())))
    }

    pub(super) fn lower_field_access_expr_with_lowered_object(
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

    fn lower_proven_index_option_expr_for_ir(
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

    pub(super) fn try_lower_registry_expr_result(
        &self,
        expr: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        Ok(crate::try_lower_leaf_expr_result(expr)?
            .map(|lowered| self.rewrite_stdlib_constant_idents_in_expr(lowered)))
    }

    pub(super) fn rewrite_stdlib_constant_idents_in_expr(
        &self,
        expr: crate::RustExpr,
    ) -> crate::RustExpr {
        match expr {
            crate::RustExpr::Ident(name) => self.rewrite_special_ident(name),
            crate::RustExpr::MethodCall {
                receiver,
                method,
                args,
            } => crate::RustExpr::MethodCall {
                receiver: Box::new(match receiver.as_ref() {
                    crate::RustExpr::Ident(name) if self.is_stdlib_constant(name) => {
                        crate::RustExpr::Ident(name.clone())
                    }
                    _ => self.rewrite_stdlib_constant_idents_in_expr(*receiver),
                }),
                method,
                args: args
                    .into_iter()
                    .map(|arg| self.rewrite_stdlib_constant_idents_in_expr(arg))
                    .collect(),
            },
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

    pub(super) fn rewrite_stdlib_constant_idents_in_stmt(
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
                let (ty, value) =
                    if is_legacy_i64_type(&ty) && (value_is_sifr_int || force_sifr_int) {
                        let value = self.coerce_expr_to_sifr_int_value(value);
                        self.sifr_int_local_bindings
                            .borrow_mut()
                            .insert(name.clone());
                        (Some(crate::RustType::Named("SifrInt".to_string())), value)
                    } else if is_result_legacy_i64_type(&ty) && value_is_sifr_int_result {
                        self.sifr_int_result_local_bindings
                            .borrow_mut()
                            .insert(name.clone());
                        (ty.map(result_i64_type_to_sifr_int), value)
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

    pub(crate) fn try_lower_structured_field_access_expr(
        &mut self,
        object: &HirExpr,
        field: &str,
        ty: &Type,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let Some(lowered_object) = crate::try_lower_leaf_or_name_expr_result(object)? else {
            return Ok(None);
        };

        Ok(Some(self.lower_field_access_expr_with_lowered_object(
            object,
            field,
            ty,
            lowered_object,
        )))
    }

    pub(crate) fn try_lower_structured_class_binop_expr(
        &mut self,
        left: &HirExpr,
        op: &str,
        right: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let method_name = match op {
            "+" => "__add__",
            "-" => "__sub__",
            _ => return Ok(None),
        };
        let Type::Class { methods, .. } = crate::resolve_alias_type_for_plain_call(left.ty())
        else {
            return Ok(None);
        };
        let Some((_, method_sig)) = methods.iter().find(|(name, _)| name == method_name) else {
            return Ok(None);
        };

        let lowered_left = if let HirExpr::FieldAccess { object, field, ty } = left {
            self.try_lower_structured_field_access_expr(object, field, ty)?
        } else {
            crate::try_lower_leaf_or_name_expr_result(left)?
        };
        let Some(lowered_left) = lowered_left else {
            return Ok(None);
        };

        let lowered_right = if let HirExpr::FieldAccess { object, field, ty } = right {
            self.try_lower_structured_field_access_expr(object, field, ty)?
        } else {
            crate::try_lower_leaf_or_name_expr_result(right)?
        };
        let Some(lowered_right) = lowered_right else {
            return Ok(None);
        };

        let left_expr = crate::RustExpr::Ref {
            mutable: false,
            expr: Box::new(lowered_left),
        };

        let right_expr = if method_sig
            .params
            .first()
            .is_some_and(|(_, _, conv)| conv.is_shared_borrow())
        {
            crate::RustExpr::Ref {
                mutable: false,
                expr: Box::new(lowered_right),
            }
        } else {
            lowered_right
        };

        Ok(Some(crate::RustExpr::BinOp {
            left: Box::new(left_expr),
            op: op.to_string(),
            right: Box::new(right_expr),
        }))
    }

    pub(crate) fn try_lower_structured_index_expr(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
        result_ty: &Type,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let object_ty = crate::resolve_alias_type_for_plain_call(object.ty());
        let option_inner_ty = if let Type::Union(members) = object_ty {
            let mut inner = None;
            for member in members {
                let member = crate::resolve_alias_type_for_plain_call(member);
                if matches!(member, Type::None) {
                    continue;
                }
                if inner.is_some() {
                    inner = None;
                    break;
                }
                inner = Some(member);
            }
            inner
        } else {
            None
        };
        let index_base_ty = if matches!(
            object_ty,
            Type::Dict(_, _) | Type::List(_) | Type::Str | Type::Tuple(_)
        ) {
            Some(object_ty)
        } else if matches!(
            option_inner_ty,
            Some(Type::Dict(_, _) | Type::List(_) | Type::Str | Type::Tuple(_))
        ) {
            option_inner_ty
        } else {
            None
        };
        let Some(index_base_ty) = index_base_ty else {
            return Ok(None);
        };

        let suppress_self_field_clone = matches!(object, HirExpr::FieldAccess { object: inner, .. }
            if matches!(inner.as_ref(), HirExpr::Name { name, .. } if name == "self"))
            && self.pending_self_field_clone_suppression == 0;
        if suppress_self_field_clone {
            self.pending_self_field_clone_suppression += 1;
        }

        let lowered = (|| -> Result<Option<crate::RustExpr>, crate::CodegenError> {
            let lowered_object = if let HirExpr::FieldAccess {
                object: inner,
                field,
                ty,
            } = object
            {
                self.try_lower_structured_field_access_expr(inner, field, ty)?
            } else {
                crate::try_lower_leaf_or_name_expr_result(object)?
            };
            let lowered_object = match lowered_object {
                Some(expr) => expr,
                None => match self.try_lower_registry_expr_strict(object) {
                    Some(expr) => expr,
                    None => return Ok(None),
                },
            };

            let lowered_index = match crate::try_lower_leaf_or_name_expr_result(index)? {
                Some(expr) => expr,
                None => match self.try_lower_registry_expr_strict(index) {
                    Some(expr) => expr,
                    None => return Ok(None),
                },
            };

            let build_inner_index = |container_expr: crate::RustExpr| -> Option<crate::RustExpr> {
                let lowered_expr = match index_base_ty {
                    Type::Dict(key_ty, value_ty) => {
                        let projection_method =
                            crate::helpers::option_projection_method_for_owned_type(
                                value_ty.as_ref(),
                            );
                        let key_is_string_like = matches!(
                            crate::resolve_alias_type_for_plain_call(key_ty.as_ref()),
                            Type::Str | Type::LiteralStr(_)
                        );
                        let key_arg = if let HirExpr::StringLiteral(value) = index {
                            crate::RustExpr::Ident(format!("{value:?}"))
                        } else if key_is_string_like {
                            crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                    lowered_index.clone(),
                                ))),
                                method: "as_str".to_string(),
                                args: vec![],
                            }
                        } else {
                            crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(lowered_index.clone()),
                            }
                        };
                        let lowered_get = crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(container_expr),
                                method: "get".to_string(),
                                args: vec![key_arg],
                            }),
                            method: projection_method.to_string(),
                            args: vec![],
                        };
                        if crate::helpers::is_option_type(value_ty.as_ref()) {
                            crate::RustExpr::MethodCall {
                                receiver: Box::new(lowered_get),
                                method: "and_then".to_string(),
                                args: vec![crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__v".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(crate::RustExpr::Ident("__v".to_string())),
                                    is_move: false,
                                }],
                            }
                        } else {
                            lowered_get
                        }
                    }
                    Type::List(element_ty) => {
                        let projection_method =
                            crate::helpers::option_projection_method_for_owned_type(
                                element_ty.as_ref(),
                            );
                        let object_name = "__sifr_index_list".to_string();
                        let index_name = "__sifr_index_i".to_string();
                        let normalized_name = "__sifr_index_norm".to_string();
                        crate::RustExpr::Block {
                            stmts: vec![
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: object_name.clone(),
                                    ty: None,
                                    value: crate::RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(container_expr),
                                    },
                                },
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: index_name.clone(),
                                    ty: None,
                                    value: lowered_index.clone(),
                                },
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: normalized_name.clone(),
                                    ty: None,
                                    value: crate::RustExpr::If {
                                        cond: Box::new(crate::RustExpr::BinOp {
                                            left: Box::new(crate::RustExpr::Ident(
                                                index_name.clone(),
                                            )),
                                            op: "<".to_string(),
                                            right: Box::new(crate::RustExpr::Literal(
                                                crate::RustLiteral::Int(0),
                                            )),
                                        }),
                                        then_expr: Box::new(crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::BinOp {
                                                left: Box::new(crate::RustExpr::Cast {
                                                    expr: Box::new(crate::RustExpr::MethodCall {
                                                        receiver: Box::new(crate::RustExpr::Ident(
                                                            object_name.clone(),
                                                        )),
                                                        method: "len".to_string(),
                                                        args: vec![],
                                                    }),
                                                    ty: crate::RustType::I64,
                                                }),
                                                op: "+".to_string(),
                                                right: Box::new(crate::RustExpr::Ident(
                                                    index_name.clone(),
                                                )),
                                            }),
                                            ty: crate::RustType::Named("usize".to_string()),
                                        }),
                                        else_expr: Some(Box::new(crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::Ident(index_name)),
                                            ty: crate::RustType::Named("usize".to_string()),
                                        })),
                                    },
                                },
                            ],
                            expr: Some(Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident(object_name)),
                                    method: "get".to_string(),
                                    args: vec![crate::RustExpr::Ident(normalized_name)],
                                }),
                                method: projection_method.to_string(),
                                args: vec![],
                            })),
                        }
                    }
                    Type::Bytes => {
                        let object_name = "__sifr_index_bytes".to_string();
                        let index_name = "__sifr_index_i".to_string();
                        let normalized_name = "__sifr_index_norm".to_string();
                        crate::RustExpr::Block {
                            stmts: vec![
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: object_name.clone(),
                                    ty: None,
                                    value: crate::RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(container_expr),
                                    },
                                },
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: index_name.clone(),
                                    ty: None,
                                    value: lowered_index.clone(),
                                },
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: normalized_name.clone(),
                                    ty: None,
                                    value: crate::RustExpr::If {
                                        cond: Box::new(crate::RustExpr::BinOp {
                                            left: Box::new(crate::RustExpr::Ident(
                                                index_name.clone(),
                                            )),
                                            op: "<".to_string(),
                                            right: Box::new(crate::RustExpr::Literal(
                                                crate::RustLiteral::Int(0),
                                            )),
                                        }),
                                        then_expr: Box::new(crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::BinOp {
                                                left: Box::new(crate::RustExpr::Cast {
                                                    expr: Box::new(crate::RustExpr::MethodCall {
                                                        receiver: Box::new(crate::RustExpr::Ident(
                                                            object_name.clone(),
                                                        )),
                                                        method: "len".to_string(),
                                                        args: vec![],
                                                    }),
                                                    ty: crate::RustType::I64,
                                                }),
                                                op: "+".to_string(),
                                                right: Box::new(crate::RustExpr::Ident(
                                                    index_name.clone(),
                                                )),
                                            }),
                                            ty: crate::RustType::Named("usize".to_string()),
                                        }),
                                        else_expr: Some(Box::new(crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::Ident(index_name)),
                                            ty: crate::RustType::Named("usize".to_string()),
                                        })),
                                    },
                                },
                            ],
                            expr: Some(Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident(object_name)),
                                    method: "get".to_string(),
                                    args: vec![crate::RustExpr::Ident(normalized_name)],
                                }),
                                method: "map".to_string(),
                                args: vec![crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__byte".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(crate::RustExpr::Cast {
                                        expr: Box::new(crate::RustExpr::Deref(Box::new(
                                            crate::RustExpr::Ident("__byte".to_string()),
                                        ))),
                                        ty: crate::RustType::Named("u8".to_string()),
                                    }),
                                    is_move: false,
                                }],
                            })),
                        }
                    }
                    Type::Str => {
                        let object_name = "__sifr_index_str".to_string();
                        let index_name = "__sifr_index_i".to_string();
                        let normalized_name = "__sifr_index_norm".to_string();
                        crate::RustExpr::Block {
                            stmts: vec![
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: object_name.clone(),
                                    ty: None,
                                    value: crate::RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(container_expr),
                                    },
                                },
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: index_name.clone(),
                                    ty: None,
                                    value: lowered_index.clone(),
                                },
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: normalized_name.clone(),
                                    ty: None,
                                    value: crate::RustExpr::If {
                                        cond: Box::new(crate::RustExpr::BinOp {
                                            left: Box::new(crate::RustExpr::Ident(
                                                index_name.clone(),
                                            )),
                                            op: "<".to_string(),
                                            right: Box::new(crate::RustExpr::Literal(
                                                crate::RustLiteral::Int(0),
                                            )),
                                        }),
                                        then_expr: Box::new(crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::BinOp {
                                                left: Box::new(crate::RustExpr::Cast {
                                                    expr: Box::new(crate::RustExpr::MethodCall {
                                                        receiver: Box::new(
                                                            crate::RustExpr::MethodCall {
                                                                receiver: Box::new(
                                                                    crate::RustExpr::Ident(
                                                                        object_name.clone(),
                                                                    ),
                                                                ),
                                                                method: "chars".to_string(),
                                                                args: vec![],
                                                            },
                                                        ),
                                                        method: "count".to_string(),
                                                        args: vec![],
                                                    }),
                                                    ty: crate::RustType::I64,
                                                }),
                                                op: "+".to_string(),
                                                right: Box::new(crate::RustExpr::Ident(
                                                    index_name.clone(),
                                                )),
                                            }),
                                            ty: crate::RustType::Named("usize".to_string()),
                                        }),
                                        else_expr: Some(Box::new(crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::Ident(index_name)),
                                            ty: crate::RustType::Named("usize".to_string()),
                                        })),
                                    },
                                },
                            ],
                            expr: Some(Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident(object_name)),
                                        method: "chars".to_string(),
                                        args: vec![],
                                    }),
                                    method: "nth".to_string(),
                                    args: vec![crate::RustExpr::Ident(normalized_name)],
                                }),
                                method: "map".to_string(),
                                args: vec![crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "c".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident("c".to_string())),
                                        method: "to_string".to_string(),
                                        args: vec![],
                                    }),
                                    is_move: false,
                                }],
                            })),
                        }
                    }
                    Type::Tuple(elements) => {
                        let HirExpr::IntLiteral(idx) = index else {
                            return None;
                        };
                        let Ok(idx) = usize::try_from(*idx) else {
                            return None;
                        };
                        if idx >= elements.len() {
                            return None;
                        }
                        crate::RustExpr::Field {
                            expr: Box::new(container_expr),
                            field: idx.to_string(),
                        }
                    }
                    _ => return None,
                };
                Some(lowered_expr)
            };

            if let Some(inner_ty) = option_inner_ty {
                if !matches!(
                    inner_ty,
                    Type::Dict(_, _) | Type::List(_) | Type::Bytes | Type::Str | Type::Tuple(_)
                ) {
                    return Ok(None);
                }
                let Some(mut inner_expr) =
                    build_inner_index(crate::RustExpr::Ident("__v".to_string()))
                else {
                    return Ok(None);
                };
                if let (Type::Tuple(elements), HirExpr::IntLiteral(raw_idx)) = (inner_ty, index) {
                    if let Ok(idx) = usize::try_from(*raw_idx) {
                        if let Some(element_ty) = elements.get(idx) {
                            if !crate::helpers::is_copy_type_for_codegen(element_ty) {
                                inner_expr = crate::RustExpr::MethodCall {
                                    receiver: Box::new(inner_expr),
                                    method: "clone".to_string(),
                                    args: vec![],
                                };
                            }
                        }
                    }
                }
                let projection_method = if matches!(inner_ty, Type::Tuple(_)) {
                    "map"
                } else {
                    "and_then"
                };
                let option_expr = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_object))),
                        method: "as_ref".to_string(),
                        args: vec![],
                    }),
                    method: projection_method.to_string(),
                    args: vec![crate::RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "__v".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(inner_expr),
                        is_move: false,
                    }],
                };
                if crate::helpers::is_option_type(result_ty) {
                    return Ok(Some(option_expr));
                }
                if matches!(inner_ty, Type::Tuple(_)) {
                    return Ok(Some(Self::lower_proven_index_option_expr_for_ir(
                        option_expr,
                        "__sifr_index_value",
                        "compiler-verified tuple index should be in range",
                    )));
                }
                return Err(crate::CodegenError::new(
                    "internal codegen invariant violated: index on optional list/dict/bytes/str produced non-optional result type",
                ));
            }

            let Some(lowered_expr) = build_inner_index(lowered_object) else {
                return Ok(None);
            };
            if crate::helpers::is_option_type(result_ty) || matches!(index_base_ty, Type::Tuple(_))
            {
                return Ok(Some(lowered_expr));
            }
            match index_base_ty {
                Type::List(_) | Type::Bytes | Type::Str => Ok(Some(Self::lower_proven_index_option_expr_for_ir(
                    lowered_expr,
                    "__sifr_index_value",
                    "compiler-verified index should be in range",
                ))),
                Type::Dict(_, _) => Err(crate::CodegenError::new(
                    "internal codegen invariant violated: dict index produced non-optional result type",
                )),
                _ => Err(crate::CodegenError::new(
                    "internal codegen invariant violated: list/dict/bytes/str index produced non-optional result type",
                )),
            }
        })();

        if suppress_self_field_clone && self.pending_self_field_clone_suppression > 0 {
            self.pending_self_field_clone_suppression -= 1;
        }
        lowered
    }

    fn rewrite_special_ident(&self, name: String) -> crate::RustExpr {
        if self.local_binding_types.contains_key(&name) {
            return crate::RustExpr::Ident(name);
        }

        if self.is_stdlib_constant(&name) {
            return match name.as_str() {
                "pi" => crate::RustExpr::Path(vec![
                    "std".to_string(),
                    "f64".to_string(),
                    "consts".to_string(),
                    "PI".to_string(),
                ]),
                "e" => crate::RustExpr::Path(vec![
                    "std".to_string(),
                    "f64".to_string(),
                    "consts".to_string(),
                    "E".to_string(),
                ]),
                "tau" => crate::RustExpr::Path(vec![
                    "std".to_string(),
                    "f64".to_string(),
                    "consts".to_string(),
                    "TAU".to_string(),
                ]),
                "inf" => crate::RustExpr::Path(vec!["f64".to_string(), "INFINITY".to_string()]),
                "nan" => crate::RustExpr::Path(vec!["f64".to_string(), "NAN".to_string()]),
                _ => crate::RustExpr::Ident(name),
            };
        }

        if let Some((_ty, rust_name)) = self.module_constants.get(&name) {
            if let Some(mapped) = parse_module_constant_expr(rust_name) {
                return mapped;
            }
        }

        crate::RustExpr::Ident(name)
    }

    fn coerce_expr_to_sifr_int(&self, expr: crate::RustExpr) -> crate::RustExpr {
        match expr {
            crate::RustExpr::Ident(name) if self.is_registered_sifr_int_local(&name) => {
                crate::RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(crate::RustExpr::Ident(name)),
                }
            }
            crate::RustExpr::Paren(inner) => {
                crate::RustExpr::Paren(Box::new(self.coerce_expr_to_sifr_int(*inner)))
            }
            crate::RustExpr::BinOp { left, op, right }
                if is_sifr_int_arithmetic_op(&op)
                    && (self.is_sifr_int_expr(&left) || self.is_sifr_int_expr(&right)) =>
            {
                crate::RustExpr::BinOp {
                    left: Box::new(self.coerce_expr_to_sifr_int(*left)),
                    op,
                    right: Box::new(self.coerce_expr_to_sifr_int(*right)),
                }
            }
            other if self.is_sifr_int_expr(&other) => other,
            crate::RustExpr::Cast {
                expr,
                ty: crate::RustType::I64,
            } => sifr_int_from_i64_expr(*expr),
            other => sifr_int_from_i64_expr(other),
        }
    }

    fn sifr_int_known_nonzero_floor_expr(
        &self,
        op: &str,
        left: crate::RustExpr,
        right: crate::RustExpr,
    ) -> crate::RustExpr {
        let method = match op {
            "/" => "floor_div_known_nonzero",
            "%" => "floor_mod_known_nonzero",
            _ => unreachable!("SifrInt floor rewrite only handles floor division and modulo"),
        };
        crate::RustExpr::MethodCall {
            receiver: Box::new(self.coerce_expr_to_sifr_int_method_receiver(left)),
            method: method.to_string(),
            args: vec![self.coerce_expr_to_sifr_int_comparison_operand(right)],
        }
    }

    fn coerce_expr_to_sifr_int_method_receiver(&self, expr: crate::RustExpr) -> crate::RustExpr {
        match expr {
            crate::RustExpr::Ident(name) if self.is_registered_sifr_int_local(&name) => {
                crate::RustExpr::Ident(name)
            }
            crate::RustExpr::Paren(inner) => crate::RustExpr::Paren(Box::new(
                self.coerce_expr_to_sifr_int_method_receiver(*inner),
            )),
            crate::RustExpr::UnaryOp { op, operand } if op == "-" => {
                crate::RustExpr::Paren(Box::new(crate::RustExpr::UnaryOp { op, operand }))
            }
            other if self.is_sifr_int_expr(&other) => other,
            crate::RustExpr::Cast {
                expr,
                ty: crate::RustType::I64,
            } => sifr_int_from_i64_expr(*expr),
            other => sifr_int_from_i64_expr(other),
        }
    }

    pub(super) fn coerce_expr_to_sifr_int_value(&self, expr: crate::RustExpr) -> crate::RustExpr {
        match expr {
            crate::RustExpr::Ident(name) if self.is_registered_sifr_int_local(&name) => {
                crate::RustExpr::Clone(Box::new(crate::RustExpr::Ident(name)))
            }
            crate::RustExpr::Paren(inner) => {
                crate::RustExpr::Paren(Box::new(self.coerce_expr_to_sifr_int_value(*inner)))
            }
            crate::RustExpr::BinOp { left, op, right }
                if is_sifr_int_arithmetic_op(&op)
                    && (self.is_sifr_int_expr(&left) || self.is_sifr_int_expr(&right)) =>
            {
                crate::RustExpr::BinOp {
                    left: Box::new(self.coerce_expr_to_sifr_int(*left)),
                    op,
                    right: Box::new(self.coerce_expr_to_sifr_int(*right)),
                }
            }
            other if self.is_sifr_int_expr(&other) => other,
            crate::RustExpr::Cast {
                expr,
                ty: crate::RustType::I64,
            } => sifr_int_from_i64_expr(*expr),
            other => sifr_int_from_i64_expr(other),
        }
    }

    pub(super) fn coerce_result_int_expr_to_sifr_int_value(
        &self,
        expr: crate::RustExpr,
    ) -> crate::RustExpr {
        match expr {
            crate::RustExpr::FnCall { func, args } if is_ok_result_path(&func) => {
                let args = args
                    .into_iter()
                    .map(|arg| self.coerce_expr_to_sifr_int_value(arg))
                    .collect();
                crate::RustExpr::FnCall { func, args }
            }
            crate::RustExpr::Paren(inner) => crate::RustExpr::Paren(Box::new(
                self.coerce_result_int_expr_to_sifr_int_value(*inner),
            )),
            other if self.is_sifr_int_result_expr(&other) => other,
            other => other,
        }
    }

    fn coerce_expr_to_sifr_int_comparison_operand(&self, expr: crate::RustExpr) -> crate::RustExpr {
        let coerced = self.coerce_expr_to_sifr_int(expr);
        if matches!(coerced, crate::RustExpr::Ref { .. }) {
            return coerced;
        }
        crate::RustExpr::Ref {
            mutable: false,
            expr: Box::new(coerced),
        }
    }

    pub(super) fn is_registered_sifr_int_local(&self, name: &str) -> bool {
        self.sifr_int_local_bindings.borrow().contains(name)
    }

    pub(super) fn is_forced_sifr_int_local(&self, name: &str) -> bool {
        self.sifr_int_forced_local_bindings.borrow().contains(name)
    }

    pub(super) fn is_sifr_int_expr(&self, expr: &crate::RustExpr) -> bool {
        match expr {
            crate::RustExpr::FnCall { func, args } => {
                (args.is_empty() && self.is_sifr_int_module_constant_func(func))
                    || self.is_sifr_int_returning_function_call(func)
                    || matches!(
                        func.as_ref(),
                        crate::RustExpr::Path(path)
                            if string_path_matches(path, &["SifrInt", "from_i64"])
                                || string_path_matches(path, &["sifr_runtime", "SifrInt", "from_i64"])
                    )
            }
            crate::RustExpr::Ident(name) => self.is_registered_sifr_int_local(name),
            crate::RustExpr::BinOp { left, op, right } if is_sifr_int_arithmetic_op(op) => {
                self.is_sifr_int_expr(left) || self.is_sifr_int_expr(right)
            }
            crate::RustExpr::MethodCall {
                receiver, method, ..
            } if matches!(
                method.as_str(),
                "floor_div_known_nonzero" | "floor_mod_known_nonzero"
            ) =>
            {
                self.is_sifr_int_expr(receiver)
            }
            crate::RustExpr::UnaryOp { op, operand } if op == "-" => self.is_sifr_int_expr(operand),
            crate::RustExpr::Paren(inner) => self.is_sifr_int_expr(inner),
            crate::RustExpr::Ref { expr, .. } => self.is_sifr_int_expr(expr),
            crate::RustExpr::Clone(expr) => self.is_sifr_int_expr(expr),
            crate::RustExpr::Try(expr) => self.is_sifr_int_result_expr(expr),
            _ => false,
        }
    }

    pub(super) fn is_sifr_int_result_expr(&self, expr: &crate::RustExpr) -> bool {
        match expr {
            crate::RustExpr::Block {
                expr: Some(inner), ..
            } => self.is_sifr_int_result_expr(inner),
            crate::RustExpr::MethodCall {
                receiver, method, ..
            } if method == "ok_or_else" => self.is_sifr_int_checked_floor_option_expr(receiver),
            crate::RustExpr::MethodCall {
                receiver, method, ..
            } => self.is_sifr_int_result_returning_method_call(receiver, method),
            crate::RustExpr::FnCall { func, .. } => {
                self.is_sifr_int_result_returning_function_call(func)
            }
            crate::RustExpr::Ident(name) => self.is_registered_sifr_int_result_local(name),
            crate::RustExpr::Paren(inner) => self.is_sifr_int_result_expr(inner),
            _ => false,
        }
    }

    fn is_sifr_int_checked_floor_option_expr(&self, expr: &crate::RustExpr) -> bool {
        matches!(
            expr,
            crate::RustExpr::MethodCall {
                method,
                ..
            } if matches!(method.as_str(), "checked_floor_div" | "checked_floor_mod")
        )
    }

    fn is_sifr_int_module_constant_func(&self, func: &crate::RustExpr) -> bool {
        let Some(func_name) = rust_expr_identifier_path(func) else {
            return false;
        };
        self.module_constants.values().any(|(ty, rust_name)| {
            matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Int)
                && rust_name
                    .strip_suffix("()")
                    .is_some_and(|const_func| const_func == func_name)
        })
    }

    fn is_sifr_int_returning_function_call(&self, func: &crate::RustExpr) -> bool {
        rust_expr_identifier_path(func).is_some_and(|name| self.function_returns_sifr_int(&name))
    }

    fn is_sifr_int_result_returning_function_call(&self, func: &crate::RustExpr) -> bool {
        rust_expr_identifier_path(func).is_some_and(|name| {
            self.sifr_int_result_function_returns
                .borrow()
                .contains(&name)
        })
    }

    fn is_sifr_int_result_returning_method_call(
        &self,
        receiver: &crate::RustExpr,
        method: &str,
    ) -> bool {
        self.rust_expr_class_name(receiver)
            .is_some_and(|class_name| {
                self.sifr_int_result_method_returns.borrow().contains(
                    &crate::function_emitter::result_method_key(&class_name, method),
                )
            })
    }

    fn rust_expr_class_name(&self, expr: &crate::RustExpr) -> Option<String> {
        match expr {
            crate::RustExpr::Ident(name) if name == "self" => self.current_class_name.clone(),
            crate::RustExpr::Ident(name) => self.local_binding_types.get(name).and_then(|ty| {
                match crate::resolve_alias_type_for_plain_call(ty) {
                    Type::Class { name, .. } => Some(name.clone()),
                    _ => None,
                }
            }),
            crate::RustExpr::Paren(inner) => self.rust_expr_class_name(inner),
            _ => None,
        }
    }

    pub(super) fn function_returns_sifr_int(&self, name: &str) -> bool {
        self.sifr_int_function_returns.borrow().contains(name)
    }

    pub(super) fn function_param_lowers_to_sifr_int(&self, name: &str, idx: usize) -> bool {
        self.sifr_int_function_params
            .borrow()
            .get(name)
            .is_some_and(|params| params.contains(&idx))
    }

    pub(super) fn function_param_lowers_to_sifr_int_result(&self, name: &str, idx: usize) -> bool {
        self.sifr_int_result_function_params
            .borrow()
            .get(name)
            .is_some_and(|params| params.contains(&idx))
    }

    fn is_registered_sifr_int_result_local(&self, name: &str) -> bool {
        self.sifr_int_result_local_bindings.borrow().contains(name)
    }
}

fn parse_module_constant_expr(rust_name: &str) -> Option<crate::RustExpr> {
    if let Some(func) = rust_name.strip_suffix("()") {
        let func_expr = parse_identifier_path_expr(func)?;
        return Some(crate::RustExpr::FnCall {
            func: Box::new(func_expr),
            args: vec![],
        });
    }
    parse_identifier_path_expr(rust_name)
}

fn parse_identifier_path_expr(name: &str) -> Option<crate::RustExpr> {
    let segments = name
        .split("::")
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.is_empty() || !segments.iter().all(|segment| is_ident(segment)) {
        return None;
    }
    if segments.len() == 1 {
        return Some(crate::RustExpr::Ident(segments[0].to_string()));
    }
    Some(crate::RustExpr::Path(
        segments.into_iter().map(ToString::to_string).collect(),
    ))
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(ch) if ch == '_' || ch.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn sifr_int_from_i64_expr(expr: crate::RustExpr) -> crate::RustExpr {
    crate::RustExpr::FnCall {
        func: Box::new(crate::RustExpr::Path(vec![
            "SifrInt".to_string(),
            "from_i64".to_string(),
        ])),
        args: vec![expr],
    }
}

fn is_sifr_int_arithmetic_op(op: &str) -> bool {
    matches!(op, "+" | "-" | "*")
}

fn is_sifr_int_checked_floor_op(op: &str) -> bool {
    matches!(op, "/" | "%")
}

fn is_sifr_int_comparison_op(op: &str) -> bool {
    matches!(op, "==" | "!=" | "<" | "<=" | ">" | ">=")
}

fn is_sifr_int_operand_coercion_op(op: &str) -> bool {
    is_sifr_int_arithmetic_op(op) || is_sifr_int_comparison_op(op)
}

fn is_legacy_i64_type(ty: &Option<crate::RustType>) -> bool {
    matches!(ty, Some(crate::RustType::I64))
        || matches!(ty, Some(crate::RustType::Named(name)) if name == "i64")
}

fn is_result_legacy_i64_type(ty: &Option<crate::RustType>) -> bool {
    matches!(ty, Some(crate::RustType::Result(ok, _)) if is_legacy_i64_rust_type(ok))
        || matches!(ty, Some(crate::RustType::Named(name)) if name.starts_with("Result<i64, "))
}

fn is_legacy_i64_rust_type(ty: &crate::RustType) -> bool {
    matches!(ty, crate::RustType::I64)
        || matches!(ty, crate::RustType::Named(name) if name == "i64")
}

fn result_i64_type_to_sifr_int(ty: crate::RustType) -> crate::RustType {
    match ty {
        crate::RustType::Result(ok, err) if is_legacy_i64_rust_type(&ok) => {
            crate::RustType::Result(Box::new(crate::RustType::Named("SifrInt".to_string())), err)
        }
        crate::RustType::Named(name) if name.starts_with("Result<i64, ") => {
            crate::RustType::Named(name.replacen("Result<i64, ", "Result<SifrInt, ", 1))
        }
        other => other,
    }
}

fn is_proven_nonzero_integer_expr(expr: &crate::RustExpr) -> bool {
    match expr {
        crate::RustExpr::Literal(crate::RustLiteral::Int(value)) => *value != 0,
        crate::RustExpr::Cast {
            expr,
            ty: crate::RustType::I64,
        } => is_proven_nonzero_integer_expr(expr),
        crate::RustExpr::UnaryOp { op, operand } if op == "-" => {
            is_proven_nonzero_integer_expr(operand)
        }
        crate::RustExpr::Paren(inner) => is_proven_nonzero_integer_expr(inner),
        crate::RustExpr::FnCall { func, args }
            if args.len() == 1
                && matches!(
                    func.as_ref(),
                    crate::RustExpr::Path(path)
                        if string_path_matches(path, &["SifrInt", "from_i64"])
                            || string_path_matches(path, &["sifr_runtime", "SifrInt", "from_i64"])
                ) =>
        {
            is_proven_nonzero_integer_expr(&args[0])
        }
        _ => false,
    }
}

fn rust_expr_identifier_path(expr: &crate::RustExpr) -> Option<String> {
    match expr {
        crate::RustExpr::Ident(name) => Some(name.clone()),
        crate::RustExpr::Path(path) => Some(path.join("::")),
        _ => None,
    }
}

fn string_path_matches(path: &[String], expected: &[&str]) -> bool {
    path.len() == expected.len()
        && path
            .iter()
            .zip(expected)
            .all(|(segment, expected_segment)| segment == expected_segment)
}

fn is_ok_result_path(expr: &crate::RustExpr) -> bool {
    match expr {
        crate::RustExpr::Path(path) => string_path_matches(path, &["Ok"]),
        crate::RustExpr::Ident(name) => name == "Ok",
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RustExpr, RustLiteral, RustStmt, RustType};

    fn emitter_with_large_int_const() -> RustEmitter {
        let mut emitter = RustEmitter::new();
        emitter.module_constants.insert(
            "BIG_LIMIT".to_string(),
            (Type::Int, "__const_BIG_LIMIT()".to_string()),
        );
        emitter
    }

    #[test]
    fn rewrites_large_int_module_const_arithmetic_to_sifr_int_operands() {
        let emitter = emitter_with_large_int_const();
        let rewritten = emitter.rewrite_stdlib_constant_idents_in_expr(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("BIG_LIMIT".to_string())),
            op: "+".to_string(),
            right: Box::new(RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                ty: RustType::I64,
            }),
        });

        let RustExpr::BinOp { left, op, right } = rewritten else {
            panic!("expected SifrInt binary expression");
        };
        assert_eq!(op, "+");
        assert!(matches!(
            left.as_ref(),
            RustExpr::FnCall { func, args }
                if args.is_empty()
                    && matches!(func.as_ref(), RustExpr::Ident(name) if name == "__const_BIG_LIMIT")
        ));
        assert!(matches!(
            right.as_ref(),
            RustExpr::FnCall { func, args }
                if args.len() == 1
                    && matches!(func.as_ref(), RustExpr::Path(path) if path.as_slice() == ["SifrInt", "from_i64"])
        ));
    }

    #[test]
    fn rewrites_large_int_floor_division_by_nonzero_literal_to_checked_runtime_call() {
        let emitter = emitter_with_large_int_const();
        let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: false,
            name: "quotient".to_string(),
            ty: Some(RustType::I64),
            value: RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("BIG_LIMIT".to_string())),
                op: "/".to_string(),
                right: Box::new(RustExpr::Cast {
                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(3))),
                    ty: RustType::I64,
                }),
            },
        });

        let RustStmt::Let {
            ty: Some(RustType::Named(ty)),
            value:
                RustExpr::MethodCall {
                    receiver,
                    method,
                    args,
                },
            ..
        } = rewritten
        else {
            panic!("expected SifrInt floor division let");
        };
        assert_eq!(ty, "SifrInt");
        assert_eq!(method, "floor_div_known_nonzero");
        assert!(matches!(
            receiver.as_ref(),
            RustExpr::FnCall { func, args }
                if args.is_empty()
                    && matches!(func.as_ref(), RustExpr::Ident(name) if name == "__const_BIG_LIMIT")
        ));
        assert!(matches!(
            args.as_slice(),
            [RustExpr::Ref {
                mutable: false,
                expr,
            }] if matches!(
                expr.as_ref(),
                RustExpr::FnCall { func, args }
                    if args.len() == 1
                        && matches!(func.as_ref(), RustExpr::Path(path) if path.as_slice() == ["SifrInt", "from_i64"])
            )
        ));
    }

    #[test]
    fn rewrites_large_int_modulo_by_nonzero_literal_to_checked_runtime_call() {
        let emitter = emitter_with_large_int_const();
        let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: false,
            name: "remainder".to_string(),
            ty: Some(RustType::I64),
            value: RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("BIG_LIMIT".to_string())),
                op: "%".to_string(),
                right: Box::new(RustExpr::Cast {
                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(3))),
                    ty: RustType::I64,
                }),
            },
        });

        assert!(matches!(
            rewritten,
            RustStmt::Let {
                ty: Some(RustType::Named(ref ty)),
                value: RustExpr::MethodCall { ref method, .. },
                ..
            } if ty == "SifrInt" && method == "floor_mod_known_nonzero"
        ));
    }

    #[test]
    fn rewrites_large_int_module_const_let_type_to_sifr_int() {
        let emitter = emitter_with_large_int_const();
        let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: false,
            name: "x".to_string(),
            ty: Some(RustType::I64),
            value: RustExpr::Ident("BIG_LIMIT".to_string()),
        });

        assert!(matches!(
            rewritten,
            RustStmt::Let {
                ty: Some(RustType::Named(ref name)),
                value: RustExpr::FnCall { .. },
                ..
            } if name == "SifrInt"
        ));
    }

    #[test]
    fn local_binding_shadows_large_int_module_const_rewrite() {
        let mut emitter = emitter_with_large_int_const();
        emitter
            .local_binding_types
            .insert("BIG_LIMIT".to_string(), Type::Int);

        let rewritten = emitter
            .rewrite_stdlib_constant_idents_in_expr(RustExpr::Ident("BIG_LIMIT".to_string()));

        assert!(matches!(rewritten, RustExpr::Ident(name) if name == "BIG_LIMIT"));
    }

    #[test]
    fn rewrites_registered_sifr_int_local_arithmetic_to_sifr_int_operands() {
        let emitter = emitter_with_large_int_const();
        let _ = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: false,
            name: "oversized_local".to_string(),
            ty: Some(RustType::I64),
            value: RustExpr::Ident("BIG_LIMIT".to_string()),
        });

        let rewritten = emitter.rewrite_stdlib_constant_idents_in_expr(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("oversized_local".to_string())),
            op: "+".to_string(),
            right: Box::new(RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(2))),
                ty: RustType::I64,
            }),
        });

        let RustExpr::BinOp { left, op, right } = rewritten else {
            panic!("expected SifrInt local binary expression");
        };
        assert_eq!(op, "+");
        assert!(matches!(
            left.as_ref(),
            RustExpr::Ref { mutable: false, expr }
                if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "oversized_local")
        ));
        assert!(matches!(
            right.as_ref(),
            RustExpr::FnCall { func, args }
                if args.len() == 1
                    && matches!(func.as_ref(), RustExpr::Path(path) if path.as_slice() == ["SifrInt", "from_i64"])
        ));
    }

    #[test]
    fn rewrites_large_int_module_const_comparison_to_sifr_int_operands() {
        let emitter = emitter_with_large_int_const();
        let rewritten = emitter.rewrite_stdlib_constant_idents_in_expr(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("BIG_LIMIT".to_string())),
            op: ">".to_string(),
            right: Box::new(RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(100))),
                ty: RustType::I64,
            }),
        });

        let RustExpr::BinOp { left, op, right } = rewritten else {
            panic!("expected SifrInt comparison expression");
        };
        assert_eq!(op, ">");
        assert!(matches!(
            left.as_ref(),
            RustExpr::Ref { mutable: false, expr }
                if matches!(
                    expr.as_ref(),
                    RustExpr::FnCall { func, args }
                        if args.is_empty()
                            && matches!(func.as_ref(), RustExpr::Ident(name) if name == "__const_BIG_LIMIT")
                )
        ));
        assert!(matches!(
            right.as_ref(),
            RustExpr::Ref { mutable: false, expr }
                if matches!(
                    expr.as_ref(),
                    RustExpr::FnCall { func, args }
                        if args.len() == 1
                            && matches!(func.as_ref(), RustExpr::Path(path) if path.as_slice() == ["SifrInt", "from_i64"])
                )
        ));
    }

    #[test]
    fn rewrites_registered_sifr_int_local_comparison_to_borrowed_operands() {
        let emitter = emitter_with_large_int_const();
        let _ = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: false,
            name: "oversized_local".to_string(),
            ty: Some(RustType::I64),
            value: RustExpr::Ident("BIG_LIMIT".to_string()),
        });

        let rewritten = emitter.rewrite_stdlib_constant_idents_in_expr(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("oversized_local".to_string())),
            op: "<".to_string(),
            right: Box::new(RustExpr::Ident("BIG_LIMIT".to_string())),
        });

        let RustExpr::BinOp { left, op, right } = rewritten else {
            panic!("expected SifrInt local comparison expression");
        };
        assert_eq!(op, "<");
        assert!(matches!(
            left.as_ref(),
            RustExpr::Ref { mutable: false, expr }
                if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "oversized_local")
        ));
        assert!(matches!(
            right.as_ref(),
            RustExpr::Ref { mutable: false, expr }
                if matches!(
                    expr.as_ref(),
                    RustExpr::FnCall { func, args }
                        if args.is_empty()
                            && matches!(func.as_ref(), RustExpr::Ident(name) if name == "__const_BIG_LIMIT")
            )
        ));
    }

    #[test]
    fn rewrites_forced_sifr_int_assignment_target_storage() {
        let emitter = RustEmitter::new();
        emitter
            .sifr_int_forced_local_bindings
            .borrow_mut()
            .insert("total".to_string());

        let rewritten_let = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: true,
            name: "total".to_string(),
            ty: Some(RustType::I64),
            value: RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                ty: RustType::I64,
            },
        });

        assert!(matches!(
            rewritten_let,
            RustStmt::Let {
                ty: Some(RustType::Named(ref name)),
                value: RustExpr::FnCall { .. },
                ..
            } if name == "SifrInt"
        ));

        let rewritten_assign = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Assign {
            target: RustExpr::Ident("total".to_string()),
            value: RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(2))),
                ty: RustType::I64,
            },
        });

        assert!(matches!(
            rewritten_assign,
            RustStmt::Assign {
                target: RustExpr::Ident(ref name),
                value: RustExpr::FnCall { func, args },
            } if name == "total"
                && args.len() == 1
                && matches!(func.as_ref(), RustExpr::Path(path) if path.as_slice() == ["SifrInt", "from_i64"])
        ));
    }

    #[test]
    fn rewrites_sifr_int_value_position_aliases_to_clone() {
        let emitter = RustEmitter::new();
        emitter
            .sifr_int_local_bindings
            .borrow_mut()
            .insert("source".to_string());
        emitter
            .sifr_int_forced_local_bindings
            .borrow_mut()
            .insert("target".to_string());

        let rewritten_let = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: false,
            name: "target".to_string(),
            ty: Some(RustType::I64),
            value: RustExpr::Ident("source".to_string()),
        });
        assert!(matches!(
            rewritten_let,
            RustStmt::Let {
                ty: Some(RustType::Named(ref name)),
                value: RustExpr::Clone(inner),
                ..
            } if name == "SifrInt"
                && matches!(inner.as_ref(), RustExpr::Ident(source) if source == "source")
        ));

        let rewritten_assign = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Assign {
            target: RustExpr::Ident("target".to_string()),
            value: RustExpr::Ident("source".to_string()),
        });
        assert!(matches!(
            rewritten_assign,
            RustStmt::Assign {
                target: RustExpr::Ident(ref target),
                value: RustExpr::Clone(inner),
            } if target == "target"
                && matches!(inner.as_ref(), RustExpr::Ident(source) if source == "source")
        ));
    }

    #[test]
    fn rewrites_forced_sifr_int_augassign_to_assignment() {
        let emitter = RustEmitter::new();
        emitter
            .sifr_int_forced_local_bindings
            .borrow_mut()
            .insert("total".to_string());

        let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::AugAssign {
            target: RustExpr::Ident("total".to_string()),
            op: "+".to_string(),
            value: RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(2))),
                ty: RustType::I64,
            },
        });

        let RustStmt::Assign { target, value } = rewritten else {
            panic!("expected SifrInt augassign rewrite to plain assignment");
        };
        assert!(matches!(target, RustExpr::Ident(ref name) if name == "total"));
        assert!(matches!(
            value,
            RustExpr::BinOp { left, op, right }
                if op == "+"
                    && matches!(
                        left.as_ref(),
                        RustExpr::Ref { mutable: false, expr }
                            if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "total")
                    )
                    && matches!(
                        right.as_ref(),
                        RustExpr::FnCall { func, args }
                            if args.len() == 1
                                && matches!(func.as_ref(), RustExpr::Path(path) if path.as_slice() == ["SifrInt", "from_i64"])
                    )
        ));
    }

    #[test]
    fn rewrites_sifr_int_augassign_registered_source_to_borrowed_operand() {
        let emitter = RustEmitter::new();
        emitter
            .sifr_int_forced_local_bindings
            .borrow_mut()
            .insert("total".to_string());
        emitter
            .sifr_int_local_bindings
            .borrow_mut()
            .insert("source".to_string());

        let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::AugAssign {
            target: RustExpr::Ident("total".to_string()),
            op: "+".to_string(),
            value: RustExpr::Ident("source".to_string()),
        });

        let RustStmt::Assign { value, .. } = rewritten else {
            panic!("expected SifrInt augassign rewrite to plain assignment");
        };
        assert!(matches!(
            value,
            RustExpr::BinOp { right, .. }
                if matches!(
                    right.as_ref(),
                    RustExpr::Ref { mutable: false, expr }
                        if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "source")
                )
        ));
    }

    #[test]
    fn rewrites_sifr_int_augassign_for_supported_ops() {
        for op in ["+", "-", "*"] {
            let emitter = RustEmitter::new();
            emitter
                .sifr_int_forced_local_bindings
                .borrow_mut()
                .insert("total".to_string());

            let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::AugAssign {
                target: RustExpr::Ident("total".to_string()),
                op: op.to_string(),
                value: RustExpr::Cast {
                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(2))),
                    ty: RustType::I64,
                },
            });

            assert!(matches!(
                rewritten,
                RustStmt::Assign {
                    value: RustExpr::BinOp { op: ref rewritten_op, .. },
                    ..
                } if rewritten_op == op
            ));
        }
    }

    #[test]
    fn rewrites_sifr_int_floor_mod_augassign_by_nonzero_literal_to_assignment() {
        for (op, expected_method) in [
            ("/", "floor_div_known_nonzero"),
            ("%", "floor_mod_known_nonzero"),
        ] {
            let emitter = RustEmitter::new();
            emitter
                .sifr_int_forced_local_bindings
                .borrow_mut()
                .insert("total".to_string());

            let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::AugAssign {
                target: RustExpr::Ident("total".to_string()),
                op: op.to_string(),
                value: RustExpr::Cast {
                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(3))),
                    ty: RustType::I64,
                },
            });

            assert!(matches!(
                rewritten,
                RustStmt::Assign {
                    target: RustExpr::Ident(ref target),
                    value:
                        RustExpr::MethodCall {
                            receiver,
                            ref method,
                            ref args,
                        },
                } if target == "total"
                    && method == expected_method
                    && matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "total")
                    && matches!(
                        args.as_slice(),
                        [RustExpr::Ref {
                            mutable: false,
                            expr,
                        }] if matches!(
                            expr.as_ref(),
                            RustExpr::FnCall { func, args }
                                if args.len() == 1
                                    && matches!(func.as_ref(), RustExpr::Path(path) if path.as_slice() == ["SifrInt", "from_i64"])
                        )
                    )
            ));
        }
    }

    #[test]
    fn rewrites_sifr_int_returning_function_call_let_type() {
        let emitter = RustEmitter::new();
        emitter
            .sifr_int_function_returns
            .borrow_mut()
            .insert("make_big".to_string());

        let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: false,
            name: "value".to_string(),
            ty: Some(RustType::I64),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("make_big".to_string())),
                args: vec![],
            },
        });

        assert!(matches!(
            rewritten,
            RustStmt::Let {
                ty: Some(RustType::Named(ref name)),
                ..
            } if name == "SifrInt"
        ));
    }

    #[test]
    fn rewrites_sifr_int_returning_function_call_named_i64_let_type() {
        let emitter = RustEmitter::new();
        emitter
            .sifr_int_function_returns
            .borrow_mut()
            .insert("make_big".to_string());

        let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: false,
            name: "value".to_string(),
            ty: Some(RustType::Named("i64".to_string())),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("make_big".to_string())),
                args: vec![],
            },
        });

        assert!(matches!(
            rewritten,
            RustStmt::Let {
                ty: Some(RustType::Named(ref name)),
                ..
            } if name == "SifrInt"
        ));
    }

    #[test]
    fn rewrites_sifr_int_returning_function_call_with_args_let_type() {
        let emitter = RustEmitter::new();
        emitter
            .sifr_int_function_returns
            .borrow_mut()
            .insert("make_big_with_arg".to_string());

        let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: false,
            name: "value".to_string(),
            ty: Some(RustType::Named("i64".to_string())),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("make_big_with_arg".to_string())),
                args: vec![RustExpr::Cast {
                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(3))),
                    ty: RustType::I64,
                }],
            },
        });

        assert!(matches!(
            rewritten,
            RustStmt::Let {
                ty: Some(RustType::Named(ref name)),
                ..
            } if name == "SifrInt"
        ));
    }

    #[test]
    fn closure_block_returns_do_not_inherit_sifr_int_return_state() {
        let emitter = RustEmitter::new();
        emitter.current_sifr_int_return.set(true);

        let rewritten = emitter.rewrite_stdlib_constant_idents_in_expr(RustExpr::ClosureBlock {
            params: vec![],
            body: vec![RustStmt::Return(Some(RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(42))),
                ty: RustType::I64,
            }))],
            is_move: false,
        });

        let RustExpr::ClosureBlock { body, .. } = rewritten else {
            panic!("expected closure block");
        };
        assert!(matches!(
            body.as_slice(),
            [RustStmt::Return(Some(RustExpr::Cast {
                ty: RustType::I64,
                ..
            }))]
        ));
        assert!(emitter.current_sifr_int_return.get());
    }
}
