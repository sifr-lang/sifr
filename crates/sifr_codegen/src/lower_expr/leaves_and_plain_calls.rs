use super::{
    detect_is_some_guard_name, is_bool_like_simple, is_enum_like_simple, is_int_like_simple,
    is_mixed_simple_float_binop, is_mixed_simple_float_floor_division_binop, is_numeric_simple,
    is_option_like_simple, is_promoted_fixed_width_integer_binop, is_reserved_builtin_call_func,
    is_safe_simple_binop, is_safe_simple_compare, is_simple_int_true_division_binop,
    is_string_like_simple, normalize_binop_op, normalize_compare_op, resolve_alias_type,
    try_lower_guarded_option_compare_expr, try_lower_mixed_float_operand_expr,
    try_lower_none_identity_compare_expr, try_lower_option_none_compare_expr,
    try_lower_promoted_integer_operand_expr, try_lower_simple_binop_operand_expr,
    try_lower_simple_compare_operand_expr, try_lower_simple_constructor_call_expr,
    try_lower_simple_dict_comp_expr, try_lower_simple_dict_literal_expr,
    try_lower_simple_divmod_call_expr, try_lower_simple_filter_call_expr,
    try_lower_simple_fstring_expr, try_lower_simple_generator_expr, try_lower_simple_index_expr,
    try_lower_simple_lambda_expr, try_lower_simple_list_comp_expr, try_lower_simple_map_call_expr,
    try_lower_simple_method_call_expr, try_lower_simple_range_operand_expr,
    try_lower_simple_set_comp_expr, try_lower_simple_set_literal_expr, try_lower_simple_slice_expr,
};
use crate::{CodegenError, RustExpr, RustLiteral, RustStmt, RustType};
use sifr_ir::{HirExpr, HirIteratorOp};
use sifr_type_system::Type;
use std::cell::RefCell;

