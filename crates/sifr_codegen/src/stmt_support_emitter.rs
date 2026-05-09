use crate::hir_analysis::queries;
use crate::{RustEmitter, RustExpr, RustStmt};
use sifr_hir::{HirExceptHandler, HirExpr, HirFStringPart, HirIteratorOp, HirStmt};
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
            | "JsonIntegerRangeError"
            | "JsonLimitError"
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

fn canonical_plain_call_name_for_ir(func: &str) -> &str {
    func.strip_prefix("__compat_sifr_math_")
        .or_else(|| func.strip_prefix("__compat_sifr_heapq_"))
        .unwrap_or(func)
}

fn supports_nonempty_pop_narrowing_type_for_ir(object_ty: &Type) -> bool {
    match crate::resolve_alias_type_for_plain_call(object_ty) {
        Type::List(_) => true,
        Type::Class { name, .. } => is_deque_class_name_for_ir(name),
        _ => false,
    }
}

fn is_deque_class_name_for_ir(name: &str) -> bool {
    name == "deque"
        || name
            .rsplit_once('.')
            .is_some_and(|(_, tail)| tail == "deque")
}

fn is_narrowable_pop_call_for_ir(method: &str, args: &[HirExpr]) -> bool {
    match method {
        "pop" => matches!(args, [] | [HirExpr::IntLiteral(0)]),
        "popleft" => args.is_empty(),
        _ => false,
    }
}

fn unwrap_compiler_verified_nonempty_pop_result_for_ir(
    object_ty: &Type,
    method: &str,
    args: &[HirExpr],
    method_return_ty: &Type,
    lowered_expr: RustExpr,
) -> RustExpr {
    if !supports_nonempty_pop_narrowing_type_for_ir(object_ty) {
        return lowered_expr;
    }
    if !is_narrowable_pop_call_for_ir(method, args) {
        return lowered_expr;
    }
    if crate::helpers::is_option_type(method_return_ty) {
        return lowered_expr;
    }
    RustExpr::Block {
        stmts: vec![RustStmt::LetElse {
            pattern: "Some(__sifr_nonempty_pop_value)".to_string(),
            value: lowered_expr,
            else_body: vec![RustStmt::Expr(RustExpr::MacroCall {
                name: "unreachable".to_string(),
                args: vec![RustExpr::Literal(crate::RustLiteral::Str(
                    "compiler-verified non-empty pop should return Some".to_string(),
                ))],
            })],
        }],
        expr: Some(Box::new(RustExpr::Ident(
            "__sifr_nonempty_pop_value".to_string(),
        ))),
    }
}

fn iterator_call_func_name(op: &HirIteratorOp) -> &'static str {
    match op {
        HirIteratorOp::Iter => "iter",
        HirIteratorOp::Next => "next",
        HirIteratorOp::Reversed => "reversed",
        HirIteratorOp::Map => "map",
        HirIteratorOp::Filter => "filter",
        HirIteratorOp::Zip => "zip",
        HirIteratorOp::Enumerate => "enumerate",
    }
}

fn call_expr_parts(expr: &HirExpr) -> Option<(&str, &[HirExpr])> {
    match expr {
        HirExpr::Call { func, args, .. } => Some((func.as_str(), args.as_slice())),
        HirExpr::IteratorCall { op, args, .. } => {
            Some((iterator_call_func_name(op), args.as_slice()))
        }
        _ => None,
    }
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
    fn class_has_next_protocol(ty: &Type) -> bool {
        let Type::Class { methods, .. } = ty.resolve_alias() else {
            return false;
        };
        methods.iter().any(|(name, ft)| {
            name == "__next__"
                && ft.params.is_empty()
                && matches!(ft.return_type.as_ref().resolve_alias(), Type::Union(members) if {
                    let has_none = members
                        .iter()
                        .any(|member| matches!(member.resolve_alias(), Type::None));
                    let non_none = members
                        .iter()
                        .filter(|member| !matches!(member.resolve_alias(), Type::None))
                        .count();
                    has_none && non_none == 1
                })
        })
    }

    matches!(
        ty,
        Type::Alias { name: alias_name, .. } if alias_name.starts_with("__compat_defaultdict_")
    ) || matches!(ty.resolve_alias(), Type::Iterator(_))
        || class_has_next_protocol(ty)
}

fn type_contains_any_or_unknown(ty: &Type) -> bool {
    match crate::resolve_alias_type_for_plain_call(ty) {
        Type::Any | Type::Unknown => true,
        Type::List(inner)
        | Type::Set(inner)
        | Type::Iterable(inner)
        | Type::Iterator(inner)
        | Type::Alias { body: inner, .. } => type_contains_any_or_unknown(inner),
        Type::Dict(key, value) | Type::Result(key, value) => {
            type_contains_any_or_unknown(key) || type_contains_any_or_unknown(value)
        }
        Type::Tuple(elements) | Type::Union(elements) | Type::Intersection(elements) => {
            elements.iter().any(type_contains_any_or_unknown)
        }
        Type::Callable(params, _, ret) => {
            params.iter().any(type_contains_any_or_unknown) || type_contains_any_or_unknown(ret)
        }
        Type::Function(ft) => {
            ft.params
                .iter()
                .any(|(_, param_ty, _)| type_contains_any_or_unknown(param_ty))
                || type_contains_any_or_unknown(&ft.return_type)
        }
        _ => false,
    }
}

impl RustEmitter {
    pub(super) fn wrap_option_local_value_for_ir(
        target_ty: &Type,
        value: &HirExpr,
        value_ty: &Type,
        lowered_value: crate::RustExpr,
    ) -> crate::RustExpr {
        if !crate::helpers::is_option_type(target_ty) {
            return lowered_value;
        }
        if matches!(value, HirExpr::NoneLiteral) || matches!(value_ty, Type::None) {
            return crate::RustExpr::Literal(crate::RustLiteral::None);
        }
        if crate::helpers::is_option_type(value_ty) {
            return lowered_value;
        }
        crate::RustExpr::FnCall {
            func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
            args: vec![lowered_value],
        }
    }

    pub(super) fn coerce_local_value_for_target_type_for_ir(
        &mut self,
        target_ty: &Type,
        value: &HirExpr,
        lowered_value: crate::RustExpr,
    ) -> Result<crate::RustExpr, crate::CodegenError> {
        if matches!(
            crate::resolve_alias_type_for_plain_call(target_ty),
            Type::Iterable(_)
        ) {
            if let Some(coerced) =
                crate::intrinsic_method_emitters::registry_iterable_to_vec_expr(self, value)
            {
                return Ok(coerced);
            }
            return Err(crate::CodegenError::new(
                "failed to coerce iterable local binding value",
            ));
        }
        let value_ty = if let HirExpr::Name { name, ty } = value {
            if self.none_widened_local_bindings.contains(name)
                || matches!(
                    crate::resolve_alias_type_for_plain_call(ty),
                    Type::Any | Type::Unknown
                )
            {
                self.local_binding_types.get(name).unwrap_or(ty)
            } else {
                ty
            }
        } else {
            value.ty()
        };
        if let Some(coerced) = crate::fixed_width_literal_expr_for_target(target_ty, value) {
            return Ok(coerced);
        }
        Ok(Self::wrap_option_local_value_for_ir(
            target_ty,
            value,
            value_ty,
            lowered_value,
        ))
    }

    pub(crate) fn force_unwrap_option_expr_for_ir(
        value_expr: crate::RustExpr,
        guard_message: &str,
    ) -> crate::RustExpr {
        crate::RustExpr::Block {
            stmts: vec![crate::RustStmt::LetElse {
                pattern: "Some(__sifr_unwrapped_option_value)".to_string(),
                value: value_expr,
                else_body: vec![crate::RustStmt::Expr(crate::RustExpr::MacroCall {
                    name: "unreachable".to_string(),
                    args: vec![crate::RustExpr::Literal(crate::RustLiteral::Str(
                        guard_message.to_string(),
                    ))],
                })],
            }],
            expr: Some(Box::new(crate::RustExpr::Ident(
                "__sifr_unwrapped_option_value".to_string(),
            ))),
        }
    }

    fn uses_debug_display_format_for_ir(ty: &Type) -> bool {
        match crate::resolve_alias_type_for_plain_call(ty) {
            Type::Int
            | Type::FixedInt(_)
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
            | Type::AsyncFunction(_)
            | Type::Coroutine(_, _)
            | Type::Task(_, _)
            | Type::TaskResult(_, _)
            | Type::BlockingTask(_, _)
            | Type::Awaitable(_)
            | Type::AsyncIterator(_, _)
            | Type::AsyncGenerator(_, _)
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
        if let HirExpr::IteratorCall { op, args, .. } = iter_expr {
            if *op == HirIteratorOp::Iter && args.len() == 1 {
                return self.lower_structural_iter_source_expr_for_ir(&args[0], None);
            }
        }
        self.lower_structural_iter_source_expr_for_ir(iter_expr, None)
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

    fn try_lower_generator_expr_for_ir(
        &mut self,
        value_expr: &HirExpr,
        var: &str,
        iter_expr: &HirExpr,
        filter: Option<&HirExpr>,
        result_ty: &Type,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        if var.contains(',')
            || !matches!(
                Self::resolve_alias_type_for_loop_iter(result_ty),
                Type::Any | Type::Iterator(_)
            )
        {
            return Ok(None);
        }

        let Some(iter_chain) = self.lower_comprehension_iter_for_ir(iter_expr)? else {
            return Ok(None);
        };
        let Some(lowered_value_expr) = self.lower_stmt_expr_for_ir(value_expr)? else {
            return Ok(None);
        };
        let lowered_body = if let Some(filter_expr) = filter {
            let Some(lowered_filter_expr) = self.lower_stmt_expr_for_ir(filter_expr)? else {
                return Ok(None);
            };
            RustExpr::If {
                cond: Box::new(lowered_filter_expr),
                then_expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                    args: vec![lowered_value_expr],
                }),
                else_expr: Some(Box::new(RustExpr::Literal(crate::RustLiteral::None))),
            }
        } else {
            RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                args: vec![lowered_value_expr],
            }
        };

