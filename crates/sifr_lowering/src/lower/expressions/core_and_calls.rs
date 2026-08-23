use super::LowerCtx;
use super::async_await::lower_await;
use super::builtin_calls::{lower_bytes_type_factory_call, lower_defaultdict_constructor_call};
use super::container_literal_diagnostics::{
    container_literal_type_conflict, reject_unhashable_container_type,
};
use super::expression_diagnostics;
use super::expression_operators::{lower_binop, lower_compare, lower_unaryop};
use super::fstring_support::lower_fstring_expr;
use super::literals::{lower_bytes_literal, lower_number_literal};
use super::name_diagnostics;
use super::ownership_diagnostics;
use super::sequence_guard_detection::{
    detect_false_exit_sequence_guards, detect_true_sequence_guards,
};
use super::subscript_type::resolve_subscript_result_type;
use super::task_calls::{TaskCallLowering, lower_task_module_call};
pub(in crate::lower) use super::tuple_unpack::{
    lower_star_unpack_assign, lower_tuple_unpack_assign,
};
use super::{
    CallLowering, lower_dict_comp, lower_generator_expr, lower_lambda, lower_list_comp,
    lower_method_call, lower_named_expr, lower_regular_call, lower_set_comp,
    lower_shadowable_builtin_call, lower_unshadowed_builtin_call,
};
use crate::hir_nodes::HirExpr;
use crate::lower::parallel_calls;
use crate::lower::python_interop::reject_python_context_borrow_created_value;
use crate::lower::task_join_set_calls::{
    JoinSetConstructorLowering, lower_task_join_set_constructor,
};
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{
    BoolOp, Expr, ExprAttribute, ExprBoolOp, ExprCall, ExprDict, ExprList, ExprName, ExprSet,
    ExprSubscript, ExprTuple,
};
use sifr_type_system::{FunctionType, ParamConvention, Type, make_union, type_check_bool_op};
pub(in crate::lower) fn lower_expr(expr: &Expr, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let lowered = match expr {
        Expr::NumberLiteral(num) => lower_number_literal(num),
        Expr::BytesLiteral(bytes) => Some(lower_bytes_literal(bytes)),
        Expr::StringLiteral(s) => {
            let value = s.value.to_str().to_string();
            Some(HirExpr::StringLiteral(value))
        }
        Expr::BooleanLiteral(b) => Some(HirExpr::BoolLiteral(b.value)),
        Expr::NoneLiteral(_) => Some(HirExpr::NoneLiteral),
        Expr::EllipsisLiteral(ellipsis) => {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
                "ellipsis is only supported as the complete body of a Rust interop declaration"
                    .to_string(),
                ellipsis.range(),
            );
            None
        }
        Expr::Name(name) => lower_name(name, ctx),
        Expr::BinOp(binop) => lower_binop(binop, ctx),
        Expr::UnaryOp(unary) => lower_unaryop(unary, ctx),
        Expr::Compare(cmp) => lower_compare(cmp, ctx),
        Expr::BoolOp(boolop) => lower_boolop(boolop, ctx),
        Expr::Call(call) => lower_call(call, ctx),
        Expr::Await(await_expr) => lower_await(await_expr, ctx),
        Expr::If(if_expr) => super::if_expression::lower_if_expr(if_expr, ctx),
        Expr::List(list) => lower_list_literal(list, ctx),
        Expr::Set(set) => lower_set_literal(set, ctx),
        Expr::Dict(dict) => lower_dict_literal(dict, ctx),
        Expr::Tuple(tuple) => lower_tuple_literal(tuple, ctx),
        Expr::Subscript(sub) => lower_subscript(sub, ctx),
        Expr::Attribute(attr) => lower_attribute(attr, ctx),
        Expr::FString(fstring) => lower_fstring_expr(fstring, ctx),
        Expr::Named(named) => lower_named_expr(named, ctx),
        Expr::Lambda(lambda) => lower_lambda(lambda, ctx),
        Expr::ListComp(comp) => lower_list_comp(comp, ctx),
        Expr::SetComp(comp) => super::async_comprehensions::lower_set_comp(comp, ctx)
            .unwrap_or_else(|| lower_set_comp(comp, ctx)),
        Expr::DictComp(comp) => super::async_comprehensions::lower_dict_comp(comp, ctx)
            .unwrap_or_else(|| lower_dict_comp(comp, ctx)),
        Expr::Generator(generator) => lower_generator_expr(generator, ctx),
        Expr::YieldFrom(yield_from) if ctx.current_function_is_async_generator => {
            expression_diagnostics::unsupported_form(
                ctx,
                "async yield from is not supported in v1; use async for over the source and yield values explicitly",
                yield_from.range(),
            );
            None
        }
        _ => {
            expression_diagnostics::unsupported_form(
                ctx,
                "unsupported expression type",
                expr.range(),
            );
            None
        }
    }?;
    reject_python_context_borrow_created_value(&lowered, expr.range(), ctx);
    Some(lowered)
}
pub(in crate::lower) fn callable_signature(
    expr: &HirExpr,
) -> Option<(Vec<Type>, Vec<ParamConvention>, Type)> {
    match expr.ty().resolve_alias() {
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
        Type::Class { methods, .. } | Type::Protocol { methods, .. } => methods
            .iter()
            .find(|(name, _)| name == "__call__")
            .map(|(_, ft)| {
                (
                    ft.params.iter().map(|(_, ty, _)| ty.clone()).collect(),
                    ft.params
                        .iter()
                        .map(|(_, _, convention)| *convention)
                        .collect(),
                    *ft.return_type.clone(),
                )
            }),
        _ => None,
    }
}
pub(super) fn canonicalize_class_surface_type(ty: &Type) -> Type {
    match ty {
        Type::List(elem) => Type::List(Box::new(canonicalize_class_surface_type(elem))),
        Type::Set(elem) => Type::Set(Box::new(canonicalize_class_surface_type(elem))),
        Type::Dict(key, value) => Type::Dict(
            Box::new(canonicalize_class_surface_type(key)),
            Box::new(canonicalize_class_surface_type(value)),
        ),
        Type::Tuple(elements) => Type::Tuple(
            elements
                .iter()
                .map(canonicalize_class_surface_type)
                .collect(),
        ),
        Type::Union(members) => make_union(
            members
                .iter()
                .map(canonicalize_class_surface_type)
                .collect(),
        ),
        Type::Result(ok, err) => Type::Result(
            Box::new(canonicalize_class_surface_type(ok)),
            Box::new(canonicalize_class_surface_type(err)),
        ),
        Type::Callable(params, conventions, ret) => Type::Callable(
            params.iter().map(canonicalize_class_surface_type).collect(),
            conventions.clone(),
            Box::new(canonicalize_class_surface_type(ret)),
        ),
        Type::AsyncCallable(params, conventions, ret) => Type::AsyncCallable(
            params.iter().map(canonicalize_class_surface_type).collect(),
            conventions.clone(),
            Box::new(canonicalize_class_surface_type(ret)),
        ),
        Type::Function(ft) => Type::Function(FunctionType {
            receiver: ft.receiver,
            params: ft
                .params
                .iter()
                .map(|(name, param_ty, convention)| {
                    (
                        name.clone(),
                        canonicalize_class_surface_type(param_ty),
                        *convention,
                    )
                })
                .collect(),
            return_type: Box::new(canonicalize_class_surface_type(&ft.return_type)),
        }),
        Type::Alias {
            name,
            type_args,
            body,
        } => Type::Alias {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(canonicalize_class_surface_type)
                .collect(),
            body: Box::new(canonicalize_class_surface_type(body)),
        },
        Type::Class { .. } | Type::Protocol { .. } => ty.clone(),
        _ => ty.clone(),
    }
}
pub(in crate::lower) fn lower_name(name: &ExprName, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let var_name = name.id.to_string();
    if ctx.compiler_intrinsics.contains_key(&var_name) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
            format!("compiler intrinsic callable '{var_name}' may only be used as a direct call"),
            name.range(),
        );
        return None;
    }
    if let Some(info) = ctx.scope.lookup(&var_name) {
        let is_moved = info.is_moved;
        let ty = info.effective_type().clone();
        let binding_id = info.binding_id;
        if is_moved {
            ownership_diagnostics::use_after_move(ctx, &var_name, name.range());
        }
        return Some(HirExpr::Name {
            name: var_name,
            binding_id: Some(binding_id),
            ty,
        });
    }
    if let Some(ft) = ctx.functions.get(&var_name) {
        let ft = ft.clone();
        let ty = if ctx.async_functions.contains(&var_name)
            && !ctx.async_generator_functions.contains(&var_name)
        {
            Type::AsyncFunction(ft)
        } else {
            Type::Function(ft)
        };
        return Some(HirExpr::Name {
            name: var_name,
            binding_id: None,
            ty,
        });
    }
    match var_name.as_str() {
        "True" => return Some(HirExpr::BoolLiteral(true)),
        "False" => return Some(HirExpr::BoolLiteral(false)),
        _ => {}
    }

    name_diagnostics::undefined_variable(ctx, &var_name, name.range());
    None
}