thread_local! {
    static ALLOWED_PLAIN_CALLS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

pub(crate) fn with_allowed_plain_calls<T>(allowed_calls: &[String], f: impl FnOnce() -> T) -> T {
    ALLOWED_PLAIN_CALLS.with(|calls| {
        {
            let mut calls_mut = calls.borrow_mut();
            calls_mut.extend(allowed_calls.iter().cloned());
        }
        let result = f();
        {
            let mut calls_mut = calls.borrow_mut();
            let trunc_len = calls_mut.len().saturating_sub(allowed_calls.len());
            calls_mut.truncate(trunc_len);
        }
        result
    })
}

pub(super) fn is_allowed_plain_call(func: &str) -> bool {
    ALLOWED_PLAIN_CALLS.with(|calls| calls.borrow().iter().any(|name| name == func))
}

pub fn fixed_width_literal_expr_for_target(target_ty: &Type, value: &HirExpr) -> Option<RustExpr> {
    let Type::FixedInt(fixed) = crate::resolve_alias_type_for_plain_call(target_ty) else {
        return None;
    };
    let literal = integer_literal_decimal(value)?;
    Some(RustExpr::Verbatim(format!(
        "{literal}{}",
        fixed.rust_name()
    )))
}

pub(super) fn integer_literal_decimal(value: &HirExpr) -> Option<String> {
    match value {
        HirExpr::IntLiteral(value) => Some(value.to_string()),
        HirExpr::LargeIntLiteral(value) => Some(value.clone()),
        HirExpr::UnaryOp { op, operand, .. } if op == "+" => integer_literal_decimal(operand),
        HirExpr::UnaryOp { op, operand, .. } if op == "-" => {
            let value = integer_literal_decimal(operand)?;
            Some(format!("-{value}"))
        }
        _ => None,
    }
}

pub(super) fn iterator_op_func_name(op: &HirIteratorOp) -> &'static str {
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

pub fn try_lower_leaf_expr_result(expr: &HirExpr) -> Result<Option<RustExpr>, CodegenError> {
    validate_leaf_expr_shape(expr)?;
    Ok(try_lower_leaf_expr(expr))
}

pub(super) fn validate_leaf_expr_shape(expr: &HirExpr) -> Result<(), CodegenError> {
    if let HirExpr::Compare {
        ops, comparators, ..
    } = expr
    {
        if !ops.is_empty() && ops.len() != comparators.len() {
            return Err(CodegenError::new(
                "invalid compare expression shape: ops/comparators length mismatch",
            ));
        }
    }
    Ok(())
}

pub(crate) fn is_leaf_expr_candidate(expr: &HirExpr) -> bool {
    match expr {
        HirExpr::IntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::Name { .. }
        | HirExpr::EnumVariant { .. }
        | HirExpr::UnaryOp { .. }
        | HirExpr::BinOp { .. }
        | HirExpr::IfExpr { .. }
        | HirExpr::TupleLiteral { .. }
        | HirExpr::ListLiteral { .. }
        | HirExpr::RangeLiteral { .. }
        | HirExpr::FieldAccess { .. }
        | HirExpr::ContainsOp { .. }
        | HirExpr::QuestionMark { .. }
        | HirExpr::Await { .. }
        | HirExpr::OkWrap { .. }
        | HirExpr::ErrWrap { .. }
        | HirExpr::WalrusExpr { .. }
        | HirExpr::SuperCall { .. }
        | HirExpr::FString { .. }
        | HirExpr::Lambda { .. }
        | HirExpr::IteratorCall { .. } => true,
        HirExpr::Compare {
            ops, comparators, ..
        } => !ops.is_empty() && ops.len() == comparators.len(),
        HirExpr::BoolOp { values, .. } => values.len() >= 2,
        _ => false,
    }
}

/// Lowers leaf expressions that don't require emitter state.
/// This is the first incremental IR rollout from `emit_expr` string writes
/// to IR + renderer output.
pub fn try_lower_leaf_expr(expr: &HirExpr) -> Option<RustExpr> {
    match expr {
        HirExpr::IntLiteral(v) => Some(RustExpr::Cast {
            expr: Box::new(RustExpr::Literal(RustLiteral::Int(*v))),
            ty: RustType::I64,
        }),
        HirExpr::FloatLiteral(v) if v.is_finite() => Some(RustExpr::Cast {
            expr: Box::new(RustExpr::Literal(RustLiteral::Float(*v))),
            ty: RustType::F64,
        }),
        HirExpr::FloatLiteral(v) => Some(RustExpr::Literal(RustLiteral::Float(*v))),
        HirExpr::StringLiteral(s) => Some(RustExpr::Literal(RustLiteral::Str(s.clone()))),
        HirExpr::BoolLiteral(v) => Some(RustExpr::Literal(RustLiteral::Bool(*v))),
        HirExpr::NoneLiteral => Some(RustExpr::Literal(RustLiteral::None)),
        HirExpr::Name { name, ty, .. }
            if is_bool_like_simple(ty)
                || is_numeric_simple(ty)
                || is_string_like_simple(ty)
                || is_enum_like_simple(ty) =>
        {
            Some(RustExpr::Ident(name.clone()))
        }
        HirExpr::EnumVariant {
            enum_name, variant, ..
        } => Some(RustExpr::Path(vec![
            sifr_type_system::source_class_rust_name(enum_name),
            variant.clone(),
        ])),
        HirExpr::UnaryOp { op, operand, .. } => match op.as_str() {
            "-" => Some(RustExpr::UnaryOp {
                op: "-".to_string(),
                operand: Box::new(try_lower_leaf_expr(operand)?),
            }),
            "+" => Some(try_lower_leaf_expr(operand)?),
            "~" if is_int_like_simple(operand.ty()) => {
                let lowered_operand = try_lower_leaf_expr(operand).or_else(|| {
                    if let HirExpr::Name { name, .. } = operand.as_ref() {
                        return Some(RustExpr::Ident(name.clone()));
                    }
                    None
                })?;
                Some(RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(lowered_operand),
                })
            }
            "not" if is_bool_like_simple(operand.ty()) => {
                let lowered_operand = try_lower_leaf_expr(operand).or_else(|| {
                    if let HirExpr::Name { name, .. } = operand.as_ref() {
                        return Some(RustExpr::Ident(name.clone()));
                    }
                    None
                })?;
                Some(RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(lowered_operand),
                })
            }
            "not" if is_option_like_simple(operand.ty()) => {
                if let HirExpr::Name { name, .. } = operand.as_ref() {
                    return Some(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(name.clone())),
                        method: "is_none".to_string(),
                        args: vec![],
                    });
                }
                None
            }
            _ => None,
        },
        HirExpr::BinOp {
            left,
            op,
            right,
            ty,
        } => {
            if (is_option_like_simple(left.ty()) || is_option_like_simple(right.ty()))
                && !is_option_like_simple(ty)
            {
                return None;
            }
            if !is_safe_simple_binop(op, left.ty(), right.ty(), ty) {
                return None;
            }
            if is_mixed_simple_float_binop(op, left.ty(), right.ty(), ty)
                || is_mixed_simple_float_floor_division_binop(op, left.ty(), right.ty(), ty)
                || is_simple_int_true_division_binop(op, left.ty(), right.ty(), ty)
            {
                return Some(RustExpr::BinOp {
                    left: Box::new(try_lower_mixed_float_operand_expr(left)?),
                    op: normalize_binop_op(op).to_string(),
                    right: Box::new(try_lower_mixed_float_operand_expr(right)?),
                });
            }
            if is_promoted_fixed_width_integer_binop(op, left.ty(), right.ty(), ty) {
                return Some(RustExpr::BinOp {
                    left: Box::new(try_lower_promoted_integer_operand_expr(left)?),
                    op: normalize_binop_op(op).to_string(),
                    right: Box::new(try_lower_promoted_integer_operand_expr(right)?),
                });
            }
            Some(RustExpr::BinOp {
                left: Box::new(try_lower_simple_binop_operand_expr(left)?),
                op: normalize_binop_op(op).to_string(),
                right: Box::new(try_lower_simple_binop_operand_expr(right)?),
            })
        }
        HirExpr::Compare {
            left,
            ops,
            comparators,
            ..
        } if !ops.is_empty() && ops.len() == comparators.len() => {
            if ops.len() == 1 {
                let right = comparators.first()?;
                if let Some(lowered) = try_lower_option_none_compare_expr(left, &ops[0], right) {
                    return Some(lowered);
                }
                if let Some(lowered) = try_lower_none_identity_compare_expr(left, &ops[0], right) {
                    return Some(lowered);
                }
            }

            let mut lhs_expr = left.as_ref();
            let mut lowered_chain: Option<RustExpr> = None;

            for (idx, op) in ops.iter().enumerate() {
                let rhs_expr = comparators.get(idx)?;
                let normalized_op = normalize_compare_op(op);
                if !is_safe_simple_compare(normalized_op, lhs_expr.ty(), rhs_expr.ty()) {
                    return None;
                }

                let cmp = RustExpr::BinOp {
                    left: Box::new(try_lower_simple_compare_operand_expr(lhs_expr)?),
                    op: normalized_op.to_string(),
                    right: Box::new(try_lower_simple_compare_operand_expr(rhs_expr)?),
                };

                lowered_chain = Some(if let Some(existing) = lowered_chain {
                    RustExpr::BinOp {
                        left: Box::new(existing),
                        op: "&&".to_string(),
                        right: Box::new(cmp),
                    }
                } else {
                    cmp
                });

                lhs_expr = rhs_expr;
            }

            lowered_chain
        }
        HirExpr::BoolOp { op, values, .. } if values.len() >= 2 => {
            let lowered_op = match op.as_str() {
                "and" => "&&",
                "or" => "||",
                _ => return None,
            };
            if op == "and" && values.len() == 2 {
                if let Some(guarded_name) = detect_is_some_guard_name(&values[0]) {
                    if let Some(lowered_guarded_compare) =
                        try_lower_guarded_option_compare_expr(&values[1], &guarded_name)
                    {
                        return Some(RustExpr::BinOp {
                            left: Box::new(try_lower_leaf_expr(&values[0])?),
                            op: lowered_op.to_string(),
                            right: Box::new(lowered_guarded_compare),
                        });
                    }
                }
            }

            let mut iter = values.iter();
            let mut lowered = try_lower_leaf_expr(iter.next()?)?;
            for value in iter {
                lowered = RustExpr::BinOp {
                    left: Box::new(lowered),
                    op: lowered_op.to_string(),
                    right: Box::new(try_lower_leaf_expr(value)?),
                };
            }
            Some(lowered)
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => Some(RustExpr::If {
            cond: Box::new(try_lower_leaf_expr(condition)?),
            then_expr: Box::new(try_lower_leaf_expr(then_expr)?),
            else_expr: Some(Box::new(try_lower_leaf_expr(else_expr)?)),
        }),
        HirExpr::TupleLiteral { elements, ty } => {
            let lowered = elements
                .iter()
                .map(try_lower_leaf_expr)
                .collect::<Option<Vec<_>>>()?;
            if crate::homogeneous_large_tuple_backing_array(ty).is_some() {
                Some(RustExpr::Array(lowered))
            } else {
                Some(RustExpr::Tuple(lowered))
            }
        }
        HirExpr::ListLiteral { elements, ty } => {
            if elements.is_empty() {
                if let Some(lowered) = super::typed_empty_list_expr(ty) {
                    return Some(lowered);
                }
            }
            let list_ty = resolve_alias_type(ty);
            let mut lowered = elements
                .iter()
                .map(|element| {
                    try_lower_leaf_expr(element).map(|lowered| {
                        let lowered =
                            crate::RustEmitter::clone_non_copy_name_expr_for_ir(element, lowered);
                        if let Type::List(element_ty) = list_ty {
                            crate::helpers::adapt_collection_value_for_target(
                                element_ty.as_ref(),
                                element,
                                lowered,
                            )
                        } else {
                            lowered
                        }
                    })
                })
                .collect::<Option<Vec<_>>>()?;
            if matches!(list_ty, Type::Bytes) {
                lowered = lowered
                    .into_iter()
                    .map(|element| RustExpr::Cast {
                        expr: Box::new(element),
                        ty: RustType::Named("u8".to_string()),
                    })
                    .collect();
            }
            Some(RustExpr::Vec(lowered))
        }
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            let lowered_range = RustExpr::Range {
                start: Box::new(try_lower_simple_range_operand_expr(start)?),
                end: Box::new(try_lower_simple_range_operand_expr(end)?),
            };

            if let Some(step_expr) = step.as_ref() {
                Some(RustExpr::MethodCall {
                    receiver: Box::new(lowered_range),
                    method: "step_by".to_string(),
                    args: vec![RustExpr::Cast {
                        expr: Box::new(try_lower_simple_range_operand_expr(step_expr)?),
                        ty: RustType::Named("usize".to_string()),
                    }],
                })
            } else {
                Some(lowered_range)
            }
        }
        HirExpr::FieldAccess { .. } => None,
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => {
            if matches!(
                collection.as_ref(),
                HirExpr::Index { object, .. }
                    if matches!(
                        object.ty(),
                        Type::Alias { name, .. }
                            if crate::intrinsics::is_collection_defaultdict_storage_alias(name)
                    )
            ) {
                return None;
            }
            let collection_ty = resolve_alias_type(collection.ty());
            let method = match collection_ty {
                Type::Dict(_, _) => "contains_key",
                Type::List(_) | Type::Set(_) | Type::Range | Type::Str => "contains",
                _ => return None,
            };
            let mut lowered_element = try_lower_leaf_or_name_expr(element)?;
            if let Some(element_ty) = collection_ty.contains_element_type() {
                let owned_element = if matches!(element.as_ref(), HirExpr::Name { .. })
                    && !crate::helpers::is_copy_type_for_codegen(element.ty())
                {
                    RustExpr::Clone(Box::new(lowered_element.clone()))
                } else {
                    lowered_element.clone()
                };
                if let Some(wrapped) =
                    crate::helpers::wrap_union_member_expr(&element_ty, element.ty(), owned_element)
                {
                    lowered_element = wrapped;
                }
            }
            let arg = RustExpr::Ref {
                mutable: false,
                expr: Box::new(lowered_element),
            };
            Some(RustExpr::MethodCall {
                receiver: Box::new(try_lower_leaf_or_name_expr(collection)?),
                method: method.to_string(),
                args: vec![arg],
            })
        }
        HirExpr::QuestionMark { expr, .. } => {
            Some(RustExpr::Try(Box::new(try_lower_leaf_or_name_expr(expr)?)))
        }
        HirExpr::Await { value, .. } => {
            let lowered_value = if let HirExpr::Call { func, args, .. } = value.as_ref() {
                if func == "__sifr_task_sleep" {
                    try_lower_task_sleep_call_expr(args)?
                } else if await_call_needs_convention_aware_lowering(args) {
                    return None;
                } else {
                    let lowered_args = args
                        .iter()
                        .map(try_lower_leaf_or_name_expr)
                        .collect::<Option<Vec<_>>>()?;
                    RustExpr::FnCall {
                        func: Box::new(crate::stmt_support_emitter::plain_call_target_for_ir(func)),
                        args: lowered_args,
                    }
                }
            } else if let HirExpr::MethodCall {
                object,
                method,
                args,
                ..
            } = value.as_ref()
            {
                try_lower_simple_method_call_expr(object, method, args)?
            } else if matches!(
                resolve_alias_type(value.ty()),
                Type::Task(_, _) | Type::BlockingTask(_, _)
            ) {
                RustExpr::MethodCall {
                    receiver: Box::new(try_lower_leaf_or_name_expr(value)?),
                    method: "join".to_string(),
                    args: vec![],
                }
            } else {
                try_lower_leaf_or_name_expr(value)?
            };
            Some(RustExpr::Await(Box::new(lowered_value)))
        }
        HirExpr::OkWrap { value, .. } => Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
            args: vec![try_lower_leaf_or_name_expr(value)?],
        }),
        HirExpr::ErrWrap { value, .. } => Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
            args: vec![try_lower_leaf_or_name_expr(value)?],
        }),
        HirExpr::WalrusExpr { name, value, .. } => Some(RustExpr::Block {
            stmts: vec![RustStmt::Let {
                mutable: false,
                name: name.clone(),
                ty: None,
                value: try_lower_leaf_or_name_expr(value)?,
            }],
            expr: Some(Box::new(RustExpr::Ident(name.clone()))),
        }),
        HirExpr::SuperCall {
            parent_type,
            method,
            args,
            ..
        } => {
            let mut lowered_args = args
                .iter()
                .map(try_lower_leaf_or_name_expr)
                .collect::<Option<Vec<_>>>()?;
            if method != "new" {
                lowered_args.insert(0, RustExpr::Ident("self".to_string()));
            }
            Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    parent_type.rust_type(),
                    method.clone(),
                ])),
                args: lowered_args,
            })
        }
        HirExpr::FString { parts, .. } => try_lower_simple_fstring_expr(parts),
        HirExpr::Lambda { params, body, .. } => try_lower_simple_lambda_expr(params, body),
        HirExpr::Call { func, args, .. } => try_lower_simple_call_expr(func, args),
        HirExpr::PythonCall { func, args, .. } => try_lower_simple_call_expr(func, args),
        HirExpr::IteratorCall { op, args, .. } => match op {
            HirIteratorOp::Map => try_lower_simple_map_call_expr(args),
            HirIteratorOp::Filter => try_lower_simple_filter_call_expr(args),
            _ => try_lower_simple_call_expr(iterator_op_func_name(op), args),
        },
        HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } => try_lower_simple_method_call_expr(object, method, args),
        HirExpr::ConstructorCall {
            class_name, args, ..
        } => try_lower_simple_constructor_call_expr(class_name, args),
        HirExpr::Index { object, index, ty } => try_lower_simple_index_expr(object, index, ty),
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            try_lower_simple_slice_expr(object, start.as_deref(), stop.as_deref(), step.as_deref())
        }
        HirExpr::DictLiteral { keys, values, ty } => {
            try_lower_simple_dict_literal_expr(keys, values, ty)
        }
        HirExpr::SetLiteral { elements, ty } => try_lower_simple_set_literal_expr(elements, ty),
        HirExpr::ListComp { .. } | HirExpr::DictComp { .. } | HirExpr::SetComp { .. } => {
            try_lower_simple_comprehension_expr(expr)
        }
        HirExpr::GeneratorExpr {
            expr,
            var,
            iter,
            filter,
            ty,
        } => try_lower_simple_generator_expr(expr, var, iter, filter.as_deref(), ty),
        _ => None,
    }
}

