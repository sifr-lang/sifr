use super::async_await::{coroutine_result_type, lower_await};
use super::async_for::async_iterator_parts;
use super::builtin_calls::{
    callable_builtin_element_type, lower_bytes_constructor_call, lower_bytes_type_factory_call,
    lower_chr_call, lower_defaultdict_constructor_call, lower_dict_constructor_call,
    lower_isinstance_call, lower_len_call, lower_list_constructor_call, lower_ord_call,
    lower_range_call, lower_reveal_type_call, lower_set_constructor_call,
    lower_tuple_constructor_call, DEFAULTDICT_INT_ALIAS, DEFAULTDICT_LIST_ALIAS,
    DEFAULTDICT_SET_ALIAS,
};
use super::bytes_methods::{resolve_bytes_method_type, resolve_str_encode_method_type};
use super::call_argument_ranges::{call_argument_ranges_by_param, type_param_argument_range};
use super::classes::is_hashable_type;
use super::compat_imports::{
    resolve_bare_python_compat_call_alias, resolve_python_compat_call_alias,
};
use super::container_literal_diagnostics::container_literal_type_conflict;
use super::decimal_methods::{
    decimal_conversion_error_type, lower_bigdecimal_constructor_call,
    lower_decimal_constructor_call, resolve_decimal_method_type,
};
use super::defaultdict_refinement::refine_defaultdict_binding_expr;
use super::diagnostics::list_append_argument_type_mismatch;
use super::empty_collection_refinement::{
    refine_empty_list_binding_expr, refine_empty_set_binding_expr,
};
use super::expression_abs::lower_abs_call;
use super::expression_diagnostics;
use super::expression_functional_builtins::{
    lower_any_all_call, lower_filter_call, lower_map_call, lower_zip_call,
};
use super::expression_iter_builtins::{lower_enumerate_call, lower_reversed_call};
use super::expression_operators::{lower_binop, lower_compare, lower_unaryop};
use super::expression_sum_sorted::{lower_sorted_call, lower_sum_call};
use super::fixed_width_arithmetic_methods::resolve_fixed_width_method_type;
use super::fstring_support::lower_fstring_expr;
use super::generic_constructor_specialization::refine_constructor_return_type_from_args;
use super::generic_receiver_specialization::refine_generic_class_binding_expr;
use super::integer_literals::canonical_large_int_literal_text;
use super::method_call_args::{
    lower_function_call_args, lower_method_call_args, lower_signature_call_args,
    resolved_method_arg_ranges, validate_dict_update_arg, validate_list_extend_arg,
    validate_set_iterable_arg,
};
use super::method_diagnostics::{
    method_count_range, reject_exact_method_arg_count, reject_max_method_arg_count,
    reject_method_arg_count, reject_no_method_args,
};
use super::min_max_validation::validate_variadic_min_max_operands;
use super::mutating_methods::{
    invalidate_collection_flow_facts_for_method, reject_immutable_parameter_method_mutation,
};
use super::name_diagnostics;
use super::nonempty_method_narrowing::refine_nonempty_method_return_type;
use super::numeric_sentinels::{
    float_sentinel_expr, float_sentinel_kind_from_call, normalize_min_max_numeric_sentinels,
};
use super::ownership_diagnostics;
use super::protocol_diagnostics;
use super::sequence_guard_detection::{
    detect_false_exit_sequence_guards, detect_true_sequence_guards,
};
use super::subscript_type::resolve_subscript_result_type;
use super::task_calls::{lower_task_module_call, TaskCallLowering};
use super::task_handle_calls::{is_task_handle_type, lower_task_handle_method_call};
use super::task_scope_calls as tsc;
pub(super) use super::tuple_unpack::{lower_star_unpack_assign, lower_tuple_unpack_assign};
use super::type_bounds::{type_satisfies_bound, type_satisfies_constraint};
use super::typevar_shape_compat::is_compatible_with_unresolved_typevars;
use super::typing_and_functions::resolve_annotation_expr;
use super::{
    collect_type_vars, decode_typevar_constraint, infer_type_var_bindings, substitute_type_vars,
    LowerCtx,
};
use crate::hir_nodes::{HirExpr, HirIteratorOp, HirParam};
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{
    BoolOp, Expr, ExprAttribute, ExprBoolOp, ExprBytesLiteral, ExprCall, ExprDict, ExprDictComp,
    ExprGenerator, ExprLambda, ExprList, ExprListComp, ExprName, ExprNamed, ExprNumberLiteral,
    ExprSet, ExprSetComp, ExprSubscript, ExprTuple, Number,
};
use sifr_type_system::{
    make_union, type_check_bool_op, FunctionType, OwnershipKind, ParamConvention, Type,
};
use std::collections::HashMap;
pub(super) fn lower_expr(expr: &Expr, ctx: &mut LowerCtx) -> Option<HirExpr> {
    match expr {
        Expr::NumberLiteral(num) => lower_number_literal(num),
        Expr::BytesLiteral(bytes) => Some(lower_bytes_literal(bytes)),
        Expr::StringLiteral(s) => {
            let value = s.value.to_str().to_string();
            Some(HirExpr::StringLiteral(value))
        }
        Expr::BooleanLiteral(b) => Some(HirExpr::BoolLiteral(b.value)),
        Expr::NoneLiteral(_) => Some(HirExpr::NoneLiteral),
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
        Expr::SetComp(comp) => lower_set_comp(comp, ctx),
        Expr::DictComp(comp) => lower_dict_comp(comp, ctx),
        Expr::Generator(gen) => lower_generator_expr(gen, ctx),
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
    }
}
pub(super) fn lower_number_literal(num: &ExprNumberLiteral) -> Option<HirExpr> {
    match &num.value {
        Number::Int(i) => {
            if let Some(val) = i.as_i64() {
                Some(HirExpr::IntLiteral(val))
            } else {
                Some(HirExpr::LargeIntLiteral(canonical_large_int_literal_text(
                    i,
                )))
            }
        }
        Number::Float(f) => Some(HirExpr::FloatLiteral(*f)),
        Number::Complex { .. } => None, // Not supported.
    }
}
pub(super) fn lower_bytes_literal(bytes: &ExprBytesLiteral) -> HirExpr {
    let mut elements = Vec::new();
    for part in &bytes.value {
        for value in part.as_slice() {
            elements.push(HirExpr::IntLiteral(i64::from(*value)));
        }
    }
    HirExpr::ListLiteral {
        elements,
        ty: Type::Bytes,
    }
}
pub(super) fn callable_signature(
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
fn canonicalize_class_surface_type(ty: &Type) -> Type {
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
        Type::Function(ft) => Type::Function(FunctionType {
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
pub(super) fn lower_name(name: &ExprName, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let var_name = name.id.to_string();
    if let Some(info) = ctx.scope.lookup(&var_name) {
        let is_moved = info.is_moved;
        let ty = info.effective_type().clone();
        if is_moved {
            ownership_diagnostics::use_after_move(ctx, &var_name, name.range());
        }
        return Some(HirExpr::Name { name: var_name, ty });
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
        return Some(HirExpr::Name { name: var_name, ty });
    }
    match var_name.as_str() {
        "True" => return Some(HirExpr::BoolLiteral(true)),
        "False" => return Some(HirExpr::BoolLiteral(false)),
        _ => {}
    }

    name_diagnostics::undefined_variable(ctx, &var_name, name.range());
    None
}

pub(super) fn lower_boolop(boolop: &ExprBoolOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
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

fn first_call_keyword_range(call: &ExprCall) -> TextRange {
    call.arguments
        .keywords
        .first()
        .map_or_else(|| call.func.range(), |keyword| keyword.range)
}

fn call_arity_range(call: &ExprCall) -> TextRange {
    call.arguments
        .args
        .last()
        .map_or_else(|| call.func.range(), Ranged::range)
}

pub(super) fn lower_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let compat_alias = resolve_python_compat_call_alias(call, ctx);
    if let (None, Expr::Attribute(attr)) = (&compat_alias, call.func.as_ref()) {
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
    let func_name = if let Some(alias) = compat_alias {
        alias
    } else if let Expr::Name(n) = call.func.as_ref() {
        resolve_bare_python_compat_call_alias(n.id.as_str(), ctx)
            .unwrap_or_else(|| n.id.to_string())
    } else {
        expression_diagnostics::call_not_callable_or_arity(
            ctx,
            "only simple function calls are supported".to_string(),
            call.func.range(),
        );
        return None;
    };
    // Handle `cls(...)` in @classmethod as constructor call for the current class
    if func_name == "cls" {
        if let Some(ref class_name) = ctx.current_class {
            let class_name = class_name.clone();
            if let Some(class_ty) = ctx.class_types.get(&class_name).cloned() {
                // Lower arguments
                let mut args = Vec::new();
                for arg in &call.arguments.args {
                    let expr = lower_expr(arg, ctx)?;
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

    let builtin_is_shadowed =
        ctx.scope.lookup(&func_name).is_some() || ctx.functions.contains_key(&func_name);

    if !builtin_is_shadowed {
        if func_name == "defaultdict" {
            return lower_defaultdict_constructor_call(call, ctx);
        }

        if func_name == "list" {
            return lower_list_constructor_call(call, ctx);
        }

        if func_name == "tuple" {
            return lower_tuple_constructor_call(call, ctx);
        }

        if func_name == "dict" {
            return lower_dict_constructor_call(call, ctx);
        }

        if func_name == "set" {
            return lower_set_constructor_call(call, ctx);
        }

        if func_name == "bytes" {
            return lower_bytes_constructor_call(call, ctx);
        }

        if func_name == "ord" {
            return lower_ord_call(call, ctx);
        }

        if func_name == "chr" {
            return lower_chr_call(call, ctx);
        }

        // Special handling for range() built-in
        if func_name == "range" {
            return lower_range_call(call, ctx);
        }

        // Special handling for len() built-in
        if func_name == "len" {
            return lower_len_call(call, ctx);
        }

        // iter(iterable) -> Iterator[T]
        if func_name == "iter" {
            if !call.arguments.keywords.is_empty() {
                expression_diagnostics::call_unexpected_keyword(
                    ctx,
                    "iter() does not accept keyword arguments".to_string(),
                    first_call_keyword_range(call),
                );
                return None;
            }
            if call.arguments.args.len() != 1 {
                expression_diagnostics::call_wrong_positional_count(
                    ctx,
                    format!(
                        "iter() takes exactly 1 argument, got {}",
                        call.arguments.args.len()
                    ),
                    call_arity_range(call),
                );
                return None;
            }
            let iterable = lower_expr(&call.arguments.args[0], ctx)?;
            if matches!(iterable.ty().resolve_alias(), Type::Any | Type::Unknown) {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "iter() argument must be an iterable with a statically-known element type, got '{}'",
                        iterable.ty().display_name()
                    ),
                    call.arguments.args[0].range(),
                );
                return None;
            }
            let Some(elem_ty) = callable_builtin_element_type(iterable.ty()) else {
                if matches!(iterable.ty().resolve_alias(), Type::Tuple(_)) {
                    ctx.error_with_code_at(
                        DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT,
                        "iter() tuple argument must have one statically provable element type"
                            .to_string(),
                        call.arguments.args[0].range(),
                    );
                    return None;
                }
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "iter() argument must be iterable, got '{}'",
                        iterable.ty().display_name()
                    ),
                    call.arguments.args[0].range(),
                );
                return None;
            };
            return Some(HirExpr::IteratorCall {
                op: HirIteratorOp::Iter,
                args: vec![iterable],
                ty: Type::Iterator(Box::new(elem_ty)),
            });
        }

        // next(iterator) -> Option[T]
        if func_name == "next" {
            if !call.arguments.keywords.is_empty() {
                expression_diagnostics::call_unexpected_keyword(
                    ctx,
                    "next() does not accept keyword arguments".to_string(),
                    first_call_keyword_range(call),
                );
                return None;
            }
            if call.arguments.args.len() != 1 {
                expression_diagnostics::call_wrong_positional_count(
                    ctx,
                    format!(
                        "next() takes exactly 1 argument, got {}",
                        call.arguments.args.len()
                    ),
                    call_arity_range(call),
                );
                return None;
            }
            let iterator = lower_expr(&call.arguments.args[0], ctx)?;
            let Some(elem_ty) = iterator.ty().iterator_element_type() else {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "next() argument must be an iterator, got '{}'",
                        iterator.ty().display_name()
                    ),
                    call.arguments.args[0].range(),
                );
                return None;
            };
            return Some(HirExpr::IteratorCall {
                op: HirIteratorOp::Next,
                args: vec![iterator],
                ty: Type::Union(vec![elem_ty, Type::None]),
            });
        }

        // anext(async_iterator) -> Awaitable[Result[Option[T], E]]
        if func_name == "anext" {
            if !call.arguments.keywords.is_empty() {
                expression_diagnostics::call_unexpected_keyword(
                    ctx,
                    "anext() does not accept keyword arguments".to_string(),
                    first_call_keyword_range(call),
                );
                return None;
            }
            if call.arguments.args.len() != 1 {
                expression_diagnostics::call_wrong_positional_count(
                    ctx,
                    format!(
                        "anext() takes exactly 1 argument, got {}",
                        call.arguments.args.len()
                    ),
                    call_arity_range(call),
                );
                return None;
            }
            let iterator = lower_expr(&call.arguments.args[0], ctx)?;
            let Some((item_ty, err_ty)) = async_iterator_parts(iterator.ty()) else {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "anext() argument must be an async iterator, got '{}'",
                        iterator.ty().display_name()
                    ),
                    call.arguments.args[0].range(),
                );
                return None;
            };
            return Some(HirExpr::Call {
                func: "anext".to_string(),
                args: vec![iterator],
                ty: Type::Awaitable(Box::new(Type::Result(
                    Box::new(Type::Union(vec![item_ty, Type::None])),
                    Box::new(err_ty),
                ))),
            });
        }

        // Special handling for isinstance() built-in
        if func_name == "isinstance" {
            return lower_isinstance_call(call, ctx);
        }

        // Special handling for reveal_type() built-in
        if func_name == "reveal_type" {
            return lower_reveal_type_call(call, ctx);
        }

        // Special handling for str() conversion
        if func_name == "str" {
            if call.arguments.args.len() == 1 {
                let arg = lower_expr(&call.arguments.args[0], ctx)?;
                return Some(HirExpr::Call {
                    func: "str".to_string(),
                    args: vec![arg],
                    ty: Type::Str,
                });
            }
        }

        // pow(base, exp) -> base ** exp
        if func_name == "pow" {
            if !call.arguments.keywords.is_empty() {
                expression_diagnostics::call_unexpected_keyword(
                    ctx,
                    "pow() does not accept keyword arguments".to_string(),
                    first_call_keyword_range(call),
                );
                return None;
            }
            if call.arguments.args.len() != 2 {
                expression_diagnostics::call_wrong_positional_count(
                    ctx,
                    "pow() takes exactly 2 arguments".to_string(),
                    call_arity_range(call),
                );
                return None;
            }
            let base = lower_expr(&call.arguments.args[0], ctx)?;
            let exp = lower_expr(&call.arguments.args[1], ctx)?;
            let result_ty = if base.ty() == &Type::Int && exp.ty() == &Type::Int {
                Type::Int
            } else {
                Type::Float
            };
            return Some(HirExpr::Call {
                func: "pow".to_string(),
                args: vec![base, exp],
                ty: result_ty,
            });
        }

        // Special handling for abs() built-in
        if func_name == "abs" {
            return lower_abs_call(call, ctx);
        }

        // Special handling for hash() built-in
        if func_name == "hash" {
            if !call.arguments.keywords.is_empty() {
                expression_diagnostics::call_unexpected_keyword(
                    ctx,
                    "hash() does not accept keyword arguments".to_string(),
                    first_call_keyword_range(call),
                );
                return None;
            }
            if call.arguments.args.len() != 1 {
                expression_diagnostics::call_wrong_positional_count(
                    ctx,
                    format!(
                        "hash() takes exactly 1 argument, got {}",
                        call.arguments.args.len()
                    ),
                    call_arity_range(call),
                );
                return None;
            }
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            let ty = arg.ty().clone();
            // Check if the type is hashable
            if !is_hashable_type(&ty) {
                let type_name = ty.display_name();
                ctx.error_with_code_at(
                    DiagnosticCode::PROTO_HASHABLE_OR_COMPARABLE_REQUIRED,
                    format!("hash() argument must be hashable, got '{type_name}'"),
                    call.arguments.args[0].range(),
                );
                return None;
            }
            return Some(HirExpr::Call {
                func: "hash".to_string(),
                args: vec![arg],
                ty: Type::Int,
            });
        }

        // Special handling for round() built-in
        if func_name == "round" {
            if !call.arguments.keywords.is_empty() {
                expression_diagnostics::call_unexpected_keyword(
                    ctx,
                    "round() does not accept keyword arguments".to_string(),
                    first_call_keyword_range(call),
                );
                return None;
            }
            if call.arguments.args.is_empty() || call.arguments.args.len() > 2 {
                expression_diagnostics::call_wrong_positional_count(
                    ctx,
                    format!(
                        "round() takes 1 or 2 arguments, got {}",
                        call.arguments.args.len()
                    ),
                    call_arity_range(call),
                );
                return None;
            }
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            if !arg.ty().is_numeric() {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "round() argument must be numeric, got '{}'",
                        arg.ty().display_name()
                    ),
                    call.arguments.args[0].range(),
                );
                return None;
            }
            if call.arguments.args.len() == 2 {
                let ndigits = lower_expr(&call.arguments.args[1], ctx)?;
                return Some(HirExpr::Call {
                    func: "round".to_string(),
                    args: vec![arg, ndigits],
                    ty: Type::Float,
                });
            }
            return Some(HirExpr::Call {
                func: "round".to_string(),
                args: vec![arg],
                ty: Type::Int,
            });
        }

        // Special handling for repr() built-in
        if func_name == "repr" {
            if !call.arguments.keywords.is_empty() {
                expression_diagnostics::call_unexpected_keyword(
                    ctx,
                    "repr() does not accept keyword arguments".to_string(),
                    first_call_keyword_range(call),
                );
                return None;
            }
            if call.arguments.args.len() != 1 {
                expression_diagnostics::call_wrong_positional_count(
                    ctx,
                    format!(
                        "repr() takes exactly 1 argument, got {}",
                        call.arguments.args.len()
                    ),
                    call_arity_range(call),
                );
                return None;
            }
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            return Some(HirExpr::Call {
                func: "repr".to_string(),
                args: vec![arg],
                ty: Type::Str,
            });
        }

        if func_name == "Decimal" {
            return lower_decimal_constructor_call(call, ctx);
        }

        if func_name == "BigDecimal" {
            return lower_bigdecimal_constructor_call(call, ctx);
        }

        // Special handling for int() conversion
        if func_name == "int" {
            if !call.arguments.keywords.is_empty() {
                expression_diagnostics::call_unexpected_keyword(
                    ctx,
                    "int() does not accept keyword arguments".to_string(),
                    first_call_keyword_range(call),
                );
                return None;
            }
            if call.arguments.args.len() != 1 {
                expression_diagnostics::call_wrong_positional_count(
                    ctx,
                    format!(
                        "int() takes exactly 1 argument, got {}",
                        call.arguments.args.len()
                    ),
                    call_arity_range(call),
                );
                return None;
            }
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            let arg_ty = arg.ty().clone();
            // int(str) -> Result[int, ParseError] (fallible)
            // int(float) -> int (infallible truncation)
            // int(int) -> int (identity)
            // int(bool) -> int (True=1, False=0)
            // int(bigint) -> Result[int, OverflowError] (may overflow i64)
            // int(decimal|bigdecimal) -> Result[int, DecimalConversionError] (truncate toward zero)
            let result_ty = if arg_ty == Type::Str {
                let parse_error_ty =
                    ctx.class_types
                        .get("ParseError")
                        .cloned()
                        .unwrap_or(Type::Class {
                            name: "ParseError".to_string(),
                            fields: vec![("message".to_string(), Type::Str)],
                            methods: vec![],
                            parent_class: None,
                        });
                Type::Result(Box::new(Type::Int), Box::new(parse_error_ty))
            } else if arg_ty == Type::BigInt {
                let overflow_error_ty =
                    ctx.class_types
                        .get("OverflowError")
                        .cloned()
                        .unwrap_or(Type::Class {
                            name: "OverflowError".to_string(),
                            fields: vec![("message".to_string(), Type::Str)],
                            methods: vec![],
                            parent_class: None,
                        });
                Type::Result(Box::new(Type::Int), Box::new(overflow_error_ty))
            } else if matches!(arg_ty, Type::Decimal | Type::BigDecimal) {
                Type::Result(
                    Box::new(Type::Int),
                    Box::new(decimal_conversion_error_type(ctx)),
                )
            } else {
                Type::Int
            };
            return Some(HirExpr::Call {
                func: "int".to_string(),
                args: vec![arg],
                ty: result_ty,
            });
        }

        // bigint(n) — convert int|bigint|decimal|bigdecimal to bigint
        if func_name == "bigint" {
            ctx.warn_bigint_transition_alias(call.func.range());
            if !call.arguments.keywords.is_empty() {
                expression_diagnostics::call_unexpected_keyword(
                    ctx,
                    "bigint() does not accept keyword arguments".to_string(),
                    first_call_keyword_range(call),
                );
                return None;
            }
            if call.arguments.args.len() != 1 {
                expression_diagnostics::call_wrong_positional_count(
                    ctx,
                    format!(
                        "bigint() takes exactly 1 argument, got {}",
                        call.arguments.args.len()
                    ),
                    call_arity_range(call),
                );
                return None;
            }
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            let arg_ty = arg.ty().clone();
            if !matches!(
                arg_ty,
                Type::Int | Type::LiteralInt(_) | Type::BigInt | Type::Decimal | Type::BigDecimal
            ) {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "bigint() requires int, bigint, decimal, or bigdecimal argument, got '{}'",
                        arg_ty.display_name()
                    ),
                    call.arguments.args[0].range(),
                );
                return None;
            }
            return Some(HirExpr::Call {
                func: "bigint".to_string(),
                args: vec![arg],
                ty: Type::BigInt,
            });
        }

        if func_name == "float" {
            if !call.arguments.keywords.is_empty() {
                expression_diagnostics::call_unexpected_keyword(
                    ctx,
                    "float() does not accept keyword arguments".to_string(),
                    first_call_keyword_range(call),
                );
                return None;
            }
            if call.arguments.args.len() != 1 {
                expression_diagnostics::call_wrong_positional_count(
                    ctx,
                    format!(
                        "float() takes exactly 1 argument, got {}",
                        call.arguments.args.len()
                    ),
                    call_arity_range(call),
                );
                return None;
            }
            if let Some(kind) = float_sentinel_kind_from_call(call) {
                return Some(float_sentinel_expr(kind));
            }
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            let arg_ty = arg.ty().clone();
            let result_ty = if arg_ty == Type::Str {
                let parse_error_ty =
                    ctx.class_types
                        .get("ParseError")
                        .cloned()
                        .unwrap_or(Type::Class {
                            name: "ParseError".to_string(),
                            fields: vec![("message".to_string(), Type::Str)],
                            methods: vec![],
                            parent_class: None,
                        });
                Type::Result(Box::new(Type::Float), Box::new(parse_error_ty))
            } else if arg_ty == Type::Decimal {
                ctx.error_with_code_at(
                    DiagnosticCode::DECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN,
                    "float(decimal_value) is not allowed; decimal values are exact and cannot be converted to float"
                        .to_string(),
                    call.arguments.args[0].range(),
                );
                return None;
            } else if arg_ty == Type::BigDecimal {
                ctx.error_with_code_at(
                    DiagnosticCode::DECIMAL_BIGDECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN,
                    "float(bigdecimal_value) is not allowed; bigdecimal values are exact and cannot be converted to float"
                        .to_string(),
                    call.arguments.args[0].range(),
                );
                return None;
            } else {
                Type::Float
            };
            return Some(HirExpr::Call {
                func: "float".to_string(),
                args: vec![arg],
                ty: result_ty,
            });
        }

        // Special handling for bool() conversion
        if func_name == "bool" {
            if !call.arguments.keywords.is_empty() {
                expression_diagnostics::call_unexpected_keyword(
                    ctx,
                    "bool() does not accept keyword arguments".to_string(),
                    first_call_keyword_range(call),
                );
                return None;
            }
            if call.arguments.args.len() != 1 {
                expression_diagnostics::call_wrong_positional_count(
                    ctx,
                    format!(
                        "bool() takes exactly 1 argument, got {}",
                        call.arguments.args.len()
                    ),
                    call_arity_range(call),
                );
                return None;
            }
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            return Some(HirExpr::Call {
                func: "bool".to_string(),
                args: vec![arg],
                ty: Type::Bool,
            });
        }

        if func_name == "min" {
            if !call.arguments.keywords.is_empty() {
                expression_diagnostics::call_unexpected_keyword(
                    ctx,
                    "min() does not accept keyword arguments".to_string(),
                    first_call_keyword_range(call),
                );
                return None;
            }
            if call.arguments.args.len() >= 2 {
                let mut args = Vec::with_capacity(call.arguments.args.len());
                for arg in &call.arguments.args {
                    args.push(lower_expr(arg, ctx)?);
                }

                let mut result_ty = args[0].ty().clone();
                for index in 1..args.len() {
                    let (left, right, pair_result_ty) = normalize_min_max_numeric_sentinels(
                        &call.arguments.args[index - 1],
                        &call.arguments.args[index],
                        args[index - 1].clone(),
                        args[index].clone(),
                        ctx,
                    );
                    args[index - 1] = left;
                    args[index] = right;
                    result_ty = pair_result_ty;
                }

                if !validate_variadic_min_max_operands("min", &args, &call.arguments.args, ctx) {
                    return None;
                }
                return Some(HirExpr::Call {
                    func: "min".to_string(),
                    args,
                    ty: result_ty,
                });
            } else if call.arguments.args.len() == 1 {
                let arg = lower_expr(&call.arguments.args[0], ctx)?;
                let Some(elem_ty) = callable_builtin_element_type(arg.ty()) else {
                    expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "min() argument must be an iterable with a statically-known element type, got '{}'",
                            arg.ty().display_name()
                        ),
                        call.arguments.args[0].range(),
                    );
                    return None;
                };
                return Some(HirExpr::Call {
                    func: "min".to_string(),
                    args: vec![arg],
                    ty: Type::Union(vec![elem_ty, Type::None]),
                });
            }
            expression_diagnostics::call_wrong_positional_count(
                ctx,
                "min() takes at least 1 argument".to_string(),
                call.func.range(),
            );
            return None;
        }
        if func_name == "max" {
            if !call.arguments.keywords.is_empty() {
                expression_diagnostics::call_unexpected_keyword(
                    ctx,
                    "max() does not accept keyword arguments".to_string(),
                    first_call_keyword_range(call),
                );
                return None;
            }
            if call.arguments.args.len() >= 2 {
                let mut args = Vec::with_capacity(call.arguments.args.len());
                for arg in &call.arguments.args {
                    args.push(lower_expr(arg, ctx)?);
                }

                let mut result_ty = args[0].ty().clone();
                for index in 1..args.len() {
                    let (left, right, pair_result_ty) = normalize_min_max_numeric_sentinels(
                        &call.arguments.args[index - 1],
                        &call.arguments.args[index],
                        args[index - 1].clone(),
                        args[index].clone(),
                        ctx,
                    );
                    args[index - 1] = left;
                    args[index] = right;
                    result_ty = pair_result_ty;
                }

                if !validate_variadic_min_max_operands("max", &args, &call.arguments.args, ctx) {
                    return None;
                }
                return Some(HirExpr::Call {
                    func: "max".to_string(),
                    args,
                    ty: result_ty,
                });
            } else if call.arguments.args.len() == 1 {
                let arg = lower_expr(&call.arguments.args[0], ctx)?;
                let Some(elem_ty) = callable_builtin_element_type(arg.ty()) else {
                    expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "max() argument must be an iterable with a statically-known element type, got '{}'",
                            arg.ty().display_name()
                        ),
                        call.arguments.args[0].range(),
                    );
                    return None;
                };
                return Some(HirExpr::Call {
                    func: "max".to_string(),
                    args: vec![arg],
                    ty: Type::Union(vec![elem_ty, Type::None]),
                });
            }
            expression_diagnostics::call_wrong_positional_count(
                ctx,
                "max() takes at least 1 argument".to_string(),
                call.func.range(),
            );
            return None;
        }
        if func_name == "sum" {
            return lower_sum_call(call, ctx);
        }
        if func_name == "sorted" {
            return lower_sorted_call(call, ctx);
        }

        // reversed(iterable) -> iterator of element type
        if func_name == "reversed" {
            return lower_reversed_call(call, ctx);
        }

        // enumerate(iterable) -> iterator of (int, element) tuples
        if func_name == "enumerate" {
            return lower_enumerate_call(call, ctx);
        }

        if func_name == "zip" {
            return lower_zip_call(call, ctx);
        }
    }

    // any(iterable) -> bool
    if func_name == "any" {
        return lower_any_all_call(call, "any", ctx);
    }

    // all(iterable) -> bool
    if func_name == "all" {
        return lower_any_all_call(call, "all", ctx);
    }

    // map(func, iterable) -> iterator
    if func_name == "map" {
        return lower_map_call(call, ctx);
    }
    if func_name == "filter" {
        return lower_filter_call(call, ctx);
    }
    if func_name == "open" {
        let n_args = call.arguments.args.len();
        let _n_kwargs = call.arguments.keywords.len();
        let path_arg = if n_args >= 1 {
            lower_expr(&call.arguments.args[0], ctx)?
        } else {
            expression_diagnostics::call_missing_required_argument(
                ctx,
                "open() requires at least 1 argument: open(path) or open(path, mode)".to_string(),
                call.func.range(),
            );
            return None;
        };
        let mode_arg = if n_args >= 2 {
            lower_expr(&call.arguments.args[1], ctx)?
        } else if let Some(kw) = call
            .arguments
            .keywords
            .iter()
            .find(|k| k.arg.as_deref() == Some("mode"))
        {
            lower_expr(&kw.value, ctx)?
        } else {
            HirExpr::StringLiteral("r".to_string())
        };
        // Return type: FileHandle (raises IOError on failure — used in try/except blocks)
        // FileHandle methods are defined in io.sifr; register them here for type checking.
        let io_err_ty = Type::Class {
            name: "IOError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: vec![],
            parent_class: None,
        };
        let file_handle_ty = Type::Class {
            name: "FileHandle".to_string(),
            fields: vec![
                ("_handle".to_string(), Type::Int),
                ("_mode".to_string(), Type::Str),
            ],
            methods: vec![
                (
                    "read".to_string(),
                    FunctionType::all_borrow(
                        vec![],
                        Type::Result(Box::new(Type::Str), Box::new(io_err_ty.clone())),
                    ),
                ),
                (
                    "write".to_string(),
                    FunctionType::all_borrow(
                        vec![("data".to_string(), Type::Str)],
                        Type::Result(Box::new(Type::None), Box::new(io_err_ty.clone())),
                    ),
                ),
                (
                    "readline".to_string(),
                    FunctionType::all_borrow(
                        vec![],
                        Type::Result(
                            Box::new(Type::Union(vec![Type::Str, Type::None])),
                            Box::new(io_err_ty.clone()),
                        ),
                    ),
                ),
                (
                    "readlines".to_string(),
                    FunctionType::all_borrow(
                        vec![],
                        Type::Result(
                            Box::new(Type::List(Box::new(Type::Str))),
                            Box::new(io_err_ty.clone()),
                        ),
                    ),
                ),
                (
                    "close".to_string(),
                    FunctionType::all_borrow(vec![], Type::None),
                ),
                (
                    "read_bytes".to_string(),
                    FunctionType::all_borrow(
                        vec![],
                        Type::Result(Box::new(Type::Bytes), Box::new(io_err_ty.clone())),
                    ),
                ),
                (
                    "write_bytes".to_string(),
                    FunctionType::all_borrow(
                        vec![("data".to_string(), Type::Bytes)],
                        Type::Result(Box::new(Type::None), Box::new(io_err_ty.clone())),
                    ),
                ),
                (
                    "__enter__".to_string(),
                    FunctionType::all_borrow(
                        vec![],
                        Type::Class {
                            name: "FileHandle".to_string(),
                            fields: vec![
                                ("_handle".to_string(), Type::Int),
                                ("_mode".to_string(), Type::Str),
                            ],
                            methods: vec![],
                            parent_class: None,
                        },
                    ),
                ),
                (
                    "__exit__".to_string(),
                    FunctionType::all_borrow(vec![], Type::None),
                ),
            ],
            parent_class: None,
        };
        // Register FileHandle in the class types so method calls work
        ctx.class_types
            .insert("FileHandle".to_string(), file_handle_ty.clone());
        // Register IOError as a possible exception from this call
        ctx.try_block_error_types.insert("IOError".to_string());
        return Some(HirExpr::Call {
            func: "builtin_open".to_string(),
            args: vec![path_arg, mode_arg],
            ty: file_handle_ty,
        });
    }

    // Check if this is a Callable-typed variable being called
    let callable_info = ctx.scope.lookup(&func_name).and_then(|info| {
        if let Type::Callable(ref param_types, ref conventions, ref ret_type) = info.ty {
            Some((param_types.clone(), conventions.clone(), *ret_type.clone()))
        } else {
            None
        }
    });
    if let Some((param_types, conventions, ret_type)) = callable_info {
        // Lower arguments
        let mut args = Vec::new();
        for arg in &call.arguments.args {
            let expr = lower_expr(arg, ctx)?;
            args.push(expr);
        }
        if args.len() != param_types.len() {
            let range = if args.len() > param_types.len() {
                call.arguments.args[param_types.len()].range()
            } else {
                call.func.range()
            };
            expression_diagnostics::call_not_callable_or_arity(
                ctx,
                format!(
                    "callable '{}' expects {} argument(s), got {}",
                    func_name,
                    param_types.len(),
                    args.len()
                ),
                range,
            );
            return None;
        }
        // Type check arguments and apply convention-aware move tracking
        for (i, (arg, param_ty)) in args.iter().zip(param_types.iter()).enumerate() {
            if !arg.ty().is_assignable_to(param_ty) {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "argument {} of callable '{}': expected '{}', got '{}'",
                        i + 1,
                        func_name,
                        param_ty.display_name(),
                        arg.ty().display_name()
                    ),
                    call.arguments.args[i].range(),
                );
            }
            // Apply move tracking based on convention
            let convention = conventions
                .get(i)
                .copied()
                .unwrap_or(ParamConvention::borrow());
            if convention.is_owned() {
                // Own convention: transfer ownership, mark variable as moved
                if let HirExpr::Name { name, ty } = arg {
                    if ty.ownership() == OwnershipKind::Move {
                        ctx.scope.mark_moved(name);
                    }
                }
            }
            // Borrow/MutBorrow: no move, variable remains usable
        }
        return Some(HirExpr::Call {
            func: func_name,
            args,
            ty: ret_type,
        });
    }

    let callable_object_ft =
        ctx.scope
            .lookup(&func_name)
            .and_then(|info| match info.effective_type().resolve_alias() {
                Type::Class { methods, .. } | Type::Protocol { methods, .. } => methods
                    .iter()
                    .find(|(name, _)| name == "__call__")
                    .map(|(_, ft)| ft.clone()),
                _ => None,
            });
    if let Some(call_ft) = callable_object_ft {
        let Expr::Name(name_expr) = call.func.as_ref() else {
            expression_diagnostics::call_not_callable_or_arity(
                ctx,
                "only simple function calls are supported".to_string(),
                call.func.range(),
            );
            return None;
        };
        let object = lower_name(name_expr, ctx)?;
        let args =
            lower_signature_call_args(call, &format!("{func_name}.__call__"), &call_ft, None, ctx)?;
        return Some(HirExpr::MethodCall {
            object: Box::new(object),
            method: "__call__".to_string(),
            args,
            ty: *call_ft.return_type.clone(),
        });
    }

    let ft = ctx.functions.get(&func_name).cloned().or_else(|| {
        name_diagnostics::undefined_function(ctx, &func_name, call.func.range());
        None
    })?;
    let is_async_function = ctx.async_functions.contains(&func_name);
    let is_async_generator_function = ctx.async_generator_functions.contains(&func_name);
    if is_async_function && !ctx.current_function_is_async {
        expression_diagnostics::type_mismatch(
            ctx,
            format!(
                "async function '{func_name}' cannot be called from sync code; call it from an async function and await the returned coroutine"
            ),
            call.func.range(),
        );
        return None;
    }
    super::workload_annotations::warn_async_direct_call(ctx, &func_name, call.func.range());
    let call_defaults = ctx.function_defaults.get(&func_name).cloned();
    let call_vararg = ctx.vararg_functions.get(&func_name).copied();

    // Resolve keyword arguments to positional order
    let args = if func_name == "print" {
        let mut args = Vec::with_capacity(call.arguments.args.len());
        for arg in &call.arguments.args {
            args.push(lower_expr(arg, ctx)?);
        }
        args
    } else if func_name.ends_with("_Counter")
        && ft.params.len() == 2
        && call.arguments.args.len() == 1
        && call.arguments.keywords.is_empty()
    {
        let lowered_arg = lower_expr(&call.arguments.args[0], ctx)?;
        let source_ty = &ft.params[0].1;
        let iterable_ty = &ft.params[1].1;
        if lowered_arg.ty().is_assignable_to(source_ty)
            || is_compatible_with_unresolved_typevars(lowered_arg.ty(), source_ty)
        {
            vec![lowered_arg, HirExpr::NoneLiteral]
        } else if lowered_arg.ty().is_assignable_to(iterable_ty)
            || is_compatible_with_unresolved_typevars(lowered_arg.ty(), iterable_ty)
        {
            vec![HirExpr::NoneLiteral, lowered_arg]
        } else if matches!(lowered_arg.ty().resolve_alias(), Type::Str) {
            let iterable_arg = HirExpr::Call {
                func: "list".to_string(),
                args: vec![lowered_arg],
                ty: Type::List(Box::new(Type::Str)),
            };
            if iterable_arg.ty().is_assignable_to(iterable_ty)
                || is_compatible_with_unresolved_typevars(iterable_arg.ty(), iterable_ty)
            {
                vec![HirExpr::NoneLiteral, iterable_arg]
            } else {
                lower_function_call_args(
                    call,
                    &func_name,
                    &ft,
                    call_defaults.as_deref(),
                    call_vararg,
                    ctx,
                )?
            }
        } else {
            lower_function_call_args(
                call,
                &func_name,
                &ft,
                call_defaults.as_deref(),
                call_vararg,
                ctx,
            )?
        }
    } else {
        lower_function_call_args(
            call,
            &func_name,
            &ft,
            call_defaults.as_deref(),
            call_vararg,
            ctx,
        )?
    };

    let arg_ranges = call_argument_ranges_by_param(call, &ft);

    // Check argument types (skip for print)
    if func_name != "print" {
        let is_generic_function = ctx.generic_functions.contains_key(&func_name);
        for (i, (arg, (param_name, param_ty, _))) in args.iter().zip(ft.params.iter()).enumerate() {
            if is_generic_function {
                let mut type_vars = Vec::new();
                collect_type_vars(param_ty, &mut type_vars);
                if !type_vars.is_empty() {
                    // Generic params are validated after binding/substitution.
                    continue;
                }
            }
            if !arg.ty().is_assignable_to(param_ty) {
                let primary_range = arg_ranges
                    .get(i)
                    .copied()
                    .flatten()
                    .unwrap_or_else(|| call.range());
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISMATCH,
                    format!(
                        "argument {} ('{}') of function '{}': expected '{}', got '{}'",
                        i + 1,
                        param_name,
                        func_name,
                        param_ty.display_name(),
                        arg.ty().display_name()
                    ),
                    primary_range,
                );
            }
        }
    }

    // Exclusivity check: enforce that the same variable is not passed as mut twice,
    // or as both mut and immutable borrow in the same call.
    {
        let mut mut_borrowed: Vec<String> = Vec::new();
        let mut immut_borrowed: Vec<String> = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            if let HirExpr::Name { name, ty } = arg {
                if ty.ownership() == sifr_type_system::OwnershipKind::Move {
                    let primary_range = arg_ranges
                        .get(i)
                        .copied()
                        .flatten()
                        .unwrap_or_else(|| call.range());
                    let convention = ft
                        .params
                        .get(i)
                        .map(|(_, _, c)| *c)
                        .unwrap_or(ParamConvention::borrow());
                    if convention.is_mut_borrow() {
                        if mut_borrowed.contains(name) {
                            ownership_diagnostics::double_mutable_borrow(
                                ctx,
                                name,
                                &func_name,
                                primary_range,
                            );
                        } else if immut_borrowed.contains(name) {
                            ownership_diagnostics::mutable_borrow_after_immutable(
                                ctx,
                                name,
                                &func_name,
                                primary_range,
                            );
                        }
                        mut_borrowed.push(name.clone());
                    } else if convention.is_shared_borrow() {
                        if mut_borrowed.contains(name) {
                            ownership_diagnostics::immutable_borrow_after_mutable(
                                ctx,
                                name,
                                &func_name,
                                primary_range,
                            );
                        }
                        immut_borrowed.push(name.clone());
                    } else {
                        // Ownership transfer, including `own mut`, does not create a borrow conflict.
                    }
                }
            }
        }
    }

    // Track ownership: only mark arguments as moved when the parameter convention is Own
    // and the argument type is Move. Borrow and MutBorrow do not consume the value.
    for (i, arg) in args.iter().enumerate() {
        if let HirExpr::Name { name, ty } = arg {
            if ty.ownership() == sifr_type_system::OwnershipKind::Move {
                let convention = ft
                    .params
                    .get(i)
                    .map(|(_, _, c)| *c)
                    .unwrap_or(ParamConvention::borrow());
                if convention.is_owned() {
                    ctx.scope.mark_moved(name);
                }
            }
        }
    }

    // If this is a generic function, infer type variable bindings and substitute
    let return_type = if ctx.generic_functions.contains_key(&func_name) {
        let mut bindings = HashMap::new();
        for (arg, (_, param_ty, _)) in args.iter().zip(ft.params.iter()) {
            infer_type_var_bindings(param_ty, arg.ty(), &mut bindings);
        }
        // Re-check argument types after TypeVar substitution so repeated type
        // parameters (e.g. assert_eq[T](a: T, b: T)) enforce consistent types.
        if func_name != "print" {
            for (i, (arg, (param_name, param_ty, _))) in
                args.iter().zip(ft.params.iter()).enumerate()
            {
                let concrete_param_ty = substitute_type_vars(param_ty, &bindings);
                let mut unresolved_type_vars = Vec::new();
                collect_type_vars(&concrete_param_ty, &mut unresolved_type_vars);
                if !unresolved_type_vars.is_empty() {
                    if !is_compatible_with_unresolved_typevars(arg.ty(), &concrete_param_ty) {
                        let primary_range = arg_ranges
                            .get(i)
                            .copied()
                            .flatten()
                            .unwrap_or_else(|| call.range());
                        ctx.error_with_code_at(
                            DiagnosticCode::TYPE_MISMATCH,
                            format!(
                                "argument {} ('{}') of function '{}': expected '{}', got '{}'",
                                i + 1,
                                param_name,
                                func_name,
                                concrete_param_ty.display_name(),
                                arg.ty().display_name()
                            ),
                            primary_range,
                        );
                    }
                    continue;
                }
                if !arg.ty().is_assignable_to(&concrete_param_ty) {
                    let primary_range = arg_ranges
                        .get(i)
                        .copied()
                        .flatten()
                        .unwrap_or_else(|| call.range());
                    ctx.error_with_code_at(
                        DiagnosticCode::TYPE_MISMATCH,
                        format!(
                            "argument {} ('{}') of function '{}': expected '{}', got '{}'",
                            i + 1,
                            param_name,
                            func_name,
                            concrete_param_ty.display_name(),
                            arg.ty().display_name()
                        ),
                        primary_range,
                    );
                }
            }
        }
        // Check protocol bounds on type parameters (scoped to this function)
        if let Some(owner_bounds) = ctx.type_param_bounds.get(&func_name) {
            let owner_bounds = owner_bounds.clone();
            for (tv_name, concrete_ty) in &bindings {
                if let Some(specs) = owner_bounds.get(tv_name) {
                    let mut required_bounds = Vec::new();
                    let mut constraints = Vec::new();
                    for spec in specs {
                        if let Some(constraint_name) = decode_typevar_constraint(spec) {
                            constraints.push(constraint_name.to_string());
                        } else {
                            required_bounds.push(spec.clone());
                        }
                    }

                    for bound in required_bounds {
                        if !type_satisfies_bound(concrete_ty, &bound, ctx) {
                            protocol_diagnostics::bound_not_satisfied(
                                ctx,
                                &concrete_ty.display_name(),
                                &bound,
                                tv_name,
                                call.range(),
                            );
                        }
                    }

                    if !constraints.is_empty()
                        && !constraints.iter().any(|constraint| {
                            type_satisfies_constraint(concrete_ty, constraint, ctx)
                        })
                    {
                        let primary_range = type_param_argument_range(call, &ft, tv_name)
                            .unwrap_or_else(|| call.range());
                        ctx.error_with_code_at(
                            DiagnosticCode::TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED,
                            format!(
                                "type '{actual}' does not satisfy constraints ({constraints}) required by type parameter '{type_param}'",
                                actual = concrete_ty.display_name(),
                                constraints = constraints.join(", "),
                                type_param = tv_name
                            ),
                            primary_range,
                        );
                    }
                }
            }
        }
        if bindings.is_empty() {
            ft.return_type.as_ref().clone()
        } else {
            substitute_type_vars(&ft.return_type, &bindings)
        }
    } else {
        ft.return_type.as_ref().clone()
    };

    let return_type = refine_constructor_return_type_from_args(&ft, &args, &return_type);
    tsc::validate_shared_constructor(&func_name, &args, &arg_ranges, call, ctx);
    let call_type = if is_async_function && !is_async_generator_function {
        coroutine_result_type(&return_type)
    } else {
        return_type
    };

    // If this is a class constructor call, emit ConstructorCall
    if ctx.class_types.contains_key(&func_name) {
        Some(HirExpr::ConstructorCall {
            class_name: func_name,
            args,
            ty: call_type,
        })
    } else {
        Some(HirExpr::Call {
            func: func_name,
            args,
            ty: call_type,
        })
    }
}

