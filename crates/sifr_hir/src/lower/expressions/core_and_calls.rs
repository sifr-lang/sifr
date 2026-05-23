use super::async_await::{coroutine_result_type, lower_await};
use super::async_generator_advances::lower_anext_call;
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
use super::task_calls::{lower_asyncio_compat_call, lower_task_module_call, TaskCallLowering};
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
        Expr::SetComp(comp) => super::async_comprehensions::lower_set_comp(comp, ctx)
            .unwrap_or_else(|| lower_set_comp(comp, ctx)),
        Expr::DictComp(comp) => super::async_comprehensions::lower_dict_comp(comp, ctx)
            .unwrap_or_else(|| lower_dict_comp(comp, ctx)),
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

#[rustfmt::skip]
pub(super) fn lower_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> { let compat_alias = resolve_python_compat_call_alias(call, ctx); if let (None, Expr::Attribute(attr)) = (&compat_alias, call.func.as_ref()) { if let Some(factory_call) = lower_bytes_type_factory_call(attr, call, ctx) { return Some(factory_call); } match lower_task_module_call(attr, call, ctx) { TaskCallLowering::Lowered(expr) => return Some(expr), TaskCallLowering::Rejected => return None, TaskCallLowering::NoMatch => {} } return lower_method_call(attr, call, ctx); } let func_name = if let Some(alias) = compat_alias { alias } else if let Expr::Name(n) = call.func.as_ref() { resolve_bare_python_compat_call_alias(n.id.as_str(), ctx) .unwrap_or_else(|| n.id.to_string()) } else { expression_diagnostics::call_not_callable_or_arity( ctx, "only simple function calls are supported".to_string(), call.func.range(), ); return None; }; match lower_asyncio_compat_call(&func_name, call, ctx) { TaskCallLowering::Lowered(expr) => return Some(expr), TaskCallLowering::Rejected => return None, TaskCallLowering::NoMatch => {} } if func_name == "cls" { if let Some(ref class_name) = ctx.current_class { let class_name = class_name.clone(); if let Some(class_ty) = ctx.class_types.get(&class_name).cloned() { let mut args = Vec::new(); for arg in &call.arguments.args { let expr = lower_expr(arg, ctx)?; args.push(expr); } return Some(HirExpr::ConstructorCall { class_name, args, ty: class_ty, }); } } }  let builtin_is_shadowed = ctx.scope.lookup(&func_name).is_some() || ctx.functions.contains_key(&func_name);  if !builtin_is_shadowed { if func_name == "defaultdict" { return lower_defaultdict_constructor_call(call, ctx); }  if func_name == "list" { return lower_list_constructor_call(call, ctx); }  if func_name == "tuple" { return lower_tuple_constructor_call(call, ctx); }  if func_name == "dict" { return lower_dict_constructor_call(call, ctx); }  if func_name == "set" { return lower_set_constructor_call(call, ctx); }  if func_name == "bytes" { return lower_bytes_constructor_call(call, ctx); }  if func_name == "ord" { return lower_ord_call(call, ctx); }  if func_name == "chr" { return lower_chr_call(call, ctx); }  if func_name == "range" { return lower_range_call(call, ctx); }  if func_name == "len" { return lower_len_call(call, ctx); }  if func_name == "iter" { if !call.arguments.keywords.is_empty() { expression_diagnostics::call_unexpected_keyword( ctx, "iter() does not accept keyword arguments".to_string(), first_call_keyword_range(call), ); return None; } if call.arguments.args.len() != 1 { expression_diagnostics::call_wrong_positional_count( ctx, format!( "iter() takes exactly 1 argument, got {}", call.arguments.args.len() ), call_arity_range(call), ); return None; } let iterable = lower_expr(&call.arguments.args[0], ctx)?; if matches!(iterable.ty().resolve_alias(), Type::Any | Type::Unknown) { expression_diagnostics::type_mismatch( ctx, format!( "iter() argument must be an iterable with a statically-known element type, got '{}'", iterable.ty().display_name() ), call.arguments.args[0].range(), ); return None; } let Some(elem_ty) = callable_builtin_element_type(iterable.ty()) else { if matches!(iterable.ty().resolve_alias(), Type::Tuple(_)) { ctx.error_with_code_at( DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT, "iter() tuple argument must have one statically provable element type" .to_string(), call.arguments.args[0].range(), ); return None; } expression_diagnostics::type_mismatch( ctx, format!( "iter() argument must be iterable, got '{}'", iterable.ty().display_name() ), call.arguments.args[0].range(), ); return None; }; return Some(HirExpr::IteratorCall { op: HirIteratorOp::Iter, args: vec![iterable], ty: Type::Iterator(Box::new(elem_ty)), }); }  if func_name == "next" { if !call.arguments.keywords.is_empty() { expression_diagnostics::call_unexpected_keyword( ctx, "next() does not accept keyword arguments".to_string(), first_call_keyword_range(call), ); return None; } if call.arguments.args.len() != 1 { expression_diagnostics::call_wrong_positional_count( ctx, format!( "next() takes exactly 1 argument, got {}", call.arguments.args.len() ), call_arity_range(call), ); return None; } let iterator = lower_expr(&call.arguments.args[0], ctx)?; let Some(elem_ty) = iterator.ty().iterator_element_type() else { expression_diagnostics::type_mismatch( ctx, format!( "next() argument must be an iterator, got '{}'", iterator.ty().display_name() ), call.arguments.args[0].range(), ); return None; }; return Some(HirExpr::IteratorCall { op: HirIteratorOp::Next, args: vec![iterator], ty: Type::Union(vec![elem_ty, Type::None]), }); }  if func_name == "anext" { return lower_anext_call(call, ctx); }  if func_name == "isinstance" { return lower_isinstance_call(call, ctx); }  if func_name == "reveal_type" { return lower_reveal_type_call(call, ctx); }  if func_name == "str" { if call.arguments.args.len() == 1 { let arg = lower_expr(&call.arguments.args[0], ctx)?; return Some(HirExpr::Call { func: "str".to_string(), args: vec![arg], ty: Type::Str, }); } }  if func_name == "pow" { if !call.arguments.keywords.is_empty() { expression_diagnostics::call_unexpected_keyword( ctx, "pow() does not accept keyword arguments".to_string(), first_call_keyword_range(call), ); return None; } if call.arguments.args.len() != 2 { expression_diagnostics::call_wrong_positional_count( ctx, "pow() takes exactly 2 arguments".to_string(), call_arity_range(call), ); return None; } let base = lower_expr(&call.arguments.args[0], ctx)?; let exp = lower_expr(&call.arguments.args[1], ctx)?; let result_ty = if base.ty() == &Type::Int && exp.ty() == &Type::Int { Type::Int } else { Type::Float }; return Some(HirExpr::Call { func: "pow".to_string(), args: vec![base, exp], ty: result_ty, }); }  if func_name == "abs" { return lower_abs_call(call, ctx); }  if func_name == "hash" { if !call.arguments.keywords.is_empty() { expression_diagnostics::call_unexpected_keyword( ctx, "hash() does not accept keyword arguments".to_string(), first_call_keyword_range(call), ); return None; } if call.arguments.args.len() != 1 { expression_diagnostics::call_wrong_positional_count( ctx, format!( "hash() takes exactly 1 argument, got {}", call.arguments.args.len() ), call_arity_range(call), ); return None; } let arg = lower_expr(&call.arguments.args[0], ctx)?; let ty = arg.ty().clone(); if !is_hashable_type(&ty) { let type_name = ty.display_name(); ctx.error_with_code_at( DiagnosticCode::PROTO_HASHABLE_OR_COMPARABLE_REQUIRED, format!("hash() argument must be hashable, got '{type_name}'"), call.arguments.args[0].range(), ); return None; } return Some(HirExpr::Call { func: "hash".to_string(), args: vec![arg], ty: Type::Int, }); }  if func_name == "round" { if !call.arguments.keywords.is_empty() { expression_diagnostics::call_unexpected_keyword( ctx, "round() does not accept keyword arguments".to_string(), first_call_keyword_range(call), ); return None; } if call.arguments.args.is_empty() || call.arguments.args.len() > 2 { expression_diagnostics::call_wrong_positional_count( ctx, format!( "round() takes 1 or 2 arguments, got {}", call.arguments.args.len() ), call_arity_range(call), ); return None; } let arg = lower_expr(&call.arguments.args[0], ctx)?; if !arg.ty().is_numeric() { expression_diagnostics::type_mismatch( ctx, format!( "round() argument must be numeric, got '{}'", arg.ty().display_name() ), call.arguments.args[0].range(), ); return None; } if call.arguments.args.len() == 2 { let ndigits = lower_expr(&call.arguments.args[1], ctx)?; return Some(HirExpr::Call { func: "round".to_string(), args: vec![arg, ndigits], ty: Type::Float, }); } return Some(HirExpr::Call { func: "round".to_string(), args: vec![arg], ty: Type::Int, }); }  if func_name == "repr" { if !call.arguments.keywords.is_empty() { expression_diagnostics::call_unexpected_keyword( ctx, "repr() does not accept keyword arguments".to_string(), first_call_keyword_range(call), ); return None; } if call.arguments.args.len() != 1 { expression_diagnostics::call_wrong_positional_count( ctx, format!( "repr() takes exactly 1 argument, got {}", call.arguments.args.len() ), call_arity_range(call), ); return None; } let arg = lower_expr(&call.arguments.args[0], ctx)?; return Some(HirExpr::Call { func: "repr".to_string(), args: vec![arg], ty: Type::Str, }); }  if func_name == "Decimal" { return lower_decimal_constructor_call(call, ctx); }  if func_name == "BigDecimal" { return lower_bigdecimal_constructor_call(call, ctx); }  if func_name == "int" { if !call.arguments.keywords.is_empty() { expression_diagnostics::call_unexpected_keyword( ctx, "int() does not accept keyword arguments".to_string(), first_call_keyword_range(call), ); return None; } if call.arguments.args.len() != 1 { expression_diagnostics::call_wrong_positional_count( ctx, format!( "int() takes exactly 1 argument, got {}", call.arguments.args.len() ), call_arity_range(call), ); return None; } let arg = lower_expr(&call.arguments.args[0], ctx)?; let arg_ty = arg.ty().clone(); let result_ty = if arg_ty == Type::Str { let parse_error_ty = ctx.class_types .get("ParseError") .cloned() .unwrap_or(Type::Class { name: "ParseError".to_string(), fields: vec![("message".to_string(), Type::Str)], methods: vec![], parent_class: None, }); Type::Result(Box::new(Type::Int), Box::new(parse_error_ty)) } else if arg_ty == Type::BigInt { let overflow_error_ty = ctx.class_types .get("OverflowError") .cloned() .unwrap_or(Type::Class { name: "OverflowError".to_string(), fields: vec![("message".to_string(), Type::Str)], methods: vec![], parent_class: None, }); Type::Result(Box::new(Type::Int), Box::new(overflow_error_ty)) } else if matches!(arg_ty, Type::Decimal | Type::BigDecimal) { Type::Result( Box::new(Type::Int), Box::new(decimal_conversion_error_type(ctx)), ) } else { Type::Int }; return Some(HirExpr::Call { func: "int".to_string(), args: vec![arg], ty: result_ty, }); }  if func_name == "bigint" { ctx.warn_bigint_transition_alias(call.func.range()); if !call.arguments.keywords.is_empty() { expression_diagnostics::call_unexpected_keyword( ctx, "bigint() does not accept keyword arguments".to_string(), first_call_keyword_range(call), ); return None; } if call.arguments.args.len() != 1 { expression_diagnostics::call_wrong_positional_count( ctx, format!( "bigint() takes exactly 1 argument, got {}", call.arguments.args.len() ), call_arity_range(call), ); return None; } let arg = lower_expr(&call.arguments.args[0], ctx)?; let arg_ty = arg.ty().clone(); if !matches!( arg_ty, Type::Int | Type::LiteralInt(_) | Type::BigInt | Type::Decimal | Type::BigDecimal ) { expression_diagnostics::type_mismatch( ctx, format!( "bigint() requires int, bigint, decimal, or bigdecimal argument, got '{}'", arg_ty.display_name() ), call.arguments.args[0].range(), ); return None; } return Some(HirExpr::Call { func: "bigint".to_string(), args: vec![arg], ty: Type::BigInt, }); }  if func_name == "float" { if !call.arguments.keywords.is_empty() { expression_diagnostics::call_unexpected_keyword( ctx, "float() does not accept keyword arguments".to_string(), first_call_keyword_range(call), ); return None; } if call.arguments.args.len() != 1 { expression_diagnostics::call_wrong_positional_count( ctx, format!( "float() takes exactly 1 argument, got {}", call.arguments.args.len() ), call_arity_range(call), ); return None; } if let Some(kind) = float_sentinel_kind_from_call(call) { return Some(float_sentinel_expr(kind)); } let arg = lower_expr(&call.arguments.args[0], ctx)?; let arg_ty = arg.ty().clone(); let result_ty = if arg_ty == Type::Str { let parse_error_ty = ctx.class_types .get("ParseError") .cloned() .unwrap_or(Type::Class { name: "ParseError".to_string(), fields: vec![("message".to_string(), Type::Str)], methods: vec![], parent_class: None, }); Type::Result(Box::new(Type::Float), Box::new(parse_error_ty)) } else if arg_ty == Type::Decimal { ctx.error_with_code_at( DiagnosticCode::DECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN, "float(decimal_value) is not allowed; decimal values are exact and cannot be converted to float" .to_string(), call.arguments.args[0].range(), ); return None; } else if arg_ty == Type::BigDecimal { ctx.error_with_code_at( DiagnosticCode::DECIMAL_BIGDECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN, "float(bigdecimal_value) is not allowed; bigdecimal values are exact and cannot be converted to float" .to_string(), call.arguments.args[0].range(), ); return None; } else { Type::Float }; return Some(HirExpr::Call { func: "float".to_string(), args: vec![arg], ty: result_ty, }); }  if func_name == "bool" { if !call.arguments.keywords.is_empty() { expression_diagnostics::call_unexpected_keyword( ctx, "bool() does not accept keyword arguments".to_string(), first_call_keyword_range(call), ); return None; } if call.arguments.args.len() != 1 { expression_diagnostics::call_wrong_positional_count( ctx, format!( "bool() takes exactly 1 argument, got {}", call.arguments.args.len() ), call_arity_range(call), ); return None; } let arg = lower_expr(&call.arguments.args[0], ctx)?; return Some(HirExpr::Call { func: "bool".to_string(), args: vec![arg], ty: Type::Bool, }); }  if func_name == "min" { if !call.arguments.keywords.is_empty() { expression_diagnostics::call_unexpected_keyword( ctx, "min() does not accept keyword arguments".to_string(), first_call_keyword_range(call), ); return None; } if call.arguments.args.len() >= 2 { let mut args = Vec::with_capacity(call.arguments.args.len()); for arg in &call.arguments.args { args.push(lower_expr(arg, ctx)?); }  let mut result_ty = args[0].ty().clone(); for index in 1..args.len() { let (left, right, pair_result_ty) = normalize_min_max_numeric_sentinels( &call.arguments.args[index - 1], &call.arguments.args[index], args[index - 1].clone(), args[index].clone(), ctx, ); args[index - 1] = left; args[index] = right; result_ty = pair_result_ty; }  if !validate_variadic_min_max_operands("min", &args, &call.arguments.args, ctx) { return None; } return Some(HirExpr::Call { func: "min".to_string(), args, ty: result_ty, }); } else if call.arguments.args.len() == 1 { let arg = lower_expr(&call.arguments.args[0], ctx)?; let Some(elem_ty) = callable_builtin_element_type(arg.ty()) else { expression_diagnostics::type_mismatch( ctx, format!( "min() argument must be an iterable with a statically-known element type, got '{}'", arg.ty().display_name() ), call.arguments.args[0].range(), ); return None; }; return Some(HirExpr::Call { func: "min".to_string(), args: vec![arg], ty: Type::Union(vec![elem_ty, Type::None]), }); } expression_diagnostics::call_wrong_positional_count( ctx, "min() takes at least 1 argument".to_string(), call.func.range(), ); return None; } if func_name == "max" { if !call.arguments.keywords.is_empty() { expression_diagnostics::call_unexpected_keyword( ctx, "max() does not accept keyword arguments".to_string(), first_call_keyword_range(call), ); return None; } if call.arguments.args.len() >= 2 { let mut args = Vec::with_capacity(call.arguments.args.len()); for arg in &call.arguments.args { args.push(lower_expr(arg, ctx)?); }  let mut result_ty = args[0].ty().clone(); for index in 1..args.len() { let (left, right, pair_result_ty) = normalize_min_max_numeric_sentinels( &call.arguments.args[index - 1], &call.arguments.args[index], args[index - 1].clone(), args[index].clone(), ctx, ); args[index - 1] = left; args[index] = right; result_ty = pair_result_ty; }  if !validate_variadic_min_max_operands("max", &args, &call.arguments.args, ctx) { return None; } return Some(HirExpr::Call { func: "max".to_string(), args, ty: result_ty, }); } else if call.arguments.args.len() == 1 { let arg = lower_expr(&call.arguments.args[0], ctx)?; let Some(elem_ty) = callable_builtin_element_type(arg.ty()) else { expression_diagnostics::type_mismatch( ctx, format!( "max() argument must be an iterable with a statically-known element type, got '{}'", arg.ty().display_name() ), call.arguments.args[0].range(), ); return None; }; return Some(HirExpr::Call { func: "max".to_string(), args: vec![arg], ty: Type::Union(vec![elem_ty, Type::None]), }); } expression_diagnostics::call_wrong_positional_count( ctx, "max() takes at least 1 argument".to_string(), call.func.range(), ); return None; } if func_name == "sum" { return lower_sum_call(call, ctx); } if func_name == "sorted" { return lower_sorted_call(call, ctx); }  if func_name == "reversed" { return lower_reversed_call(call, ctx); }  if func_name == "enumerate" { return lower_enumerate_call(call, ctx); }  if func_name == "zip" { return lower_zip_call(call, ctx); } }  if func_name == "any" { return lower_any_all_call(call, "any", ctx); }  if func_name == "all" { return lower_any_all_call(call, "all", ctx); }  if func_name == "map" { return lower_map_call(call, ctx); } if func_name == "filter" { return lower_filter_call(call, ctx); } if func_name == "open" { let n_args = call.arguments.args.len(); let _n_kwargs = call.arguments.keywords.len(); let path_arg = if n_args >= 1 { lower_expr(&call.arguments.args[0], ctx)? } else { expression_diagnostics::call_missing_required_argument( ctx, "open() requires at least 1 argument: open(path) or open(path, mode)".to_string(), call.func.range(), ); return None; }; let mode_arg = if n_args >= 2 { lower_expr(&call.arguments.args[1], ctx)? } else if let Some(kw) = call .arguments .keywords .iter() .find(|k| k.arg.as_deref() == Some("mode")) { lower_expr(&kw.value, ctx)? } else { HirExpr::StringLiteral("r".to_string()) }; let io_err_ty = Type::Class { name: "IOError".to_string(), fields: vec![("message".to_string(), Type::Str)], methods: vec![], parent_class: None, }; let file_handle_ty = Type::Class { name: "FileHandle".to_string(), fields: vec![ ("_handle".to_string(), Type::Int), ("_mode".to_string(), Type::Str), ], methods: vec![ ( "read".to_string(), FunctionType::all_borrow( vec![], Type::Result(Box::new(Type::Str), Box::new(io_err_ty.clone())), ), ), ( "write".to_string(), FunctionType::all_borrow( vec![("data".to_string(), Type::Str)], Type::Result(Box::new(Type::None), Box::new(io_err_ty.clone())), ), ), ( "readline".to_string(), FunctionType::all_borrow( vec![], Type::Result( Box::new(Type::Union(vec![Type::Str, Type::None])), Box::new(io_err_ty.clone()), ), ), ), ( "readlines".to_string(), FunctionType::all_borrow( vec![], Type::Result( Box::new(Type::List(Box::new(Type::Str))), Box::new(io_err_ty.clone()), ), ), ), ( "close".to_string(), FunctionType::all_borrow(vec![], Type::None), ), ( "read_bytes".to_string(), FunctionType::all_borrow( vec![], Type::Result(Box::new(Type::Bytes), Box::new(io_err_ty.clone())), ), ), ( "write_bytes".to_string(), FunctionType::all_borrow( vec![("data".to_string(), Type::Bytes)], Type::Result(Box::new(Type::None), Box::new(io_err_ty.clone())), ), ), ( "__enter__".to_string(), FunctionType::all_borrow( vec![], Type::Class { name: "FileHandle".to_string(), fields: vec![ ("_handle".to_string(), Type::Int), ("_mode".to_string(), Type::Str), ], methods: vec![], parent_class: None, }, ), ), ( "__exit__".to_string(), FunctionType::all_borrow(vec![], Type::None), ), ], parent_class: None, }; ctx.class_types .insert("FileHandle".to_string(), file_handle_ty.clone()); ctx.try_block_error_types.insert("IOError".to_string()); return Some(HirExpr::Call { func: "builtin_open".to_string(), args: vec![path_arg, mode_arg], ty: file_handle_ty, }); }  let callable_info = ctx.scope.lookup(&func_name).and_then(|info| { if let Type::Callable(ref param_types, ref conventions, ref ret_type) = info.ty { Some((param_types.clone(), conventions.clone(), *ret_type.clone())) } else { None } }); if let Some((param_types, conventions, ret_type)) = callable_info { let mut args = Vec::new(); for arg in &call.arguments.args { let expr = lower_expr(arg, ctx)?; args.push(expr); } if args.len() != param_types.len() { let range = if args.len() > param_types.len() { call.arguments.args[param_types.len()].range() } else { call.func.range() }; expression_diagnostics::call_not_callable_or_arity( ctx, format!( "callable '{}' expects {} argument(s), got {}", func_name, param_types.len(), args.len() ), range, ); return None; } for (i, (arg, param_ty)) in args.iter().zip(param_types.iter()).enumerate() { if !arg.ty().is_assignable_to(param_ty) { expression_diagnostics::type_mismatch( ctx, format!( "argument {} of callable '{}': expected '{}', got '{}'", i + 1, func_name, param_ty.display_name(), arg.ty().display_name() ), call.arguments.args[i].range(), ); } let convention = conventions .get(i) .copied() .unwrap_or(ParamConvention::borrow()); if convention.is_owned() { if let HirExpr::Name { name, ty } = arg { if ty.ownership() == OwnershipKind::Move { ctx.scope.mark_moved(name); } } } } return Some(HirExpr::Call { func: func_name, args, ty: ret_type, }); }  let callable_object_ft = ctx.scope .lookup(&func_name) .and_then(|info| match info.effective_type().resolve_alias() { Type::Class { methods, .. } | Type::Protocol { methods, .. } => methods .iter() .find(|(name, _)| name == "__call__") .map(|(_, ft)| ft.clone()), _ => None, }); if let Some(call_ft) = callable_object_ft { let Expr::Name(name_expr) = call.func.as_ref() else { expression_diagnostics::call_not_callable_or_arity( ctx, "only simple function calls are supported".to_string(), call.func.range(), ); return None; }; let object = lower_name(name_expr, ctx)?; let args = lower_signature_call_args(call, &format!("{func_name}.__call__"), &call_ft, None, ctx)?; return Some(HirExpr::MethodCall { object: Box::new(object), method: "__call__".to_string(), args, ty: *call_ft.return_type.clone(), }); }  let ft = ctx.functions.get(&func_name).cloned().or_else(|| { name_diagnostics::undefined_function(ctx, &func_name, call.func.range()); None })?; let is_async_function = ctx.async_functions.contains(&func_name); let is_async_generator_function = ctx.async_generator_functions.contains(&func_name); if is_async_function && !ctx.current_function_is_async { expression_diagnostics::type_mismatch( ctx, format!( "async function '{func_name}' cannot be called from sync code; call it from an async function and await the returned coroutine" ), call.func.range(), ); return None; } super::workload_annotations::reject_async_direct_call(ctx, &func_name, call.func.range()); let call_defaults = ctx.function_defaults.get(&func_name).cloned(); let call_vararg = ctx.vararg_functions.get(&func_name).copied();  let args = if func_name == "print" { let mut args = Vec::with_capacity(call.arguments.args.len()); for arg in &call.arguments.args { args.push(lower_expr(arg, ctx)?); } args } else if func_name.ends_with("_Counter") && ft.params.len() == 2 && call.arguments.args.len() == 1 && call.arguments.keywords.is_empty() { let lowered_arg = lower_expr(&call.arguments.args[0], ctx)?; let source_ty = &ft.params[0].1; let iterable_ty = &ft.params[1].1; if lowered_arg.ty().is_assignable_to(source_ty) || is_compatible_with_unresolved_typevars(lowered_arg.ty(), source_ty) { vec![lowered_arg, HirExpr::NoneLiteral] } else if lowered_arg.ty().is_assignable_to(iterable_ty) || is_compatible_with_unresolved_typevars(lowered_arg.ty(), iterable_ty) { vec![HirExpr::NoneLiteral, lowered_arg] } else if matches!(lowered_arg.ty().resolve_alias(), Type::Str) { let iterable_arg = HirExpr::Call { func: "list".to_string(), args: vec![lowered_arg], ty: Type::List(Box::new(Type::Str)), }; if iterable_arg.ty().is_assignable_to(iterable_ty) || is_compatible_with_unresolved_typevars(iterable_arg.ty(), iterable_ty) { vec![HirExpr::NoneLiteral, iterable_arg] } else { lower_function_call_args( call, &func_name, &ft, call_defaults.as_deref(), call_vararg, ctx, )? } } else { lower_function_call_args( call, &func_name, &ft, call_defaults.as_deref(), call_vararg, ctx, )? } } else { lower_function_call_args( call, &func_name, &ft, call_defaults.as_deref(), call_vararg, ctx, )? };  let arg_ranges = call_argument_ranges_by_param(call, &ft);  if func_name != "print" { let is_generic_function = ctx.generic_functions.contains_key(&func_name); for (i, (arg, (param_name, param_ty, _))) in args.iter().zip(ft.params.iter()).enumerate() { if is_generic_function { let mut type_vars = Vec::new(); collect_type_vars(param_ty, &mut type_vars); if !type_vars.is_empty() { continue; } } if !arg.ty().is_assignable_to(param_ty) { let primary_range = arg_ranges .get(i) .copied() .flatten() .unwrap_or_else(|| call.range()); ctx.error_with_code_at( DiagnosticCode::TYPE_MISMATCH, format!( "argument {} ('{}') of function '{}': expected '{}', got '{}'", i + 1, param_name, func_name, param_ty.display_name(), arg.ty().display_name() ), primary_range, ); } } }  { let mut mut_borrowed: Vec<String> = Vec::new(); let mut immut_borrowed: Vec<String> = Vec::new(); for (i, arg) in args.iter().enumerate() { if let HirExpr::Name { name, ty } = arg { if ty.ownership() == sifr_type_system::OwnershipKind::Move { let primary_range = arg_ranges .get(i) .copied() .flatten() .unwrap_or_else(|| call.range()); let convention = ft .params .get(i) .map(|(_, _, c)| *c) .unwrap_or(ParamConvention::borrow()); if convention.is_mut_borrow() { if mut_borrowed.contains(name) { ownership_diagnostics::double_mutable_borrow( ctx, name, &func_name, primary_range, ); } else if immut_borrowed.contains(name) { ownership_diagnostics::mutable_borrow_after_immutable( ctx, name, &func_name, primary_range, ); } mut_borrowed.push(name.clone()); } else if convention.is_shared_borrow() { if mut_borrowed.contains(name) { ownership_diagnostics::immutable_borrow_after_mutable( ctx, name, &func_name, primary_range, ); } immut_borrowed.push(name.clone()); } else { } } } } }  for (i, arg) in args.iter().enumerate() { if let HirExpr::Name { name, ty } = arg { if ty.ownership() == sifr_type_system::OwnershipKind::Move { let convention = ft .params .get(i) .map(|(_, _, c)| *c) .unwrap_or(ParamConvention::borrow()); if convention.is_owned() { ctx.scope.mark_moved(name); } } } }  let return_type = if ctx.generic_functions.contains_key(&func_name) { let mut bindings = HashMap::new(); for (arg, (_, param_ty, _)) in args.iter().zip(ft.params.iter()) { infer_type_var_bindings(param_ty, arg.ty(), &mut bindings); } if func_name != "print" { for (i, (arg, (param_name, param_ty, _))) in args.iter().zip(ft.params.iter()).enumerate() { let concrete_param_ty = substitute_type_vars(param_ty, &bindings); let mut unresolved_type_vars = Vec::new(); collect_type_vars(&concrete_param_ty, &mut unresolved_type_vars); if !unresolved_type_vars.is_empty() { if !is_compatible_with_unresolved_typevars(arg.ty(), &concrete_param_ty) { let primary_range = arg_ranges .get(i) .copied() .flatten() .unwrap_or_else(|| call.range()); ctx.error_with_code_at( DiagnosticCode::TYPE_MISMATCH, format!( "argument {} ('{}') of function '{}': expected '{}', got '{}'", i + 1, param_name, func_name, concrete_param_ty.display_name(), arg.ty().display_name() ), primary_range, ); } continue; } if !arg.ty().is_assignable_to(&concrete_param_ty) { let primary_range = arg_ranges .get(i) .copied() .flatten() .unwrap_or_else(|| call.range()); ctx.error_with_code_at( DiagnosticCode::TYPE_MISMATCH, format!( "argument {} ('{}') of function '{}': expected '{}', got '{}'", i + 1, param_name, func_name, concrete_param_ty.display_name(), arg.ty().display_name() ), primary_range, ); } } } if let Some(owner_bounds) = ctx.type_param_bounds.get(&func_name) { let owner_bounds = owner_bounds.clone(); for (tv_name, concrete_ty) in &bindings { if let Some(specs) = owner_bounds.get(tv_name) { let mut required_bounds = Vec::new(); let mut constraints = Vec::new(); for spec in specs { if let Some(constraint_name) = decode_typevar_constraint(spec) { constraints.push(constraint_name.to_string()); } else { required_bounds.push(spec.clone()); } }  for bound in required_bounds { if !type_satisfies_bound(concrete_ty, &bound, ctx) { protocol_diagnostics::bound_not_satisfied( ctx, &concrete_ty.display_name(), &bound, tv_name, call.range(), ); } }  if !constraints.is_empty() && !constraints.iter().any(|constraint| { type_satisfies_constraint(concrete_ty, constraint, ctx) }) { let primary_range = type_param_argument_range(call, &ft, tv_name) .unwrap_or_else(|| call.range()); ctx.error_with_code_at( DiagnosticCode::TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED, format!( "type '{actual}' does not satisfy constraints ({constraints}) required by type parameter '{type_param}'", actual = concrete_ty.display_name(), constraints = constraints.join(", "), type_param = tv_name ), primary_range, ); } } } } if bindings.is_empty() { ft.return_type.as_ref().clone() } else { substitute_type_vars(&ft.return_type, &bindings) } } else { ft.return_type.as_ref().clone() };  let return_type = refine_constructor_return_type_from_args(&ft, &args, &return_type); tsc::validate_shared_constructor(&func_name, &args, &arg_ranges, call, ctx); let call_type = if is_async_function && !is_async_generator_function { coroutine_result_type(&return_type) } else { return_type };  if ctx.class_types.contains_key(&func_name) { Some(HirExpr::ConstructorCall { class_name: func_name, args, ty: call_type, }) } else { Some(HirExpr::Call { func: func_name, args, ty: call_type, }) } } 
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

