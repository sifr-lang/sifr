use crate::helpers::needs_clone_for_type;
use crate::RustEmitter;
use sifr_hir::HirExpr;
use sifr_type_system::{ParamConvention, Type};

impl RustEmitter {
    fn lower_proven_index_option_expr_for_ir(
        &self,
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
            crate::RustExpr::FnCall { func, args } => crate::RustExpr::FnCall {
                func: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*func)),
                args: args
                    .into_iter()
                    .map(|arg| self.rewrite_stdlib_constant_idents_in_expr(arg))
                    .collect(),
            },
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
            crate::RustExpr::BinOp { left, op, right } => crate::RustExpr::BinOp {
                left: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*left)),
                op,
                right: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*right)),
            },
            crate::RustExpr::UnaryOp { op, operand } => crate::RustExpr::UnaryOp {
                op,
                operand: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*operand)),
            },
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
            } => crate::RustExpr::Closure {
                params,
                body: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*body)),
                is_move,
            },
            crate::RustExpr::ClosureBlock {
                params,
                body,
                is_move,
            } => crate::RustExpr::ClosureBlock {
                params,
                body: body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
                is_move,
            },
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
            } => crate::RustStmt::Let {
                mutable,
                name,
                ty,
                value: self.rewrite_stdlib_constant_idents_in_expr(value),
            },
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
            crate::RustStmt::Assign { target, value } => crate::RustStmt::Assign {
                target: self.rewrite_stdlib_constant_idents_in_expr(target),
                value: self.rewrite_stdlib_constant_idents_in_expr(value),
            },
            crate::RustStmt::AugAssign { target, op, value } => crate::RustStmt::AugAssign {
                target: self.rewrite_stdlib_constant_idents_in_expr(target),
                op,
                value: self.rewrite_stdlib_constant_idents_in_expr(value),
            },
            crate::RustStmt::Expr(expr) => {
                crate::RustStmt::Expr(self.rewrite_stdlib_constant_idents_in_expr(expr))
            }
            crate::RustStmt::Assert { cond, msg } => crate::RustStmt::Assert {
                cond: self.rewrite_stdlib_constant_idents_in_expr(cond),
                msg: msg.map(|msg| self.rewrite_stdlib_constant_idents_in_expr(msg)),
            },
            crate::RustStmt::Return(expr) => crate::RustStmt::Return(
                expr.map(|ret| self.rewrite_stdlib_constant_idents_in_expr(ret)),
            ),
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

        if matches!(object.ty(), Type::Enum { .. }) {
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Field {
                    expr: Box::new(lowered_object),
                    field: field.to_string(),
                }),
                args: vec![],
            }));
        }

        let is_self_access = matches!(object, HirExpr::Name { name, .. } if name == "self");
        let suppress_self_clone = if is_self_access && self.pending_self_field_clone_suppression > 0
        {
            self.pending_self_field_clone_suppression -= 1;
            true
        } else {
            false
        };
        let needs_clone = is_self_access && needs_clone_for_type(ty) && !suppress_self_clone;

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
            if let Type::Class { name, .. } = object.ty() {
                Some(name.clone())
            } else {
                None
            }
        });

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

        if needs_clone {
            return Ok(Some(crate::RustExpr::MethodCall {
                receiver: Box::new(lowered_field),
                method: "clone".to_string(),
                args: vec![],
            }));
        }

        Ok(Some(lowered_field))
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
            .is_some_and(|(_, _, conv)| *conv == ParamConvention::Borrow)
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
            Some(Type::Dict(_, _) | Type::List(_) | Type::Str)
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
                    Type::Dict(key_ty, _) => {
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
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(container_expr),
                                method: "get".to_string(),
                                args: vec![key_arg],
                            }),
                            method: "cloned".to_string(),
                            args: vec![],
                        }
                    }
                    Type::List(_) => {
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
                                method: "cloned".to_string(),
                                args: vec![],
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
                if !matches!(inner_ty, Type::Dict(_, _) | Type::List(_) | Type::Str) {
                    return Ok(None);
                }
                let Some(inner_expr) = build_inner_index(crate::RustExpr::Ident("__v".to_string()))
                else {
                    return Ok(None);
                };
                let option_expr = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_object))),
                        method: "as_ref".to_string(),
                        args: vec![],
                    }),
                    method: "and_then".to_string(),
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
                return Err(crate::CodegenError::new(
                    "internal codegen invariant violated: index on optional list/dict/str produced non-optional result type",
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
                Type::List(_) | Type::Str => Ok(Some(self.lower_proven_index_option_expr_for_ir(
                    lowered_expr,
                    "__sifr_index_value",
                    "compiler-verified index should be in range",
                ))),
                Type::Dict(_, _) => Err(crate::CodegenError::new(
                    "internal codegen invariant violated: dict index produced non-optional result type",
                )),
                _ => Err(crate::CodegenError::new(
                    "internal codegen invariant violated: list/dict/str index produced non-optional result type",
                )),
            }
        })();

        if suppress_self_field_clone && self.pending_self_field_clone_suppression > 0 {
            self.pending_self_field_clone_suppression -= 1;
        }
        lowered
    }

    fn rewrite_special_ident(&self, name: String) -> crate::RustExpr {
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