pub(super) fn lower_list_literal(list: &ExprList, ctx: &mut LowerCtx) -> Option<HirExpr> {
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
        elements.push(expr);
    }

    let final_elem_ty = elem_ty.unwrap_or(Type::Any);
    let list_ty = Type::List(Box::new(final_elem_ty));

    Some(HirExpr::ListLiteral {
        elements,
        ty: list_ty,
    })
}
pub(super) fn lower_set_literal(set: &ExprSet, ctx: &mut LowerCtx) -> Option<HirExpr> {
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
        elements.push(expr);
    }

    let final_elem_ty = elem_ty.unwrap_or(Type::Any);
    let set_ty = Type::Set(Box::new(final_elem_ty));

    Some(HirExpr::SetLiteral {
        elements,
        ty: set_ty,
    })
}

pub(super) fn lower_dict_literal(dict: &ExprDict, ctx: &mut LowerCtx) -> Option<HirExpr> {
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

pub(super) fn lower_tuple_literal(tuple: &ExprTuple, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut elements = Vec::new();
    let mut elem_types = Vec::new();

    for elt in &tuple.elts {
        let expr = lower_expr(elt, ctx)?;
        elem_types.push(expr.ty().clone());
        elements.push(expr);
    }

    let tuple_ty = Type::Tuple(elem_types);

    Some(HirExpr::TupleLiteral {
        elements,
        ty: tuple_ty,
    })
}

pub(super) fn lower_subscript(sub: &ExprSubscript, ctx: &mut LowerCtx) -> Option<HirExpr> {
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

    Some(HirExpr::Index {
        object: Box::new(object),
        index: Box::new(index),
        ty: result_ty,
    })
}

pub(super) fn lower_attribute(attr: &ExprAttribute, ctx: &mut LowerCtx) -> Option<HirExpr> {
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

pub(super) fn lower_method_call(
    attr: &ExprAttribute,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    // Handle super().__init__() and super().method() calls
    if let Expr::Call(super_call) = attr.value.as_ref() {
        if let Expr::Name(name) = super_call.func.as_ref() {
            if name.id.as_str() == "super" {
                let method_name = attr.attr.to_string();
                if let Some(parent_name) = ctx.current_parent_class.clone() {
                    // Lower arguments
                    let mut args = Vec::new();
                    for arg in &call.arguments.args {
                        let expr = lower_expr(arg, ctx)?;
                        args.push(expr);
                    }

                    return Some(HirExpr::SuperCall {
                        parent_class: parent_name,
                        method: if method_name == "__init__" {
                            "new".to_string()
                        } else {
                            method_name
                        },
                        args,
                        ty: Type::None,
                    });
                }
                ctx.error_with_code_at(
                    DiagnosticCode::CLASS_INVALID_BASE,
                    "super() used outside of a class with a parent".to_string(),
                    attr.value.range(),
                );
                return None;
            }
        }
    }

    // Handle ClassName.method() calls (classmethod/staticmethod)
    if let Expr::Name(name) = attr.value.as_ref() {
        let class_name = name.id.to_string();
        if ctx.class_types.contains_key(&class_name) {
            let method_name = attr.attr.to_string();
            // Lower arguments
            let mut args = Vec::new();
            for arg in &call.arguments.args {
                let expr = lower_expr(arg, ctx)?;
                args.push(expr);
            }
            // Look up the method's return type from the class type
            if let Some(Type::Class { methods, .. }) = ctx.class_types.get(&class_name) {
                if let Some((_, ft)) = methods.iter().find(|(n, _)| n == &method_name) {
                    let return_ty = *ft.return_type.clone();
                    return Some(HirExpr::Call {
                        func: format!("{class_name}::{method_name}"),
                        args,
                        ty: return_ty,
                    });
                }
            }
            ctx.error_with_code_at(
                DiagnosticCode::CLASS_MISSING_MEMBER,
                format!("type '{class_name}' has no class/static method '{method_name}'"),
                attr.attr.range(),
            );
            return None;
        }
    }

    let mut object = lower_expr(&attr.value, ctx)?;
    let method_name = attr.attr.to_string();
    if let Some(result) = super::blocking_executor_calls::lower_thread_pool_submit_call(
        &object,
        &method_name,
        call,
        ctx,
    ) {
        return result;
    }
    if tsc::is_task_scope_type(object.ty()) && method_name == "spawn" {
        return tsc::lower_task_scope_spawn_call(object, attr, call, ctx);
    }
    if is_task_handle_type(object.ty()) {
        if let Some(expr) = lower_task_handle_method_call(object.clone(), &method_name, call, ctx) {
            return Some(expr);
        }
    }
    let object_ty_for_args = canonicalize_class_surface_type(object.ty().resolve_alias());
    let args = match &object_ty_for_args {
        Type::Class { name, methods, .. } => {
            if let Some((_, ft)) = methods
                .iter()
                .find(|(candidate, _)| candidate == &method_name)
            {
                let ft = ft.clone();
                let defaults_key = format!("{name}.{method_name}");
                let method_defaults = ctx.function_defaults.get(&defaults_key).cloned();
                lower_signature_call_args(
                    call,
                    &format!("{name}.{method_name}"),
                    &ft,
                    method_defaults.as_deref(),
                    ctx,
                )?
            } else {
                lower_method_call_args(object.ty(), &method_name, call, ctx)?
            }
        }
        Type::Protocol { name, methods, .. } => {
            if let Some((_, ft)) = methods
                .iter()
                .find(|(candidate, _)| candidate == &method_name)
            {
                let ft = ft.clone();
                lower_signature_call_args(call, &format!("{name}.{method_name}"), &ft, None, ctx)?
            } else {
                lower_method_call_args(object.ty(), &method_name, call, ctx)?
            }
        }
        _ => lower_method_call_args(object.ty(), &method_name, call, ctx)?,
    };

    if matches!(method_name.as_str(), "append" | "insert" | "extend") {
        object = refine_empty_list_binding_expr(object, &method_name, &args, ctx);
    }
    if matches!(
        method_name.as_str(),
        "add" | "remove" | "discard" | "contains"
    ) {
        if let Some(first_arg_ty) = args.first().map(|arg| arg.ty().clone()) {
            object = refine_empty_set_binding_expr(object, first_arg_ty, ctx);
        }
    }
    if let Some(refined_object) =
        refine_defaultdict_binding_expr(object.clone(), &method_name, &args, ctx)
    {
        object = refined_object;
    }
    object = refine_generic_class_binding_expr(object, &method_name, &args, ctx);
    let object_ty = object.ty().clone();
    if reject_immutable_parameter_method_mutation(
        ctx,
        &object,
        &object_ty,
        &method_name,
        attr.value.range(),
    ) {
        return None;
    }
    let method_arg_ranges = resolved_method_arg_ranges(&object_ty_for_args, &method_name, call);
    let resolved_method_type = resolve_method_type(
        &object_ty,
        &method_name,
        &args,
        &method_arg_ranges,
        attr.attr.range(),
        ctx,
    )?;
    let return_ty = refine_nonempty_method_return_type(
        &object_ty,
        &object,
        &method_name,
        &args,
        &resolved_method_type,
        ctx,
    );
    tsc::validate_channel_send_element(
        &object_ty,
        &method_name,
        &args,
        &method_arg_ranges,
        call,
        ctx,
    );
    invalidate_collection_flow_facts_for_method(ctx, &object, &object_ty, &method_name);
    if matches!(object_ty.resolve_alias(), Type::Str) && method_name == "encode" {
        let mut intrinsic_args = vec![object];
        let intrinsic_name = if args.is_empty() {
            "str_encode_utf8_result"
        } else {
            "str_encode_utf8_result_with_encoding"
        };
        if let Some(encoding) = args.first().cloned() {
            intrinsic_args.push(encoding);
        }
        return Some(HirExpr::Call {
            func: intrinsic_name.to_string(),
            args: intrinsic_args,
            ty: return_ty,
        });
    }
    if matches!(object_ty.resolve_alias(), Type::Bytes) && method_name == "decode" {
        let mut intrinsic_args = vec![object];
        let intrinsic_name = if args.is_empty() {
            "decode_utf8"
        } else {
            "decode_utf8_with_encoding"
        };
        if let Some(encoding) = args.first().cloned() {
            intrinsic_args.push(encoding);
        }
        return Some(HirExpr::Call {
            func: intrinsic_name.to_string(),
            args: intrinsic_args,
            ty: return_ty,
        });
    }

    Some(HirExpr::MethodCall {
        object: Box::new(object),
        method: method_name,
        args,
        ty: return_ty,
    })
}

/// Resolve the return type of a method call on a given type.
pub(super) fn resolve_method_type(
    object_ty: &Type,
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    let canonical_object_ty = canonicalize_class_surface_type(object_ty);
    let object_ty = &canonical_object_ty;
    if let Type::Alias {
        name: alias_name,
        body,
        ..
    } = object_ty
    {
        if matches!(
            alias_name.as_str(),
            DEFAULTDICT_INT_ALIAS | DEFAULTDICT_LIST_ALIAS | DEFAULTDICT_SET_ALIAS
        ) {
            return resolve_method_type(body, method, args, arg_ranges, method_range, ctx);
        }
    }
    if matches!(object_ty, Type::AsyncGenerator(_, _)) {
        return super::async_generator_methods::resolve_async_generator_method_type(
            object_ty,
            method,
            args,
            arg_ranges,
            method_range,
            ctx,
        );
    }
    match object_ty {
        Type::List(elem_ty) => match method {
            "append" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        "list.append",
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                if !args[0].ty().is_assignable_to(elem_ty) {
                    list_append_argument_type_mismatch(ctx, args[0].ty(), elem_ty, arg_ranges[0]);
                }
                Some(Type::None)
            }
            "extend" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        "list.extend",
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                validate_list_extend_arg(elem_ty, args[0].ty(), arg_ranges[0], ctx);
                Some(Type::None)
            }
            "insert" => {
                if args.len() != 2 {
                    reject_exact_method_arg_count(
                        ctx,
                        "list.insert",
                        2,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::None)
            }
            "clear" => {
                if !args.is_empty() {
                    reject_no_method_args(ctx, "list.clear", arg_ranges, method_range);
                    return None;
                }
                Some(Type::None)
            }
            "copy" => {
                if !args.is_empty() {
                    reject_no_method_args(ctx, "list.copy", arg_ranges, method_range);
                    return None;
                }
                Some(Type::List(elem_ty.clone()))
            }
            "reverse" => {
                if !args.is_empty() {
                    reject_no_method_args(ctx, "list.reverse", arg_ranges, method_range);
                    return None;
                }
                Some(Type::None)
            }
            "sort" => {
                if args.len() > 1 {
                    reject_max_method_arg_count(
                        ctx,
                        "list.sort",
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                if let Some(reverse_arg) = args.first() {
                    if reverse_arg.ty() != &Type::Bool {
                        expression_diagnostics::type_mismatch(
                            ctx,
                            format!(
                                "list.sort() argument 'reverse' must be 'bool', got '{}'",
                                reverse_arg.ty().display_name()
                            ),
                            arg_ranges[0],
                        );
                        return None;
                    }
                }
                Some(Type::None)
            }
            "count" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        "list.count",
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::Int)
            }
            "contains" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        "list.contains",
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::Bool)
            }
            "len" => {
                if !args.is_empty() {
                    reject_no_method_args(ctx, "list.len", arg_ranges, method_range);
                    return None;
                }
                Some(Type::Int)
            }
            "pop" => {
                if args.len() > 1 {
                    reject_max_method_arg_count(
                        ctx,
                        "list.pop",
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                if let Some(index_arg) = args.first() {
                    if index_arg.ty() != &Type::Int {
                        expression_diagnostics::type_mismatch(
                            ctx,
                            format!(
                                "list.pop() index must be 'int', got '{}'",
                                index_arg.ty().display_name()
                            ),
                            arg_ranges[0],
                        );
                    }
                }
                // pop() returns Option[T] = T | None
                Some(Type::Union(vec![*elem_ty.clone(), Type::None]))
            }
            "popleft" => {
                if !args.is_empty() {
                    reject_no_method_args(ctx, "list.popleft", arg_ranges, method_range);
                    return None;
                }
                Some(Type::Union(vec![*elem_ty.clone(), Type::None]))
            }
            "appendleft" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        "list.appendleft",
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::None)
            }
            "remove" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        "list.remove",
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::None)
            }
            "index" => {
                if args.is_empty() || args.len() > 3 {
                    reject_method_arg_count(
                        ctx,
                        format!("list.index() takes 1 to 3 arguments, got {}", args.len()),
                        method_count_range(args.len(), 3, arg_ranges, method_range),
                    );
                    return None;
                }
                for (bound_index, bound) in args.iter().enumerate().skip(1) {
                    if bound.ty() != &Type::Int {
                        expression_diagnostics::type_mismatch(
                            ctx,
                            format!(
                                "list.index() bounds must be 'int', got '{}'",
                                bound.ty().display_name()
                            ),
                            arg_ranges.get(bound_index).copied().unwrap_or(method_range),
                        );
                    }
                }
                // Returns Option[int] = int | None (safe: no panic if not found)
                Some(Type::Union(vec![Type::Int, Type::None]))
            }
            _ => {
                ctx.error_with_code_at(
                    DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                    format!("list has no method '{method}'"),
                    method_range,
                );
                None
            }
        },
        Type::Dict(key_ty, val_ty) => match method {
            "len" => {
                if !args.is_empty() {
                    reject_no_method_args(ctx, "dict.len", arg_ranges, method_range);
                    return None;
                }
                Some(Type::Int)
            }
            "keys" => {
                if !args.is_empty() {
                    reject_no_method_args(ctx, "dict.keys", arg_ranges, method_range);
                    return None;
                }
                Some(Type::List(key_ty.clone()))
            }
            "values" => {
                if !args.is_empty() {
                    reject_no_method_args(ctx, "dict.values", arg_ranges, method_range);
                    return None;
                }
                Some(Type::List(val_ty.clone()))
            }
            "items" => {
                if !args.is_empty() {
                    reject_no_method_args(ctx, "dict.items", arg_ranges, method_range);
                    return None;
                }
                Some(Type::List(Box::new(Type::Tuple(vec![
                    *key_ty.clone(),
                    *val_ty.clone(),
                ]))))
            }
            "update" => {
                if args.len() > 2 {
                    reject_max_method_arg_count(
                        ctx,
                        "dict.update",
                        2,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                if let Some(arg) = args.first() {
                    validate_dict_update_arg(key_ty, val_ty, arg.ty(), arg_ranges[0], ctx);
                }
                if let Some(keyword_dict) = args.get(1) {
                    validate_dict_update_arg(key_ty, val_ty, keyword_dict.ty(), arg_ranges[1], ctx);
                }
                Some(Type::None)
            }
            "clear" => {
                if !args.is_empty() {
                    reject_no_method_args(ctx, "dict.clear", arg_ranges, method_range);
                    return None;
                }
                Some(Type::None)
            }
            "copy" => {
                if !args.is_empty() {
                    reject_no_method_args(ctx, "dict.copy", arg_ranges, method_range);
                    return None;
                }
                Some(Type::Dict(key_ty.clone(), val_ty.clone()))
            }
            "contains" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        "dict.contains",
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                if !args[0].ty().is_assignable_to(key_ty) {
                    expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "dict.contains() key type '{}' is not compatible with dict key type '{}'",
                            args[0].ty().display_name(),
                            key_ty.display_name()
                        ),
                        arg_ranges[0],
                    );
                }
                Some(Type::Bool)
            }
            "get" => {
                if args.is_empty() || args.len() > 2 {
                    reject_method_arg_count(
                        ctx,
                        format!("dict.get() takes 1 or 2 arguments, got {}", args.len()),
                        method_count_range(args.len(), 2, arg_ranges, method_range),
                    );
                    return None;
                }
                if !args[0].ty().is_assignable_to(key_ty) {
                    expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "dict.get() key type '{}' is not compatible with dict key type '{}'",
                            args[0].ty().display_name(),
                            key_ty.display_name()
                        ),
                        arg_ranges[0],
                    );
                }
                if args.len() == 2 {
                    if !args[1].ty().is_assignable_to(val_ty) {
                        expression_diagnostics::type_mismatch(
                            ctx,
                            format!(
                                "dict.get() default type '{}' is not compatible with dict value type '{}'",
                                args[1].ty().display_name(),
                                val_ty.display_name()
                            ),
                            arg_ranges[1],
                        );
                    }
                    // When V is still unknown/Any (e.g. empty literal before specialization),
                    // preserve precision from the provided default instead of leaking `Any`.
                    if matches!(val_ty.as_ref(), Type::Any | Type::Unknown) {
                        Some(args[1].ty().clone())
                    } else {
                        // dict.get(key, default) -> V (returns default if key not found)
                        Some(*val_ty.clone())
                    }
                } else {
                    // dict.get(key) -> V | None
                    Some(Type::Union(vec![*val_ty.clone(), Type::None]))
                }
            }
            "pop" => {
                if args.is_empty() || args.len() > 2 {
                    reject_method_arg_count(
                        ctx,
                        format!("dict.pop() takes 1 or 2 arguments, got {}", args.len()),
                        method_count_range(args.len(), 2, arg_ranges, method_range),
                    );
                    return None;
                }
                if !args[0].ty().is_assignable_to(key_ty) {
                    expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "dict.pop() key type '{}' is not compatible with dict key type '{}'",
                            args[0].ty().display_name(),
                            key_ty.display_name()
                        ),
                        arg_ranges[0],
                    );
                }
                if args.len() == 2 {
                    if !args[1].ty().is_assignable_to(val_ty) {
                        expression_diagnostics::type_mismatch(
                            ctx,
                            format!(
                                "dict.pop() default type '{}' is not compatible with dict value type '{}'",
                                args[1].ty().display_name(),
                                val_ty.display_name()
                            ),
                            arg_ranges[1],
                        );
                    }
                    Some(*val_ty.clone())
                } else {
                    // pop() returns Option[V] = V | None
                    Some(Type::Union(vec![*val_ty.clone(), Type::None]))
                }
            }
            "setdefault" => {
                if args.len() != 2 {
                    reject_exact_method_arg_count(
                        ctx,
                        "dict.setdefault",
                        2,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                if !args[0].ty().is_assignable_to(key_ty) {
                    expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "dict.setdefault() key type '{}' is not compatible with dict key type '{}'",
                            args[0].ty().display_name(),
                            key_ty.display_name()
                        ),
                        arg_ranges[0],
                    );
                }
                if !args[1].ty().is_assignable_to(val_ty) {
                    expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "dict.setdefault() default type '{}' is not compatible with dict value type '{}'",
                            args[1].ty().display_name(),
                            val_ty.display_name()
                        ),
                        arg_ranges[1],
                    );
                }
                Some(*val_ty.clone())
            }
            _ => {
                ctx.error_with_code_at(
                    DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                    format!("dict has no method '{method}'"),
                    method_range,
                );
                None
            }
        },
        Type::Set(elem_ty) => match method {
            "len" => {
                if !args.is_empty() {
                    reject_no_method_args(ctx, "set.len", arg_ranges, method_range);
                    return None;
                }
                Some(Type::Int)
            }
            "add" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        "set.add",
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::None)
            }
            "remove" | "discard" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        &format!("set.{method}"),
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::None)
            }
            "contains" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        "set.contains",
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::Bool)
            }
            "clear" => {
                if !args.is_empty() {
                    reject_no_method_args(ctx, "set.clear", arg_ranges, method_range);
                    return None;
                }
                Some(Type::None)
            }
            "copy" => {
                if !args.is_empty() {
                    reject_no_method_args(ctx, "set.copy", arg_ranges, method_range);
                    return None;
                }
                Some(Type::Set(elem_ty.clone()))
            }
            "union" | "intersection" | "difference" => {
                for (index, arg) in args.iter().enumerate() {
                    validate_set_iterable_arg(elem_ty, arg.ty(), method, arg_ranges[index], ctx);
                }
                Some(Type::Set(elem_ty.clone()))
            }
            "symmetric_difference" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        &format!("set.{method}"),
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                validate_set_iterable_arg(elem_ty, args[0].ty(), method, arg_ranges[0], ctx);
                Some(Type::Set(elem_ty.clone()))
            }
            "update" | "intersection_update" | "difference_update" => {
                for (index, arg) in args.iter().enumerate() {
                    validate_set_iterable_arg(elem_ty, arg.ty(), method, arg_ranges[index], ctx);
                }
                Some(Type::None)
            }
            "symmetric_difference_update" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        &format!("set.{method}"),
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                validate_set_iterable_arg(elem_ty, args[0].ty(), method, arg_ranges[0], ctx);
                Some(Type::None)
            }
            "issubset" | "issuperset" | "isdisjoint" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        &format!("set.{method}"),
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                validate_set_iterable_arg(elem_ty, args[0].ty(), method, arg_ranges[0], ctx);
                Some(Type::Bool)
            }
            "pop" => {
                if !args.is_empty() {
                    reject_no_method_args(ctx, "set.pop", arg_ranges, method_range);
                    return None;
                }
                // Returns Option[T] = T | None (safe: no panic on empty set)
                Some(Type::Union(vec![*elem_ty.clone(), Type::None]))
            }
            _ => {
                ctx.error_with_code_at(
                    DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                    format!("set has no method '{method}'"),
                    method_range,
                );
                None
            }
        },
        Type::Str => match method {
            "len" => Some(Type::Int),
            "upper" | "lower" | "strip" | "lstrip" | "rstrip" | "title" | "capitalize"
            | "swapcase" => Some(Type::Str),
            "startswith" | "endswith" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        &format!("str.{method}"),
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::Bool)
            }
            "isdigit" | "isalpha" | "isalnum" | "isspace" | "isupper" | "islower" => {
                if !args.is_empty() {
                    reject_no_method_args(ctx, &format!("str.{method}"), arg_ranges, method_range);
                    return None;
                }
                Some(Type::Bool)
            }
            "split" => {
                if args.len() > 2 {
                    reject_method_arg_count(
                        ctx,
                        format!("str.split() takes 0 to 2 arguments, got {}", args.len()),
                        method_count_range(args.len(), 2, arg_ranges, method_range),
                    );
                    return None;
                }
                if let Some(maxsplit) = args.get(1) {
                    if maxsplit.ty() != &Type::Int {
                        expression_diagnostics::type_mismatch(
                            ctx,
                            format!(
                                "str.split() maxsplit must be 'int', got '{}'",
                                maxsplit.ty().display_name()
                            ),
                            arg_ranges[1],
                        );
                    }
                }
                Some(Type::List(Box::new(Type::Str)))
            }
            "replace" => {
                if args.len() < 2 || args.len() > 3 {
                    reject_method_arg_count(
                        ctx,
                        format!("str.replace() takes 2 or 3 arguments, got {}", args.len()),
                        method_count_range(args.len(), 3, arg_ranges, method_range),
                    );
                    return None;
                }
                if let Some(count) = args.get(2) {
                    if count.ty() != &Type::Int {
                        expression_diagnostics::type_mismatch(
                            ctx,
                            format!(
                                "str.replace() count must be 'int', got '{}'",
                                count.ty().display_name()
                            ),
                            arg_ranges[2],
                        );
                    }
                }
                Some(Type::Str)
            }
            "join" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        "str.join",
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::Str)
            }
            "count" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        "str.count",
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::Int)
            }
            "center" | "ljust" | "rjust" | "zfill" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        &format!("str.{method}"),
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::Str)
            }
            "find" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        "str.find",
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                // find() returns Option[int] = int | None
                Some(Type::Union(vec![Type::Int, Type::None]))
            }
            "encode" => resolve_str_encode_method_type(args, arg_ranges, method_range, ctx),
            _ => {
                ctx.error_with_code_at(
                    DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                    format!("str has no method '{method}'"),
                    method_range,
                );
                None
            }
        },
        Type::Bytes => resolve_bytes_method_type(method, args, arg_ranges, method_range, ctx),
        Type::FixedInt(fixed) => {
            resolve_fixed_width_method_type(*fixed, method, args, arg_ranges, method_range, ctx)
        }
        Type::Tuple(_) => match method {
            "len" => Some(Type::Int),
            "count" => {
                if args.len() != 1 {
                    reject_exact_method_arg_count(
                        ctx,
                        "tuple.count",
                        1,
                        args.len(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::Int)
            }
            "index" => {
                if args.is_empty() || args.len() > 3 {
                    reject_method_arg_count(
                        ctx,
                        format!("tuple.index() takes 1 to 3 arguments, got {}", args.len()),
                        method_count_range(args.len(), 3, arg_ranges, method_range),
                    );
                    return None;
                }
                for (bound_index, bound) in args.iter().enumerate().skip(1) {
                    if bound.ty() != &Type::Int {
                        expression_diagnostics::type_mismatch(
                            ctx,
                            format!(
                                "tuple.index() bounds must be 'int', got '{}'",
                                bound.ty().display_name()
                            ),
                            arg_ranges.get(bound_index).copied().unwrap_or(method_range),
                        );
                    }
                }
                Some(Type::Union(vec![Type::Int, Type::None]))
            }
            _ => {
                ctx.error_with_code_at(
                    DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                    format!("tuple has no method '{method}'"),
                    method_range,
                );
                None
            }
        },
        Type::Class {
            name,
            fields,
            methods,
            ..
        } => {
            if let Some((_, ft)) = methods.iter().find(|(n, _)| n == method) {
                // Check argument count
                if args.len() != ft.params.len() {
                    reject_method_arg_count(
                        ctx,
                        format!(
                            "{}.{}() takes {} argument(s), got {}",
                            name,
                            method,
                            ft.params.len(),
                            args.len()
                        ),
                        method_count_range(args.len(), ft.params.len(), arg_ranges, method_range),
                    );
                    return None;
                }
                // Check argument types
                for (i, (arg, (param_name, param_ty, _))) in
                    args.iter().zip(ft.params.iter()).enumerate()
                {
                    if !arg.ty().is_assignable_to(param_ty) {
                        expression_diagnostics::type_mismatch(
                            ctx,
                            format!(
                                "argument {} ('{}') of {}.{}(): expected '{}', got '{}'",
                                i + 1,
                                param_name,
                                name,
                                method,
                                param_ty.display_name(),
                                arg.ty().display_name()
                            ),
                            arg_ranges.get(i).copied().unwrap_or(method_range),
                        );
                    }
                }
                Some(canonicalize_class_surface_type(&ft.return_type))
            } else if let Some((_, field_ty)) = fields.iter().find(|(n, _)| n == method) {
                // Check if the field is a Callable type — allow calling it like a method
                if let Type::Callable(param_types, _, ret_type) = field_ty {
                    if args.len() != param_types.len() {
                        expression_diagnostics::call_not_callable_or_arity(
                            ctx,
                            format!(
                                "{}.{}() (callable field) takes {} argument(s), got {}",
                                name,
                                method,
                                param_types.len(),
                                args.len()
                            ),
                            method_count_range(
                                args.len(),
                                param_types.len(),
                                arg_ranges,
                                method_range,
                            ),
                        );
                        return None;
                    }
                    for (i, (arg, param_ty)) in args.iter().zip(param_types.iter()).enumerate() {
                        if !arg.ty().is_assignable_to(param_ty) {
                            expression_diagnostics::type_mismatch(
                                ctx,
                                format!(
                                    "argument {} of {}.{}(): expected '{}', got '{}'",
                                    i + 1,
                                    name,
                                    method,
                                    param_ty.display_name(),
                                    arg.ty().display_name()
                                ),
                                arg_ranges.get(i).copied().unwrap_or(method_range),
                            );
                        }
                    }
                    Some(canonicalize_class_surface_type(ret_type))
                } else {
                    expression_diagnostics::call_not_callable_or_arity(
                        ctx,
                        format!(
                            "field '{}' of class '{}' is not callable (type: '{}')",
                            method,
                            name,
                            field_ty.display_name()
                        ),
                        method_range,
                    );
                    None
                }
            } else {
                ctx.error_with_code_at(
                    DiagnosticCode::CLASS_MISSING_MEMBER,
                    format!("class '{name}' has no method '{method}'"),
                    method_range,
                );
                None
            }
        }
        Type::Protocol { name, methods, .. } => {
            if let Some((_, ft)) = methods.iter().find(|(n, _)| n == method) {
                if args.len() != ft.params.len() {
                    reject_method_arg_count(
                        ctx,
                        format!(
                            "{}.{}() takes {} argument(s), got {}",
                            name,
                            method,
                            ft.params.len(),
                            args.len()
                        ),
                        method_count_range(args.len(), ft.params.len(), arg_ranges, method_range),
                    );
                    return None;
                }
                Some(canonicalize_class_surface_type(&ft.return_type))
            } else {
                ctx.error_with_code_at(
                    DiagnosticCode::PROTO_BOUND_NOT_SATISFIED,
                    format!("protocol '{name}' has no method '{method}'"),
                    method_range,
                );
                None
            }
        }
        Type::Newtype { name, inner } => {
            // Newtype has a built-in `value()` method that returns the inner type
            if method == "value" {
                if !args.is_empty() {
                    reject_no_method_args(ctx, &format!("{name}.value"), arg_ranges, method_range);
                    return None;
                }
                Some(*inner.clone())
            } else {
                // Delegate to the inner type's methods
                resolve_method_type(inner, method, args, arg_ranges, method_range, ctx)
            }
        }
        Type::Enum { name, .. } => {
            match method {
                "name" => {
                    if !args.is_empty() {
                        reject_no_method_args(
                            ctx,
                            &format!("{name}.name"),
                            arg_ranges,
                            method_range,
                        );
                        return None;
                    }
                    Some(Type::Str)
                }
                "value" => {
                    if !args.is_empty() {
                        reject_no_method_args(
                            ctx,
                            &format!("{name}.value"),
                            arg_ranges,
                            method_range,
                        );
                        return None;
                    }
                    Some(Type::Int)
                }
                _ => {
                    // Check user-defined methods registered in functions
                    let method_key = format!("{name}.{method}");
                    if let Some(ft) = ctx.functions.get(&method_key).cloned() {
                        return Some(*ft.return_type.clone());
                    }
                    ctx.error_with_code_at(
                        DiagnosticCode::CLASS_MISSING_MEMBER,
                        format!("enum '{name}' has no method '{method}'"),
                        method_range,
                    );
                    None
                }
            }
        }
        Type::BigInt => {
            if method == "clone" {
                if !args.is_empty() {
                    reject_no_method_args(ctx, "bigint.clone", arg_ranges, method_range);
                    return None;
                }
                Some(Type::BigInt)
            } else {
                ctx.error_with_code_at(
                    DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                    format!("type 'bigint' has no method '{method}'"),
                    method_range,
                );
                None
            }
        }
        Type::Decimal | Type::BigDecimal => {
            resolve_decimal_method_type(object_ty, method, args, arg_ranges, method_range, ctx)
        }
        _ => {
            ctx.error_with_code_at(
                DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                format!(
                    "type '{}' has no method '{}'",
                    object_ty.display_name(),
                    method
                ),
                method_range,
            );
            None
        }
    }
}

