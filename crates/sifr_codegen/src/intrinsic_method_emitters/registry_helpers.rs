use crate::{RustEmitter, RustExpr};
use sifr_ir::{HirExpr, HirIteratorOp};
use sifr_type_system::{FunctionType, ParamConvention, Type};

pub(super) fn registry_uses_debug_display_format(ty: &Type) -> bool {
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
        | Type::Failure(_)
        | Type::TimeoutResult(_)
        | Type::Select2(_, _)
        | Type::BlockingTask(_, _)
        | Type::JoinSet(_, _)
        | Type::Awaitable(_)
        | Type::AsyncIterator(_, _)
        | Type::AsyncGenerator(_, _)
        | Type::PythonBuffer(_)
        | Type::PythonArrow(_)
        | Type::PythonDlpackTensor(_)
        | Type::PythonDlpackStream
        | Type::Callable(..)
        | Type::AsyncCallable(..)
        | Type::Result(_, _)
        | Type::Protocol { .. }
        | Type::Any
        | Type::Unknown
        | Type::Intersection(_)
        | Type::Never
        | Type::Template(_) => true,
        Type::Alias { body, .. } => registry_uses_debug_display_format(body),
    }
}

pub(super) fn registry_option_inner_type(ty: &Type) -> Option<Type> {
    ty.optional_member_type()
}

pub(super) fn registry_is_string_like_type(ty: &Type) -> bool {
    matches!(
        crate::resolve_alias_type_for_plain_call(ty),
        Type::Str | Type::LiteralStr(_)
    )
}

pub(super) fn registry_defaultdict_alias_parts(ty: &Type) -> Option<(&str, &Type, &Type)> {
    let Type::Alias {
        name: alias_name,
        body,
        ..
    } = ty
    else {
        return None;
    };
    if !alias_name.starts_with("__sifr_defaultdict_") {
        return None;
    }
    let Type::Dict(key_ty, value_ty) = body.resolve_alias() else {
        return None;
    };
    Some((alias_name.as_str(), key_ty.as_ref(), value_ty.as_ref()))
}

pub(super) fn registry_defaultdict_default_expr(alias_name: &str) -> RustExpr {
    match alias_name {
        "__sifr_defaultdict_int" => RustExpr::Literal(crate::RustLiteral::Int(0)),
        "__sifr_defaultdict_list" => RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
            args: vec![],
        },
        "__sifr_defaultdict_set" => RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "HashSet".to_string(),
                "new".to_string(),
            ])),
            args: vec![],
        },
        _ => RustExpr::Literal(crate::RustLiteral::Unit),
    }
}

pub(super) fn registry_defaultdict_key_arg(
    index: &HirExpr,
    lowered_index: RustExpr,
    key_ty: &Type,
) -> RustExpr {
    if let HirExpr::StringLiteral(value) = index {
        RustExpr::Literal(crate::RustLiteral::Str(value.clone()))
    } else {
        let _ = key_ty;
        RustExpr::Clone(Box::new(lowered_index))
    }
}

pub(super) fn registry_callable_signature(
    expr: &HirExpr,
) -> Option<(Vec<Type>, Vec<ParamConvention>, Type)> {
    match crate::resolve_alias_type_for_plain_call(expr.ty()) {
        Type::Function(ft) => Some((
            ft.params.iter().map(|(_, ty, _)| ty.clone()).collect(),
            ft.params
                .iter()
                .map(|(_, _, convention)| *convention)
                .collect(),
            *ft.return_type.clone(),
        )),
        Type::Callable(params, conventions, return_type) => {
            Some((params.clone(), conventions.clone(), *return_type.clone()))
        }
        _ => None,
    }
}

pub(super) fn registry_iterator_op_func_name(op: &HirIteratorOp) -> &'static str {
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

pub(super) fn registry_class_method_signature<'a>(
    methods: &'a [(String, FunctionType)],
    method_name: &str,
) -> Option<&'a FunctionType> {
    methods.iter().find_map(
        |(name, ft)| {
            if name == method_name { Some(ft) } else { None }
        },
    )
}

pub(super) fn registry_class_has_next(methods: &[(String, FunctionType)]) -> bool {
    registry_class_method_signature(methods, "__next__").is_some_and(|next_ft| {
        next_ft.params.is_empty() && next_ft.return_type.optional_member_type().is_some()
    })
}

