use super::capture_collection::collect_nested_function_captures;
use super::mutation_collection::{collect_mutated_binding_names, MutationCandidate};
use super::{
    analyze_assign, analyze_match_stmt, analyze_try_stmt, analyze_with_stmt,
    collect_compound_local_bindings, collect_compound_nonlocals, function_has_value_return,
    infer_expr_type, inference_stmt_always_exits, merge_env_types, refine_name_with_binary_context,
    str, type_contains_unknown_or_any, unify_function_return, unify_name_binding, unify_types,
};
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{AstParamConvention, Expr, Stmt, StmtFunctionDef};
use sifr_type_system::{FunctionType, Type};
use std::collections::{HashMap, HashSet};

use super::typing_and_functions::{ast_convention_to_param, resolve_annotation_expr};
use super::LowerCtx;

const MAX_INFERENCE_PASSES: usize = 8;

pub(in crate::lower) struct NestedFunctionInference {
    pub(in crate::lower) function_types: HashMap<String, FunctionType>,
    pub(in crate::lower) binding_hints: HashMap<String, Type>,
    pub(in crate::lower) function_captures: HashMap<String, Vec<(String, Type)>>,
    pub(in crate::lower) function_mutated_captures: HashMap<String, Vec<String>>,
}

#[derive(Clone)]
pub(super) struct ParamState {
    pub(super) name: String,
    pub(super) name_range: TextRange,
    pub(super) ty: Type,
    pub(super) explicit: bool,
    pub(super) convention: sifr_python_ast::AstParamConvention,
    pub(super) mutated: bool,
}

#[derive(Clone)]
pub(super) struct LocalFunctionState<'a> {
    pub(super) func: &'a StmtFunctionDef,
    pub(super) params: Vec<ParamState>,
    pub(super) return_type: Type,
    pub(super) explicit_return: bool,
    pub(super) inference_failed: bool,
    pub(super) allow_union_return_inference: bool,
}

impl LocalFunctionState<'_> {
    pub(super) fn function_type(&self) -> FunctionType {
        FunctionType {
            params: self
                .params
                .iter()
                .map(|param| {
                    let convention = inferred_param_convention(param);
                    (
                        param.name.clone(),
                        param.ty.clone(),
                        ast_convention_to_param(convention, &param.ty),
                    )
                })
                .collect(),
            return_type: Box::new(self.return_type.clone()),
        }
    }
}

pub(super) fn inferred_param_convention(param: &ParamState) -> AstParamConvention {
    if !param.mutated || param.convention.is_mutable() {
        return param.convention;
    }
    if param.ty.ownership() == sifr_type_system::OwnershipKind::Copy {
        return if param.convention.is_owned() {
            AstParamConvention::own_mut()
        } else {
            param.convention
        };
    }
    if param.convention.is_owned() {
        AstParamConvention::own_mut()
    } else {
        AstParamConvention::mut_borrow()
    }
}

#[derive(Clone, Default)]
pub(super) struct FunctionEnv {
    pub(super) vars: HashMap<String, Type>,
    pub(super) call_return_origins: HashMap<String, String>,
}

