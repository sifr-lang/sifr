use super::{
    infer_registered_call, str, type_check_binary_op, unify_function_return, CmpOp, Expr, ExprCall,
    FunctionEnv, HashMap, LocalFunctionState, LowerCtx, Operator, Type,
};
pub(super) fn analyze_assign(
    targets: &[Expr],
    value: &Expr,
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) {
    if targets.len() != 1 {
        return;
    }

    let target = &targets[0];
    match target {
        Expr::Name(name) => {
            let value_ty = infer_expr_type(value, env, states, current_function, ctx);
            if let Some(callee_name) = nested_call_target_name(value, states) {
                env.bind_call_result(name.id.to_string(), value_ty, callee_name);
            } else {
                env.bind_var(name.id.as_str(), value_ty);
            }
        }
        Expr::Subscript(sub) => {
            let value_ty = infer_expr_type(value, env, states, current_function, ctx);
            let index_ty = infer_expr_type(&sub.slice, env, states, current_function, ctx);
            let Expr::Name(object_name) = sub.value.as_ref() else {
                return;
            };
            env.record_dict_write(object_name.id.as_str(), index_ty.clone(), value_ty.clone());
            let current_object_ty = lookup_name_type(object_name.id.as_str(), env, states, ctx);
            let refined_object_ty = match current_object_ty {
                Type::Dict(key_ty, value_ty_current) => Type::Dict(
                    Box::new(unify_types(*key_ty, index_ty)),
                    Box::new(unify_types(*value_ty_current, value_ty)),
                ),
                Type::List(elem_ty) => Type::List(Box::new(unify_types(*elem_ty, value_ty))),
                other => other,
            };
            unify_name_binding(
                object_name.id.as_str(),
                refined_object_ty,
                env,
                states,
                current_function,
            );
        }
        Expr::Tuple(tuple) => {
            if let Expr::Tuple(values) = value {
                for (target_expr, value_expr) in tuple.elts.iter().zip(values.elts.iter()) {
                    if let Expr::Name(name) = target_expr {
                        let value_ty =
                            infer_expr_type(value_expr, env, states, current_function, ctx);
                        env.bind_var(name.id.as_str(), value_ty);
                    }
                }
            }
        }
        _ => {}
    }
}

pub(super) fn nested_call_target_name(
    expr: &Expr,
    states: &HashMap<String, LocalFunctionState<'_>>,
) -> Option<String> {
    let Expr::Call(call) = expr else {
        return None;
    };
    let Expr::Name(name) = call.func.as_ref() else {
        return None;
    };
    states
        .contains_key(name.id.as_str())
        .then(|| name.id.to_string())
}

pub(super) fn merge_env_types(target: &mut FunctionEnv, source: &FunctionEnv) {
    for (name, ty) in &source.vars {
        let merged = unify_types(
            target.vars.get(name).cloned().unwrap_or(Type::Unknown),
            ty.clone(),
        );
        target.vars.insert(name.clone(), merged);
    }
    target.merge_exact_dict_writes(source);
}