pub(super) fn registry_iter_from_next_method_expr(source_expr: RustExpr) -> RustExpr {
    let state_name = "__sifr_iter_state".to_string();
    RustExpr::Block {
        stmts: vec![crate::RustStmt::Let {
            mutable: true,
            name: state_name.clone(),
            ty: None,
            value: source_expr,
        }],
        expr: Some(Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "iter".to_string(),
                "from_fn".to_string(),
            ])),
            args: vec![RustExpr::Closure {
                params: vec![],
                body: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(state_name)),
                    method: "__next__".to_string(),
                    args: vec![],
                }),
                is_move: true,
            }],
        })),
    }
}

pub(super) fn registry_tuple_homogeneous_iter_expr(
    lowered: RustExpr,
    tuple_len: usize,
) -> Option<RustExpr> {
    if tuple_len == 0 {
        return None;
    }
    let tuple_binding = "__sifr_tuple_iter_src".to_string();
    let tuple_items = (0..tuple_len)
        .map(|index| RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Field {
                expr: Box::new(RustExpr::Ident(tuple_binding.clone())),
                field: index.to_string(),
            }),
            method: "clone".to_string(),
            args: vec![],
        })
        .collect();
    Some(RustExpr::Block {
        stmts: vec![crate::RustStmt::Let {
            mutable: false,
            name: tuple_binding,
            ty: None,
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                method: "clone".to_string(),
                args: vec![],
            },
        }],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Vec(tuple_items)),
            method: "into_iter".to_string(),
            args: vec![],
        })),
    })
}

pub(crate) fn registry_iterable_to_owned_iter_expr(
    emitter: &mut RustEmitter,
    expr: &HirExpr,
) -> Option<RustExpr> {
    registry_iterable_to_owned_iter_expr_with_hint(emitter, expr, None)
}

pub(super) fn registry_iterable_to_owned_iter_expr_with_hint(
    emitter: &mut RustEmitter,
    expr: &HirExpr,
    element_type_hint: Option<&Type>,
) -> Option<RustExpr> {
    let lowered = emitter.try_lower_registry_expr_strict(expr)?;
    registry_iterable_to_owned_iter_expr_from_lowered(expr, element_type_hint, lowered)
}

