use super::{
    HirExpr, HirFStringPart, HirParam, RustExpr, RustLiteral, RustParam, RustStmt, RustType, Type,
    try_lower_dict_get_key_expr, try_lower_leaf_expr, try_lower_leaf_or_name_expr,
    try_lower_simple_defaultdict_index_expr, try_lower_simple_iter_source_expr,
};
pub(super) fn try_lower_simple_index_expr(
    object: &HirExpr,
    index: &HirExpr,
    result_ty: &Type,
) -> Option<RustExpr> {
    if let Some(lowered) = try_lower_simple_defaultdict_index_expr(object, index) {
        return Some(lowered);
    }
    match resolve_alias_type(object.ty()) {
        Type::Dict(_, value_ty) => {
            let projection_method =
                crate::helpers::option_projection_method_for_owned_type(value_ty.as_ref());
            let projected = RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(try_lower_leaf_or_name_expr(object)?),
                    method: "get".to_string(),
                    args: vec![try_lower_dict_get_key_expr(index)?],
                }),
                method: projection_method.to_string(),
                args: vec![],
            };
            let projected =
                crate::helpers::normalize_safe_option_result(value_ty.as_ref(), projected);
            if is_option_like_simple(result_ty) {
                Some(projected)
            } else {
                None
            }
        }
        Type::Any => None,
        _ => None,
    }
}

pub(super) fn try_lower_simple_slice_expr(
    object: &HirExpr,
    start: Option<&HirExpr>,
    stop: Option<&HirExpr>,
    step: Option<&HirExpr>,
) -> Option<RustExpr> {
    if object.ty() != &Type::Any || step.is_some() {
        return None;
    }

    let lowered_start = start.and_then(try_lower_leaf_or_name_expr).map(Box::new);
    let lowered_stop = stop.and_then(try_lower_leaf_or_name_expr).map(Box::new);

    Some(RustExpr::Slice {
        expr: Box::new(try_lower_leaf_or_name_expr(object)?),
        start: lowered_start,
        stop: lowered_stop,
    })
}

pub(super) fn try_lower_simple_dict_literal_expr(
    keys: &[HirExpr],
    values: &[HirExpr],
    ty: &Type,
) -> Option<RustExpr> {
    if keys.len() != values.len() {
        return None;
    }
    let mut entries = Vec::with_capacity(keys.len());
    for (key, value) in keys.iter().zip(values.iter()) {
        let lowered_key = try_lower_leaf_or_name_expr(key)?;
        let lowered_value = try_lower_leaf_or_name_expr(value)?;
        let (lowered_key, lowered_value) = match resolve_alias_type(ty) {
            Type::Dict(key_ty, value_ty) => (
                crate::helpers::adapt_collection_value_for_target(
                    key_ty.as_ref(),
                    key,
                    lowered_key,
                ),
                crate::helpers::adapt_collection_value_for_target(
                    value_ty.as_ref(),
                    value,
                    lowered_value,
                ),
            ),
            _ => (lowered_key, lowered_value),
        };
        entries.push(RustExpr::Tuple(vec![lowered_key, lowered_value]));
    }

    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "HashMap".to_string(),
            "from".to_string(),
        ])),
        args: vec![RustExpr::Array(entries)],
    })
}

pub(super) fn try_lower_simple_set_literal_expr(
    elements: &[HirExpr],
    ty: &Type,
) -> Option<RustExpr> {
    let mut lowered_elements = Vec::with_capacity(elements.len());
    for element in elements {
        let lowered = try_lower_leaf_or_name_expr(element)?;
        let lowered = crate::RustEmitter::clone_non_copy_name_expr_for_ir(element, lowered);
        let lowered = match resolve_alias_type(ty) {
            Type::Set(element_ty) => crate::helpers::adapt_collection_value_for_target(
                element_ty.as_ref(),
                element,
                lowered,
            ),
            _ => lowered,
        };
        lowered_elements.push(lowered);
    }

    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "HashSet".to_string(),
            "from".to_string(),
        ])),
        args: vec![RustExpr::Array(lowered_elements)],
    })
}

