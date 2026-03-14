use sifr_python_ast::{CmpOp, Expr, ExprCall, Operator, Stmt, StmtFunctionDef};
use sifr_type_system::{type_check_binary_op, FunctionType, Type};
use std::collections::{HashMap, HashSet};

use super::typing_and_functions::{ast_convention_to_param, resolve_annotation_expr};
use super::LowerCtx;

const MAX_INFERENCE_PASSES: usize = 8;

pub(super) struct NestedFunctionInference {
    pub(super) function_types: HashMap<String, FunctionType>,
    pub(super) binding_hints: HashMap<String, Type>,
}

#[derive(Clone)]
struct ParamState {
    name: String,
    ty: Type,
    explicit: bool,
    convention: sifr_python_ast::AstParamConvention,
}

#[derive(Clone)]
struct LocalFunctionState<'a> {
    func: &'a StmtFunctionDef,
    params: Vec<ParamState>,
    return_type: Type,
    explicit_return: bool,
    inference_failed: bool,
}

impl LocalFunctionState<'_> {
    fn function_type(&self) -> FunctionType {
        FunctionType {
            params: self
                .params
                .iter()
                .map(|param| {
                    (
                        param.name.clone(),
                        param.ty.clone(),
                        ast_convention_to_param(param.convention, &param.ty),
                    )
                })
                .collect(),
            return_type: Box::new(self.return_type.clone()),
        }
    }
}

#[derive(Clone, Default)]
struct FunctionEnv {
    vars: HashMap<String, Type>,
    call_return_origins: HashMap<String, String>,
}

impl FunctionEnv {
    fn bind_var(&mut self, name: &str, ty: Type) {
        let ty = if let Some(existing) = self.vars.get(name) {
            if type_contains_unknown_or_any(&ty) {
                unify_types(existing.clone(), ty)
            } else {
                ty
            }
        } else {
            ty
        };
        self.vars.insert(name.to_string(), ty);
        self.call_return_origins.remove(name);
    }

    fn bind_call_result(&mut self, name: String, ty: Type, callee: String) {
        let ty = if let Some(existing) = self.vars.get(&name) {
            if type_contains_unknown_or_any(&ty) {
                unify_types(existing.clone(), ty)
            } else {
                ty
            }
        } else {
            ty
        };
        self.vars.insert(name.clone(), ty);
        self.call_return_origins.insert(name, callee);
    }
}

pub(super) fn infer_nested_function_types(
    stmts: &[Stmt],
    ctx: &mut LowerCtx,
) -> NestedFunctionInference {
    let mut states = collect_nested_function_states(stmts, ctx);
    if states.is_empty() {
        return NestedFunctionInference {
            function_types: HashMap::new(),
            binding_hints: HashMap::new(),
        };
    }

    let mut binding_hints = HashMap::new();
    for _ in 0..MAX_INFERENCE_PASSES {
        let previous = snapshot_signatures(&states);
        let mut env = FunctionEnv {
            vars: binding_hints.clone(),
            call_return_origins: HashMap::new(),
        };
        analyze_block(stmts, &mut env, &mut states, None, ctx);
        binding_hints = env.vars;
        if snapshot_signatures(&states) == previous {
            break;
        }
    }

    let mut env = FunctionEnv {
        vars: binding_hints,
        call_return_origins: HashMap::new(),
    };
    analyze_block(stmts, &mut env, &mut states, None, ctx);

    NestedFunctionInference {
        function_types: finalize_nested_function_types(&mut states, ctx),
        binding_hints: env.vars,
    }
}

fn collect_nested_function_states<'a>(
    stmts: &'a [Stmt],
    ctx: &mut LowerCtx,
) -> HashMap<String, LocalFunctionState<'a>> {
    let mut states = HashMap::new();

    for stmt in stmts {
        let Stmt::FunctionDef(func) = stmt else {
            continue;
        };

        let mut params = Vec::new();
        for param in &func.parameters.args {
            let name = param.parameter.name.to_string();
            let (ty, explicit) = if let Some(annotation) = &param.parameter.annotation {
                (resolve_annotation_expr(annotation, ctx), true)
            } else {
                (Type::Unknown, false)
            };
            params.push(ParamState {
                name,
                ty,
                explicit,
                convention: param.parameter.convention,
            });
        }

        let return_type = if let Some(returns) = &func.returns {
            resolve_annotation_expr(returns, ctx)
        } else {
            Type::Unknown
        };

        states.insert(
            func.name.to_string(),
            LocalFunctionState {
                func,
                params,
                return_type,
                explicit_return: func.returns.is_some(),
                inference_failed: false,
            },
        );
    }

    states
}