pub(crate) fn registry_iterable_to_owned_iter_expr_from_lowered(
    expr: &HirExpr,
    element_type_hint: Option<&Type>,
    lowered: RustExpr,
) -> Option<RustExpr> {
    let iter_plan =
        crate::helpers::plan_iterator_ownership_with_element_hint(expr, element_type_hint);
    let apply_copy_clone_yield = |iter_expr: RustExpr| match iter_plan.yield_mode {
        crate::helpers::YieldMode::Copy => RustExpr::MethodCall {
            receiver: Box::new(iter_expr),
            method: "copied".to_string(),
            args: vec![],
        },
        crate::helpers::YieldMode::Clone => RustExpr::MethodCall {
            receiver: Box::new(iter_expr),
            method: "cloned".to_string(),
            args: vec![],
        },
        crate::helpers::YieldMode::Move | crate::helpers::YieldMode::Borrow => iter_expr,
    };

    match crate::resolve_alias_type_for_plain_call(expr.ty()) {
        Type::List(_) | Type::Set(_) | Type::Iterable(_) => {
            Some(match iter_plan.source_access_mode {
                crate::helpers::SourceAccessMode::Consume => RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                    method: "into_iter".to_string(),
                    args: vec![],
                },
                crate::helpers::SourceAccessMode::Preserve => {
                    apply_copy_clone_yield(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                        method: "iter".to_string(),
                        args: vec![],
                    })
                }
            })
        }
        Type::Bytes => Some(match iter_plan.source_access_mode {
            crate::helpers::SourceAccessMode::Consume => RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                    method: "into_iter".to_string(),
                    args: vec![],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![crate::RustParam::Named {
                        name: "__byte".to_string(),
                        ty: crate::RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::Cast {
                        expr: Box::new(RustExpr::Ident("__byte".to_string())),
                        ty: crate::RustType::Named("u8".to_string()),
                    }),
                    is_move: false,
                }],
            },
            crate::helpers::SourceAccessMode::Preserve => RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                    method: "iter".to_string(),
                    args: vec![],
                }),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![crate::RustParam::Named {
                        name: "__byte".to_string(),
                        ty: crate::RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::Cast {
                        expr: Box::new(RustExpr::Deref(Box::new(RustExpr::Ident(
                            "__byte".to_string(),
                        )))),
                        ty: crate::RustType::Named("u8".to_string()),
                    }),
                    is_move: false,
                }],
            },
        }),
        Type::Iterator(_) => Some(lowered),
        Type::Range => Some(lowered),
        Type::Str => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                method: "chars".to_string(),
                args: vec![],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![crate::RustParam::Named {
                    name: "__sifr_char".to_string(),
                    ty: crate::RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__sifr_char".to_string())),
                    method: "to_string".to_string(),
                    args: vec![],
                }),
                is_move: false,
            }],
        }),
        Type::Dict(_, _) => Some(match iter_plan.source_access_mode {
            crate::helpers::SourceAccessMode::Consume => RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                method: "into_keys".to_string(),
                args: vec![],
            },
            crate::helpers::SourceAccessMode::Preserve => {
                apply_copy_clone_yield(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                    method: "keys".to_string(),
                    args: vec![],
                })
            }
        }),
        Type::Tuple(elems) if !elems.is_empty() && elems.iter().all(|elem| elem == &elems[0]) => {
            registry_tuple_homogeneous_iter_expr(lowered, elems.len())
        }
        Type::Class { name, methods, .. } => {
            let class_source = match iter_plan.source_access_mode {
                crate::helpers::SourceAccessMode::Preserve => RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                    method: "clone".to_string(),
                    args: vec![],
                },
                crate::helpers::SourceAccessMode::Consume => lowered,
            };
            if let Some(iter_ft) = registry_class_method_signature(methods, "__iter__") {
                if iter_ft.params.is_empty() {
                    let iter_call = RustExpr::MethodCall {
                        receiver: Box::new(class_source.clone()),
                        method: "__iter__".to_string(),
                        args: vec![],
                    };
                    if matches!(
                        iter_ft.return_type.as_ref().resolve_alias(),
                        Type::Class { name: ret_name, .. } if ret_name == name
                    ) && registry_class_has_next(methods)
                    {
                        return Some(registry_iter_from_next_method_expr(iter_call));
                    }
                    if let Type::Class {
                        methods: ret_methods,
                        ..
                    } = iter_ft.return_type.as_ref().resolve_alias()
                    {
                        if registry_class_has_next(ret_methods) {
                            return Some(registry_iter_from_next_method_expr(iter_call));
                        }
                    }
                    return Some(iter_call);
                }
            }
            if registry_class_has_next(methods) {
                Some(registry_iter_from_next_method_expr(class_source))
            } else {
                Some(class_source)
            }
        }
        _ => Some(lowered),
    }
}

pub(super) fn registry_box_iterator_expr(iter_expr: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
        args: vec![iter_expr],
    }
}

pub(super) fn registry_expr_is_vec_like(expr: &RustExpr) -> bool {
    match expr {
        RustExpr::Vec(_) => true,
        RustExpr::MethodCall { method, .. } => {
            method == "collect" || method.starts_with("collect::<Vec<")
        }
        RustExpr::Paren(inner) => registry_expr_is_vec_like(inner),
        _ => false,
    }
}

pub(crate) fn registry_iterable_to_vec_expr(
    emitter: &mut RustEmitter,
    expr: &HirExpr,
) -> Option<RustExpr> {
    registry_iterable_to_vec_expr_with_hint(emitter, expr, None)
}

pub(super) fn registry_iterable_to_vec_expr_with_hint(
    emitter: &mut RustEmitter,
    expr: &HirExpr,
    element_type_hint: Option<&Type>,
) -> Option<RustExpr> {
    if let HirExpr::IfExpr {
        condition,
        then_expr,
        else_expr,
        ..
    } = expr
    {
        return Some(RustExpr::If {
            cond: Box::new(emitter.try_lower_registry_expr_strict(condition)?),
            then_expr: Box::new(registry_iterable_to_vec_expr_with_hint(
                emitter,
                then_expr,
                element_type_hint,
            )?),
            else_expr: Some(Box::new(registry_iterable_to_vec_expr_with_hint(
                emitter,
                else_expr,
                element_type_hint,
            )?)),
        });
    }
    let mut iter_expr =
        registry_iterable_to_owned_iter_expr_with_hint(emitter, expr, element_type_hint)?;
    if matches!(
        &iter_expr,
        RustExpr::MethodCall { method, .. } if method == "iter" || method == "keys"
    ) {
        iter_expr = RustExpr::MethodCall {
            receiver: Box::new(iter_expr),
            method: "cloned".to_string(),
            args: vec![],
        };
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(iter_expr),
        method: "collect::<Vec<_>>".to_string(),
        args: vec![],
    })
}

