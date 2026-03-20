use super::arithmetic_warnings::check_int_overflow_risk;
use super::builtin_calls::{
    callable_builtin_element_type, callable_builtin_list_output_type,
    lower_builtin_reverseable_arg, lower_bytes_constructor_call, lower_bytes_type_factory_call,
    lower_chr_call, lower_defaultdict_constructor_call, lower_dict_constructor_call,
    lower_isinstance_call, lower_len_call, lower_list_constructor_call, lower_ord_call,
    lower_range_call, lower_reveal_type_call, lower_set_constructor_call,
    lower_tuple_constructor_call, DEFAULTDICT_INT_ALIAS, DEFAULTDICT_LIST_ALIAS,
    DEFAULTDICT_SET_ALIAS,
};
use super::bytes_methods::{resolve_bytes_method_type, resolve_str_encode_method_type};
use super::classes::is_hashable_type;
use super::compat_imports::{
    resolve_bare_python_compat_call_alias, resolve_python_compat_call_alias,
};
use super::decimal_methods::{
    decimal_conversion_error_type, resolve_decimal_method_type, validate_bigdecimal_string_literal,
    validate_decimal_string_literal,
};
use super::guarded_index::guarded_sequence_index_result_type;
use super::method_call_args::{
    lower_function_call_args, lower_method_call_args, lower_signature_call_args,
    validate_dict_update_arg, validate_list_extend_arg, validate_set_iterable_arg,
};
use super::mutating_methods::reject_immutable_parameter_method_mutation;
use super::numeric_sentinels::{
    float_sentinel_expr, float_sentinel_kind_from_call, lower_sentinel_expr_for_name_domain,
    maybe_resolve_numeric_sentinel_name_from_type, normalize_min_max_numeric_sentinels,
    retag_numeric_sentinel_name_expr,
};
pub(super) use super::tuple_unpack::{lower_star_unpack_assign, lower_tuple_unpack_assign};
use super::type_bounds::{type_satisfies_bound, type_satisfies_constraint};
use super::typing_and_functions::resolve_annotation_expr;
use super::{
    collect_type_vars, decode_typevar_constraint, infer_type_var_bindings, substitute_type_vars,
    LowerCtx,
};
use crate::hir_nodes::{HirExpr, HirFStringPart, HirIteratorOp, HirParam};
use sifr_python_ast::{
    BoolOp, CmpOp, Expr, ExprAttribute, ExprBinOp, ExprBoolOp, ExprBytesLiteral, ExprCall,
    ExprCompare, ExprDict, ExprDictComp, ExprFString, ExprGenerator, ExprIf, ExprLambda, ExprList,
    ExprListComp, ExprName, ExprNamed, ExprNumberLiteral, ExprSet, ExprSetComp, ExprSubscript,
    ExprTuple, ExprUnaryOp, FStringElement, Number, Operator, UnaryOp,
};
use sifr_type_system::{
    make_union, type_check_binary_op, type_check_bool_op, type_check_comparison,
    type_check_unary_op, FunctionType, OwnershipKind, ParamConvention, Type,
};
use std::collections::HashMap;

pub(super) fn lower_expr(expr: &Expr, ctx: &mut LowerCtx) -> Option<HirExpr> {
    match expr {
        Expr::NumberLiteral(num) => lower_number_literal(num),
        Expr::BytesLiteral(bytes) => lower_bytes_literal(bytes),
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
        Expr::If(if_expr) => lower_if_expr(if_expr, ctx),
        Expr::List(list) => lower_list_literal(list, ctx),
        Expr::Set(set) => lower_set_literal(set, ctx),
        Expr::Dict(dict) => lower_dict_literal(dict, ctx),
        Expr::Tuple(tuple) => lower_tuple_literal(tuple, ctx),
        Expr::Subscript(sub) => lower_subscript(sub, ctx),
        Expr::Attribute(attr) => lower_attribute(attr, ctx),
        Expr::FString(fstring) => lower_fstring(fstring, ctx),
        Expr::Named(named) => lower_named_expr(named, ctx),
        Expr::Lambda(lambda) => lower_lambda(lambda, ctx),
        Expr::ListComp(comp) => lower_list_comp(comp, ctx),
        Expr::SetComp(comp) => lower_set_comp(comp, ctx),
        Expr::DictComp(comp) => lower_dict_comp(comp, ctx),
        Expr::Generator(gen) => lower_generator_expr(gen, ctx),
        _ => {
            ctx.error("unsupported expression type".to_string());
            None
        }
    }
}

pub(super) fn lower_number_literal(num: &ExprNumberLiteral) -> Option<HirExpr> {
    match &num.value {
        Number::Int(i) => {
            let val = i.as_i64()?;
            Some(HirExpr::IntLiteral(val))
        }
        Number::Float(f) => Some(HirExpr::FloatLiteral(*f)),
        Number::Complex { .. } => None, // Not supported in M1
    }
}

pub(super) fn lower_bytes_literal(bytes: &ExprBytesLiteral) -> Option<HirExpr> {
    let mut elements = Vec::new();
    for part in bytes.value.iter() {
        for value in part.as_slice() {
            elements.push(HirExpr::IntLiteral(i64::from(*value)));
        }
    }
    Some(HirExpr::ListLiteral {
        elements,
        ty: Type::Bytes,
    })
}

fn callable_signature(expr: &HirExpr) -> Option<(Vec<Type>, Vec<ParamConvention>, Type)> {
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
    let var_name = name.id.clone();

    // Check if it's a known variable
    if let Some(info) = ctx.scope.lookup(&var_name) {
        let is_moved = info.is_moved;
        // Use effective type (narrowed if available)
        let ty = info.effective_type().clone();
        if is_moved {
            ctx.error(format!("use of moved value: '{var_name}'"));
        }
        return Some(HirExpr::Name { name: var_name, ty });
    }

    // Check if it's a known function
    if let Some(ft) = ctx.functions.get(&var_name) {
        let ft = ft.clone();
        return Some(HirExpr::Name {
            name: var_name,
            ty: Type::Function(ft),
        });
    }

    // Check built-in constants
    match var_name.as_str() {
        "True" => return Some(HirExpr::BoolLiteral(true)),
        "False" => return Some(HirExpr::BoolLiteral(false)),
        _ => {}
    }

    ctx.error(format!("undefined variable: '{var_name}'"));
    None
}

/// Map a binary operator to its corresponding dunder method name.
pub(super) fn op_to_dunder(op: &str) -> Option<&'static str> {
    match op {
        "+" => Some("__add__"),
        "-" => Some("__sub__"),
        "*" => Some("__mul__"),
        "/" => Some("__truediv__"),
        "//" => Some("__floordiv__"),
        "%" => Some("__mod__"),
        "**" => Some("__pow__"),
        _ => None,
    }
}

/// Shape compatibility used when generic inference leaves unresolved `TypeVar`s.
/// `TypeVar`s are treated as wildcards, but container/class structure must still match.
fn is_compatible_with_unresolved_typevars(source: &Type, target: &Type) -> bool {
    match target {
        Type::TypeVar(_) => true,
        Type::List(target_elem) => match source {
            Type::List(source_elem) => {
                is_compatible_with_unresolved_typevars(source_elem, target_elem)
            }
            _ => false,
        },
        Type::Set(target_elem) => match source {
            Type::Set(source_elem) => {
                is_compatible_with_unresolved_typevars(source_elem, target_elem)
            }
            _ => false,
        },
        Type::Dict(target_key, target_val) => match source {
            Type::Dict(source_key, source_val) => {
                is_compatible_with_unresolved_typevars(source_key, target_key)
                    && is_compatible_with_unresolved_typevars(source_val, target_val)
            }
            _ => false,
        },
        Type::Tuple(target_elems) => match source {
            Type::Tuple(source_elems) => {
                source_elems.len() == target_elems.len()
                    && source_elems
                        .iter()
                        .zip(target_elems.iter())
                        .all(|(src, dst)| is_compatible_with_unresolved_typevars(src, dst))
            }
            _ => false,
        },
        Type::Result(target_ok, target_err) => match source {
            Type::Result(source_ok, source_err) => {
                is_compatible_with_unresolved_typevars(source_ok, target_ok)
                    && is_compatible_with_unresolved_typevars(source_err, target_err)
            }
            _ => false,
        },
        Type::Class {
            name: target_name, ..
        } => match source {
            Type::Class {
                name: source_name, ..
            } => source_name == target_name,
            _ => false,
        },
        Type::Union(target_members) => target_members
            .iter()
            .any(|member| is_compatible_with_unresolved_typevars(source, member)),
        _ => source.is_assignable_to(target),
    }
}