pub(super) fn lower_lambda_with_context(
    expr: &Expr,
    context_types: &[Type],
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if let Expr::Lambda(lambda) = expr {
        let (params, body, body_ty) = ctx.with_pushed_scope(|ctx| {
            let mut params = Vec::new();
            if let Some(ref parameters) = lambda.parameters {
                for (i, param) in parameters.args.iter().enumerate() {
                    let param_name = param.parameter.name.to_string();
                    let param_ty = if let Some(ref ann) = param.parameter.annotation {
                        resolve_annotation_expr(ann, ctx)
                    } else if i < context_types.len() {
                        // Use contextual type
                        context_types[i].clone()
                    } else {
                        Type::Any
                    };
                    ctx.scope.define(param_name.clone(), param_ty.clone());
                    params.push(HirParam {
                        name: param_name,
                        ty: param_ty,
                        default: None,
                        keyword_only: false,
                        convention: ParamConvention::default(),
                    });
                }
            }

            let body = lower_expr(&lambda.body, ctx)?;
            let body_ty = body.ty().clone();
            Some((params, body, body_ty))
        })?;

        let param_types: Vec<(String, Type)> = params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();
        let fn_ty = Type::Function(FunctionType::new(param_types, body_ty));

        Some(HirExpr::Lambda {
            params,
            body: Box::new(body),
            ty: fn_ty,
        })
    } else {
        // Not a lambda, lower normally
        lower_expr(expr, ctx)
    }
}

