use super::{HirExpr, HirIteratorOp, RustExpr, RustStmt, Type};

pub(crate) fn canonical_constructor_class_name(class_name: &str) -> &str {
    class_name
}

pub(crate) fn canonical_plain_call_name_for_ir(func: &str) -> &str {
    func
}

pub(crate) fn supports_nonempty_pop_narrowing_type_for_ir(object_ty: &Type) -> bool {
    match crate::resolve_alias_type_for_plain_call(object_ty) {
        Type::List(_) => true,
        Type::Class { name, .. } => is_deque_class_name_for_ir(name),
        _ => false,
    }
}

pub(crate) fn is_deque_class_name_for_ir(name: &str) -> bool {
    name == "deque"
        || name
            .rsplit_once('.')
            .is_some_and(|(_, tail)| tail == "deque")
}

pub(crate) fn is_narrowable_pop_call_for_ir(method: &str, args: &[HirExpr]) -> bool {
    match method {
        "pop" => matches!(args, [] | [HirExpr::IntLiteral(0)]),
        "popleft" => args.is_empty(),
        _ => false,
    }
}

pub(crate) fn unwrap_compiler_verified_nonempty_pop_result_for_ir(
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

pub(crate) fn iterator_call_func_name(op: &HirIteratorOp) -> &'static str {
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

pub(crate) fn call_expr_parts(expr: &HirExpr) -> Option<(&str, &[HirExpr])> {
    match expr {
        HirExpr::Call { func, args, .. } | HirExpr::PythonCall { func, args, .. } => {
            Some((func.as_str(), args.as_slice()))
        }
        HirExpr::IteratorCall { op, args, .. } => {
            Some((iterator_call_func_name(op), args.as_slice()))
        }
        _ => None,
    }
}

pub(crate) fn should_omit_local_type_annotation(ty: &Type, value: &HirExpr) -> bool {
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
            && alias_name.starts_with("__sifr_defaultdict_") =>
        {
            let Type::Dict(key_ty, value_ty) = body.resolve_alias() else {
                return false;
            };
            matches!(key_ty.as_ref(), Type::Any | Type::Unknown)
                || matches!(value_ty.as_ref(), Type::List(elem) if matches!(elem.as_ref(), Type::Any | Type::Unknown))
                || matches!(value_ty.as_ref(), Type::Set(elem) if matches!(elem.as_ref(), Type::Any | Type::Unknown))
        }
        (_, HirExpr::MethodCall { method, args, .. })
            if method == "get"
                && args.len() == 2
                && matches!(&args[1], HirExpr::ListLiteral { elements, .. } if elements.is_empty()) =>
        {
            true
        }
        _ => false,
    }
}

pub(crate) fn should_force_mutable_binding(ty: &Type) -> bool {
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

    fn class_has_recursive_option_field(ty: &Type) -> bool {
        let Type::Class { name, fields, .. } = ty.resolve_alias() else {
            return false;
        };
        fields.iter().any(|(_, field_ty)| {
            let Type::Union(members) = field_ty.resolve_alias() else {
                return false;
            };
            members.iter().any(|member| {
                matches!(member.resolve_alias(), Type::Class { name: field_name, .. } if field_name == name)
            })
        })
    }

    matches!(
        ty,
        Type::Alias { name: alias_name, .. } if alias_name.starts_with("__sifr_defaultdict_")
    ) || matches!(ty.resolve_alias(), Type::Iterator(_))
        || matches!(ty.resolve_alias(), Type::JoinSet(_, _))
        || class_has_next_protocol(ty)
        || class_has_recursive_option_field(ty)
}

pub(crate) fn type_contains_any_or_unknown(ty: &Type) -> bool {
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
