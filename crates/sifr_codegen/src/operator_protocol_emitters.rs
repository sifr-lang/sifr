use crate::{
    ClassScope, RustEmitter, RustExpr, RustItem, RustParam, RustStmt, RustType, ScopeContext,
    Visibility, try_lower_simple_stmt_with_scope_result_and_bindings,
};
use sifr_ir::{HirClass, HirExpr, HirFunction, HirModule, HirStmt};
use sifr_type_system::{Type, source_class_rust_name};
use std::collections::{HashMap, HashSet};

type OperatorBounds = HashMap<String, HashSet<String>>;

impl RustEmitter {
    fn closed_operator_bounds(
        module: &HirModule,
        class: &HirClass,
        function: &HirFunction,
        method_bounds: &HashMap<String, OperatorBounds>,
    ) -> Option<OperatorBounds> {
        let mut closed = OperatorBounds::new();
        if let Some(by_param) = module
            .type_param_bounds
            .get(&format!("{}.{}", class.name, function.name))
        {
            for (param, bounds) in by_param {
                closed.entry(param.clone()).or_default().extend(
                    bounds
                        .iter()
                        .map(|bound| Self::operator_rust_bound(param, bound)),
                );
            }
        }
        for called in Self::collect_direct_class_method_calls(&function.body, &class.name) {
            let Some(by_param) = method_bounds.get(&called) else {
                continue;
            };
            for (param, bounds) in by_param {
                closed
                    .entry(param.clone())
                    .or_default()
                    .extend(bounds.iter().cloned());
            }
        }
        (!closed.is_empty()).then_some(closed)
    }

    pub(crate) fn emit_operator_impls(
        &mut self,
        class: &HirClass,
        module: &HirModule,
        method_bounds: &HashMap<String, OperatorBounds>,
    ) {
        for (dunder, func) in &class.operator_impls {
            let bounds = Self::closed_operator_bounds(module, class, func, method_bounds);
            match dunder.as_str() {
                "__add__" => {
                    self.emit_binop_trait_impl(class, func, "Add", "add", bounds.as_ref());
                }
                "__sub__" => {
                    self.emit_binop_trait_impl(class, func, "Sub", "sub", bounds.as_ref());
                }
                "__mul__" => {
                    self.emit_binop_trait_impl(class, func, "Mul", "mul", bounds.as_ref());
                }
                "__truediv__" => {
                    self.emit_binop_trait_impl(class, func, "Div", "div", bounds.as_ref());
                }
                "__mod__" => {
                    self.emit_binop_trait_impl(class, func, "Rem", "rem", bounds.as_ref());
                }
                "__neg__" => {
                    self.emit_unaryop_trait_impl(class, func, "Neg", "neg", bounds.as_ref());
                }
                "__eq__" => self.emit_eq_trait_impl(class, func, bounds.as_ref()),
                "__lt__" => self.emit_ord_trait_impl(class, func, bounds.as_ref()),
                "__str__" | "__repr__" => {}
                _ => {}
            }
        }
    }

    pub(crate) fn emit_binop_trait_impl(
        &mut self,
        class: &HirClass,
        func: &HirFunction,
        trait_name: &str,
        method_name: &str,
        transitive_bounds: Option<&OperatorBounds>,
    ) {
        let is_generic = !class.type_params.is_empty();
        let generic_suffix = if is_generic {
            format!("<{}>", class.type_params.join(", "))
        } else {
            String::new()
        };
        let class_with_generics =
            format!("{}{}", source_class_rust_name(&class.name), generic_suffix);
        let rhs_ty = self.operator_rhs_type(class, func.params.first(), &class_with_generics);
        let output_ty = self.operator_output_type(class, &func.return_type);
        let rhs_name = func
            .params
            .first()
            .map(|param| param.name.clone())
            .unwrap_or_else(|| "rhs".to_string());
        let body = self.lower_operator_method_body(func);

        let items = vec![
            RustItem::TypeAlias {
                name: "Output".to_string(),
                ty: RustType::Named(output_ty),
            },
            RustItem::Fn {
                name: method_name.to_string(),
                visibility: Visibility::Private,
                type_params: Vec::new(),
                params: vec![
                    Self::rust_receiver_param(func),
                    RustParam::Named {
                        name: rhs_name,
                        ty: RustType::Named(rhs_ty.clone()),
                    },
                ],
                ret: Some(RustType::Named("Self::Output".to_string())),
                body,
                is_async: false,
            },
        ];
        let impl_type_params = if is_generic {
            Self::class_function_impl_type_params(class, func, &items, transitive_bounds)
        } else {
            Vec::new()
        };
        self.body_items.push(RustItem::Impl {
            target: format!("&{class_with_generics}"),
            type_params: impl_type_params,
            trait_: Some(format!("std::ops::{trait_name}<{rhs_ty}>")),
            items,
        });
    }