pub(super) fn lower_lambda(lambda: &ExprLambda, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let (params, body, body_ty) = ctx.with_pushed_scope(|ctx| {
        let mut params = Vec::new();
        if let Some(ref parameters) = lambda.parameters {
            for param in &parameters.args {
                let param_name = param.parameter.name.to_string();
                let param_ty = if let Some(ref ann) = param.parameter.annotation {
                    resolve_annotation_expr(ann, ctx)
                } else {
                    // Unannotated lambda params start as Any and may be refined
                    // by contextual typing at call sites.
                    Type::Any
                };
                ctx.scope.define(param_name.clone(), param_ty.clone());
                params.push(HirParam {
                    name: param_name,
                    ty: param_ty,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::default(),
                });
            }
        }

        let body = lower_expr(&lambda.body, ctx)?;
        let body_ty = body.ty().clone();
        Some((params, body, body_ty))
    })?;

    // Build the function type for the lambda
    let param_types: Vec<(String, Type)> = params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    let fn_ty = Type::Function(FunctionType::new(param_types, body_ty));

    Some(HirExpr::Lambda {
        params,
        body: Box::new(body),
        ty: fn_ty,
    })
}

fn reject_invalid_expression_target(ctx: &mut LowerCtx, message: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::FLOW_INVALID_ASSIGNMENT_TARGET,
        message.to_string(),
        range,
    );
}

