use crate::hir_analysis::queries;
use crate::{RustEmitter, RustExpr, RustStmt};
use sifr_hir::{HirExceptHandler, HirExpr, HirFStringPart, HirStmt};
use sifr_type_system::Type;

fn io_error_kind_for_handler(error_type: &str) -> Option<&'static str> {
    match error_type {
        "FileNotFoundError" => Some("FileNotFound"),
        "PermissionError" => Some("PermissionDenied"),
        "FileExistsError" => Some("FileExists"),
        "IsADirectoryError" => Some("IsADirectory"),
        "NotADirectoryError" => Some("NotADirectory"),
        "DirectoryNotEmptyError" => Some("DirectoryNotEmpty"),
        _ => None,
    }
}

fn select_try_error_type(handlers: &[HirExceptHandler]) -> String {
    if handlers.iter().any(|handler| {
        let Some(error_type) = handler.error_type.as_deref() else {
            return false;
        };
        error_type == "IOError" || io_error_kind_for_handler(error_type).is_some()
    }) {
        return "IOError".to_string();
    }

    handlers
        .first()
        .and_then(|handler| handler.error_resolved_type.as_ref())
        .map(|ty| crate::render_type(&crate::sifr_type_to_rust_type(ty)))
        .unwrap_or_else(|| "Error".to_string())
}

fn can_construct_error_from_message_for_ir(ty_name: &str) -> bool {
    matches!(
        ty_name,
        "Error"
            | "ValueError"
            | "TypeError"
            | "NameError"
            | "ParseError"
            | "OverflowError"
            | "ZeroDivisionError"
            | "LookupError"
            | "IndexError"
            | "KeyError"
            | "RuntimeError"
            | "AssertionError"
            | "ImportError"
            | "IOError"
            | "RegexError"
            | "HashlibError"
            | "DecimalConversionError"
    )
}

enum HandlerMatchCondition {
    Unsupported,
    Always,
    Expr(RustExpr),
}

fn canonical_constructor_class_name(class_name: &str) -> &str {
    class_name
        .strip_prefix("__compat_sifr_collections_")
        .unwrap_or(class_name)
}

fn should_omit_local_type_annotation(ty: &Type, value: &HirExpr) -> bool {
    match (ty, value) {
        (resolved_ty, HirExpr::Call { func, args, .. })
            if matches!(
                crate::resolve_alias_type_for_plain_call(resolved_ty),
                Type::Set(_)
            ) && func == "set"
                && args.is_empty() =>
        {
            true
        }
        (
            Type::Alias {
                name: alias_name,
                body,
                ..
            },
            HirExpr::Call { func, args, .. },
        ) if func == alias_name
            && args.is_empty()
            && alias_name.starts_with("__compat_defaultdict_") =>
        {
            let Type::Dict(key_ty, value_ty) = body.resolve_alias() else {
                return false;
            };
            matches!(key_ty.as_ref(), Type::Any | Type::Unknown)
                || matches!(value_ty.as_ref(), Type::List(elem) if matches!(elem.as_ref(), Type::Any | Type::Unknown))
                || matches!(value_ty.as_ref(), Type::Set(elem) if matches!(elem.as_ref(), Type::Any | Type::Unknown))
        }
        _ => false,
    }
}

fn should_force_mutable_binding(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Alias { name: alias_name, .. } if alias_name.starts_with("__compat_defaultdict_")
    ) || matches!(ty.resolve_alias(), Type::Iterator(_))
}

impl RustEmitter {
    pub(super) fn wrap_option_local_value_for_ir(
        target_ty: &Type,
        value: &HirExpr,
        lowered_value: crate::RustExpr,
    ) -> crate::RustExpr {
        if !crate::helpers::is_option_type(target_ty) {
            return lowered_value;
        }
        if matches!(value, HirExpr::NoneLiteral) || matches!(value.ty(), Type::None) {
            return crate::RustExpr::Literal(crate::RustLiteral::None);
        }
        if crate::helpers::is_option_type(value.ty()) {
            return lowered_value;
        }
        crate::RustExpr::FnCall {
            func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
            args: vec![lowered_value],
        }
    }

    fn uses_debug_display_format_for_ir(ty: &Type) -> bool {
        match crate::resolve_alias_type_for_plain_call(ty) {
            Type::Int
            | Type::Float
            | Type::Bool
            | Type::Str
            | Type::None
            | Type::Range
            | Type::Union(_)
            | Type::LiteralInt(_)
            | Type::LiteralStr(_)
            | Type::LiteralBool(_)
            | Type::Class { .. }
            | Type::Newtype { .. }
            | Type::TypeVar(_)
            | Type::Enum { .. }
            | Type::BigInt
            | Type::Decimal
            | Type::BigDecimal => false,
            Type::List(_)
            | Type::Bytes
            | Type::Dict(_, _)
            | Type::Set(_)
            | Type::Tuple(_)
            | Type::Iterable(_)
            | Type::Iterator(_)
            | Type::Function(_)
            | Type::Callable(..)
            | Type::Result(_, _)
            | Type::Protocol { .. }
            | Type::Any
            | Type::Unknown
            | Type::Intersection(_)
            | Type::Never => true,
            Type::Alias { body, .. } => Self::uses_debug_display_format_for_ir(body),
        }
    }

    fn option_inner_type_for_ir(ty: &Type) -> Option<&Type> {
        let resolved = crate::resolve_alias_type_for_plain_call(ty);
        let Type::Union(members) = resolved else {
            return None;
        };
        if members.len() != 2 || !members.iter().any(|member| matches!(member, Type::None)) {
            return None;
        }
        members.iter().find(|member| !matches!(member, Type::None))
    }