pub(in crate::lower) fn lower_boolop(boolop: &ExprBoolOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let op_str = match boolop.op {
        BoolOp::And => "and",
        BoolOp::Or => "or",
    };

    let saved_sequence_guards = ctx.save_sequence_guards();
    let mut values = Vec::new();
    for (index, val) in boolop.values.iter().enumerate() {
        if index > 0 {
            let prev_expr = &boolop.values[index - 1];
            let guards = match boolop.op {
                // For `a and b`, `b` is evaluated only if `a` is true.
                BoolOp::And => detect_true_sequence_guards(prev_expr, ctx),
                // For `a or b`, `b` is evaluated only if `a` is false.
                BoolOp::Or => detect_false_exit_sequence_guards(prev_expr, ctx),
            };
            for guard in guards {
                ctx.add_sequence_guard(guard);
            }
        }
        let Some(expr) = lower_expr(val, ctx) else {
            ctx.restore_sequence_guards(&saved_sequence_guards);
            return None;
        };
        values.push(expr);
    }
    ctx.restore_sequence_guards(&saved_sequence_guards);

    for (index, val) in values.iter().enumerate() {
        if let Err((code, message)) = type_check_bool_op(val.ty(), op_str, &Type::Bool) {
            ctx.error_with_code_at(code, message, boolop.values[index].range());
            return None;
        }
    }

    Some(HirExpr::BoolOp {
        op: op_str.to_string(),
        values,
        ty: Type::Bool,
    })
}