fn reject_invalid_expression_iteration(ctx: &mut LowerCtx, iter_ty: &Type, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::FLOW_INVALID_ITERATION,
        format!("cannot iterate over type '{}'", iter_ty.display_name()),
        range,
    );
}

fn reject_unsupported_expression_form(ctx: &mut LowerCtx, message: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
        message.to_string(),
        range,
    );
}

pub(super) fn lower_list_comp(comp: &ExprListComp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if comp.generators.is_empty() {
        reject_unsupported_expression_form(
            ctx,
            "list comprehension must have at least one generator",
            comp.range(),
        );
        return None;
    }

    if super::async_comprehension_diagnostics::reject_deferred_async_comprehension_shape(
        ctx,
        "list",
        &comp.generators,
        comp.range(),
    ) {
        return None;
    }

    let mut generators = Vec::new();
    let mut pushed_scopes = 0;
    let result = (|| {
        // Process each generator: push scope, define var, lower iter
        for gen in &comp.generators {
            let var_name = match &gen.target {
                Expr::Name(n) => n.id.to_string(),
                Expr::Tuple(tup) => {
                    let names: Vec<String> = tup
                        .elts
                        .iter()
                        .filter_map(|e| {
                            if let Expr::Name(n) = e {
                                Some(n.id.to_string())
                            } else {
                                None
                            }
                        })
                        .collect();
                    if names.len() != tup.elts.len() {
                        reject_invalid_expression_target(
                            ctx,
                            "comprehension tuple target must contain only simple names",
                            gen.target.range(),
                        );
                        return None;
                    }
                    names.join(",")
                }
                _ => {
                    reject_invalid_expression_target(
                        ctx,
                        "comprehension target must be a simple name or tuple",
                        gen.target.range(),
                    );
                    return None;
                }
            };

            let iter_source_expr = lower_expr(&gen.iter, ctx)?;
            let iter_ty = iter_source_expr.ty().clone();
            let Some(elem_ty) = callable_builtin_element_type(&iter_ty) else {
                reject_invalid_expression_iteration(ctx, &iter_ty, gen.iter.range());
                return None;
            };

            ctx.scope.push();
            pushed_scopes += 1;
            if var_name.contains(',') {
                let names: Vec<&str> = var_name.split(',').collect();
                if let Type::Tuple(elem_types) = &elem_ty {
                    for (i, name) in names.iter().enumerate() {
                        let ty = elem_types.get(i).cloned().unwrap_or(Type::Any);
                        ctx.scope.define((*name).to_string(), ty);
                    }
                } else {
                    for name in &names {
                        ctx.scope.define((*name).to_string(), Type::Any);
                    }
                }
            } else {
                ctx.scope.define(var_name.clone(), elem_ty.clone());
            }

            let filter = if gen.ifs.is_empty() {
                None
            } else {
                let first = lower_expr(&gen.ifs[0], ctx)?;
                if gen.ifs.len() == 1 {
                    Some(first)
                } else {
                    let mut combined = first;
                    for cond in &gen.ifs[1..] {
                        let next = lower_expr(cond, ctx)?;
                        combined = HirExpr::BoolOp {
                            op: "and".to_string(),
                            values: vec![combined, next],
                            ty: Type::Bool,
                        };
                    }
                    Some(combined)
                }
            };

            let iter_expr = lower_iterator_protocol_entry(iter_source_expr, elem_ty);
            generators.push((var_name, iter_expr, filter));
        }

        // Lower the expression (all generator vars are in scope)
        let expr = lower_expr(&comp.elt, ctx)?;
        let expr_ty = expr.ty().clone();
        let result_ty = Type::List(Box::new(expr_ty));

        Some(HirExpr::ListComp {
            expr: Box::new(expr),
            generators,
            ty: result_ty,
        })
    })();
    ctx.pop_scopes(pushed_scopes);
    result
}