pub(super) fn infer_expr_type(
    expr: &Expr,
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) -> Type {
    match expr {
        Expr::Name(name) => lookup_name_type(name.id.as_str(), env, states, ctx),
        Expr::NumberLiteral(num) => match &num.value {
            sifr_python_ast::Number::Int(_) => Type::Int,
            sifr_python_ast::Number::Float(_) => Type::Float,
            sifr_python_ast::Number::Complex { .. } => Type::Unknown,
        },
        Expr::StringLiteral(_) => Type::Str,
        Expr::BytesLiteral(_) => Type::Bytes,
        Expr::BooleanLiteral(_) => Type::Bool,
        Expr::NoneLiteral(_) => Type::None,
        Expr::List(list) => infer_list_literal_type(&list.elts, env, states, current_function, ctx),
        Expr::Tuple(tuple) => Type::Tuple(
            tuple
                .elts
                .iter()
                .map(|elt| infer_expr_type(elt, env, states, current_function, ctx))
                .collect(),
        ),
        Expr::Dict(dict) => infer_dict_literal_type(dict, env, states, current_function, ctx),
        Expr::Call(call) => infer_call_type(call, env, states, current_function, ctx),
        Expr::Attribute(_) => Type::Unknown,
        Expr::Subscript(sub) => infer_subscript_type(
            sub.value.as_ref(),
            sub.slice.as_ref(),
            env,
            states,
            current_function,
            ctx,
        ),
        Expr::BinOp(binop) => infer_binop_type(
            binop.left.as_ref(),
            binop.right.as_ref(),
            binop.op,
            env,
            states,
            current_function,
            ctx,
        ),
        Expr::Compare(compare) => {
            let left_ty = infer_expr_type(&compare.left, env, states, current_function, ctx);
            for comparator in &compare.comparators {
                let comparator_ty = infer_expr_type(comparator, env, states, current_function, ctx);
                if let Expr::Name(name) = compare.left.as_ref() {
                    refine_name_with_compare_context(
                        name.id.as_str(),
                        &left_ty,
                        &comparator_ty,
                        compare.ops[0],
                        env,
                        states,
                        current_function,
                    );
                }
                if let Expr::Name(name) = comparator {
                    refine_name_with_compare_context(
                        name.id.as_str(),
                        &comparator_ty,
                        &left_ty,
                        compare.ops[0],
                        env,
                        states,
                        current_function,
                    );
                }
            }
            Type::Bool
        }
        Expr::BoolOp(boolop) => {
            for value in &boolop.values {
                let _ = infer_expr_type(value, env, states, current_function, ctx);
            }
            Type::Bool
        }
        Expr::UnaryOp(unary) => {
            let operand_ty = infer_expr_type(&unary.operand, env, states, current_function, ctx);
            match unary.op {
                sifr_python_ast::UnaryOp::Not => Type::Bool,
                _ => operand_ty,
            }
        }
        Expr::If(if_expr) => {
            let _ = infer_expr_type(&if_expr.test, env, states, current_function, ctx);
            let body_ty = infer_expr_type(&if_expr.body, env, states, current_function, ctx);
            let else_ty = infer_expr_type(&if_expr.orelse, env, states, current_function, ctx);
            unify_types(body_ty, else_ty)
        }
        Expr::Slice(_) => Type::Unknown,
        _ => Type::Unknown,
    }
}

pub(super) fn infer_list_literal_type(
    elements: &[Expr],
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) -> Type {
    let mut elem_ty = Type::Unknown;
    for element in elements {
        elem_ty = unify_types(
            elem_ty,
            infer_expr_type(element, env, states, current_function, ctx),
        );
    }
    Type::List(Box::new(elem_ty))
}

pub(super) fn infer_dict_literal_type(
    dict: &sifr_python_ast::ExprDict,
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) -> Type {
    let mut key_ty = Type::Unknown;
    let mut value_ty = Type::Unknown;
    for item in &dict.items {
        let Some(key) = item.key.as_ref() else {
            continue;
        };
        let value = &item.value;
        key_ty = unify_types(
            key_ty,
            infer_expr_type(key, env, states, current_function, ctx),
        );
        value_ty = unify_types(
            value_ty,
            infer_expr_type(value, env, states, current_function, ctx),
        );
    }
    Type::Dict(Box::new(key_ty), Box::new(value_ty))
}