pub(super) fn first_call_keyword_range(call: &ExprCall) -> TextRange {
    call.arguments
        .keywords
        .first()
        .map_or_else(|| call.func.range(), |keyword| keyword.range)
}

pub(super) fn call_arity_range(call: &ExprCall) -> TextRange {
    call.arguments
        .args
        .last()
        .map_or_else(|| call.func.range(), Ranged::range)
}

pub(in crate::lower) fn lower_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if let Expr::Subscript(subscript) = call.func.as_ref() {
        match lower_task_join_set_constructor(subscript, call, ctx) {
            JoinSetConstructorLowering::Lowered(expr) => {
                return Some(expr);
            }
            JoinSetConstructorLowering::Rejected => return None,
            JoinSetConstructorLowering::NoMatch => {}
        }
    }
    if let Expr::Attribute(attr) = call.func.as_ref() {
        if let Some(factory_call) = lower_bytes_type_factory_call(attr, call, ctx) {
            return Some(factory_call);
        }
        match lower_task_module_call(attr, call, ctx) {
            TaskCallLowering::Lowered(expr) => return Some(expr),
            TaskCallLowering::Rejected => return None,
            TaskCallLowering::NoMatch => {}
        }
        return lower_method_call(attr, call, ctx);
    }
    let func_name = if let Expr::Name(n) = call.func.as_ref() {
        n.id.to_string()
    } else {
        expression_diagnostics::call_not_callable_or_arity(
            ctx,
            "only simple function calls are supported".to_string(),
            call.func.range(),
        );
        return None;
    };
    if func_name == "ExitCause" && !ctx.is_sysroot_private_declaration() {
        ctx.error_with_code_at(
            sifr_diagnostics::DiagnosticCode::PYCTX_INVALID_DECLARATION,
            "invalid Python context declaration: ExitCause values are compiler-constructed and cannot be created directly"
                .to_string(),
            call.range(),
        );
        return None;
    }
    if ctx.explicit_defaultdict_bindings.contains(&func_name)
        && ctx.scope.lookup(&func_name).is_none()
    {
        return lower_defaultdict_constructor_call(call, ctx);
    }
    // Handle `cls(...)` in @classmethod as constructor call for the current class
    if func_name == "cls" {
        if let Some(ref class_name) = ctx.current_class {
            let class_name = class_name.clone();
            if let Some(class_ty) = ctx.class_types.get(&class_name).cloned() {
                // Lower arguments
                let mut args = Vec::new();
                for arg in &call.arguments.args {
                    let expr = lower_expr(arg, ctx)?;
                    consume_affine_value_name(&expr, arg.range(), ctx);
                    args.push(expr);
                }
                return Some(HirExpr::ConstructorCall {
                    class_name,
                    args,
                    ty: class_ty,
                });
            }
        }
    }

    if let Some(result) = parallel_calls::lower_parallel_imported_call(&func_name, call, ctx) {
        return result;
    }

    let builtin_is_shadowed =
        ctx.scope.lookup(&func_name).is_some() || ctx.functions.contains_key(&func_name);

    if !builtin_is_shadowed {
        match lower_unshadowed_builtin_call(&func_name, call, ctx) {
            Some(CallLowering::Lowered(expr)) => return Some(expr),
            Some(CallLowering::NoMatch) => {}
            None => return None,
        }
    }

    if !builtin_is_shadowed {
        match lower_shadowable_builtin_call(&func_name, call, ctx) {
            Some(CallLowering::Lowered(expr)) => return Some(expr),
            Some(CallLowering::NoMatch) => {}
            None => return None,
        }
    }

    lower_regular_call(func_name, call, ctx)
}
pub(in crate::lower) fn lower_list_literal(list: &ExprList, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut elements = Vec::new();
    let mut elem_ty: Option<Type> = None;

    for elt in &list.elts {
        let expr = lower_expr(elt, ctx)?;
        let ty = expr.ty().clone();
        if let Some(ref expected) = elem_ty {
            if !ty.is_assignable_to(expected) {
                container_literal_type_conflict(ctx, "list element", expected, &ty, elt.range());
            }
        } else {
            elem_ty = Some(ty);
        }
        consume_affine_value_name(&expr, elt.range(), ctx);
        elements.push(expr);
    }

    let final_elem_ty = elem_ty.unwrap_or(Type::Any);
    let list_ty = Type::List(Box::new(final_elem_ty));

    Some(HirExpr::ListLiteral {
        elements,
        ty: list_ty,
    })
}
pub(in crate::lower) fn lower_set_literal(set: &ExprSet, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut elements = Vec::new();
    let mut elem_ty: Option<Type> = None;

    for elt in &set.elts {
        let expr = lower_expr(elt, ctx)?;
        let ty = expr.ty().clone();
        if let Some(ref expected) = elem_ty {
            if !ty.is_assignable_to(expected) {
                container_literal_type_conflict(ctx, "set element", expected, &ty, elt.range());
            }
        } else {
            elem_ty = Some(ty);
        }
        consume_affine_value_name(&expr, elt.range(), ctx);
        elements.push(expr);
    }

    let final_elem_ty = elem_ty.unwrap_or(Type::Any);
    if reject_unhashable_container_type(ctx, "set element", &final_elem_ty, set.range()) {
        return None;
    }
    let set_ty = Type::Set(Box::new(final_elem_ty));

    Some(HirExpr::SetLiteral {
        elements,
        ty: set_ty,
    })
}