pub(super) fn lower_set_comp(comp: &ExprSetComp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if super::async_comprehension_diagnostics::reject_deferred_async_comprehension_shape(
        ctx,
        "set",
        &comp.generators,
        comp.range(),
    ) {
        return None;
    }

    let mut generators = Vec::new();
    let mut pushed_scopes = 0;
    let result = (|| {
        for gen in &comp.generators {
            let var_name = if let Expr::Name(n) = &gen.target {
                n.id.to_string()
            } else {
                reject_invalid_expression_target(
                    ctx,
                    "set comprehension target must be a simple name",
                    gen.target.range(),
                );
                return None;
            };
            let iter_source_expr = lower_expr(&gen.iter, ctx)?;
            let iter_ty = iter_source_expr.ty().clone();
            let Some(elem_ty) = callable_builtin_element_type(&iter_ty) else {
                reject_invalid_expression_iteration(ctx, &iter_ty, gen.iter.range());
                return None;
            };
            ctx.scope.push();
            pushed_scopes += 1;
            ctx.scope.define(var_name.clone(), elem_ty.clone());
            let filter = if gen.ifs.is_empty() {
                None
            } else {
                Some(lower_expr(&gen.ifs[0], ctx)?)
            };
            let iter_expr = lower_iterator_protocol_entry(iter_source_expr, elem_ty);
            generators.push((var_name, iter_expr, filter));
        }
        let expr = lower_expr(&comp.elt, ctx)?;
        let expr_ty = expr.ty().clone();
        let result_ty = Type::Set(Box::new(expr_ty));
        Some(HirExpr::SetComp {
            expr: Box::new(expr),
            generators,
            ty: result_ty,
        })
    })();
    ctx.pop_scopes(pushed_scopes);
    result
}