fn snapshot_signatures(
    states: &HashMap<String, LocalFunctionState<'_>>,
) -> Vec<(String, Vec<Type>, Type)> {
    let mut snapshot = states
        .iter()
        .map(|(name, state)| {
            (
                name.clone(),
                state.params.iter().map(|param| param.ty.clone()).collect(),
                state.return_type.clone(),
            )
        })
        .collect::<Vec<_>>();
    snapshot.sort_by(|left, right| left.0.cmp(&right.0));
    snapshot
}

fn finalize_nested_function_types(
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    ctx: &mut LowerCtx,
) -> HashMap<String, FunctionType> {
    let mut result = HashMap::new();

    for state in states.values_mut() {
        for param in &mut state.params {
            if !param.explicit && param.ty.is_unknown() {
                state.inference_failed = true;
                ctx.error(format!(
                    "parameter '{}' in function '{}' is missing a type annotation and could not be inferred",
                    param.name, state.func.name
                ));
                param.ty = Type::Any;
            }
        }

        if !state.explicit_return {
            state.return_type = finalize_return_type(state);
            if state.return_type.is_unknown() {
                state.inference_failed = true;
                ctx.error(format!(
                    "function '{}' return type could not be inferred deterministically",
                    state.func.name
                ));
                state.return_type = Type::Any;
            }
        }

        result.insert(state.func.name.to_string(), state.function_type());
    }

    result
}

fn finalize_return_type(state: &LocalFunctionState<'_>) -> Type {
    if state.return_type.is_unknown() {
        if function_has_value_return(&state.func.body) {
            Type::Unknown
        } else {
            Type::None
        }
    } else {
        state.return_type.clone()
    }
}

fn function_has_value_return(stmts: &[Stmt]) -> bool {
    for stmt in stmts {
        match stmt {
            Stmt::Return(ret) => {
                if ret.value.is_some() {
                    return true;
                }
            }
            Stmt::If(if_stmt) => {
                if function_has_value_return(&if_stmt.body) {
                    return true;
                }
                for clause in &if_stmt.elif_else_clauses {
                    if function_has_value_return(&clause.body) {
                        return true;
                    }
                }
            }
            Stmt::While(while_stmt) => {
                if function_has_value_return(&while_stmt.body) {
                    return true;
                }
                if !while_stmt.orelse.is_empty() && function_has_value_return(&while_stmt.orelse) {
                    return true;
                }
            }
            Stmt::For(for_stmt) => {
                if function_has_value_return(&for_stmt.body) {
                    return true;
                }
                if !for_stmt.orelse.is_empty() && function_has_value_return(&for_stmt.orelse) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn analyze_block(
    stmts: &[Stmt],
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) {
    for stmt in stmts {
        analyze_stmt(stmt, env, states, current_function, ctx);
    }
}

fn collect_current_function_local_bindings(stmts: &[Stmt], bindings: &mut HashSet<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Assign(assign) => {
                collect_assignment_target_names(&assign.targets, bindings);
            }
            Stmt::AnnAssign(assign) => {
                collect_assignment_target_names(
                    std::slice::from_ref(assign.target.as_ref()),
                    bindings,
                );
            }
            Stmt::AugAssign(assign) => {
                collect_assignment_target_names(
                    std::slice::from_ref(assign.target.as_ref()),
                    bindings,
                );
            }
            Stmt::For(for_stmt) => {
                collect_assignment_target_names(
                    std::slice::from_ref(for_stmt.target.as_ref()),
                    bindings,
                );
                collect_current_function_local_bindings(&for_stmt.body, bindings);
                collect_current_function_local_bindings(&for_stmt.orelse, bindings);
            }
            Stmt::While(while_stmt) => {
                collect_current_function_local_bindings(&while_stmt.body, bindings);
                collect_current_function_local_bindings(&while_stmt.orelse, bindings);
            }
            Stmt::If(if_stmt) => {
                collect_current_function_local_bindings(&if_stmt.body, bindings);
                for clause in &if_stmt.elif_else_clauses {
                    collect_current_function_local_bindings(&clause.body, bindings);
                }
            }
            Stmt::With(with_stmt) => {
                for item in &with_stmt.items {
                    if let Some(optional_vars) = &item.optional_vars {
                        collect_assignment_target_names(
                            std::slice::from_ref(optional_vars.as_ref()),
                            bindings,
                        );
                    }
                }
                collect_current_function_local_bindings(&with_stmt.body, bindings);
            }
            Stmt::FunctionDef(func) => {
                bindings.insert(func.name.to_string());
            }
            _ => {}
        }
    }
}

fn collect_assignment_target_names(targets: &[Expr], bindings: &mut HashSet<String>) {
    for target in targets {
        match target {
            Expr::Name(name) => {
                bindings.insert(name.id.clone());
            }
            Expr::Tuple(tuple) => {
                collect_assignment_target_names(&tuple.elts, bindings);
            }
            _ => {}
        }
    }
}

fn collect_nonlocal_names(stmts: &[Stmt], names: &mut HashSet<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Nonlocal(nonlocal_stmt) => {
                for name in &nonlocal_stmt.names {
                    names.insert(name.to_string());
                }
            }
            Stmt::For(for_stmt) => {
                collect_nonlocal_names(&for_stmt.body, names);
                collect_nonlocal_names(&for_stmt.orelse, names);
            }
            Stmt::While(while_stmt) => {
                collect_nonlocal_names(&while_stmt.body, names);
                collect_nonlocal_names(&while_stmt.orelse, names);
            }
            Stmt::If(if_stmt) => {
                collect_nonlocal_names(&if_stmt.body, names);
                for clause in &if_stmt.elif_else_clauses {
                    collect_nonlocal_names(&clause.body, names);
                }
            }
            Stmt::With(with_stmt) => {
                collect_nonlocal_names(&with_stmt.body, names);
            }
            _ => {}
        }
    }
}