pub(in crate::lower) fn lower_dict_literal(dict: &ExprDict, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut keys = Vec::new();
    let mut values = Vec::new();
    let mut key_ty: Option<Type> = None;
    let mut val_ty: Option<Type> = None;

    for item in &dict.items {
        if let Some(ref key_expr) = item.key {
            let key = lower_expr(key_expr, ctx)?;
            let kt = key.ty().clone();
            if let Some(ref expected) = key_ty {
                if !kt.is_assignable_to(expected) {
                    container_literal_type_conflict(
                        ctx,
                        "dict key",
                        expected,
                        &kt,
                        key_expr.range(),
                    );
                }
            } else {
                key_ty = Some(kt);
            }
            if reject_unhashable_container_type(ctx, "dict key", key.ty(), key_expr.range()) {
                return None;
            }
            keys.push(key);
        } else {
            expression_diagnostics::type_mismatch(
                ctx,
                "dict unpacking (**) not supported".to_string(),
                item.value.range(),
            );
            return None;
        }

        let val = lower_expr(&item.value, ctx)?;
        let vt = val.ty().clone();
        if let Some(ref expected) = val_ty {
            if !vt.is_assignable_to(expected) {
                container_literal_type_conflict(
                    ctx,
                    "dict value",
                    expected,
                    &vt,
                    item.value.range(),
                );
            }
        } else {
            val_ty = Some(vt);
        }
        consume_affine_value_name(&val, item.value.range(), ctx);
        values.push(val);
    }

    let final_key_ty = key_ty.unwrap_or(Type::Any);
    let final_val_ty = val_ty.unwrap_or(Type::Any);
    let dict_ty = Type::Dict(Box::new(final_key_ty), Box::new(final_val_ty));

    Some(HirExpr::DictLiteral {
        keys,
        values,
        ty: dict_ty,
    })
}