pub(super) fn lower_dict_comp(comp: &ExprDictComp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if super::async_comprehension_diagnostics::reject_deferred_async_comprehension_shape(
        ctx,
        "dict",
        &comp.generators,
        comp.range(),
    ) {
        return None;
    }

    let mut generators = Vec::new();
    let mut pushed_scopes = 0;
    let result = (|| {
        for gen in &comp.generators {
            let var_name = match &gen.target {
                Expr::Name(n) => n.id.to_string(),
                Expr::Tuple(tup) => {
                    let names: Vec<String> = tup
                        .elts
                        .iter()
                        .filter_map(|e| {
                            if let Expr::Name(n) = e {
                                Some(n.id.to_string())
                            } else {
                                None
                            }
                        })
                        .collect();
                    if names.len() != tup.elts.len() {
                        reject_invalid_expression_target(
                            ctx,
                            "dict comprehension tuple target must contain only simple names",
                            gen.target.range(),
                        );
                        return None;
                    }
                    names.join(",")
                }
                _ => {
                    reject_invalid_expression_target(
                        ctx,
                        "dict comprehension target must be a simple name or tuple",
                        gen.target.range(),
                    );
                    return None;
                }
            };
            let iter_source_expr = lower_expr(&gen.iter, ctx)?;
            let iter_ty = iter_source_expr.ty().clone();
            let Some(elem_ty) = callable_builtin_element_type(&iter_ty) else {
                reject_invalid_expression_iteration(ctx, &iter_ty, gen.iter.range());
                return None;
            };
            ctx.scope.push();
            pushed_scopes += 1;
            if var_name.contains(',') {
                let names: Vec<&str> = var_name.split(',').collect();
                if let Type::Tuple(elem_types) = &elem_ty {
                    for (i, name) in names.iter().enumerate() {
                        let ty = elem_types.get(i).cloned().unwrap_or(Type::Any);
                        ctx.scope.define((*name).to_string(), ty);
                    }
                } else {
                    for name in &names {
                        ctx.scope.define((*name).to_string(), Type::Any);
                    }
                }
            } else {
                ctx.scope.define(var_name.clone(), elem_ty.clone());
            }
            let filter = if gen.ifs.is_empty() {
                None
            } else {
                Some(lower_expr(&gen.ifs[0], ctx)?)
            };
            let iter_expr = lower_iterator_protocol_entry(iter_source_expr, elem_ty);
            generators.push((var_name, iter_expr, filter));
        }
        let key_expr = lower_expr(&comp.key, ctx)?;
        let val_expr = lower_expr(&comp.value, ctx)?;
        let key_ty = key_expr.ty().clone();
        let val_ty = val_expr.ty().clone();
        let result_ty = Type::Dict(Box::new(key_ty), Box::new(val_ty));
        Some(HirExpr::DictComp {
            key_expr: Box::new(key_expr),
            val_expr: Box::new(val_expr),
            generators,
            ty: result_ty,
        })
    })();
    ctx.pop_scopes(pushed_scopes);
    result
}