pub(crate) fn try_lower_simple_comprehension_expr(expr: &HirExpr) -> Option<RustExpr> {
    match expr {
        HirExpr::ListComp {
            expr,
            generators,
            ty,
        } => try_lower_simple_list_comp_expr(expr, generators, ty),
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ty,
        } => try_lower_simple_dict_comp_expr(key_expr, val_expr, generators, ty),
        HirExpr::SetComp {
            expr,
            generators,
            ty,
        } => try_lower_simple_set_comp_expr(expr, generators, ty),
        _ => None,
    }
}

pub(super) fn try_lower_leaf_or_name_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let Some(lowered) = try_lower_leaf_expr(expr) {
        return Some(lowered);
    }
    if let HirExpr::Name { name, .. } = expr {
        return Some(RustExpr::Ident(name.clone()));
    }
    None
}

pub(super) fn try_lower_simple_call_expr(func: &str, args: &[HirExpr]) -> Option<RustExpr> {
    if func == "__sifr_python_omitted_argument" && args.is_empty() {
        return Some(RustExpr::Literal(RustLiteral::None));
    }
    if func == "__sifr_python_present_argument" {
        let [value] = args else {
            return None;
        };
        return Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![try_lower_leaf_or_name_expr(value)?],
        });
    }
    if func == "__sifr_task_sleep" {
        return try_lower_task_sleep_call_expr(args);
    }
    if args.iter().any(|arg| {
        matches!(
            resolve_alias_type(arg.ty()),
            Type::Class {
                parent_class: Some(_),
                ..
            }
        )
    }) {
        return None;
    }
    if func == "__sifr_task_gather" {
        let [handles] = args else {
            return None;
        };
        return Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Ident(func.to_string())),
            args: vec![try_lower_leaf_or_name_expr(handles)?],
        });
    }
    if func == "__sifr_task_race" {
        let [handles] = args else {
            return None;
        };
        return Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Ident(func.to_string())),
            args: vec![try_lower_leaf_or_name_expr(handles)?],
        });
    }
    if func == "__sifr_task_select" {
        let [first, second] = args else {
            return None;
        };
        return Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Ident(func.to_string())),
            args: vec![
                try_lower_leaf_or_name_expr(first)?,
                try_lower_leaf_or_name_expr(second)?,
            ],
        });
    }
    if func == "__sifr_join_set_new" {
        if !args.is_empty() {
            return None;
        }
        return Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Ident(func.to_string())),
            args: vec![],
        });
    }
    if func == "__sifr_spawn_blocking_infallible"
        || func == "__sifr_spawn_blocking_result"
        || func == "__sifr_spawn_cpu_infallible"
        || func == "__sifr_spawn_cpu_result"
    {
        let [worker] = args else {
            return None;
        };
        return Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Ident(func.to_string())),
            args: vec![try_lower_leaf_or_name_expr(worker)?],
        });
    }
    if func == "__sifr_parallel_map" || func == "__sifr_parallel_try_map" {
        let [items, worker] = args else {
            return None;
        };
        return Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Ident(func.to_string())),
            args: vec![
                try_lower_leaf_or_name_expr(items)?,
                try_lower_leaf_or_name_expr(worker)?,
            ],
        });
    }
    if func == "__sifr_pool_map" || func == "__sifr_pool_try_map" {
        let [pool, items, worker] = args else {
            return None;
        };
        return Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Ident(func.to_string())),
            args: vec![
                RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(try_lower_leaf_or_name_expr(pool)?),
                },
                try_lower_leaf_or_name_expr(items)?,
                try_lower_leaf_or_name_expr(worker)?,
            ],
        });
    }
    if func == "anext" {
        let [iterator] = args else {
            return None;
        };
        return Some(RustExpr::MethodCall {
            receiver: Box::new(try_lower_leaf_or_name_expr(iterator)?),
            method: "anext".to_string(),
            args: vec![],
        });
    }
    if func == "hash" {
        return try_lower_simple_hash_call_expr(args);
    }
    if func == "divmod" {
        return try_lower_simple_divmod_call_expr(args);
    }
    if func == "map" {
        return try_lower_simple_map_call_expr(args);
    }
    if func == "filter" {
        return try_lower_simple_filter_call_expr(args);
    }

    if is_reserved_builtin_call_func(func) {
        return None;
    }
    // Keep namespaced calls on the structured emitter path so ownership/convention
    // handling can use full signature metadata.
    if func.contains("::") {
        return None;
    }
    if !is_allowed_plain_call(func) {
        return None;
    }
    // Result-typed arguments frequently need target-parameter error coercion.
    if args
        .iter()
        .any(|arg| matches!(resolve_alias_type(arg.ty()), Type::Result(_, _)))
    {
        return None;
    }

    let lowered_args = args
        .iter()
        .map(try_lower_leaf_or_name_expr)
        .collect::<Option<Vec<_>>>()?;

    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Ident(func.to_string())),
        args: lowered_args,
    })
}