        let generator_chain = RustExpr::MethodCall {
            receiver: Box::new(iter_chain),
            method: "filter_map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![crate::RustParam::Named {
                    name: var.to_string(),
                    ty: crate::RustType::Named("_".to_string()),
                }],
                body: Box::new(lowered_body),
                is_move: false,
            }],
        };
        if matches!(
            Self::resolve_alias_type_for_loop_iter(result_ty),
            Type::Iterator(_)
        ) {
            return Ok(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                args: vec![generator_chain],
            }));
        }
        Ok(Some(generator_chain))
    }

    fn lower_structured_nested_list_subscript_assign_stmt_for_ir(
        &mut self,
        object: &str,
        outer_index: &HirExpr,
        inner_index: &HirExpr,
        value: &HirExpr,
        target_elem_ty: &Type,
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
        let lowered_value = Self::clone_non_copy_name_expr_for_ir(value, lowered_value);
        let lowered_value = if value.ty().rust_type().starts_with('&')
            && !target_elem_ty.rust_type().starts_with('&')
        {
            crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_value))),
                method: "clone".to_string(),
                args: vec![],
            }
        } else {
            lowered_value
        };

        let value_is_option = if crate::helpers::is_option_type(value.ty()) {
            true
        } else if let HirExpr::Name { name, ty } = value {
            matches!(
                crate::resolve_alias_type_for_plain_call(ty),
                Type::Any | Type::Unknown
            ) && self
                .local_binding_types
                .get(name)
                .is_some_and(crate::helpers::is_option_type)
        } else {
            false
        };
        let assign_into_elem = if value_is_option && !crate::helpers::is_option_type(target_elem_ty)
        {
            RustStmt::IfLet {
                pattern: "Some(__nested_assign_value)".to_string(),
                expr: RustExpr::Ident("__nested_assign_value".to_string()),
                then_body: vec![RustStmt::Assign {
                    target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
                    value: RustExpr::Ident("__nested_assign_value".to_string()),
                }],
                else_body: None,
            }
        } else {
            RustStmt::Assign {
                target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
                value: RustExpr::Ident("__nested_assign_value".to_string()),
            }
        };
        let index_is_option = |expr: &HirExpr| {
            if crate::helpers::is_option_type(expr.ty()) {
                return true;
            }
            if let HirExpr::Name { name, ty } = expr {
                return matches!(
                    crate::resolve_alias_type_for_plain_call(ty),
                    Type::Any | Type::Unknown
                ) && self
                    .local_binding_types
                    .get(name)
                    .is_some_and(crate::helpers::is_option_type);
            }
            false
        };
        let outer_index_is_option = index_is_option(outer_index);
        let inner_index_is_option = index_is_option(inner_index);

        let mut inner_then_body = vec![RustStmt::Let {
            mutable: false,
            name: "__ii_norm".to_string(),
            ty: None,
            value: crate::build_normalized_list_index_i64_expr(
                RustExpr::Ident("__row".to_string()),
                "__ii_raw",
            ),
        }];
        inner_then_body.push(RustStmt::If {
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
                then_body: vec![assign_into_elem],
                else_body: None,
            }],
            else_body: None,
        });
        let inner_body = if inner_index_is_option {
            vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__ii_raw_opt".to_string(),
                    ty: None,
                    value: lowered_inner_index,
                },
                RustStmt::IfLet {
                    pattern: "Some(__ii_raw)".to_string(),
                    expr: RustExpr::Ident("__ii_raw_opt".to_string()),
                    then_body: inner_then_body,
                    else_body: None,
                },
            ]
        } else {
            let mut inner_body = vec![RustStmt::Let {
                mutable: false,
                name: "__ii_raw".to_string(),
                ty: None,
                value: lowered_inner_index,
            }];
            inner_body.extend(inner_then_body);
            inner_body
        };

        let mut outer_then_body = vec![RustStmt::Let {
            mutable: false,
            name: "__oi_norm".to_string(),
            ty: None,
            value: crate::build_normalized_list_index_i64_expr(
                RustExpr::Ident(object.to_string()),
                "__oi_raw",
            ),
        }];
        outer_then_body.push(RustStmt::If {
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
                then_body: inner_body,
                else_body: None,
            }],
            else_body: None,
        });

        let outer_body = if outer_index_is_option {
            vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__oi_raw_opt".to_string(),
                    ty: None,
                    value: lowered_outer_index,
                },
                RustStmt::IfLet {
                    pattern: "Some(__oi_raw)".to_string(),
                    expr: RustExpr::Ident("__oi_raw_opt".to_string()),
                    then_body: outer_then_body,
                    else_body: None,
                },
            ]
        } else {
            let mut outer_body = vec![RustStmt::Let {
                mutable: false,
                name: "__oi_raw".to_string(),
                ty: None,
                value: lowered_outer_index,
            }];
            outer_body.extend(outer_then_body);
            outer_body
        };

        Ok(Some(RustStmt::Block(vec![
            RustStmt::Let {
                mutable: false,
                name: "__nested_assign_value".to_string(),
                ty: None,
                value: lowered_value,
            },
            RustStmt::Block(outer_body),
        ])))
    }

    fn lower_structured_attribute_nested_list_subscript_assign_stmt_for_ir(
        &mut self,
        object: &str,
        field: &str,
        outer_index: &HirExpr,
        inner_index: &HirExpr,
        value: &HirExpr,
        field_ty: &Type,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let Type::List(inner) = Self::resolve_alias_type_for_loop_iter(field_ty) else {
            return Ok(None);
        };
        let Type::List(target_elem_ty) = Self::resolve_alias_type_for_loop_iter(inner) else {
            return Ok(None);
        };
        let Some(lowered_outer_index) = self.lower_stmt_expr_for_ir(outer_index)? else {
            return Ok(None);
        };
        let Some(lowered_inner_index) = self.lower_stmt_expr_for_ir(inner_index)? else {
            return Ok(None);
        };
        let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(None);
        };
        let lowered_value = Self::clone_non_copy_name_expr_for_ir(value, lowered_value);
        let lowered_value = if value.ty().rust_type().starts_with('&')
            && !target_elem_ty.rust_type().starts_with('&')
        {
            crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_value))),
                method: "clone".to_string(),
                args: vec![],
            }
        } else {
            lowered_value
        };

        let value_is_option = if crate::helpers::is_option_type(value.ty()) {
            true
        } else if let HirExpr::Name { name, ty } = value {
            matches!(
                crate::resolve_alias_type_for_plain_call(ty),
                Type::Any | Type::Unknown
            ) && self
                .local_binding_types
                .get(name)
                .is_some_and(crate::helpers::is_option_type)
        } else {
            false
        };
        let assign_into_elem =
            if value_is_option && !crate::helpers::is_option_type(target_elem_ty.as_ref()) {
                RustStmt::IfLet {
                    pattern: "Some(__nested_assign_value)".to_string(),
                    expr: RustExpr::Ident("__nested_assign_value".to_string()),
                    then_body: vec![RustStmt::Assign {
                        target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
                        value: RustExpr::Ident("__nested_assign_value".to_string()),
                    }],
                    else_body: None,
                }
            } else {
                RustStmt::Assign {
                    target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
                    value: RustExpr::Ident("__nested_assign_value".to_string()),
                }
            };
        let field_receiver = || RustExpr::Field {
            expr: Box::new(Self::object_name_expr_for_ir(object)),
            field: field.to_string(),
        };
        let index_is_option = |expr: &HirExpr| {
            if crate::helpers::is_option_type(expr.ty()) {
                return true;
            }
            if let HirExpr::Name { name, ty } = expr {
                return matches!(
                    crate::resolve_alias_type_for_plain_call(ty),
                    Type::Any | Type::Unknown
                ) && self
                    .local_binding_types
                    .get(name)
                    .is_some_and(crate::helpers::is_option_type);
            }
            false
        };
        let outer_index_is_option = index_is_option(outer_index);
        let inner_index_is_option = index_is_option(inner_index);

        let mut inner_then_body = vec![RustStmt::Let {
            mutable: false,
            name: "__ii_norm".to_string(),
            ty: None,
            value: crate::build_normalized_list_index_i64_expr(
                RustExpr::Ident("__row".to_string()),
                "__ii_raw",
            ),
        }];
        inner_then_body.push(RustStmt::If {
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
                then_body: vec![assign_into_elem],
                else_body: None,
            }],
            else_body: None,
        });
        let inner_body = if inner_index_is_option {
            vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__ii_raw_opt".to_string(),
                    ty: None,
                    value: lowered_inner_index,
                },
                RustStmt::IfLet {
                    pattern: "Some(__ii_raw)".to_string(),
                    expr: RustExpr::Ident("__ii_raw_opt".to_string()),
                    then_body: inner_then_body,
                    else_body: None,
                },
            ]
        } else {
            let mut inner_body = vec![RustStmt::Let {
                mutable: false,
                name: "__ii_raw".to_string(),
                ty: None,
                value: lowered_inner_index,
            }];
            inner_body.extend(inner_then_body);
            inner_body
        };

        let mut outer_then_body = vec![RustStmt::Let {
            mutable: false,
            name: "__oi_norm".to_string(),
            ty: None,
            value: crate::build_normalized_list_index_i64_expr(field_receiver(), "__oi_raw"),
        }];
        outer_then_body.push(RustStmt::If {
            cond: RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("__oi_norm".to_string())),
                op: ">=".to_string(),
                right: Box::new(RustExpr::Literal(crate::RustLiteral::Int(0))),
            },
            then_body: vec![RustStmt::IfLet {
                pattern: "Some(__row)".to_string(),
                expr: RustExpr::MethodCall {
                    receiver: Box::new(field_receiver()),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Cast {
                        expr: Box::new(RustExpr::Ident("__oi_norm".to_string())),
                        ty: crate::RustType::Named("usize".to_string()),
                    }],
                },
                then_body: inner_body,
                else_body: None,
            }],
            else_body: None,
        });

        let outer_body = if outer_index_is_option {
            vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__oi_raw_opt".to_string(),
                    ty: None,
                    value: lowered_outer_index,
                },
                RustStmt::IfLet {
                    pattern: "Some(__oi_raw)".to_string(),
                    expr: RustExpr::Ident("__oi_raw_opt".to_string()),
                    then_body: outer_then_body,
                    else_body: None,
                },
            ]
        } else {
            let mut outer_body = vec![RustStmt::Let {
                mutable: false,
                name: "__oi_raw".to_string(),
                ty: None,
                value: lowered_outer_index,
            }];
            outer_body.extend(outer_then_body);
            outer_body
        };

        Ok(Some(RustStmt::Block(vec![
            RustStmt::Let {
                mutable: false,
                name: "__nested_assign_value".to_string(),
                ty: None,
                value: lowered_value,
            },
            RustStmt::Block(outer_body),
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
        let clone_non_copy_name = |expr: &HirExpr, lowered: crate::RustExpr| {
            if matches!(expr, HirExpr::Name { .. })
                && !crate::helpers::is_copy_type_for_codegen(expr.ty())
            {
                crate::RustExpr::Clone(Box::new(lowered))
            } else {
                lowered
            }
        };

        match Self::resolve_alias_type_for_loop_iter(object_ty) {
            Type::List(_) => Ok(Some(RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__assign_value".to_string(),
                    ty: None,
                    value: clone_non_copy_name(value, lowered_value),
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
                    clone_non_copy_name(index, lowered_index)
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
                        value: clone_non_copy_name(value, lowered_value),
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

    pub(crate) fn clone_non_copy_name_expr_for_ir(
        expr: &HirExpr,
        lowered: crate::RustExpr,
    ) -> crate::RustExpr {
        if matches!(expr, HirExpr::Name { .. })
            && !crate::helpers::is_copy_type_for_codegen(expr.ty())
        {
            crate::RustExpr::Clone(Box::new(lowered))
        } else {
            lowered
        }
    }

    fn build_dict_lookup_key_arg_for_ir(lowered_index: crate::RustExpr) -> crate::RustExpr {
        if matches!(
            lowered_index,
            crate::RustExpr::Literal(crate::RustLiteral::Str(_))
        ) {
            lowered_index
        } else {
            crate::RustExpr::Ref {
                mutable: false,
                expr: Box::new(lowered_index),
            }
        }
    }

    fn build_subscript_augassign_elem_stmt_for_ir(
        op: &str,
        lowered_value: crate::RustExpr,
    ) -> Option<crate::RustStmt> {
        if op == "**=" {
            return Some(crate::RustStmt::Assign {
                target: crate::RustExpr::Deref(Box::new(crate::RustExpr::Ident(
                    "__elem".to_string(),
                ))),
                value: crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Ident("__elem".to_string())),
                    method: "pow".to_string(),
                    args: vec![crate::RustExpr::Cast {
                        expr: Box::new(lowered_value),
                        ty: crate::RustType::Named("u32".to_string()),
                    }],
                },
            });
        }
        if op == "//=" {
            return Some(crate::RustStmt::Assign {
                target: crate::RustExpr::Deref(Box::new(crate::RustExpr::Ident(
                    "__elem".to_string(),
                ))),
                value: crate::RustExpr::BinOp {
                    left: Box::new(crate::RustExpr::Deref(Box::new(crate::RustExpr::Ident(
                        "__elem".to_string(),
                    )))),
                    op: "/".to_string(),
                    right: Box::new(lowered_value),
                },
            });
        }
        let rust_op = op.strip_suffix('=')?;
        Some(crate::RustStmt::AugAssign {
            target: crate::RustExpr::Deref(Box::new(crate::RustExpr::Ident("__elem".to_string()))),
            op: rust_op.to_string(),
            value: lowered_value,
        })
    }

    fn lower_subscript_augassign_stmt_for_ir(
        &mut self,
        object: &str,
        index: &HirExpr,
        op: &str,
        value: &HirExpr,
        object_ty: &Type,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        if !matches!(
            op,
            "+=" | "-=" | "*=" | "/=" | "%=" | "//=" | "**=" | "&=" | "|=" | "^=" | "<<=" | ">>="
        ) {
            return Ok(None);
        }
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };
        let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(None);
        };
        if op == "+="
            && matches!(
                Self::resolve_alias_type_for_loop_iter(object_ty),
                Type::List(elem_ty)
                    if matches!(
                        crate::resolve_alias_type_for_plain_call(elem_ty.as_ref()),
                        Type::Str | Type::LiteralStr(_)
                    )
            )
        {
            let push_arg = if let HirExpr::StringLiteral(val) = value {
                crate::RustExpr::Ident(format!("{val:?}"))
            } else {
                crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_value))),
                    method: "as_str".to_string(),
                    args: vec![],
                }
            };
            return Ok(Some(RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__idx_raw".to_string(),
                    ty: None,
                    value: lowered_index,
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__idx_norm".to_string(),
                    ty: None,
                    value: crate::build_normalized_list_index_i64_expr(
                        crate::RustExpr::Ident(object.to_string()),
                        "__idx_raw",
                    ),
                },
                RustStmt::If {
                    cond: crate::RustExpr::BinOp {
                        left: Box::new(crate::RustExpr::Ident("__idx_norm".to_string())),
                        op: ">=".to_string(),
                        right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                    },
                    then_body: vec![RustStmt::IfLet {
                        pattern: "Some(__elem)".to_string(),
                        expr: crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident(object.to_string())),
                            method: "get_mut".to_string(),
                            args: vec![crate::RustExpr::Cast {
                                expr: Box::new(crate::RustExpr::Ident("__idx_norm".to_string())),
                                ty: crate::RustType::Named("usize".to_string()),
                            }],
                        },
                        then_body: vec![RustStmt::Expr(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident("__elem".to_string())),
                            method: "push_str".to_string(),
                            args: vec![push_arg],
                        })],
                        else_body: None,
                    }],
                    else_body: None,
                },
            ])));
        }
        let lowered_value = Self::clone_non_copy_name_expr_for_ir(value, lowered_value);
        let Some(lowered_body_stmt) =
            Self::build_subscript_augassign_elem_stmt_for_ir(op, lowered_value)
        else {
            return Ok(None);
        };

        if matches!(
            object_ty,
            Type::Alias { name: alias_name, .. } if alias_name == "__compat_defaultdict_int"
        ) {
            let lowered_index = Self::clone_non_copy_name_expr_for_ir(index, lowered_index);
            return Ok(Some(RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__elem".to_string(),
                    ty: None,
                    value: crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident(object.to_string())),
                            method: "entry".to_string(),
                            args: vec![lowered_index],
                        }),
                        method: "or_insert".to_string(),
                        args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                    },
                },
                lowered_body_stmt,
            ])));
        }

        match Self::resolve_alias_type_for_loop_iter(object_ty) {
            Type::List(_) => Ok(Some(RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__idx_raw".to_string(),
                    ty: None,
                    value: lowered_index,
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__idx_norm".to_string(),
                    ty: None,
                    value: crate::build_normalized_list_index_i64_expr(
                        crate::RustExpr::Ident(object.to_string()),
                        "__idx_raw",
                    ),
                },
                RustStmt::IfLet {
                    pattern: "Some(__elem)".to_string(),
                    expr: crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(object.to_string())),
                        method: "get_mut".to_string(),
                        args: vec![crate::RustExpr::Cast {
                            expr: Box::new(crate::RustExpr::Ident("__idx_norm".to_string())),
                            ty: crate::RustType::Named("usize".to_string()),
                        }],
                    },
                    then_body: vec![lowered_body_stmt],
                    else_body: None,
                },
            ]))),
            Type::Dict(_, _) => {
                let key_arg = Self::build_dict_lookup_key_arg_for_ir(
                    Self::clone_non_copy_name_expr_for_ir(index, lowered_index),
                );
                Ok(Some(RustStmt::IfLet {
                    pattern: "Some(__elem)".to_string(),
                    expr: crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(object.to_string())),
                        method: "get_mut".to_string(),
                        args: vec![key_arg],
                    },
                    then_body: vec![lowered_body_stmt],
                    else_body: None,
                }))
            }
            _ => Ok(None),
        }
    }

    fn lower_delete_stmt_for_ir(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
            return Ok(None);
        };
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };
        match Self::resolve_alias_type_for_loop_iter(object.ty()) {
            Type::List(_) => Ok(Some(RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__delete_target".to_string(),
                    ty: None,
                    value: crate::RustExpr::Ref {
                        mutable: true,
                        expr: Box::new(lowered_object),
                    },
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__idx_raw".to_string(),
                    ty: None,
                    value: lowered_index,
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__idx_norm".to_string(),
                    ty: None,
                    value: crate::build_normalized_list_index_i64_expr(
                        crate::RustExpr::Ident("__delete_target".to_string()),
                        "__idx_raw",
                    ),
                },
                RustStmt::If {
                    cond: crate::RustExpr::BinOp {
                        left: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Ident("__idx_norm".to_string())),
                            op: ">=".to_string(),
                            right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                        }),
                        op: "&&".to_string(),
                        right: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Cast {
                                expr: Box::new(crate::RustExpr::Ident("__idx_norm".to_string())),
                                ty: crate::RustType::Named("usize".to_string()),
                            }),
                            op: "<".to_string(),
                            right: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident(
                                    "__delete_target".to_string(),
                                )),
                                method: "len".to_string(),
                                args: vec![],
                            }),
                        }),
                    },
                    then_body: vec![RustStmt::Let {
                        mutable: false,
                        name: "_".to_string(),
                        ty: None,
                        value: crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident(
                                "__delete_target".to_string(),
                            )),
                            method: "remove".to_string(),
                            args: vec![crate::RustExpr::Cast {
                                expr: Box::new(crate::RustExpr::Ident("__idx_norm".to_string())),
                                ty: crate::RustType::Named("usize".to_string()),
                            }],
                        },
                    }],
                    else_body: None,
                },
            ]))),
            Type::Dict(_, _) => Ok(Some(RustStmt::Let {
                mutable: false,
                name: "_".to_string(),
                ty: None,
                value: crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_object),
                    method: "remove".to_string(),
                    args: vec![Self::build_dict_lookup_key_arg_for_ir(
                        Self::clone_non_copy_name_expr_for_ir(index, lowered_index),
                    )],
                },
            })),
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
        let Type::List(elem) = Self::resolve_alias_type_for_loop_iter(inner) else {
            return Ok(false);
        };

        let Some(lowered) = self.lower_structured_nested_list_subscript_assign_stmt_for_ir(
            object,
            outer_index,
            inner_index,
            value,
            elem,
        )?
        else {
            return Ok(false);
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    pub(crate) fn try_lower_structured_subscript_augassign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::SubscriptAugAssign {
            object,
            index,
            op,
            value,
            object_ty,
        } = stmt
        else {
            return Ok(false);
        };
        let Some(lowered) =
            self.lower_subscript_augassign_stmt_for_ir(object, index, op, value, object_ty)?
        else {
            return Ok(false);
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    pub(crate) fn try_lower_structured_delete_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::Delete { object, index } = stmt else {
            return Ok(false);
        };
        let Some(lowered) = self.lower_delete_stmt_for_ir(object, index)? else {
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
                | HirExpr::IteratorCall { .. }
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
                            let is_recursive_container_param = matches!(
                                crate::resolve_alias_type_for_plain_call(param_ty),
                                Type::List(elem)
                                    if matches!(
                                        crate::resolve_alias_type_for_plain_call(elem.as_ref()),
                                        Type::Class { name, .. } if name == class_name
                                    )
                            ) || matches!(
                                crate::resolve_alias_type_for_plain_call(param_ty),
                                Type::Dict(_, value_ty)
                                    if matches!(
                                        crate::resolve_alias_type_for_plain_call(value_ty.as_ref()),
                                        Type::Class { name, .. } if name == class_name
                                    )
                            );
                            let resolved_param = crate::resolve_alias_type_for_plain_call(param_ty);
                            if !crate::helpers::is_option_type(resolved_param) {
                                if (is_recursive_ctor_field || is_recursive_container_param)
                                    && !Self::is_box_new_call_expr_for_ir(lowered_arg)
                                {
                                    *lowered_arg = crate::RustExpr::FnCall {
                                        func: Box::new(crate::RustExpr::Path(vec![
                                            "Box".to_string(),
                                            "new".to_string(),
                                        ])),
                                        args: vec![lowered_arg.clone()],
                                    };
                                }
                                continue;
                            }
                            let needs_box_inner = param_ty.rust_type().starts_with("Option<Box<")
                                || is_recursive_ctor_field;
                            if !needs_box_inner || matches!(args[idx], HirExpr::NoneLiteral) {
                                continue;
                            }
                            let arg_is_option = crate::helpers::is_option_type(args[idx].ty());
                            if arg_is_option {
                                *lowered_arg =
                                    Self::ensure_option_box_inner_for_ir(lowered_arg.clone());
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
                let is_recursive_container_arg = matches!(
                    crate::resolve_alias_type_for_plain_call(args[idx].ty()),
                    Type::List(elem)
                        if matches!(
                            crate::resolve_alias_type_for_plain_call(elem.as_ref()),
                            Type::Class { name, .. } if name == class_name
                        )
                ) || matches!(
                    crate::resolve_alias_type_for_plain_call(args[idx].ty()),
                    Type::Dict(_, value_ty)
                        if matches!(
                            crate::resolve_alias_type_for_plain_call(value_ty.as_ref()),
                            Type::Class { name, .. } if name == class_name
                        )
                );
                if (!is_recursive_ctor_field && !is_recursive_container_arg)
                    || matches!(args[idx], HirExpr::NoneLiteral)
                {
                    continue;
                }
                let resolved_arg_ty = crate::resolve_alias_type_for_plain_call(args[idx].ty());
                if crate::helpers::is_option_type(resolved_arg_ty) {
                    *lowered_arg = Self::ensure_option_box_inner_for_ir(lowered_arg.clone());
                } else if !Self::is_box_new_call_expr_for_ir(lowered_arg) {
                    *lowered_arg = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "Box".to_string(),
                            "new".to_string(),
                        ])),
                        args: vec![lowered_arg.clone()],
                    };
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
                lowered_element = Self::clone_non_copy_name_expr_for_ir(element, lowered_element);
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
                lowered_elements.push(Self::clone_non_copy_name_expr_for_ir(
                    element,
                    lowered_element,
                ));
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
        if let HirExpr::GeneratorExpr {
            expr: value_expr,
            var,
            iter,
            filter,
            ty,
        } = expr
        {
            if let Some(lowered_generator) =
                self.try_lower_generator_expr_for_ir(value_expr, var, iter, filter.as_deref(), ty)?
            {
                return Ok(Some(lowered_generator));
            }
        }
        if let Some((func, args)) = call_expr_parts(expr) {
            if let Some(lowered_intrinsic) = self.try_lower_registry_intrinsic_call_expr(func, args)
            {
                return Ok(Some(lowered_intrinsic));
            }
            if let Some(lowered_builtin) =
                self.try_lower_registry_builtin_call_expr(func, args, Some(expr.ty()))
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
            let canonical_func = canonical_plain_call_name_for_ir(func);
            lowered_args = self.adapt_plain_call_args_with_signature_for_ir(
                canonical_func,
                args,
                lowered_args,
            );
            if let Some(captures) = self.nested_fn_captures.get(func).cloned() {
                for capture in captures {
                    lowered_args.push(self.lower_recursive_capture_arg_for_ir(&capture));
                }
            }
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(
                    canonical_func
                        .split("::")
                        .map(ToString::to_string)
                        .collect(),
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
            if method == "append" && args.len() == 1 {
                if let HirExpr::Index {
                    object: index_object,
                    index,
                    ..
                } = object.as_ref()
                {
                    let index_object_ty =
                        crate::resolve_alias_type_for_plain_call(index_object.ty());
                    if let Type::Dict(_, value_ty) = index_object_ty {
                        if matches!(
                            crate::resolve_alias_type_for_plain_call(value_ty.as_ref()),
                            Type::List(_)
                        ) {
                            let Some(lowered_object) = self.lower_stmt_expr_for_ir(index_object)?
                            else {
                                return Ok(None);
                            };
                            let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
                                return Ok(None);
                            };
                            let Some(lowered_arg) = self.lower_stmt_expr_for_ir(&args[0])? else {
                                return Ok(None);
                            };
                            let lowered_index =
                                Self::clone_non_copy_name_expr_for_ir(index, lowered_index);
                            let lowered_arg =
                                Self::clone_non_copy_name_expr_for_ir(&args[0], lowered_arg);
                            let key_arg = Self::build_dict_lookup_key_arg_for_ir(lowered_index);
                            return Ok(Some(crate::RustExpr::Block {
                                stmts: vec![crate::RustStmt::IfLet {
                                    pattern: "Some(__elem)".to_string(),
                                    expr: crate::RustExpr::MethodCall {
                                        receiver: Box::new(lowered_object),
                                        method: "get_mut".to_string(),
                                        args: vec![key_arg],
                                    },
                                    then_body: vec![crate::RustStmt::Expr(
                                        crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Ident(
                                                "__elem".to_string(),
                                            )),
                                            method: "push".to_string(),
                                            args: vec![lowered_arg],
                                        },
                                    )],
                                    else_body: None,
                                }],
                                expr: None,
                            }));
                        }
                    }
                }
            }
            let needs_field_clone_suppression =
                self.method_call_needs_field_clone_suppression(object, method);
            let suppression_prev = self.pending_self_field_clone_suppression;
            if needs_field_clone_suppression {
                self.pending_self_field_clone_suppression += 1;
            }
            let lowered_registry =
                self.try_lower_registry_method_call_expr(object, method, args, expr.ty());
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
            let effective_object_ty = self.effective_method_object_ty(object);
            if method == "append"
                && lowered_args.len() == 1
                && matches!(
                    crate::resolve_alias_type_for_plain_call(&effective_object_ty),
                    Type::List(_)
                )
            {
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_object),
                    method: "push".to_string(),
                    args: lowered_args,
                }));
            }
            if method == "cloned"
                && lowered_args.is_empty()
                && matches!(
                    crate::resolve_alias_type_for_plain_call(&effective_object_ty),
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
            if let Some(method_params) =
                self.resolve_registry_method_params(&effective_object_ty, method)
            {
                let method_receiver_class =
                    match crate::resolve_alias_type_for_plain_call(&effective_object_ty) {
                        Type::Class { name, .. } => Some(name.clone()),
                        _ => None,
                    };
                for (idx, lowered_arg) in lowered_args.iter_mut().enumerate() {
                    if method_receiver_class.as_ref().is_some_and(|class_name| {
                        self.method_param_lowers_to_sifr_int_result(class_name, method, idx)
                    }) {
                        *lowered_arg = self.coerce_result_int_expr_to_sifr_int_value(
                            self.rewrite_stdlib_constant_idents_in_expr(lowered_arg.clone()),
                        );
                        continue;
                    }
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
            let lowered_method = unwrap_compiler_verified_nonempty_pop_result_for_ir(
                &effective_object_ty,
                method,
                args,
                expr.ty(),
                lowered_method,
            );
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
                    Type::List(_) | Type::Bytes => {
                        let copy_slice_elements =
                            match crate::resolve_alias_type_for_plain_call(object.ty()) {
                                Type::Bytes => true,
                                Type::List(element_ty) => {
                                    crate::helpers::is_copy_type_for_codegen(element_ty.as_ref())
                                }
                                _ => false,
                            };
                        Ok(Some(crate::RustExpr::Block {
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
                                                            receiver: Box::new(
                                                                crate::RustExpr::Ident(
                                                                    "_result".to_string(),
                                                                ),
                                                            ),
                                                            method: "push".to_string(),
                                                            args: vec![if copy_slice_elements {
                                                                crate::RustExpr::Deref(Box::new(
                                                                    crate::RustExpr::Ident(
                                                                        "_el".to_string(),
                                                                    ),
                                                                ))
                                                            } else {
                                                                crate::RustExpr::Clone(Box::new(
                                                                    crate::RustExpr::Ident(
                                                                        "_el".to_string(),
                                                                    ),
                                                                ))
                                                            }],
                                                        },
                                                    )],
                                                    else_body: None,
                                                },
                                                crate::RustStmt::AugAssign {
                                                    target: crate::RustExpr::Ident(
                                                        "_i".to_string(),
                                                    ),
                                                    op: "+".to_string(),
                                                    value: crate::RustExpr::Cast {
                                                        expr: Box::new(crate::RustExpr::Ident(
                                                            "_step".to_string(),
                                                        )),
                                                        ty: crate::RustType::Named(
                                                            "usize".to_string(),
                                                        ),
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
                                                            receiver: Box::new(
                                                                crate::RustExpr::Ident(
                                                                    "_v".to_string(),
                                                                ),
                                                            ),
                                                            method: "get".to_string(),
                                                            args: vec![crate::RustExpr::Cast {
                                                                expr: Box::new(
                                                                    crate::RustExpr::Ident(
                                                                        "_i".to_string(),
                                                                    ),
                                                                ),
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
                                                                args: vec![
                                                                    if copy_slice_elements {
                                                                        crate::RustExpr::Deref(Box::new(
                                                                    crate::RustExpr::Ident(
                                                                        "_el".to_string(),
                                                                    ),
                                                                ))
                                                                    } else {
                                                                        crate::RustExpr::Clone(Box::new(
                                                                    crate::RustExpr::Ident(
                                                                        "_el".to_string(),
                                                                    ),
                                                                ))
                                                                    },
                                                                ],
                                                            },
                                                        )],
                                                        else_body: None,
                                                    }],
                                                    else_body: None,
                                                },
                                                crate::RustStmt::AugAssign {
                                                    target: crate::RustExpr::Ident(
                                                        "_i".to_string(),
                                                    ),
                                                    op: "+".to_string(),
                                                    value: crate::RustExpr::Ident(
                                                        "_step".to_string(),
                                                    ),
                                                },
                                            ],
                                        },
                                    ]),
                                },
                            ],
                            expr: Some(Box::new(crate::RustExpr::Ident("_result".to_string()))),
                        }))
                    }
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
            let lowered_start_raw = if let Some(start_expr) = start {
                let Some(start_lowered) = self.lower_stmt_expr_for_ir(start_expr)? else {
                    return Ok(None);
                };
                Some(start_lowered)
            } else {
                None
            };
            let lowered_stop_raw = if let Some(stop_expr) = stop {
                let Some(stop_lowered) = self.lower_stmt_expr_for_ir(stop_expr)? else {
                    return Ok(None);
                };
                Some(stop_lowered)
            } else {
                None
            };
            let normalize_bound_i64 =
                |raw_opt: Option<crate::RustExpr>, default_value: crate::RustExpr| {
                    let Some(raw) = raw_opt else {
                        return default_value;
                    };
                    crate::RustExpr::If {
                        cond: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(raw.clone()),
                            op: "<".to_string(),
                            right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                        }),
                        then_expr: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                crate::RustExpr::BinOp {
                                    left: Box::new(crate::RustExpr::Ident(
                                        "_slice_len_i64".to_string(),
                                    )),
                                    op: "+".to_string(),
                                    right: Box::new(raw.clone()),
                                },
                            ))),
                            method: "max".to_string(),
                            args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                        }),
                        else_expr: Some(Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(raw),
                            method: "min".to_string(),
                            args: vec![crate::RustExpr::Ident("_slice_len_i64".to_string())],
                        })),
                    }
                };

            match crate::resolve_alias_type_for_plain_call(object.ty()) {
                Type::Str => {
                    let start_i64 = normalize_bound_i64(
                        lowered_start_raw,
                        crate::RustExpr::Literal(crate::RustLiteral::Int(0)),
                    );
                    let stop_i64 = normalize_bound_i64(
                        lowered_stop_raw,
                        crate::RustExpr::Ident("_slice_len_i64".to_string()),
                    );
                    let start_usize = crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::Ident("_slice_start_i64".to_string())),
                        ty: crate::RustType::Named("usize".to_string()),
                    };
                    let take_count = crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                crate::RustExpr::BinOp {
                                    left: Box::new(crate::RustExpr::Ident(
                                        "_slice_stop_i64".to_string(),
                                    )),
                                    op: "-".to_string(),
                                    right: Box::new(crate::RustExpr::Ident(
                                        "_slice_start_i64".to_string(),
                                    )),
                                },
                            ))),
                            method: "max".to_string(),
                            args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                        }),
                        ty: crate::RustType::Named("usize".to_string()),
                    };
                    let iter = crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident(
                                    "_slice_src".to_string(),
                                )),
                                method: "chars".to_string(),
                                args: vec![],
                            }),
                            method: "skip".to_string(),
                            args: vec![start_usize],
                        }),
                        method: "take".to_string(),
                        args: vec![take_count],
                    };
                    let slice_expr = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "String".to_string(),
                            "from_iter".to_string(),
                        ])),
                        args: vec![iter],
                    };
                    return Ok(Some(crate::RustExpr::Block {
                        stmts: vec![
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_slice_src".to_string(),
                                ty: None,
                                value: lowered_object,
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_slice_len_i64".to_string(),
                                ty: None,
                                value: crate::RustExpr::Cast {
                                    expr: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Ident(
                                                "_slice_src".to_string(),
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
                                name: "_slice_start_i64".to_string(),
                                ty: None,
                                value: start_i64,
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_slice_stop_i64".to_string(),
                                ty: None,
                                value: stop_i64,
                            },
                        ],
                        expr: Some(Box::new(slice_expr)),
                    }));
                }
                Type::List(_) | Type::Bytes => {
                    let start_i64 = normalize_bound_i64(
                        lowered_start_raw,
                        crate::RustExpr::Literal(crate::RustLiteral::Int(0)),
                    );
                    let stop_i64 = normalize_bound_i64(
                        lowered_stop_raw,
                        crate::RustExpr::Ident("_slice_len_i64".to_string()),
                    );
                    let start_usize = crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::Ident("_slice_start_i64".to_string())),
                        ty: crate::RustType::Named("usize".to_string()),
                    };
                    let take_count = crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                crate::RustExpr::BinOp {
                                    left: Box::new(crate::RustExpr::Ident(
                                        "_slice_stop_i64".to_string(),
                                    )),
                                    op: "-".to_string(),
                                    right: Box::new(crate::RustExpr::Ident(
                                        "_slice_start_i64".to_string(),
                                    )),
                                },
                            ))),
                            method: "max".to_string(),
                            args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                        }),
                        ty: crate::RustType::Named("usize".to_string()),
                    };
                    let iter = crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident(
                                        "_slice_src".to_string(),
                                    )),
                                    method: "iter".to_string(),
                                    args: vec![],
                                }),
                                method: "skip".to_string(),
                                args: vec![start_usize],
                            }),
                            method: "take".to_string(),
                            args: vec![take_count],
                        }),
                        method: "cloned".to_string(),
                        args: vec![],
                    };
                    let slice_expr = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "Vec".to_string(),
                            "from_iter".to_string(),
                        ])),
                        args: vec![iter],
                    };
                    return Ok(Some(crate::RustExpr::Block {
                        stmts: vec![
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_slice_src".to_string(),
                                ty: None,
                                value: lowered_object,
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_slice_len_i64".to_string(),
                                ty: None,
                                value: crate::RustExpr::Cast {
                                    expr: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident(
                                            "_slice_src".to_string(),
                                        )),
                                        method: "len".to_string(),
                                        args: vec![],
                                    }),
                                    ty: crate::RustType::I64,
                                },
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_slice_start_i64".to_string(),
                                ty: None,
                                value: start_i64,
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_slice_stop_i64".to_string(),
                                ty: None,
                                value: stop_i64,
                            },
                        ],
                        expr: Some(Box::new(slice_expr)),
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
                    Type::Dict(_, value_ty) => {
                        let projection_method =
                            crate::helpers::option_projection_method_for_owned_type(
                                value_ty.as_ref(),
                            );
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
                            method: projection_method.to_string(),
                            args: vec![],
                        }
                    }
                    Type::List(element_ty) => {
                        let projection_method =
                            crate::helpers::option_projection_method_for_owned_type(
                                element_ty.as_ref(),
                            );
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident("__v".to_string())),
                                method: "get".to_string(),
                                args: vec![crate::RustExpr::Cast {
                                    expr: Box::new(lowered_index),
                                    ty: crate::RustType::Named("usize".to_string()),
                                }],
                            }),
                            method: projection_method.to_string(),
                            args: vec![],
                        }
                    }
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
                                ty: crate::RustType::Named("u8".to_string()),
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
                Type::Dict(_, value_ty) => {
                    let key_arg = if matches!(index.as_ref(), HirExpr::StringLiteral(_)) {
                        lowered_index
                    } else {
                        crate::RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(lowered_index),
                        }
                    };
                    if index_returns_option {
                        let projection_method =
                            crate::helpers::option_projection_method_for_owned_type(
                                value_ty.as_ref(),
                            );
                        return Ok(Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(lowered_object),
                                method: "get".to_string(),
                                args: vec![key_arg],
                            }),
                            method: projection_method.to_string(),
                            args: vec![],
                        }));
                    }
                    let indexed_expr = crate::RustExpr::Index {
                        expr: Box::new(lowered_object),
                        index: Box::new(key_arg),
                    };
                    return Ok(Some(
                        if crate::helpers::is_copy_type_for_codegen(value_ty.as_ref()) {
                            indexed_expr
                        } else {
                            crate::RustExpr::Clone(Box::new(indexed_expr))
                        },
                    ));
                }
                Type::List(element_ty) => {
                    let list_index = crate::RustExpr::Cast {
                        expr: Box::new(lowered_index),
                        ty: crate::RustType::Named("usize".to_string()),
                    };
                    if index_returns_option {
                        let projection_method =
                            crate::helpers::option_projection_method_for_owned_type(
                                element_ty.as_ref(),
                            );
                        return Ok(Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(lowered_object),
                                method: "get".to_string(),
                                args: vec![list_index],
                            }),
                            method: projection_method.to_string(),
                            args: vec![],
                        }));
                    }
                    let indexed_expr = crate::RustExpr::Index {
                        expr: Box::new(lowered_object),
                        index: Box::new(list_index),
                    };
                    return Ok(Some(
                        if crate::helpers::is_copy_type_for_codegen(element_ty.as_ref()) {
                            indexed_expr
                        } else {
                            crate::RustExpr::Clone(Box::new(indexed_expr))
                        },
                    ));
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
                                    ty: crate::RustType::Named("u8".to_string()),
                                }),
                                is_move: false,
                            }],
                        }));
                    }
                    return Ok(Some(crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::Index {
                            expr: Box::new(lowered_object),
                            index: Box::new(list_index),
                        }),
                        ty: crate::RustType::Named("u8".to_string()),
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
                Type::Class { methods, .. } | Type::Protocol { methods, .. } => {
                    if let Some((_, getitem_ft)) = methods
                        .iter()
                        .find(|(name, ft)| name == "__getitem__" && ft.params.len() == 1)
                    {
                        let key_convention = getitem_ft.params[0].2;
                        let index_arg = if key_convention.is_shared_borrow()
                            || key_convention.is_mut_borrow()
                        {
                            crate::RustExpr::Ref {
                                mutable: key_convention.is_mut_borrow(),
                                expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_index))),
                            }
                        } else {
                            lowered_index
                        };
                        return Ok(Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(lowered_object),
                            method: "__getitem__".to_string(),
                            args: vec![index_arg],
                        }));
                    }
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
                Type::List(_) | Type::Set(_) | Type::Range => crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_collection))),
                    method: "contains".to_string(),
                    args: vec![crate::RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_element))),
                    }],
                },
                Type::Str => crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_collection))),
                    method: "contains".to_string(),
                    args: vec![crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_element))),
                        method: "as_str".to_string(),
                        args: vec![],
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
                if let Some(lowered) = Self::try_lower_collection_truthiness_condition_for_ir(expr)
                {
                    return Ok(Some(lowered));
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
            if !ops.is_empty() && ops.len() == comparators.len() {
                let mut lhs_expr = left.as_ref();
                let mut lowered_chain: Option<crate::RustExpr> = None;
                for (idx, op) in ops.iter().enumerate() {
                    let Some(rhs_expr) = comparators.get(idx) else {
                        unreachable!("compare ops/comparators lengths checked equal");
                    };
                    let lowered_op = match op.as_str() {
                        "==" | "!=" | "<" | "<=" | ">" | ">=" => op.clone(),
                        "is" => "==".to_string(),
                        "is not" => "!=".to_string(),
                        _ => return Ok(None),
                    };
                    let Some(lowered_left) = self.lower_stmt_expr_for_ir(lhs_expr)? else {
                        return Ok(None);
                    };
                    let Some(lowered_right) = self.lower_stmt_expr_for_ir(rhs_expr)? else {
                        return Ok(None);
                    };
                    let lowered_left = if matches!(lhs_expr, HirExpr::Name { name, ty }
                        if (self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name))
                            && ty.ownership() != sifr_type_system::OwnershipKind::Copy)
                    {
                        crate::RustExpr::Clone(Box::new(lowered_left))
                    } else {
                        lowered_left
                    };
                    let lowered_right = if matches!(rhs_expr, HirExpr::Name { name, ty }
                        if (self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name))
                            && ty.ownership() != sifr_type_system::OwnershipKind::Copy)
                    {
                        crate::RustExpr::Clone(Box::new(lowered_right))
                    } else {
                        lowered_right
                    };
                    let left_is_option = crate::helpers::is_option_type(lhs_expr.ty());
                    let right_is_option = crate::helpers::is_option_type(rhs_expr.ty());
                    let left_none_like = matches!(lhs_expr, HirExpr::NoneLiteral)
                        || matches!(
                            crate::resolve_alias_type_for_plain_call(lhs_expr.ty()),
                            Type::None
                        );
                    let right_none_like = matches!(rhs_expr, HirExpr::NoneLiteral)
                        || matches!(
                            crate::resolve_alias_type_for_plain_call(rhs_expr.ty()),
                            Type::None
                        );
                    let left_ty = crate::resolve_alias_type_for_plain_call(lhs_expr.ty());
                    let right_ty = crate::resolve_alias_type_for_plain_call(rhs_expr.ty());
                    let (mut lowered_left, mut lowered_right) =
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
                        };
                    if !left_is_option
                        && !right_is_option
                        && matches!(left_ty, Type::Float)
                        && matches!(right_ty, Type::Int | Type::LiteralInt(_))
                    {
                        lowered_right = crate::RustExpr::Cast {
                            expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_right))),
                            ty: crate::RustType::F64,
                        };
                    } else if !left_is_option
                        && !right_is_option
                        && matches!(right_ty, Type::Float)
                        && matches!(left_ty, Type::Int | Type::LiteralInt(_))
                    {
                        lowered_left = crate::RustExpr::Cast {
                            expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_left))),
                            ty: crate::RustType::F64,
                        };
                    }
                    let lowered_cmp = crate::RustExpr::BinOp {
                        left: Box::new(lowered_left),
                        op: lowered_op,
                        right: Box::new(lowered_right),
                    };
                    lowered_chain = Some(if let Some(existing) = lowered_chain {
                        crate::RustExpr::BinOp {
                            left: Box::new(existing),
                            op: "&&".to_string(),
                            right: Box::new(lowered_cmp),
                        }
                    } else {
                        lowered_cmp
                    });
                    lhs_expr = rhs_expr;
                }
                return Ok(lowered_chain.map(|expr| crate::RustExpr::Paren(Box::new(expr))));
            }
        }
        if let HirExpr::BoolOp { op, values, ty } = expr {
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
            let lower_boolop_operand =
                |this: &mut Self,
                 operand: &HirExpr|
                 -> Result<Option<crate::RustExpr>, crate::CodegenError> {
                    if matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Bool) {
                        this.lower_condition_expr_for_ir(operand)
                    } else {
                        this.lower_stmt_expr_for_ir(operand)
                    }
                };
            let Some(mut acc) = lower_boolop_operand(self, first)? else {
                return Ok(None);
            };
            for value in iter {
                let Some(lowered_value) = lower_boolop_operand(self, value)? else {
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

            if matches!(op.as_str(), "//" | "%")
                && matches!(resolved_left_ty, Type::Int | Type::LiteralInt(_))
                && matches!(resolved_right_ty, Type::Int | Type::LiteralInt(_))
                && is_result_int_division_error_type(resolved_result_ty)
            {
                let method = if op == "//" {
                    "checked_floor_div"
                } else {
                    "checked_floor_mod"
                };
                return Ok(Some(crate::RustExpr::Block {
                    stmts: vec![
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__sifr_floor_left".to_string(),
                            ty: Some(crate::RustType::Named("SifrInt".to_string())),
                            value: self.coerce_expr_to_sifr_int_value(lowered_left),
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__sifr_floor_right".to_string(),
                            ty: Some(crate::RustType::Named("SifrInt".to_string())),
                            value: self.coerce_expr_to_sifr_int_value(lowered_right),
                        },
                    ],
                    expr: Some(Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident(
                                "__sifr_floor_left".to_string(),
                            )),
                            method: method.to_string(),
                            args: vec![crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(crate::RustExpr::Ident(
                                    "__sifr_floor_right".to_string(),
                                )),
                            }],
                        }),
                        method: "ok_or_else".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![],
                            body: Box::new(crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "DivisionError".to_string(),
                                    "new".to_string(),
                                ])),
                                args: vec![crate::RustExpr::Literal(crate::RustLiteral::Str(
                                    "division by zero".to_string(),
                                ))],
                            }),
                            is_move: false,
                        }],
                    })),
                }));
            }

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

            if op == "*"
                && (matches!(resolved_result_ty, Type::List(_))
                    || matches!(resolved_result_ty, Type::Bytes))
            {
                let is_collection_like = |candidate: &Type| {
                    matches!(candidate, Type::List(_)) || matches!(candidate, Type::Bytes)
                };
                let is_count_like =
                    |candidate: &Type| matches!(candidate, Type::Int | Type::LiteralInt(_));
                let (collection_expr, count_expr) = match (
                    (
                        is_collection_like(resolved_left_ty),
                        is_count_like(resolved_right_ty),
                    ),
                    (
                        is_collection_like(resolved_right_ty),
                        is_count_like(resolved_left_ty),
                    ),
                ) {
                    ((true, true), _) => (lowered_left.clone(), lowered_right.clone()),
                    (_, (true, true)) => (lowered_right.clone(), lowered_left.clone()),
                    _ => return Ok(None),
                };
                return Ok(Some(crate::RustExpr::Block {
                    stmts: vec![
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__sifr_repeat_src".to_string(),
                            ty: None,
                            value: crate::RustExpr::Clone(Box::new(crate::RustExpr::Paren(
                                Box::new(collection_expr),
                            ))),
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__sifr_repeat_n".to_string(),
                            ty: None,
                            value: count_expr,
                        },
                    ],
                    expr: Some(Box::new(crate::RustExpr::If {
                        cond: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Ident("__sifr_repeat_n".to_string())),
                            op: "<=".to_string(),
                            right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                        }),
                        then_expr: Box::new(crate::RustExpr::Vec(vec![])),
                        else_expr: Some(Box::new(crate::RustExpr::Block {
                            stmts: vec![
                                crate::RustStmt::Let {
                                    mutable: true,
                                    name: "__sifr_repeat_out".to_string(),
                                    ty: None,
                                    value: crate::RustExpr::Vec(vec![]),
                                },
                                crate::RustStmt::For {
                                    var: "_".to_string(),
                                    iter: crate::RustExpr::Range {
                                        start: Box::new(crate::RustExpr::Literal(
                                            crate::RustLiteral::Int(0),
                                        )),
                                        end: Box::new(crate::RustExpr::Ident(
                                            "__sifr_repeat_n".to_string(),
                                        )),
                                    },
                                    body: vec![crate::RustStmt::Expr(
                                        crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Ident(
                                                "__sifr_repeat_out".to_string(),
                                            )),
                                            method: "extend".to_string(),
                                            args: vec![crate::RustExpr::MethodCall {
                                                receiver: Box::new(crate::RustExpr::MethodCall {
                                                    receiver: Box::new(crate::RustExpr::Paren(
                                                        Box::new(crate::RustExpr::Ident(
                                                            "__sifr_repeat_src".to_string(),
                                                        )),
                                                    )),
                                                    method: "iter".to_string(),
                                                    args: vec![],
                                                }),
                                                method: "cloned".to_string(),
                                                args: vec![],
                                            }],
                                        },
                                    )],
                                },
                            ],
                            expr: Some(Box::new(crate::RustExpr::Ident(
                                "__sifr_repeat_out".to_string(),
                            ))),
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
        if let Some((func, args)) = call_expr_parts(expr) {
            if func == "print" {
                return self.lower_print_call_expr_for_ir(args);
            }
            if let Some(lowered_intrinsic) = self.try_lower_registry_intrinsic_call_expr(func, args)
            {
                return Ok(Some(lowered_intrinsic));
            }
            if let Some(lowered_builtin) =
                self.try_lower_registry_builtin_call_expr(func, args, Some(expr.ty()))
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
            let canonical_func = canonical_plain_call_name_for_ir(func);
            lowered_args = self.adapt_plain_call_args_with_signature_for_ir(
                canonical_func,
                args,
                lowered_args,
            );
            if let Some(captures) = self.nested_fn_captures.get(func).cloned() {
                for capture in captures {
                    lowered_args.push(self.lower_recursive_capture_arg_for_ir(&capture));
                }
            }
            let lowered_func = if canonical_func.contains("::") {
                crate::RustExpr::Path(canonical_func.split("::").map(str::to_string).collect())
            } else {
                crate::RustExpr::Ident(canonical_func.to_string())
            };
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(lowered_func),
                args: lowered_args,
            }));
        }

        match expr {
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

                let lowered =
                    self.try_lower_registry_method_call_expr(object, method, args, expr.ty());

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
                let Some(inner) = args_iter.next() else {
                    unreachable!("Some(_) call must have exactly one argument");
                };
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

    fn ensure_option_box_inner_for_ir(expr: crate::RustExpr) -> crate::RustExpr {
        if matches!(expr, crate::RustExpr::Literal(crate::RustLiteral::None)) {
            return expr;
        }
        if Self::is_some_call_expr_for_ir(&expr) {
            return Self::ensure_some_box_inner_for_ir(expr);
        }
        crate::RustExpr::MethodCall {
            receiver: Box::new(expr),
            method: "map".to_string(),
            args: vec![crate::RustExpr::Closure {
                params: vec![crate::RustParam::Named {
                    name: "__sifr_option_value".to_string(),
                    ty: crate::RustType::Named("_".to_string()),
                }],
                body: Box::new(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "Box".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![crate::RustExpr::Ident("__sifr_option_value".to_string())],
                }),
                is_move: false,
            }],
        }
    }

    pub(crate) fn adapt_plain_call_args_with_signature_for_ir(
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
            let effective_arg_ty = if let HirExpr::Name { name, ty } = hir_arg {
                if self.none_widened_local_bindings.contains(name) {
                    self.local_binding_types
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| ty.clone())
                } else if matches!(
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
                hir_arg.ty().clone()
            };
            let arg_is_option = crate::helpers::is_option_type(&effective_arg_ty);
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
                if !arg_is_option && !matches!(hir_arg, HirExpr::NoneLiteral) {
                    let wrapped_inner = Self::clone_non_copy_name_expr_for_ir(hir_arg, lowered_arg);
                    lowered_arg = if needs_box_inner {
                        Self::ensure_some_box_inner_for_ir(wrapped_inner)
                    } else {
                        crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                            args: vec![wrapped_inner],
                        }
                    };
                } else if needs_box_inner {
                    lowered_arg = Self::ensure_option_box_inner_for_ir(lowered_arg);
                }
            } else if arg_is_option {
                if !crate::helpers::is_copy_type_for_codegen(&effective_arg_ty) {
                    lowered_arg = crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
                        method: "clone".to_string(),
                        args: vec![],
                    };
                }
                lowered_arg = Self::force_unwrap_option_expr_for_ir(
                    lowered_arg,
                    "compiler-verified option argument should be Some",
                );
            }

            if self.function_param_lowers_to_sifr_int(func, idx) {
                let lowered_arg = self.rewrite_stdlib_constant_idents_in_expr(lowered_arg);
                adapted.push(self.coerce_expr_to_sifr_int_value(lowered_arg));
                continue;
            }
            if self.function_param_lowers_to_sifr_int_result(func, idx) {
                let lowered_arg = self.rewrite_stdlib_constant_idents_in_expr(lowered_arg);
                adapted.push(self.coerce_result_int_expr_to_sifr_int_value(lowered_arg));
                continue;
            }

            let param_rust_type = param_ty.rust_type();
            if param_rust_type.starts_with("Box<")
                && !Self::is_box_new_call_expr_for_ir(&lowered_arg)
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
                || matches!(
                    (hir_arg, &lowered_arg),
                    (
                        HirExpr::Name { name, .. },
                        crate::RustExpr::Ident(lowered_name)
                    ) if lowered_name == name
                        && (self.borrowed_params.contains(name)
                            || self.mut_borrowed_params.contains(name))
                );
            let already_mut_borrowed =
                matches!(lowered_arg, crate::RustExpr::Ref { mutable: true, .. })
                    || matches!(
                        (hir_arg, &lowered_arg),
                        (
                            HirExpr::Name { name, .. },
                            crate::RustExpr::Ident(lowered_name)
                        ) if lowered_name == name && self.mut_borrowed_params.contains(name)
                    );

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
        if self.recursive_capture_lowers_to_sifr_int(capture) {
            let rewritten = self.rewrite_stdlib_constant_idents_in_expr(ident);
            return self.coerce_expr_to_sifr_int_value(rewritten);
        }
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
            Type::List(element_ty) => {
                let indexed_expr = crate::RustExpr::Index {
                    expr: Box::new(lowered_object),
                    index: Box::new(crate::RustExpr::Cast {
                        expr: Box::new(lowered_index),
                        ty: crate::RustType::Named("usize".to_string()),
                    }),
                };
                if crate::helpers::is_copy_type_for_codegen(element_ty.as_ref()) {
                    indexed_expr
                } else {
                    crate::RustExpr::Clone(Box::new(indexed_expr))
                }
            }
            Type::Bytes => crate::RustExpr::Cast {
                expr: Box::new(crate::RustExpr::Index {
                    expr: Box::new(lowered_object),
                    index: Box::new(crate::RustExpr::Cast {
                        expr: Box::new(lowered_index),
                        ty: crate::RustType::Named("usize".to_string()),
                    }),
                }),
                ty: crate::RustType::Named("u8".to_string()),
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
        let coerce_return = |this: &mut Self,
                             lowered: crate::RustExpr|
         -> Result<crate::RustExpr, crate::CodegenError> {
            if let Some(target_ty) = return_ty {
                let coerced =
                    this.coerce_local_value_for_target_type_for_ir(target_ty, value, lowered)?;
                if this.current_sifr_int_result_return.get()
                    && is_result_int_division_error_type(target_ty)
                {
                    return Ok(this.coerce_result_int_expr_to_sifr_int_value(coerced));
                }
                return Ok(coerced);
            }
            Ok(lowered)
        };
        if self.current_class_name.is_some()
            && matches!(value, HirExpr::Name { name, .. } if name == "self")
        {
            return Ok(Some(coerce_return(
                self,
                crate::RustExpr::Clone(Box::new(crate::RustExpr::Ident("self".to_string()))),
            )?));
        }

        if let Some(clone_expr) = self.borrowed_return_name_clone_expr_for_ir(value) {
            return Ok(Some(coerce_return(self, clone_expr)?));
        }

        if let Some(target_ty) = return_ty {
            if matches!(
                crate::resolve_alias_type_for_plain_call(target_ty),
                Type::Iterator(_) | Type::Iterable(_)
            ) {
                if let Some(lowered_iter_return) =
                    self.lower_escaping_iter_return_expr_for_ir(value)?
                {
                    return Ok(Some(coerce_return(self, lowered_iter_return)?));
                }
            }

            if matches!(
                crate::resolve_alias_type_for_plain_call(target_ty),
                Type::Iterator(_)
            ) && !matches!(
                crate::resolve_alias_type_for_plain_call(value.ty()),
                Type::Iterator(_)
            ) && crate::resolve_alias_type_for_plain_call(value.ty())
                .iterable_element_type()
                .is_some()
            {
                if let Some(lowered_iter_source) =
                    self.lower_iter_source_expr_for_ir_with_mode(value, true, None, None)?
                {
                    return Ok(Some(coerce_return(self, lowered_iter_source)?));
                }
            }
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
            return Ok(Some(coerce_return(self, lowered_leaf)?));
        }
        if let Some(lowered_expr) = self.lower_stmt_expr_for_ir(value)? {
            return Ok(Some(coerce_return(
                self,
                self.rewrite_stdlib_constant_idents_in_expr(lowered_expr),
            )?));
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
                crate::try_lower_simple_stmt_with_scope_result_and_bindings(
                    stmt,
                    &self.mutated_vars,
                    &self.borrowed_params,
                    &self.local_binding_types,
                    &scope_ctx,
                )?
            } else {
                None
            };

            let should_bypass_simple_lowering = matches!(
                stmt,
                HirStmt::NestedFunction { .. } | HirStmt::Assign { .. }
            ) || matches!(stmt, HirStmt::Let { ty, .. } if self.type_contains_generic_class(ty));
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
                let effective_ty = if type_contains_any_or_unknown(ty) {
                    self.local_binding_types
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| ty.clone())
                } else {
                    ty.clone()
                };
                let is_generic_class = matches!(
                    &effective_ty,
                    Type::Class {
                        name: class_name,
                        ..
                    } if self.generic_classes.contains(class_name)
                );
                let lowered_value =
                    if effective_ty.ownership() == sifr_type_system::OwnershipKind::Move {
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
                    self.coerce_local_value_for_target_type_for_ir(&effective_ty, value, lowered)?
                };
                let lowered_ty = if name == "_"
                    || is_generic_class
                    || should_omit_local_type_annotation(&effective_ty, value)
                {
                    None
                } else if matches!(
                    crate::resolve_alias_type_for_plain_call(&effective_ty),
                    Type::Int | Type::LiteralInt(_)
                ) && self.is_sifr_int_expr(&lowered_value)
                {
                    Some(crate::RustType::Named("SifrInt".to_string()))
                } else if is_result_int_division_error_type(&effective_ty)
                    && self.is_sifr_int_result_expr(&lowered_value)
                {
                    Some(result_int_to_sifr_int_rust_type(&effective_ty))
                } else {
                    Some(self.rust_ir_type_with_generics(&effective_ty))
                };
                (
                    vec![RustStmt::Let {
                        mutable: self.mutated_vars.contains(name)
                            || should_force_mutable_binding(&effective_ty),
                        name: name.clone(),
                        ty: lowered_ty,
                        value: lowered_value,
                    }],
                    true,
                )
            } else if let HirStmt::Assign { name, value } = stmt {
                let Some(lowered_value) = self.lower_rendered_expr_for_ir(value)? else {
                    return Ok(None);
                };
                let lowered_value = if let Some(target_ty) =
                    self.local_binding_types.get(name).cloned()
                {
                    let mut lowered = self.coerce_local_value_for_target_type_for_ir(
                        &target_ty,
                        value,
                        lowered_value,
                    )?;
                    if !crate::helpers::is_option_type(&target_ty)
                        && crate::helpers::is_option_type(value.ty())
                    {
                        let fallback = if crate::helpers::is_copy_type_for_codegen(&target_ty) {
                            crate::RustExpr::Ident(name.clone())
                        } else {
                            crate::RustExpr::Clone(Box::new(crate::RustExpr::Ident(name.clone())))
                        };
                        lowered = crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                            method: "unwrap_or".to_string(),
                            args: vec![fallback],
                        };
                    }
                    lowered
                } else {
                    lowered_value
                };
                (
                    vec![RustStmt::Assign {
                        target: crate::RustExpr::Ident(name.clone()),
                        value: lowered_value,
                    }],
                    true,
                )
            } else if let HirStmt::AugAssign { name, op, value } = stmt {
                let value_ty = Self::resolve_alias_type_for_loop_iter(value.ty());
                if op == "+=" {
                    match value_ty {
                        Type::Str => {
                            let arg_expr = if let HirExpr::StringLiteral(val) = value {
                                crate::RustExpr::Ident(format!("{val:?}"))
                            } else {
                                let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                                    return Ok(None);
                                };
                                crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                        value_expr,
                                    ))),
                                    method: "as_str".to_string(),
                                    args: vec![],
                                }
                            };
                            (
                                vec![crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                                    method: "push_str".to_string(),
                                    args: vec![arg_expr],
                                })],
                                true,
                            )
                        }
                        Type::List(_) => {
                            let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                                return Ok(None);
                            };
                            (
                                vec![crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                                    method: "extend".to_string(),
                                    args: vec![value_expr],
                                })],
                                true,
                            )
                        }
                        _ => {
                            let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
                                return Ok(None);
                            };
                            (
                                vec![RustStmt::AugAssign {
                                    target: crate::RustExpr::Ident(name.clone()),
                                    op: "+".to_string(),
                                    value: lowered_value,
                                }],
                                true,
                            )
                        }
                    }
                } else if op == "**=" {
                    let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                        return Ok(None);
                    };
                    (
                        vec![crate::RustStmt::Assign {
                            target: crate::RustExpr::Ident(name.clone()),
                            value: crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                    crate::RustExpr::Ident(name.clone()),
                                ))),
                                method: "pow".to_string(),
                                args: vec![crate::RustExpr::Cast {
                                    expr: Box::new(value_expr),
                                    ty: crate::RustType::Named("u32".to_string()),
                                }],
                            },
                        }],
                        true,
                    )
                } else {
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
                }
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
                let Type::List(elem) = Self::resolve_alias_type_for_loop_iter(inner) else {
                    return Ok(None);
                };
                let Some(lowered_stmt) = self
                    .lower_structured_nested_list_subscript_assign_stmt_for_ir(
                        object,
                        outer_index,
                        inner_index,
                        value,
                        elem,
                    )?
                else {
                    return Ok(None);
                };
                (vec![lowered_stmt], true)
            } else if let HirStmt::AttributeNestedSubscriptAssign {
                object,
                field,
                outer_index,
                inner_index,
                value,
                field_ty,
            } = stmt
            {
                let Some(lowered_stmt) = self
                    .lower_structured_attribute_nested_list_subscript_assign_stmt_for_ir(
                        object,
                        field,
                        outer_index,
                        inner_index,
                        value,
                        field_ty,
                    )?
                else {
                    return Ok(None);
                };
                (vec![lowered_stmt], true)
            } else if let HirStmt::SubscriptAugAssign {
                object,
                index,
                op,
                value,
                object_ty,
            } = stmt
            {
                let Some(lowered_stmt) = self
                    .lower_subscript_augassign_stmt_for_ir(object, index, op, value, object_ty)?
                else {
                    return Ok(None);
                };
                (vec![lowered_stmt], true)
            } else if let HirStmt::Delete { object, index } = stmt {
                let Some(lowered_stmt) = self.lower_delete_stmt_for_ir(object, index)? else {
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
                let key_is_non_copy_name = matches!(index, HirExpr::Name { .. })
                    && matches!(
                        crate::resolve_alias_type_for_plain_call(index.ty()),
                        Type::Str | Type::LiteralStr(_)
                    );

                let Some(mut index_expr) = self.lower_rendered_expr_for_ir(index)? else {
                    return Ok(None);
                };
                if key_needs_clone || key_is_non_copy_name {
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
                field_ty,
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
                let value_expr = self.adapt_field_assign_value_for_recursive_storage(
                    object,
                    field,
                    field_ty,
                    value_expr,
                    value.ty(),
                );
                (
                    vec![RustStmt::Assign {
                        target,
                        value: value_expr,
                    }],
                    true,
                )
            } else if let HirStmt::NestedFieldAssign {
                object,
                field,
                field_ty,
                nested_field,
                nested_field_ty,
                value,
            } = stmt
            {
                let Some(lowered_stmt) = self.lower_nested_field_assign_stmt_for_ir(
                    object,
                    field,
                    field_ty,
                    nested_field,
                    nested_field_ty,
                    value,
                )?
                else {
                    return Ok(None);
                };
                (vec![lowered_stmt], true)
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
            } else if let HirStmt::Yield { value } = stmt {
                let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
                    return Ok(None);
                };
                (
                    vec![RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident("_yields".to_string())),
                        method: "push".to_string(),
                        args: vec![lowered_value],
                    })],
                    true,
                )
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
                target_ty,
                iter,
                body,
                else_body,
                ..
            } = stmt
            {
                if else_body.is_some() {
                    return Ok(None);
                }
                let Some(lowered_iter) = self.try_lower_for_iter_expr_for_ir(iter, target_ty)?
                else {
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
            } else if let HirStmt::AsyncWith { kind, target, body } = stmt {
                let Some(lowered_async_with) =
                    self.try_lower_async_with_stmt_for_ir(kind, target.as_deref(), body)?
                else {
                    return Ok(None);
                };
                (vec![lowered_async_with], true)
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
        if let HirExpr::BoolOp { op, values, .. } = condition {
            let lowered_op = match op.as_str() {
                "and" => "&&",
                "or" => "||",
                _ => return Ok(None),
            };
            let mut lowered_values = Vec::with_capacity(values.len());
            for value in values {
                let Some(lowered_value) = self.lower_condition_expr_for_ir(value)? else {
                    return Ok(None);
                };
                lowered_values.push(lowered_value);
            }
            let mut lowered_values_iter = lowered_values.into_iter();
            let Some(mut iter_expr) = lowered_values_iter.next() else {
                return Ok(None);
            };
            for rhs in lowered_values_iter {
                iter_expr = crate::RustExpr::BinOp {
                    left: Box::new(iter_expr),
                    op: lowered_op.to_string(),
                    right: Box::new(rhs),
                };
            }
            return Ok(Some(iter_expr));
        }
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
        if let HirExpr::UnaryOp { op, operand, .. } = condition {
            if op == "not" && Self::option_inner_type_for_ir(operand.ty()).is_some() {
                let Some(lowered_option_expr) = self.lower_stmt_expr_for_ir(operand)? else {
                    return Ok(None);
                };
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_option_expr),
                    method: "is_none".to_string(),
                    args: vec![],
                }));
            }
        }
        if let Some(option_inner_ty) = Self::option_inner_type_for_ir(condition.ty()) {
            let Some(lowered_option_expr) = self.lower_stmt_expr_for_ir(condition)? else {
                return Ok(None);
            };
            if matches!(
                crate::resolve_alias_type_for_plain_call(option_inner_ty),
                Type::Bool | Type::LiteralBool(_)
            ) {
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_option_expr),
                    method: "is_some_and".to_string(),
                    args: vec![crate::RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "__v".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(crate::RustExpr::Ident("__v".to_string())),
                        is_move: false,
                    }],
                }));
            }
            if matches!(
                crate::resolve_alias_type_for_plain_call(option_inner_ty),
                Type::Int | Type::LiteralInt(_) | Type::BigInt | Type::Float
            ) {
                let Some(zero_literal) =
                    Self::zero_literal_for_numeric_truthiness_type_for_ir(option_inner_ty)
                else {
                    unreachable!("numeric Option truthiness guard must have a zero literal");
                };
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_option_expr),
                    method: "is_some_and".to_string(),
                    args: vec![crate::RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "__v".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Ident("__v".to_string())),
                            op: "!=".to_string(),
                            right: Box::new(zero_literal),
                        }),
                        is_move: false,
                    }],
                }));
            }
            return Ok(Some(crate::RustExpr::MethodCall {
                receiver: Box::new(lowered_option_expr),
                method: "is_some".to_string(),
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

    fn option_binding_pattern_for_ir(&self, option_var: &str) -> String {
        let is_borrowed_param = self.borrowed_params.contains(option_var)
            || self.mut_borrowed_params.contains(option_var);
        if is_borrowed_param {
            format!("Some({option_var})")
        } else {
            format!("Some(mut {option_var})")
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
        match condition {
            HirExpr::Name { name, ty } => Some(crate::RustExpr::BinOp {
                left: Box::new(crate::RustExpr::Ident(name.clone())),
                op: "!=".to_string(),
                right: Box::new(Self::zero_literal_for_numeric_truthiness_type_for_ir(ty)?),
            }),
            HirExpr::MethodCall {
                object,
                method,
                args,
                ty,
            } if method == "len" && args.is_empty() => {
                let HirExpr::Name { name, .. } = object.as_ref() else {
                    return None;
                };
                let lhs = crate::RustExpr::Cast {
                    expr: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                        method: "len".to_string(),
                        args: vec![],
                    }),
                    ty: crate::RustType::I64,
                };
                Some(crate::RustExpr::BinOp {
                    left: Box::new(lhs),
                    op: "!=".to_string(),
                    right: Box::new(Self::zero_literal_for_numeric_truthiness_type_for_ir(ty)?),
                })
            }
            HirExpr::UnaryOp { op, operand, .. } if op == "not" => match operand.as_ref() {
                HirExpr::Name { name, ty } => Some(crate::RustExpr::BinOp {
                    left: Box::new(crate::RustExpr::Ident(name.clone())),
                    op: "==".to_string(),
                    right: Box::new(Self::zero_literal_for_numeric_truthiness_type_for_ir(ty)?),
                }),
                HirExpr::MethodCall {
                    object,
                    method,
                    args,
                    ty,
                } if method == "len" && args.is_empty() => {
                    let HirExpr::Name { name, .. } = object.as_ref() else {
                        return None;
                    };
                    let lhs = crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                            method: "len".to_string(),
                            args: vec![],
                        }),
                        ty: crate::RustType::I64,
                    };
                    Some(crate::RustExpr::BinOp {
                        left: Box::new(lhs),
                        op: "==".to_string(),
                        right: Box::new(Self::zero_literal_for_numeric_truthiness_type_for_ir(ty)?),
                    })
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn zero_literal_for_numeric_truthiness_type_for_ir(ty: &Type) -> Option<crate::RustExpr> {
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

    fn try_lower_for_iter_expr_for_ir(
        &mut self,
        iter: &HirExpr,
        target_ty: &Type,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let HirExpr::IteratorCall { op, args, .. } = iter {
            if *op == HirIteratorOp::Iter && args.len() == 1 {
                return self.lower_structural_iter_source_expr_for_ir(&args[0], Some(target_ty));
            }
            if *op == HirIteratorOp::Enumerate && args.len() == 1 {
                return self.lower_enumerate_for_iter_expr_for_ir(&args[0], Some(target_ty));
            }
        }
        if let HirExpr::Call { func, args, .. } = iter {
            if func == "iter" && args.len() == 1 {
                return self.lower_structural_iter_source_expr_for_ir(&args[0], Some(target_ty));
            }
            if func == "enumerate" && args.len() == 1 {
                return self.lower_enumerate_for_iter_expr_for_ir(&args[0], Some(target_ty));
            }
        }
        self.lower_structural_iter_source_expr_for_ir(iter, Some(target_ty))
    }

    fn lower_enumerate_iter_chain_for_ir(iter_source: crate::RustExpr) -> crate::RustExpr {
        crate::RustExpr::MethodCall {
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
        }
    }

    fn lower_enumerate_for_iter_expr_for_ir(
        &mut self,
        source: &HirExpr,
        element_type_hint: Option<&Type>,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let Some(iter_source) =
            self.lower_structural_iter_source_expr_for_ir(source, element_type_hint)?
        else {
            return Ok(None);
        };
        Ok(Some(Self::lower_enumerate_iter_chain_for_ir(iter_source)))
    }

    fn lower_structural_iter_source_expr_for_ir(
        &mut self,
        source: &HirExpr,
        element_type_hint: Option<&Type>,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        self.lower_iter_source_expr_for_ir_with_mode(source, false, element_type_hint, None)
    }

    fn lower_iter_source_expr_for_ir(
        &mut self,
        source: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        self.lower_iter_source_expr_for_ir_with_mode(source, true, None, None)
    }

    fn lower_escaping_iter_return_expr_for_ir(
        &mut self,
        value: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let source = match value {
            HirExpr::IteratorCall { op, args, .. }
                if *op == HirIteratorOp::Iter && args.len() == 1 =>
            {
                &args[0]
            }
            HirExpr::Call { func, args, .. } if func == "iter" && args.len() == 1 => &args[0],
            _ => return Ok(None),
        };

        let can_consume_owned_source = match source {
            HirExpr::Name { name, .. } => {
                !self.borrowed_params.contains(name) && !self.mut_borrowed_params.contains(name)
            }
            HirExpr::FieldAccess { .. } | HirExpr::MethodCall { .. } => true,
            _ => matches!(
                crate::helpers::classify_value_category(source),
                crate::helpers::ValueCategory::Temporary
            ),
        };

        if !can_consume_owned_source {
            return Ok(None);
        }

        self.lower_iter_source_expr_for_ir_with_mode(
            source,
            true,
            None,
            Some(crate::helpers::SourceAccessMode::Consume),
        )
    }

    fn class_method_signature_for_iter_for_ir<'a>(
        methods: &'a [(String, sifr_type_system::FunctionType)],
        method_name: &str,
    ) -> Option<&'a sifr_type_system::FunctionType> {
        methods.iter().find_map(
            |(name, ft)| {
                if name == method_name {
                    Some(ft)
                } else {
                    None
                }
            },
        )
    }

    fn class_has_next_for_iter_for_ir(
        methods: &[(String, sifr_type_system::FunctionType)],
    ) -> bool {
        Self::class_method_signature_for_iter_for_ir(methods, "__next__").is_some_and(|next_ft| {
            next_ft.params.is_empty()
                && matches!(next_ft.return_type.as_ref().resolve_alias(), Type::Union(members) if {
                    let has_none = members
                        .iter()
                        .any(|member| matches!(member.resolve_alias(), Type::None));
                    let non_none = members
                        .iter()
                        .filter(|member| !matches!(member.resolve_alias(), Type::None))
                        .count();
                    has_none && non_none == 1
                })
        })
    }

    fn class_next_iter_expr_for_ir(source_expr: crate::RustExpr) -> crate::RustExpr {
        let state_name = "__sifr_for_iter_state".to_string();
        crate::RustExpr::FnCall {
            func: Box::new(crate::RustExpr::Path(vec![
                "std".to_string(),
                "iter".to_string(),
                "from_fn".to_string(),
            ])),
            args: vec![crate::RustExpr::Block {
                stmts: vec![crate::RustStmt::Let {
                    mutable: true,
                    name: state_name.clone(),
                    ty: None,
                    value: source_expr,
                }],
                expr: Some(Box::new(crate::RustExpr::Closure {
                    params: vec![],
                    body: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(state_name)),
                        method: "__next__".to_string(),
                        args: vec![],
                    }),
                    is_move: true,
                })),
            }],
        }
    }

    fn apply_copy_clone_yield_mode_for_ir(
        iter_expr: crate::RustExpr,
        yield_mode: crate::helpers::YieldMode,
    ) -> crate::RustExpr {
        match yield_mode {
            crate::helpers::YieldMode::Copy => crate::RustExpr::MethodCall {
                receiver: Box::new(iter_expr),
                method: "copied".to_string(),
                args: vec![],
            },
            crate::helpers::YieldMode::Clone => crate::RustExpr::MethodCall {
                receiver: Box::new(iter_expr),
                method: "cloned".to_string(),
                args: vec![],
            },
            crate::helpers::YieldMode::Move | crate::helpers::YieldMode::Borrow => iter_expr,
        }
    }

    fn wrap_iterator_expr_for_mode_for_ir(
        iterator_expr: crate::RustExpr,
        prefer_boxed_iterator: bool,
    ) -> crate::RustExpr {
        if prefer_boxed_iterator {
            crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec![
                    "Box".to_string(),
                    "new".to_string(),
                ])),
                args: vec![iterator_expr],
            }
        } else {
            iterator_expr
        }
    }

    fn lower_homogeneous_tuple_iter_expr(
        lowered_source: crate::RustExpr,
        tuple_len: usize,
        source_access_mode: crate::helpers::SourceAccessMode,
        yield_mode: crate::helpers::YieldMode,
    ) -> crate::RustExpr {
        let tuple_binding = "__sifr_tuple_iter_src".to_string();
        let bound_value = match source_access_mode {
            crate::helpers::SourceAccessMode::Preserve => crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_source))),
                method: "clone".to_string(),
                args: vec![],
            },
            crate::helpers::SourceAccessMode::Consume => lowered_source,
        };
        let tuple_items = (0..tuple_len)
            .map(|index| {
                let field_expr = crate::RustExpr::Field {
                    expr: Box::new(crate::RustExpr::Ident(tuple_binding.clone())),
                    field: index.to_string(),
                };
                match yield_mode {
                    crate::helpers::YieldMode::Copy | crate::helpers::YieldMode::Move => field_expr,
                    crate::helpers::YieldMode::Clone | crate::helpers::YieldMode::Borrow => {
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(field_expr),
                            method: "clone".to_string(),
                            args: vec![],
                        }
                    }
                }
            })
            .collect();
        crate::RustExpr::Block {
            stmts: vec![crate::RustStmt::Let {
                mutable: false,
                name: tuple_binding,
                ty: None,
                value: bound_value,
            }],
            expr: Some(Box::new(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Vec(tuple_items)),
                method: "into_iter".to_string(),
                args: vec![],
            })),
        }
    }

    fn lower_iter_source_expr_for_ir_with_mode(
        &mut self,
        source: &HirExpr,
        prefer_boxed_iterator: bool,
        element_type_hint: Option<&Type>,
        source_access_mode_override: Option<crate::helpers::SourceAccessMode>,
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
        let mut plan =
            crate::helpers::plan_iterator_ownership_with_element_hint(source, element_type_hint);
        if let Some(source_access_mode) = source_access_mode_override {
            plan.source_access_mode = source_access_mode;
            if matches!(
                source_access_mode,
                crate::helpers::SourceAccessMode::Consume
            ) {
                plan.yield_mode = crate::helpers::YieldMode::Move;
            }
        }

        if matches!(source_ty, Type::Iterator(_))
            || matches!(source, HirExpr::GeneratorExpr { .. })
            || self.is_generator_call(source)
            || Self::is_iterator_like_expr_for_ir(&lowered_source)
        {
            return Ok(Some(lowered_source));
        }

        if let Type::Class { name, methods, .. } = source_ty {
            let class_source = match plan.source_access_mode {
                crate::helpers::SourceAccessMode::Preserve => crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_source.clone()))),
                    method: "clone".to_string(),
                    args: vec![],
                },
                crate::helpers::SourceAccessMode::Consume => lowered_source.clone(),
            };
            if let Some(iter_ft) = Self::class_method_signature_for_iter_for_ir(methods, "__iter__")
            {
                if iter_ft.params.is_empty() {
                    let iter_call = crate::RustExpr::MethodCall {
                        receiver: Box::new(class_source.clone()),
                        method: "__iter__".to_string(),
                        args: vec![],
                    };
                    if matches!(
                        iter_ft.return_type.as_ref().resolve_alias(),
                        Type::Class { name: ret_name, .. } if ret_name == name
                    ) && Self::class_has_next_for_iter_for_ir(methods)
                    {
                        return Ok(Some(Self::class_next_iter_expr_for_ir(iter_call)));
                    }
                    if let Type::Class {
                        methods: ret_methods,
                        ..
                    } = iter_ft.return_type.as_ref().resolve_alias()
                    {
                        if Self::class_has_next_for_iter_for_ir(ret_methods) {
                            return Ok(Some(Self::class_next_iter_expr_for_ir(iter_call)));
                        }
                    }
                    return Ok(Some(iter_call));
                }
            }
            if Self::class_has_next_for_iter_for_ir(methods) {
                return Ok(Some(Self::class_next_iter_expr_for_ir(class_source)));
            }
            return Ok(Some(lowered_source));
        }

        let iterator_expr = match source_ty {
            Type::List(_) | Type::Set(_) | Type::Iterable(_) => match plan.source_access_mode {
                crate::helpers::SourceAccessMode::Consume => crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_source),
                    method: "into_iter".to_string(),
                    args: vec![],
                },
                crate::helpers::SourceAccessMode::Preserve => {
                    Self::apply_copy_clone_yield_mode_for_ir(
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(lowered_source),
                            method: "iter".to_string(),
                            args: vec![],
                        },
                        plan.yield_mode,
                    )
                }
            },
            Type::Bytes => match plan.source_access_mode {
                crate::helpers::SourceAccessMode::Consume => crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(lowered_source),
                        method: "into_iter".to_string(),
                        args: vec![],
                    }),
                    method: "map".to_string(),
                    args: vec![crate::RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "__byte".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(crate::RustExpr::Cast {
                            expr: Box::new(crate::RustExpr::Ident("__byte".to_string())),
                            ty: crate::RustType::Named("u8".to_string()),
                        }),
                        is_move: false,
                    }],
                },
                crate::helpers::SourceAccessMode::Preserve => crate::RustExpr::MethodCall {
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
                            expr: Box::new(crate::RustExpr::Deref(Box::new(
                                crate::RustExpr::Ident("__byte".to_string()),
                            ))),
                            ty: crate::RustType::Named("u8".to_string()),
                        }),
                        is_move: false,
                    }],
                },
            },
            Type::Dict(_, _) => match plan.source_access_mode {
                crate::helpers::SourceAccessMode::Consume => crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_source),
                    method: "into_keys".to_string(),
                    args: vec![],
                },
                crate::helpers::SourceAccessMode::Preserve => {
                    Self::apply_copy_clone_yield_mode_for_ir(
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(lowered_source),
                            method: "keys".to_string(),
                            args: vec![],
                        },
                        plan.yield_mode,
                    )
                }
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
            Type::Range => lowered_source,
            Type::Tuple(elems)
                if !elems.is_empty() && elems.iter().all(|elem| elem == &elems[0]) =>
            {
                Self::lower_homogeneous_tuple_iter_expr(
                    lowered_source,
                    elems.len(),
                    plan.source_access_mode,
                    plan.yield_mode,
                )
            }
            _ => return Ok(Some(lowered_source)),
        };
        Ok(Some(Self::wrap_iterator_expr_for_mode_for_ir(
            iterator_expr,
            prefer_boxed_iterator,
        )))
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
                    "into_iter"
                        | "into_keys"
                        | "map"
                        | "filter"
                        | "filter_map"
                        | "zip"
                        | "chain"
                        | "enumerate"
                        | "copied"
                        | "cloned"
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

    fn try_lower_async_with_stmt_for_ir(
        &mut self,
        kind: &sifr_hir::HirAsyncWithKind,
        target: Option<&str>,
        body: &[HirStmt],
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        if let sifr_hir::HirAsyncWithKind::TaskTimeout { duration } = kind {
            let Some(_) = self.lower_rendered_expr_for_ir(duration)? else {
                return Ok(None);
            };
        }
        let Some(mut lowered_body) = self.try_lower_stmt_block_for_ir(body)? else {
            return Ok(None);
        };
        if let Some(target) = target {
            lowered_body.insert(
                0,
                crate::RustStmt::Let {
                    mutable: false,
                    name: target.to_string(),
                    ty: None,
                    value: crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "__SifrTaskScope".to_string(),
                            "new".to_string(),
                        ])),
                        args: vec![],
                    },
                },
            );
        }
        Ok(Some(RustStmt::Block(lowered_body)))
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
        let effective_name_ty = |operand: &HirExpr, emitter: &Self| -> Option<Type> {
            let HirExpr::Name { name, ty } = operand else {
                return None;
            };
            if matches!(
                crate::resolve_alias_type_for_plain_call(ty),
                Type::Any | Type::Unknown
            ) {
                if let Some(bound_ty) = emitter.local_binding_types.get(name) {
                    return Some(bound_ty.clone());
                }
            }
            Some(ty.clone())
        };

        let lower_operand =
            |operand: &HirExpr, emitter: &Self| -> Option<(crate::RustExpr, bool, Type)> {
                let HirExpr::Name { name, .. } = operand else {
                    return None;
                };
                let borrowed = emitter.borrowed_params.contains(name)
                    || emitter.mut_borrowed_params.contains(name);
                let effective_ty = effective_name_ty(operand, emitter)?;
                let ident = crate::RustExpr::Ident(name.clone());
                let lowered = if borrowed {
                    crate::RustExpr::Deref(Box::new(ident))
                } else {
                    ident
                };
                Some((lowered, borrowed, effective_ty))
            };

        let (mut lowered_left, left_borrowed, left_ty) = lower_operand(left, self)?;
        let (mut lowered_right, right_borrowed, right_ty) = lower_operand(rhs, self)?;
        if !left_borrowed && !right_borrowed {
            return None;
        }
        let left_is_option = crate::helpers::is_option_type(&left_ty);
        let right_is_option = crate::helpers::is_option_type(&right_ty);
        let left_none_like = matches!(
            crate::resolve_alias_type_for_plain_call(&left_ty),
            Type::None
        );
        let right_none_like = matches!(
            crate::resolve_alias_type_for_plain_call(&right_ty),
            Type::None
        );

        if left_is_option && !right_is_option && !right_none_like {
            if right_borrowed
                && crate::resolve_alias_type_for_plain_call(&right_ty).ownership()
                    != sifr_type_system::OwnershipKind::Copy
            {
                lowered_right = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_right))),
                    method: "clone".to_string(),
                    args: vec![],
                };
            }
            lowered_right = crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                args: vec![lowered_right],
            };
        } else if !left_is_option && right_is_option && !left_none_like {
            if left_borrowed
                && crate::resolve_alias_type_for_plain_call(&left_ty).ownership()
                    != sifr_type_system::OwnershipKind::Copy
            {
                lowered_left = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_left))),
                    method: "clone".to_string(),
                    args: vec![],
                };
            }
            lowered_left = crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                args: vec![lowered_left],
            };
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
            HirExpr::Call { args, .. } | HirExpr::IteratorCall { args, .. } => args
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
            if let Some(option_vars) = self.detect_or_is_none_vars_with_bindings_for_ir(condition) {
                let pattern = format!(
                    "({})",
                    option_vars
                        .iter()
                        .map(|option_var| self.option_binding_pattern_for_ir(option_var))
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
            if let Some(option_var) = crate::helpers::detect_is_none_var(condition)
                .or_else(|| crate::helpers::detect_not_option_truthiness(condition))
            {
                return Ok(Some(RustStmt::LetElse {
                    pattern: self.option_binding_pattern_for_ir(&option_var),
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
                        .map(|option_var| self.option_binding_pattern_for_ir(option_var))
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
                pattern: self.option_binding_pattern_for_ir(&option_var),
                expr: self.option_binding_value_expr_for_ir(&option_var),
                then_body: lowered_then_body,
                else_body: nested_else,
            }));
        }

        if let Some(option_vars) = crate::helpers::detect_and_not_none_vars(condition) {
            let mut chain_then = lowered_then_body;
            for option_var in option_vars.iter().rev() {
                chain_then = vec![RustStmt::IfLet {
                    pattern: self.option_binding_pattern_for_ir(option_var),
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
                pattern: self.option_binding_pattern_for_ir(&option_var),
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
                    pattern: self.option_binding_pattern_for_ir(&option_var),
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

    fn detect_or_is_none_vars_with_bindings_for_ir(&self, expr: &HirExpr) -> Option<Vec<String>> {
        let HirExpr::BoolOp { op, values, .. } = expr else {
            return crate::helpers::detect_or_is_none_vars(expr);
        };
        if op != "or" {
            return crate::helpers::detect_or_is_none_vars(expr);
        }
        let mut vars = Vec::new();
        for value in values {
            let HirExpr::Compare {
                left,
                ops,
                comparators,
                ..
            } = value
            else {
                return None;
            };
            if ops.len() != 1
                || !(ops[0] == "is" || ops[0] == "==")
                || !matches!(comparators[0], HirExpr::NoneLiteral)
            {
                return None;
            }
            let HirExpr::Name { name, ty } = left.as_ref() else {
                return None;
            };
            let option_like = crate::helpers::is_option_type(ty)
                || self
                    .local_binding_types
                    .get(name)
                    .is_some_and(crate::helpers::is_option_type);
            if !option_like {
                return None;
            }
            vars.push(name.clone());
        }
        if vars.len() >= 2 {
            Some(vars)
        } else {
            None
        }
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
            target_ty,
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
        let lowered_iter = self.try_lower_for_iter_expr_for_ir(iter, target_ty)?;
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
            field_ty,
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
        let value_expr = self.adapt_field_assign_value_for_recursive_storage(
            object,
            field,
            field_ty,
            value_expr,
            value.ty(),
        );
        let lowered = crate::RustStmt::Assign {
            target,
            value: value_expr,
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    fn lower_nested_field_assign_stmt_for_ir(
        &mut self,
        object: &str,
        field: &str,
        field_ty: &Type,
        nested_field: &str,
        nested_field_ty: &Type,
        value: &HirExpr,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(None);
        };
        let value_expr = self.adapt_field_assign_value_for_recursive_storage(
            object,
            nested_field,
            nested_field_ty,
            value_expr,
            value.ty(),
        );
        let outer_target = crate::RustExpr::Field {
            expr: Box::new(Self::object_name_expr_for_ir(object)),
            field: field.to_string(),
        };
        if crate::helpers::is_option_type(field_ty) {
            return Ok(Some(crate::RustStmt::IfLet {
                pattern: "Some(__nested_obj)".to_string(),
                expr: crate::RustExpr::MethodCall {
                    receiver: Box::new(outer_target),
                    method: "as_mut".to_string(),
                    args: vec![],
                },
                then_body: vec![RustStmt::Assign {
                    target: crate::RustExpr::Field {
                        expr: Box::new(crate::RustExpr::Ident("__nested_obj".to_string())),
                        field: nested_field.to_string(),
                    },
                    value: value_expr,
                }],
                else_body: None,
            }));
        }
        Ok(Some(RustStmt::Assign {
            target: crate::RustExpr::Field {
                expr: Box::new(outer_target),
                field: nested_field.to_string(),
            },
            value: value_expr,
        }))
    }

    pub(crate) fn try_lower_structured_nested_field_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::NestedFieldAssign {
            object,
            field,
            field_ty,
            nested_field,
            nested_field_ty,
            value,
        } = stmt
        else {
            return Ok(false);
        };
        let Some(lowered) = self.lower_nested_field_assign_stmt_for_ir(
            object,
            field,
            field_ty,
            nested_field,
            nested_field_ty,
            value,
        )?
        else {
            return Ok(false);
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    fn optional_recursive_class_name(field_ty: &Type) -> Option<String> {
        let Type::Union(members) = field_ty.resolve_alias() else {
            return None;
        };
        let mut class_name: Option<String> = None;
        let mut has_none = false;
        for member in members {
            match member.resolve_alias() {
                Type::Class { name, .. } => class_name = Some(name.clone()),
                Type::None => has_none = true,
                _ => {}
            }
        }
        if has_none {
            class_name
        } else {
            None
        }
    }

    fn optional_class_name(ty: &Type) -> Option<String> {
        let Type::Union(members) = ty.resolve_alias() else {
            return None;
        };
        if members.len() != 2 {
            return None;
        }
        let mut class_name: Option<String> = None;
        let mut has_none = false;
        for member in members {
            match member.resolve_alias() {
                Type::Class { name, .. } => class_name = Some(name.clone()),
                Type::None => has_none = true,
                _ => return None,
            }
        }
        if has_none {
            class_name
        } else {
            None
        }
    }

    fn recursive_field_needs_boxing(&self, object: &str, field: &str, field_ty: &Type) -> bool {
        if object == "self"
            && self.current_class_name.as_ref().is_some_and(|class_name| {
                self.recursive_fields
                    .contains(&(class_name.clone(), field.to_string()))
            })
        {
            return true;
        }
        if let Some(class_name) = Self::optional_recursive_class_name(field_ty) {
            return self
                .recursive_fields
                .contains(&(class_name, field.to_string()));
        }
        false
    }

    fn adapt_field_assign_value_for_recursive_storage(
        &self,
        object: &str,
        field: &str,
        field_ty: &Type,
        value_expr: RustExpr,
        value_ty: &Type,
    ) -> RustExpr {
        if !self.recursive_field_needs_boxing(object, field, field_ty) {
            return value_expr;
        }

        let Some(class_name) = Self::optional_recursive_class_name(field_ty) else {
            if !Self::is_box_new_call_expr_for_ir(&value_expr) {
                return RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                    args: vec![value_expr],
                };
            }
            return value_expr;
        };

        let value_class_matches = matches!(
            value_ty.resolve_alias(),
            Type::Class { name, .. } if name == &class_name
        );

        if value_class_matches {
            let boxed_expr = if Self::is_box_new_call_expr_for_ir(&value_expr) {
                value_expr
            } else {
                RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                    args: vec![value_expr],
                }
            };
            return RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                args: vec![boxed_expr],
            };
        }

        if Self::optional_class_name(value_ty).as_deref() == Some(class_name.as_str()) {
            return RustExpr::MethodCall {
                receiver: Box::new(value_expr),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![crate::RustParam::Named {
                        name: "__sifr_recursive_value".to_string(),
                        ty: crate::RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                        args: vec![RustExpr::Ident("__sifr_recursive_value".to_string())],
                    }),
                    is_move: false,
                }],
            };
        }

        value_expr
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
        let key_is_non_copy_name = matches!(index, HirExpr::Name { .. })
            && matches!(
                crate::resolve_alias_type_for_plain_call(index.ty()),
                Type::Str | Type::LiteralStr(_)
            );

        let Some(mut index_expr) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(false);
        };
        if key_needs_clone || key_is_non_copy_name {
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

    pub(crate) fn try_lower_structured_attribute_nested_subscript_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::AttributeNestedSubscriptAssign {
            object,
            field,
            outer_index,
            inner_index,
            value,
            field_ty,
        } = stmt
        else {
            return Ok(false);
        };

        let Some(lowered) = self
            .lower_structured_attribute_nested_list_subscript_assign_stmt_for_ir(
                object,
                field,
                outer_index,
                inner_index,
                value,
                field_ty,
            )?
        else {
            return Ok(false);
        };
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

fn is_result_int_division_error_type(ty: &Type) -> bool {
    let Type::Result(ok_ty, err_ty) = ty else {
        return false;
    };
    matches!(
        crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
        Type::Int | Type::LiteralInt(_)
    ) && matches!(
        crate::resolve_alias_type_for_plain_call(err_ty.as_ref()),
        Type::Class { name, .. } if name == "DivisionError"
    )
}

fn result_int_to_sifr_int_rust_type(ty: &Type) -> crate::RustType {
    let Type::Result(_, err_ty) = ty else {
        return crate::RustType::Named(ty.rust_type());
    };
    crate::RustType::Result(
        Box::new(crate::RustType::Named("SifrInt".to_string())),
        Box::new(crate::sifr_type_to_rust_type(err_ty)),
    )
}