pub(super) fn try_lower_simple_list_comp_expr(
    expr: &HirExpr,
    generators: &[(String, HirExpr, Option<HirExpr>)],
    ty: &Type,
) -> Option<RustExpr> {
    if generators.is_empty() || !matches!(resolve_alias_type(ty), Type::Any | Type::List(_)) {
        return None;
    }

    let result_ident = "__sifr_list_comp".to_string();
    let lowered_expr = try_lower_leaf_or_name_expr(expr)?;
    let lowered_expr =
        crate::ownership_plan::materialize_comprehension_value(expr, lowered_expr, generators);
    let lowered_expr = match resolve_alias_type(ty) {
        Type::List(element_ty) => crate::helpers::adapt_collection_value_for_target(
            element_ty.as_ref(),
            expr,
            lowered_expr,
        ),
        _ => lowered_expr,
    };
    let mut nested_body = vec![RustStmt::Expr(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(result_ident.clone())),
        method: "push".to_string(),
        args: vec![lowered_expr],
    })];

    for (var, iter_expr, maybe_filter) in generators.iter().rev() {
        if var.contains(',') {
            return None;
        }
        let iter = try_lower_simple_iter_source_expr(iter_expr)?;
        let loop_body = if let Some(filter) = maybe_filter {
            vec![RustStmt::If {
                cond: try_lower_leaf_or_name_expr(filter)?,
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

    Some(RustExpr::Block {
        stmts,
        expr: Some(Box::new(RustExpr::Ident(result_ident))),
    })
}

pub(super) fn try_lower_simple_dict_comp_expr(
    key_expr: &HirExpr,
    val_expr: &HirExpr,
    generators: &[(String, HirExpr, Option<HirExpr>)],
    ty: &Type,
) -> Option<RustExpr> {
    if generators.len() != 1 || !matches!(resolve_alias_type(ty), Type::Any | Type::Dict(_, _)) {
        return None;
    }

    let (var, iter_expr, maybe_filter) = generators.first()?;
    if var.contains(',') {
        return None;
    }

    let iter = try_lower_simple_iter_source_expr(iter_expr)?;

    let result_ident = "__sifr_dict_comp".to_string();
    let lowered_key = crate::helpers::clone_dict_key_for_reused_value(
        key_expr,
        val_expr,
        try_lower_leaf_or_name_expr(key_expr)?,
    );
    let lowered_value = try_lower_leaf_or_name_expr(val_expr)?;
    let (lowered_key, lowered_value) = match resolve_alias_type(ty) {
        Type::Dict(key_ty, value_ty) => (
            crate::helpers::adapt_collection_value_for_target(
                key_ty.as_ref(),
                key_expr,
                lowered_key,
            ),
            crate::helpers::adapt_collection_value_for_target(
                value_ty.as_ref(),
                val_expr,
                lowered_value,
            ),
        ),
        _ => (lowered_key, lowered_value),
    };
    let insert_stmt = RustStmt::Expr(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(result_ident.clone())),
        method: "insert".to_string(),
        args: vec![lowered_key, lowered_value],
    });

    let loop_body = if let Some(filter) = maybe_filter {
        vec![RustStmt::If {
            cond: try_lower_leaf_or_name_expr(filter)?,
            then_body: vec![insert_stmt],
            else_body: None,
        }]
    } else {
        vec![insert_stmt]
    };

    let stmts = vec![
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
    ];

    Some(RustExpr::Block {
        stmts,
        expr: Some(Box::new(RustExpr::Ident(result_ident))),
    })
}

pub(super) fn try_lower_simple_set_comp_expr(
    expr: &HirExpr,
    generators: &[(String, HirExpr, Option<HirExpr>)],
    ty: &Type,
) -> Option<RustExpr> {
    if generators.len() != 1 || !matches!(resolve_alias_type(ty), Type::Any | Type::Set(_)) {
        return None;
    }

    let (var, iter_expr, maybe_filter) = generators.first()?;
    if var.contains(',') {
        return None;
    }

    let iter = try_lower_simple_iter_source_expr(iter_expr)?;

    let result_ident = "__sifr_set_comp".to_string();
    let lowered_expr = try_lower_leaf_or_name_expr(expr)?;
    let lowered_expr = match resolve_alias_type(ty) {
        Type::Set(element_ty) => crate::helpers::adapt_collection_value_for_target(
            element_ty.as_ref(),
            expr,
            lowered_expr,
        ),
        _ => lowered_expr,
    };
    let insert_stmt = RustStmt::Expr(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(result_ident.clone())),
        method: "insert".to_string(),
        args: vec![lowered_expr],
    });

    let loop_body = if let Some(filter) = maybe_filter {
        vec![RustStmt::If {
            cond: try_lower_leaf_or_name_expr(filter)?,
            then_body: vec![insert_stmt],
            else_body: None,
        }]
    } else {
        vec![insert_stmt]
    };

    let stmts = vec![
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
    ];

    Some(RustExpr::Block {
        stmts,
        expr: Some(Box::new(RustExpr::Ident(result_ident))),
    })
}