impl FunctionEnv {
    pub(super) fn bind_var(&mut self, name: &str, ty: Type) {
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

    pub(super) fn bind_call_result(&mut self, name: String, ty: Type, callee: String) {
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

pub(in crate::lower) fn infer_nested_function_types(
    stmts: &[Stmt],
    ctx: &mut LowerCtx,
) -> NestedFunctionInference {
    infer_function_types(stmts, ctx, false)
}

pub(in crate::lower) fn infer_module_function_types(
    stmts: &[Stmt],
    ctx: &mut LowerCtx,
) -> NestedFunctionInference {
    infer_function_types(stmts, ctx, true)
}

fn infer_function_types(
    stmts: &[Stmt],
    ctx: &mut LowerCtx,
    allow_union_return_inference: bool,
) -> NestedFunctionInference {
    let mut states = collect_function_states(stmts, ctx, allow_union_return_inference);
    let outer_bindings = ctx
        .scope
        .visible_local_bindings()
        .into_iter()
        .collect::<HashMap<_, _>>();
    if states.is_empty() {
        let mut env = FunctionEnv {
            vars: outer_bindings,
            call_return_origins: HashMap::new(),
        };
        analyze_block(stmts, &mut env, &mut states, None, ctx);
        return NestedFunctionInference {
            function_types: HashMap::new(),
            binding_hints: env.vars,
            function_captures: HashMap::new(),
            function_mutated_captures: HashMap::new(),
        };
    }

    let mut binding_hints = outer_bindings;
    let max_passes = if allow_union_return_inference {
        states.len().saturating_add(1)
    } else {
        MAX_INFERENCE_PASSES
    };
    for _ in 0..max_passes {
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
    let function_captures = collect_nested_function_captures(stmts, &env, &states);
    let function_mutated_captures = states
        .iter()
        .map(|(name, state)| {
            let candidate_names = function_captures
                .get(name)
                .into_iter()
                .flatten()
                .map(|(capture, ty)| (capture.clone(), MutationCandidate::Exact(ty.clone())))
                .collect::<HashMap<_, _>>();
            let mut mutated = collect_mutated_binding_names(&state.func.body, &candidate_names)
                .into_iter()
                .collect::<Vec<_>>();
            mutated.sort();
            (name.clone(), mutated)
        })
        .collect();

    NestedFunctionInference {
        function_types: finalize_nested_function_types(&mut states, ctx),
        binding_hints: env.vars,
        function_captures,
        function_mutated_captures,
    }
}

fn collect_function_states<'a>(
    stmts: &'a [Stmt],
    ctx: &mut LowerCtx,
    allow_union_return_inference: bool,
) -> HashMap<String, LocalFunctionState<'a>> {
    let mut states = HashMap::new();

    for stmt in stmts {
        let Stmt::FunctionDef(func) = stmt else {
            continue;
        };

        let mut params = Vec::new();
        let resolved_params = func
            .parameters
            .args
            .iter()
            .map(|param| {
                let name = param.parameter.name.to_string();
                let (ty, explicit) = if let Some(annotation) = &param.parameter.annotation {
                    (resolve_annotation_expr(annotation, ctx), true)
                } else {
                    (Type::Unknown, false)
                };
                (name, ty, explicit)
            })
            .collect::<Vec<_>>();
        let param_types = resolved_params
            .iter()
            .map(|(name, ty, _)| {
                let candidate = if ty.is_unknown() {
                    MutationCandidate::InferFromUsage
                } else {
                    MutationCandidate::Exact(ty.clone())
                };
                (name.clone(), candidate)
            })
            .collect::<HashMap<_, _>>();
        let mutated_params = collect_mutated_binding_names(&func.body, &param_types);
        for (param, (name, ty, explicit)) in func.parameters.args.iter().zip(resolved_params) {
            params.push(ParamState {
                name,
                name_range: param.parameter.name.range(),
                ty,
                explicit,
                convention: param.parameter.convention,
                mutated: mutated_params.contains(param.parameter.name.as_str()),
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
                allow_union_return_inference,
            },
        );
    }

    states
}

pub(super) fn snapshot_signatures(
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

pub(super) fn finalize_nested_function_types(
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    ctx: &mut LowerCtx,
) -> HashMap<String, FunctionType> {
    let mut result = HashMap::new();

    for state in states.values_mut() {
        for param in &mut state.params {
            if !param.explicit && param.ty.is_unknown() {
                state.inference_failed = true;
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISSING_ANNOTATION,
                    format!(
                        "parameter '{}' in function '{}' is missing a type annotation and could not be inferred",
                        param.name, state.func.name
                    ),
                    param.name_range,
                );
                param.ty = Type::Any;
            }
        }

        if !state.explicit_return {
            if state.return_type.has_conflicting_class_specializations() {
                state.return_type = Type::Unknown;
                state.inference_failed = true;
            }
            state.return_type = finalize_return_type(state);
            if state.return_type.is_unknown() {
                state.inference_failed = true;
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISSING_ANNOTATION,
                    format!(
                        "function '{}' return type could not be inferred deterministically",
                        state.func.name
                    ),
                    state.func.name.range(),
                );
                state.return_type = Type::Any;
            }
        }

        result.insert(state.func.name.to_string(), state.function_type());
    }

    result
}

pub(super) fn finalize_return_type(state: &LocalFunctionState<'_>) -> Type {
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

pub(super) fn analyze_block(
    stmts: &[Stmt],
    env: &mut FunctionEnv,
    states: &mut HashMap<String, LocalFunctionState<'_>>,
    current_function: Option<&str>,
    ctx: &LowerCtx,
) {
    for stmt in stmts {
        analyze_stmt(stmt, env, states, current_function, ctx);
        if inference_stmt_always_exits(stmt) {
            break;
        }
    }
}

pub(super) fn collect_current_function_local_bindings(
    stmts: &[Stmt],
    bindings: &mut HashSet<String>,
) {
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
            _ if collect_compound_local_bindings(stmt, bindings) => {}
            _ => {}
        }
    }
}

pub(super) fn collect_assignment_target_names(targets: &[Expr], bindings: &mut HashSet<String>) {
    for target in targets {
        match target {
            Expr::Name(name) => {
                bindings.insert(name.id.to_string());
            }
            Expr::Tuple(tuple) => {
                collect_assignment_target_names(&tuple.elts, bindings);
            }
            _ => {}
        }
    }
}

pub(super) fn collect_nonlocal_names(stmts: &[Stmt], names: &mut HashSet<String>) {
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
            _ if collect_compound_nonlocals(stmt, names) => {}
            _ => {}
        }
    }
}

pub(super) fn analyze_stmt(
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
            match for_stmt.target.as_ref() {
                Expr::Name(name) => {
                    body_env.bind_var(name.id.as_str(), elem_ty);
                }
                Expr::Tuple(tuple) => {
                    let tuple_member_tys = match &elem_ty {
                        Type::Tuple(member_tys) => member_tys.clone(),
                        _ => vec![Type::Unknown; tuple.elts.len()],
                    };
                    for (index, elt) in tuple.elts.iter().enumerate() {
                        let Expr::Name(name) = elt else {
                            continue;
                        };
                        let binding_ty = tuple_member_tys
                            .get(index)
                            .cloned()
                            .unwrap_or(Type::Unknown);
                        body_env.bind_var(name.id.as_str(), binding_ty);
                    }
                }
                _ => {}
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
        Stmt::Match(match_stmt) => {
            analyze_match_stmt(match_stmt, env, states, current_function, ctx);
        }
        Stmt::Try(try_stmt) => {
            analyze_try_stmt(try_stmt, env, states, current_function, ctx);
        }
        Stmt::With(with_stmt) => {
            analyze_with_stmt(with_stmt, env, states, current_function, ctx);
        }
        Stmt::FunctionDef(func) => {
            let Some(state) = states.get(func.name.as_str()).cloned() else {
                return;
            };
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
