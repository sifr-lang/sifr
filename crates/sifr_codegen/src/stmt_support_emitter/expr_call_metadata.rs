use super::{HirExpr, HirIteratorOp, RustExpr, RustStmt, Type};

pub(crate) fn canonical_constructor_class_name(class_name: &str, ty: &Type) -> String {
    match ty.resolve_alias() {
        class @ Type::Class { .. } => {
            let rust_type = crate::sifr_type_to_rust_type(class);
            match rust_type {
                // Constructor calls rely on Rust inference for nominal type parameters. Emitting
                // the HIR's fallback `Any` arguments here can add bounds that the constructor did
                // not require and rejects otherwise valid calls such as `Channel()`.
                crate::RustType::Generic { base, .. } => base,
                other => crate::render_type(&other),
            }
        }
        _ => sifr_type_system::source_class_rust_name(class_name),
    }
}

pub(crate) fn canonical_plain_call_name_for_ir(func: &str) -> &str {
    func.split_once("::<").map_or(func, |(name, _)| name)
}

pub(crate) fn plain_call_target_for_ir(func: &str) -> RustExpr {
    if func.contains("::") {
        RustExpr::Path(func.split("::").map(str::to_string).collect())
    } else {
        RustExpr::Ident(func.to_string())
    }
}

pub(crate) fn generic_call_target_for_ir(func: &str, type_args: &[Type]) -> RustExpr {
    let type_args = type_args
        .iter()
        .map(|ty| crate::render_type(&crate::sifr_type_to_rust_type(ty)))
        .collect::<Vec<_>>()
        .join(", ");
    RustExpr::Verbatim(format!("{func}::<{type_args}>"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_call_targets_split_namespaced_functions_into_paths() {
        assert!(matches!(
            plain_call_target_for_ir("Point::origin"),
            RustExpr::Path(parts) if parts == ["Point", "origin"]
        ));
        assert!(matches!(
            plain_call_target_for_ir("compute"),
            RustExpr::Ident(name) if name == "compute"
        ));
    }

    #[test]
    fn constructor_names_leave_nominal_type_arguments_to_rust_inference() {
        let ty = Type::Class {
            identity: None,
            name: "Channel".to_string(),
            type_args: vec![Type::Any],
            fields: Default::default(),
            methods: Default::default(),
            parent_class: None,
        };

        assert_eq!(canonical_constructor_class_name("Channel", &ty), "Channel");
    }
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
    object_expr: RustExpr,
    is_deque_data_field: bool,
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
    let is_deque = is_deque_data_field
        || matches!(
            crate::resolve_alias_type_for_plain_call(object_ty),
            Type::Class { name, .. } if is_deque_class_name_for_ir(name)
        );
    let remove_from_end = method == "pop" && args.is_empty();
    if !is_deque {
        let index = if remove_from_end {
            RustExpr::BinOp {
                left: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(object_expr.clone()),
                    method: "len".to_string(),
                    args: vec![],
                }),
                op: "-".to_string(),
                right: Box::new(RustExpr::Cast {
                    expr: Box::new(RustExpr::Literal(crate::RustLiteral::Int(1))),
                    ty: crate::RustType::Named("usize".to_string()),
                }),
            }
        } else {
            RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(crate::RustLiteral::Int(0))),
                ty: crate::RustType::Named("usize".to_string()),
            }
        };
        return RustExpr::MethodCall {
            receiver: Box::new(object_expr),
            method: "remove".to_string(),
            args: vec![index],
        };
    }

    let index = if remove_from_end {
        RustExpr::BinOp {
            left: Box::new(RustExpr::MethodCall {
                receiver: Box::new(object_expr.clone()),
                method: "len".to_string(),
                args: vec![],
            }),
            op: "-".to_string(),
            right: Box::new(RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(crate::RustLiteral::Int(1))),
                ty: crate::RustType::Named("usize".to_string()),
            }),
        }
    } else {
        RustExpr::Cast {
            expr: Box::new(RustExpr::Literal(crate::RustLiteral::Int(0))),
            ty: crate::RustType::Named("usize".to_string()),
        }
    };
    RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__sifr_nonempty_pop_index".to_string(),
                ty: None,
                value: index,
            },
            RustStmt::Let {
                mutable: true,
                name: "__sifr_nonempty_pop_values".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(object_expr),
                        method: "drain".to_string(),
                        args: vec![RustExpr::Range {
                            start: Box::new(RustExpr::Ident(
                                "__sifr_nonempty_pop_index".to_string(),
                            )),
                            end: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident(
                                    "__sifr_nonempty_pop_index".to_string(),
                                )),
                                op: "+".to_string(),
                                right: Box::new(RustExpr::Cast {
                                    expr: Box::new(RustExpr::Literal(crate::RustLiteral::Int(1))),
                                    ty: crate::RustType::Named("usize".to_string()),
                                }),
                            }),
                        }],
                    }),
                    method: "collect::<Vec<_>>".to_string(),
                    args: vec![],
                },
            },
        ],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__sifr_nonempty_pop_values".to_string())),
            method: "remove".to_string(),
            args: vec![RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(crate::RustLiteral::Int(0))),
                ty: crate::RustType::Named("usize".to_string()),
            }],
        })),
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
        HirExpr::Call { func, args, .. }
        | HirExpr::GenericCall { func, args, .. }
        | HirExpr::PythonCall { func, args, .. } => Some((func.as_str(), args.as_slice())),
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

pub(crate) fn should_force_mutable_binding(
    ty: &Type,
    recursive_fields: &std::collections::HashSet<(String, String)>,
) -> bool {
    fn class_has_next_protocol(ty: &Type) -> bool {
        let Type::Class { methods, .. } = ty.resolve_alias() else {
            return false;
        };
        methods.iter().any(|(name, ft)| {
            name == "__next__"
                && ft.params.is_empty()
                && ft.return_type.optional_member_type().is_some()
        })
    }

    fn class_has_recursive_option_field(
        ty: &Type,
        recursive_fields: &std::collections::HashSet<(String, String)>,
    ) -> bool {
        let Type::Class { name, fields, .. } = ty.resolve_alias() else {
            return false;
        };
        fields.iter().any(|(field_name, field_ty)| {
            crate::helpers::is_option_type(field_ty)
                && recursive_fields.contains(&(name.clone(), field_name.clone()))
        })
    }

    matches!(
        ty,
        Type::Alias { name: alias_name, .. } if alias_name.starts_with("__sifr_defaultdict_")
    ) || matches!(ty.resolve_alias(), Type::Iterator(_))
        || matches!(ty.resolve_alias(), Type::JoinSet(_, _))
        || class_has_next_protocol(ty)
        || class_has_recursive_option_field(ty, recursive_fields)
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
        Type::Callable(params, _, ret) | Type::AsyncCallable(params, _, ret) => {
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
