use super::LowerCtx;
use super::builtin_calls::{DEFAULTDICT_INT_ALIAS, DEFAULTDICT_LIST_ALIAS, DEFAULTDICT_SET_ALIAS};
use super::declaration_hint_safety::safe_direct_assignment_names;
use crate::hir_nodes::HirExpr;
use sifr_python_ast::{Expr, Stmt};
use sifr_type_system::{Type, widen_literal};
use std::collections::HashSet;

pub(in crate::lower) fn safe_defaultdict_hint_names_for_block(stmts: &[Stmt]) -> HashSet<String> {
    safe_direct_assignment_names(stmts, is_unseeded_defaultdict_call)
}

fn is_unseeded_defaultdict_call(expr: &Expr) -> bool {
    let Expr::Call(call) = expr else {
        return false;
    };
    if call.arguments.args.len() != 1 || !call.arguments.keywords.is_empty() {
        return false;
    }
    let (Expr::Name(function), Some(Expr::Name(factory))) =
        (call.func.as_ref(), call.arguments.args.first())
    else {
        return false;
    };
    function.id == "defaultdict" && matches!(factory.id.as_str(), "int" | "list" | "set")
}

pub(in crate::lower) fn order_independent_defaultdict_hint(expr: &Expr, hint: &Type) -> bool {
    let Expr::Call(call) = expr else {
        return false;
    };
    if call.arguments.args.len() != 1 || !call.arguments.keywords.is_empty() {
        return false;
    }
    let (Expr::Name(function), Some(Expr::Name(factory))) =
        (call.func.as_ref(), call.arguments.args.first())
    else {
        return false;
    };
    if function.id != "defaultdict" {
        return false;
    }
    let expected_alias = match factory.id.as_str() {
        "int" => DEFAULTDICT_INT_ALIAS,
        "list" => DEFAULTDICT_LIST_ALIAS,
        "set" => DEFAULTDICT_SET_ALIAS,
        _ => return false,
    };
    let Type::Alias { name, body, .. } = hint else {
        return false;
    };
    name == expected_alias && !body.contains_unknown_or_any()
}

pub(in crate::lower) fn refine_defaultdict_int_augassign_key(
    object_name: &str,
    object_ty: Type,
    index_ty: &Type,
    ctx: &mut LowerCtx,
) -> Type {
    let Type::Alias {
        name,
        type_args,
        body,
    } = &object_ty
    else {
        return object_ty;
    };
    if name != DEFAULTDICT_INT_ALIAS {
        return object_ty;
    }
    let Type::Dict(key_ty, value_ty) = body.as_ref() else {
        return object_ty;
    };
    if !matches!(key_ty.as_ref(), Type::Any | Type::Unknown) {
        return object_ty;
    }
    let inferred_key_ty = widen_literal(index_ty);
    if matches!(inferred_key_ty, Type::Any | Type::Unknown) {
        return object_ty;
    }

    let refined_ty = Type::Alias {
        name: name.clone(),
        type_args: type_args.clone(),
        body: Box::new(Type::Dict(Box::new(inferred_key_ty), value_ty.clone())),
    };
    let _ = ctx.scope.set_type(object_name, refined_ty.clone());
    ctx.narrow_var_with_flow(
        object_name,
        refined_ty.clone(),
        "defaultdict-int-augassign-key-refinement".to_string(),
        true,
    );
    ctx.pending_container_specialization_patches
        .insert(object_name.to_string(), refined_ty.clone());
    refined_ty
}

pub(in crate::lower) fn refine_defaultdict_binding_expr(
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
    let HirExpr::Name {
        name,
        binding_id,
        ty,
    } = object.as_ref()
    else {
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
    ctx.narrow_var_with_flow(
        name,
        refined_ty.clone(),
        "defaultdict-refinement".to_string(),
        true,
    );
    Some(HirExpr::Index {
        object: Box::new(HirExpr::Name {
            name: name.clone(),
            binding_id: *binding_id,
            ty: refined_ty,
        }),
        index,
        ty: inferred_value_ty,
    })
}