fn analyze_stmt(
    stmt: &Stmt,
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) {
    match stmt {
        Stmt::AnnAssign(assign) => {
            if let Expr::Name(name) = assign.target.as_ref() {
                if let Some(value) = &assign.value {
                    let value_ty = infer_expr_type(value, env, states, current_function, ctx);
                    env.bind_var(name.id.as_str(), value_ty);
                }
            }
        }
        Stmt::Assign(assign) => analyze_assign(
            assign.targets.as_slice(),
            assign.value.as_ref(),
            env,
            states,
            current_function,
            ctx,
        ),
        Stmt::AugAssign(aug) => {
            let value_ty = infer_expr_type(&aug.value, env, states, current_function, ctx);
            if let Expr::Name(name) = aug.target.as_ref() {
                refine_name_with_binary_context(
                    name.id.as_str(),
                    &value_ty,
                    aug.op,
                    env,
                    states,
                    current_function,
                );
            }
        }
        Stmt::Expr(expr_stmt) => {
            let _ = infer_expr_type(&expr_stmt.value, env, states, current_function, ctx);
        }
        Stmt::Return(ret) => {
            if let Some(value) = &ret.value {
                let return_ty = infer_expr_type(value, env, states, current_function, ctx);
                if let Some(function_name) = current_function {
                    unify_function_return(function_name, return_ty, states);
                }
            } else if let Some(function_name) = current_function {
                unify_function_return(function_name, Type::None, states);
            }
        }
        Stmt::If(if_stmt) => {
            let _ = infer_expr_type(&if_stmt.test, env, states, current_function, ctx);

            let mut then_env = env.clone();
            analyze_block(&if_stmt.body, &mut then_env, states, current_function, ctx);
            merge_env_types(env, &then_env);

            for clause in &if_stmt.elif_else_clauses {
                if let Some(test) = &clause.test {
                    let _ = infer_expr_type(test, env, states, current_function, ctx);
                }
                let mut elif_env = env.clone();
                analyze_block(&clause.body, &mut elif_env, states, current_function, ctx);
                merge_env_types(env, &elif_env);
            }

            if let Some(else_clause) = if_stmt
                .elif_else_clauses
                .iter()
                .find(|clause| clause.test.is_none())
            {
                let mut else_env = env.clone();
                analyze_block(
                    &else_clause.body,
                    &mut else_env,
                    states,
                    current_function,
                    ctx,
                );
                merge_env_types(env, &else_env);
            }
        }
        Stmt::While(while_stmt) => {
            let _ = infer_expr_type(&while_stmt.test, env, states, current_function, ctx);
            let mut body_env = env.clone();
            analyze_block(
                &while_stmt.body,
                &mut body_env,
                states,
                current_function,
                ctx,
            );
            merge_env_types(env, &body_env);
            if !while_stmt.orelse.is_empty() {
                let mut else_env = env.clone();
                analyze_block(
                    &while_stmt.orelse,
                    &mut else_env,
                    states,
                    current_function,
                    ctx,
                );
                merge_env_types(env, &else_env);
            }
        }
        Stmt::For(for_stmt) => {
            let iter_ty = infer_expr_type(&for_stmt.iter, env, states, current_function, ctx);
            let elem_ty = iter_ty.iterable_element_type().unwrap_or(Type::Unknown);
            let mut body_env = env.clone();
            if let Expr::Name(name) = for_stmt.target.as_ref() {
                body_env.bind_var(name.id.as_str(), elem_ty);
            }
            analyze_block(&for_stmt.body, &mut body_env, states, current_function, ctx);
            merge_env_types(env, &body_env);
            if !for_stmt.orelse.is_empty() {
                let mut else_env = env.clone();
                analyze_block(
                    &for_stmt.orelse,
                    &mut else_env,
                    states,
                    current_function,
                    ctx,
                );
                merge_env_types(env, &else_env);
            }
        }
        Stmt::FunctionDef(func) => {
            if !states.contains_key(func.name.as_str()) {
                return;
            }
            let state = states
                .get(func.name.as_str())
                .cloned()
                .expect("state present");
            let outer_names = env.vars.keys().cloned().collect::<HashSet<_>>();
            let param_names = state
                .params
                .iter()
                .map(|param| param.name.clone())
                .collect::<HashSet<_>>();
            let mut local_bindings = HashSet::new();
            collect_current_function_local_bindings(&func.body, &mut local_bindings);
            let mut nonlocal_names = HashSet::new();
            collect_nonlocal_names(&func.body, &mut nonlocal_names);
            let mut nested_env = env.clone();
            for param in &state.params {
                nested_env.bind_var(param.name.as_str(), param.ty.clone());
            }
            analyze_block(
                &func.body,
                &mut nested_env,
                states,
                Some(func.name.as_str()),
                ctx,
            );
            for name in outer_names {
                let Some(refined_ty) = nested_env.vars.get(&name).cloned() else {
                    continue;
                };
                if param_names.contains(&name) {
                    continue;
                }
                if local_bindings.contains(&name) && !nonlocal_names.contains(&name) {
                    continue;
                }
                unify_name_binding(name.as_str(), refined_ty, env, states, current_function);
            }
        }
        _ => {}
    }
}