pub(super) fn try_lower_simple_generator_expr(
    expr: &HirExpr,
    var: &str,
    iter: &HirExpr,
    filter: Option<&HirExpr>,
    ty: &Type,
) -> Option<RustExpr> {
    if !matches!(resolve_alias_type(ty), Type::Any | Type::Iterator(_))
        || filter.is_some()
        || var.contains(',')
    {
        return None;
    }

    let iter_chain = try_lower_simple_iter_source_expr(iter)?;

    Some(RustExpr::MethodCall {
        receiver: Box::new(iter_chain),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: var.to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(try_lower_leaf_or_name_expr(expr)?),
            is_move: false,
        }],
    })
}

pub(super) fn is_reserved_builtin_call_func(func: &str) -> bool {
    matches!(
        func,
        "print"
            | "isinstance"
            | "list"
            | "str"
            | "tuple"
            | "pow"
            | "abs"
            | "hash"
            | "round"
            | "repr"
            | "dict"
            | "int"
            | "Decimal"
            | "BigDecimal"
            | "float"
            | "bool"
            | "ord"
            | "chr"
            | "min"
            | "max"
            | "sum"
            | "sorted"
            | "reversed"
            | "enumerate"
            | "zip"
            | "any"
            | "all"
            | "map"
            | "filter"
            | "builtin_open"
    )
}

pub(super) fn try_lower_simple_fstring_expr(parts: &[HirFStringPart]) -> Option<RustExpr> {
    let mut format_str = String::new();
    let mut lowered_args = Vec::new();

    for part in parts {
        match part {
            HirFStringPart::Literal(s) => {
                for ch in s.chars() {
                    match ch {
                        '{' => format_str.push_str("{{"),
                        '}' => format_str.push_str("}}"),
                        _ => format_str.push(ch),
                    }
                }
            }
            HirFStringPart::Expr(expr) => {
                if is_option_like_simple(expr.ty()) {
                    return None;
                }
                format_str.push_str("{}");
                if matches!(resolve_alias_type(expr.ty()), Type::None) {
                    lowered_args.push(RustExpr::Literal(crate::RustLiteral::Str(
                        "None".to_string(),
                    )));
                } else {
                    lowered_args.push(try_lower_leaf_or_name_expr(expr)?);
                }
            }
        }
    }

    Some(RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str,
        args: lowered_args,
    })
}

pub(super) fn try_lower_simple_lambda_expr(
    params: &[HirParam],
    body: &HirExpr,
) -> Option<RustExpr> {
    if params.iter().any(|param| param.ty != Type::Any) {
        return None;
    }

    let lowered_params = params
        .iter()
        .map(|param| RustParam::Named {
            name: param.name.clone(),
            ty: RustType::Named("_".to_string()),
        })
        .collect::<Vec<_>>();

    Some(RustExpr::Closure {
        params: lowered_params,
        body: Box::new(try_lower_leaf_or_name_expr(body)?),
        is_move: false,
    })
}

pub(super) fn is_numeric_simple(ty: &Type) -> bool {
    normalize_simple_numeric_scalar_type(ty).is_some()
}

pub(super) fn is_int_like_simple(ty: &Type) -> bool {
    matches!(normalize_simple_numeric_scalar_type(ty), Some("int"))
}