pub(super) fn infer_call_type(
    call: &ExprCall,
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) -> Type {
    match call.func.as_ref() {
        Expr::Name(name) => {
            if name.id == "range" {
                for arg in &call.arguments.args {
                    let _ = infer_expr_type(arg, env, states, current_function, ctx);
                }
                return Type::Range;
            }
            if name.id == "enumerate" {
                let elem_ty = call
                    .arguments
                    .args
                    .first()
                    .map(|arg| infer_expr_type(arg, env, states, current_function, ctx))
                    .and_then(|arg_ty| arg_ty.iterable_element_type())
                    .unwrap_or(Type::Unknown);
                return Type::Iterator(Box::new(Type::Tuple(vec![Type::Int, elem_ty])));
            }
            if name.id == "zip" {
                let tuple_members = call
                    .arguments
                    .args
                    .iter()
                    .map(|arg| infer_expr_type(arg, env, states, current_function, ctx))
                    .map(|arg_ty| arg_ty.iterable_element_type().unwrap_or(Type::Unknown))
                    .collect::<Vec<_>>();
                return Type::Iterator(Box::new(Type::Tuple(tuple_members)));
            }
            if name.id == "list" {
                let elem_ty = call
                    .arguments
                    .args
                    .first()
                    .map(|arg| infer_expr_type(arg, env, states, current_function, ctx))
                    .and_then(|arg_ty| arg_ty.iterable_element_type())
                    .unwrap_or(Type::Unknown);
                return Type::List(Box::new(elem_ty));
            }
            if name.id == "set" {
                let elem_ty = call
                    .arguments
                    .args
                    .first()
                    .map(|arg| infer_expr_type(arg, env, states, current_function, ctx))
                    .and_then(|arg_ty| arg_ty.iterable_element_type())
                    .unwrap_or(Type::Unknown);
                return Type::Set(Box::new(elem_ty));
            }
            if name.id == "dict" {
                if let Some(arg) = call.arguments.args.first() {
                    let arg_ty = infer_expr_type(arg, env, states, current_function, ctx);
                    let item_ty = arg_ty.iterable_element_type().unwrap_or(Type::Unknown);
                    if let Type::Tuple(item_members) = item_ty {
                        if item_members.len() == 2 {
                            return Type::Dict(
                                Box::new(item_members[0].clone()),
                                Box::new(item_members[1].clone()),
                            );
                        }
                    }
                }
                return Type::Dict(Box::new(Type::Unknown), Box::new(Type::Unknown));
            }
            if name.id == "sorted" {
                let elem_ty = call
                    .arguments
                    .args
                    .first()
                    .map(|arg| infer_expr_type(arg, env, states, current_function, ctx))
                    .and_then(|arg_ty| arg_ty.iterable_element_type())
                    .unwrap_or(Type::Unknown);
                return Type::List(Box::new(elem_ty));
            }
            if name.id == "sum" {
                let elem_ty = call
                    .arguments
                    .args
                    .first()
                    .map(|arg| infer_expr_type(arg, env, states, current_function, ctx))
                    .and_then(|arg_ty| arg_ty.iterable_element_type())
                    .unwrap_or(Type::Unknown);
                return match elem_ty.resolve_alias() {
                    Type::FixedInt(fixed) if fixed.supports_current_int_builtin_widening() => {
                        Type::Int
                    }
                    _ => elem_ty,
                };
            }
            if name.id == "Counter" {
                let key_ty = call
                    .arguments
                    .args
                    .first()
                    .map(|arg| infer_expr_type(arg, env, states, current_function, ctx))
                    .and_then(|arg_ty| arg_ty.iterable_element_type())
                    .unwrap_or(Type::Unknown);
                return Type::Dict(Box::new(key_ty), Box::new(Type::Int));
            }
            if name.id == "len" {
                if let Some(arg) = call.arguments.args.first() {
                    let _ = infer_expr_type(arg, env, states, current_function, ctx);
                }
                return Type::Int;
            }
            if name.id == "abs" {
                let arg_ty = call
                    .arguments
                    .args
                    .first()
                    .map(|arg| infer_expr_type(arg, env, states, current_function, ctx))
                    .unwrap_or(Type::Unknown);
                return match arg_ty.resolve_alias() {
                    Type::FixedInt(fixed) if fixed.supports_current_int_builtin_widening() => {
                        Type::Int
                    }
                    _ => arg_ty,
                };
            }
            if name.id == "max" || name.id == "min" {
                let mut result = Type::Unknown;
                for arg in &call.arguments.args {
                    let arg_ty = infer_expr_type(arg, env, states, current_function, ctx);
                    result = unify_types(result, arg_ty);
                }
                return result;
            }
            if let Some(state) = states.get(name.id.as_str()).cloned() {
                let inferred = infer_registered_call(
                    call,
                    &state.function_type(),
                    env,
                    states,
                    current_function,
                    ctx,
                );
                for (index, arg_ty) in inferred.positional_types.into_iter().enumerate() {
                    if let Some(param_name) =
                        state.params.get(index).map(|param| param.name.clone())
                    {
                        unify_function_param(name.id.as_str(), param_name.as_str(), arg_ty, states);
                    }
                }
                return inferred.return_type;
            }
            if let Some(function_type) = ctx.functions.get(name.id.as_str()).cloned() {
                return infer_registered_call(
                    call,
                    &function_type,
                    env,
                    states,
                    current_function,
                    ctx,
                )
                .return_type;
            }
            Type::Unknown
        }
        Expr::Attribute(attr) => infer_attribute_call_type(
            attr.value.as_ref(),
            attr.attr.as_str(),
            &call.arguments.args,
            env,
            states,
            current_function,
            ctx,
        ),
        _ => Type::Unknown,
    }
}