pub(in crate::lower) fn lower_tuple_literal(
    tuple: &ExprTuple,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    let mut elements = Vec::new();
    let mut elem_types = Vec::new();

    for elt in &tuple.elts {
        let expr = lower_expr(elt, ctx)?;
        elem_types.push(expr.ty().clone());
        consume_affine_value_name(&expr, elt.range(), ctx);
        elements.push(expr);
    }

    let tuple_ty = Type::Tuple(elem_types);

    Some(HirExpr::TupleLiteral {
        elements,
        ty: tuple_ty,
    })
}

pub(in crate::lower) fn consume_affine_value_name(
    expr: &HirExpr,
    range: ruff_text_size::TextRange,
    ctx: &mut LowerCtx,
) {
    if !expr.ty().contains_affine_resource() {
        return;
    }
    match expr {
        HirExpr::Name { name, .. } => {
            if ctx.borrowed_params.contains(name) {
                ownership_diagnostics::borrowed_affine_parameter_escape(ctx, name, "move", range);
            } else if ctx.scope.is_moved(name) {
                ownership_diagnostics::use_after_move(ctx, name, range);
            } else {
                ctx.mark_moved_with_flow(name);
            }
        }
        HirExpr::IfExpr {
            then_expr,
            else_expr,
            ..
        } => {
            // Only one branch moves at runtime, but both candidate bindings
            // must be unavailable afterward because the chosen path is not a
            // compile-time ownership fact.
            consume_affine_value_name(then_expr, range, ctx);
            consume_affine_value_name(else_expr, range, ctx);
        }
        HirExpr::OkWrap { value, .. }
        | HirExpr::ErrWrap { value, .. }
        | HirExpr::QuestionMark { expr: value, .. }
        | HirExpr::WalrusExpr { value, .. } => consume_affine_value_name(value, range, ctx),
        _ => {}
    }
}

pub(in crate::lower) fn consume_owned_value(
    expr: &HirExpr,
    range: ruff_text_size::TextRange,
    ctx: &mut LowerCtx,
) {
    if let HirExpr::Name { name, ty, .. } = expr {
        if ty.ownership() == sifr_type_system::OwnershipKind::Move {
            if ty.contains_affine_resource() && ctx.borrowed_params.contains(name) {
                ownership_diagnostics::borrowed_affine_parameter_escape(
                    ctx,
                    name,
                    "pass as an owned argument",
                    range,
                );
            } else if ty.contains_affine_resource() && ctx.scope.is_moved(name) {
                ownership_diagnostics::use_after_move(ctx, name, range);
            } else {
                ctx.mark_moved_with_flow(name);
            }
        }
    } else {
        consume_affine_value_name(expr, range, ctx);
    }
}