pub(super) fn is_fixed_width_int_like_simple(ty: &Type) -> bool {
    matches!(
        resolve_alias_type(ty),
        Type::FixedInt(fixed) if fixed.supports_current_scalar_promotion_to_int()
    )
}

pub(super) fn is_float_like_simple(ty: &Type) -> bool {
    matches!(normalize_simple_numeric_scalar_type(ty), Some("float"))
}

pub(super) fn is_bool_like_simple(ty: &Type) -> bool {
    matches!(normalize_simple_compare_scalar_type(ty), Some("bool"))
}

pub(super) fn is_string_like_simple(ty: &Type) -> bool {
    matches!(normalize_simple_compare_scalar_type(ty), Some("str"))
}

pub(super) fn resolve_alias_type(ty: &Type) -> &Type {
    match ty {
        Type::Alias { body, .. } => resolve_alias_type(body),
        _ => ty,
    }
}

pub(super) fn is_enum_like_simple(ty: &Type) -> bool {
    matches!(resolve_alias_type(ty), Type::Enum { .. })
}

pub(super) fn is_option_like_simple(ty: &Type) -> bool {
    ty.optional_member_type().is_some()
}

pub(super) fn detect_is_some_guard_name(expr: &HirExpr) -> Option<String> {
    if let HirExpr::MethodCall {
        object,
        method,
        args,
        ..
    } = expr
    {
        if method != "is_some" || !args.is_empty() {
            return None;
        }
        let HirExpr::Name { name, .. } = object.as_ref() else {
            return None;
        };
        return Some(name.clone());
    }
    let HirExpr::Compare {
        left,
        ops,
        comparators,
        ..
    } = expr
    else {
        return None;
    };
    if ops.len() != 1 || comparators.len() != 1 || !matches!(ops[0].as_str(), "is not" | "!=") {
        return None;
    }
    let rhs = comparators.first()?;
    match (left.as_ref(), rhs) {
        (HirExpr::Name { name, .. }, HirExpr::NoneLiteral)
        | (HirExpr::NoneLiteral, HirExpr::Name { name, .. }) => Some(name.clone()),
        _ => None,
    }
}

pub(super) fn normalize_compare_op(op: &str) -> &str {
    match op {
        "is" => "==",
        "is not" => "!=",
        _ => op,
    }
}

pub(super) fn normalize_binop_op(op: &str) -> &str {
    match op {
        "//" => "/",
        _ => op,
    }
}

pub(super) fn is_mixed_simple_float_binop(
    op: &str,
    left_ty: &Type,
    right_ty: &Type,
    result_ty: &Type,
) -> bool {
    if !matches!(op, "/" | "+" | "-" | "*" | "%") {
        return false;
    }
    if !is_float_like_simple(result_ty) {
        return false;
    }
    (is_int_like_simple(left_ty) && is_float_like_simple(right_ty))
        || (is_float_like_simple(left_ty) && is_int_like_simple(right_ty))
}

pub(super) fn is_mixed_simple_float_floor_division_binop(
    op: &str,
    left_ty: &Type,
    right_ty: &Type,
    result_ty: &Type,
) -> bool {
    op == "//"
        && is_float_like_simple(result_ty)
        && ((is_int_like_simple(left_ty) && is_float_like_simple(right_ty))
            || (is_float_like_simple(left_ty) && is_int_like_simple(right_ty)))
}

pub(super) fn is_simple_int_true_division_binop(
    op: &str,
    left_ty: &Type,
    right_ty: &Type,
    result_ty: &Type,
) -> bool {
    op == "/"
        && is_float_like_simple(result_ty)
        && is_int_like_simple(left_ty)
        && is_int_like_simple(right_ty)
}

pub(super) fn is_promoted_fixed_width_integer_binop(
    op: &str,
    left_ty: &Type,
    right_ty: &Type,
    result_ty: &Type,
) -> bool {
    matches!(op, "+" | "-" | "*")
        && is_int_like_simple(result_ty)
        && (is_fixed_width_int_like_simple(left_ty) || is_fixed_width_int_like_simple(right_ty))
        && (is_int_like_simple(left_ty) || is_fixed_width_int_like_simple(left_ty))
        && (is_int_like_simple(right_ty) || is_fixed_width_int_like_simple(right_ty))
}