pub(super) fn infer_attribute_call_type(
    object: &Expr,
    method: &str,
    args: &[Expr],
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) -> Type {
    if let Expr::Name(module_name) = object {
        match (module_name.id.as_str(), method) {
            ("heapq", "heapify") => {
                if let Some(arg) = args.first() {
                    let _ = infer_expr_type(arg, env, states, current_function, ctx);
                }
                return Type::None;
            }
            ("heapq", "heappop") => {
                if let Some(arg) = args.first() {
                    let arg_ty = infer_expr_type(arg, env, states, current_function, ctx);
                    if let Type::List(elem_ty) = arg_ty {
                        return *elem_ty;
                    }
                }
                return Type::Unknown;
            }
            ("heapq", "heappush") => {
                for arg in args {
                    let _ = infer_expr_type(arg, env, states, current_function, ctx);
                }
                return Type::None;
            }
            _ => {}
        }
    }

    let object_ty = infer_expr_type(object, env, states, current_function, ctx);
    let arg_types = args
        .iter()
        .map(|arg| infer_expr_type(arg, env, states, current_function, ctx))
        .collect::<Vec<_>>();

    if let Expr::Name(name) = object {
        match method {
            "append" => {
                let elem_ty = arg_types.first().cloned().unwrap_or(Type::Unknown);
                unify_name_binding(
                    name.id.as_str(),
                    Type::List(Box::new(elem_ty)),
                    env,
                    states,
                    current_function,
                );
                return Type::None;
            }
            "copy" => {
                return object_ty;
            }
            "pop" => {
                if let Type::List(elem_ty) = object_ty {
                    unify_name_binding(
                        name.id.as_str(),
                        Type::List(elem_ty.clone()),
                        env,
                        states,
                        current_function,
                    );
                    return *elem_ty;
                }
                unify_name_binding(
                    name.id.as_str(),
                    Type::List(Box::new(Type::Unknown)),
                    env,
                    states,
                    current_function,
                );
                return Type::Unknown;
            }
            "sort" => {
                return Type::None;
            }
            "keys" => {
                if let Type::Dict(key_ty, _) = &object_ty {
                    return Type::List(Box::new(*key_ty.clone()));
                }
                return Type::List(Box::new(Type::Unknown));
            }
            "values" => {
                if let Type::Dict(_, value_ty) = &object_ty {
                    return Type::List(Box::new(*value_ty.clone()));
                }
                return Type::List(Box::new(Type::Unknown));
            }
            "items" => {
                if let Type::Dict(key_ty, value_ty) = &object_ty {
                    return Type::List(Box::new(Type::Tuple(vec![
                        *key_ty.clone(),
                        *value_ty.clone(),
                    ])));
                }
                return Type::List(Box::new(Type::Tuple(vec![Type::Unknown, Type::Unknown])));
            }
            "get" => {
                if let Type::Dict(key_ty, value_ty) = &object_ty {
                    if let Some(key_arg_ty) = arg_types.first() {
                        let refined_key = unify_types(*key_ty.clone(), key_arg_ty.clone());
                        unify_name_binding(
                            name.id.as_str(),
                            Type::Dict(Box::new(refined_key), value_ty.clone()),
                            env,
                            states,
                            current_function,
                        );
                    }
                    if arg_types.len() == 2 {
                        return unify_types(*value_ty.clone(), arg_types[1].clone());
                    }
                    return *value_ty.clone();
                }
                if arg_types.len() == 2 {
                    return arg_types[1].clone();
                }
                return Type::Unknown;
            }
            "setdefault" => {
                if arg_types.len() == 2 {
                    let key_ty = arg_types[0].clone();
                    let value_ty = arg_types[1].clone();
                    unify_name_binding(
                        name.id.as_str(),
                        Type::Dict(Box::new(key_ty), Box::new(value_ty.clone())),
                        env,
                        states,
                        current_function,
                    );
                    return value_ty;
                }
                return Type::Unknown;
            }
            _ => {}
        }
    }

    match method {
        "copy" => object_ty,
        "split" => {
            if matches!(object_ty, Type::Str) {
                Type::List(Box::new(Type::Str))
            } else {
                Type::List(Box::new(Type::Unknown))
            }
        }
        "append" | "pop" | "sort" | "heapify" | "heappush" => Type::None,
        _ => Type::Unknown,
    }
}