pub(in crate::lower) fn lower_subscript(
    sub: &ExprSubscript,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    let object = lower_expr(&sub.value, ctx)?;
    let object_ty = object.ty().clone();

    // Check if the slice is a Slice expression (x[start:stop] or x[start:stop:step])
    if let Expr::Slice(slice_expr) = sub.slice.as_ref() {
        let start = if let Some(ref s) = slice_expr.lower {
            Some(Box::new(lower_expr(s, ctx)?))
        } else {
            None
        };
        let stop = if let Some(ref s) = slice_expr.upper {
            Some(Box::new(lower_expr(s, ctx)?))
        } else {
            None
        };
        let step = if let Some(ref s) = slice_expr.step {
            Some(Box::new(lower_expr(s, ctx)?))
        } else {
            None
        };

        // Determine result type for slicing
        let result_ty = match &object_ty {
            Type::List(elem_ty) => Type::List(elem_ty.clone()),
            Type::Bytes => Type::Bytes,
            Type::Str => Type::Str,
            Type::Tuple(elems) => {
                // Compile-time tuple slicing: indices must be integer literals
                if let (Some(start_expr), Some(stop_expr)) = (&start, &stop) {
                    if let (HirExpr::IntLiteral(s), HirExpr::IntLiteral(e)) =
                        (start_expr.as_ref(), stop_expr.as_ref())
                    {
                        let Ok(len_i64) = i64::try_from(elems.len()) else {
                            expression_diagnostics::type_mismatch(
                                ctx,
                                "tuple too large for slicing index computation".to_string(),
                                sub.slice.range(),
                            );
                            return Some(HirExpr::Slice {
                                object: Box::new(object),
                                start,
                                stop,
                                step,
                                ty: Type::Any,
                            });
                        };
                        let normalize = |idx: i64| if idx < 0 { len_i64 + idx } else { idx };
                        let s = normalize(*s);
                        let e = normalize(*e);
                        if s <= e {
                            if let (Ok(s_usize), Ok(e_usize)) =
                                (usize::try_from(s), usize::try_from(e))
                            {
                                if e_usize <= elems.len() {
                                    Type::Tuple(elems[s_usize..e_usize].to_vec())
                                } else {
                                    expression_diagnostics::type_mismatch(
                                        ctx,
                                        "tuple slice indices out of range".to_string(),
                                        sub.slice.range(),
                                    );
                                    Type::Any
                                }
                            } else {
                                expression_diagnostics::type_mismatch(
                                    ctx,
                                    "tuple slice indices out of range".to_string(),
                                    sub.slice.range(),
                                );
                                Type::Any
                            }
                        } else {
                            expression_diagnostics::type_mismatch(
                                ctx,
                                "tuple slice indices out of range".to_string(),
                                sub.slice.range(),
                            );
                            Type::Any
                        }
                    } else {
                        expression_diagnostics::type_mismatch(
                            ctx,
                            "tuple slicing requires compile-time constant indices".to_string(),
                            sub.slice.range(),
                        );
                        Type::Any
                    }
                } else {
                    // Partial slice on tuple
                    let s = start
                        .as_ref()
                        .and_then(|e| match e.as_ref() {
                            HirExpr::IntLiteral(v) => usize::try_from(*v).ok(),
                            _ => None,
                        })
                        .unwrap_or(0);
                    let e = stop
                        .as_ref()
                        .and_then(|e| match e.as_ref() {
                            HirExpr::IntLiteral(v) => usize::try_from(*v).ok(),
                            _ => None,
                        })
                        .unwrap_or(elems.len());
                    if s <= e && e <= elems.len() {
                        Type::Tuple(elems[s..e].to_vec())
                    } else {
                        Type::Tuple(elems.clone())
                    }
                }
            }
            _ => {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!("cannot slice type '{}'", object_ty.display_name()),
                    sub.slice.range(),
                );
                Type::Any
            }
        };

        if result_ty.contains_affine_resource() {
            ctx.error_with_code_at(
                DiagnosticCode::PYZC_INVALID_DECLARATION,
                "cannot slice an aggregate containing an affine Python resource; slicing would duplicate the resource"
                    .to_string(),
                sub.range(),
            );
        }

        return Some(HirExpr::Slice {
            object: Box::new(object),
            start,
            stop,
            step,
            ty: result_ty,
        });
    }

    let index = lower_expr(&sub.slice, ctx)?;
    let index_ty = index.ty().clone();

    let result_ty = resolve_subscript_result_type(sub, &object_ty, &index, &index_ty, ctx);

    if result_ty.contains_affine_resource() {
        ctx.error_with_code_at(
            DiagnosticCode::PYZC_INVALID_DECLARATION,
            "cannot project an affine Python resource through indexing; use a consuming aggregate operation"
                .to_string(),
            sub.range(),
        );
    }

    Some(HirExpr::Index {
        object: Box::new(object),
        index: Box::new(index),
        ty: result_ty,
    })
}