pub(super) fn is_safe_simple_compare(op: &str, left_ty: &Type, right_ty: &Type) -> bool {
    if !matches!(op, "==" | "!=" | "<" | "<=" | ">" | ">=") {
        return false;
    }
    let left_unaliased = resolve_alias_type(left_ty);
    let right_unaliased = resolve_alias_type(right_ty);
    if left_unaliased == right_unaliased && matches!(left_unaliased, Type::TypeVar(_)) {
        return true;
    }
    if left_unaliased == right_unaliased && matches!(left_unaliased, Type::Enum { .. }) {
        return matches!(op, "==" | "!=");
    }
    let left_norm = normalize_simple_compare_scalar_type(left_ty);
    let right_norm = normalize_simple_compare_scalar_type(right_ty);
    left_norm.is_some() && left_norm == right_norm
}

pub(super) fn is_safe_simple_binop(
    op: &str,
    left_ty: &Type,
    right_ty: &Type,
    result_ty: &Type,
) -> bool {
    if op == "//" {
        if is_mixed_simple_float_floor_division_binop(op, left_ty, right_ty, result_ty) {
            return true;
        }
        return is_same_simple_numeric_kind(left_ty, right_ty)
            && is_same_simple_numeric_kind(left_ty, result_ty)
            && (is_int_like_simple(left_ty) || is_float_like_simple(left_ty));
    }
    if op == "/" {
        if is_mixed_simple_float_binop(op, left_ty, right_ty, result_ty)
            || is_simple_int_true_division_binop(op, left_ty, right_ty, result_ty)
        {
            return true;
        }
        return is_same_simple_numeric_kind(left_ty, right_ty)
            && is_same_simple_numeric_kind(left_ty, result_ty)
            && is_float_like_simple(left_ty);
    }
    if matches!(op, "+" | "-" | "*" | "%")
        && is_mixed_simple_float_binop(op, left_ty, right_ty, result_ty)
    {
        return true;
    }
    if is_promoted_fixed_width_integer_binop(op, left_ty, right_ty, result_ty) {
        return true;
    }
    if matches!(op, "&" | "|" | "^" | "<<" | ">>") {
        return is_same_simple_numeric_kind(left_ty, right_ty)
            && is_same_simple_numeric_kind(left_ty, result_ty)
            && is_int_like_simple(left_ty);
    }
    if !matches!(op, "+" | "-" | "*" | "%") {
        return false;
    }
    is_same_simple_numeric_kind(left_ty, right_ty)
        && is_same_simple_numeric_kind(left_ty, result_ty)
        && is_numeric_simple(left_ty)
}

pub(super) fn is_same_simple_numeric_kind(left: &Type, right: &Type) -> bool {
    let Some(left_kind) = normalize_simple_numeric_scalar_type(left) else {
        return false;
    };
    normalize_simple_numeric_scalar_type(right).is_some_and(|right_kind| right_kind == left_kind)
}

pub(super) fn try_lower_option_none_compare_expr(
    left: &HirExpr,
    op: &str,
    right: &HirExpr,
) -> Option<RustExpr> {
    let name_expr = if matches!(right, HirExpr::NoneLiteral) {
        left
    } else if matches!(left, HirExpr::NoneLiteral) {
        right
    } else {
        return None;
    };
    let HirExpr::Name { name, ty, .. } = name_expr else {
        return None;
    };
    if !is_option_like_simple(ty) {
        return None;
    }
    let method = match op {
        "is" | "==" => "is_none",
        "is not" | "!=" => "is_some",
        _ => return None,
    };
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(name.clone())),
        method: method.to_string(),
        args: vec![],
    })
}