pub(super) fn infer_subscript_type(
    object: &Expr,
    index: &Expr,
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) -> Type {
    let object_ty = infer_expr_type(object, env, states, current_function, ctx);
    let index_ty = infer_expr_type(index, env, states, current_function, ctx);
    if let Expr::Name(name) = index {
        if !matches!(index_ty, Type::Str) {
            unify_name_binding(name.id.as_str(), Type::Int, env, states, current_function);
        }
    }

    if let Expr::Slice(_) = index {
        return match object_ty {
            Type::List(elem_ty) => Type::List(elem_ty),
            Type::Str => Type::Str,
            other => other,
        };
    }

    match object_ty {
        Type::List(elem_ty) => *elem_ty,
        Type::Dict(_, value_ty) => *value_ty,
        Type::Str => Type::Str,
        Type::Tuple(elements) => elements.first().cloned().unwrap_or(Type::Unknown),
        _ => Type::Unknown,
    }
}

pub(super) fn infer_binop_type(
    left: &Expr,
    right: &Expr,
    op: Operator,
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) -> Type {
    let left_ty = infer_expr_type(left, env, states, current_function, ctx);
    let right_ty = infer_expr_type(right, env, states, current_function, ctx);

    if let Expr::Name(name) = left {
        refine_name_with_binary_context(
            name.id.as_str(),
            &right_ty,
            op,
            env,
            states,
            current_function,
        );
    }
    if let Expr::Name(name) = right {
        refine_name_with_binary_context(
            name.id.as_str(),
            &left_ty,
            op,
            env,
            states,
            current_function,
        );
    }

    let op_str = match op {
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
        Operator::MatMult => return Type::Unknown,
    };

    type_check_binary_op(&left_ty, op_str, &right_ty)
        .unwrap_or_else(|_| infer_numeric_result_type(&left_ty, &right_ty, op))
}

pub(super) fn infer_numeric_result_type(left_ty: &Type, right_ty: &Type, op: Operator) -> Type {
    match op {
        Operator::Div => Type::Float,
        Operator::Add | Operator::Sub | Operator::Mult | Operator::Pow => {
            if matches!(left_ty, Type::Float) || matches!(right_ty, Type::Float) {
                Type::Float
            } else if matches!(left_ty, Type::Str) || matches!(right_ty, Type::Str) {
                Type::Str
            } else {
                Type::Int
            }
        }
        Operator::FloorDiv
        | Operator::Mod
        | Operator::BitAnd
        | Operator::BitOr
        | Operator::BitXor
        | Operator::LShift
        | Operator::RShift => Type::Int,
        Operator::MatMult => Type::Unknown,
    }
}

pub(super) fn refine_name_with_binary_context(
    name: &str,
    other_ty: &Type,
    op: Operator,
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
) {
    let inferred = match op {
        Operator::Add => {
            if matches!(other_ty, Type::Str) {
                Type::Str
            } else if matches!(other_ty, Type::Float) {
                Type::Float
            } else {
                Type::Int
            }
        }
        Operator::Mult => {
            if matches!(other_ty, Type::Float) {
                Type::Float
            } else {
                Type::Int
            }
        }
        Operator::Sub
        | Operator::FloorDiv
        | Operator::Mod
        | Operator::BitAnd
        | Operator::BitOr
        | Operator::BitXor
        | Operator::LShift
        | Operator::RShift => Type::Int,
        Operator::Div | Operator::Pow => Type::Float,
        Operator::MatMult => Type::Unknown,
    };

    unify_name_binding(name, inferred, env, states, current_function);
}

pub(super) fn refine_name_with_compare_context(
    name: &str,
    current_ty: &Type,
    other_ty: &Type,
    op: CmpOp,
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
) {
    let _ = current_ty;
    let inferred = match op {
        CmpOp::Eq | CmpOp::NotEq | CmpOp::Lt | CmpOp::LtE | CmpOp::Gt | CmpOp::GtE => {
            if matches!(other_ty, Type::Int) {
                Type::Int
            } else if matches!(other_ty, Type::Float) {
                Type::Float
            } else {
                Type::Unknown
            }
        }
        _ => Type::Unknown,
    };

    if !inferred.is_unknown() {
        unify_name_binding(name, inferred, env, states, current_function);
    }
}

pub(super) fn lookup_name_type(
    name: &str,
    env: &FunctionEnv,
    states: &HashMap<String, LocalFunctionState<'_>>,
    ctx: &LowerCtx,
) -> Type {
    if let Some(ty) = env.vars.get(name) {
        return ty.clone();
    }
    if let Some(info) = ctx.scope.lookup(name) {
        return info.effective_type().clone();
    }
    if let Some(state) = states.get(name) {
        return Type::Function(state.function_type());
    }
    match name {
        "True" | "False" => Type::Bool,
        _ => Type::Unknown,
    }
}