pub(super) fn registry_iterable_to_set_expr(
    emitter: &mut RustEmitter,
    expr: &HirExpr,
) -> Option<RustExpr> {
    Some(RustExpr::MethodCall {
        receiver: Box::new(registry_iterable_to_owned_iter_expr(emitter, expr)?),
        method: "collect::<std::collections::HashSet<_>>".to_string(),
        args: vec![],
    })
}

pub(super) fn registry_dict_source_to_map_expr(
    emitter: &mut RustEmitter,
    expr: &HirExpr,
) -> Option<RustExpr> {
    let lowered = emitter.try_lower_registry_expr_strict(expr)?;
    match crate::resolve_alias_type_for_plain_call(expr.ty()) {
        Type::Dict(_, _) => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
            method: "clone".to_string(),
            args: vec![],
        }),
        Type::List(_) | Type::Set(_) => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                    method: "clone".to_string(),
                    args: vec![],
                }),
                method: "into_iter".to_string(),
                args: vec![],
            }),
            method: "collect::<HashMap<_, _>>".to_string(),
            args: vec![],
        }),
        Type::Any | Type::Unknown => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                method: "into_iter".to_string(),
                args: vec![],
            }),
            method: "collect::<HashMap<_, _>>".to_string(),
            args: vec![],
        }),
        _ => None,
    }
}

pub(super) fn registry_call_callable_with_owned_args(
    emitter: &mut RustEmitter,
    callable: &HirExpr,
    arg_bindings: &[(String, Type)],
) -> Option<RustExpr> {
    let callable_expr = emitter.try_lower_registry_expr_strict(callable)?;
    let (param_types, conventions, _) = registry_callable_signature(callable)?;
    if param_types.len() != arg_bindings.len() {
        return None;
    }
    let mut lowered_args = Vec::with_capacity(arg_bindings.len());
    for (((name, arg_ty), param_ty), convention) in arg_bindings
        .iter()
        .zip(param_types.iter())
        .zip(conventions.iter())
    {
        let mut lowered_arg = RustExpr::Ident(name.clone());
        let arg_is_option = crate::helpers::is_option_type(arg_ty);
        let param_is_option = crate::helpers::is_option_type(param_ty);
        let adapted_arg =
            emitter.consuming_value_conversion_for_ir(param_ty, arg_ty, lowered_arg.clone());
        let option_value_adapted = adapted_arg != lowered_arg;
        lowered_arg = adapted_arg;
        if param_is_option && !arg_is_option {
            lowered_arg = RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                args: vec![lowered_arg],
            };
        } else if !param_is_option && arg_is_option && !option_value_adapted {
            return None;
        }

        if matches!(
            crate::sifr_type_to_rust_type(param_ty),
            crate::RustType::Boxed(_)
        ) && !matches!(&lowered_arg, RustExpr::FnCall { func, .. } if registry_is_box_new_ctor(func.as_ref()))
        {
            lowered_arg = RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                args: vec![lowered_arg],
            };
        }

        lowered_arg = if convention.is_borrowed() {
            RustExpr::Ref {
                mutable: convention.is_mutable(),
                expr: Box::new(lowered_arg),
            }
        } else {
            lowered_arg
        };
        lowered_args.push(lowered_arg);
    }
    Some(RustExpr::FnCall {
        func: Box::new(callable_expr),
        args: lowered_args,
    })
}

/// Calls a callable from a comparator whose bindings are already shared
/// references. Shared-borrow parameters receive the reference directly;
/// owned parameters require an explicit clone of the referent. Mutable-borrow
/// parameters cannot be satisfied from an immutable sort comparator.
pub(super) fn registry_call_callable_with_shared_ref_args(
    emitter: &mut RustEmitter,
    callable: &HirExpr,
    arg_bindings: &[(String, Type)],
) -> Option<RustExpr> {
    let callable_expr = emitter.try_lower_registry_expr_strict(callable)?;
    let (param_types, conventions, _) = registry_callable_signature(callable)?;
    if param_types.len() != arg_bindings.len() {
        return None;
    }
    let lowered_args = arg_bindings
        .iter()
        .zip(param_types.iter())
        .zip(conventions.iter())
        .map(|(((name, arg_ty), param_ty), convention)| {
            if convention.is_mut_borrow() || arg_ty != param_ty {
                return None;
            }
            let reference = RustExpr::Ident(name.clone());
            if convention.is_owned() {
                Some(RustExpr::MethodCall {
                    receiver: Box::new(reference),
                    method: "clone".to_string(),
                    args: vec![],
                })
            } else {
                Some(reference)
            }
        })
        .collect::<Option<Vec<_>>>()?;
    Some(RustExpr::FnCall {
        func: Box::new(callable_expr),
        args: lowered_args,
    })
}