pub(super) fn try_lower_none_identity_compare_expr(
    left: &HirExpr,
    op: &str,
    right: &HirExpr,
) -> Option<RustExpr> {
    if !matches!(op, "is" | "is not" | "==" | "!=") {
        return None;
    }
    let is_equal_op = matches!(op, "is" | "==");
    let other = if matches!(right, HirExpr::NoneLiteral) {
        left
    } else if matches!(left, HirExpr::NoneLiteral) {
        right
    } else {
        return None;
    };
    if !matches!(other, HirExpr::Name { .. } | HirExpr::NoneLiteral)
        && !is_option_like_simple(other.ty())
    {
        // The structured path preserves effects before folding a typed comparison.
        return None;
    }
    if matches!(other, HirExpr::NoneLiteral) || matches!(resolve_alias_type(other.ty()), Type::None)
    {
        return Some(RustExpr::Literal(RustLiteral::Bool(is_equal_op)));
    }
    if is_option_like_simple(other.ty()) {
        return Some(RustExpr::MethodCall {
            receiver: Box::new(try_lower_simple_compare_operand_expr(other)?),
            method: if is_equal_op { "is_none" } else { "is_some" }.to_string(),
            args: vec![],
        });
    }
    if !matches!(
        resolve_alias_type(other.ty()),
        Type::Any | Type::Unknown | Type::TypeVar(_)
    ) {
        return Some(RustExpr::Literal(RustLiteral::Bool(!is_equal_op)));
    }
    None
}

pub(super) fn try_lower_guarded_option_compare_expr(
    expr: &HirExpr,
    guarded_name: &str,
) -> Option<RustExpr> {
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
    let normalized_op = normalize_compare_op(&ops[0]);
    if !matches!(normalized_op, "==" | "!=") {
        return None;
    }
    let rhs_expr = comparators.first()?;
    let (option_side, other_side, option_is_left) = match (left.as_ref(), rhs_expr) {
        (HirExpr::Name { name, .. }, other) if name == guarded_name => (left.as_ref(), other, true),
        (other, HirExpr::Name { name, .. }) if name == guarded_name => (rhs_expr, other, false),
        _ => return None,
    };
    if !crate::helpers::is_option_type(option_side.ty()) {
        return None;
    }
    if matches!(other_side, HirExpr::NoneLiteral) {
        return None;
    }
    let lowered_option = if let HirExpr::Name { name, .. } = option_side {
        RustExpr::Ident(name.clone())
    } else {
        try_lower_simple_compare_operand_expr(option_side)?
    };
    let mut lowered_other = try_lower_simple_compare_operand_expr(other_side)?;
    if !crate::helpers::is_copy_type_for_codegen(other_side.ty()) {
        lowered_other = crate::ownership_plan::materialize_owned_value(
            other_side.ty(),
            RustExpr::Paren(Box::new(lowered_other)),
        );
    }
    let lowered_some = RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
        args: vec![lowered_other],
    };
    let (lowered_left, lowered_right) = if option_is_left {
        (lowered_option, lowered_some)
    } else {
        (lowered_some, lowered_option)
    };
    Some(RustExpr::BinOp {
        left: Box::new(lowered_left),
        op: normalized_op.to_string(),
        right: Box::new(lowered_right),
    })
}

pub(super) fn try_lower_simple_compare_operand_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let HirExpr::Name { name, ty, .. } = expr {
        if normalize_simple_compare_scalar_type(ty).is_some()
            || is_enum_like_simple(ty)
            || matches!(resolve_alias_type(ty), Type::TypeVar(_))
        {
            return Some(RustExpr::Ident(name.clone()));
        }
    }
    try_lower_leaf_expr(expr)
}

pub(super) fn normalize_simple_compare_scalar_type(ty: &Type) -> Option<&'static str> {
    match ty {
        Type::Alias { body, .. } => normalize_simple_compare_scalar_type(body),
        Type::Int | Type::LiteralInt(_) => Some("int"),
        Type::Float => Some("float"),
        Type::Bool | Type::LiteralBool(_) => Some("bool"),
        Type::Str | Type::LiteralStr(_) => Some("str"),
        _ => None,
    }
}

pub(super) fn normalize_simple_numeric_scalar_type(ty: &Type) -> Option<&'static str> {
    match ty {
        Type::Alias { body, .. } => normalize_simple_numeric_scalar_type(body),
        Type::Int | Type::LiteralInt(_) => Some("int"),
        Type::Float => Some("float"),
        _ => None,
    }
}