pub(super) fn lower_binop(binop: &ExprBinOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let left = lower_expr(&binop.left, ctx)?;
    let right = lower_expr(&binop.right, ctx)?;

    let op_str = match binop.op {
        Operator::Add => "+",
        Operator::Sub => "-",
        Operator::Mult => "*",
        Operator::Div => "/",
        Operator::FloorDiv => "//",
        Operator::Mod => "%",
        Operator::Pow => "**",
        Operator::BitAnd => "&",
        Operator::BitOr => "|",
        Operator::BitXor => "^",
        Operator::LShift => "<<",
        Operator::RShift => ">>",
        Operator::MatMult => {
            ctx.error("matrix multiplication operator (@) is not supported".to_string());
            return None;
        }
    };

    match type_check_binary_op(left.ty(), op_str, right.ty()) {
        Ok(result_ty) => {
            if result_ty == Type::Int {
                check_int_overflow_risk(op_str, &left, &right, ctx);
            }
            Some(HirExpr::BinOp {
                left: Box::new(left),
                op: op_str.to_string(),
                right: Box::new(right),
                ty: result_ty,
            })
        }
        Err(e) => {
            // Check for operator overloading on class types
            if let Type::Class { methods, .. } = left.ty() {
                if let Some(dunder) = op_to_dunder(op_str) {
                    if let Some((_, ft)) = methods.iter().find(|(n, _)| n == dunder) {
                        let result_ty = *ft.return_type.clone();
                        return Some(HirExpr::BinOp {
                            left: Box::new(left),
                            op: op_str.to_string(),
                            right: Box::new(right),
                            ty: result_ty,
                        });
                    }
                }
            }
            ctx.error(e.message);
            None
        }
    }
}

pub(super) fn lower_unaryop(unary: &ExprUnaryOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let operand = lower_expr(&unary.operand, ctx)?;

    let op_str = match unary.op {
        UnaryOp::USub => "-",
        UnaryOp::UAdd => "+",
        UnaryOp::Not => "not",
        UnaryOp::Invert => "~",
    };

    match type_check_unary_op(op_str, operand.ty()) {
        Ok(result_ty) => Some(HirExpr::UnaryOp {
            op: op_str.to_string(),
            operand: Box::new(operand),
            ty: result_ty,
        }),
        Err(e) => {
            ctx.error(e.message);
            None
        }
    }
}

pub(super) fn lower_compare(cmp: &ExprCompare, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut left = lower_expr(&cmp.left, ctx)?;

    // Handle `in` and `not in` operators specially
    if cmp.ops.len() == 1 {
        match &cmp.ops[0] {
            CmpOp::In => {
                let mut collection = lower_expr(&cmp.comparators[0], ctx)?;
                collection = refine_empty_set_binding_expr(collection, left.ty().clone(), ctx);
                let collection_ty = collection.ty().clone();
                if let Some(elem_ty) = collection_ty.contains_element_type() {
                    if !left.ty().is_assignable_to(&elem_ty) {
                        ctx.error(format!(
                            "'in' operator: element type '{}' is not compatible with collection element type '{}'",
                            left.ty().display_name(),
                            elem_ty.display_name()
                        ));
                    }
                } else {
                    ctx.error(format!(
                        "'in' operator not supported for type '{}'",
                        collection_ty.display_name()
                    ));
                }
                return Some(HirExpr::ContainsOp {
                    element: Box::new(left),
                    collection: Box::new(collection),
                    ty: Type::Bool,
                });
            }
            CmpOp::NotIn => {
                let mut collection = lower_expr(&cmp.comparators[0], ctx)?;
                collection = refine_empty_set_binding_expr(collection, left.ty().clone(), ctx);
                let collection_ty = collection.ty().clone();
                if let Some(elem_ty) = collection_ty.contains_element_type() {
                    if !left.ty().is_assignable_to(&elem_ty) {
                        ctx.error(format!(
                            "'not in' operator: element type '{}' is not compatible with collection element type '{}'",
                            left.ty().display_name(),
                            elem_ty.display_name()
                        ));
                    }
                } else {
                    ctx.error(format!(
                        "'not in' operator not supported for type '{}'",
                        collection_ty.display_name()
                    ));
                }
                // Wrap in a UnaryOp not
                let contains = HirExpr::ContainsOp {
                    element: Box::new(left),
                    collection: Box::new(collection),
                    ty: Type::Bool,
                };
                return Some(HirExpr::UnaryOp {
                    op: "not".to_string(),
                    operand: Box::new(contains),
                    ty: Type::Bool,
                });
            }
            _ => {}
        }
    }

    let mut ops = Vec::new();
    let mut comparators = Vec::new();

    for (op, comparator) in cmp.ops.iter().zip(cmp.comparators.iter()) {
        let op_str = match op {
            CmpOp::Eq => "==",
            CmpOp::NotEq => "!=",
            CmpOp::Lt => "<",
            CmpOp::Gt => ">",
            CmpOp::LtE => "<=",
            CmpOp::GtE => ">=",
            CmpOp::Is => "is",
            CmpOp::IsNot => "is not",
            _ => {
                ctx.error("unsupported comparison operator".to_string());
                return None;
            }
        };

        let mut right = if let Some(retagged_right) =
            lower_sentinel_expr_for_name_domain(comparator, &left, ctx)
        {
            retagged_right
        } else {
            lower_expr(comparator, ctx)?
        };
        maybe_resolve_numeric_sentinel_name_from_type(&left, right.ty(), ctx);
        maybe_resolve_numeric_sentinel_name_from_type(&right, left.ty(), ctx);
        left = retag_numeric_sentinel_name_expr(left, ctx);
        if let Some(retagged_right) = lower_sentinel_expr_for_name_domain(comparator, &left, ctx) {
            right = retagged_right;
        } else {
            right = retag_numeric_sentinel_name_expr(right, ctx);
        }

        // `is` and `is not` are identity checks (used for None comparison)
        // They don't need type_check_comparison
        if op_str != "is" && op_str != "is not" {
            if let Err(e) = type_check_comparison(left.ty(), op_str, right.ty()) {
                // Check for operator overloading on class types
                let has_overload = match left.ty() {
                    Type::Class { methods, .. } => {
                        let dunder = match op_str {
                            "==" | "!=" => "__eq__",
                            "<" | ">" | "<=" | ">=" => "__lt__",
                            _ => "",
                        };
                        !dunder.is_empty() && methods.iter().any(|(n, _)| n == dunder)
                    }
                    _ => false,
                };
                if !has_overload {
                    ctx.error(e.message);
                    return None;
                }
            }
        }

        ops.push(op_str.to_string());
        comparators.push(right);
    }

    Some(HirExpr::Compare {
        left: Box::new(left),
        ops,
        comparators,
        ty: Type::Bool,
    })
}

pub(super) fn lower_boolop(boolop: &ExprBoolOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let op_str = match boolop.op {
        BoolOp::And => "and",
        BoolOp::Or => "or",
    };

    let mut values = Vec::new();
    for val in &boolop.values {
        let expr = lower_expr(val, ctx)?;
        values.push(expr);
    }

    // Check all values are Bool
    for val in &values {
        if let Err(e) = type_check_bool_op(val.ty(), op_str, &Type::Bool) {
            ctx.error(e.message);
            return None;
        }
    }

    Some(HirExpr::BoolOp {
        op: op_str.to_string(),
        values,
        ty: Type::Bool,
    })
}