fn analyze_assign(
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
                env.bind_call_result(name.id.clone(), value_ty, callee_name);
            } else {
                env.bind_var(name.id.as_str(), value_ty);
            }
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

fn nested_call_target_name(
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
        .then(|| name.id.clone())
}

fn merge_env_types(target: &mut FunctionEnv, source: &FunctionEnv) {
    for (name, ty) in &source.vars {
        let merged = unify_types(
            target.vars.get(name).cloned().unwrap_or(Type::Unknown),
            ty.clone(),
        );
        target.vars.insert(name.clone(), merged);
    }
}

fn infer_expr_type(
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

fn infer_list_literal_type(
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

fn infer_dict_literal_type(
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

fn infer_call_type(
    call: &ExprCall,
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) -> Type {
    match call.func.as_ref() {
        Expr::Name(name) => {
            if name.id == "len" {
                if let Some(arg) = call.arguments.args.first() {
                    let _ = infer_expr_type(arg, env, states, current_function, ctx);
                }
                return Type::Int;
            }
            if name.id == "abs" {
                return call
                    .arguments
                    .args
                    .first()
                    .map(|arg| infer_expr_type(arg, env, states, current_function, ctx))
                    .unwrap_or(Type::Unknown);
            }
            if let Some(state) = states.get(name.id.as_str()).cloned() {
                for (index, arg) in call.arguments.args.iter().enumerate() {
                    let arg_ty = infer_expr_type(arg, env, states, current_function, ctx);
                    if let Some(param_name) =
                        state.params.get(index).map(|param| param.name.clone())
                    {
                        unify_function_param(name.id.as_str(), param_name.as_str(), arg_ty, states);
                    }
                }
                return states
                    .get(name.id.as_str())
                    .map(|state| state.return_type.clone())
                    .unwrap_or(Type::Unknown);
            }
            if let Some(function_type) = ctx.functions.get(name.id.as_str()) {
                for arg in &call.arguments.args {
                    let _ = infer_expr_type(arg, env, states, current_function, ctx);
                }
                return (*function_type.return_type).clone();
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

fn infer_attribute_call_type(
    object: &Expr,
    method: &str,
    args: &[Expr],
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) -> Type {
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
            _ => {}
        }
    }

    match method {
        "copy" => object_ty,
        "append" | "pop" | "sort" => Type::None,
        _ => Type::Unknown,
    }
}

fn infer_subscript_type(
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

fn infer_binop_type(
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

fn infer_numeric_result_type(left_ty: &Type, right_ty: &Type, op: Operator) -> Type {
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

fn refine_name_with_binary_context(
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

fn refine_name_with_compare_context(
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

fn lookup_name_type(
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

fn unify_name_binding(
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

fn unify_function_param(
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

fn unify_function_return(
    function_name: &str,
    incoming: Type,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
) {
    let Some(state) = states.get_mut(function_name) else {
        return;
    };
    if !state.explicit_return && has_conflicting_inference(&state.return_type, &incoming) {
        state.return_type = Type::Unknown;
        state.inference_failed = true;
    } else {
        state.return_type = unify_types(state.return_type.clone(), incoming);
    }
}

fn has_conflicting_inference(current: &Type, incoming: &Type) -> bool {
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

fn unify_types(current: Type, incoming: Type) -> Type {
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

fn type_contains_unknown_or_any(ty: &Type) -> bool {
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

fn collapse_literal(ty: Type) -> Type {
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