pub(in crate::lower) fn lower_attribute(
    attr: &ExprAttribute,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    let field_name = attr.attr.to_string();

    // Check for enum variant access: Color.RED
    if let Expr::Name(name) = attr.value.as_ref() {
        let class_name = name.id.to_string();
        if let Some(ty) = ctx.class_types.get(&class_name).cloned() {
            if let Type::Enum { ref variants, .. } = ty {
                if variants.iter().any(|(v, _)| v == &field_name) {
                    return Some(HirExpr::EnumVariant {
                        enum_name: class_name,
                        variant: field_name,
                        ty,
                    });
                }
            }
        }
    }

    let object = lower_expr(&attr.value, ctx)?;
    let object_ty = object.ty().clone();
    let resolved_object_ty = canonicalize_class_surface_type(object_ty.resolve_alias())
        .resolve_alias()
        .clone();

    // Check if the object is a class instance with this field
    if let Type::Class {
        name: _, fields, ..
    } = &resolved_object_ty
    {
        if let Some((_, field_ty)) = fields.iter().find(|(n, _)| n == &field_name) {
            if field_ty.contains_affine_resource() {
                ctx.error_with_code_at(
                    DiagnosticCode::PYZC_INVALID_DECLARATION,
                    "cannot project a field containing an affine Python resource; move the aggregate as a whole"
                        .to_string(),
                    attr.range(),
                );
                return None;
            }
            return Some(HirExpr::FieldAccess {
                object: Box::new(object),
                field: field_name,
                ty: field_ty.clone(),
            });
        }
        ctx.error_with_code_at(
            DiagnosticCode::CLASS_MISSING_MEMBER,
            format!(
                "type '{type_name}' has no field '{field}'",
                type_name = object_ty.display_name(),
                field = field_name
            ),
            attr.attr.range(),
        );
        return None;
    }

    if let Some(field_ty) =
        super::attribute_access::optional_class_union_field_type(&resolved_object_ty, &field_name)
    {
        if field_ty.contains_affine_resource() {
            ctx.error_with_code_at(
                DiagnosticCode::PYZC_INVALID_DECLARATION,
                "cannot project a field containing an affine Python resource; move the aggregate as a whole"
                    .to_string(),
                attr.range(),
            );
            return None;
        }
        return Some(HirExpr::FieldAccess {
            object: Box::new(object),
            field: field_name,
            ty: field_ty,
        });
    }

    // Check if the object is an enum instance - access .name or .value
    if let Type::Enum {
        name: enum_name, ..
    } = &resolved_object_ty
    {
        match field_name.as_str() {
            "name" => {
                return Some(HirExpr::FieldAccess {
                    object: Box::new(object),
                    field: "name".to_string(),
                    ty: Type::Str,
                });
            }
            "value" => {
                return Some(HirExpr::FieldAccess {
                    object: Box::new(object),
                    field: "value".to_string(),
                    ty: Type::Int,
                });
            }
            _ => {
                ctx.error_with_code_at(
                    DiagnosticCode::CLASS_MISSING_MEMBER,
                    format!("enum '{enum_name}' has no attribute '{field_name}'"),
                    attr.attr.range(),
                );
                return None;
            }
        }
    }

    // Not a class field access -- report unsupported
    expression_diagnostics::unsupported_form(
        ctx,
        &format!(
            "attribute access '.{field_name}' is not supported as an expression; use as a method call"
        ),
        attr.range(),
    );
    None
}
