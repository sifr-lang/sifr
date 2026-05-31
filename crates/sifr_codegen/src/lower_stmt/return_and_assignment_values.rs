use super::{
    is_alias_equivalent_type, is_none_type, is_okwrap_none_expr, is_option_like_type,
    resolve_alias_type, try_lower_leaf_expr, try_lower_leaf_or_name_expr,
    try_lower_name_ident_expr, HashSet, HirExpr, RustExpr, RustLiteral, RustStmt,
    SimpleStmtLoweringCtx, Type,
};

pub(super) fn try_lower_simple_return_stmt(
    value: &HirExpr,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    if ctx.in_display_impl {
        return None;
    }
    if ctx.in_class_scope && matches!(value, HirExpr::Name { name, .. } if name == "self") {
        return Some(vec![RustStmt::Return(Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("self".to_string())),
            method: "clone".to_string(),
            args: vec![],
        }))]);
    }
    let option_return = ctx.return_type.is_some_and(is_option_like_type);
    if matches!(value.ty(), Type::TypeVar(_)) {
        return None;
    }
    if ctx.return_type.is_some_and(|ty| {
        matches!(
            resolve_alias_type(ty),
            Type::Iterable(_) | Type::Iterator(_)
        )
    }) {
        return None;
    }

    if option_return {
        if is_option_like_type(value.ty()) && !is_none_type(value.ty()) {
            return Some(vec![RustStmt::Return(Some(try_lower_name_ident_expr(
                value,
            )?))]);
        }
        if matches!(value, HirExpr::NoneLiteral) {
            return Some(vec![RustStmt::Return(Some(RustExpr::Literal(
                RustLiteral::None,
            )))]);
        }
        if is_none_type(value.ty()) {
            if matches!(value, HirExpr::Name { .. }) {
                return Some(vec![RustStmt::Return(Some(RustExpr::Literal(
                    RustLiteral::None,
                )))]);
            }
            return None;
        }
        let lowered = try_lower_leaf_or_name_expr(value)?;
        return Some(vec![RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![lowered],
        }))]);
    }

    if matches!(value, HirExpr::NoneLiteral)
        || is_none_type(value.ty())
        || is_okwrap_none_expr(value)
    {
        if let Some(return_ty) = ctx.return_type {
            match resolve_alias_type(return_ty) {
                Type::Result(ok_ty, _) if is_none_type(ok_ty.as_ref()) => {
                    return Some(vec![RustStmt::Return(Some(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                        args: vec![RustExpr::Literal(RustLiteral::Unit)],
                    }))]);
                }
                Type::None => return Some(vec![RustStmt::Return(None)]),
                _ => {}
            }
        }
    }

    if let Some(return_ty) = ctx.return_type {
        if let Type::Union(members) = resolve_alias_type(return_ty) {
            if is_option_like_type(value.ty()) && !matches!(value.ty(), Type::None) {
                return None;
            }
            let lowered = try_lower_leaf_or_name_expr(value)?;
            let variant = crate::helpers::find_union_variant(members, value.ty())?;
            let enum_name = return_ty.union_enum_name();
            return Some(vec![RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![enum_name, variant])),
                args: vec![lowered],
            }))]);
        }
    }
    Some(vec![RustStmt::Return(Some(try_lower_leaf_or_name_expr(
        value,
    )?))])
}

pub(super) fn try_lower_simple_let_value(ty: &Type, value: &HirExpr) -> Option<RustExpr> {
    if let Some(lowered) = crate::fixed_width_literal_expr_for_target(ty, value) {
        return Some(lowered);
    }
    if is_option_like_type(ty) && matches!(value, HirExpr::NoneLiteral) {
        return Some(RustExpr::Literal(RustLiteral::None));
    }
    if is_option_like_type(ty) && is_option_like_type(value.ty()) && !is_none_type(value.ty()) {
        return try_lower_name_ident_expr(value);
    }
    if is_option_like_type(ty) && !is_option_like_type(value.ty()) && !is_none_type(value.ty()) {
        return Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![try_lower_leaf_or_name_expr(value)?],
        });
    }
    if is_option_like_type(ty) && is_none_type(value.ty()) {
        if matches!(value, HirExpr::Name { .. }) {
            return Some(RustExpr::Literal(RustLiteral::None));
        }
        return None;
    }
    if is_none_type(ty) && matches!(value, HirExpr::NoneLiteral) {
        return Some(RustExpr::Literal(RustLiteral::Unit));
    }
    if matches!(
        crate::resolve_alias_type_for_plain_call(ty),
        Type::Task(_, _)
    ) {
        return try_lower_leaf_expr(value);
    }
    if !is_alias_equivalent_type(ty, value.ty()) {
        return None;
    }
    try_lower_leaf_or_name_expr(value)
}

pub(super) fn try_lower_simple_assign_value(
    value: &HirExpr,
    borrowed_params: &HashSet<String>,
) -> Option<RustExpr> {
    // Preserve TypeVar assignment behavior for borrowed params by appending `.clone()`.
    if matches!(value.ty(), Type::TypeVar(_))
        && matches!(value, HirExpr::Name { name, .. } if borrowed_params.contains(name))
    {
        return None;
    }
    if matches!(
        value,
        HirExpr::BinOp { op, ty, .. }
            if op == "+"
                && matches!(
                    crate::resolve_alias_type_for_plain_call(ty),
                    Type::Str | Type::LiteralStr(_)
                )
    ) {
        return None;
    }
    try_lower_leaf_or_name_expr(value)
}

pub(super) fn try_lower_simple_field_assign_stmt(
    _object: &str,
    _field: &str,
    _value: &HirExpr,
) -> Option<Vec<RustStmt>> {
    // Keep field assignments on the structured path so class/recursive storage
    // adaptations (boxing and option handling) are consistently applied.
    None
}

pub(super) fn try_lower_simple_aug_assign_value(op: &str, value: &HirExpr) -> Option<RustExpr> {
    let is_numeric_op = matches!(op, "+=" | "-=" | "*=" | "/=" | "//=" | "%=");
    let is_int_only_op = matches!(op, "&=" | "|=" | "^=" | "<<=" | ">>=");
    let supports_op = match resolve_alias_type(value.ty()) {
        Type::Int | Type::LiteralInt(_) => is_numeric_op || is_int_only_op,
        Type::Float => is_numeric_op,
        _ => false,
    };
    if !supports_op {
        return None;
    }
    try_lower_leaf_or_name_expr(value)
}

pub(super) fn try_lower_simple_augassign_stmt(
    target: RustExpr,
    op: &str,
    value: &HirExpr,
) -> Option<Vec<RustStmt>> {
    Some(vec![RustStmt::AugAssign {
        target,
        op: normalize_augassign_op(op),
        value: try_lower_simple_aug_assign_value(op, value)?,
    }])
}

pub(super) fn normalize_augassign_op(op: &str) -> String {
    if op == "//=" {
        "/".to_string()
    } else {
        op.strip_suffix('=').unwrap_or(op).to_string()
    }
}