pub(super) fn lower_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let compat_alias = resolve_python_compat_call_alias(call, ctx);
    if let (None, Expr::Attribute(attr)) = (&compat_alias, call.func.as_ref()) {
        if let Some(factory_call) = lower_bytes_type_factory_call(attr, call, ctx) {
            return Some(factory_call);
        }
        return lower_method_call(attr, call, ctx);
    }
    let func_name = if let Some(alias) = compat_alias {
        alias
    } else if let Expr::Name(n) = call.func.as_ref() {
        resolve_bare_python_compat_call_alias(n.id.as_str(), ctx).unwrap_or_else(|| n.id.clone())
    } else {
        ctx.error("only simple function calls are supported".to_string());
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
                ctx.error("iter() does not accept keyword arguments".to_string());
                return None;
            }
            if call.arguments.args.len() != 1 {
                ctx.error(format!(
                    "iter() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ));
                return None;
            }
            let iterable = lower_expr(&call.arguments.args[0], ctx)?;
            if matches!(iterable.ty().resolve_alias(), Type::Any | Type::Unknown) {
                ctx.error(format!(
                    "iter() argument must be an iterable with a statically-known element type, got '{}'",
                    iterable.ty().display_name()
                ));
                return None;
            }
            let Some(elem_ty) = callable_builtin_element_type(iterable.ty()) else {
                if matches!(iterable.ty().resolve_alias(), Type::Tuple(_)) {
                    ctx.error(
                        "iter() tuple argument must have one statically provable element type"
                            .to_string(),
                    );
                    return None;
                }
                ctx.error(format!(
                    "iter() argument must be iterable, got '{}'",
                    iterable.ty().display_name()
                ));
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
                ctx.error("next() does not accept keyword arguments".to_string());
                return None;
            }
            if call.arguments.args.len() != 1 {
                ctx.error(format!(
                    "next() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ));
                return None;
            }
            let iterator = lower_expr(&call.arguments.args[0], ctx)?;
            let elem_ty = match iterator.ty().resolve_alias() {
                Type::Iterator(elem) => *elem.clone(),
                _ => {
                    ctx.error(format!(
                        "next() argument must be an iterator, got '{}'",
                        iterator.ty().display_name()
                    ));
                    return None;
                }
            };
            return Some(HirExpr::IteratorCall {
                op: HirIteratorOp::Next,
                args: vec![iterator],
                ty: Type::Union(vec![elem_ty, Type::None]),
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
            if call.arguments.args.len() != 2 {
                ctx.error("pow() takes exactly 2 arguments".to_string());
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
            if call.arguments.args.len() != 1 {
                ctx.error(format!(
                    "abs() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ));
                return None;
            }
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            let ty = arg.ty().clone();
            if !ty.is_numeric() {
                ctx.error(format!(
                    "abs() argument must be numeric, got '{}'",
                    ty.display_name()
                ));
                return None;
            }
            return Some(HirExpr::Call {
                func: "abs".to_string(),
                args: vec![arg],
                ty,
            });
        }

        // Special handling for hash() built-in
        if func_name == "hash" {
            if call.arguments.args.len() != 1 {
                ctx.error(format!(
                    "hash() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ));
                return None;
            }
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            let ty = arg.ty().clone();
            // Check if the type is hashable
            if !is_hashable_type(&ty) {
                ctx.error(format!(
                    "hash() argument must be hashable, got '{}'",
                    ty.display_name()
                ));
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
            if call.arguments.args.is_empty() || call.arguments.args.len() > 2 {
                ctx.error(format!(
                    "round() takes 1 or 2 arguments, got {}",
                    call.arguments.args.len()
                ));
                return None;
            }
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            if !arg.ty().is_numeric() {
                ctx.error(format!(
                    "round() argument must be numeric, got '{}'",
                    arg.ty().display_name()
                ));
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
            if call.arguments.args.len() != 1 {
                ctx.error(format!(
                    "repr() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ));
                return None;
            }
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            return Some(HirExpr::Call {
                func: "repr".to_string(),
                args: vec![arg],
                ty: Type::Str,
            });
        }

        // Decimal("...") / Decimal(int|bigint|bigdecimal)
        if func_name == "Decimal" {
            if call.arguments.args.len() != 1 {
                ctx.error(format!(
                    "[E2505] Decimal() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ));
                return None;
            }
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            let arg_ty = arg.ty().clone();
            let decimal_conversion_error_ty = ctx
                .class_types
                .get("DecimalConversionError")
                .cloned()
                .unwrap_or(Type::Class {
                    name: "DecimalConversionError".to_string(),
                    fields: vec![("message".to_string(), Type::Str)],
                    methods: vec![],
                    parent_class: None,
                });
            let result_ty = match arg_ty {
                Type::Str => {
                    if let Expr::StringLiteral(lit) = &call.arguments.args[0] {
                        validate_decimal_string_literal(lit.value.to_str(), ctx)?;
                    } else {
                        ctx.error(
                            "[E2501] Decimal() string construction requires a string literal"
                                .to_string(),
                        );
                        return None;
                    }
                    Type::Decimal
                }
                Type::Int | Type::LiteralInt(_) | Type::Decimal => Type::Decimal,
                Type::BigInt | Type::BigDecimal => Type::Result(
                    Box::new(Type::Decimal),
                    Box::new(decimal_conversion_error_ty),
                ),
                Type::Float => {
                    ctx.error(
                    "[E2505] Decimal(float_value) is not allowed; use Decimal(\"...\") for exact construction"
                        .to_string(),
                );
                    return None;
                }
                _ => {
                    ctx.error(format!(
                    "[E2505] Decimal() requires str, int, bigint, decimal, or bigdecimal argument, got '{}'",
                    arg_ty.display_name()
                ));
                    return None;
                }
            };
            return Some(HirExpr::Call {
                func: "Decimal".to_string(),
                args: vec![arg],
                ty: result_ty,
            });
        }

        // BigDecimal("...") / BigDecimal(int|bigint|decimal)
        if func_name == "BigDecimal" {
            if call.arguments.args.len() != 1 {
                ctx.error(format!(
                    "[E2506] BigDecimal() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ));
                return None;
            }
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            let arg_ty = arg.ty().clone();
            match arg_ty {
                Type::Str => {
                    if let Expr::StringLiteral(lit) = &call.arguments.args[0] {
                        validate_bigdecimal_string_literal(lit.value.to_str(), ctx)?;
                    } else {
                        ctx.error(
                            "[E2502] BigDecimal() string construction requires a string literal"
                                .to_string(),
                        );
                        return None;
                    }
                }
                Type::Int
                | Type::LiteralInt(_)
                | Type::BigInt
                | Type::Decimal
                | Type::BigDecimal => {}
                Type::Float => {
                    ctx.error(
                    "[E2506] BigDecimal(float_value) is not allowed; use BigDecimal(\"...\") for exact construction"
                        .to_string(),
                );
                    return None;
                }
                _ => {
                    ctx.error(format!(
                    "[E2506] BigDecimal() requires str, int, bigint, decimal, or bigdecimal argument, got '{}'",
                    arg_ty.display_name()
                ));
                    return None;
                }
            }
            return Some(HirExpr::Call {
                func: "BigDecimal".to_string(),
                args: vec![arg],
                ty: Type::BigDecimal,
            });
        }

        // Special handling for int() conversion
        if func_name == "int" {
            if call.arguments.args.len() != 1 {
                ctx.error(format!(
                    "int() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ));
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
            if call.arguments.args.len() != 1 {
                ctx.error(format!(
                    "bigint() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ));
                return None;
            }
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            let arg_ty = arg.ty().clone();
            if !matches!(
                arg_ty,
                Type::Int | Type::LiteralInt(_) | Type::BigInt | Type::Decimal | Type::BigDecimal
            ) {
                ctx.error(format!(
                    "bigint() requires int, bigint, decimal, or bigdecimal argument, got '{}'",
                    arg_ty.display_name()
                ));
                return None;
            }
            return Some(HirExpr::Call {
                func: "bigint".to_string(),
                args: vec![arg],
                ty: Type::BigInt,
            });
        }

        // Special handling for float() conversion
        if func_name == "float" {
            if call.arguments.args.len() != 1 {
                ctx.error(format!(
                    "float() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ));
                return None;
            }
            if let Some(kind) = float_sentinel_kind_from_call(call) {
                return Some(float_sentinel_expr(kind));
            }
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            let arg_ty = arg.ty().clone();
            // float(str) -> Result[float, ParseError] (fallible)
            // float(int) -> float (infallible widening)
            // float(float) -> float (identity)
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
                ctx.error(
                "[E2505] float(decimal_value) is not allowed; decimal values are exact and cannot be converted to float"
                    .to_string(),
            );
                return None;
            } else if arg_ty == Type::BigDecimal {
                ctx.error(
                "[E2506] float(bigdecimal_value) is not allowed; bigdecimal values are exact and cannot be converted to float"
                    .to_string(),
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
            if call.arguments.args.len() != 1 {
                ctx.error(format!(
                    "bool() takes exactly 1 argument, got {}",
                    call.arguments.args.len()
                ));
                return None;
            }
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            return Some(HirExpr::Call {
                func: "bool".to_string(),
                args: vec![arg],
                ty: Type::Bool,
            });
        }

        // --- Built-in generic functions ---

        // min(iterable) or min(a, b) -> element type
        if func_name == "min" {
            if call.arguments.args.len() == 2 {
                // min(a, b) -> std::cmp::min(a, b)
                let a = lower_expr(&call.arguments.args[0], ctx)?;
                let b = lower_expr(&call.arguments.args[1], ctx)?;
                let (a, b, result_ty) = normalize_min_max_numeric_sentinels(
                    &call.arguments.args[0],
                    &call.arguments.args[1],
                    a,
                    b,
                    ctx,
                );
                return Some(HirExpr::Call {
                    func: "min".to_string(),
                    args: vec![a, b],
                    ty: result_ty,
                });
            } else if call.arguments.args.len() == 1 {
                let arg = lower_expr(&call.arguments.args[0], ctx)?;
                let elem_ty = if let Type::List(elem) = arg.ty() {
                    *elem.clone()
                } else {
                    ctx.error(format!(
                        "min() argument must be a list, got '{}'",
                        arg.ty().display_name()
                    ));
                    return None;
                };
                // Returns Option[T] = T | None (safe: None on empty list)
                return Some(HirExpr::Call {
                    func: "min".to_string(),
                    args: vec![arg],
                    ty: Type::Union(vec![elem_ty, Type::None]),
                });
            }
            ctx.error("min() takes 1 or 2 arguments".to_string());
            return None;
        }

        // max(iterable) or max(a, b) -> element type
        if func_name == "max" {
            if call.arguments.args.len() == 2 {
                // max(a, b) -> std::cmp::max(a, b)
                let a = lower_expr(&call.arguments.args[0], ctx)?;
                let b = lower_expr(&call.arguments.args[1], ctx)?;
                let (a, b, result_ty) = normalize_min_max_numeric_sentinels(
                    &call.arguments.args[0],
                    &call.arguments.args[1],
                    a,
                    b,
                    ctx,
                );
                return Some(HirExpr::Call {
                    func: "max".to_string(),
                    args: vec![a, b],
                    ty: result_ty,
                });
            } else if call.arguments.args.len() == 1 {
                let arg = lower_expr(&call.arguments.args[0], ctx)?;
                let elem_ty = if let Type::List(elem) = arg.ty() {
                    *elem.clone()
                } else {
                    ctx.error(format!(
                        "max() argument must be a list, got '{}'",
                        arg.ty().display_name()
                    ));
                    return None;
                };
                // Returns Option[T] = T | None (safe: None on empty list)
                return Some(HirExpr::Call {
                    func: "max".to_string(),
                    args: vec![arg],
                    ty: Type::Union(vec![elem_ty, Type::None]),
                });
            }
            ctx.error("max() takes 1 or 2 arguments".to_string());
            return None;
        }

        // sum(iterable) -> element type (int or float)
        if func_name == "sum" {
            if call.arguments.args.len() != 1 {
                ctx.error("sum() takes exactly 1 argument".to_string());
                return None;
            }
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            let elem_ty = if let Type::List(elem) = arg.ty() {
                *elem.clone()
            } else {
                ctx.error(format!(
                    "sum() argument must be a list, got '{}'",
                    arg.ty().display_name()
                ));
                return None;
            };
            return Some(HirExpr::Call {
                func: "sum".to_string(),
                args: vec![arg],
                ty: elem_ty,
            });
        }

        // sorted(iterable) -> list of element type
        if func_name == "sorted" {
            if call.arguments.args.len() > 1 {
                ctx.error("sorted() takes at most 1 positional argument".to_string());
                return None;
            }
            let mut iterable_keyword = None;
            let mut key_keyword = None;
            let mut reverse_keyword = None;
            for keyword in &call.arguments.keywords {
                let Some(name) = keyword.arg.as_ref() else {
                    ctx.error("sorted() does not support unpacked keyword arguments".to_string());
                    return None;
                };
                match name.as_str() {
                    "iterable" => {
                        if iterable_keyword.is_some() {
                            ctx.error(
                                "sorted() got multiple values for keyword argument 'iterable'"
                                    .to_string(),
                            );
                            return None;
                        }
                        iterable_keyword = Some(keyword);
                    }
                    "key" => {
                        if key_keyword.is_some() {
                            ctx.error(
                                "sorted() got multiple values for keyword argument 'key'"
                                    .to_string(),
                            );
                            return None;
                        }
                        key_keyword = Some(keyword);
                    }
                    "reverse" => {
                        if reverse_keyword.is_some() {
                            ctx.error(
                                "sorted() got multiple values for keyword argument 'reverse'"
                                    .to_string(),
                            );
                            return None;
                        }
                        reverse_keyword = Some(keyword);
                    }
                    other => {
                        ctx.error(format!(
                            "sorted() got an unexpected keyword argument '{other}'"
                        ));
                        return None;
                    }
                }
            }
            let iterable = match (call.arguments.args.first(), iterable_keyword) {
                (Some(_), Some(_)) => {
                    ctx.error("sorted() got multiple values for argument 'iterable'".to_string());
                    return None;
                }
                (Some(arg), None) => lower_expr(arg, ctx)?,
                (None, Some(keyword)) => lower_expr(&keyword.value, ctx)?,
                (None, None) => {
                    ctx.error("sorted() missing required argument 'iterable'".to_string());
                    return None;
                }
            };
            let Some(elem_ty) = callable_builtin_element_type(iterable.ty()) else {
                ctx.error(format!(
                    "sorted() argument must be an iterable with a statically-known element type, got '{}'",
                    iterable.ty().display_name()
                ));
                return None;
            };
            let mut key_arg = None;
            let mut reverse_arg = HirExpr::BoolLiteral(false);
            if let Some(keyword) = key_keyword {
                let lowered = if matches!(keyword.value, Expr::NoneLiteral(_)) {
                    lower_expr(&keyword.value, ctx)?
                } else {
                    lower_lambda_with_context(&keyword.value, std::slice::from_ref(&elem_ty), ctx)?
                };
                if !matches!(lowered, HirExpr::NoneLiteral) {
                    let Some((param_types, _conventions, _return_ty)) =
                        callable_signature(&lowered)
                    else {
                        ctx.error("sorted() keyword argument 'key' must be callable".to_string());
                        return None;
                    };
                    if param_types.len() != 1 {
                        ctx.error(
                            "sorted() key callable must accept exactly 1 argument".to_string(),
                        );
                        return None;
                    }
                }
                key_arg = Some(lowered);
            }
            if let Some(keyword) = reverse_keyword {
                let lowered = lower_expr(&keyword.value, ctx)?;
                if lowered.ty() != &Type::Bool {
                    ctx.error(format!(
                        "sorted() keyword argument 'reverse' must be 'bool', got '{}'",
                        lowered.ty().display_name()
                    ));
                    return None;
                }
                reverse_arg = lowered;
            }
            let list_ty = callable_builtin_list_output_type(iterable.ty())?;
            let mut args = vec![iterable];
            if let Some(key_arg) = key_arg {
                args.push(key_arg);
                args.push(reverse_arg);
            } else if !matches!(reverse_arg, HirExpr::BoolLiteral(false)) {
                args.push(HirExpr::NoneLiteral);
                args.push(reverse_arg);
            }
            return Some(HirExpr::Call {
                func: "sorted".to_string(),
                args,
                ty: list_ty,
            });
        }

        // reversed(iterable) -> iterator of element type
        if func_name == "reversed" {
            let arg = lower_builtin_reverseable_arg(call, "reversed", ctx)?;
            let Some(elem_ty) = callable_builtin_element_type(arg.ty()) else {
                ctx.error(format!(
                    "reversed() argument must be an iterable with a statically-known element type, got '{}'",
                    arg.ty().display_name()
                ));
                return None;
            };
            return Some(HirExpr::IteratorCall {
                op: HirIteratorOp::Reversed,
                args: vec![arg],
                ty: Type::Iterator(Box::new(elem_ty)),
            });
        }

        // enumerate(iterable) -> iterator of (int, element) tuples
        if func_name == "enumerate" {
            if call.arguments.args.is_empty() || call.arguments.args.len() > 2 {
                ctx.error("enumerate() takes 1 or 2 arguments".to_string());
                return None;
            }
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            let Some(elem_ty) = callable_builtin_element_type(arg.ty()) else {
                ctx.error(format!(
                    "enumerate() argument must be an iterable with a statically-known element type, got '{}'",
                    arg.ty().display_name()
                ));
                return None;
            };
            let start = if call.arguments.args.len() == 2 {
                let lowered = lower_expr(&call.arguments.args[1], ctx)?;
                if lowered.ty() != &Type::Int {
                    ctx.error(format!(
                        "enumerate() start argument must be 'int', got '{}'",
                        lowered.ty().display_name()
                    ));
                    return None;
                }
                lowered
            } else if let Some(keyword) = call
                .arguments
                .keywords
                .iter()
                .find(|keyword| keyword.arg.as_deref() == Some("start"))
            {
                let lowered = lower_expr(&keyword.value, ctx)?;
                if lowered.ty() != &Type::Int {
                    ctx.error(format!(
                        "enumerate() keyword argument 'start' must be 'int', got '{}'",
                        lowered.ty().display_name()
                    ));
                    return None;
                }
                lowered
            } else {
                HirExpr::IntLiteral(0)
            };
            for keyword in &call.arguments.keywords {
                let Some(name) = keyword.arg.as_ref() else {
                    ctx.error(
                        "enumerate() does not support unpacked keyword arguments".to_string(),
                    );
                    return None;
                };
                if name.as_str() != "start" {
                    ctx.error(format!(
                        "enumerate() got an unexpected keyword argument '{name}'"
                    ));
                    return None;
                }
                if call.arguments.args.len() == 2 {
                    ctx.error("enumerate() got multiple values for argument 'start'".to_string());
                    return None;
                }
            }
            let tuple_ty = Type::Tuple(vec![Type::Int, elem_ty]);
            let result_ty = Type::Iterator(Box::new(tuple_ty));
            let args = if matches!(start, HirExpr::IntLiteral(0)) {
                vec![arg]
            } else {
                vec![arg, start]
            };
            return Some(HirExpr::IteratorCall {
                op: HirIteratorOp::Enumerate,
                args,
                ty: result_ty,
            });
        }

        // zip(*iters) -> iterator of tuples
        if func_name == "zip" {
            if !call.arguments.keywords.is_empty() {
                ctx.error("zip() does not accept keyword arguments in this phase".to_string());
                return None;
            }
            let mut args = Vec::with_capacity(call.arguments.args.len());
            let mut elem_types = Vec::with_capacity(call.arguments.args.len());
            for (index, arg_expr) in call.arguments.args.iter().enumerate() {
                let arg = lower_expr(arg_expr, ctx)?;
                let Some(elem_ty) = callable_builtin_element_type(arg.ty()) else {
                    ctx.error(format!(
                        "zip() argument {} must be an iterable with a statically-known element type, got '{}'",
                        index + 1,
                        arg.ty().display_name()
                    ));
                    return None;
                };
                elem_types.push(elem_ty);
                args.push(arg);
            }
            let result_ty = Type::Iterator(Box::new(Type::Tuple(elem_types)));
            return Some(HirExpr::IteratorCall {
                op: HirIteratorOp::Zip,
                args,
                ty: result_ty,
            });
        }
    }

    // any(iterable) -> bool
    if func_name == "any" {
        if call.arguments.args.len() != 1 {
            ctx.error("any() takes exactly 1 argument".to_string());
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        return Some(HirExpr::Call {
            func: "any".to_string(),
            args: vec![arg],
            ty: Type::Bool,
        });
    }

    // all(iterable) -> bool
    if func_name == "all" {
        if call.arguments.args.len() != 1 {
            ctx.error("all() takes exactly 1 argument".to_string());
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        return Some(HirExpr::Call {
            func: "all".to_string(),
            args: vec![arg],
            ty: Type::Bool,
        });
    }

    // map(func, iterable) -> iterator
    if func_name == "map" {
        if !call.arguments.keywords.is_empty() {
            ctx.error("map() does not accept keyword arguments in this phase".to_string());
            return None;
        }
        if call.arguments.args.len() < 2 {
            ctx.error("map() takes a callable followed by at least one iterable".to_string());
            return None;
        }
        let mut iter_args = Vec::with_capacity(call.arguments.args.len() - 1);
        let mut context_types = Vec::with_capacity(call.arguments.args.len() - 1);
        for arg_expr in call.arguments.args.iter().skip(1) {
            let iter_arg = lower_expr(arg_expr, ctx)?;
            let Some(elem_ty) = callable_builtin_element_type(iter_arg.ty()) else {
                ctx.error(format!(
                    "map() iterable arguments must have statically-known element types, got '{}'",
                    iter_arg.ty().display_name()
                ));
                return None;
            };
            context_types.push(elem_ty);
            iter_args.push(iter_arg);
        }
        let func_arg = lower_lambda_with_context(&call.arguments.args[0], &context_types, ctx)?;
        let Some((param_types, _conventions, result_elem_ty)) = callable_signature(&func_arg)
        else {
            ctx.error("map() first argument must be callable".to_string());
            return None;
        };
        if param_types.len() != context_types.len() {
            ctx.error(format!(
                "map() callable expects {} argument(s), got {} iterable(s)",
                param_types.len(),
                context_types.len()
            ));
            return None;
        }
        let result_ty = Type::Iterator(Box::new(result_elem_ty));
        return Some(HirExpr::IteratorCall {
            op: HirIteratorOp::Map,
            args: std::iter::once(func_arg).chain(iter_args).collect(),
            ty: result_ty,
        });
    }

    // filter(func, iterable) -> list (same element type)
    if func_name == "filter" {
        if call.arguments.args.len() != 2 {
            ctx.error("filter() takes exactly 2 arguments (function, iterable)".to_string());
            return None;
        }
        // Lower iterable first to get element type for contextual lambda typing
        let iter_arg = lower_expr(&call.arguments.args[1], ctx)?;
        let elem_ty = match iter_arg.ty() {
            Type::List(elem) => *elem.clone(),
            _ => Type::Any,
        };
        // Lower lambda with contextual typing
        let func_arg = lower_lambda_with_context(&call.arguments.args[0], &[elem_ty], ctx)?;
        let result_ty = iter_arg.ty().clone();
        return Some(HirExpr::IteratorCall {
            op: HirIteratorOp::Filter,
            args: vec![func_arg, iter_arg],
            ty: result_ty,
        });
    }

    // open(path, mode="r") -> FileHandle  — built-in file open (raises IOError on failure)
    // Matches Python's open() behavior: raises on error, returns FileHandle directly.
    if func_name == "open" {
        let n_args = call.arguments.args.len();
        let _n_kwargs = call.arguments.keywords.len();
        let path_arg = if n_args >= 1 {
            lower_expr(&call.arguments.args[0], ctx)?
        } else {
            ctx.error(
                "open() requires at least 1 argument: open(path) or open(path, mode)".to_string(),
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
            ctx.error(format!(
                "callable '{}' expects {} argument(s), got {}",
                func_name,
                param_types.len(),
                args.len()
            ));
            return None;
        }
        // Type check arguments and apply convention-aware move tracking
        for (i, (arg, param_ty)) in args.iter().zip(param_types.iter()).enumerate() {
            if !arg.ty().is_assignable_to(param_ty) {
                ctx.error(format!(
                    "argument {} of callable '{}': expected '{}', got '{}'",
                    i + 1,
                    func_name,
                    param_ty.display_name(),
                    arg.ty().display_name()
                ));
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
            ctx.error("only simple function calls are supported".to_string());
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
        ctx.error(format!("undefined function: '{func_name}'"));
        None
    })?;

    // Resolve keyword arguments to positional order
    let args = if func_name == "print" {
        let mut args = Vec::with_capacity(call.arguments.args.len());
        for arg in &call.arguments.args {
            args.push(lower_expr(arg, ctx)?);
        }
        args
    } else {
        let defaults = ctx.function_defaults.get(&func_name).cloned();
        lower_function_call_args(
            call,
            &func_name,
            &ft,
            defaults.as_deref(),
            ctx.vararg_functions.get(&func_name).copied(),
            ctx,
        )?
    };

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
                ctx.error(format!(
                    "argument {} ('{}') of function '{}': expected '{}', got '{}'",
                    i + 1,
                    param_name,
                    func_name,
                    param_ty.display_name(),
                    arg.ty().display_name()
                ));
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
                    let convention = ft
                        .params
                        .get(i)
                        .map(|(_, _, c)| *c)
                        .unwrap_or(ParamConvention::borrow());
                    if convention.is_mut_borrow() {
                        if mut_borrowed.contains(name) {
                            ctx.error(format!(
                                "cannot borrow '{name}' as mutable more than once in the same call to '{func_name}'"
                            ));
                        } else if immut_borrowed.contains(name) {
                            ctx.error(format!(
                                "cannot borrow '{name}' as mutable because it is already borrowed as immutable in the same call to '{func_name}'"
                            ));
                        }
                        mut_borrowed.push(name.clone());
                    } else if convention.is_shared_borrow() {
                        if mut_borrowed.contains(name) {
                            ctx.error(format!(
                                "cannot borrow '{name}' as immutable because it is already borrowed as mutable in the same call to '{func_name}'"
                            ));
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
                        ctx.error(format!(
                            "argument {} ('{}') of function '{}': expected '{}', got '{}'",
                            i + 1,
                            param_name,
                            func_name,
                            concrete_param_ty.display_name(),
                            arg.ty().display_name()
                        ));
                    }
                    continue;
                }
                if !arg.ty().is_assignable_to(&concrete_param_ty) {
                    ctx.error(format!(
                        "argument {} ('{}') of function '{}': expected '{}', got '{}'",
                        i + 1,
                        param_name,
                        func_name,
                        concrete_param_ty.display_name(),
                        arg.ty().display_name()
                    ));
                }
            }
        }
        // Check protocol bounds on type parameters (scoped to this function)
        let mut bound_errors: Vec<String> = Vec::new();
        if let Some(owner_bounds) = ctx.type_param_bounds.get(&func_name) {
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
                            bound_errors.push(format!(
                                "type '{}' does not implement protocol '{}' required by type parameter '{}'",
                                concrete_ty.display_name(),
                                bound,
                                tv_name
                            ));
                        }
                    }

                    if !constraints.is_empty()
                        && !constraints.iter().any(|constraint| {
                            type_satisfies_constraint(concrete_ty, constraint, ctx)
                        })
                    {
                        bound_errors.push(format!(
                            "type '{}' does not satisfy constraints ({}) required by type parameter '{}'",
                            concrete_ty.display_name(),
                            constraints.join(", "),
                            tv_name
                        ));
                    }
                }
            }
        }
        for err in bound_errors {
            ctx.error(err);
        }
        if bindings.is_empty() {
            *ft.return_type
        } else {
            substitute_type_vars(&ft.return_type, &bindings)
        }
    } else {
        *ft.return_type
    };

    // If this is a class constructor call, emit ConstructorCall
    if ctx.class_types.contains_key(&func_name) {
        Some(HirExpr::ConstructorCall {
            class_name: func_name,
            args,
            ty: return_type,
        })
    } else {
        Some(HirExpr::Call {
            func: func_name,
            args,
            ty: return_type,
        })
    }
}

pub(super) fn lower_fstring(fstring: &ExprFString, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut parts = Vec::new();

    for part in &fstring.value {
        match part {
            sifr_python_ast::FStringPart::Literal(s) => {
                parts.push(HirFStringPart::Literal(s.to_string()));
            }
            sifr_python_ast::FStringPart::FString(fs) => {
                for element in &fs.elements {
                    match element {
                        FStringElement::Literal(lit) => {
                            parts.push(HirFStringPart::Literal(lit.value.to_string()));
                        }
                        FStringElement::Expression(expr_elem) => {
                            let expr = lower_expr(&expr_elem.expression, ctx)?;
                            parts.push(HirFStringPart::Expr(expr));
                        }
                    }
                }
            }
        }
    }

    Some(HirExpr::FString {
        parts,
        ty: Type::Str,
    })
}

pub(super) fn lower_list_literal(list: &ExprList, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut elements = Vec::new();
    let mut elem_ty: Option<Type> = None;

    for elt in &list.elts {
        let expr = lower_expr(elt, ctx)?;
        let ty = expr.ty().clone();
        if let Some(ref expected) = elem_ty {
            if !ty.is_assignable_to(expected) {
                ctx.error(format!(
                    "list element type mismatch: expected '{}', got '{}'",
                    expected.display_name(),
                    ty.display_name()
                ));
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
                ctx.error(format!(
                    "set element type mismatch: expected '{}', got '{}'",
                    expected.display_name(),
                    ty.display_name()
                ));
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
                    ctx.error(format!(
                        "dict key type mismatch: expected '{}', got '{}'",
                        expected.display_name(),
                        kt.display_name()
                    ));
                }
            } else {
                key_ty = Some(kt);
            }
            keys.push(key);
        } else {
            ctx.error("dict unpacking (**) not supported".to_string());
            return None;
        }

        let val = lower_expr(&item.value, ctx)?;
        let vt = val.ty().clone();
        if let Some(ref expected) = val_ty {
            if !vt.is_assignable_to(expected) {
                ctx.error(format!(
                    "dict value type mismatch: expected '{}', got '{}'",
                    expected.display_name(),
                    vt.display_name()
                ));
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
                            ctx.error("tuple too large for slicing index computation".to_string());
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
                                    ctx.error("tuple slice indices out of range".to_string());
                                    Type::Any
                                }
                            } else {
                                ctx.error("tuple slice indices out of range".to_string());
                                Type::Any
                            }
                        } else {
                            ctx.error("tuple slice indices out of range".to_string());
                            Type::Any
                        }
                    } else {
                        ctx.error(
                            "tuple slicing requires compile-time constant indices".to_string(),
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
                ctx.error(format!("cannot slice type '{}'", object_ty.display_name()));
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

    let result_ty =
        if let Some(guarded_ty) = guarded_sequence_index_result_type(sub, &object_ty, ctx) {
            guarded_ty
        } else {
            object_ty.index_result_type(&index_ty).unwrap_or_else(|| {
                ctx.error(format!(
                    "cannot index type '{}' with '{}'",
                    object_ty.display_name(),
                    index_ty.display_name()
                ));
                Type::Any
            })
        };

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
        let class_name = name.id.clone();
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
        ctx.error(format!(
            "type '{}' has no field '{}'",
            object_ty.display_name(),
            field_name
        ));
        return None;
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
                ctx.error(format!(
                    "enum '{enum_name}' has no attribute '{field_name}'"
                ));
                return None;
            }
        }
    }

    // Not a class field access -- report unsupported
    ctx.error(format!(
        "attribute access '.{field_name}' is not supported as an expression; use as a method call"
    ));
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
                ctx.error("super() used outside of a class with a parent".to_string());
                return None;
            }
        }
    }

    // Handle ClassName.method() calls (classmethod/staticmethod)
    if let Expr::Name(name) = attr.value.as_ref() {
        let class_name = name.id.clone();
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
            ctx.error(format!(
                "type '{class_name}' has no class/static method '{method_name}'"
            ));
            return None;
        }
    }

    let mut object = lower_expr(&attr.value, ctx)?;
    let method_name = attr.attr.to_string();
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
    let object_ty = object.ty().clone();

    if reject_immutable_parameter_method_mutation(ctx, &object, &object_ty, &method_name) {
        return None;
    }

    // Resolve method return type based on object type and method name
    let return_ty = resolve_method_type(&object_ty, &method_name, &args, ctx)?;

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

fn refine_defaultdict_binding_expr(
    expr: HirExpr,
    method_name: &str,
    args: &[HirExpr],
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    let inferred_value_ty = match method_name {
        "append" if args.len() == 1 => Type::List(Box::new(args[0].ty().clone())),
        "add" if args.len() == 1 => Type::Set(Box::new(args[0].ty().clone())),
        _ => return None,
    };
    let HirExpr::Index { object, index, .. } = expr else {
        return None;
    };
    let HirExpr::Name { name, ty } = object.as_ref() else {
        return None;
    };
    let Type::Alias {
        name: alias_name,
        body,
        ..
    } = ty
    else {
        return None;
    };
    if !matches!(
        alias_name.as_str(),
        DEFAULTDICT_INT_ALIAS | DEFAULTDICT_LIST_ALIAS | DEFAULTDICT_SET_ALIAS
    ) {
        return None;
    }
    let Type::Dict(key_ty, value_ty) = body.as_ref() else {
        return None;
    };
    let expected_unrefined = match alias_name.as_str() {
        DEFAULTDICT_LIST_ALIAS => Type::List(Box::new(Type::Any)),
        DEFAULTDICT_SET_ALIAS => Type::Set(Box::new(Type::Any)),
        DEFAULTDICT_INT_ALIAS => Type::Int,
        _ => return None,
    };
    if *value_ty.as_ref() != expected_unrefined {
        return None;
    }
    let refined_key_ty = if matches!(key_ty.as_ref(), Type::Any | Type::Unknown) {
        index.ty().clone()
    } else {
        *key_ty.clone()
    };
    let refined_ty = Type::Alias {
        name: alias_name.clone(),
        type_args: Vec::new(),
        body: Box::new(Type::Dict(
            Box::new(refined_key_ty),
            Box::new(inferred_value_ty.clone()),
        )),
    };
    ctx.scope.narrow_var(name, refined_ty.clone());
    Some(HirExpr::Index {
        object: Box::new(HirExpr::Name {
            name: name.clone(),
            ty: refined_ty,
        }),
        index,
        ty: inferred_value_ty,
    })
}

fn refine_empty_set_binding_expr(
    expr: HirExpr,
    inferred_elem_ty: Type,
    ctx: &mut LowerCtx,
) -> HirExpr {
    let HirExpr::Name { name, ty } = &expr else {
        return expr;
    };
    let Type::Set(inner) = ty.resolve_alias() else {
        return expr;
    };
    if !matches!(inner.as_ref(), Type::Any | Type::Unknown) {
        return expr;
    }
    let refined_ty = Type::Set(Box::new(inferred_elem_ty));
    ctx.scope.narrow_var(name, refined_ty.clone());
    HirExpr::Name {
        name: name.clone(),
        ty: refined_ty,
    }
}

/// Resolve the return type of a method call on a given type.
pub(super) fn resolve_method_type(
    object_ty: &Type,
    method: &str,
    args: &[HirExpr],
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
            return resolve_method_type(body, method, args, ctx);
        }
    }
    match object_ty {
        Type::List(elem_ty) => match method {
            "append" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "list.append() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                if !args[0].ty().is_assignable_to(elem_ty) {
                    ctx.error(format!(
                        "list.append() argument type '{}' is not compatible with list element type '{}'",
                        args[0].ty().display_name(),
                        elem_ty.display_name()
                    ));
                }
                Some(Type::None)
            }
            "extend" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "list.extend() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                validate_list_extend_arg(elem_ty, args[0].ty(), ctx);
                Some(Type::None)
            }
            "insert" => {
                if args.len() != 2 {
                    ctx.error(format!(
                        "list.insert() takes exactly 2 arguments, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::None)
            }
            "clear" => {
                if !args.is_empty() {
                    ctx.error("list.clear() takes no arguments".to_string());
                    return None;
                }
                Some(Type::None)
            }
            "copy" => {
                if !args.is_empty() {
                    ctx.error("list.copy() takes no arguments".to_string());
                    return None;
                }
                Some(Type::List(elem_ty.clone()))
            }
            "reverse" => {
                if !args.is_empty() {
                    ctx.error("list.reverse() takes no arguments".to_string());
                    return None;
                }
                Some(Type::None)
            }
            "sort" => {
                if !args.is_empty() {
                    ctx.error("list.sort() takes no arguments in this milestone".to_string());
                    return None;
                }
                Some(Type::None)
            }
            "count" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "list.count() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Int)
            }
            "contains" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "list.contains() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Bool)
            }
            "len" => {
                if !args.is_empty() {
                    ctx.error("list.len() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Int)
            }
            "pop" => {
                if args.len() > 1 {
                    ctx.error(format!(
                        "list.pop() takes at most 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                if let Some(index_arg) = args.first() {
                    if index_arg.ty() != &Type::Int {
                        ctx.error(format!(
                            "list.pop() index must be 'int', got '{}'",
                            index_arg.ty().display_name()
                        ));
                    }
                }
                // pop() returns Option[T] = T | None
                Some(Type::Union(vec![*elem_ty.clone(), Type::None]))
            }
            "popleft" => {
                if !args.is_empty() {
                    ctx.error("list.popleft() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Union(vec![*elem_ty.clone(), Type::None]))
            }
            "appendleft" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "list.appendleft() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::None)
            }
            "remove" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "list.remove() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::None)
            }
            "index" => {
                if args.is_empty() || args.len() > 3 {
                    ctx.error(format!(
                        "list.index() takes 1 to 3 arguments, got {}",
                        args.len()
                    ));
                    return None;
                }
                for bound in args.iter().skip(1) {
                    if bound.ty() != &Type::Int {
                        ctx.error(format!(
                            "list.index() bounds must be 'int', got '{}'",
                            bound.ty().display_name()
                        ));
                    }
                }
                // Returns Option[int] = int | None (safe: no panic if not found)
                Some(Type::Union(vec![Type::Int, Type::None]))
            }
            _ => {
                ctx.error(format!("list has no method '{method}'"));
                None
            }
        },
        Type::Dict(key_ty, val_ty) => match method {
            "len" => {
                if !args.is_empty() {
                    ctx.error("dict.len() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Int)
            }
            "keys" => {
                if !args.is_empty() {
                    ctx.error("dict.keys() takes no arguments".to_string());
                    return None;
                }
                Some(Type::List(key_ty.clone()))
            }
            "values" => {
                if !args.is_empty() {
                    ctx.error("dict.values() takes no arguments".to_string());
                    return None;
                }
                Some(Type::List(val_ty.clone()))
            }
            "items" => {
                if !args.is_empty() {
                    ctx.error("dict.items() takes no arguments".to_string());
                    return None;
                }
                Some(Type::List(Box::new(Type::Tuple(vec![
                    *key_ty.clone(),
                    *val_ty.clone(),
                ]))))
            }
            "update" => {
                if args.len() > 2 {
                    ctx.error(format!(
                        "dict.update() takes at most 2 arguments, got {}",
                        args.len()
                    ));
                    return None;
                }
                if let Some(arg) = args.first() {
                    validate_dict_update_arg(key_ty, val_ty, arg.ty(), ctx);
                }
                if let Some(keyword_dict) = args.get(1) {
                    validate_dict_update_arg(key_ty, val_ty, keyword_dict.ty(), ctx);
                }
                Some(Type::None)
            }
            "clear" => {
                if !args.is_empty() {
                    ctx.error("dict.clear() takes no arguments".to_string());
                    return None;
                }
                Some(Type::None)
            }
            "copy" => {
                if !args.is_empty() {
                    ctx.error("dict.copy() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Dict(key_ty.clone(), val_ty.clone()))
            }
            "contains" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "dict.contains() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Bool)
            }
            "get" => {
                if args.is_empty() || args.len() > 2 {
                    ctx.error(format!(
                        "dict.get() takes 1 or 2 arguments, got {}",
                        args.len()
                    ));
                    return None;
                }
                if args.len() == 2 {
                    if !args[1].ty().is_assignable_to(val_ty) {
                        ctx.error(format!(
                            "dict.get() default type '{}' is not compatible with dict value type '{}'",
                            args[1].ty().display_name(),
                            val_ty.display_name()
                        ));
                    }
                    // dict.get(key, default) -> V (returns default if key not found)
                    Some(*val_ty.clone())
                } else {
                    // dict.get(key) -> V | None
                    Some(Type::Union(vec![*val_ty.clone(), Type::None]))
                }
            }
            "pop" => {
                if args.is_empty() || args.len() > 2 {
                    ctx.error(format!(
                        "dict.pop() takes 1 or 2 arguments, got {}",
                        args.len()
                    ));
                    return None;
                }
                if args.len() == 2 {
                    if !args[1].ty().is_assignable_to(val_ty) {
                        ctx.error(format!(
                            "dict.pop() default type '{}' is not compatible with dict value type '{}'",
                            args[1].ty().display_name(),
                            val_ty.display_name()
                        ));
                    }
                    Some(*val_ty.clone())
                } else {
                    // pop() returns Option[V] = V | None
                    Some(Type::Union(vec![*val_ty.clone(), Type::None]))
                }
            }
            "setdefault" => {
                if args.len() != 2 {
                    ctx.error(format!(
                        "dict.setdefault() takes exactly 2 arguments, got {}",
                        args.len()
                    ));
                    return None;
                }
                if !args[1].ty().is_assignable_to(val_ty) {
                    ctx.error(format!(
                        "dict.setdefault() default type '{}' is not compatible with dict value type '{}'",
                        args[1].ty().display_name(),
                        val_ty.display_name()
                    ));
                }
                Some(*val_ty.clone())
            }
            _ => {
                ctx.error(format!("dict has no method '{method}'"));
                None
            }
        },
        Type::Set(elem_ty) => match method {
            "len" => {
                if !args.is_empty() {
                    ctx.error("set.len() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Int)
            }
            "add" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "set.add() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::None)
            }
            "remove" | "discard" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "set.{}() takes exactly 1 argument, got {}",
                        method,
                        args.len()
                    ));
                    return None;
                }
                Some(Type::None)
            }
            "contains" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "set.contains() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Bool)
            }
            "clear" => {
                if !args.is_empty() {
                    ctx.error("set.clear() takes no arguments".to_string());
                    return None;
                }
                Some(Type::None)
            }
            "copy" => {
                if !args.is_empty() {
                    ctx.error("set.copy() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Set(elem_ty.clone()))
            }
            "union" | "intersection" | "difference" => {
                for arg in args {
                    validate_set_iterable_arg(elem_ty, arg.ty(), method, ctx);
                }
                Some(Type::Set(elem_ty.clone()))
            }
            "symmetric_difference" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "set.{}() takes exactly 1 argument, got {}",
                        method,
                        args.len()
                    ));
                    return None;
                }
                validate_set_iterable_arg(elem_ty, args[0].ty(), method, ctx);
                Some(Type::Set(elem_ty.clone()))
            }
            "update" | "intersection_update" | "difference_update" => {
                for arg in args {
                    validate_set_iterable_arg(elem_ty, arg.ty(), method, ctx);
                }
                Some(Type::None)
            }
            "symmetric_difference_update" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "set.{}() takes exactly 1 argument, got {}",
                        method,
                        args.len()
                    ));
                    return None;
                }
                validate_set_iterable_arg(elem_ty, args[0].ty(), method, ctx);
                Some(Type::None)
            }
            "issubset" | "issuperset" | "isdisjoint" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "set.{}() takes exactly 1 argument, got {}",
                        method,
                        args.len()
                    ));
                    return None;
                }
                validate_set_iterable_arg(elem_ty, args[0].ty(), method, ctx);
                Some(Type::Bool)
            }
            "pop" => {
                if !args.is_empty() {
                    ctx.error("set.pop() takes no arguments".to_string());
                    return None;
                }
                // Returns Option[T] = T | None (safe: no panic on empty set)
                Some(Type::Union(vec![*elem_ty.clone(), Type::None]))
            }
            _ => {
                ctx.error(format!("set has no method '{method}'"));
                None
            }
        },
        Type::Str => match method {
            "len" => Some(Type::Int),
            "upper" | "lower" | "strip" | "lstrip" | "rstrip" | "title" | "capitalize"
            | "swapcase" => Some(Type::Str),
            "startswith" | "endswith" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "str.{}() takes exactly 1 argument, got {}",
                        method,
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Bool)
            }
            "isdigit" | "isalpha" | "isalnum" | "isspace" | "isupper" | "islower" => {
                if !args.is_empty() {
                    ctx.error(format!("str.{method}() takes no arguments"));
                    return None;
                }
                Some(Type::Bool)
            }
            "split" => {
                if args.len() > 2 {
                    ctx.error(format!(
                        "str.split() takes 0 to 2 arguments, got {}",
                        args.len()
                    ));
                    return None;
                }
                if let Some(maxsplit) = args.get(1) {
                    if maxsplit.ty() != &Type::Int {
                        ctx.error(format!(
                            "str.split() maxsplit must be 'int', got '{}'",
                            maxsplit.ty().display_name()
                        ));
                    }
                }
                Some(Type::List(Box::new(Type::Str)))
            }
            "replace" => {
                if args.len() < 2 || args.len() > 3 {
                    ctx.error(format!(
                        "str.replace() takes 2 or 3 arguments, got {}",
                        args.len()
                    ));
                    return None;
                }
                if let Some(count) = args.get(2) {
                    if count.ty() != &Type::Int {
                        ctx.error(format!(
                            "str.replace() count must be 'int', got '{}'",
                            count.ty().display_name()
                        ));
                    }
                }
                Some(Type::Str)
            }
            "join" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "str.join() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Str)
            }
            "count" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "str.count() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Int)
            }
            "center" | "ljust" | "rjust" | "zfill" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "str.{}() takes exactly 1 argument, got {}",
                        method,
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Str)
            }
            "find" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "str.find() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                // find() returns Option[int] = int | None
                Some(Type::Union(vec![Type::Int, Type::None]))
            }
            "encode" => resolve_str_encode_method_type(args, ctx),
            _ => {
                ctx.error(format!("str has no method '{method}'"));
                None
            }
        },
        Type::Bytes => resolve_bytes_method_type(method, args, ctx),
        Type::Tuple(_) => match method {
            "len" => Some(Type::Int),
            "count" => {
                if args.len() != 1 {
                    ctx.error(format!(
                        "tuple.count() takes exactly 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                Some(Type::Int)
            }
            "index" => {
                if args.is_empty() || args.len() > 3 {
                    ctx.error(format!(
                        "tuple.index() takes 1 to 3 arguments, got {}",
                        args.len()
                    ));
                    return None;
                }
                for bound in args.iter().skip(1) {
                    if bound.ty() != &Type::Int {
                        ctx.error(format!(
                            "tuple.index() bounds must be 'int', got '{}'",
                            bound.ty().display_name()
                        ));
                    }
                }
                Some(Type::Union(vec![Type::Int, Type::None]))
            }
            _ => {
                ctx.error(format!("tuple has no method '{method}'"));
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
                    ctx.error(format!(
                        "{}.{}() takes {} argument(s), got {}",
                        name,
                        method,
                        ft.params.len(),
                        args.len()
                    ));
                    return None;
                }
                // Check argument types
                for (i, (arg, (param_name, param_ty, _))) in
                    args.iter().zip(ft.params.iter()).enumerate()
                {
                    if !arg.ty().is_assignable_to(param_ty) {
                        ctx.error(format!(
                            "argument {} ('{}') of {}.{}(): expected '{}', got '{}'",
                            i + 1,
                            param_name,
                            name,
                            method,
                            param_ty.display_name(),
                            arg.ty().display_name()
                        ));
                    }
                }
                Some(canonicalize_class_surface_type(&ft.return_type))
            } else if let Some((_, field_ty)) = fields.iter().find(|(n, _)| n == method) {
                // Check if the field is a Callable type — allow calling it like a method
                if let Type::Callable(param_types, _, ret_type) = field_ty {
                    if args.len() != param_types.len() {
                        ctx.error(format!(
                            "{}.{}() (callable field) takes {} argument(s), got {}",
                            name,
                            method,
                            param_types.len(),
                            args.len()
                        ));
                        return None;
                    }
                    for (i, (arg, param_ty)) in args.iter().zip(param_types.iter()).enumerate() {
                        if !arg.ty().is_assignable_to(param_ty) {
                            ctx.error(format!(
                                "argument {} of {}.{}(): expected '{}', got '{}'",
                                i + 1,
                                name,
                                method,
                                param_ty.display_name(),
                                arg.ty().display_name()
                            ));
                        }
                    }
                    Some(canonicalize_class_surface_type(ret_type))
                } else {
                    ctx.error(format!(
                        "field '{}' of class '{}' is not callable (type: '{}')",
                        method,
                        name,
                        field_ty.display_name()
                    ));
                    None
                }
            } else {
                ctx.error(format!("class '{name}' has no method '{method}'"));
                None
            }
        }
        Type::Protocol { name, methods, .. } => {
            if let Some((_, ft)) = methods.iter().find(|(n, _)| n == method) {
                if args.len() != ft.params.len() {
                    ctx.error(format!(
                        "{}.{}() takes {} argument(s), got {}",
                        name,
                        method,
                        ft.params.len(),
                        args.len()
                    ));
                }
                Some(canonicalize_class_surface_type(&ft.return_type))
            } else {
                ctx.error(format!("protocol '{name}' has no method '{method}'"));
                None
            }
        }
        Type::Newtype { name, inner } => {
            // Newtype has a built-in `value()` method that returns the inner type
            if method == "value" {
                if !args.is_empty() {
                    ctx.error(format!("{name}.value() takes no arguments"));
                    return None;
                }
                Some(*inner.clone())
            } else {
                // Delegate to the inner type's methods
                resolve_method_type(inner, method, args, ctx)
            }
        }
        Type::Enum { name, .. } => {
            match method {
                "name" => {
                    if !args.is_empty() {
                        ctx.error(format!("{name}.name() takes no arguments"));
                        return None;
                    }
                    Some(Type::Str)
                }
                "value" => {
                    if !args.is_empty() {
                        ctx.error(format!("{name}.value() takes no arguments"));
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
                    ctx.error(format!("enum '{name}' has no method '{method}'"));
                    None
                }
            }
        }
        Type::BigInt => {
            if method == "clone" {
                if !args.is_empty() {
                    ctx.error("bigint.clone() takes no arguments".to_string());
                    return None;
                }
                Some(Type::BigInt)
            } else {
                ctx.error(format!("type 'bigint' has no method '{method}'"));
                None
            }
        }
        Type::Decimal | Type::BigDecimal => {
            resolve_decimal_method_type(object_ty, method, args, ctx)
        }
        _ => {
            ctx.error(format!(
                "type '{}' has no method '{}'",
                object_ty.display_name(),
                method
            ));
            None
        }
    }
}

pub(super) fn lower_if_expr(if_expr: &ExprIf, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let condition = lower_expr(&if_expr.test, ctx)?;
    let then_expr = lower_expr(&if_expr.body, ctx)?;
    let else_expr = lower_expr(&if_expr.orelse, ctx)?;

    let then_ty = then_expr.ty().clone();
    let else_ty = else_expr.ty().clone();

    if !then_ty.is_assignable_to(&else_ty) && !else_ty.is_assignable_to(&then_ty) {
        ctx.error(format!(
            "if expression branches have incompatible types: '{}' and '{}'",
            then_ty.display_name(),
            else_ty.display_name()
        ));
        return None;
    }

    Some(HirExpr::IfExpr {
        condition: Box::new(condition),
        then_expr: Box::new(then_expr),
        else_expr: Box::new(else_expr),
        ty: then_ty,
    })
}

/// Lower a lambda or regular expression with contextual type information for parameters.
/// If the expression is a lambda, use `context_types` for untyped parameters.
/// If it's not a lambda, just lower it normally.
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
                    // Lambda params without annotations: infer as Any for now
                    // Contextual typing will refine this at call sites
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

pub(super) fn lower_list_comp(comp: &ExprListComp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if comp.generators.is_empty() {
        ctx.error("list comprehension must have at least one generator".to_string());
        return None;
    }

    let mut generators = Vec::new();
    let mut pushed_scopes = 0;
    let result = (|| {
        // Process each generator: push scope, define var, lower iter
        for gen in &comp.generators {
            let var_name = match &gen.target {
                Expr::Name(n) => n.id.clone(),
                Expr::Tuple(tup) => {
                    let names: Vec<String> = tup
                        .elts
                        .iter()
                        .filter_map(|e| {
                            if let Expr::Name(n) = e {
                                Some(n.id.clone())
                            } else {
                                None
                            }
                        })
                        .collect();
                    if names.len() != tup.elts.len() {
                        ctx.error(
                            "comprehension tuple target must contain only simple names".to_string(),
                        );
                        return None;
                    }
                    names.join(",")
                }
                _ => {
                    ctx.error("comprehension target must be a simple name or tuple".to_string());
                    return None;
                }
            };

            let iter_source_expr = lower_expr(&gen.iter, ctx)?;
            let iter_ty = iter_source_expr.ty().clone();
            let elem_ty = match &iter_ty {
                Type::List(elem) => *elem.clone(),
                Type::Set(elem) => *elem.clone(),
                Type::Str => Type::Str,
                Type::Range => Type::Int,
                Type::Dict(key, _) => *key.clone(),
                Type::Tuple(elems) if !elems.is_empty() => elems[0].clone(),
                _ => {
                    ctx.error(format!(
                        "cannot iterate over type '{}'",
                        iter_ty.display_name()
                    ));
                    return None;
                }
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

            let iter_expr = HirExpr::IteratorCall { op: HirIteratorOp::Iter, args: vec![iter_source_expr], ty: Type::Iterator(Box::new(elem_ty)) };
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
    let mut generators = Vec::new();
    let mut pushed_scopes = 0;
    let result = (|| {
        for gen in &comp.generators {
            let var_name = if let Expr::Name(n) = &gen.target {
                n.id.clone()
            } else {
                ctx.error("set comprehension target must be a simple name".to_string());
                return None;
            };
            let iter_source_expr = lower_expr(&gen.iter, ctx)?;
            let iter_ty = iter_source_expr.ty().clone();
            let elem_ty = match &iter_ty {
                Type::List(elem) => *elem.clone(),
                Type::Set(elem) => *elem.clone(),
                Type::Range => Type::Int,
                _ => {
                    ctx.error(format!(
                        "cannot iterate over type '{}'",
                        iter_ty.display_name()
                    ));
                    return None;
                }
            };
            ctx.scope.push();
            pushed_scopes += 1;
            ctx.scope.define(var_name.clone(), elem_ty.clone());
            let filter = if gen.ifs.is_empty() {
                None
            } else {
                Some(lower_expr(&gen.ifs[0], ctx)?)
            };
            let iter_expr = HirExpr::IteratorCall { op: HirIteratorOp::Iter, args: vec![iter_source_expr], ty: Type::Iterator(Box::new(elem_ty)) };
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
    let mut generators = Vec::new();
    let mut pushed_scopes = 0;
    let result = (|| {
        for gen in &comp.generators {
            let var_name = match &gen.target {
                Expr::Name(n) => n.id.clone(),
                Expr::Tuple(tup) => {
                    let names: Vec<String> = tup
                        .elts
                        .iter()
                        .filter_map(|e| {
                            if let Expr::Name(n) = e {
                                Some(n.id.clone())
                            } else {
                                None
                            }
                        })
                        .collect();
                    names.join(",")
                }
                _ => {
                    ctx.error(
                        "dict comprehension target must be a simple name or tuple".to_string(),
                    );
                    return None;
                }
            };
            let iter_source_expr = lower_expr(&gen.iter, ctx)?;
            let iter_ty = iter_source_expr.ty().clone();
            let elem_ty = match &iter_ty {
                Type::List(elem) => *elem.clone(),
                Type::Set(elem) => *elem.clone(),
                Type::Range => Type::Int,
                Type::Dict(key, _) => *key.clone(),
                _ => {
                    ctx.error(format!(
                        "cannot iterate over type '{}'",
                        iter_ty.display_name()
                    ));
                    return None;
                }
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
            let iter_expr = HirExpr::IteratorCall { op: HirIteratorOp::Iter, args: vec![iter_source_expr], ty: Type::Iterator(Box::new(elem_ty)) };
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
    // Only support single generator: (expr for var in iter) or (expr for var in iter if cond)
    if gen.generators.len() != 1 {
        ctx.error("only single-generator generator expressions are supported".to_string());
        return None;
    }

    let comp = &gen.generators[0];

    let var_name = if let Expr::Name(n) = &comp.target {
        n.id.clone()
    } else {
        ctx.error("generator target must be a simple name".to_string());
        return None;
    };
    let iter_source_expr = lower_expr(&comp.iter, ctx)?;
    let iter_ty = iter_source_expr.ty().clone();
    let Some(elem_ty) = callable_builtin_element_type(&iter_ty) else {
        ctx.error(format!(
            "cannot iterate over type '{}'",
            iter_ty.display_name()
        ));
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
    let iter_expr = HirExpr::IteratorCall { op: HirIteratorOp::Iter, args: vec![iter_source_expr], ty: Type::Iterator(Box::new(elem_ty)) };
    Some(HirExpr::GeneratorExpr {
        expr: Box::new(expr),
        var: var_name,
        iter: Box::new(iter_expr),
        filter,
        ty: result_ty,
    })
}

pub(super) fn lower_named_expr(named: &ExprNamed, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let name = if let Expr::Name(n) = named.target.as_ref() {
        n.id.clone()
    } else {
        ctx.error("walrus operator target must be a simple name".to_string());
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
