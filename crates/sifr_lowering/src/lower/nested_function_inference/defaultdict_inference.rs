use super::{
    lookup_name_type, unify_name_binding, unify_types, Expr, ExprCall, FunctionEnv, HashMap,
    LocalFunctionState, LowerCtx, Type, DEFAULTDICT_INT_ALIAS, DEFAULTDICT_LIST_ALIAS,
    DEFAULTDICT_SET_ALIAS,
};
use sifr_python_ast::visitor::{self, Visitor};

pub(super) fn defaultdict_shape_expr_is_lowering_exact(expr: &Expr, env: &FunctionEnv) -> bool {
    let mut visitor = LoweringInexactExprVisitor {
        inexact_bindings: &env.lowering_inexact_bindings,
        found: false,
    };
    visitor.visit_expr(expr);
    !visitor.found
}

struct LoweringInexactExprVisitor<'env> {
    inexact_bindings: &'env std::collections::HashSet<String>,
    found: bool,
}

impl<'ast> Visitor<'ast> for LoweringInexactExprVisitor<'_> {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        if self.found {
            return;
        }
        match expr {
            Expr::Name(name) if self.inexact_bindings.contains(name.id.as_str()) => {
                self.found = true;
                return;
            }
            Expr::Subscript(subscript) if !matches!(subscript.slice.as_ref(), Expr::Slice(_)) => {
                self.found = true;
                return;
            }
            _ => {}
        }
        visitor::walk_expr(self, expr);
    }
}

pub(super) fn infer_defaultdict_call_type(call: &ExprCall) -> Option<Type> {
    let Expr::Name(name) = call.func.as_ref() else {
        return None;
    };
    if name.id != "defaultdict" || call.arguments.args.len() != 1 {
        return None;
    }
    let Some(Expr::Name(factory)) = call.arguments.args.first() else {
        return None;
    };
    let (alias, value_ty) = match factory.id.as_str() {
        "int" => (DEFAULTDICT_INT_ALIAS, Type::Int),
        "list" => (DEFAULTDICT_LIST_ALIAS, Type::List(Box::new(Type::Unknown))),
        "set" => (DEFAULTDICT_SET_ALIAS, Type::Set(Box::new(Type::Unknown))),
        _ => return None,
    };
    Some(Type::alias(
        alias,
        Type::Dict(Box::new(Type::Unknown), Box::new(value_ty)),
    ))
}

pub(super) fn is_unresolved_defaultdict_inference_type(ty: &Type) -> bool {
    let Type::Alias { name, body, .. } = ty else {
        return false;
    };
    matches!(
        name.as_str(),
        DEFAULTDICT_INT_ALIAS | DEFAULTDICT_LIST_ALIAS | DEFAULTDICT_SET_ALIAS
    ) && body.contains_unknown_or_any()
}

pub(super) struct DefaultdictMethodCall<'a> {
    pub(super) object: &'a Expr,
    pub(super) method: &'a str,
    pub(super) args: &'a [Expr],
    pub(super) arg_types: &'a [Type],
}

pub(super) fn refine_defaultdict_method_call(
    call: &DefaultdictMethodCall<'_>,
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) {
    if call
        .args
        .first()
        .is_some_and(|arg| !defaultdict_shape_expr_is_lowering_exact(arg, env))
    {
        return;
    }
    let Expr::Subscript(subscript) = call.object else {
        return;
    };
    let Expr::Name(name) = subscript.value.as_ref() else {
        return;
    };
    let Type::Alias {
        name: alias_name,
        type_args,
        body,
    } = lookup_name_type(name.id.as_str(), env, states, ctx)
    else {
        return;
    };
    let Type::Dict(key_ty, value_ty) = body.as_ref() else {
        return;
    };
    let refined_value_ty = match (alias_name.as_str(), call.method, value_ty.as_ref()) {
        (DEFAULTDICT_SET_ALIAS, "add", Type::Set(elem_ty)) if call.arg_types.len() == 1 => {
            Type::Set(Box::new(unify_types(
                *elem_ty.clone(),
                call.arg_types[0].clone(),
            )))
        }
        (DEFAULTDICT_LIST_ALIAS, "append", Type::List(elem_ty)) if call.arg_types.len() == 1 => {
            Type::List(Box::new(unify_types(
                *elem_ty.clone(),
                call.arg_types[0].clone(),
            )))
        }
        _ => return,
    };
    unify_name_binding(
        name.id.as_str(),
        Type::Alias {
            name: alias_name,
            type_args,
            body: Box::new(Type::Dict(key_ty.clone(), Box::new(refined_value_ty))),
        },
        env,
        states,
        current_function,
    );
}

pub(super) fn refine_defaultdict_subscript(
    object: &Expr,
    index: &Expr,
    object_ty: &Type,
    index_ty: &Type,
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
) -> Option<Type> {
    if !defaultdict_shape_expr_is_lowering_exact(index, env) {
        return None;
    }
    let Expr::Name(name) = object else {
        return None;
    };
    let Type::Alias {
        name: alias_name,
        type_args,
        body,
    } = object_ty
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
    unify_name_binding(
        name.id.as_str(),
        Type::Alias {
            name: alias_name.clone(),
            type_args: type_args.clone(),
            body: Box::new(Type::Dict(
                Box::new(unify_types(*key_ty.clone(), index_ty.clone())),
                value_ty.clone(),
            )),
        },
        env,
        states,
        current_function,
    );
    Some(*value_ty.clone())
}

pub(super) fn unify_matching_defaultdict_aliases(current: &Type, incoming: &Type) -> Option<Type> {
    let (
        Type::Alias {
            name: current_name,
            type_args,
            body: current_body,
        },
        Type::Alias {
            name: incoming_name,
            body: incoming_body,
            ..
        },
    ) = (current, incoming)
    else {
        return None;
    };
    if current_name != incoming_name
        || !matches!(
            current_name.as_str(),
            DEFAULTDICT_INT_ALIAS | DEFAULTDICT_LIST_ALIAS | DEFAULTDICT_SET_ALIAS
        )
    {
        return None;
    }
    Some(Type::Alias {
        name: current_name.clone(),
        type_args: type_args.clone(),
        body: Box::new(unify_types(
            (**current_body).clone(),
            (**incoming_body).clone(),
        )),
    })
}