pub(super) fn unify_name_binding(
    name: &str,
    incoming: Type,
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
) {
    let existing = env.vars.get(name).cloned().unwrap_or(Type::Unknown);
    let merged = unify_types(existing, incoming);
    env.vars.insert(name.to_string(), merged.clone());

    if let Some(function_name) = current_function {
        if let Some(state) = states.get_mut(function_name) {
            if let Some(param) = state.params.iter_mut().find(|param| param.name == name) {
                if !param.explicit && has_conflicting_inference(&param.ty, &merged) {
                    param.ty = Type::Unknown;
                    state.inference_failed = true;
                } else {
                    param.ty = unify_types(param.ty.clone(), merged.clone());
                }
            }
        }
    }

    if let Some(callee_name) = env.call_return_origins.get(name).cloned() {
        unify_function_return(callee_name.as_str(), merged, states);
    }
}

pub(super) fn unify_function_param(
    function_name: &str,
    param_name: &str,
    incoming: Type,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
) {
    let Some(state) = states.get_mut(function_name) else {
        return;
    };
    let Some(param) = state
        .params
        .iter_mut()
        .find(|param| param.name == param_name)
    else {
        return;
    };
    if !param.explicit && has_conflicting_inference(&param.ty, &incoming) {
        param.ty = Type::Unknown;
        state.inference_failed = true;
    } else {
        param.ty = unify_types(param.ty.clone(), incoming);
    }
}

pub(super) fn has_conflicting_inference(current: &Type, incoming: &Type) -> bool {
    match (current, incoming) {
        (Type::Unknown, _) | (_, Type::Unknown) => false,
        (Type::List(current_elem), Type::List(incoming_elem)) => {
            has_conflicting_inference(current_elem, incoming_elem)
        }
        (Type::Dict(current_key, current_value), Type::Dict(incoming_key, incoming_value)) => {
            has_conflicting_inference(current_key, incoming_key)
                || has_conflicting_inference(current_value, incoming_value)
        }
        _ => !current.is_assignable_to(incoming) && !incoming.is_assignable_to(current),
    }
}

pub(super) fn unify_types(current: Type, incoming: Type) -> Type {
    let current = collapse_literal(current);
    let incoming = collapse_literal(incoming);

    if current.is_unknown() {
        return incoming;
    }
    if incoming.is_unknown() {
        return current;
    }
    if current == incoming {
        return current;
    }

    match (&current, &incoming) {
        (Type::List(current_elem), Type::List(incoming_elem)) => Type::List(Box::new(unify_types(
            (**current_elem).clone(),
            (**incoming_elem).clone(),
        ))),
        (Type::Dict(current_key, current_value), Type::Dict(incoming_key, incoming_value)) => {
            Type::Dict(
                Box::new(unify_types(
                    (**current_key).clone(),
                    (**incoming_key).clone(),
                )),
                Box::new(unify_types(
                    (**current_value).clone(),
                    (**incoming_value).clone(),
                )),
            )
        }
        (Type::Float, Type::Int) | (Type::Int, Type::Float) => Type::Float,
        _ if incoming.is_assignable_to(&current) => current,
        _ if current.is_assignable_to(&incoming) => incoming,
        _ => current,
    }
}

pub(super) fn type_contains_unknown_or_any(ty: &Type) -> bool {
    match ty {
        Type::Unknown | Type::Any => true,
        Type::List(elem) => type_contains_unknown_or_any(elem),
        Type::Dict(key, value) => {
            type_contains_unknown_or_any(key) || type_contains_unknown_or_any(value)
        }
        Type::Tuple(elements) => elements.iter().any(type_contains_unknown_or_any),
        _ => false,
    }
}

pub(super) fn collapse_literal(ty: Type) -> Type {
    match ty {
        Type::LiteralInt(_) => Type::Int,
        Type::LiteralStr(_) => Type::Str,
        Type::LiteralBool(_) => Type::Bool,
        Type::List(elem_ty) => Type::List(Box::new(collapse_literal(*elem_ty))),
        Type::Dict(key_ty, value_ty) => Type::Dict(
            Box::new(collapse_literal(*key_ty)),
            Box::new(collapse_literal(*value_ty)),
        ),
        other => other,
    }
}