    pub(crate) fn emit_unaryop_trait_impl(
        &mut self,
        class: &HirClass,
        func: &HirFunction,
        trait_name: &str,
        method_name: &str,
        transitive_bounds: Option<&OperatorBounds>,
    ) {
        let body = self.lower_operator_method_body(func);
        let items = vec![
            RustItem::TypeAlias {
                name: "Output".to_string(),
                ty: RustType::Named(self.operator_output_type(class, &func.return_type)),
            },
            RustItem::Fn {
                name: method_name.to_string(),
                visibility: Visibility::Private,
                type_params: Vec::new(),
                params: vec![Self::rust_receiver_param(func)],
                ret: Some(RustType::Named("Self::Output".to_string())),
                body,
                is_async: false,
            },
        ];
        self.body_items.push(RustItem::Impl {
            target: Self::class_impl_target(class),
            type_params: Self::class_function_impl_type_params(
                class,
                func,
                &items,
                transitive_bounds,
            ),
            trait_: Some(format!("std::ops::{trait_name}")),
            items,
        });
    }

    pub(crate) fn emit_eq_trait_impl(
        &mut self,
        class: &HirClass,
        func: &HirFunction,
        transitive_bounds: Option<&OperatorBounds>,
    ) {
        let other_name = func
            .params
            .first()
            .map(|param| param.name.clone())
            .unwrap_or_else(|| "other".to_string());
        let body = self.lower_operator_method_body(func);
        let class_target = Self::class_impl_target(class);
        let items = vec![RustItem::Fn {
            name: "eq".to_string(),
            visibility: Visibility::Private,
            type_params: Vec::new(),
            params: vec![
                Self::rust_receiver_param(func),
                RustParam::Named {
                    name: other_name,
                    ty: RustType::Ref {
                        mutable: false,
                        inner: Box::new(RustType::Named(class_target.clone())),
                    },
                },
            ],
            ret: Some(RustType::Bool),
            body,
            is_async: false,
        }];
        self.body_items.push(RustItem::Impl {
            target: class_target,
            type_params: Self::class_function_impl_type_params(
                class,
                func,
                &items,
                transitive_bounds,
            ),
            trait_: Some("PartialEq".to_string()),
            items,
        });
    }