pub(super) fn registry_nested_zip_field_expr(
    base: RustExpr,
    total: usize,
    index: usize,
) -> RustExpr {
    if total == 1 {
        return base;
    }
    let left_count = total - 1;
    if index < left_count {
        registry_nested_zip_field_expr(
            RustExpr::Field {
                expr: Box::new(base),
                field: "0".to_string(),
            },
            left_count,
            index,
        )
    } else {
        RustExpr::Field {
            expr: Box::new(base),
            field: "1".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_callable_argument_uses_union_representation_conversion() {
        let source = sifr_type_system::make_union(vec![Type::Str, Type::None]);
        let target = sifr_type_system::make_union(vec![Type::Int, Type::Str, Type::None]);
        let callable = HirExpr::Name {
            name: "handler".to_string(),
            binding_id: None,
            ty: Type::Callable(
                vec![target.clone()],
                vec![ParamConvention::own()],
                Box::new(Type::Bool),
            ),
        };
        let mut emitter = RustEmitter::new();

        let lowered = registry_call_callable_with_owned_args(
            &mut emitter,
            &callable,
            &[("value".to_string(), source)],
        )
        .expect("owned callable argument should lower");
        let rendered = crate::render_expr(&lowered);

        assert!(rendered.contains(").map("), "{rendered}");
        assert!(rendered.contains(".unwrap_or("), "{rendered}");
        assert!(rendered.contains(&target.union_enum_name()), "{rendered}");
        assert!(
            !rendered.contains("compiler-verified callable argument should be Some"),
            "{rendered}"
        );
    }

    #[test]
    fn owned_callable_argument_rejects_unadapted_optional_input() {
        let source = sifr_type_system::make_union(vec![Type::Int, Type::None]);
        let callable = HirExpr::Name {
            name: "handler".to_string(),
            binding_id: None,
            ty: Type::Callable(
                vec![Type::Int],
                vec![ParamConvention::own()],
                Box::new(Type::Bool),
            ),
        };
        let mut emitter = RustEmitter::new();

        assert!(
            registry_call_callable_with_owned_args(
                &mut emitter,
                &callable,
                &[("value".to_string(), source)],
            )
            .is_none()
        );
    }

    #[test]
    fn owned_callable_argument_preserves_class_upcasts() {
        let parent = Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "Parent".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        };
        let child = Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "Child".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: Some("Parent".to_string()),
        };
        let callable = HirExpr::Name {
            name: "handler".to_string(),
            binding_id: None,
            ty: Type::Callable(
                vec![parent],
                vec![ParamConvention::own()],
                Box::new(Type::Bool),
            ),
        };
        let mut emitter = RustEmitter::new();

        let lowered = registry_call_callable_with_owned_args(
            &mut emitter,
            &callable,
            &[("value".to_string(), child)],
        )
        .expect("class upcast should lower");
        let rendered = crate::render_expr(&lowered);

        assert!(
            rendered.contains("::std::convert::Into::<Parent>::into(value)"),
            "{rendered}"
        );
    }
}

pub(super) fn registry_zip_iter_expr(
    emitter: &mut RustEmitter,
    args: &[HirExpr],
) -> Option<RustExpr> {
    let mut iter = args.iter();
    let mut acc = registry_iterable_to_owned_iter_expr(emitter, iter.next()?)?;
    for arg in iter {
        let next_iter = registry_iterable_to_owned_iter_expr(emitter, arg)?;
        acc = RustExpr::MethodCall {
            receiver: Box::new(acc),
            method: "zip".to_string(),
            args: vec![next_iter],
        };
    }
    Some(acc)
}

pub(super) fn registry_can_construct_error_from_message(ty_name: &str) -> bool {
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
            | "TimeoutError"
    )
}

pub(super) fn registry_is_box_new_ctor(expr: &RustExpr) -> bool {
    matches!(expr, RustExpr::Path(path) if path.len() == 2 && path[0] == "Box" && path[1] == "new")
        || matches!(expr, RustExpr::Ident(name) if name == "Box::new")
}