    fn collect_stmt_string_concat_parts_for_ir<'a>(
        expr: &'a HirExpr,
        parts: &mut Vec<&'a HirExpr>,
    ) {
        if let HirExpr::BinOp {
            left,
            op,
            right,
            ty,
        } = expr
        {
            if op == "+" && matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Str) {
                Self::collect_stmt_string_concat_parts_for_ir(left, parts);
                Self::collect_stmt_string_concat_parts_for_ir(right, parts);
                return;
            }
        }
        parts.push(expr);
    }

    fn try_lower_stmt_string_concat_expr_for_ir(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let HirExpr::BinOp {
            left,
            op,
            right,
            ty,
        } = expr
        else {
            return Ok(None);
        };
        if op != "+" || !matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Str) {
            return Ok(None);
        }

        let mut parts = Vec::new();
        Self::collect_stmt_string_concat_parts_for_ir(left, &mut parts);
        Self::collect_stmt_string_concat_parts_for_ir(right, &mut parts);

        if parts
            .iter()
            .all(|part| matches!(part, HirExpr::StringLiteral(_)))
        {
            let mut combined = String::new();
            for part in parts {
                if let HirExpr::StringLiteral(value) = part {
                    combined.push_str(value);
                }
            }
            return Ok(Some(crate::RustExpr::Literal(crate::RustLiteral::Str(
                combined,
            ))));
        }

        let mut lowered_parts = Vec::with_capacity(parts.len());
        for part in parts {
            let Some(lowered_part) = self.lower_stmt_expr_for_ir(part)? else {
                return Ok(None);
            };
            lowered_parts.push(lowered_part);
        }
        Ok(Some(crate::RustExpr::FormatMacro {
            name: "format".to_string(),
            format_str: "{}".repeat(lowered_parts.len()),
            args: lowered_parts,
        }))
    }

    fn resolve_alias_type_for_loop_iter(ty: &Type) -> &Type {
        match ty {
            Type::Alias { body, .. } => Self::resolve_alias_type_for_loop_iter(body),
            _ => ty,
        }
    }

    fn int_i64_literal_expr(value: i64) -> RustExpr {
        RustExpr::Cast {
            expr: Box::new(RustExpr::Literal(crate::RustLiteral::Int(value))),
            ty: crate::RustType::I64,
        }
    }

    fn negative_range_step_magnitude(step_expr: &HirExpr) -> Option<i64> {
        match step_expr {
            HirExpr::IntLiteral(value) if *value < 0 => value.checked_abs(),
            HirExpr::UnaryOp { op, operand, .. } if op == "-" => match operand.as_ref() {
                HirExpr::IntLiteral(value) if *value > 0 => Some(*value),
                _ => None,
            },
            _ => None,
        }
    }

    fn try_lower_range_iter_expr_for_ir(
        &mut self,
        start: &HirExpr,
        end: &HirExpr,
        step: Option<&HirExpr>,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        let Some(step_expr) = step else {
            return Ok(None);
        };
        let Some(step_magnitude) = Self::negative_range_step_magnitude(step_expr) else {
            return Ok(None);
        };
        let Some(lowered_start) = self.lower_stmt_expr_for_ir(start)? else {
            return Ok(None);
        };
        let Some(lowered_end) = self.lower_stmt_expr_for_ir(end)? else {
            return Ok(None);
        };

        let reversed_iter = RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Range {
                start: Box::new(RustExpr::BinOp {
                    left: Box::new(lowered_end),
                    op: "+".to_string(),
                    right: Box::new(Self::int_i64_literal_expr(1)),
                }),
                end: Box::new(RustExpr::BinOp {
                    left: Box::new(lowered_start),
                    op: "+".to_string(),
                    right: Box::new(Self::int_i64_literal_expr(1)),
                }),
            }),
            method: "rev".to_string(),
            args: vec![],
        };
        if step_magnitude == 1 {
            return Ok(Some(reversed_iter));
        }
        Ok(Some(RustExpr::MethodCall {
            receiver: Box::new(reversed_iter),
            method: "step_by".to_string(),
            args: vec![RustExpr::Cast {
                expr: Box::new(Self::int_i64_literal_expr(step_magnitude)),
                ty: crate::RustType::Named("usize".to_string()),
            }],
        }))
    }

    fn lower_comprehension_iter_for_ir(
        &mut self,
        iter_expr: &HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        if let HirExpr::RangeLiteral {
            start, end, step, ..
        } = iter_expr
        {
            if let Some(lowered_range_iter) =
                self.try_lower_range_iter_expr_for_ir(start, end, step.as_deref())?
            {
                return Ok(Some(lowered_range_iter));
            }
        }
        let Some(lowered_iter) = self.lower_stmt_expr_for_ir(iter_expr)? else {
            return Ok(None);
        };
        if matches!(iter_expr.ty(), Type::Range) {
            return Ok(Some(lowered_iter));
        }
        Ok(Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(lowered_iter),
                method: "clone".to_string(),
                args: vec![],
            }),
            method: "into_iter".to_string(),
            args: vec![],
        }))
    }

    fn try_lower_comprehension_expr_for_ir(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        match expr {
            HirExpr::ListComp {
                expr,
                generators,
                ty,
            } if matches!(
                Self::resolve_alias_type_for_loop_iter(ty),
                Type::Any | Type::List(_)
            ) =>
            {
                if generators.is_empty() || generators.iter().any(|(var, _, _)| var.contains(',')) {
                    return Ok(None);
                }

                let result_ident = "__sifr_list_comp".to_string();
                let Some(lowered_expr) = self.lower_stmt_expr_for_ir(expr)? else {
                    return Ok(None);
                };
                let mut nested_body = vec![RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(result_ident.clone())),
                    method: "push".to_string(),
                    args: vec![lowered_expr],
                })];

                for (var, iter_expr, maybe_filter) in generators.iter().rev() {
                    let Some(iter) = self.lower_comprehension_iter_for_ir(iter_expr)? else {
                        return Ok(None);
                    };
                    let loop_body = if let Some(filter) = maybe_filter {
                        let Some(lowered_filter) = self.lower_stmt_expr_for_ir(filter)? else {
                            return Ok(None);
                        };
                        vec![RustStmt::If {
                            cond: lowered_filter,
                            then_body: nested_body,
                            else_body: None,
                        }]
                    } else {
                        nested_body
                    };
                    nested_body = vec![RustStmt::For {
                        var: var.clone(),
                        iter,
                        body: loop_body,
                    }];
                }

                let mut stmts = vec![RustStmt::Let {
                    mutable: true,
                    name: result_ident.clone(),
                    ty: None,
                    value: RustExpr::Vec(vec![]),
                }];
                stmts.extend(nested_body);

                Ok(Some(RustExpr::Block {
                    stmts,
                    expr: Some(Box::new(RustExpr::Ident(result_ident))),
                }))
            }
            HirExpr::DictComp {
                key_expr,
                val_expr,
                generators,
                ty,
            } if generators.len() == 1
                && matches!(
                    Self::resolve_alias_type_for_loop_iter(ty),
                    Type::Any | Type::Dict(_, _)
                ) =>
            {
                let Some((var, iter_expr, maybe_filter)) = generators.first() else {
                    return Ok(None);
                };
                if var.contains(',') {
                    return Ok(None);
                }
                let Some(iter) = self.lower_comprehension_iter_for_ir(iter_expr)? else {
                    return Ok(None);
                };
                let Some(lowered_key) = self.lower_stmt_expr_for_ir(key_expr)? else {
                    return Ok(None);
                };
                let Some(lowered_value) = self.lower_stmt_expr_for_ir(val_expr)? else {
                    return Ok(None);
                };

                let result_ident = "__sifr_dict_comp".to_string();
                let insert_stmt = RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(result_ident.clone())),
                    method: "insert".to_string(),
                    args: vec![lowered_key, lowered_value],
                });

                let loop_body = if let Some(filter) = maybe_filter {
                    let Some(lowered_filter) = self.lower_stmt_expr_for_ir(filter)? else {
                        return Ok(None);
                    };
                    vec![RustStmt::If {
                        cond: lowered_filter,
                        then_body: vec![insert_stmt],
                        else_body: None,
                    }]
                } else {
                    vec![insert_stmt]
                };

                Ok(Some(RustExpr::Block {
                    stmts: vec![
                        RustStmt::Let {
                            mutable: true,
                            name: result_ident.clone(),
                            ty: None,
                            value: RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec![
                                    "HashMap".to_string(),
                                    "new".to_string(),
                                ])),
                                args: vec![],
                            },
                        },
                        RustStmt::For {
                            var: var.clone(),
                            iter,
                            body: loop_body,
                        },
                    ],
                    expr: Some(Box::new(RustExpr::Ident(result_ident))),
                }))
            }
            HirExpr::SetComp {
                expr,
                generators,
                ty,
            } if generators.len() == 1
                && matches!(
                    Self::resolve_alias_type_for_loop_iter(ty),
                    Type::Any | Type::Set(_)
                ) =>
            {
                let Some((var, iter_expr, maybe_filter)) = generators.first() else {
                    return Ok(None);
                };
                if var.contains(',') {
                    return Ok(None);
                }
                let Some(iter) = self.lower_comprehension_iter_for_ir(iter_expr)? else {
                    return Ok(None);
                };
                let Some(lowered_expr) = self.lower_stmt_expr_for_ir(expr)? else {
                    return Ok(None);
                };

                let result_ident = "__sifr_set_comp".to_string();
                let insert_stmt = RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(result_ident.clone())),
                    method: "insert".to_string(),
                    args: vec![lowered_expr],
                });

                let loop_body = if let Some(filter) = maybe_filter {
                    let Some(lowered_filter) = self.lower_stmt_expr_for_ir(filter)? else {
                        return Ok(None);
                    };
                    vec![RustStmt::If {
                        cond: lowered_filter,
                        then_body: vec![insert_stmt],
                        else_body: None,
                    }]
                } else {
                    vec![insert_stmt]
                };

                Ok(Some(RustExpr::Block {
                    stmts: vec![
                        RustStmt::Let {
                            mutable: true,
                            name: result_ident.clone(),
                            ty: None,
                            value: RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec![
                                    "HashSet".to_string(),
                                    "new".to_string(),
                                ])),
                                args: vec![],
                            },
                        },
                        RustStmt::For {
                            var: var.clone(),
                            iter,
                            body: loop_body,
                        },
                    ],
                    expr: Some(Box::new(RustExpr::Ident(result_ident))),
                }))
            }
            _ => Ok(None),
        }
    }

    fn lower_structured_nested_list_subscript_assign_stmt_for_ir(
        &mut self,
        object: &str,
        outer_index: &HirExpr,
        inner_index: &HirExpr,
        value: &HirExpr,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let Some(lowered_outer_index) = self.lower_stmt_expr_for_ir(outer_index)? else {
            return Ok(None);
        };
        let Some(lowered_inner_index) = self.lower_stmt_expr_for_ir(inner_index)? else {
            return Ok(None);
        };
        let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(None);
        };

        Ok(Some(RustStmt::Block(vec![
            RustStmt::Let {
                mutable: false,
                name: "__nested_assign_value".to_string(),
                ty: None,
                value: lowered_value,
            },
            RustStmt::Let {
                mutable: false,
                name: "__oi_raw".to_string(),
                ty: None,
                value: lowered_outer_index,
            },
            RustStmt::Let {
                mutable: false,
                name: "__oi_norm".to_string(),
                ty: None,
                value: crate::build_normalized_list_index_i64_expr(
                    RustExpr::Ident(object.to_string()),
                    "__oi_raw",
                ),
            },
            RustStmt::If {
                cond: RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__oi_norm".to_string())),
                    op: ">=".to_string(),
                    right: Box::new(RustExpr::Literal(crate::RustLiteral::Int(0))),
                },
                then_body: vec![RustStmt::IfLet {
                    pattern: "Some(__row)".to_string(),
                    expr: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(object.to_string())),
                        method: "get_mut".to_string(),
                        args: vec![RustExpr::Cast {
                            expr: Box::new(RustExpr::Ident("__oi_norm".to_string())),
                            ty: crate::RustType::Named("usize".to_string()),
                        }],
                    },
                    then_body: vec![
                        RustStmt::Let {
                            mutable: false,
                            name: "__ii_raw".to_string(),
                            ty: None,
                            value: lowered_inner_index,
                        },
                        RustStmt::Let {
                            mutable: false,
                            name: "__ii_norm".to_string(),
                            ty: None,
                            value: crate::build_normalized_list_index_i64_expr(
                                RustExpr::Ident("__row".to_string()),
                                "__ii_raw",
                            ),
                        },
                        RustStmt::If {
                            cond: RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__ii_norm".to_string())),
                                op: ">=".to_string(),
                                right: Box::new(RustExpr::Literal(crate::RustLiteral::Int(0))),
                            },
                            then_body: vec![RustStmt::IfLet {
                                pattern: "Some(__elem)".to_string(),
                                expr: RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__row".to_string())),
                                    method: "get_mut".to_string(),
                                    args: vec![RustExpr::Cast {
                                        expr: Box::new(RustExpr::Ident("__ii_norm".to_string())),
                                        ty: crate::RustType::Named("usize".to_string()),
                                    }],
                                },
                                then_body: vec![RustStmt::Assign {
                                    target: RustExpr::Deref(Box::new(RustExpr::Ident(
                                        "__elem".to_string(),
                                    ))),
                                    value: RustExpr::Ident("__nested_assign_value".to_string()),
                                }],
                                else_body: None,
                            }],
                            else_body: None,
                        },
                    ],
                    else_body: None,
                }],
                else_body: None,
            },
        ])))
    }

    fn lower_subscript_assign_stmt_for_ir(
        &mut self,
        object: &str,
        index: &HirExpr,
        value: &HirExpr,
        object_ty: &Type,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };
        let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(None);
        };

        match Self::resolve_alias_type_for_loop_iter(object_ty) {
            Type::List(_) => Ok(Some(RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__assign_value".to_string(),
                    ty: None,
                    value: lowered_value,
                },
                crate::build_list_subscript_assign_stmt(
                    RustExpr::Ident(object.to_string()),
                    lowered_index,
                    RustExpr::Ident("__assign_value".to_string()),
                ),
            ]))),
            Type::Dict(key_ty, _) => {
                let key_needs_clone = matches!(key_ty.as_ref(), Type::Str | Type::TypeVar(_))
                    && matches!(index, HirExpr::Name { name, .. }
                        if self.borrowed_params.contains(name.as_str())
                            || self.mut_borrowed_params.contains(name.as_str()));
                let lowered_index = if key_needs_clone {
                    RustExpr::Clone(Box::new(lowered_index))
                } else {
                    lowered_index
                };
                Ok(Some(RustStmt::Block(vec![
                    RustStmt::Let {
                        mutable: false,
                        name: "__assign_key".to_string(),
                        ty: None,
                        value: lowered_index,
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "__assign_value".to_string(),
                        ty: None,
                        value: lowered_value,
                    },
                    crate::build_dict_subscript_assign_stmt(
                        RustExpr::Ident(object.to_string()),
                        RustExpr::Ident("__assign_key".to_string()),
                        RustExpr::Ident("__assign_value".to_string()),
                    ),
                ])))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn try_lower_structured_subscript_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::SubscriptAssign {
            object,
            index,
            value,
            object_ty,
        } = stmt
        else {
            return Ok(false);
        };

        let Some(lowered) =
            self.lower_subscript_assign_stmt_for_ir(object, index, value, object_ty)?
        else {
            return Ok(false);
        };

        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    pub(crate) fn try_lower_structured_nested_subscript_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::NestedSubscriptAssign {
            object,
            outer_index,
            inner_index,
            value,
            object_ty,
        } = stmt
        else {
            return Ok(false);
        };

        let Type::List(inner) = Self::resolve_alias_type_for_loop_iter(object_ty) else {
            return Ok(false);
        };
        if !matches!(Self::resolve_alias_type_for_loop_iter(inner), Type::List(_)) {
            return Ok(false);
        }

        let Some(lowered) = self.lower_structured_nested_list_subscript_assign_stmt_for_ir(
            object,
            outer_index,
            inner_index,
            value,
        )?
        else {
            return Ok(false);
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    pub(crate) fn lower_stmt_expr_for_ir(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let skip_leaf_registry_lowering = matches!(
            expr,
            HirExpr::Call { .. }
                | HirExpr::ConstructorCall { .. }
                | HirExpr::MethodCall { .. }
                | HirExpr::BinOp { .. }
                | HirExpr::UnaryOp { .. }
                | HirExpr::Compare { .. }
                | HirExpr::BoolOp { .. }
                | HirExpr::Slice { .. }
        );
        if !skip_leaf_registry_lowering {
            if let Some(lowered) = self.try_lower_registry_expr_result(expr)? {
                return Ok(Some(lowered));
            }
        }
        if let HirExpr::Call { func, args, .. } = expr {
            if func == "print" {
                return self.lower_print_call_expr_for_ir(args);
            }
        }
        if let HirExpr::FieldAccess { object, field, ty } = expr {
            if let Some(lowered) = self.try_lower_structured_field_access_expr(object, field, ty)? {
                return Ok(Some(lowered));
            }
        }
        if let HirExpr::ConstructorCall {
            class_name, args, ..
        } = expr
        {
            let emitted_class_name = canonical_constructor_class_name(class_name).to_string();
            let ctor_key = format!("{emitted_class_name}::new");
            let ctor_params = self
                .func_signatures
                .get(&ctor_key)
                .map(|(params, _)| params.clone());
            if let Some(mut lowered_ctor) =
                self.try_lower_registry_plain_call_with_signature(&ctor_key, args)
            {
                if let Some(params) = ctor_params.as_ref() {
                    if let crate::RustExpr::FnCall {
                        args: lowered_args, ..
                    } = &mut lowered_ctor
                    {
                        for (idx, lowered_arg) in lowered_args.iter_mut().enumerate() {
                            let Some((param_ty, _)) = params.get(idx) else {
                                continue;
                            };
                            let is_recursive_ctor_field = self
                                .class_field_order
                                .get(class_name)
                                .and_then(|fields| fields.get(idx))
                                .is_some_and(|field_name| {
                                    self.recursive_fields
                                        .contains(&(class_name.clone(), field_name.clone()))
                                });
                            let resolved_param = crate::resolve_alias_type_for_plain_call(param_ty);
                            if !crate::helpers::is_option_type(resolved_param) {
                                continue;
                            }
                            let needs_box_inner = param_ty.rust_type().starts_with("Option<Box<")
                                || is_recursive_ctor_field;
                            if !needs_box_inner || matches!(args[idx], HirExpr::NoneLiteral) {
                                continue;
                            }
                            let arg_is_option = crate::helpers::is_option_type(args[idx].ty());
                            if arg_is_option {
                                if Self::is_some_call_expr_for_ir(lowered_arg) {
                                    *lowered_arg =
                                        Self::ensure_some_box_inner_for_ir(lowered_arg.clone());
                                }
                            } else {
                                *lowered_arg =
                                    Self::ensure_some_box_inner_for_ir(lowered_arg.clone());
                            }
                        }
                    }
                }
                return Ok(Some(lowered_ctor));
            }
            let mut lowered_args = Vec::with_capacity(args.len());
            for arg in args {
                let Some(lowered_arg) = self.lower_stmt_expr_for_ir(arg)? else {
                    return Ok(None);
                };
                let adapted_arg = if let HirExpr::Name { name, ty } = arg {
                    if (self.borrowed_params.contains(name)
                        || self.mut_borrowed_params.contains(name))
                        && ty.ownership() != sifr_type_system::OwnershipKind::Copy
                    {
                        crate::RustExpr::Clone(Box::new(lowered_arg))
                    } else {
                        lowered_arg
                    }
                } else {
                    lowered_arg
                };
                lowered_args.push(adapted_arg);
            }
            for (idx, lowered_arg) in lowered_args.iter_mut().enumerate() {
                let is_recursive_ctor_field = self
                    .class_field_order
                    .get(class_name)
                    .and_then(|fields| fields.get(idx))
                    .is_some_and(|field_name| {
                        self.recursive_fields
                            .contains(&(class_name.clone(), field_name.clone()))
                    });
                if !is_recursive_ctor_field || matches!(args[idx], HirExpr::NoneLiteral) {
                    continue;
                }
                let arg_is_option = crate::helpers::is_option_type(args[idx].ty());
                if arg_is_option {
                    if Self::is_some_call_expr_for_ir(lowered_arg) {
                        *lowered_arg = Self::ensure_some_box_inner_for_ir(lowered_arg.clone());
                    }
                } else {
                    *lowered_arg = Self::ensure_some_box_inner_for_ir(lowered_arg.clone());
                }
            }
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec![
                    emitted_class_name,
                    "new".to_string(),
                ])),
                args: lowered_args,
            }));
        }
        if let HirExpr::FString { parts, .. } = expr {
            let mut format_str = String::new();
            let mut args = Vec::new();
            for part in parts {
                match part {
                    HirFStringPart::Literal(text) => {
                        format_str.push_str(&text.replace('{', "{{").replace('}', "}}"));
                    }
                    HirFStringPart::Expr(inner) => {
                        let Some(lowered_inner) = self.lower_stmt_expr_for_ir(inner)? else {
                            return Ok(None);
                        };
                        format_str.push_str("{}");
                        args.push(lowered_inner);
                    }
                }
            }
            return Ok(Some(crate::RustExpr::FormatMacro {
                name: "format".to_string(),
                format_str,
                args,
            }));
        }
        if let HirExpr::ListLiteral { elements, ty } = expr {
            let mut lowered_elements = Vec::with_capacity(elements.len());
            let list_ty = crate::resolve_alias_type_for_plain_call(ty);
            for element in elements {
                let Some(mut lowered_element) = self.lower_stmt_expr_for_ir(element)? else {
                    return Ok(None);
                };
                if matches!(list_ty, Type::Bytes) {
                    lowered_element = crate::RustExpr::Cast {
                        expr: Box::new(lowered_element),
                        ty: crate::RustType::Named("u8".to_string()),
                    };
                }
                lowered_elements.push(lowered_element);
            }
            return Ok(Some(crate::RustExpr::Vec(lowered_elements)));
        }
        if let HirExpr::TupleLiteral { elements, .. } = expr {
            let mut lowered_elements = Vec::with_capacity(elements.len());
            for element in elements {
                let Some(lowered_element) = self.lower_stmt_expr_for_ir(element)? else {
                    return Ok(None);
                };
                lowered_elements.push(lowered_element);
            }
            return Ok(Some(crate::RustExpr::Tuple(lowered_elements)));
        }
        if let HirExpr::DictLiteral { keys, values, .. } = expr {
            if keys.len() != values.len() {
                return Ok(None);
            }
            let mut stmts = Vec::with_capacity(keys.len() + 1);
            stmts.push(crate::RustStmt::Let {
                mutable: true,
                name: "__dict".to_string(),
                ty: None,
                value: crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "HashMap".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                },
            });
            for (key, value) in keys.iter().zip(values.iter()) {
                let Some(lowered_key) = self.lower_stmt_expr_for_ir(key)? else {
                    return Ok(None);
                };
                let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
                    return Ok(None);
                };
                stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Ident("__dict".to_string())),
                    method: "insert".to_string(),
                    args: vec![lowered_key, lowered_value],
                }));
            }
            return Ok(Some(crate::RustExpr::Block {
                stmts,
                expr: Some(Box::new(crate::RustExpr::Ident("__dict".to_string()))),
            }));
        }
        if let HirExpr::SetLiteral { elements, .. } = expr {
            let mut stmts = Vec::with_capacity(elements.len() + 1);
            stmts.push(crate::RustStmt::Let {
                mutable: true,
                name: "__set".to_string(),
                ty: None,
                value: crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "HashSet".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                },
            });
            for element in elements {
                let Some(lowered_element) = self.lower_stmt_expr_for_ir(element)? else {
                    return Ok(None);
                };
                stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Ident("__set".to_string())),
                    method: "insert".to_string(),
                    args: vec![lowered_element],
                }));
            }
            return Ok(Some(crate::RustExpr::Block {
                stmts,
                expr: Some(Box::new(crate::RustExpr::Ident("__set".to_string()))),
            }));
        }
        if let Some(lowered_comprehension) = self.try_lower_comprehension_expr_for_ir(expr)? {
            return Ok(Some(lowered_comprehension));
        }
        if let HirExpr::Call { func, args, .. } = expr {
            if let Some(lowered_intrinsic) = self.try_lower_registry_intrinsic_call_expr(func, args)
            {
                return Ok(Some(lowered_intrinsic));
            }
            if let Some(lowered_builtin) = self.try_lower_registry_builtin_call_expr(func, args) {
                return Ok(Some(lowered_builtin));
            }
            if let Some(lowered_plain) =
                self.try_lower_registry_plain_call_with_signature(func, args)
            {
                return Ok(Some(lowered_plain));
            }
            if func == "iter" && args.len() == 1 {
                return self.lower_iter_source_expr_for_ir(&args[0]);
            }
            if func == "next" && args.len() == 1 {
                let Some(lowered_iterator) = self.lower_stmt_expr_for_ir(&args[0])? else {
                    return Ok(None);
                };
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_iterator),
                    method: "next".to_string(),
                    args: vec![],
                }));
            }
            if func == "str" && args.is_empty() {
                return Ok(Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "String".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                }));
            }
            if func == "str" && args.len() == 1 {
                let arg = &args[0];
                let Some(lowered_arg) = self.lower_stmt_expr_for_ir(arg)? else {
                    return Ok(None);
                };
                if let Some(inner) = Self::option_inner_type_for_ir(arg.ty()) {
                    let format_str = if Self::uses_debug_display_format_for_ir(inner) {
                        "{:?}".to_string()
                    } else {
                        "{}".to_string()
                    };
                    return Ok(Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
                        method: "map_or".to_string(),
                        args: vec![
                            crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Literal(
                                    crate::RustLiteral::Str("None".to_string()),
                                )),
                                method: "to_string".to_string(),
                                args: vec![],
                            },
                            crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__v".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::FormatMacro {
                                    name: "format".to_string(),
                                    format_str,
                                    args: vec![crate::RustExpr::Ident("__v".to_string())],
                                }),
                                is_move: false,
                            },
                        ],
                    }));
                }
                return Ok(Some(crate::RustExpr::FormatMacro {
                    name: "format".to_string(),
                    format_str: if Self::uses_debug_display_format_for_ir(arg.ty()) {
                        "{:?}".to_string()
                    } else {
                        "{}".to_string()
                    },
                    args: vec![lowered_arg],
                }));
            }
            let mut lowered_args = Vec::with_capacity(args.len());
            for arg in args {
                let Some(lowered_arg) = self.lower_stmt_expr_for_ir(arg)? else {
                    return Ok(None);
                };
                lowered_args.push(lowered_arg);
            }
            lowered_args =
                self.adapt_plain_call_args_with_signature_for_ir(func, args, lowered_args);
            if let Some(captures) = self.nested_fn_captures.get(func).cloned() {
                for capture in captures {
                    lowered_args.push(self.lower_recursive_capture_arg_for_ir(&capture));
                }
            }
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(
                    func.split("::").map(ToString::to_string).collect(),
                )),
                args: lowered_args,
            }));
        }
        if let HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } = expr
        {
            let needs_field_clone_suppression =
                self.method_call_needs_field_clone_suppression(object, method);
            let suppression_prev = self.pending_self_field_clone_suppression;
            if needs_field_clone_suppression {
                self.pending_self_field_clone_suppression += 1;
            }
            let lowered_registry = self.try_lower_registry_method_call_expr(
                crate::resolve_alias_type_for_plain_call(object.ty()),
                object,
                method,
                args,
            );
            if needs_field_clone_suppression
                && self.pending_self_field_clone_suppression > suppression_prev
            {
                self.pending_self_field_clone_suppression -= 1;
            }
            if let Some(lowered_registry) = lowered_registry {
                return Ok(Some(lowered_registry));
            }

            let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
                return Ok(None);
            };
            let mut lowered_args = Vec::with_capacity(args.len());
            for arg in args {
                let Some(lowered_arg) = self.lower_stmt_expr_for_ir(arg)? else {
                    return Ok(None);
                };
                lowered_args.push(lowered_arg);
            }
            if method == "cloned"
                && lowered_args.is_empty()
                && matches!(
                    crate::resolve_alias_type_for_plain_call(object.ty()),
                    Type::List(_)
                )
            {
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_object),
                    method: "clone".to_string(),
                    args: vec![],
                }));
            }
            if method == "cloned" && lowered_args.is_empty() {
                let collected_vec = match &lowered_object {
                    crate::RustExpr::MethodCall { method, .. } => {
                        method == "collect" || method.starts_with("collect::<")
                    }
                    crate::RustExpr::Paren(inner) => {
                        matches!(
                            inner.as_ref(),
                            crate::RustExpr::MethodCall { method, .. }
                                if method == "collect" || method.starts_with("collect::<")
                        )
                    }
                    _ => false,
                };
                if collected_vec {
                    return Ok(Some(lowered_object));
                }
            }
            if let Some(method_params) = self.resolve_registry_method_params(object.ty(), method) {
                for (idx, lowered_arg) in lowered_args.iter_mut().enumerate() {
                    if let (Some((param_ty, convention)), Some(arg)) =
                        (method_params.get(idx), args.get(idx))
                    {
                        *lowered_arg = self.apply_registry_method_arg_convention(
                            arg,
                            param_ty,
                            *convention,
                            lowered_arg.clone(),
                        );
                    }
                }
            }
            let lowered_method = crate::RustExpr::MethodCall {
                receiver: Box::new(lowered_object),
                method: method.clone(),
                args: lowered_args,
            };
            if matches!(
                crate::resolve_alias_type_for_plain_call(expr.ty()),
                Type::Int
            ) && matches!(method.as_str(), "len" | "count")
            {
                return Ok(Some(crate::RustExpr::Cast {
                    expr: Box::new(lowered_method),
                    ty: crate::RustType::I64,
                }));
            }
            return Ok(Some(lowered_method));
        }
        if let HirExpr::QuestionMark { expr: inner, .. } = expr {
            let Some(lowered_inner) = self.lower_stmt_expr_for_ir(inner)? else {
                return Ok(None);
            };
            if let Some(target_err_ty) = self.try_closure_error_type.last().cloned() {
                let resolved_inner_ty = crate::resolve_alias_type_for_plain_call(inner.ty());
                if let Type::Result(_, inner_err_ty) = resolved_inner_ty {
                    let inner_err_ty_name =
                        crate::render_type(&crate::sifr_type_to_rust_type(inner_err_ty));
                    if inner_err_ty_name != target_err_ty
                        && can_construct_error_from_message_for_ir(&target_err_ty)
                    {
                        let ctor_func = if target_err_ty.contains("::") {
                            let mut path: Vec<String> =
                                target_err_ty.split("::").map(str::to_string).collect();
                            path.push("new".to_string());
                            crate::RustExpr::Path(path)
                        } else {
                            crate::RustExpr::Path(vec![target_err_ty.clone(), "new".to_string()])
                        };
                        return Ok(Some(crate::RustExpr::Try(Box::new(
                            crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_inner))),
                                method: "map_err".to_string(),
                                args: vec![crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__e".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(crate::RustExpr::FnCall {
                                        func: Box::new(ctor_func),
                                        args: vec![crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Ident(
                                                "__e".to_string(),
                                            )),
                                            method: "to_string".to_string(),
                                            args: vec![],
                                        }],
                                    }),
                                    is_move: false,
                                }],
                            },
                        ))));
                    }
                }
            }
            return Ok(Some(crate::RustExpr::Try(Box::new(lowered_inner))));
        }
        if let HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } = expr
        {
            let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
                return Ok(None);
            };
            if let Some(step_expr) = step {
                let Some(lowered_step) = self.lower_stmt_expr_for_ir(step_expr)? else {
                    return Ok(None);
                };

                let lowered_start = if let Some(start_expr) = start {
                    let Some(start_lowered) = self.lower_stmt_expr_for_ir(start_expr)? else {
                        return Ok(None);
                    };
                    crate::RustExpr::Block {
                        stmts: vec![crate::RustStmt::Let {
                            mutable: false,
                            name: "_sv".to_string(),
                            ty: None,
                            value: start_lowered,
                        }],
                        expr: Some(Box::new(crate::RustExpr::If {
                            cond: Box::new(crate::RustExpr::BinOp {
                                left: Box::new(crate::RustExpr::Ident("_sv".to_string())),
                                op: "<".to_string(),
                                right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(
                                    0,
                                ))),
                            }),
                            then_expr: Box::new(crate::RustExpr::Cast {
                                expr: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                        crate::RustExpr::BinOp {
                                            left: Box::new(crate::RustExpr::Ident(
                                                "_len".to_string(),
                                            )),
                                            op: "+".to_string(),
                                            right: Box::new(crate::RustExpr::Ident(
                                                "_sv".to_string(),
                                            )),
                                        },
                                    ))),
                                    method: "max".to_string(),
                                    args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(
                                        0,
                                    ))],
                                }),
                                ty: crate::RustType::Named("usize".to_string()),
                            }),
                            else_expr: Some(Box::new(crate::RustExpr::Cast {
                                expr: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident("_sv".to_string())),
                                    method: "min".to_string(),
                                    args: vec![crate::RustExpr::Ident("_len".to_string())],
                                }),
                                ty: crate::RustType::Named("usize".to_string()),
                            })),
                        })),
                    }
                } else {
                    crate::RustExpr::If {
                        cond: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Ident("_step".to_string())),
                            op: ">".to_string(),
                            right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                        }),
                        then_expr: Box::new(crate::RustExpr::Cast {
                            expr: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                            ty: crate::RustType::Named("usize".to_string()),
                        }),
                        else_expr: Some(Box::new(crate::RustExpr::Cast {
                            expr: Box::new(crate::RustExpr::Paren(Box::new(
                                crate::RustExpr::BinOp {
                                    left: Box::new(crate::RustExpr::Ident("_len".to_string())),
                                    op: "-".to_string(),
                                    right: Box::new(crate::RustExpr::Literal(
                                        crate::RustLiteral::Int(1),
                                    )),
                                },
                            ))),
                            ty: crate::RustType::Named("usize".to_string()),
                        })),
                    }
                };

                let lowered_stop = if let Some(stop_expr) = stop {
                    let Some(stop_lowered) = self.lower_stmt_expr_for_ir(stop_expr)? else {
                        return Ok(None);
                    };
                    crate::RustExpr::Block {
                        stmts: vec![crate::RustStmt::Let {
                            mutable: false,
                            name: "_ev".to_string(),
                            ty: None,
                            value: stop_lowered,
                        }],
                        expr: Some(Box::new(crate::RustExpr::If {
                            cond: Box::new(crate::RustExpr::BinOp {
                                left: Box::new(crate::RustExpr::Ident("_ev".to_string())),
                                op: "<".to_string(),
                                right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(
                                    0,
                                ))),
                            }),
                            then_expr: Box::new(crate::RustExpr::Cast {
                                expr: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                        crate::RustExpr::BinOp {
                                            left: Box::new(crate::RustExpr::Ident(
                                                "_len".to_string(),
                                            )),
                                            op: "+".to_string(),
                                            right: Box::new(crate::RustExpr::Ident(
                                                "_ev".to_string(),
                                            )),
                                        },
                                    ))),
                                    method: "max".to_string(),
                                    args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(
                                        0,
                                    ))],
                                }),
                                ty: crate::RustType::Named("usize".to_string()),
                            }),
                            else_expr: Some(Box::new(crate::RustExpr::Cast {
                                expr: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident("_ev".to_string())),
                                    method: "min".to_string(),
                                    args: vec![crate::RustExpr::Ident("_len".to_string())],
                                }),
                                ty: crate::RustType::Named("usize".to_string()),
                            })),
                        })),
                    }
                } else {
                    crate::RustExpr::If {
                        cond: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Ident("_step".to_string())),
                            op: ">".to_string(),
                            right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                        }),
                        then_expr: Box::new(crate::RustExpr::Cast {
                            expr: Box::new(crate::RustExpr::Ident("_len".to_string())),
                            ty: crate::RustType::Named("usize".to_string()),
                        }),
                        else_expr: Some(Box::new(crate::RustExpr::Path(vec![
                            "usize".to_string(),
                            "MAX".to_string(),
                        ]))),
                    }
                };

                return match crate::resolve_alias_type_for_plain_call(object.ty()) {
                    Type::List(_) | Type::Bytes => Ok(Some(crate::RustExpr::Block {
                        stmts: vec![
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_v".to_string(),
                                ty: None,
                                value: crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Paren(Box::new(
                                        lowered_object,
                                    ))),
                                },
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_len".to_string(),
                                ty: None,
                                value: crate::RustExpr::Cast {
                                    expr: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident(
                                            "_v".to_string(),
                                        )),
                                        method: "len".to_string(),
                                        args: vec![],
                                    }),
                                    ty: crate::RustType::I64,
                                },
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_step".to_string(),
                                ty: None,
                                value: lowered_step,
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_start".to_string(),
                                ty: None,
                                value: lowered_start,
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_stop".to_string(),
                                ty: None,
                                value: lowered_stop,
                            },
                            crate::RustStmt::Let {
                                mutable: true,
                                name: "_result".to_string(),
                                ty: None,
                                value: crate::RustExpr::FnCall {
                                    func: Box::new(crate::RustExpr::Path(vec![
                                        "Vec".to_string(),
                                        "new".to_string(),
                                    ])),
                                    args: vec![],
                                },
                            },
                            crate::RustStmt::If {
                                cond: crate::RustExpr::BinOp {
                                    left: Box::new(crate::RustExpr::Ident("_step".to_string())),
                                    op: ">".to_string(),
                                    right: Box::new(crate::RustExpr::Literal(
                                        crate::RustLiteral::Int(0),
                                    )),
                                },
                                then_body: vec![
                                    crate::RustStmt::Let {
                                        mutable: true,
                                        name: "_i".to_string(),
                                        ty: None,
                                        value: crate::RustExpr::Ident("_start".to_string()),
                                    },
                                    crate::RustStmt::While {
                                        cond: crate::RustExpr::BinOp {
                                            left: Box::new(crate::RustExpr::Ident(
                                                "_i".to_string(),
                                            )),
                                            op: "<".to_string(),
                                            right: Box::new(crate::RustExpr::Ident(
                                                "_stop".to_string(),
                                            )),
                                        },
                                        body: vec![
                                            crate::RustStmt::IfLet {
                                                pattern: "Some(_el)".to_string(),
                                                expr: crate::RustExpr::MethodCall {
                                                    receiver: Box::new(crate::RustExpr::Ident(
                                                        "_v".to_string(),
                                                    )),
                                                    method: "get".to_string(),
                                                    args: vec![crate::RustExpr::Ident(
                                                        "_i".to_string(),
                                                    )],
                                                },
                                                then_body: vec![crate::RustStmt::Expr(
                                                    crate::RustExpr::MethodCall {
                                                        receiver: Box::new(crate::RustExpr::Ident(
                                                            "_result".to_string(),
                                                        )),
                                                        method: "push".to_string(),
                                                        args: vec![crate::RustExpr::Clone(
                                                            Box::new(crate::RustExpr::Ident(
                                                                "_el".to_string(),
                                                            )),
                                                        )],
                                                    },
                                                )],
                                                else_body: None,
                                            },
                                            crate::RustStmt::AugAssign {
                                                target: crate::RustExpr::Ident("_i".to_string()),
                                                op: "+".to_string(),
                                                value: crate::RustExpr::Cast {
                                                    expr: Box::new(crate::RustExpr::Ident(
                                                        "_step".to_string(),
                                                    )),
                                                    ty: crate::RustType::Named("usize".to_string()),
                                                },
                                            },
                                        ],
                                    },
                                ],
                                else_body: Some(vec![
                                    crate::RustStmt::Let {
                                        mutable: true,
                                        name: "_i".to_string(),
                                        ty: None,
                                        value: crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::Ident(
                                                "_start".to_string(),
                                            )),
                                            ty: crate::RustType::I64,
                                        },
                                    },
                                    crate::RustStmt::Let {
                                        mutable: false,
                                        name: "_stop_i".to_string(),
                                        ty: None,
                                        value: crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::Ident(
                                                "_stop".to_string(),
                                            )),
                                            ty: crate::RustType::I64,
                                        },
                                    },
                                    crate::RustStmt::While {
                                        cond: crate::RustExpr::BinOp {
                                            left: Box::new(crate::RustExpr::Ident(
                                                "_i".to_string(),
                                            )),
                                            op: ">".to_string(),
                                            right: Box::new(crate::RustExpr::Ident(
                                                "_stop_i".to_string(),
                                            )),
                                        },
                                        body: vec![
                                            crate::RustStmt::If {
                                                cond: crate::RustExpr::BinOp {
                                                    left: Box::new(crate::RustExpr::Ident(
                                                        "_i".to_string(),
                                                    )),
                                                    op: ">=".to_string(),
                                                    right: Box::new(crate::RustExpr::Literal(
                                                        crate::RustLiteral::Int(0),
                                                    )),
                                                },
                                                then_body: vec![crate::RustStmt::IfLet {
                                                    pattern: "Some(_el)".to_string(),
                                                    expr: crate::RustExpr::MethodCall {
                                                        receiver: Box::new(crate::RustExpr::Ident(
                                                            "_v".to_string(),
                                                        )),
                                                        method: "get".to_string(),
                                                        args: vec![crate::RustExpr::Cast {
                                                            expr: Box::new(crate::RustExpr::Ident(
                                                                "_i".to_string(),
                                                            )),
                                                            ty: crate::RustType::Named(
                                                                "usize".to_string(),
                                                            ),
                                                        }],
                                                    },
                                                    then_body: vec![crate::RustStmt::Expr(
                                                        crate::RustExpr::MethodCall {
                                                            receiver: Box::new(
                                                                crate::RustExpr::Ident(
                                                                    "_result".to_string(),
                                                                ),
                                                            ),
                                                            method: "push".to_string(),
                                                            args: vec![crate::RustExpr::Clone(
                                                                Box::new(crate::RustExpr::Ident(
                                                                    "_el".to_string(),
                                                                )),
                                                            )],
                                                        },
                                                    )],
                                                    else_body: None,
                                                }],
                                                else_body: None,
                                            },
                                            crate::RustStmt::AugAssign {
                                                target: crate::RustExpr::Ident("_i".to_string()),
                                                op: "+".to_string(),
                                                value: crate::RustExpr::Ident("_step".to_string()),
                                            },
                                        ],
                                    },
                                ]),
                            },
                        ],
                        expr: Some(Box::new(crate::RustExpr::Ident("_result".to_string()))),
                    })),
                    Type::Str => Ok(Some(crate::RustExpr::Block {
                        stmts: vec![
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_s".to_string(),
                                ty: None,
                                value: crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Paren(Box::new(
                                        lowered_object,
                                    ))),
                                },
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_len".to_string(),
                                ty: None,
                                value: crate::RustExpr::Cast {
                                    expr: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Ident(
                                                "_s".to_string(),
                                            )),
                                            method: "chars".to_string(),
                                            args: vec![],
                                        }),
                                        method: "count".to_string(),
                                        args: vec![],
                                    }),
                                    ty: crate::RustType::I64,
                                },
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_step".to_string(),
                                ty: None,
                                value: lowered_step,
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_start".to_string(),
                                ty: None,
                                value: lowered_start,
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_stop".to_string(),
                                ty: None,
                                value: lowered_stop,
                            },
                            crate::RustStmt::Let {
                                mutable: true,
                                name: "_result".to_string(),
                                ty: None,
                                value: crate::RustExpr::FnCall {
                                    func: Box::new(crate::RustExpr::Path(vec![
                                        "String".to_string(),
                                        "new".to_string(),
                                    ])),
                                    args: vec![],
                                },
                            },
                            crate::RustStmt::If {
                                cond: crate::RustExpr::BinOp {
                                    left: Box::new(crate::RustExpr::Ident("_step".to_string())),
                                    op: ">".to_string(),
                                    right: Box::new(crate::RustExpr::Literal(
                                        crate::RustLiteral::Int(0),
                                    )),
                                },
                                then_body: vec![
                                    crate::RustStmt::Let {
                                        mutable: true,
                                        name: "_i".to_string(),
                                        ty: None,
                                        value: crate::RustExpr::Ident("_start".to_string()),
                                    },
                                    crate::RustStmt::While {
                                        cond: crate::RustExpr::BinOp {
                                            left: Box::new(crate::RustExpr::Ident(
                                                "_i".to_string(),
                                            )),
                                            op: "<".to_string(),
                                            right: Box::new(crate::RustExpr::Ident(
                                                "_stop".to_string(),
                                            )),
                                        },
                                        body: vec![
                                            crate::RustStmt::IfLet {
                                                pattern: "Some(_ch)".to_string(),
                                                expr: crate::RustExpr::MethodCall {
                                                    receiver: Box::new(
                                                        crate::RustExpr::MethodCall {
                                                            receiver: Box::new(
                                                                crate::RustExpr::Ident(
                                                                    "_s".to_string(),
                                                                ),
                                                            ),
                                                            method: "chars".to_string(),
                                                            args: vec![],
                                                        },
                                                    ),
                                                    method: "nth".to_string(),
                                                    args: vec![crate::RustExpr::Ident(
                                                        "_i".to_string(),
                                                    )],
                                                },
                                                then_body: vec![crate::RustStmt::Expr(
                                                    crate::RustExpr::MethodCall {
                                                        receiver: Box::new(crate::RustExpr::Ident(
                                                            "_result".to_string(),
                                                        )),
                                                        method: "push".to_string(),
                                                        args: vec![crate::RustExpr::Ident(
                                                            "_ch".to_string(),
                                                        )],
                                                    },
                                                )],
                                                else_body: None,
                                            },
                                            crate::RustStmt::AugAssign {
                                                target: crate::RustExpr::Ident("_i".to_string()),
                                                op: "+".to_string(),
                                                value: crate::RustExpr::Cast {
                                                    expr: Box::new(crate::RustExpr::Ident(
                                                        "_step".to_string(),
                                                    )),
                                                    ty: crate::RustType::Named("usize".to_string()),
                                                },
                                            },
                                        ],
                                    },
                                ],
                                else_body: Some(vec![
                                    crate::RustStmt::Let {
                                        mutable: true,
                                        name: "_i".to_string(),
                                        ty: None,
                                        value: crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::Ident(
                                                "_start".to_string(),
                                            )),
                                            ty: crate::RustType::I64,
                                        },
                                    },
                                    crate::RustStmt::Let {
                                        mutable: false,
                                        name: "_stop_i".to_string(),
                                        ty: None,
                                        value: crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::Ident(
                                                "_stop".to_string(),
                                            )),
                                            ty: crate::RustType::I64,
                                        },
                                    },
                                    crate::RustStmt::While {
                                        cond: crate::RustExpr::BinOp {
                                            left: Box::new(crate::RustExpr::Ident(
                                                "_i".to_string(),
                                            )),
                                            op: ">".to_string(),
                                            right: Box::new(crate::RustExpr::Ident(
                                                "_stop_i".to_string(),
                                            )),
                                        },
                                        body: vec![
                                            crate::RustStmt::If {
                                                cond: crate::RustExpr::BinOp {
                                                    left: Box::new(crate::RustExpr::Ident(
                                                        "_i".to_string(),
                                                    )),
                                                    op: ">=".to_string(),
                                                    right: Box::new(crate::RustExpr::Literal(
                                                        crate::RustLiteral::Int(0),
                                                    )),
                                                },
                                                then_body: vec![crate::RustStmt::IfLet {
                                                    pattern: "Some(_ch)".to_string(),
                                                    expr: crate::RustExpr::MethodCall {
                                                        receiver: Box::new(
                                                            crate::RustExpr::MethodCall {
                                                                receiver: Box::new(
                                                                    crate::RustExpr::Ident(
                                                                        "_s".to_string(),
                                                                    ),
                                                                ),
                                                                method: "chars".to_string(),
                                                                args: vec![],
                                                            },
                                                        ),
                                                        method: "nth".to_string(),
                                                        args: vec![crate::RustExpr::Cast {
                                                            expr: Box::new(crate::RustExpr::Ident(
                                                                "_i".to_string(),
                                                            )),
                                                            ty: crate::RustType::Named(
                                                                "usize".to_string(),
                                                            ),
                                                        }],
                                                    },
                                                    then_body: vec![crate::RustStmt::Expr(
                                                        crate::RustExpr::MethodCall {
                                                            receiver: Box::new(
                                                                crate::RustExpr::Ident(
                                                                    "_result".to_string(),
                                                                ),
                                                            ),
                                                            method: "push".to_string(),
                                                            args: vec![crate::RustExpr::Ident(
                                                                "_ch".to_string(),
                                                            )],
                                                        },
                                                    )],
                                                    else_body: None,
                                                }],
                                                else_body: None,
                                            },
                                            crate::RustStmt::AugAssign {
                                                target: crate::RustExpr::Ident("_i".to_string()),
                                                op: "+".to_string(),
                                                value: crate::RustExpr::Ident("_step".to_string()),
                                            },
                                        ],
                                    },
                                ]),
                            },
                        ],
                        expr: Some(Box::new(crate::RustExpr::Ident("_result".to_string()))),
                    })),
                    _ => Ok(None),
                };
            }
            let lowered_start_i64 = if let Some(start_expr) = start {
                let Some(start_lowered) = self.lower_stmt_expr_for_ir(start_expr)? else {
                    return Ok(None);
                };
                crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(start_lowered))),
                    method: "max".to_string(),
                    args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                }
            } else {
                crate::RustExpr::Literal(crate::RustLiteral::Int(0))
            };
            let lowered_start = crate::RustExpr::Cast {
                expr: Box::new(lowered_start_i64.clone()),
                ty: crate::RustType::Named("usize".to_string()),
            };

            let lowered_take_count = if let Some(stop_expr) = stop {
                let Some(stop_lowered) = self.lower_stmt_expr_for_ir(stop_expr)? else {
                    return Ok(None);
                };
                let clamped_stop_i64 = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(stop_lowered))),
                    method: "max".to_string(),
                    args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                };
                Some(crate::RustExpr::Cast {
                    expr: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(
                            crate::RustExpr::BinOp {
                                left: Box::new(clamped_stop_i64),
                                op: "-".to_string(),
                                right: Box::new(lowered_start_i64.clone()),
                            },
                        ))),
                        method: "max".to_string(),
                        args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                    }),
                    ty: crate::RustType::Named("usize".to_string()),
                })
            } else {
                None
            };

            match crate::resolve_alias_type_for_plain_call(object.ty()) {
                Type::Str => {
                    let mut iter = crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_object))),
                            method: "chars".to_string(),
                            args: vec![],
                        }),
                        method: "skip".to_string(),
                        args: vec![lowered_start],
                    };
                    if let Some(take_count) = lowered_take_count {
                        iter = crate::RustExpr::MethodCall {
                            receiver: Box::new(iter),
                            method: "take".to_string(),
                            args: vec![take_count],
                        };
                    }
                    return Ok(Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "String".to_string(),
                            "from_iter".to_string(),
                        ])),
                        args: vec![iter],
                    }));
                }
                Type::List(_) | Type::Bytes => {
                    let base_iter = crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_object))),
                            method: "iter".to_string(),
                            args: vec![],
                        }),
                        method: "skip".to_string(),
                        args: vec![lowered_start],
                    };
                    let iter = if let Some(take_count) = lowered_take_count {
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(base_iter),
                                method: "take".to_string(),
                                args: vec![take_count],
                            }),
                            method: "cloned".to_string(),
                            args: vec![],
                        }
                    } else {
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(base_iter),
                            method: "cloned".to_string(),
                            args: vec![],
                        }
                    };
                    return Ok(Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "Vec".to_string(),
                            "from_iter".to_string(),
                        ])),
                        args: vec![iter],
                    }));
                }
                _ => return Ok(None),
            }
        }
        if let HirExpr::OkWrap { value, .. } = expr {
            let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
                return Ok(None);
            };
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                args: vec![lowered_value],
            }));
        }
        if let HirExpr::ErrWrap { value, .. } = expr {
            let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
                return Ok(None);
            };
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Err".to_string()])),
                args: vec![lowered_value],
            }));
        }
        if let HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } = expr
        {
            let Some(lowered_condition) = self.lower_stmt_expr_for_ir(condition)? else {
                return Ok(None);
            };
            let Some(lowered_then) = self.lower_stmt_expr_for_ir(then_expr)? else {
                return Ok(None);
            };
            let Some(lowered_else) = self.lower_stmt_expr_for_ir(else_expr)? else {
                return Ok(None);
            };
            return Ok(Some(crate::RustExpr::If {
                cond: Box::new(lowered_condition),
                then_expr: Box::new(lowered_then),
                else_expr: Some(Box::new(lowered_else)),
            }));
        }
        if let HirExpr::RangeLiteral {
            start, end, step, ..
        } = expr
        {
            let Some(lowered_start) = self.lower_stmt_expr_for_ir(start)? else {
                return Ok(None);
            };
            let Some(lowered_end) = self.lower_stmt_expr_for_ir(end)? else {
                return Ok(None);
            };
            let lowered_range = crate::RustExpr::Range {
                start: Box::new(lowered_start),
                end: Box::new(lowered_end),
            };
            if let Some(step_expr) = step {
                let Some(lowered_step) = self.lower_stmt_expr_for_ir(step_expr)? else {
                    return Ok(None);
                };
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_range),
                    method: "step_by".to_string(),
                    args: vec![crate::RustExpr::Cast {
                        expr: Box::new(lowered_step),
                        ty: crate::RustType::Named("usize".to_string()),
                    }],
                }));
            }
            return Ok(Some(lowered_range));
        }
        if let HirExpr::Index {
            object, index, ty, ..
        } = expr
        {
            if !crate::helpers::is_option_type(ty) {
                if let Some(lowered) = self.lower_non_option_index_expr_for_ir(object, index)? {
                    return Ok(Some(lowered));
                }
            }
            if let Some(lowered) = self.try_lower_structured_index_expr(object, index, ty)? {
                return Ok(Some(lowered));
            }
            let object_ty = crate::resolve_alias_type_for_plain_call(object.ty());
            let index_returns_option = crate::helpers::is_option_type(ty);
            let option_inner_ty = if let Type::Union(members) = object_ty {
                let mut non_none = members.iter().filter(|m| !matches!(m, Type::None));
                let first = non_none.next();
                if non_none.next().is_none() && members.iter().any(|m| matches!(m, Type::None)) {
                    first
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(inner_ty) = option_inner_ty {
                let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
                    return Ok(None);
                };
                let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
                    return Ok(None);
                };
                let option_index_expr = match inner_ty {
                    Type::Dict(_, _) => {
                        let key_arg = if matches!(index.as_ref(), HirExpr::StringLiteral(_)) {
                            lowered_index
                        } else {
                            crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(lowered_index),
                            }
                        };
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident("__v".to_string())),
                                method: "get".to_string(),
                                args: vec![key_arg],
                            }),
                            method: "cloned".to_string(),
                            args: vec![],
                        }
                    }
                    Type::List(_) => crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident("__v".to_string())),
                            method: "get".to_string(),
                            args: vec![crate::RustExpr::Cast {
                                expr: Box::new(lowered_index),
                                ty: crate::RustType::Named("usize".to_string()),
                            }],
                        }),
                        method: "cloned".to_string(),
                        args: vec![],
                    },
                    Type::Bytes => crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident("__v".to_string())),
                            method: "get".to_string(),
                            args: vec![crate::RustExpr::Cast {
                                expr: Box::new(lowered_index),
                                ty: crate::RustType::Named("usize".to_string()),
                            }],
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
                                ty: crate::RustType::I64,
                            }),
                            is_move: false,
                        }],
                    },
                    Type::Str => crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident("__v".to_string())),
                                method: "chars".to_string(),
                                args: vec![],
                            }),
                            method: "nth".to_string(),
                            args: vec![crate::RustExpr::Cast {
                                expr: Box::new(lowered_index),
                                ty: crate::RustType::Named("usize".to_string()),
                            }],
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
                    },
                    _ => return Ok(None),
                };
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(lowered_object),
                        method: "as_ref".to_string(),
                        args: vec![],
                    }),
                    method: "and_then".to_string(),
                    args: vec![crate::RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "__v".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(option_index_expr),
                        is_move: false,
                    }],
                }));
            }

            let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
                return Ok(None);
            };
            let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
                return Ok(None);
            };
            match object_ty {
                Type::Dict(_, _) => {
                    let key_arg = if matches!(index.as_ref(), HirExpr::StringLiteral(_)) {
                        lowered_index
                    } else {
                        crate::RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(lowered_index),
                        }
                    };
                    if index_returns_option {
                        return Ok(Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(lowered_object),
                                method: "get".to_string(),
                                args: vec![key_arg],
                            }),
                            method: "cloned".to_string(),
                            args: vec![],
                        }));
                    }
                    return Ok(Some(crate::RustExpr::Clone(Box::new(
                        crate::RustExpr::Index {
                            expr: Box::new(lowered_object),
                            index: Box::new(key_arg),
                        },
                    ))));
                }
                Type::List(_) => {
                    let list_index = crate::RustExpr::Cast {
                        expr: Box::new(lowered_index),
                        ty: crate::RustType::Named("usize".to_string()),
                    };
                    if index_returns_option {
                        return Ok(Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(lowered_object),
                                method: "get".to_string(),
                                args: vec![list_index],
                            }),
                            method: "cloned".to_string(),
                            args: vec![],
                        }));
                    }
                    return Ok(Some(crate::RustExpr::Clone(Box::new(
                        crate::RustExpr::Index {
                            expr: Box::new(lowered_object),
                            index: Box::new(list_index),
                        },
                    ))));
                }
                Type::Bytes => {
                    let list_index = crate::RustExpr::Cast {
                        expr: Box::new(lowered_index),
                        ty: crate::RustType::Named("usize".to_string()),
                    };
                    if index_returns_option {
                        return Ok(Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(lowered_object),
                                method: "get".to_string(),
                                args: vec![list_index],
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
                                    ty: crate::RustType::I64,
                                }),
                                is_move: false,
                            }],
                        }));
                    }
                    return Ok(Some(crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::Clone(Box::new(crate::RustExpr::Index {
                            expr: Box::new(lowered_object),
                            index: Box::new(list_index),
                        }))),
                        ty: crate::RustType::I64,
                    }));
                }
                Type::Str => {
                    let nth_expr = crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(lowered_object),
                            method: "chars".to_string(),
                            args: vec![],
                        }),
                        method: "nth".to_string(),
                        args: vec![crate::RustExpr::Cast {
                            expr: Box::new(lowered_index),
                            ty: crate::RustType::Named("usize".to_string()),
                        }],
                    };
                    if index_returns_option {
                        return Ok(Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(nth_expr),
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
                        }));
                    }
                    return Err(crate::CodegenError::new(
                        "internal codegen invariant violated: string index produced non-optional result type",
                    ));
                }
                Type::Tuple(_) => {
                    let HirExpr::IntLiteral(idx) = index.as_ref() else {
                        return Ok(None);
                    };
                    return Ok(Some(crate::RustExpr::Field {
                        expr: Box::new(lowered_object),
                        field: idx.to_string(),
                    }));
                }
                _ => {}
            }
        }
        if let HirExpr::ContainsOp {
            element,
            collection,
            ..
        } = expr
        {
            let Some(lowered_element) = self.lower_stmt_expr_for_ir(element)? else {
                return Ok(None);
            };
            let Some(lowered_collection) = self.lower_stmt_expr_for_ir(collection)? else {
                return Ok(None);
            };
            let lowered = match crate::resolve_alias_type_for_plain_call(collection.ty()) {
                Type::Dict(_, _) => {
                    let key_arg = if let HirExpr::StringLiteral(value) = element.as_ref() {
                        crate::RustExpr::Literal(crate::RustLiteral::Str(value.clone()))
                    } else if let HirExpr::Name { name, ty } = element.as_ref() {
                        if self.borrowed_params.contains(name)
                            || self.mut_borrowed_params.contains(name)
                        {
                            if matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Str) {
                                crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                        lowered_element,
                                    ))),
                                    method: "as_str".to_string(),
                                    args: vec![],
                                }
                            } else {
                                lowered_element
                            }
                        } else {
                            crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_element))),
                            }
                        }
                    } else {
                        crate::RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_element))),
                        }
                    };
                    crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_collection))),
                        method: "contains_key".to_string(),
                        args: vec![key_arg],
                    }
                }
                Type::List(_) | Type::Set(_) | Type::Str => crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_collection))),
                    method: "contains".to_string(),
                    args: vec![crate::RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_element))),
                    }],
                },
                Type::Bytes => crate::RustExpr::Block {
                    stmts: vec![crate::RustStmt::Let {
                        mutable: false,
                        name: "__byte_candidate".to_string(),
                        ty: None,
                        value: lowered_element,
                    }],
                    expr: Some(Box::new(crate::RustExpr::If {
                        cond: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::BinOp {
                                left: Box::new(crate::RustExpr::Ident(
                                    "__byte_candidate".to_string(),
                                )),
                                op: "<".to_string(),
                                right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(
                                    0,
                                ))),
                            }),
                            op: "||".to_string(),
                            right: Box::new(crate::RustExpr::BinOp {
                                left: Box::new(crate::RustExpr::Ident(
                                    "__byte_candidate".to_string(),
                                )),
                                op: ">".to_string(),
                                right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(
                                    255,
                                ))),
                            }),
                        }),
                        then_expr: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Bool(
                            false,
                        ))),
                        else_expr: Some(Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                lowered_collection,
                            ))),
                            method: "contains".to_string(),
                            args: vec![crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(crate::RustExpr::Cast {
                                    expr: Box::new(crate::RustExpr::Ident(
                                        "__byte_candidate".to_string(),
                                    )),
                                    ty: crate::RustType::Named("u8".to_string()),
                                }),
                            }],
                        })),
                    })),
                },
                _ => return Ok(None),
            };
            return Ok(Some(lowered));
        }
        if let HirExpr::UnaryOp { op, operand, .. } = expr {
            if op == "not" {
                if let Some(option_var) = crate::helpers::detect_option_truthiness(operand) {
                    return Ok(Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(option_var)),
                        method: "is_none".to_string(),
                        args: vec![],
                    }));
                }
            }
            let Some(lowered_operand) = self.lower_stmt_expr_for_ir(operand)? else {
                return Ok(None);
            };
            let lowered = match op.as_str() {
                "not" => crate::RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(crate::RustExpr::Paren(Box::new(lowered_operand))),
                },
                "~" => crate::RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(crate::RustExpr::Paren(Box::new(lowered_operand))),
                },
                "-" => crate::RustExpr::UnaryOp {
                    op: "-".to_string(),
                    operand: Box::new(crate::RustExpr::Paren(Box::new(lowered_operand))),
                },
                "+" => crate::RustExpr::Paren(Box::new(lowered_operand)),
                _ => return Ok(None),
            };
            return Ok(Some(lowered));
        }
        if let HirExpr::Compare {
            left,
            ops,
            comparators,
            ..
        } = expr
        {
            if ops.len() == 1 && comparators.len() == 1 {
                let lowered_op = match ops[0].as_str() {
                    "==" | "!=" | "<" | "<=" | ">" | ">=" => ops[0].clone(),
                    "is" => "==".to_string(),
                    "is not" => "!=".to_string(),
                    _ => return Ok(None),
                };
                let Some(lowered_left) = self.lower_stmt_expr_for_ir(left)? else {
                    return Ok(None);
                };
                let Some(lowered_right) = self.lower_stmt_expr_for_ir(&comparators[0])? else {
                    return Ok(None);
                };
                let lowered_left = if matches!(left.as_ref(), HirExpr::Name { name, ty }
                    if (self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name))
                        && ty.ownership() != sifr_type_system::OwnershipKind::Copy)
                {
                    crate::RustExpr::Clone(Box::new(lowered_left))
                } else {
                    lowered_left
                };
                let lowered_right = if matches!(&comparators[0], HirExpr::Name { name, ty }
                    if (self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name))
                        && ty.ownership() != sifr_type_system::OwnershipKind::Copy)
                {
                    crate::RustExpr::Clone(Box::new(lowered_right))
                } else {
                    lowered_right
                };
                let left_is_option = crate::helpers::is_option_type(left.ty());
                let right_is_option = crate::helpers::is_option_type(comparators[0].ty());
                let left_none_like = matches!(left.as_ref(), HirExpr::NoneLiteral)
                    || matches!(
                        crate::resolve_alias_type_for_plain_call(left.ty()),
                        Type::None
                    );
                let right_none_like = matches!(&comparators[0], HirExpr::NoneLiteral)
                    || matches!(
                        crate::resolve_alias_type_for_plain_call(comparators[0].ty()),
                        Type::None
                    );
                let (lowered_left, lowered_right) = if matches!(lowered_op.as_str(), "==" | "!=") {
                    if left_is_option && !right_is_option && !right_none_like {
                        (
                            lowered_left,
                            crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                                args: vec![lowered_right],
                            },
                        )
                    } else if !left_is_option && right_is_option && !left_none_like {
                        (
                            crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                                args: vec![lowered_left],
                            },
                            lowered_right,
                        )
                    } else {
                        (lowered_left, lowered_right)
                    }
                } else {
                    (lowered_left, lowered_right)
                };
                return Ok(Some(crate::RustExpr::BinOp {
                    left: Box::new(lowered_left),
                    op: lowered_op,
                    right: Box::new(lowered_right),
                }));
            }
        }
        if let HirExpr::BoolOp { op, values, .. } = expr {
            let lowered_op = match op.as_str() {
                "and" => "&&",
                "or" => "||",
                _ => return Ok(None),
            };
            if values.is_empty() {
                return Ok(None);
            }
            let mut iter = values.iter();
            let Some(first) = iter.next() else {
                return Ok(None);
            };
            let Some(mut acc) = self.lower_stmt_expr_for_ir(first)? else {
                return Ok(None);
            };
            for value in iter {
                let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
                    return Ok(None);
                };
                acc = crate::RustExpr::BinOp {
                    left: Box::new(crate::RustExpr::Paren(Box::new(acc))),
                    op: lowered_op.to_string(),
                    right: Box::new(crate::RustExpr::Paren(Box::new(lowered_value))),
                };
            }
            return Ok(Some(crate::RustExpr::Paren(Box::new(acc))));
        }
        if let HirExpr::BinOp {
            left,
            op,
            right,
            ty,
        } = expr
        {
            if let Some(lowered) = self.try_lower_structured_class_binop_expr(left, op, right)? {
                return Ok(Some(lowered));
            }
            if let Some(lowered) = self.try_lower_stmt_string_concat_expr_for_ir(expr)? {
                return Ok(Some(lowered));
            }
            let Some(lowered_left) = self.lower_stmt_expr_for_ir(left)? else {
                return Ok(None);
            };
            let Some(lowered_right) = self.lower_stmt_expr_for_ir(right)? else {
                return Ok(None);
            };
            let resolved_result_ty = crate::resolve_alias_type_for_plain_call(ty);
            let resolved_left_ty = crate::resolve_alias_type_for_plain_call(left.ty());
            let resolved_right_ty = crate::resolve_alias_type_for_plain_call(right.ty());

            if op == "*" && matches!(resolved_result_ty, Type::Str) {
                let (string_expr, count_expr) = match (
                    matches!(resolved_left_ty, Type::Str),
                    matches!(resolved_right_ty, Type::Str),
                ) {
                    (true, false) => (lowered_left.clone(), lowered_right.clone()),
                    (false, true) => (lowered_right.clone(), lowered_left.clone()),
                    _ => return Ok(None),
                };
                return Ok(Some(crate::RustExpr::Block {
                    stmts: vec![crate::RustStmt::Let {
                        mutable: false,
                        name: "__n".to_string(),
                        ty: None,
                        value: count_expr,
                    }],
                    expr: Some(Box::new(crate::RustExpr::If {
                        cond: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Ident("__n".to_string())),
                            op: "<=".to_string(),
                            right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                        }),
                        then_expr: Box::new(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "String".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![],
                        }),
                        else_expr: Some(Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(string_expr))),
                            method: "repeat".to_string(),
                            args: vec![crate::RustExpr::Cast {
                                expr: Box::new(crate::RustExpr::Ident("__n".to_string())),
                                ty: crate::RustType::Named("usize".to_string()),
                            }],
                        })),
                    })),
                }));
            }

            if op == "+"
                && (matches!(resolved_result_ty, Type::List(_))
                    || matches!(resolved_result_ty, Type::Bytes))
                && (matches!(resolved_left_ty, Type::List(_))
                    || matches!(resolved_left_ty, Type::Bytes))
                && (matches!(resolved_right_ty, Type::List(_))
                    || matches!(resolved_right_ty, Type::Bytes))
            {
                return Ok(Some(crate::RustExpr::Block {
                    stmts: vec![
                        crate::RustStmt::Let {
                            mutable: true,
                            name: "__v".to_string(),
                            ty: None,
                            value: crate::RustExpr::Clone(Box::new(crate::RustExpr::Paren(
                                Box::new(lowered_left.clone()),
                            ))),
                        },
                        crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident("__v".to_string())),
                            method: "extend".to_string(),
                            args: vec![crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                        lowered_right.clone(),
                                    ))),
                                    method: "iter".to_string(),
                                    args: vec![],
                                }),
                                method: "cloned".to_string(),
                                args: vec![],
                            }],
                        }),
                    ],
                    expr: Some(Box::new(crate::RustExpr::Ident("__v".to_string()))),
                }));
            }

            let runtime_exit_block = |message: &str| crate::RustExpr::Block {
                stmts: vec![crate::RustStmt::Expr(crate::RustExpr::FormatMacro {
                    name: "eprintln".to_string(),
                    format_str: message.to_string(),
                    args: vec![],
                })],
                expr: Some(Box::new(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "std".to_string(),
                        "process".to_string(),
                        "exit".to_string(),
                    ])),
                    args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(1))],
                })),
            };
            let bigdecimal_default_context_expr = || {
                let base_ctx = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "bigdecimal".to_string(),
                            "Context".to_string(),
                            "default".to_string(),
                        ])),
                        args: vec![],
                    }),
                    method: "with_rounding_mode".to_string(),
                    args: vec![crate::RustExpr::Path(vec![
                        "bigdecimal".to_string(),
                        "RoundingMode".to_string(),
                        "HalfEven".to_string(),
                    ])],
                };
                crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(base_ctx.clone()),
                        method: "with_prec".to_string(),
                        args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(28))],
                    }),
                    method: "unwrap_or_else".to_string(),
                    args: vec![crate::RustExpr::Closure {
                        params: vec![],
                        body: Box::new(base_ctx),
                        is_move: false,
                    }],
                }
            };
            let round_bigdecimal_with_default_context =
                |value: crate::RustExpr| crate::RustExpr::MethodCall {
                    receiver: Box::new(bigdecimal_default_context_expr()),
                    method: "round_decimal_ref".to_string(),
                    args: vec![crate::RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(crate::RustExpr::Paren(Box::new(value))),
                    }],
                };

            let mut lowered_left = lowered_left;
            let mut lowered_right = lowered_right;
            let is_move_arith_op = matches!(op.as_str(), "+" | "-" | "*" | "/" | "//" | "%" | "**");
            if is_move_arith_op {
                if matches!(resolved_left_ty, Type::BigInt) {
                    lowered_left = crate::RustExpr::Clone(Box::new(lowered_left));
                }
                if matches!(resolved_right_ty, Type::BigInt) {
                    lowered_right = crate::RustExpr::Clone(Box::new(lowered_right));
                }
                if matches!(resolved_left_ty, Type::BigDecimal) {
                    lowered_left = crate::RustExpr::Clone(Box::new(lowered_left));
                }
                if matches!(resolved_right_ty, Type::BigDecimal) {
                    lowered_right = crate::RustExpr::Clone(Box::new(lowered_right));
                }
            }
            if matches!(
                resolved_result_ty,
                Type::Int | Type::Float | Type::LiteralInt(_) | Type::TypeVar(_) | Type::BigInt
            ) {
                if Self::option_inner_type_for_ir(ty).is_none() {
                    if Self::option_inner_type_for_ir(left.ty()).is_some()
                        || Self::option_inner_type_for_ir(right.ty()).is_some()
                    {
                        return Err(crate::CodegenError::new(
                            "internal codegen invariant violated: numeric expression kept optional operand in non-optional context",
                        ));
                    }
                }
                if matches!(resolved_result_ty, Type::Float) {
                    if matches!(resolved_left_ty, Type::Int | Type::LiteralInt(_)) {
                        lowered_left = crate::RustExpr::Cast {
                            expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_left))),
                            ty: crate::RustType::F64,
                        };
                    }
                    if matches!(resolved_right_ty, Type::Int | Type::LiteralInt(_)) {
                        lowered_right = crate::RustExpr::Cast {
                            expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_right))),
                            ty: crate::RustType::F64,
                        };
                    }
                }
            }

            if matches!(resolved_result_ty, Type::Decimal) {
                let lower_bigint_to_decimal =
                    |value: crate::RustExpr| crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "Decimal".to_string(),
                                "from_str_exact".to_string(),
                            ])),
                            args: vec![crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Paren(Box::new(value))),
                                    method: "to_string".to_string(),
                                    args: vec![],
                                }),
                                method: "as_str".to_string(),
                                args: vec![],
                            }],
                        }),
                        method: "unwrap_or_else".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__e".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::MacroCall {
                                name: "unreachable".to_string(),
                                args: vec![],
                            }),
                            is_move: false,
                        }],
                    };
                if matches!(resolved_left_ty, Type::Int | Type::LiteralInt(_)) {
                    lowered_left = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "Decimal".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![lowered_left],
                    };
                } else if matches!(resolved_left_ty, Type::BigInt) {
                    lowered_left = lower_bigint_to_decimal(lowered_left);
                }
                if op != "**" && matches!(resolved_right_ty, Type::Int | Type::LiteralInt(_)) {
                    lowered_right = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "Decimal".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![lowered_right],
                    };
                } else if op != "**" && matches!(resolved_right_ty, Type::BigInt) {
                    lowered_right = lower_bigint_to_decimal(lowered_right);
                }
            }

            if matches!(resolved_result_ty, Type::BigDecimal) {
                let lower_decimal_to_bigdecimal =
                    |value: crate::RustExpr| crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(value))),
                                method: "to_string".to_string(),
                                args: vec![],
                            }),
                            method: "parse::<BigDecimal>".to_string(),
                            args: vec![],
                        }),
                        method: "unwrap_or_else".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__e".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::MacroCall {
                                name: "unreachable".to_string(),
                                args: vec![],
                            }),
                            is_move: false,
                        }],
                    };
                if matches!(
                    resolved_left_ty,
                    Type::Int | Type::LiteralInt(_) | Type::BigInt
                ) {
                    lowered_left = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "BigDecimal".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![lowered_left],
                    };
                } else if matches!(resolved_left_ty, Type::Decimal) {
                    lowered_left = lower_decimal_to_bigdecimal(lowered_left);
                }
                if op != "**"
                    && matches!(
                        resolved_right_ty,
                        Type::Int | Type::LiteralInt(_) | Type::BigInt
                    )
                {
                    lowered_right = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "BigDecimal".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![lowered_right],
                    };
                } else if op != "**" && matches!(resolved_right_ty, Type::Decimal) {
                    lowered_right = lower_decimal_to_bigdecimal(lowered_right);
                }
            }

            if matches!(resolved_result_ty, Type::Decimal)
                && matches!(op.as_str(), "/" | "//" | "%")
            {
                let invalid_message = match op.as_str() {
                    "/" => "runtime error: decimal division failed (division by zero or overflow)",
                    "//" => {
                        "runtime error: decimal floor-division failed (division by zero or overflow)"
                    }
                    _ => "runtime error: decimal modulo failed (division by zero or overflow)",
                };
                let success_expr = match op.as_str() {
                    "/" => crate::RustExpr::Ident("__q".to_string()),
                    "//" => crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident("__q".to_string())),
                        method: "floor".to_string(),
                        args: vec![],
                    },
                    "%" => crate::RustExpr::BinOp {
                        left: Box::new(crate::RustExpr::Ident("__l".to_string())),
                        op: "-".to_string(),
                        right: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident("__q".to_string())),
                                method: "floor".to_string(),
                                args: vec![],
                            }),
                            op: "*".to_string(),
                            right: Box::new(crate::RustExpr::Ident("__r".to_string())),
                        }),
                    },
                    _ => return Ok(None),
                };
                return Ok(Some(crate::RustExpr::Block {
                    stmts: vec![
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__l".to_string(),
                            ty: None,
                            value: lowered_left,
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__r".to_string(),
                            ty: None,
                            value: lowered_right,
                        },
                    ],
                    expr: Some(Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "Decimal".to_string(),
                                "checked_div".to_string(),
                            ])),
                            args: vec![
                                crate::RustExpr::Ident("__l".to_string()),
                                crate::RustExpr::Ident("__r".to_string()),
                            ],
                        }),
                        method: "map_or_else".to_string(),
                        args: vec![
                            crate::RustExpr::Closure {
                                params: vec![],
                                body: Box::new(runtime_exit_block(invalid_message)),
                                is_move: false,
                            },
                            crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__q".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(success_expr),
                                is_move: false,
                            },
                        ],
                    })),
                }));
            }

            if matches!(resolved_result_ty, Type::BigDecimal)
                && matches!(op.as_str(), "+" | "-" | "*")
            {
                return Ok(Some(round_bigdecimal_with_default_context(
                    crate::RustExpr::BinOp {
                        left: Box::new(lowered_left),
                        op: op.clone(),
                        right: Box::new(lowered_right),
                    },
                )));
            }

            if matches!(resolved_result_ty, Type::BigDecimal)
                && matches!(op.as_str(), "/" | "//" | "%")
            {
                let invalid_message = match op.as_str() {
                    "/" => "runtime error: bigdecimal division by zero",
                    "//" => "runtime error: bigdecimal floor-division by zero",
                    _ => "runtime error: bigdecimal modulo by zero",
                };
                let zero_check = crate::RustExpr::BinOp {
                    left: Box::new(crate::RustExpr::Ident("__r".to_string())),
                    op: "==".to_string(),
                    right: Box::new(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "BigDecimal".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                    }),
                };
                let success_expr = match op.as_str() {
                    "/" => round_bigdecimal_with_default_context(crate::RustExpr::BinOp {
                        left: Box::new(crate::RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(crate::RustExpr::Ident("__l".to_string())),
                        }),
                        op: "/".to_string(),
                        right: Box::new(crate::RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(crate::RustExpr::Ident("__r".to_string())),
                        }),
                    }),
                    "//" => round_bigdecimal_with_default_context(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(crate::RustExpr::Ident("__l".to_string())),
                            }),
                            op: "/".to_string(),
                            right: Box::new(crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(crate::RustExpr::Ident("__r".to_string())),
                            }),
                        }),
                        method: "with_scale_round".to_string(),
                        args: vec![
                            crate::RustExpr::Literal(crate::RustLiteral::Int(0)),
                            crate::RustExpr::Path(vec![
                                "bigdecimal".to_string(),
                                "RoundingMode".to_string(),
                                "Floor".to_string(),
                            ]),
                        ],
                    }),
                    "%" => {
                        let floored_q = crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::BinOp {
                                left: Box::new(crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Ident("__l".to_string())),
                                }),
                                op: "/".to_string(),
                                right: Box::new(crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Ident("__r".to_string())),
                                }),
                            }),
                            method: "with_scale_round".to_string(),
                            args: vec![
                                crate::RustExpr::Literal(crate::RustLiteral::Int(0)),
                                crate::RustExpr::Path(vec![
                                    "bigdecimal".to_string(),
                                    "RoundingMode".to_string(),
                                    "Floor".to_string(),
                                ]),
                            ],
                        };
                        round_bigdecimal_with_default_context(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(crate::RustExpr::Ident("__l".to_string())),
                            }),
                            op: "-".to_string(),
                            right: Box::new(crate::RustExpr::BinOp {
                                left: Box::new(floored_q),
                                op: "*".to_string(),
                                right: Box::new(crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Ident("__r".to_string())),
                                }),
                            }),
                        })
                    }
                    _ => return Ok(None),
                };
                return Ok(Some(crate::RustExpr::Block {
                    stmts: vec![
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__l".to_string(),
                            ty: None,
                            value: lowered_left,
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__r".to_string(),
                            ty: None,
                            value: lowered_right,
                        },
                    ],
                    expr: Some(Box::new(crate::RustExpr::If {
                        cond: Box::new(zero_check),
                        then_expr: Box::new(runtime_exit_block(invalid_message)),
                        else_expr: Some(Box::new(success_expr)),
                    })),
                }));
            }

            if op == "**" {
                if matches!(resolved_left_ty, Type::Float)
                    || matches!(resolved_right_ty, Type::Float)
                    || matches!(resolved_result_ty, Type::Float)
                {
                    return Ok(Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_left))),
                        method: "powf".to_string(),
                        args: vec![crate::RustExpr::Cast {
                            expr: Box::new(lowered_right),
                            ty: crate::RustType::F64,
                        }],
                    }));
                }
                if matches!(resolved_result_ty, Type::Decimal) {
                    let exponent_i64 = if matches!(resolved_right_ty, Type::BigInt) {
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "i64".to_string(),
                                    "try_from".to_string(),
                                ])),
                                args: vec![crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Paren(Box::new(
                                        lowered_right.clone(),
                                    ))),
                                }],
                            }),
                            method: "map_or_else".to_string(),
                            args: vec![
                                crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__e".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(runtime_exit_block(
                                        "runtime error: decimal exponent is out of i64 range",
                                    )),
                                    is_move: false,
                                },
                                crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__v".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(crate::RustExpr::Ident("__v".to_string())),
                                    is_move: false,
                                },
                            ],
                        }
                    } else {
                        lowered_right.clone()
                    };
                    return Ok(Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "<Decimal as rust_decimal::MathematicalOps>".to_string(),
                                "checked_powi".to_string(),
                            ])),
                            args: vec![
                                crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_left))),
                                },
                                exponent_i64,
                            ],
                        }),
                        method: "map_or_else".to_string(),
                        args: vec![
                            crate::RustExpr::Closure {
                                params: vec![],
                                body: Box::new(runtime_exit_block(
                                    "runtime error: decimal exponentiation failed",
                                )),
                                is_move: false,
                            },
                            crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__v".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::Ident("__v".to_string())),
                                is_move: false,
                            },
                        ],
                    }));
                }
                if matches!(resolved_result_ty, Type::BigDecimal) {
                    let exponent_i64 = if matches!(resolved_right_ty, Type::BigInt) {
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "i64".to_string(),
                                    "try_from".to_string(),
                                ])),
                                args: vec![crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Paren(Box::new(
                                        lowered_right.clone(),
                                    ))),
                                }],
                            }),
                            method: "map_or_else".to_string(),
                            args: vec![
                                crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__e".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(runtime_exit_block(
                                        "runtime error: bigdecimal exponent is out of i64 range",
                                    )),
                                    is_move: false,
                                },
                                crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__v".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(crate::RustExpr::Ident("__v".to_string())),
                                    is_move: false,
                                },
                            ],
                        }
                    } else {
                        lowered_right.clone()
                    };
                    return Ok(Some(round_bigdecimal_with_default_context(
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_left))),
                            method: "powi_with_context".to_string(),
                            args: vec![
                                exponent_i64,
                                crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(bigdecimal_default_context_expr()),
                                },
                            ],
                        },
                    )));
                }
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_left))),
                    method: "pow".to_string(),
                    args: vec![crate::RustExpr::Cast {
                        expr: Box::new(lowered_right),
                        ty: crate::RustType::Named("u32".to_string()),
                    }],
                }));
            }
            return Ok(Some(crate::RustExpr::BinOp {
                left: Box::new(lowered_left),
                op: if op == "//" {
                    "/".to_string()
                } else {
                    op.clone()
                },
                right: Box::new(lowered_right),
            }));
        }
        crate::try_lower_leaf_or_name_expr_result(expr)
    }

    pub(crate) fn try_lower_stmt_expr_statement_only(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        match expr {
            HirExpr::Call { func, args, .. } => {
                if func == "print" {
                    return self.lower_print_call_expr_for_ir(args);
                }
                if let Some(lowered_intrinsic) =
                    self.try_lower_registry_intrinsic_call_expr(func, args)
                {
                    return Ok(Some(lowered_intrinsic));
                }
                if let Some(lowered_builtin) = self.try_lower_registry_builtin_call_expr(func, args)
                {
                    return Ok(Some(lowered_builtin));
                }
                if let Some(lowered_plain) =
                    self.try_lower_registry_plain_call_with_signature(func, args)
                {
                    return Ok(Some(lowered_plain));
                }
                if func == "iter" && args.len() == 1 {
                    return self.lower_iter_source_expr_for_ir(&args[0]);
                }
                if func == "next" && args.len() == 1 {
                    let Some(lowered_iterator) = self.lower_stmt_expr_for_ir(&args[0])? else {
                        return Ok(None);
                    };
                    return Ok(Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(lowered_iterator),
                        method: "next".to_string(),
                        args: vec![],
                    }));
                }
                let mut lowered_args = Vec::with_capacity(args.len());
                for arg in args {
                    let Some(lowered_arg) = self.lower_stmt_expr_for_ir(arg)? else {
                        return Ok(None);
                    };
                    lowered_args.push(self.rewrite_stdlib_constant_idents_in_expr(lowered_arg));
                }
                lowered_args =
                    self.adapt_plain_call_args_with_signature_for_ir(func, args, lowered_args);
                if let Some(captures) = self.nested_fn_captures.get(func).cloned() {
                    for capture in captures {
                        lowered_args.push(self.lower_recursive_capture_arg_for_ir(&capture));
                    }
                }
                let lowered_func = if func.contains("::") {
                    crate::RustExpr::Path(func.split("::").map(str::to_string).collect())
                } else {
                    crate::RustExpr::Ident(func.clone())
                };
                Ok(Some(crate::RustExpr::FnCall {
                    func: Box::new(lowered_func),
                    args: lowered_args,
                }))
            }
            HirExpr::MethodCall {
                object,
                method,
                args,
                ..
            } => {
                let needs_self_field_clone_suppression =
                    self.method_call_needs_field_clone_suppression(object, method);
                let suppression_prev = self.pending_self_field_clone_suppression;
                if needs_self_field_clone_suppression {
                    self.pending_self_field_clone_suppression += 1;
                }

                let lowered = self.try_lower_registry_method_call_expr(
                    crate::resolve_alias_type_for_plain_call(object.ty()),
                    object,
                    method,
                    args,
                );

                if needs_self_field_clone_suppression
                    && self.pending_self_field_clone_suppression > suppression_prev
                {
                    self.pending_self_field_clone_suppression -= 1;
                }

                Ok(lowered)
            }
            _ => Ok(None),
        }
    }

    fn lower_print_call_expr_for_ir(
        &mut self,
        args: &[HirExpr],
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if args.is_empty() {
            return Ok(Some(crate::RustExpr::MacroCall {
                name: "println".to_string(),
                args: vec![],
            }));
        }

        if args.len() == 1 {
            let arg = &args[0];
            if let HirExpr::StringLiteral(value) = arg {
                let escaped = value
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('{', "{{")
                    .replace('}', "}}");
                return Ok(Some(crate::RustExpr::FormatMacro {
                    name: "println".to_string(),
                    format_str: escaped,
                    args: vec![],
                }));
            }
            if let HirExpr::FString { parts, .. } = arg {
                let mut format_str = String::new();
                let mut lowered_args = Vec::new();
                for part in parts {
                    match part {
                        HirFStringPart::Literal(text) => {
                            format_str.push_str(&text.replace('{', "{{").replace('}', "}}"));
                        }
                        HirFStringPart::Expr(inner) => {
                            let Some(lowered_inner) = self.lower_stmt_expr_for_ir(inner)? else {
                                return Ok(None);
                            };
                            format_str.push_str("{}");
                            if let Some(option_inner_ty) =
                                Self::option_inner_type_for_ir(inner.ty())
                            {
                                let option_format_str =
                                    if Self::uses_debug_display_format_for_ir(option_inner_ty) {
                                        "{:?}".to_string()
                                    } else {
                                        "{}".to_string()
                                    };
                                lowered_args.push(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                        lowered_inner,
                                    ))),
                                    method: "map_or".to_string(),
                                    args: vec![
                                        crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Literal(
                                                crate::RustLiteral::Str("None".to_string()),
                                            )),
                                            method: "to_string".to_string(),
                                            args: vec![],
                                        },
                                        crate::RustExpr::Closure {
                                            params: vec![crate::RustParam::Named {
                                                name: "__v".to_string(),
                                                ty: crate::RustType::Named("_".to_string()),
                                            }],
                                            body: Box::new(crate::RustExpr::FormatMacro {
                                                name: "format".to_string(),
                                                format_str: option_format_str,
                                                args: vec![crate::RustExpr::Ident(
                                                    "__v".to_string(),
                                                )],
                                            }),
                                            is_move: false,
                                        },
                                    ],
                                });
                            } else {
                                lowered_args.push(lowered_inner);
                            }
                        }
                    }
                }
                return Ok(Some(crate::RustExpr::FormatMacro {
                    name: "println".to_string(),
                    format_str,
                    args: lowered_args,
                }));
            }

            let Some(lowered_arg) = self.lower_stmt_expr_for_ir(arg)? else {
                return Ok(None);
            };
            if let Some(inner) = Self::option_inner_type_for_ir(arg.ty()) {
                let option_format_str = if Self::uses_debug_display_format_for_ir(inner) {
                    "{:?}".to_string()
                } else {
                    "{}".to_string()
                };
                return Ok(Some(crate::RustExpr::FormatMacro {
                    name: "println".to_string(),
                    format_str: "{}".to_string(),
                    args: vec![crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
                        method: "map_or".to_string(),
                        args: vec![
                            crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Literal(
                                    crate::RustLiteral::Str("None".to_string()),
                                )),
                                method: "to_string".to_string(),
                                args: vec![],
                            },
                            crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__v".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::FormatMacro {
                                    name: "format".to_string(),
                                    format_str: option_format_str,
                                    args: vec![crate::RustExpr::Ident("__v".to_string())],
                                }),
                                is_move: false,
                            },
                        ],
                    }],
                }));
            }
            let format_str = if Self::uses_debug_display_format_for_ir(arg.ty()) {
                "{:?}"
            } else {
                "{}"
            };
            return Ok(Some(crate::RustExpr::FormatMacro {
                name: "println".to_string(),
                format_str: format_str.to_string(),
                args: vec![lowered_arg],
            }));
        }

        if let HirExpr::StringLiteral(fmt) = &args[0] {
            let mut lowered_args = Vec::with_capacity(args.len().saturating_sub(1));
            for arg in args.iter().skip(1) {
                let Some(lowered_arg) = self.lower_stmt_expr_for_ir(arg)? else {
                    return Ok(None);
                };
                lowered_args.push(lowered_arg);
            }
            return Ok(Some(crate::RustExpr::FormatMacro {
                name: "println".to_string(),
                format_str: fmt.clone(),
                args: lowered_args,
            }));
        }

        let mut format_parts = Vec::with_capacity(args.len());
        let mut lowered_args = Vec::with_capacity(args.len());
        for arg in args {
            let Some(lowered_arg) = self.lower_stmt_expr_for_ir(arg)? else {
                return Ok(None);
            };
            if let Some(inner) = Self::option_inner_type_for_ir(arg.ty()) {
                let option_format_str = if Self::uses_debug_display_format_for_ir(inner) {
                    "{:?}".to_string()
                } else {
                    "{}".to_string()
                };
                lowered_args.push(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
                    method: "map_or".to_string(),
                    args: vec![
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Str(
                                "None".to_string(),
                            ))),
                            method: "to_string".to_string(),
                            args: vec![],
                        },
                        crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__v".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::FormatMacro {
                                name: "format".to_string(),
                                format_str: option_format_str,
                                args: vec![crate::RustExpr::Ident("__v".to_string())],
                            }),
                            is_move: false,
                        },
                    ],
                });
                format_parts.push("{}");
            } else {
                lowered_args.push(lowered_arg);
                format_parts.push(if Self::uses_debug_display_format_for_ir(arg.ty()) {
                    "{:?}"
                } else {
                    "{}"
                });
            }
        }
        Ok(Some(crate::RustExpr::FormatMacro {
            name: "println".to_string(),
            format_str: format_parts.join(" "),
            args: lowered_args,
        }))
    }

    fn object_name_expr_for_ir(object: &str) -> crate::RustExpr {
        if object.contains("::") {
            return crate::RustExpr::Path(object.split("::").map(ToString::to_string).collect());
        }
        crate::RustExpr::Ident(object.to_string())
    }

    fn is_some_call_expr_for_ir(expr: &crate::RustExpr) -> bool {
        matches!(
            expr,
            crate::RustExpr::FnCall { func, .. }
                if matches!(func.as_ref(), crate::RustExpr::Path(path) if path.len() == 1 && path[0] == "Some")
                    || matches!(func.as_ref(), crate::RustExpr::Ident(name) if name == "Some")
        )
    }

    fn is_box_new_call_expr_for_ir(expr: &crate::RustExpr) -> bool {
        matches!(
            expr,
            crate::RustExpr::FnCall { func, .. }
                if matches!(func.as_ref(), crate::RustExpr::Path(path) if path.len() == 2 && path[0] == "Box" && path[1] == "new")
                    || matches!(func.as_ref(), crate::RustExpr::Ident(name) if name == "Box::new")
        )
    }

    fn ensure_some_box_inner_for_ir(expr: crate::RustExpr) -> crate::RustExpr {
        match expr {
            crate::RustExpr::FnCall { func, args }
                if matches!(func.as_ref(), crate::RustExpr::Path(path) if path.len() == 1 && path[0] == "Some")
                    && args.len() == 1 =>
            {
                let mut args_iter = args.into_iter();
                let inner = args_iter
                    .next()
                    .expect("checked args.len() == 1 for Some(_) call");
                if Self::is_box_new_call_expr_for_ir(&inner) {
                    crate::RustExpr::FnCall {
                        func,
                        args: vec![inner],
                    }
                } else {
                    crate::RustExpr::FnCall {
                        func,
                        args: vec![crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "Box".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![inner],
                        }],
                    }
                }
            }
            other => crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                args: vec![crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "Box".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![other],
                }],
            },
        }
    }

    fn adapt_plain_call_args_with_signature_for_ir(
        &self,
        func: &str,
        hir_args: &[HirExpr],
        lowered_args: Vec<crate::RustExpr>,
    ) -> Vec<crate::RustExpr> {
        let Some(param_info) = self.resolve_plain_call_param_info(func, hir_args.len()) else {
            return lowered_args;
        };
        if param_info.len() < hir_args.len() || lowered_args.len() != hir_args.len() {
            return lowered_args;
        }

        let mut adapted = Vec::with_capacity(lowered_args.len());
        let ctor_class_name = func.strip_suffix("::new");
        for (idx, (((param_ty, convention), hir_arg), mut lowered_arg)) in param_info
            .iter()
            .take(hir_args.len())
            .zip(hir_args.iter())
            .zip(lowered_args.into_iter())
            .enumerate()
        {
            let resolved_param = crate::resolve_alias_type_for_plain_call(param_ty);
            let borrowed_name_arg = matches!(hir_arg, HirExpr::Name { name, ty }
                if self.borrowed_params.contains(name)
                    || self.mut_borrowed_params.contains(name)
                    || ty.rust_type().starts_with('&'));

            if crate::helpers::is_option_type(resolved_param) {
                let is_recursive_ctor_param = ctor_class_name
                    .and_then(|class_name| {
                        self.class_field_order
                            .get(class_name)
                            .and_then(|fields| fields.get(idx))
                            .map(|field_name| {
                                self.recursive_fields
                                    .contains(&(class_name.to_owned(), field_name.clone()))
                            })
                    })
                    .unwrap_or(false);
                let needs_box_inner =
                    param_ty.rust_type().starts_with("Option<Box<") || is_recursive_ctor_param;
                if !crate::helpers::is_option_type(hir_arg.ty())
                    && !matches!(hir_arg, HirExpr::NoneLiteral)
                {
                    lowered_arg = if needs_box_inner {
                        Self::ensure_some_box_inner_for_ir(lowered_arg)
                    } else {
                        crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                            args: vec![lowered_arg],
                        }
                    };
                } else if needs_box_inner && Self::is_some_call_expr_for_ir(&lowered_arg) {
                    lowered_arg = Self::ensure_some_box_inner_for_ir(lowered_arg);
                }
            }

            let param_rust_type = param_ty.rust_type();
            if param_rust_type.starts_with("Box<dyn ")
                && !hir_arg.ty().rust_type().starts_with("Box<dyn ")
            {
                lowered_arg = crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "Box".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![lowered_arg],
                };
            }

            if convention.is_owned() && borrowed_name_arg {
                lowered_arg = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
                    method: "clone".to_string(),
                    args: vec![],
                };
            }

            let expects_shared_ref_type =
                param_ty.rust_type().starts_with('&') && !param_ty.rust_type().starts_with("&mut ");
            let expects_mut_ref_type = param_ty.rust_type().starts_with("&mut ");
            let needs_shared_borrow = expects_shared_ref_type
                || (convention.is_shared_borrow()
                    && (param_ty.ownership() != sifr_type_system::OwnershipKind::Copy
                        || matches!(resolved_param, Type::TypeVar(_) | Type::Any)));
            let needs_mut_borrow = expects_mut_ref_type
                || (convention.is_mut_borrow()
                    && (param_ty.ownership() != sifr_type_system::OwnershipKind::Copy
                        || matches!(resolved_param, Type::TypeVar(_) | Type::Any)));
            let already_borrowed = matches!(lowered_arg, crate::RustExpr::Ref { .. })
                || matches!(hir_arg, HirExpr::Name { name, .. }
                    if self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name));
            let already_mut_borrowed = matches!(
                lowered_arg,
                crate::RustExpr::Ref { mutable: true, .. }
            ) || matches!(hir_arg, HirExpr::Name { name, .. } if self.mut_borrowed_params.contains(name));

            if needs_shared_borrow || needs_mut_borrow {
                lowered_arg = Self::clone_moved_names_in_borrowed_aggregate(hir_arg, lowered_arg);
            }

            if needs_shared_borrow && !already_borrowed {
                lowered_arg = crate::RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(lowered_arg),
                };
            } else if needs_mut_borrow && !already_mut_borrowed {
                lowered_arg = crate::RustExpr::Ref {
                    mutable: true,
                    expr: Box::new(lowered_arg),
                };
            }

            adapted.push(lowered_arg);
        }
        adapted
    }

    pub(crate) fn lower_recursive_capture_arg_for_ir(
        &self,
        capture: &crate::NestedFnCapture,
    ) -> crate::RustExpr {
        let ident = crate::RustExpr::Ident(capture.name.clone());
        if capture.convention.is_mut_borrow() {
            if self.mut_borrowed_params.contains(&capture.name) {
                return ident;
            }
            return crate::RustExpr::Ref {
                mutable: true,
                expr: Box::new(ident),
            };
        }
        if capture.convention.is_shared_borrow() {
            if self.borrowed_params.contains(&capture.name)
                || self.mut_borrowed_params.contains(&capture.name)
            {
                return ident;
            }
            return crate::RustExpr::Ref {
                mutable: false,
                expr: Box::new(ident),
            };
        }
        ident
    }

    fn borrowed_return_name_clone_expr_for_ir(&self, value: &HirExpr) -> Option<crate::RustExpr> {
        let HirExpr::Name { name, .. } = value else {
            return None;
        };
        if !(self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name)) {
            return None;
        }
        Some(crate::RustExpr::Clone(Box::new(crate::RustExpr::Ident(
            name.clone(),
        ))))
    }

    fn lower_non_option_index_expr_for_ir(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let object_ty = crate::resolve_alias_type_for_plain_call(object.ty());
        if !matches!(
            object_ty,
            Type::Tuple(_) | Type::List(_) | Type::Bytes | Type::Str
        ) {
            return Ok(None);
        }

        let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
            return Ok(None);
        };
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };

        let lowered = match object_ty {
            Type::Tuple(elements) => {
                let HirExpr::IntLiteral(raw_idx) = index else {
                    return Ok(None);
                };
                let Ok(idx) = usize::try_from(*raw_idx) else {
                    return Ok(None);
                };
                if idx >= elements.len() {
                    return Ok(None);
                }
                crate::RustExpr::Field {
                    expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_object))),
                    field: idx.to_string(),
                }
            }
            Type::List(_) => crate::RustExpr::Clone(Box::new(crate::RustExpr::Index {
                expr: Box::new(lowered_object),
                index: Box::new(crate::RustExpr::Cast {
                    expr: Box::new(lowered_index),
                    ty: crate::RustType::Named("usize".to_string()),
                }),
            })),
            Type::Bytes => crate::RustExpr::Cast {
                expr: Box::new(crate::RustExpr::Clone(Box::new(crate::RustExpr::Index {
                    expr: Box::new(lowered_object),
                    index: Box::new(crate::RustExpr::Cast {
                        expr: Box::new(lowered_index),
                        ty: crate::RustType::Named("usize".to_string()),
                    }),
                }))),
                ty: crate::RustType::I64,
            },
            Type::Str => {
                let nth_expr = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(lowered_object),
                        method: "chars".to_string(),
                        args: vec![],
                    }),
                    method: "nth".to_string(),
                    args: vec![crate::RustExpr::Cast {
                        expr: Box::new(lowered_index),
                        ty: crate::RustType::Named("usize".to_string()),
                    }],
                };
                crate::RustExpr::Block {
                    stmts: vec![crate::RustStmt::LetElse {
                        pattern: "Some(__indexed_char)".to_string(),
                        value: nth_expr,
                        else_body: vec![crate::RustStmt::Expr(crate::RustExpr::MacroCall {
                            name: "unreachable".to_string(),
                            args: vec![crate::RustExpr::Literal(crate::RustLiteral::Str(
                                "compiler-verified string index should be in range".to_string(),
                            ))],
                        })],
                    }],
                    expr: Some(Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident("__indexed_char".to_string())),
                        method: "to_string".to_string(),
                        args: vec![],
                    })),
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(lowered))
    }

    fn lower_return_value_expr_for_ir(
        &mut self,
        value: &HirExpr,
        return_ty: Option<&Type>,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let wrap_option_return = |lowered: crate::RustExpr| -> crate::RustExpr {
            if let Some(target_ty) = return_ty {
                return Self::wrap_option_local_value_for_ir(target_ty, value, lowered);
            }
            lowered
        };
        if self.current_class_name.is_some()
            && matches!(value, HirExpr::Name { name, .. } if name == "self")
        {
            return Ok(Some(wrap_option_return(crate::RustExpr::Clone(Box::new(
                crate::RustExpr::Ident("self".to_string()),
            )))));
        }

        if let Some(clone_expr) = self.borrowed_return_name_clone_expr_for_ir(value) {
            return Ok(Some(wrap_option_return(clone_expr)));
        }

        if return_ty.is_some_and(|ty| !crate::helpers::is_option_type(ty))
            && matches!(value, HirExpr::Index { .. })
        {
            let HirExpr::Index { object, index, .. } = value else {
                unreachable!();
            };
            if let Some(lowered) = self.lower_non_option_index_expr_for_ir(object, index)? {
                return Ok(Some(lowered));
            }
        }

        if let Some(lowered_leaf) = crate::try_lower_leaf_or_name_expr_result(value)? {
            return Ok(Some(wrap_option_return(lowered_leaf)));
        }
        if let Some(lowered_expr) = self.lower_stmt_expr_for_ir(value)? {
            return Ok(Some(wrap_option_return(
                self.rewrite_stdlib_constant_idents_in_expr(lowered_expr),
            )));
        }
        Ok(None)
    }

    pub(super) fn lower_rendered_expr_for_ir(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let HirExpr::Index {
            object, index, ty, ..
        } = expr
        {
            if !crate::helpers::is_option_type(ty) {
                if let Some(lowered) = self.lower_non_option_index_expr_for_ir(object, index)? {
                    return Ok(Some(lowered));
                }
            }
        }
        if let Some(lowered_leaf) = crate::try_lower_leaf_or_name_expr_result(expr)? {
            return Ok(Some(lowered_leaf));
        }
        if let Some(lowered_expr) = self.lower_stmt_expr_for_ir(expr)? {
            return Ok(Some(
                self.rewrite_stdlib_constant_idents_in_expr(lowered_expr),
            ));
        }
        Ok(None)
    }

    fn try_lower_stmt_block_for_ir(
        &mut self,
        stmts: &[HirStmt],
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let scope_ctx = crate::ScopeContext {
            function_return_type: self.current_return_type.clone(),
            in_generator_closure: self.emission_ctx.in_generator_closure,
            in_display_impl: self.emission_ctx.in_display_impl,
            in_loop_with_else: self.current_loop_has_else(),
            class_scope: if self.current_class_name.is_some() {
                crate::ClassScope::Inside
            } else {
                crate::ClassScope::Outside
            },
        };

        let mut lowered_block = Vec::new();
        for stmt in stmts {
            let maybe_simple_lowered = if self.try_closure_depth == 0 {
                crate::try_lower_simple_stmt_with_scope_result(
                    stmt,
                    &self.mutated_vars,
                    &self.borrowed_params,
                    &scope_ctx,
                )?
            } else {
                None
            };

            let should_bypass_simple_lowering =
                matches!(stmt, HirStmt::Let { ty, .. } if self.type_contains_generic_class(ty));
            let maybe_simple_lowered = if should_bypass_simple_lowering {
                None
            } else {
                maybe_simple_lowered
            };
            let (lowered_stmts, skip_rewrite) = if let Some(lowered_stmts) = maybe_simple_lowered {
                (lowered_stmts, false)
            } else if let HirStmt::TupleUnpack { targets, value } = stmt {
                let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
                    return Ok(None);
                };
                (
                    crate::lower_tuple_unpack_targets(targets, lowered_value, &self.mutated_vars),
                    false,
                )
            } else if let HirStmt::Let {
                name, ty, value, ..
            } = stmt
            {
                let is_generic_class = matches!(ty, Type::Class { name: class_name, .. } if self.generic_classes.contains(class_name));
                let lowered_value = if ty.ownership() == sifr_type_system::OwnershipKind::Move {
                    if let HirExpr::Name {
                        name: value_name, ..
                    } = value
                    {
                        if self.borrowed_params.contains(value_name)
                            || self.mut_borrowed_params.contains(value_name)
                        {
                            Some(crate::RustExpr::Clone(Box::new(crate::RustExpr::Ident(
                                value_name.clone(),
                            ))))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
                let lowered_value = if let Some(clone_expr) = lowered_value {
                    clone_expr
                } else {
                    let Some(lowered) = self.lower_rendered_expr_for_ir(value)? else {
                        return Ok(None);
                    };
                    Self::wrap_option_local_value_for_ir(ty, value, lowered)
                };
                (
                    vec![RustStmt::Let {
                        mutable: self.mutated_vars.contains(name)
                            || should_force_mutable_binding(ty),
                        name: name.clone(),
                        ty: if is_generic_class || should_omit_local_type_annotation(ty, value) {
                            None
                        } else {
                            Some(self.rust_ir_type_with_generics(ty))
                        },
                        value: lowered_value,
                    }],
                    true,
                )
            } else if let HirStmt::Assign { name, value } = stmt {
                let Some(lowered_value) = self.lower_rendered_expr_for_ir(value)? else {
                    return Ok(None);
                };
                (
                    vec![RustStmt::Assign {
                        target: crate::RustExpr::Ident(name.clone()),
                        value: lowered_value,
                    }],
                    true,
                )
            } else if let HirStmt::AugAssign { name, op, value } = stmt {
                let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
                    return Ok(None);
                };
                let normalized_op = if op == "//=" {
                    "/".to_string()
                } else {
                    op.strip_suffix('=').unwrap_or(op).to_string()
                };
                (
                    vec![RustStmt::AugAssign {
                        target: crate::RustExpr::Ident(name.clone()),
                        op: normalized_op,
                        value: lowered_value,
                    }],
                    true,
                )
            } else if let HirStmt::SubscriptAssign {
                object,
                index,
                value,
                object_ty,
            } = stmt
            {
                let Some(lowered_stmt) =
                    self.lower_subscript_assign_stmt_for_ir(object, index, value, object_ty)?
                else {
                    return Ok(None);
                };
                (vec![lowered_stmt], true)
            } else if let HirStmt::NestedSubscriptAssign {
                object,
                outer_index,
                inner_index,
                value,
                object_ty,
            } = stmt
            {
                let Type::List(inner) = Self::resolve_alias_type_for_loop_iter(object_ty) else {
                    return Ok(None);
                };
                if !matches!(Self::resolve_alias_type_for_loop_iter(inner), Type::List(_)) {
                    return Ok(None);
                }
                let Some(lowered_stmt) = self
                    .lower_structured_nested_list_subscript_assign_stmt_for_ir(
                        object,
                        outer_index,
                        inner_index,
                        value,
                    )?
                else {
                    return Ok(None);
                };
                (vec![lowered_stmt], true)
            } else if let HirStmt::AttributeSubscriptAssign {
                object,
                field,
                index,
                value,
                field_ty,
            } = stmt
            {
                let Type::Dict(key_ty, _) = field_ty else {
                    return Ok(None);
                };

                let key_needs_clone = matches!(key_ty.as_ref(), Type::Str | Type::TypeVar(_))
                    && matches!(index, HirExpr::Name { name, .. }
                            if self.borrowed_params.contains(name.as_str()) || self.mut_borrowed_params.contains(name.as_str()));

                let Some(mut index_expr) = self.lower_rendered_expr_for_ir(index)? else {
                    return Ok(None);
                };
                if key_needs_clone {
                    index_expr = crate::RustExpr::Clone(Box::new(index_expr));
                }
                let Some(value_expr) = self.lower_rendered_expr_for_ir(value)? else {
                    return Ok(None);
                };

                let receiver = crate::RustExpr::Field {
                    expr: Box::new(Self::object_name_expr_for_ir(object)),
                    field: field.clone(),
                };
                (
                    vec![crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(receiver),
                        method: "insert".to_string(),
                        args: vec![index_expr, value_expr],
                    })],
                    true,
                )
            } else if let HirStmt::FieldAssign {
                object,
                field,
                value,
            } = stmt
            {
                let target = crate::RustExpr::Field {
                    expr: Box::new(Self::object_name_expr_for_ir(object)),
                    field: field.clone(),
                };
                let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                    return Ok(None);
                };
                (
                    vec![RustStmt::Assign {
                        target,
                        value: value_expr,
                    }],
                    true,
                )
            } else if let HirStmt::Assert { test, msg } = stmt {
                let Some(lowered_test) = self.lower_rendered_expr_for_ir(test)? else {
                    return Ok(None);
                };
                let lowered_msg = if let Some(msg_expr) = msg {
                    let Some(lowered) = self.lower_rendered_expr_for_ir(msg_expr)? else {
                        return Ok(None);
                    };
                    Some(lowered)
                } else {
                    None
                };
                (
                    vec![RustStmt::Assert {
                        cond: lowered_test,
                        msg: lowered_msg,
                    }],
                    true,
                )
            } else if let HirStmt::Expr { expr } = stmt {
                let Some(lowered_expr) = self.lower_rendered_expr_for_ir(expr)? else {
                    return Ok(None);
                };
                (vec![RustStmt::Expr(lowered_expr)], true)
            } else if let HirStmt::Return { value } = stmt {
                let return_ty_snapshot = self.current_return_type.clone();
                let lowered_return_stmt = if let Some(value) = value {
                    if self.emission_ctx.in_display_impl && self.try_closure_depth == 0 {
                        let Some(display_expr) = self
                            .lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
                        else {
                            return Ok(None);
                        };
                        RustStmt::Return(Some(crate::RustExpr::MacroCall {
                            name: "write".to_string(),
                            args: vec![
                                crate::RustExpr::Ident("f".to_string()),
                                crate::RustExpr::Literal(crate::RustLiteral::Str("{}".to_string())),
                                display_expr,
                            ],
                        }))
                    } else if self.try_closure_depth > 0 {
                        let wrap_option = self
                            .try_closure_option_wrap
                            .last()
                            .copied()
                            .unwrap_or(false);
                        let Some(mut lowered_return_value) = self
                            .lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
                        else {
                            return Ok(None);
                        };

                        if !wrap_option {
                            if let Some(return_ty) = return_ty_snapshot.as_ref() {
                                if let Type::Result(ok_ty, _) =
                                    crate::resolve_alias_type_for_plain_call(return_ty)
                                {
                                    let value_is_none_like = matches!(value, HirExpr::NoneLiteral)
                                        || matches!(
                                            crate::resolve_alias_type_for_plain_call(value.ty()),
                                            Type::None
                                        );
                                    if value_is_none_like
                                        && matches!(
                                            crate::resolve_alias_type_for_plain_call(
                                                ok_ty.as_ref()
                                            ),
                                            Type::None
                                        )
                                    {
                                        lowered_return_value = crate::RustExpr::FnCall {
                                            func: Box::new(crate::RustExpr::Path(vec![
                                                "Ok".to_string()
                                            ])),
                                            args: vec![crate::RustExpr::Literal(
                                                crate::RustLiteral::Unit,
                                            )],
                                        };
                                    }
                                }
                            }
                        }

                        let try_payload = if wrap_option {
                            crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                                args: vec![lowered_return_value],
                            }
                        } else {
                            lowered_return_value
                        };
                        RustStmt::Return(Some(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                            args: vec![try_payload],
                        }))
                    } else {
                        let Some(lowered_return_value) = self
                            .lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
                        else {
                            return Ok(None);
                        };
                        RustStmt::Return(Some(lowered_return_value))
                    }
                } else if self.try_closure_depth > 0 {
                    let wrap_option = self
                        .try_closure_option_wrap
                        .last()
                        .copied()
                        .unwrap_or(false);
                    if wrap_option {
                        RustStmt::Return(Some(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                            args: vec![crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                                args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                            }],
                        }))
                    } else {
                        let direct_result_none =
                            return_ty_snapshot.as_ref().is_some_and(|ret_ty| {
                                match crate::resolve_alias_type_for_plain_call(ret_ty) {
                                    Type::Result(ok_ty, _) => matches!(
                                        crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
                                        Type::None
                                    ),
                                    _ => false,
                                }
                            });
                        if direct_result_none {
                            RustStmt::Return(Some(crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                                args: vec![crate::RustExpr::FnCall {
                                    func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                                    args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                                }],
                            }))
                        } else {
                            RustStmt::Return(Some(crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                                args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                            }))
                        }
                    }
                } else if self.emission_ctx.in_display_impl {
                    RustStmt::Return(Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                        args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                    }))
                } else {
                    RustStmt::Return(None)
                };
                (vec![lowered_return_stmt], true)
            } else if let HirStmt::Raise { value } = stmt {
                let Some(lowered) = self.lower_stmt_expr_for_ir(value)? else {
                    return Ok(None);
                };
                (
                    vec![RustStmt::Return(Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Err".to_string()])),
                        args: vec![lowered],
                    }))],
                    true,
                )
            } else if let HirStmt::If {
                condition,
                then_body,
                elif_clauses,
                else_body,
            } = stmt
            {
                let Some(lowered_if_stmt) = self.try_lower_if_stmt_for_ir(
                    condition,
                    then_body,
                    elif_clauses,
                    else_body.as_deref(),
                )?
                else {
                    return Ok(None);
                };
                (vec![lowered_if_stmt], true)
            } else if let HirStmt::While {
                condition,
                body,
                else_body,
            } = stmt
            {
                let has_else = else_body.is_some();
                let Some(lowered_cond) = self.lower_condition_expr_for_ir(condition)? else {
                    return Ok(None);
                };
                self.loop_else_stack.push(has_else);
                let Some(lowered_body) = self.try_lower_stmt_block_for_ir(body)? else {
                    let popped = self.loop_else_stack.pop();
                    debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
                    return Ok(None);
                };
                let popped = self.loop_else_stack.pop();
                debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
                if let Some(else_body) = else_body {
                    let Some(lowered_else_body) = self.try_lower_stmt_block_for_ir(else_body)?
                    else {
                        return Ok(None);
                    };
                    (
                        vec![RustStmt::Block(vec![
                            RustStmt::Let {
                                mutable: true,
                                name: "_broke".to_string(),
                                ty: Some(crate::RustType::Bool),
                                value: crate::RustExpr::Literal(crate::RustLiteral::Bool(false)),
                            },
                            RustStmt::While {
                                cond: lowered_cond,
                                body: lowered_body,
                            },
                            RustStmt::If {
                                cond: crate::RustExpr::UnaryOp {
                                    op: "!".to_string(),
                                    operand: Box::new(crate::RustExpr::Paren(Box::new(
                                        crate::RustExpr::Ident("_broke".to_string()),
                                    ))),
                                },
                                then_body: lowered_else_body,
                                else_body: None,
                            },
                        ])],
                        true,
                    )
                } else {
                    (
                        vec![RustStmt::While {
                            cond: lowered_cond,
                            body: lowered_body,
                        }],
                        true,
                    )
                }
            } else if let HirStmt::For {
                target,
                iter,
                body,
                else_body,
                ..
            } = stmt
            {
                if else_body.is_some() {
                    return Ok(None);
                }
                let Some(lowered_iter) = self.try_lower_for_iter_expr_for_ir(iter)? else {
                    return Ok(None);
                };
                let Some(lowered_body) = self.try_lower_stmt_block_for_ir(body)? else {
                    return Ok(None);
                };
                let var = if target.contains(',') {
                    let names = target
                        .split(',')
                        .map(str::trim)
                        .filter(|name| !name.is_empty())
                        .collect::<Vec<_>>();
                    if names.is_empty() {
                        return Ok(None);
                    }
                    format!("({})", names.join(", "))
                } else {
                    target.clone()
                };
                (
                    vec![RustStmt::For {
                        var,
                        iter: lowered_iter,
                        body: lowered_body,
                    }],
                    true,
                )
            } else if let HirStmt::With { items, body } = stmt {
                let Some(lowered_with) = self.try_lower_with_stmt_for_ir(items, body)? else {
                    return Ok(None);
                };
                (vec![lowered_with], true)
            } else if let HirStmt::TryExcept { body, handlers, .. } = stmt {
                let Some(lowered_try_except) =
                    self.try_lower_try_except_stmt_for_ir(body, handlers)?
                else {
                    return Ok(None);
                };
                (lowered_try_except, true)
            } else {
                return Ok(None);
            };
            if skip_rewrite {
                lowered_block.extend(lowered_stmts);
            } else {
                lowered_block.extend(
                    lowered_stmts
                        .into_iter()
                        .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt)),
                );
            }
        }
        Ok(Some(lowered_block))
    }

    fn lower_condition_expr_for_ir(
        &mut self,
        condition: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let Some(option_var) = crate::helpers::detect_option_truthiness(condition) {
            return Ok(Some(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Ident(option_var)),
                method: "is_some".to_string(),
                args: vec![],
            }));
        }
        if let Some(option_var) = crate::helpers::detect_not_option_truthiness(condition) {
            return Ok(Some(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Ident(option_var)),
                method: "is_none".to_string(),
                args: vec![],
            }));
        }
        if let Some(lowered) = Self::try_lower_collection_truthiness_condition_for_ir(condition) {
            return Ok(Some(lowered));
        }
        if let Some(lowered) = Self::try_lower_numeric_truthiness_condition_for_ir(condition) {
            return Ok(Some(lowered));
        }
        if let Some(lowered) = self.try_lower_borrowed_name_compare_condition_for_ir(condition) {
            return Ok(Some(lowered));
        }
        if self.condition_uses_borrowed_name_for_ir(condition) {
            if let Some(lowered) = self.lower_stmt_expr_for_ir(condition)? {
                return Ok(Some(self.rewrite_stdlib_constant_idents_in_expr(lowered)));
            }
        }
        self.lower_rendered_expr_for_ir(condition)
    }

    fn option_binding_value_expr_for_ir(&self, option_var: &str) -> crate::RustExpr {
        let base = crate::RustExpr::Ident(option_var.to_string());
        if self.borrowed_params.contains(option_var)
            || self.mut_borrowed_params.contains(option_var)
        {
            crate::RustExpr::MethodCall {
                receiver: Box::new(base),
                method: "as_ref".to_string(),
                args: vec![],
            }
        } else {
            base
        }
    }

    fn try_lower_collection_truthiness_condition_for_ir(
        condition: &HirExpr,
    ) -> Option<crate::RustExpr> {
        fn is_collection_truthy_type(ty: &Type) -> bool {
            matches!(
                crate::resolve_alias_type_for_plain_call(ty),
                Type::List(_) | Type::Dict(_, _) | Type::Set(_) | Type::Str | Type::Tuple(_)
            )
        }

        if let HirExpr::Name { name, ty } = condition {
            if is_collection_truthy_type(ty) {
                return Some(crate::RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                        method: "is_empty".to_string(),
                        args: vec![],
                    }),
                });
            }
        }

        if let HirExpr::UnaryOp { op, operand, .. } = condition {
            if op == "not" {
                if let HirExpr::Name { name, ty } = operand.as_ref() {
                    if is_collection_truthy_type(ty) {
                        return Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                            method: "is_empty".to_string(),
                            args: vec![],
                        });
                    }
                }
            }
        }

        None
    }

    fn try_lower_numeric_truthiness_condition_for_ir(
        condition: &HirExpr,
    ) -> Option<crate::RustExpr> {
        fn zero_literal_for_type(ty: &Type) -> Option<crate::RustExpr> {
            match crate::resolve_alias_type_for_plain_call(ty) {
                Type::Int | Type::LiteralInt(_) => Some(crate::RustExpr::Cast {
                    expr: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                    ty: crate::RustType::I64,
                }),
                Type::BigInt => Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "BigInt".to_string(),
                        "from".to_string(),
                    ])),
                    args: vec![crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                        ty: crate::RustType::I64,
                    }],
                }),
                Type::Float => Some(crate::RustExpr::Cast {
                    expr: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Float(0.0))),
                    ty: crate::RustType::F64,
                }),
                _ => None,
            }
        }

        match condition {
            HirExpr::Name { name, ty } => Some(crate::RustExpr::BinOp {
                left: Box::new(crate::RustExpr::Ident(name.clone())),
                op: "!=".to_string(),
                right: Box::new(zero_literal_for_type(ty)?),
            }),
            HirExpr::UnaryOp { op, operand, .. } if op == "not" => {
                let HirExpr::Name { name, ty } = operand.as_ref() else {
                    return None;
                };
                Some(crate::RustExpr::BinOp {
                    left: Box::new(crate::RustExpr::Ident(name.clone())),
                    op: "==".to_string(),
                    right: Box::new(zero_literal_for_type(ty)?),
                })
            }
            _ => None,
        }
    }

    fn try_lower_for_iter_expr_for_ir(
        &mut self,
        iter: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let HirExpr::Call { func, args, .. } = iter {
            if func == "iter" && args.len() == 1 {
                return self.lower_iter_source_expr_for_ir(&args[0]);
            }
            if func == "enumerate" && args.len() == 1 {
                let Some(lowered_arg) = self.lower_rendered_expr_for_ir(&args[0])? else {
                    return Ok(None);
                };
                let iter_source = match Self::resolve_alias_type_for_loop_iter(args[0].ty()) {
                    Type::List(_) => crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(lowered_arg),
                            method: "iter".to_string(),
                            args: vec![],
                        }),
                        method: "cloned".to_string(),
                        args: vec![],
                    },
                    Type::Bytes => crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(lowered_arg),
                            method: "iter".to_string(),
                            args: vec![],
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
                                ty: crate::RustType::I64,
                            }),
                            is_move: false,
                        }],
                    },
                    Type::Dict(_, _) => crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(lowered_arg),
                            method: "keys".to_string(),
                            args: vec![],
                        }),
                        method: "cloned".to_string(),
                        args: vec![],
                    },
                    Type::Str => crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(lowered_arg),
                            method: "chars".to_string(),
                            args: vec![],
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
                    },
                    _ => lowered_arg,
                };
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(iter_source),
                        method: "enumerate".to_string(),
                        args: vec![],
                    }),
                    method: "map".to_string(),
                    args: vec![crate::RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "(i, v)".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(crate::RustExpr::Tuple(vec![
                            crate::RustExpr::Cast {
                                expr: Box::new(crate::RustExpr::Ident("i".to_string())),
                                ty: crate::RustType::I64,
                            },
                            crate::RustExpr::Ident("v".to_string()),
                        ])),
                        is_move: false,
                    }],
                }));
            }
        }
        self.lower_iter_source_expr_for_ir(iter)
    }

    fn lower_iter_source_expr_for_ir(
        &mut self,
        source: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let HirExpr::RangeLiteral {
            start, end, step, ..
        } = source
        {
            if let Some(lowered_range_iter) =
                self.try_lower_range_iter_expr_for_ir(start, end, step.as_deref())?
            {
                return Ok(Some(lowered_range_iter));
            }
        }

        let Some(lowered_source) = self.lower_rendered_expr_for_ir(source)? else {
            return Ok(None);
        };
        let lowered_source = Self::normalize_for_loop_iter_expr(lowered_source);
        let source_ty = Self::resolve_alias_type_for_loop_iter(source.ty());

        if matches!(source_ty, Type::Iterator(_))
            || matches!(source, HirExpr::GeneratorExpr { .. })
            || self.is_generator_call(source)
            || Self::is_iterator_like_expr_for_ir(&lowered_source)
        {
            return Ok(Some(lowered_source));
        }

        let iterator_expr = match source_ty {
            Type::List(_) | Type::Set(_) | Type::Iterable(_) => crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_source),
                    method: "iter".to_string(),
                    args: vec![],
                }),
                method: "cloned".to_string(),
                args: vec![],
            },
            Type::Bytes => crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_source),
                    method: "iter".to_string(),
                    args: vec![],
                }),
                method: "map".to_string(),
                args: vec![crate::RustExpr::Closure {
                    params: vec![crate::RustParam::Named {
                        name: "__byte".to_string(),
                        ty: crate::RustType::Named("_".to_string()),
                    }],
                    body: Box::new(crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::Deref(Box::new(crate::RustExpr::Ident(
                            "__byte".to_string(),
                        )))),
                        ty: crate::RustType::I64,
                    }),
                    is_move: false,
                }],
            },
            Type::Dict(_, _) => crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_source),
                    method: "keys".to_string(),
                    args: vec![],
                }),
                method: "cloned".to_string(),
                args: vec![],
            },
            Type::Str => crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_source),
                    method: "chars".to_string(),
                    args: vec![],
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
            },
            Type::Range => crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_source))),
                    method: "clone".to_string(),
                    args: vec![],
                }),
                method: "into_iter".to_string(),
                args: vec![],
            },
            _ => return Ok(Some(lowered_source)),
        };
        Ok(Some(crate::RustExpr::FnCall {
            func: Box::new(crate::RustExpr::Path(vec![
                "Box".to_string(),
                "new".to_string(),
            ])),
            args: vec![iterator_expr],
        }))
    }

    fn is_collect_call_expr(expr: &crate::RustExpr) -> bool {
        match expr {
            crate::RustExpr::MethodCall { method, .. } => {
                method == "collect" || method.starts_with("collect::<")
            }
            crate::RustExpr::Paren(inner) => Self::is_collect_call_expr(inner),
            _ => false,
        }
    }

    fn normalize_for_loop_iter_expr(expr: crate::RustExpr) -> crate::RustExpr {
        if let crate::RustExpr::MethodCall {
            receiver,
            method,
            args,
        } = expr
        {
            if method == "cloned" && args.is_empty() && Self::is_collect_call_expr(&receiver) {
                return *receiver;
            }
            return crate::RustExpr::MethodCall {
                receiver,
                method,
                args,
            };
        }
        expr
    }

    fn is_iterator_like_expr_for_ir(expr: &crate::RustExpr) -> bool {
        match expr {
            crate::RustExpr::MethodCall {
                receiver, method, ..
            } => {
                matches!(
                    method.as_str(),
                    "into_iter" | "map" | "filter" | "zip" | "chain" | "enumerate"
                ) || Self::is_iterator_like_expr_for_ir(receiver)
            }
            crate::RustExpr::FnCall { func, args } => {
                Self::is_iterator_like_expr_for_ir(func)
                    || args.iter().any(Self::is_iterator_like_expr_for_ir)
            }
            crate::RustExpr::Paren(inner)
            | crate::RustExpr::Try(inner)
            | crate::RustExpr::Await(inner)
            | crate::RustExpr::Deref(inner)
            | crate::RustExpr::Clone(inner) => Self::is_iterator_like_expr_for_ir(inner),
            _ => false,
        }
    }

    fn try_lower_with_stmt_for_ir(
        &mut self,
        items: &[(String, HirExpr, bool)],
        body: &[HirStmt],
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let mut lowered_items = Vec::with_capacity(items.len());
        for (var, value, has_cm) in items {
            let Some(lowered_value) = self.lower_rendered_expr_for_ir(value)? else {
                return Ok(None);
            };
            let binding = if queries::stmts_reference_var(body, var)
                || items
                    .iter()
                    .any(|(other_var, _, _)| other_var != var && other_var.contains(var))
            {
                var.clone()
            } else {
                format!("_{var}")
            };
            let class_name = if *has_cm {
                let Type::Class { name, .. } = value.ty() else {
                    return Ok(None);
                };
                Some(name.clone())
            } else {
                None
            };
            lowered_items.push(crate::RustWithItem {
                binding,
                value: lowered_value,
                has_cm: *has_cm,
                class_name,
            });
        }
        let Some(lowered_body) = self.try_lower_stmt_block_for_ir(body)? else {
            return Ok(None);
        };
        Ok(Some(RustStmt::With {
            items: lowered_items,
            body: lowered_body,
        }))
    }

    fn try_lower_borrowed_name_compare_condition_for_ir(
        &self,
        expr: &HirExpr,
    ) -> Option<crate::RustExpr> {
        let HirExpr::Compare {
            left,
            ops,
            comparators,
            ..
        } = expr
        else {
            return None;
        };
        if ops.len() != 1 || comparators.len() != 1 {
            return None;
        }
        let rhs = comparators.first()?;
        let lowered_op = match ops[0].as_str() {
            "==" | "!=" | "<" | "<=" | ">" | ">=" => ops[0].as_str(),
            "is" => "==",
            "is not" => "!=",
            _ => return None,
        };

        let lower_operand =
            |operand: &HirExpr, emitter: &Self| -> Option<(crate::RustExpr, bool)> {
                let HirExpr::Name { name, .. } = operand else {
                    return None;
                };
                let borrowed = emitter.borrowed_params.contains(name)
                    || emitter.mut_borrowed_params.contains(name);
                let ident = crate::RustExpr::Ident(name.clone());
                let lowered = if borrowed {
                    crate::RustExpr::Deref(Box::new(ident))
                } else {
                    ident
                };
                Some((lowered, borrowed))
            };

        let (lowered_left, left_borrowed) = lower_operand(left, self)?;
        let (lowered_right, right_borrowed) = lower_operand(rhs, self)?;
        if !left_borrowed && !right_borrowed {
            return None;
        }

        Some(crate::RustExpr::BinOp {
            left: Box::new(lowered_left),
            op: lowered_op.to_string(),
            right: Box::new(lowered_right),
        })
    }

    fn condition_uses_borrowed_name_for_ir(&self, expr: &HirExpr) -> bool {
        match expr {
            HirExpr::Name { name, .. } => {
                self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name)
            }
            HirExpr::Compare {
                left, comparators, ..
            } => {
                self.condition_uses_borrowed_name_for_ir(left)
                    || comparators
                        .iter()
                        .any(|expr| self.condition_uses_borrowed_name_for_ir(expr))
            }
            HirExpr::BoolOp { values, .. } => values
                .iter()
                .any(|expr| self.condition_uses_borrowed_name_for_ir(expr)),
            HirExpr::BinOp { left, right, .. } => {
                self.condition_uses_borrowed_name_for_ir(left)
                    || self.condition_uses_borrowed_name_for_ir(right)
            }
            HirExpr::UnaryOp { operand, .. } => self.condition_uses_borrowed_name_for_ir(operand),
            HirExpr::Index { object, index, .. } => {
                self.condition_uses_borrowed_name_for_ir(object)
                    || self.condition_uses_borrowed_name_for_ir(index)
            }
            HirExpr::FieldAccess { object, .. } => self.condition_uses_borrowed_name_for_ir(object),
            HirExpr::MethodCall { object, args, .. } => {
                self.condition_uses_borrowed_name_for_ir(object)
                    || args
                        .iter()
                        .any(|expr| self.condition_uses_borrowed_name_for_ir(expr))
            }
            HirExpr::Call { args, .. } => args
                .iter()
                .any(|expr| self.condition_uses_borrowed_name_for_ir(expr)),
            HirExpr::TupleLiteral { elements, .. } | HirExpr::ListLiteral { elements, .. } => {
                elements
                    .iter()
                    .any(|expr| self.condition_uses_borrowed_name_for_ir(expr))
            }
            HirExpr::DictLiteral { keys, values, .. } => {
                keys.iter()
                    .any(|expr| self.condition_uses_borrowed_name_for_ir(expr))
                    || values
                        .iter()
                        .any(|expr| self.condition_uses_borrowed_name_for_ir(expr))
            }
            HirExpr::SetLiteral { elements, .. } => elements
                .iter()
                .any(|expr| self.condition_uses_borrowed_name_for_ir(expr)),
            HirExpr::IfExpr {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                self.condition_uses_borrowed_name_for_ir(condition)
                    || self.condition_uses_borrowed_name_for_ir(then_expr)
                    || self.condition_uses_borrowed_name_for_ir(else_expr)
            }
            HirExpr::WalrusExpr { value, .. } => self.condition_uses_borrowed_name_for_ir(value),
            HirExpr::GeneratorExpr {
                expr, iter, filter, ..
            } => {
                self.condition_uses_borrowed_name_for_ir(expr)
                    || self.condition_uses_borrowed_name_for_ir(iter)
                    || filter
                        .as_ref()
                        .is_some_and(|cond| self.condition_uses_borrowed_name_for_ir(cond))
            }
            HirExpr::ListComp {
                expr, generators, ..
            } => {
                self.condition_uses_borrowed_name_for_ir(expr)
                    || generators.iter().any(|(_, iter, cond)| {
                        self.condition_uses_borrowed_name_for_ir(iter)
                            || cond
                                .as_ref()
                                .is_some_and(|cond| self.condition_uses_borrowed_name_for_ir(cond))
                    })
            }
            HirExpr::DictComp {
                key_expr,
                val_expr,
                generators,
                ..
            } => {
                self.condition_uses_borrowed_name_for_ir(key_expr)
                    || self.condition_uses_borrowed_name_for_ir(val_expr)
                    || generators.iter().any(|(_, iter, cond)| {
                        self.condition_uses_borrowed_name_for_ir(iter)
                            || cond
                                .as_ref()
                                .is_some_and(|cond| self.condition_uses_borrowed_name_for_ir(cond))
                    })
            }
            HirExpr::SetComp {
                expr, generators, ..
            } => {
                self.condition_uses_borrowed_name_for_ir(expr)
                    || generators.iter().any(|(_, iter, cond)| {
                        self.condition_uses_borrowed_name_for_ir(iter)
                            || cond
                                .as_ref()
                                .is_some_and(|cond| self.condition_uses_borrowed_name_for_ir(cond))
                    })
            }
            HirExpr::RangeLiteral {
                start, end, step, ..
            } => {
                self.condition_uses_borrowed_name_for_ir(start)
                    || self.condition_uses_borrowed_name_for_ir(end)
                    || step
                        .as_ref()
                        .is_some_and(|step| self.condition_uses_borrowed_name_for_ir(step))
            }
            HirExpr::ContainsOp {
                element,
                collection,
                ..
            } => {
                self.condition_uses_borrowed_name_for_ir(element)
                    || self.condition_uses_borrowed_name_for_ir(collection)
            }
            HirExpr::Slice {
                object,
                start,
                stop,
                step,
                ..
            } => {
                self.condition_uses_borrowed_name_for_ir(object)
                    || start
                        .as_ref()
                        .is_some_and(|start| self.condition_uses_borrowed_name_for_ir(start))
                    || stop
                        .as_ref()
                        .is_some_and(|stop| self.condition_uses_borrowed_name_for_ir(stop))
                    || step
                        .as_ref()
                        .is_some_and(|step| self.condition_uses_borrowed_name_for_ir(step))
            }
            HirExpr::Lambda { body, .. } => self.condition_uses_borrowed_name_for_ir(body),
            HirExpr::QuestionMark { expr, .. } => self.condition_uses_borrowed_name_for_ir(expr),
            HirExpr::OkWrap { value, .. } | HirExpr::ErrWrap { value, .. } => {
                self.condition_uses_borrowed_name_for_ir(value)
            }
            HirExpr::SuperCall { args, .. } => args
                .iter()
                .any(|expr| self.condition_uses_borrowed_name_for_ir(expr)),
            _ => false,
        }
    }

    fn try_lower_if_stmt_for_ir(
        &mut self,
        condition: &HirExpr,
        then_body: &[HirStmt],
        elif_clauses: &[(HirExpr, Vec<HirStmt>)],
        else_body: Option<&[HirStmt]>,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        if elif_clauses.is_empty()
            && else_body.is_none()
            && queries::block_control_flow_effect(then_body).always_exits()
        {
            let Some(lowered_then_body) = self.try_lower_stmt_block_for_ir(then_body)? else {
                return Ok(None);
            };
            if let Some(option_var) = crate::helpers::detect_is_none_var(condition)
                .or_else(|| crate::helpers::detect_not_option_truthiness(condition))
            {
                return Ok(Some(RustStmt::LetElse {
                    pattern: format!("Some({option_var})"),
                    value: self.option_binding_value_expr_for_ir(&option_var),
                    else_body: lowered_then_body,
                }));
            }
            if let Some(option_vars) =
                crate::helpers::detect_or_not_option_truthiness_vars(condition)
            {
                let pattern = format!(
                    "({})",
                    option_vars
                        .iter()
                        .map(|option_var| format!("Some({option_var})"))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                let value = crate::RustExpr::Tuple(
                    option_vars
                        .iter()
                        .map(|option_var| self.option_binding_value_expr_for_ir(option_var))
                        .collect(),
                );
                return Ok(Some(RustStmt::LetElse {
                    pattern,
                    value,
                    else_body: lowered_then_body,
                }));
            }
        }

        let mut nested_else = if let Some(else_body) = else_body {
            let Some(lowered_else) = self.try_lower_stmt_block_for_ir(else_body)? else {
                return Ok(None);
            };
            Some(lowered_else)
        } else {
            None
        };

        for (elif_cond, elif_body) in elif_clauses.iter().rev() {
            let Some(lowered_elif) =
                self.try_lower_if_clause_for_ir(elif_cond, elif_body, nested_else)?
            else {
                return Ok(None);
            };
            nested_else = Some(vec![lowered_elif]);
        }

        self.try_lower_if_clause_for_ir(condition, then_body, nested_else)
    }

    fn try_lower_if_clause_for_ir(
        &mut self,
        condition: &HirExpr,
        then_body: &[HirStmt],
        nested_else: Option<Vec<RustStmt>>,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let Some(lowered_then_body) = self.try_lower_stmt_block_for_ir(then_body)? else {
            return Ok(None);
        };

        if let Some(option_var) = crate::helpers::detect_is_not_none_var(condition) {
            return Ok(Some(RustStmt::IfLet {
                pattern: format!("Some({option_var})"),
                expr: self.option_binding_value_expr_for_ir(&option_var),
                then_body: lowered_then_body,
                else_body: nested_else,
            }));
        }

        if let Some(option_vars) = crate::helpers::detect_and_not_none_vars(condition) {
            let mut chain_then = lowered_then_body;
            for option_var in option_vars.iter().rev() {
                chain_then = vec![RustStmt::IfLet {
                    pattern: format!("Some({option_var})"),
                    expr: self.option_binding_value_expr_for_ir(option_var),
                    then_body: chain_then,
                    else_body: None,
                }];
            }
            let Some(mut chain_root) = chain_then.into_iter().next() else {
                return Ok(None);
            };
            if let RustStmt::IfLet { else_body, .. } = &mut chain_root {
                *else_body = nested_else;
            }
            return Ok(Some(chain_root));
        }

        if let Some(option_var) = crate::helpers::detect_option_truthiness(condition) {
            return Ok(Some(RustStmt::IfLet {
                pattern: format!("Some({option_var})"),
                expr: self.option_binding_value_expr_for_ir(&option_var),
                then_body: lowered_then_body,
                else_body: nested_else,
            }));
        }

        if let Some(option_var) = crate::helpers::detect_is_none_var(condition) {
            let Some(lowered_cond) = self.lower_condition_expr_for_ir(condition)? else {
                return Ok(None);
            };
            let lowered_else = nested_else.map(|else_body| {
                vec![RustStmt::IfLet {
                    pattern: format!("Some({option_var})"),
                    expr: self.option_binding_value_expr_for_ir(&option_var),
                    then_body: else_body,
                    else_body: None,
                }]
            });
            return Ok(Some(RustStmt::If {
                cond: lowered_cond,
                then_body: lowered_then_body,
                else_body: lowered_else,
            }));
        }

        let Some(lowered_cond) = self.lower_condition_expr_for_ir(condition)? else {
            return Ok(None);
        };
        Ok(Some(RustStmt::If {
            cond: lowered_cond,
            then_body: lowered_then_body,
            else_body: nested_else,
        }))
    }

    /// Emit a generator initialization statement (always mutable for closure capture)
    pub(super) fn emit_generator_init_stmt(&mut self, stmt: &HirStmt) {
        if let HirStmt::Let {
            name, ty, value, ..
        } = stmt
        {
            let Ok(Some(lowered_value)) = self.lower_stmt_expr_for_ir(value) else {
                panic!(
                    "structured generator-init expression emission missing for production path: {value:?}"
                );
            };
            self.push_captured_stmt(&crate::RustStmt::Let {
                mutable: true,
                name: name.clone(),
                ty: Some(self.rust_ir_type_with_generics(ty)),
                value: lowered_value,
            });
            return;
        }

        let Ok(true) = self.try_lower_structured_stmt(stmt) else {
            panic!(
                "structured generator-init statement emission missing for production path: {stmt:?}"
            );
        };
    }

    pub(super) fn emit_lowered_stmts(&mut self, lowered_stmts: &[RustStmt]) {
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

    pub(super) fn current_loop_has_else(&self) -> bool {
        self.loop_else_stack.last().copied().unwrap_or(false)
    }

    pub(crate) fn try_lower_structured_return_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::Return { value } = stmt else {
            return Ok(false);
        };
        let return_ty_snapshot = self.current_return_type.clone();

        if let Some(value) = value {
            if self.emission_ctx.in_display_impl && self.try_closure_depth == 0 {
                let Some(display_expr) =
                    self.lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
                else {
                    return Ok(false);
                };
                self.push_captured_stmt(&RustStmt::Return(Some(crate::RustExpr::MacroCall {
                    name: "write".to_string(),
                    args: vec![
                        crate::RustExpr::Ident("f".to_string()),
                        crate::RustExpr::Literal(crate::RustLiteral::Str("{}".to_string())),
                        display_expr,
                    ],
                })));
                return Ok(true);
            }
            if self.try_closure_depth > 0 {
                let wrap_option = self
                    .try_closure_option_wrap
                    .last()
                    .copied()
                    .unwrap_or(false);

                let Some(mut lowered_return_value) =
                    self.lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
                else {
                    return Ok(false);
                };

                if !wrap_option {
                    if let Some(return_ty) = return_ty_snapshot.as_ref() {
                        if let Type::Result(ok_ty, _) =
                            crate::resolve_alias_type_for_plain_call(return_ty)
                        {
                            let value_is_none_like = matches!(value, HirExpr::NoneLiteral)
                                || matches!(
                                    crate::resolve_alias_type_for_plain_call(value.ty()),
                                    Type::None
                                );
                            if value_is_none_like
                                && matches!(
                                    crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
                                    Type::None
                                )
                            {
                                lowered_return_value = crate::RustExpr::FnCall {
                                    func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                                    args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                                };
                            }
                        }
                    }
                }

                let try_payload = if wrap_option {
                    crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                        args: vec![lowered_return_value],
                    }
                } else {
                    lowered_return_value
                };
                self.push_captured_stmt(&RustStmt::Return(Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                    args: vec![try_payload],
                })));
                return Ok(true);
            }

            let Some(lowered_return_value) =
                self.lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
            else {
                return Ok(false);
            };
            self.push_captured_stmt(&RustStmt::Return(Some(lowered_return_value)));
            return Ok(true);
        }

        if self.try_closure_depth > 0 {
            let wrap_option = self
                .try_closure_option_wrap
                .last()
                .copied()
                .unwrap_or(false);
            if wrap_option {
                self.push_captured_stmt(&RustStmt::Return(Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                    args: vec![crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                        args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                    }],
                })));
            } else {
                let direct_result_none = return_ty_snapshot.as_ref().is_some_and(|ret_ty| {
                    match crate::resolve_alias_type_for_plain_call(ret_ty) {
                        Type::Result(ok_ty, _) => matches!(
                            crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
                            Type::None
                        ),
                        _ => false,
                    }
                });
                if direct_result_none {
                    self.push_captured_stmt(&RustStmt::Return(Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                        args: vec![crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                            args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                        }],
                    })));
                } else {
                    self.push_captured_stmt(&RustStmt::Return(Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                        args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                    })));
                }
            }
        } else if self.emission_ctx.in_display_impl {
            self.push_captured_stmt(&RustStmt::Return(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
            })));
        } else {
            self.push_captured_stmt(&RustStmt::Return(None));
        }
        Ok(true)
    }

    pub(crate) fn try_lower_structured_raise_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::Raise { value } = stmt else {
            return Ok(false);
        };
        let Some(lowered) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(false);
        };
        self.push_captured_stmt(&RustStmt::Return(Some(crate::RustExpr::FnCall {
            func: Box::new(crate::RustExpr::Path(vec!["Err".to_string()])),
            args: vec![lowered],
        })));
        Ok(true)
    }

    pub(crate) fn try_lower_structured_if_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } = stmt
        else {
            return Ok(false);
        };

        if elif_clauses.is_empty() && else_body.is_none() {
            if let HirExpr::Compare {
                left,
                ops,
                comparators,
                ..
            } = condition
            {
                if let HirExpr::WalrusExpr { name, value, ty } = left.as_ref() {
                    let Some(lowered_value) = self.lower_rendered_expr_for_ir(value)? else {
                        return Ok(false);
                    };
                    let walrus_compare_expr = HirExpr::Compare {
                        left: Box::new(HirExpr::Name {
                            name: name.clone(),
                            ty: ty.clone(),
                        }),
                        ops: ops.clone(),
                        comparators: comparators.clone(),
                        ty: condition.ty().clone(),
                    };
                    let Some(lowered_cond) =
                        self.lower_rendered_expr_for_ir(&walrus_compare_expr)?
                    else {
                        return Ok(false);
                    };
                    let Some(lowered_then_body) = self.try_lower_stmt_block_for_ir(then_body)?
                    else {
                        return Ok(false);
                    };

                    self.push_captured_stmt(&RustStmt::Let {
                        mutable: false,
                        name: name.clone(),
                        ty: None,
                        value: lowered_value,
                    });
                    self.push_captured_stmt(&RustStmt::If {
                        cond: lowered_cond,
                        then_body: lowered_then_body,
                        else_body: None,
                    });
                    return Ok(true);
                }
            }
        }

        if let Some((var_name, first_variant, first_enum_name, _)) =
            crate::helpers::detect_isinstance_union(condition)
        {
            let mut branch_specs: Vec<(String, &[HirStmt])> = vec![(first_variant, then_body)];
            let mut needed_variants = vec![branch_specs[0].0.clone()];
            let mut all_isinstance = true;
            for (elif_cond, elif_body) in elif_clauses {
                let Some((elif_var, elif_variant, _, _)) =
                    crate::helpers::detect_isinstance_union(elif_cond)
                else {
                    all_isinstance = false;
                    break;
                };
                if elif_var != var_name {
                    all_isinstance = false;
                    break;
                }
                needed_variants.push(elif_variant.clone());
                branch_specs.push((elif_variant, elif_body.as_slice()));
            }
            if all_isinstance {
                let enum_name = self.resolve_union_enum_name(&first_enum_name, &needed_variants);
                let mut nested_else = if let Some(else_body) = else_body {
                    let remaining_variants = self
                        .union_enums
                        .get(&enum_name)
                        .map(|members| {
                            members
                                .iter()
                                .map(Type::union_variant_name)
                                .filter(|variant| !needed_variants.contains(variant))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    let Some(lowered_else_body) = self.try_lower_stmt_block_for_ir(else_body)?
                    else {
                        return Ok(false);
                    };
                    if remaining_variants.len() == 1 {
                        let else_mutated = queries::collect_mutated_vars(else_body, None);
                        let else_binding = if else_mutated.contains(&var_name) {
                            format!("mut {var_name}")
                        } else {
                            var_name.clone()
                        };
                        Some(vec![RustStmt::IfLet {
                            pattern: format!(
                                "{enum_name}::{}({else_binding})",
                                remaining_variants[0]
                            ),
                            expr: RustExpr::Ident(var_name.clone()),
                            then_body: lowered_else_body,
                            else_body: Some(vec![RustStmt::Expr(RustExpr::FormatMacro {
                                name: "unreachable".to_string(),
                                format_str:
                                    "sifr union narrowing fell through exhaustive branch chain"
                                        .to_string(),
                                args: vec![],
                            })]),
                        }])
                    } else {
                        Some(lowered_else_body)
                    }
                } else {
                    None
                };

                for (variant_name, body) in branch_specs.iter().rev() {
                    let mutated = queries::collect_mutated_vars(body, None);
                    let binding = if mutated.contains(&var_name) {
                        format!("mut {var_name}")
                    } else {
                        var_name.clone()
                    };
                    let Some(lowered_body) = self.try_lower_stmt_block_for_ir(body)? else {
                        return Ok(false);
                    };
                    nested_else = Some(vec![RustStmt::IfLet {
                        pattern: format!("{enum_name}::{variant_name}({binding})"),
                        expr: RustExpr::Ident(var_name.clone()),
                        then_body: lowered_body,
                        else_body: nested_else,
                    }]);
                }

                let Some(root) = nested_else.and_then(|stmts| stmts.into_iter().next()) else {
                    return Ok(false);
                };
                self.push_captured_stmt(&root);
                return Ok(true);
            }
        }

        if elif_clauses.is_empty() {
            if let Some((var_name, variant_name, enum_name, other_variants)) =
                crate::helpers::detect_isinstance_union(condition)
            {
                let mut needed_variants = vec![variant_name.clone()];
                needed_variants.extend(other_variants.iter().map(|(variant, _)| variant.clone()));
                let enum_name = self.resolve_union_enum_name(&enum_name, &needed_variants);

                let then_mutated = queries::collect_mutated_vars(then_body, None);
                let then_binding = if then_mutated.contains(&var_name) {
                    format!("mut {var_name}")
                } else {
                    var_name.clone()
                };
                let Some(lowered_then_body) = self.try_lower_stmt_block_for_ir(then_body)? else {
                    return Ok(false);
                };

                let mut arms = vec![crate::RustMatchArm {
                    pattern: format!("{enum_name}::{variant_name}({then_binding})"),
                    bindings: vec![],
                    guard: None,
                    body: lowered_then_body,
                }];

                if let Some(else_body) = else_body {
                    let else_mutated = queries::collect_mutated_vars(else_body, None);
                    let else_binding = if else_mutated.contains(&var_name) {
                        format!("mut {var_name}")
                    } else {
                        var_name.clone()
                    };
                    let Some(lowered_else_body) = self.try_lower_stmt_block_for_ir(else_body)?
                    else {
                        return Ok(false);
                    };
                    if other_variants.len() == 1 {
                        let (other_variant, _) = &other_variants[0];
                        arms.push(crate::RustMatchArm {
                            pattern: format!("{enum_name}::{other_variant}({else_binding})"),
                            bindings: vec![],
                            guard: None,
                            body: lowered_else_body,
                        });
                    } else {
                        arms.push(crate::RustMatchArm {
                            pattern: "_".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: lowered_else_body,
                        });
                    }
                } else {
                    arms.push(crate::RustMatchArm {
                        pattern: "_".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![],
                    });
                }

                self.push_captured_stmt(&RustStmt::Match {
                    expr: RustExpr::Ident(var_name),
                    arms,
                });
                return Ok(true);
            }
        }

        let Some(lowered_if_stmt) = self.try_lower_if_stmt_for_ir(
            condition,
            then_body,
            elif_clauses,
            else_body.as_deref(),
        )?
        else {
            return Ok(false);
        };
        self.push_captured_stmt(&lowered_if_stmt);
        Ok(true)
    }

    fn resolve_union_enum_name(&self, preferred: &str, needed_variants: &[String]) -> String {
        if self.union_enums.contains_key(preferred) {
            return preferred.to_string();
        }
        for (candidate, members) in &self.union_enums {
            if needed_variants.iter().all(|needed| {
                members
                    .iter()
                    .any(|member| member.union_variant_name() == *needed)
            }) {
                return candidate.clone();
            }
        }
        preferred.to_string()
    }

    pub(crate) fn try_lower_structured_while_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::While {
            condition,
            body,
            else_body,
        } = stmt
        else {
            return Ok(false);
        };
        let has_else = else_body.is_some();
        let Some(lowered_cond) = self.lower_condition_expr_for_ir(condition)? else {
            return Ok(false);
        };
        self.loop_else_stack.push(has_else);
        let lowered_body = self.try_lower_stmt_block_for_ir(body)?;
        let popped = self.loop_else_stack.pop();
        debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
        let Some(lowered_body) = lowered_body else {
            return Ok(false);
        };

        if let Some(else_body) = else_body {
            let Some(lowered_else_body) = self.try_lower_stmt_block_for_ir(else_body)? else {
                return Ok(false);
            };
            self.push_captured_stmt(&RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: true,
                    name: "_broke".to_string(),
                    ty: Some(crate::RustType::Bool),
                    value: crate::RustExpr::Literal(crate::RustLiteral::Bool(false)),
                },
                RustStmt::While {
                    cond: lowered_cond,
                    body: lowered_body,
                },
                RustStmt::If {
                    cond: crate::RustExpr::UnaryOp {
                        op: "!".to_string(),
                        operand: Box::new(crate::RustExpr::Paren(Box::new(
                            crate::RustExpr::Ident("_broke".to_string()),
                        ))),
                    },
                    then_body: lowered_else_body,
                    else_body: None,
                },
            ]));
            return Ok(true);
        }

        self.push_captured_stmt(&RustStmt::While {
            cond: lowered_cond,
            body: lowered_body,
        });
        Ok(true)
    }

    pub(crate) fn try_lower_structured_for_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::For {
            target,
            iter,
            body,
            else_body,
            ..
        } = stmt
        else {
            return Ok(false);
        };
        let has_else = else_body.is_some();
        let var = if target.contains(',') {
            let names = target
                .split(',')
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .collect::<Vec<_>>();
            if names.is_empty() {
                return Ok(false);
            }
            format!("({})", names.join(", "))
        } else {
            target.clone()
        };

        self.loop_else_stack.push(has_else);
        let lowered_iter = self.try_lower_for_iter_expr_for_ir(iter)?;
        let lowered_body = self.try_lower_stmt_block_for_ir(body)?;
        let popped = self.loop_else_stack.pop();
        debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
        let Some(lowered_iter) = lowered_iter else {
            return Ok(false);
        };
        let Some(lowered_body) = lowered_body else {
            return Ok(false);
        };

        if let Some(else_body) = else_body {
            let Some(lowered_else_body) = self.try_lower_stmt_block_for_ir(else_body)? else {
                return Ok(false);
            };
            self.push_captured_stmt(&RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: true,
                    name: "_broke".to_string(),
                    ty: Some(crate::RustType::Bool),
                    value: crate::RustExpr::Literal(crate::RustLiteral::Bool(false)),
                },
                RustStmt::For {
                    var,
                    iter: lowered_iter,
                    body: lowered_body,
                },
                RustStmt::If {
                    cond: crate::RustExpr::UnaryOp {
                        op: "!".to_string(),
                        operand: Box::new(crate::RustExpr::Paren(Box::new(
                            crate::RustExpr::Ident("_broke".to_string()),
                        ))),
                    },
                    then_body: lowered_else_body,
                    else_body: None,
                },
            ]));
            return Ok(true);
        }

        self.push_captured_stmt(&RustStmt::For {
            var,
            iter: lowered_iter,
            body: lowered_body,
        });
        Ok(true)
    }

    pub(crate) fn try_lower_structured_with_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::With { items, body } = stmt else {
            return Ok(false);
        };
        let Some(lowered_with) = self.try_lower_with_stmt_for_ir(items, body)? else {
            return Ok(false);
        };
        self.push_captured_stmt(&lowered_with);
        Ok(true)
    }

    pub(crate) fn try_lower_structured_try_except_stmt(&mut self, stmt: &HirStmt) -> bool {
        let HirStmt::TryExcept { body, handlers, .. } = stmt else {
            return false;
        };
        let lowered = match self.try_lower_try_except_stmt_for_ir(body, handlers) {
            Ok(Some(lowered)) => lowered,
            Ok(None) => return false,
            Err(_) => return false,
        };
        for lowered_stmt in lowered {
            self.push_captured_stmt(&lowered_stmt);
        }
        true
    }

    fn try_lower_try_except_stmt_for_ir(
        &mut self,
        body: &[HirStmt],
        handlers: &[HirExceptHandler],
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        if handlers.is_empty() {
            return Ok(None);
        }
        let err_ty = select_try_error_type(handlers);
        let capture_returns =
            queries::body_contains_return(body) && self.current_return_type.is_some();
        let direct_return_capture = capture_returns
            && queries::block_control_flow_effect(body).always_exits()
            && handlers
                .iter()
                .all(|handler| queries::block_control_flow_effect(&handler.body).always_exits());
        let ok_ty = if capture_returns {
            if let Some(return_ty) = self.current_return_type.as_ref() {
                if direct_return_capture {
                    crate::render_type(&crate::sifr_type_to_rust_type(return_ty))
                } else {
                    format!(
                        "Option<{}>",
                        crate::render_type(&crate::sifr_type_to_rust_type(return_ty))
                    )
                }
            } else {
                "()".to_string()
            }
        } else {
            "()".to_string()
        };

        let mut closure_body = {
            if capture_returns {
                self.try_closure_depth += 1;
                self.try_closure_option_wrap.push(!direct_return_capture);
            }
            self.try_closure_error_type.push(err_ty.clone());
            let lowered = self.try_lower_stmt_block_for_ir(body)?;
            if capture_returns {
                self.try_closure_depth -= 1;
                self.try_closure_option_wrap.pop();
            }
            self.try_closure_error_type.pop();
            let Some(lowered) = lowered else {
                return Ok(None);
            };
            lowered
        };

        if !capture_returns {
            closure_body.push(RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("Ok".to_string())),
                args: vec![RustExpr::Literal(crate::RustLiteral::Unit)],
            })));
        } else if direct_return_capture {
            closure_body.push(RustStmt::Expr(RustExpr::FormatMacro {
                name: "unreachable".to_string(),
                format_str: "sifr try/except return capture fell through".to_string(),
                args: vec![],
            }));
        } else {
            closure_body.push(RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("Ok".to_string())),
                args: vec![RustExpr::Literal(crate::RustLiteral::None)],
            })));
        }

        let mut lowered = vec![RustStmt::Let {
            mutable: false,
            name: "__sifr_try_res".to_string(),
            ty: Some(crate::RustType::Result(
                Box::new(crate::RustType::Named(ok_ty.clone())),
                Box::new(crate::RustType::Named(err_ty.clone())),
            )),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Paren(Box::new(RustExpr::ClosureBlock {
                    params: vec![],
                    body: closure_body,
                    is_move: false,
                }))),
                args: vec![],
            },
        }];

        if capture_returns {
            let mut arms = vec![crate::RustMatchArm {
                pattern: if direct_return_capture {
                    "Ok(__sifr_ret_val)".to_string()
                } else {
                    "Ok(Some(__sifr_ret_val))".to_string()
                },
                bindings: vec!["__sifr_ret_val".to_string()],
                guard: None,
                body: vec![RustStmt::Return(Some(RustExpr::Ident(
                    "__sifr_ret_val".to_string(),
                )))],
            }];
            if !direct_return_capture {
                arms.push(crate::RustMatchArm {
                    pattern: "Ok(None)".to_string(),
                    bindings: vec![],
                    guard: None,
                    body: vec![],
                });
            }
            let Some(handler_chain) =
                self.lower_try_except_handler_chain_for_ir(handlers, "__sifr_try_err", &err_ty)?
            else {
                return Ok(None);
            };
            arms.push(crate::RustMatchArm {
                pattern: "Err(__sifr_try_err)".to_string(),
                bindings: vec!["__sifr_try_err".to_string()],
                guard: None,
                body: handler_chain,
            });
            lowered.push(RustStmt::Match {
                expr: RustExpr::Ident("__sifr_try_res".to_string()),
                arms,
            });
        } else {
            let Some(handler_chain) =
                self.lower_try_except_handler_chain_for_ir(handlers, "__sifr_try_err", &err_ty)?
            else {
                return Ok(None);
            };
            lowered.push(RustStmt::IfLet {
                pattern: "Err(__sifr_try_err)".to_string(),
                expr: RustExpr::Ident("__sifr_try_res".to_string()),
                then_body: handler_chain,
                else_body: None,
            });
        }
        Ok(Some(lowered))
    }

    fn try_except_handler_condition_expr(
        handler: &HirExceptHandler,
        err_ident: &str,
        err_ty: &str,
    ) -> HandlerMatchCondition {
        let Some(error_type) = handler.error_type.as_deref() else {
            return HandlerMatchCondition::Always;
        };
        if error_type == "Error" {
            return HandlerMatchCondition::Always;
        }
        if err_ty == "IOError" {
            if error_type == "IOError" {
                return HandlerMatchCondition::Always;
            }
            if let Some(kind) = io_error_kind_for_handler(error_type) {
                return HandlerMatchCondition::Expr(RustExpr::BinOp {
                    left: Box::new(RustExpr::Field {
                        expr: Box::new(RustExpr::Ident(err_ident.to_string())),
                        field: "kind".to_string(),
                    }),
                    op: "==".to_string(),
                    right: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Literal(crate::RustLiteral::Str(
                            kind.to_string(),
                        ))),
                        method: "to_string".to_string(),
                        args: vec![],
                    }),
                });
            }
            return HandlerMatchCondition::Unsupported;
        }
        if error_type == err_ty {
            return HandlerMatchCondition::Always;
        }
        HandlerMatchCondition::Unsupported
    }

    fn lower_try_except_handler_chain_for_ir(
        &mut self,
        handlers: &[HirExceptHandler],
        err_ident: &str,
        err_ty: &str,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let mut branches: Vec<(Option<RustExpr>, Vec<RustStmt>)> = Vec::new();
        for handler in handlers {
            let condition = Self::try_except_handler_condition_expr(handler, err_ident, err_ty);
            if matches!(condition, HandlerMatchCondition::Unsupported) {
                continue;
            }

            let mut handler_body = Vec::new();
            let handler_name = handler.name.as_deref().unwrap_or("_e");
            if handler_name != "_" {
                handler_body.push(RustStmt::Let {
                    mutable: false,
                    name: handler_name.to_string(),
                    ty: None,
                    value: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(err_ident.to_string())),
                        method: "clone".to_string(),
                        args: vec![],
                    },
                });
            }
            match self.try_lower_stmt_block_for_ir(&handler.body) {
                Ok(Some(lowered_handler_body)) => handler_body.extend(lowered_handler_body),
                Ok(None) => return Ok(None),
                Err(err) => return Err(err),
            }

            let cond_expr = match condition {
                HandlerMatchCondition::Always => None,
                HandlerMatchCondition::Expr(cond) => Some(cond),
                HandlerMatchCondition::Unsupported => continue,
            };
            branches.push((cond_expr, handler_body));
        }

        if branches.is_empty() {
            return Ok(Some(vec![RustStmt::Let {
                mutable: false,
                name: "_".to_string(),
                ty: None,
                value: RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Ident(err_ident.to_string())),
                },
            }]));
        }

        let mut current_else: Option<Vec<RustStmt>> = None;
        for (cond, body) in branches.into_iter().rev() {
            if let Some(cond) = cond {
                current_else = Some(vec![RustStmt::If {
                    cond,
                    then_body: body,
                    else_body: current_else,
                }]);
            } else {
                current_else = Some(body);
            }
        }
        Ok(Some(current_else.unwrap_or_default()))
    }

    pub(crate) fn try_lower_structured_field_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::FieldAssign {
            object,
            field,
            value,
        } = stmt
        else {
            return Ok(false);
        };

        let target = crate::RustExpr::Field {
            expr: Box::new(Self::object_name_expr_for_ir(object)),
            field: field.clone(),
        };

        if self.current_class_name.as_deref() == Some("deque") && field == "_data" {
            if let HirExpr::ListLiteral { elements, .. } = value {
                let value_expr = if elements.is_empty() {
                    crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "VecDeque".to_string(),
                            "new".to_string(),
                        ])),
                        args: vec![],
                    }
                } else {
                    let Some(list_expr) = self.lower_stmt_expr_for_ir(value)? else {
                        return Ok(false);
                    };
                    crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "VecDeque".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![list_expr],
                    }
                };
                let lowered = crate::RustStmt::Assign {
                    target,
                    value: value_expr,
                };
                self.emit_lowered_stmts(std::slice::from_ref(&lowered));
                return Ok(true);
            }
        }

        let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(false);
        };
        let value_expr = if object == "self"
            && self.current_class_name.as_ref().is_some_and(|class_name| {
                self.recursive_fields
                    .contains(&(class_name.clone(), field.clone()))
            })
            && !Self::is_box_new_call_expr_for_ir(&value_expr)
        {
            crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec![
                    "Box".to_string(),
                    "new".to_string(),
                ])),
                args: vec![value_expr],
            }
        } else {
            value_expr
        };
        let lowered = crate::RustStmt::Assign {
            target,
            value: value_expr,
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    pub(crate) fn try_lower_structured_attribute_subscript_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::AttributeSubscriptAssign {
            object,
            field,
            index,
            value,
            field_ty,
        } = stmt
        else {
            return Ok(false);
        };
        let Type::Dict(key_ty, _) = field_ty else {
            return Ok(false);
        };

        let key_needs_clone = matches!(key_ty.as_ref(), Type::Str | Type::TypeVar(_))
            && matches!(index, HirExpr::Name { name, .. }
                if self.borrowed_params.contains(name.as_str()) || self.mut_borrowed_params.contains(name.as_str()));

        let Some(mut index_expr) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(false);
        };
        if key_needs_clone {
            index_expr = crate::RustExpr::Clone(Box::new(index_expr));
        }
        let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(false);
        };

        let receiver = crate::RustExpr::Field {
            expr: Box::new(Self::object_name_expr_for_ir(object)),
            field: field.clone(),
        };
        let lowered = crate::RustStmt::Expr(crate::RustExpr::MethodCall {
            receiver: Box::new(receiver),
            method: "insert".to_string(),
            args: vec![index_expr, value_expr],
        });
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    pub(crate) fn try_lower_structured_assert_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::Assert { test, msg } = stmt else {
            return Ok(false);
        };

        let Some(lowered_test) = self.lower_rendered_expr_for_ir(test)? else {
            return Ok(false);
        };
        let lowered_msg = if let Some(msg_expr) = msg {
            let Some(lowered) = self.lower_rendered_expr_for_ir(msg_expr)? else {
                return Ok(false);
            };
            Some(lowered)
        } else {
            None
        };
        self.push_captured_stmt(&RustStmt::Assert {
            cond: lowered_test,
            msg: lowered_msg,
        });
        Ok(true)
    }

    pub(crate) fn try_lower_structured_aug_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::AugAssign { name, op, value } = stmt else {
            return Ok(false);
        };
        let value_ty = Self::resolve_alias_type_for_loop_iter(value.ty());

        if op == "+=" {
            match value_ty {
                Type::Str => {
                    let arg_expr = if let HirExpr::StringLiteral(val) = value {
                        crate::RustExpr::Ident(format!("{val:?}"))
                    } else {
                        let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                            return Ok(false);
                        };
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(value_expr))),
                            method: "as_str".to_string(),
                            args: vec![],
                        }
                    };
                    let lowered = crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                        method: "push_str".to_string(),
                        args: vec![arg_expr],
                    });
                    self.emit_lowered_stmts(std::slice::from_ref(&lowered));
                    return Ok(true);
                }
                Type::List(_) => {
                    let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                        return Ok(false);
                    };
                    let lowered = crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                        method: "extend".to_string(),
                        args: vec![value_expr],
                    });
                    self.emit_lowered_stmts(std::slice::from_ref(&lowered));
                    return Ok(true);
                }
                _ => {}
            }
        }

        if op == "**=" {
            let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                return Ok(false);
            };
            let pow_value = crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Paren(Box::new(crate::RustExpr::Ident(
                    name.clone(),
                )))),
                method: "pow".to_string(),
                args: vec![crate::RustExpr::Cast {
                    expr: Box::new(value_expr),
                    ty: crate::RustType::Named("u32".to_string()),
                }],
            };
            let lowered = crate::RustStmt::Assign {
                target: crate::RustExpr::Ident(name.clone()),
                value: pow_value,
            };
            self.emit_lowered_stmts(std::slice::from_ref(&lowered));
            return Ok(true);
        }
        let rust_op = match op.as_str() {
            "+=" => "+=",
            "-=" => "-=",
            "*=" => "*=",
            "/=" | "//=" => "/=",
            "%=" => "%=",
            _ => return Ok(false),
        };

        let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(false);
        };
        let lowered = crate::RustStmt::AugAssign {
            target: crate::RustExpr::Ident(name.clone()),
            op: rust_op.to_string(),
            value: value_expr,
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }
}