fn await_call_needs_convention_aware_lowering(args: &[HirExpr]) -> bool {
    args.iter().any(|arg| {
        !crate::helpers::is_copy_type_for_codegen(arg.ty())
            || matches!(
                arg.ty().resolve_alias(),
                Type::Function(_) | Type::AsyncFunction(_) | Type::AsyncCallable(..)
            )
    })
}

pub(super) fn try_lower_task_sleep_call_expr(args: &[HirExpr]) -> Option<RustExpr> {
    let [duration] = args else {
        return None;
    };
    let duration_expr = try_lower_task_duration_expr(duration, "__sifr_task_sleep_seconds")?;
    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "tokio".to_string(),
            "time".to_string(),
            "sleep".to_string(),
        ])),
        args: vec![duration_expr],
    })
}

pub(crate) fn try_lower_task_duration_expr(
    duration: &HirExpr,
    seconds_name: &str,
) -> Option<RustExpr> {
    let seconds = RustExpr::Cast {
        expr: Box::new(try_lower_leaf_or_name_expr(duration)?),
        ty: RustType::F64,
    };
    let seconds_name = seconds_name.to_string();
    let finite_positive = RustExpr::BinOp {
        left: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(seconds_name.clone())),
            method: "is_finite".to_string(),
            args: vec![],
        }),
        op: "&&".to_string(),
        right: Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident(seconds_name.clone())),
            op: ">".to_string(),
            right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
        }),
    };
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: seconds_name.clone(),
            ty: Some(RustType::F64),
            value: seconds,
        }],
        expr: Some(Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "time".to_string(),
                "Duration".to_string(),
                "from_secs_f64".to_string(),
            ])),
            args: vec![RustExpr::If {
                cond: Box::new(finite_positive),
                then_expr: Box::new(RustExpr::Ident(seconds_name)),
                else_expr: Some(Box::new(RustExpr::Literal(RustLiteral::Float(0.0)))),
            }],
        })),
    })
}

pub(super) fn try_lower_simple_hash_call_expr(args: &[HirExpr]) -> Option<RustExpr> {
    let [arg] = args else {
        return None;
    };
    let lowered_arg = try_lower_leaf_or_name_expr(arg)?;
    let hasher_ident = "__sifr_hash".to_string();

    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: true,
                name: hasher_ident.clone(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "collections".to_string(),
                        "hash_map".to_string(),
                        "DefaultHasher".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                },
            },
            RustStmt::Expr(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "hash".to_string(),
                    "Hash".to_string(),
                    "hash".to_string(),
                ])),
                args: vec![
                    RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(lowered_arg),
                    },
                    RustExpr::Ref {
                        mutable: true,
                        expr: Box::new(RustExpr::Ident(hasher_ident.clone())),
                    },
                ],
            }),
        ],
        expr: Some(Box::new(RustExpr::Cast {
            expr: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "hash".to_string(),
                    "Hasher".to_string(),
                    "finish".to_string(),
                ])),
                args: vec![RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Ident(hasher_ident)),
                }],
            }),
            ty: RustType::I64,
        })),
    })
}