    pub(crate) fn emit_ord_trait_impl(
        &mut self,
        class: &HirClass,
        func: &HirFunction,
        transitive_bounds: Option<&OperatorBounds>,
    ) {
        let other_name = func
            .params
            .first()
            .map(|param| param.name.clone())
            .unwrap_or_else(|| "other".to_string());
        let class_target = Self::class_impl_target(class);
        let helper_items = vec![RustItem::Fn {
            name: "__sifr_lt".to_string(),
            visibility: Visibility::Private,
            type_params: Vec::new(),
            params: vec![
                Self::rust_receiver_param(func),
                RustParam::Named {
                    name: other_name.clone(),
                    ty: RustType::Ref {
                        mutable: false,
                        inner: Box::new(RustType::Named(class_target.clone())),
                    },
                },
            ],
            ret: Some(RustType::Bool),
            body: self.lower_operator_method_body(func),
            is_async: false,
        }];
        let helper_type_params =
            Self::class_function_impl_type_params(class, func, &helper_items, transitive_bounds);
        self.body_items.push(RustItem::Impl {
            target: class_target.clone(),
            type_params: helper_type_params,
            trait_: None,
            items: helper_items,
        });

        let less_call = RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("self".to_string())),
            method: "__sifr_lt".to_string(),
            args: vec![RustExpr::Ident(other_name.clone())],
        };
        let greater_call = RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(other_name.clone())),
            method: "__sifr_lt".to_string(),
            args: vec![RustExpr::Ident("self".to_string())],
        };
        let ordering = |variant: &str| {
            RustExpr::Path(vec![
                "std".to_string(),
                "cmp".to_string(),
                "Ordering".to_string(),
                variant.to_string(),
            ])
        };
        let ordering_expr = RustExpr::If {
            cond: Box::new(less_call),
            then_expr: Box::new(ordering("Less")),
            else_expr: Some(Box::new(RustExpr::If {
                cond: Box::new(greater_call),
                then_expr: Box::new(ordering("Greater")),
                else_expr: Some(Box::new(ordering("Equal"))),
            })),
        };
        let items = vec![RustItem::Fn {
            name: "partial_cmp".to_string(),
            visibility: Visibility::Private,
            type_params: Vec::new(),
            params: vec![
                RustParam::SelfParam { mutable: false },
                RustParam::Named {
                    name: other_name,
                    ty: RustType::Ref {
                        mutable: false,
                        inner: Box::new(RustType::Named(class_target.clone())),
                    },
                },
            ],
            ret: Some(RustType::Option(Box::new(RustType::Named(
                "std::cmp::Ordering".to_string(),
            )))),
            body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                args: vec![ordering_expr],
            }))],
            is_async: false,
        }];
        let mut trait_type_params =
            Self::class_function_impl_type_params(class, func, &items, transitive_bounds);
        let custom_eq = class
            .operator_impls
            .iter()
            .find(|(name, _)| name == "__eq__")
            .map(|(_, function)| function);
        for param in &mut trait_type_params {
            let equality_required = custom_eq.map_or_else(
                || {
                    class.fields.iter().any(|(_, field)| {
                        Self::type_mentions_type_param(field, param.name.as_str())
                    })
                },
                |eq| {
                    Self::extra_bound_items_for_type_param(param.name.as_str(), &eq.body)
                        .iter()
                        .any(|bound| bound == "PartialEq")
                },
            );
            let equality_implied = param
                .bounds
                .iter()
                .any(|bound| bound == "PartialEq" || bound == "PartialOrd" || bound == "Eq");
            if equality_required && !equality_implied {
                param.bounds.push("PartialEq".to_string());
            }
        }
        self.body_items.push(RustItem::Impl {
            target: class_target,
            type_params: trait_type_params,
            trait_: Some("PartialOrd".to_string()),
            items,
        });
    }

    pub(crate) fn lower_operator_method_body(&mut self, func: &HirFunction) -> Vec<RustStmt> {
        self.lower_function_like_body(
            func,
            "structured operator method statement lowering missing for IR-first emission",
            "structured operator method statement lowering failed for IR-first emission",
            Self::try_lower_operator_stmt_ir,
        )
    }

    pub(crate) fn try_lower_operator_stmt_ir(&mut self, stmt: &HirStmt) -> Option<Vec<RustStmt>> {
        match stmt {
            HirStmt::Let {
                name, ty, value, ..
            } => {
                let lowered_value = self.lower_operator_expr_ir(value)?;
                Some(vec![RustStmt::Let {
                    mutable: self.mutated_vars.contains(name)
                        || matches!(ty.resolve_alias(), Type::Iterator(_)),
                    name: name.clone(),
                    ty: Some(crate::sifr_type_to_rust_type(ty)),
                    value: lowered_value,
                }])
            }
            HirStmt::Assign { name, value } => {
                let lowered_value = self.lower_operator_expr_ir(value)?;
                Some(vec![RustStmt::Assign {
                    target: RustExpr::Ident(name.clone()),
                    value: lowered_value,
                }])
            }
            HirStmt::Expr { expr } => {
                let lowered_expr = self.lower_operator_expr_ir(expr)?;
                Some(vec![RustStmt::Expr(lowered_expr)])
            }
            HirStmt::Return { value } => {
                let lowered = value.as_ref().map(|expr| {
                    self.lower_operator_expr_ir(expr).map(|lowered| match expr {
                        HirExpr::Name { name, ty, .. }
                            if (self.borrowed_params.contains(name)
                                || self.mut_borrowed_params.contains(name))
                                && !crate::helpers::is_copy_type_for_codegen(ty) =>
                        {
                            RustExpr::Clone(Box::new(lowered))
                        }
                        _ => lowered,
                    })
                });
                match lowered {
                    Some(Some(expr)) => Some(vec![RustStmt::Return(Some(expr))]),
                    Some(None) => None,
                    None => Some(vec![RustStmt::Return(None)]),
                }
            }
            HirStmt::If {
                condition,
                then_body,
                elif_clauses,
                else_body,
            } => {
                let mut nested_else = if let Some(body) = else_body.as_ref() {
                    Some(self.lower_operator_stmt_block_ir(body)?)
                } else {
                    None
                };
                for (elif_cond, elif_body) in elif_clauses.iter().rev() {
                    nested_else = Some(vec![self.lower_operator_if_clause_ir(
                        elif_cond,
                        elif_body,
                        nested_else,
                    )?]);
                }
                Some(vec![self.lower_operator_if_clause_ir(
                    condition,
                    then_body,
                    nested_else,
                )?])
            }
            HirStmt::For {
                target,
                target_ty,
                iter,
                body,
                else_body,
                ..
            } => {
                if else_body.is_some() {
                    return None;
                }
                let char_set_loop = Self::should_lower_string_set_loop_target_as_char(
                    target, target_ty, iter, body,
                );
                let iter = if char_set_loop {
                    self.lower_operator_string_chars_for_iter_ir(iter)?
                } else {
                    self.lower_operator_for_iter_ir(iter)?
                };
                Some(vec![RustStmt::For {
                    var: target.clone(),
                    iter,
                    body: self.lower_operator_stmt_block_ir(body)?,
                }])
            }
            HirStmt::SubscriptAssign {
                object,
                index,
                value,
                ..
            } => Some(vec![RustStmt::Assign {
                target: RustExpr::Index {
                    expr: Box::new(RustExpr::Ident(object.clone())),
                    index: Box::new(self.lower_operator_expr_ir(index)?),
                },
                value: self.lower_operator_expr_ir(value)?,
            }]),
            HirStmt::AugAssign { name, op, value } => Some(vec![RustStmt::AugAssign {
                target: RustExpr::Ident(name.clone()),
                op: op.clone(),
                value: self.lower_operator_expr_ir(value)?,
            }]),
            HirStmt::SubscriptAugAssign {
                object,
                index,
                op,
                value,
                ..
            } => Some(vec![RustStmt::AugAssign {
                target: RustExpr::Index {
                    expr: Box::new(RustExpr::Ident(object.clone())),
                    index: Box::new(self.lower_operator_expr_ir(index)?),
                },
                op: op.clone(),
                value: self.lower_operator_expr_ir(value)?,
            }]),
            _ => None,
        }
    }

    pub(crate) fn lower_operator_stmt_block_ir(
        &mut self,
        body: &[HirStmt],
    ) -> Option<Vec<RustStmt>> {
        let scope_ctx = ScopeContext {
            function_return_type: self.current_return_type.clone(),
            in_generator_closure: false,
            in_display_impl: false,
            in_loop_with_else: false,
            class_scope: ClassScope::Inside,
        };
        let mut lowered = Vec::new();
        for stmt in body {
            match try_lower_simple_stmt_with_scope_result_and_bindings(
                stmt,
                &self.mutated_vars,
                &self.borrowed_params,
                &self.mut_borrowed_params,
                &self.local_binding_types,
                &self.recursive_fields,
                &scope_ctx,
            ) {
                Ok(Some(lowered_stmt)) => lowered.extend(
                    lowered_stmt
                        .into_iter()
                        .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt)),
                ),
                Ok(None) => {
                    let lowered_stmt = self.try_lower_operator_stmt_ir(stmt)?;
                    lowered.extend(lowered_stmt);
                }
                Err(_) => return None,
            }
        }
        Some(lowered)
    }

    pub(crate) fn lower_operator_if_clause_ir(
        &mut self,
        condition: &HirExpr,
        then_body: &[HirStmt],
        nested_else: Option<Vec<RustStmt>>,
    ) -> Option<RustStmt> {
        let lowered_then_body = self.lower_operator_stmt_block_ir(then_body)?;

        if let Some(option_var) = crate::helpers::detect_is_not_none_var(condition) {
            return Some(RustStmt::IfLet {
                pattern: format!("Some({option_var})"),
                expr: RustExpr::Ident(option_var),
                then_body: lowered_then_body,
                else_body: nested_else,
            });
        }

        if let Some(option_vars) = crate::helpers::detect_and_not_none_vars(condition) {
            return Self::lower_operator_if_not_none_chain_ir(
                &option_vars,
                lowered_then_body,
                nested_else,
            );
        }

        if let Some(option_var) = Self::detect_option_truthiness_alias_for_operator(condition) {
            return Some(RustStmt::IfLet {
                pattern: format!("Some({option_var})"),
                expr: RustExpr::Ident(option_var),
                then_body: lowered_then_body,
                else_body: nested_else,
            });
        }

        if let Some(option_var) = crate::helpers::detect_is_none_var(condition) {
            let lowered_cond = self.lower_operator_expr_ir(condition)?;
            let lowered_else = nested_else.map(|else_body| {
                vec![RustStmt::IfLet {
                    pattern: format!("Some({option_var})"),
                    expr: RustExpr::Ident(option_var.clone()),
                    then_body: else_body,
                    else_body: None,
                }]
            });
            return Some(RustStmt::If {
                cond: lowered_cond,
                then_body: lowered_then_body,
                else_body: lowered_else,
            });
        }

        Some(RustStmt::If {
            cond: self.lower_operator_expr_ir(condition)?,
            then_body: lowered_then_body,
            else_body: nested_else,
        })
    }

    pub(crate) fn lower_operator_if_not_none_chain_ir(
        option_vars: &[String],
        lowered_then_body: Vec<RustStmt>,
        nested_else: Option<Vec<RustStmt>>,
    ) -> Option<RustStmt> {
        let mut chain_then = lowered_then_body;
        for option_var in option_vars.iter().rev() {
            chain_then = vec![RustStmt::IfLet {
                pattern: format!("Some({option_var})"),
                expr: RustExpr::Ident(option_var.clone()),
                then_body: chain_then,
                else_body: None,
            }];
        }
        let mut chain_root = chain_then.into_iter().next()?;
        if let RustStmt::IfLet { else_body, .. } = &mut chain_root {
            *else_body = nested_else;
        }
        Some(chain_root)
    }

    pub(crate) fn lower_operator_for_iter_ir(&mut self, iter: &HirExpr) -> Option<RustExpr> {
        let lowered_iter = self.lower_operator_expr_ir(iter)?;
        Some(
            match Self::resolve_alias_type_for_operator_loop_iter(iter.ty()) {
                Type::List(_) => RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(lowered_iter),
                        method: "iter".to_string(),
                        args: vec![],
                    }),
                    method: "cloned".to_string(),
                    args: vec![],
                },
                Type::Dict(_, _) => RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(lowered_iter),
                        method: "keys".to_string(),
                        args: vec![],
                    }),
                    method: "cloned".to_string(),
                    args: vec![],
                },
                Type::Str => RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(lowered_iter),
                        method: "chars".to_string(),
                        args: vec![],
                    }),
                    method: "map".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![RustParam::Named {
                            name: "c".to_string(),
                            ty: RustType::Named("_".to_string()),
                        }],
                        body: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("c".to_string())),
                            method: "to_string".to_string(),
                            args: vec![],
                        }),
                        is_move: false,
                    }],
                },
                _ => lowered_iter,
            },
        )
    }

    pub(crate) fn lower_operator_string_chars_for_iter_ir(
        &mut self,
        iter: &HirExpr,
    ) -> Option<RustExpr> {
        if let HirExpr::IteratorCall { op, args, .. } = iter {
            if *op == sifr_ir::HirIteratorOp::Iter && args.len() == 1 {
                return self.lower_operator_string_chars_for_iter_ir(&args[0]);
            }
        }
        if let HirExpr::Call { func, args, .. } = iter {
            if func == "iter" && args.len() == 1 {
                return self.lower_operator_string_chars_for_iter_ir(&args[0]);
            }
        }
        let lowered_iter = self.lower_operator_expr_ir(iter)?;
        Some(RustExpr::MethodCall {
            receiver: Box::new(lowered_iter),
            method: "chars".to_string(),
            args: vec![],
        })
    }

    pub(crate) fn detect_option_truthiness_alias_for_operator(expr: &HirExpr) -> Option<String> {
        let HirExpr::Name { name, ty, .. } = expr else {
            return None;
        };
        if Self::is_option_like_for_operator(ty) {
            return Some(name.clone());
        }
        None
    }

    pub(crate) fn is_option_like_for_operator(ty: &Type) -> bool {
        ty.optional_member_type().is_some()
    }

    pub(crate) fn resolve_alias_type_for_operator_loop_iter(ty: &Type) -> &Type {
        match ty {
            Type::Alias { body, .. } => Self::resolve_alias_type_for_operator_loop_iter(body),
            _ => ty,
        }
    }

    pub(crate) fn lower_operator_expr_ir(&mut self, expr: &HirExpr) -> Option<RustExpr> {
        if !matches!(expr, HirExpr::Compare { .. }) {
            if let Some(lowered) = self.lower_stmt_expr_for_ir(expr).ok().flatten() {
                return Some(self.rewrite_stdlib_constant_idents_in_expr(lowered));
            }
        }
        if let Some(lowered) = crate::try_lower_leaf_or_name_expr_result(expr)
            .ok()
            .flatten()
        {
            return Some(self.rewrite_stdlib_constant_idents_in_expr(lowered));
        }
        match expr {
            HirExpr::FieldAccess { object, field, .. } => Some(RustExpr::Field {
                expr: Box::new(self.lower_operator_expr_ir(object)?),
                field: field.clone(),
            }),
            HirExpr::MethodCall {
                object,
                method,
                args,
                ..
            } => Some(RustExpr::MethodCall {
                receiver: Box::new(self.lower_operator_expr_ir(object)?),
                method: method.clone(),
                args: args
                    .iter()
                    .map(|arg| self.lower_operator_expr_ir(arg))
                    .collect::<Option<Vec<_>>>()?,
            }),
            HirExpr::Index { object, index, .. } => Some(RustExpr::Index {
                expr: Box::new(self.lower_operator_expr_ir(object)?),
                index: Box::new(self.lower_operator_expr_ir(index)?),
            }),
            HirExpr::BinOp {
                left, op, right, ..
            } => Some(RustExpr::BinOp {
                left: Box::new(self.lower_operator_expr_ir(left)?),
                op: if op == "//" {
                    "/".to_string()
                } else {
                    op.clone()
                },
                right: Box::new(self.lower_operator_expr_ir(right)?),
            }),
            HirExpr::BoolOp { op, values, .. } => {
                let rendered_op = if op == "and" { "&&" } else { "||" };
                let mut values_iter = values.iter();
                let first = self.lower_operator_expr_ir(values_iter.next()?)?;
                let combined = values_iter.try_fold(first, |acc, value| {
                    Some(RustExpr::BinOp {
                        left: Box::new(acc),
                        op: rendered_op.to_string(),
                        right: Box::new(self.lower_operator_expr_ir(value)?),
                    })
                })?;
                Some(combined)
            }
            HirExpr::Compare {
                left,
                ops,
                comparators,
                ..
            } => {
                if ops.is_empty() || comparators.is_empty() {
                    return None;
                }
                let mut chain: Option<RustExpr> = None;
                let mut prev = self.lower_operator_comparison_operand(left)?;
                for (op, cmp) in ops.iter().zip(comparators.iter()) {
                    let right = self.lower_operator_comparison_operand(cmp)?;
                    let compare_expr = RustExpr::BinOp {
                        left: Box::new(prev.clone()),
                        op: Self::map_operator_compare_op(op)?.to_string(),
                        right: Box::new(right.clone()),
                    };
                    chain = Some(if let Some(existing) = chain {
                        RustExpr::BinOp {
                            left: Box::new(existing),
                            op: "&&".to_string(),
                            right: Box::new(compare_expr),
                        }
                    } else {
                        compare_expr
                    });
                    prev = right;
                }
                chain
            }
            HirExpr::UnaryOp { op, operand, .. } => Some(RustExpr::UnaryOp {
                op: match op.as_str() {
                    "not" => "!".to_string(),
                    _ => op.clone(),
                },
                operand: Box::new(self.lower_operator_expr_ir(operand)?),
            }),
            _ => None,
        }
    }

    fn lower_operator_comparison_operand(&mut self, expr: &HirExpr) -> Option<RustExpr> {
        match expr {
            HirExpr::FieldAccess { object, field, .. } => Some(RustExpr::Field {
                expr: Box::new(self.lower_operator_comparison_operand(object)?),
                field: field.clone(),
            }),
            HirExpr::Name { name, .. } => Some(RustExpr::Ident(name.clone())),
            _ => self.lower_operator_expr_ir(expr),
        }
    }

    pub(crate) fn map_operator_compare_op(op: &str) -> Option<&'static str> {
        match op {
            "==" => Some("=="),
            "!=" => Some("!="),
            "<" => Some("<"),
            "<=" => Some("<="),
            ">" => Some(">"),
            ">=" => Some(">="),
            "is" => Some("=="),
            "is not" => Some("!="),
            _ => None,
        }
    }
}