pub(super) fn lower_generator_expr(gen: &ExprGenerator, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if gen.generators.iter().any(|generator| generator.is_async) {
        super::async_comprehension_diagnostics::reject_async_generator_expression(ctx, gen.range());
        return None;
    }

    // Only support single generator: (expr for var in iter) or (expr for var in iter if cond)
    if gen.generators.len() != 1 {
        reject_unsupported_expression_form(
            ctx,
            "only single-generator generator expressions are supported",
            gen.range(),
        );
        return None;
    }

    let comp = &gen.generators[0];

    let var_name = if let Expr::Name(n) = &comp.target {
        n.id.to_string()
    } else {
        reject_invalid_expression_target(
            ctx,
            "generator target must be a simple name",
            comp.target.range(),
        );
        return None;
    };
    let iter_source_expr = lower_expr(&comp.iter, ctx)?;
    let iter_ty = iter_source_expr.ty().clone();
    let Some(elem_ty) = callable_builtin_element_type(&iter_ty) else {
        reject_invalid_expression_iteration(ctx, &iter_ty, comp.iter.range());
        return None;
    };

    let (expr, expr_ty, filter) = ctx.with_pushed_scope(|ctx| {
        ctx.scope.define(var_name.clone(), elem_ty.clone());
        let expr = lower_expr(&gen.elt, ctx)?;
        let expr_ty = expr.ty().clone();
        let filter = if comp.ifs.is_empty() {
            None
        } else {
            let first = lower_expr(&comp.ifs[0], ctx)?;
            if comp.ifs.len() == 1 {
                Some(Box::new(first))
            } else {
                let mut combined = first;
                for cond in &comp.ifs[1..] {
                    let next = lower_expr(cond, ctx)?;
                    combined = HirExpr::BoolOp {
                        op: "and".to_string(),
                        values: vec![combined, next],
                        ty: Type::Bool,
                    };
                }
                Some(Box::new(combined))
            }
        };
        Some((expr, expr_ty, filter))
    })?;
    let result_ty = Type::Iterator(Box::new(expr_ty));
    let iter_expr = lower_iterator_protocol_entry(iter_source_expr, elem_ty);
    Some(HirExpr::GeneratorExpr {
        expr: Box::new(expr),
        var: var_name,
        iter: Box::new(iter_expr),
        filter,
        ty: result_ty,
    })
}

fn lower_iterator_protocol_entry(iter_source_expr: HirExpr, elem_ty: Type) -> HirExpr {
    HirExpr::IteratorCall {
        op: HirIteratorOp::Iter,
        args: vec![iter_source_expr],
        ty: Type::Iterator(Box::new(elem_ty)),
    }
}

pub(super) fn lower_named_expr(named: &ExprNamed, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let name = if let Expr::Name(n) = named.target.as_ref() {
        n.id.to_string()
    } else {
        reject_invalid_expression_target(
            ctx,
            "walrus operator target must be a simple name",
            named.target.range(),
        );
        return None;
    };

    let value = lower_expr(&named.value, ctx)?;
    let ty = value.ty().clone();

    // Define the variable in the current scope
    ctx.scope.define(name.clone(), ty.clone());

    Some(HirExpr::WalrusExpr {
        name,
        value: Box::new(value),
        ty,
    })
}
