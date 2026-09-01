use crate::{HirExpr, HirFunction, HirModule, HirStmt, RustEmitter, Type};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum FunctionTypeParamBound {
    Trait(String),
    OutputSelf(&'static str),
}

pub(crate) type FunctionTypeParamBounds =
    HashMap<String, HashMap<String, HashSet<FunctionTypeParamBound>>>;

impl FunctionTypeParamBound {
    pub(crate) fn render_for(&self, type_param: &str) -> String {
        match self {
            Self::Trait(bound) => bound.clone(),
            Self::OutputSelf(trait_path) => {
                format!("{trait_path}<Output = {type_param}>")
            }
        }
    }

    fn requires_addable_support(&self) -> bool {
        matches!(self, Self::Trait(bound) if bound == "__SifrAdd")
    }
}

pub(crate) fn module_requires_addable_support(
    module: &HirModule,
    function_bounds: &FunctionTypeParamBounds,
) -> bool {
    let module_function_names = module
        .functions
        .iter()
        .map(|function| function.name.as_str())
        .collect::<HashSet<_>>();
    function_bounds
        .values()
        .flat_map(HashMap::values)
        .flatten()
        .any(FunctionTypeParamBound::requires_addable_support)
        || module.classes.iter().any(|class| {
            class
                .methods
                .iter()
                .chain(class.operator_impls.iter().map(|(_, method)| method))
                .any(|method| {
                    class
                        .type_params
                        .iter()
                        .chain(&method.type_params)
                        .any(|type_param| {
                            collect_lexical_generic_effects(method, &module_function_names)
                                .reachable_functions
                                .iter()
                                .flat_map(|reachable| {
                                    direct_type_param_bounds(type_param, &reachable.body)
                                })
                                .any(|bound| bound.requires_addable_support())
                        })
                })
        })
}

pub(crate) fn direct_type_param_bounds(
    type_param: &str,
    body: &[HirStmt],
) -> Vec<FunctionTypeParamBound> {
    let requirements =
        crate::hir_analysis::queries::collect_typevar_operator_requirements(body, type_param);
    let mut bounds = Vec::new();
    if requirements.needs_add {
        bounds.push(FunctionTypeParamBound::Trait("__SifrAdd".to_string()));
    }
    if requirements.needs_sub {
        bounds.push(FunctionTypeParamBound::OutputSelf("std::ops::Sub"));
    }
    if requirements.needs_mul {
        bounds.push(FunctionTypeParamBound::OutputSelf("std::ops::Mul"));
    }
    if requirements.needs_div {
        bounds.push(FunctionTypeParamBound::OutputSelf("std::ops::Div"));
    }
    if requirements.needs_rem {
        bounds.push(FunctionTypeParamBound::OutputSelf("std::ops::Rem"));
    }
    if requirements.needs_neg {
        bounds.push(FunctionTypeParamBound::OutputSelf("std::ops::Neg"));
    }
    if requirements.needs_partial_eq {
        bounds.push(FunctionTypeParamBound::Trait("PartialEq".to_string()));
    }
    if requirements.needs_partial_ord {
        bounds.push(FunctionTypeParamBound::Trait("PartialOrd".to_string()));
    }
    if requirements.needs_display {
        bounds.push(FunctionTypeParamBound::Trait(
            "std::fmt::Display".to_string(),
        ));
    }
    bounds
}

#[derive(Debug, Clone)]
pub(crate) struct GenericCallSite {
    pub(crate) function: String,
    explicit_type_args: Vec<Type>,
    argument_types: Vec<Type>,
    result_type: Type,
}

struct LexicalScope {
    nested_functions: HashMap<String, (String, HirFunction)>,
    shadowed_names: HashSet<String>,
}

struct LexicalGenericEffects {
    reachable_functions: Vec<HirFunction>,
    module_calls: Vec<GenericCallSite>,
}

impl RustEmitter {
    /// Compute the least fixed point of Rust trait requirements for module-level
    /// generic functions. Source-declared protocol bounds are authoritative,
    /// direct operations add their concrete Rust traits, and forwarding calls
    /// propagate the callee requirements onto the corresponding caller type
    /// parameters.
    pub(crate) fn closed_function_type_param_bounds(
        module: &HirModule,
    ) -> HashMap<String, HashMap<String, HashSet<FunctionTypeParamBound>>> {
        let functions = module
            .functions
            .iter()
            .map(|function| (function.name.as_str(), function))
            .collect::<HashMap<_, _>>();
        let module_function_names = functions.keys().copied().collect::<HashSet<_>>();
        let mut requirements = module
            .functions
            .iter()
            .filter(|function| !function.type_params.is_empty())
            .map(|function| {
                let by_param = function
                    .type_params
                    .iter()
                    .map(|type_param| {
                        let effects =
                            collect_lexical_generic_effects(function, &module_function_names);
                        let mut bounds = effects
                            .reachable_functions
                            .iter()
                            .flat_map(|reachable| {
                                direct_type_param_bounds(type_param, &reachable.body)
                            })
                            .collect::<HashSet<_>>();
                        if function_type_param_needs_hash_eq(function, type_param) {
                            bounds.insert(FunctionTypeParamBound::Trait(
                                "std::hash::Hash".to_string(),
                            ));
                            bounds.insert(FunctionTypeParamBound::Trait("Eq".to_string()));
                        }
                        if let Some(declared) = module
                            .type_param_bounds
                            .get(&function.name)
                            .and_then(|by_param| by_param.get(type_param))
                        {
                            for specification in declared {
                                bounds.extend(rust_bounds_for_typevar_spec(specification));
                            }
                        }
                        (type_param.clone(), bounds)
                    })
                    .collect::<HashMap<_, _>>();
                (function.name.clone(), by_param)
            })
            .collect::<HashMap<_, _>>();

        loop {
            let mut changed = false;
            for caller in module
                .functions
                .iter()
                .filter(|function| !function.type_params.is_empty())
            {
                for call in
                    collect_reachable_module_generic_call_sites(caller, &module_function_names)
                {
                    let Some(callee) = functions.get(call.function.as_str()) else {
                        continue;
                    };
                    let Some(callee_requirements) = requirements.get(&callee.name).cloned() else {
                        continue;
                    };
                    for (callee_type_param, bounds) in callee_requirements {
                        let forwarded = forwarded_type_params(
                            &caller.type_params,
                            callee,
                            &call,
                            &callee_type_param,
                        );
                        for caller_type_param in forwarded {
                            let target = requirements
                                .entry(caller.name.clone())
                                .or_default()
                                .entry(caller_type_param)
                                .or_default();
                            let before = target.len();
                            target.extend(bounds.iter().cloned());
                            changed |= target.len() != before;
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }

        requirements
    }
}

fn rust_bounds_for_typevar_spec(specification: &str) -> Vec<FunctionTypeParamBound> {
    match specification {
        "Comparable" => vec![FunctionTypeParamBound::Trait("PartialOrd".to_string())],
        "Addable" => vec![FunctionTypeParamBound::Trait("__SifrAdd".to_string())],
        "Hashable" => vec![
            FunctionTypeParamBound::Trait("std::hash::Hash".to_string()),
            FunctionTypeParamBound::Trait("Eq".to_string()),
        ],
        _ => Vec::new(),
    }
}

fn function_type_param_needs_hash_eq(function: &HirFunction, type_param: &str) -> bool {
    function
        .params
        .iter()
        .any(|param| type_contains_hash_key_param(&param.ty, type_param))
        || type_contains_hash_key_param(&function.return_type, type_param)
}

fn type_contains_hash_key_param(ty: &Type, type_param: &str) -> bool {
    match ty.resolve_alias() {
        Type::Dict(key, value) => {
            RustEmitter::type_mentions_type_param(key, type_param)
                || type_contains_hash_key_param(value, type_param)
        }
        Type::Set(value) => RustEmitter::type_mentions_type_param(value, type_param),
        Type::List(value)
        | Type::Iterable(value)
        | Type::Iterator(value)
        | Type::Newtype { inner: value, .. } => type_contains_hash_key_param(value, type_param),
        Type::Tuple(values) | Type::Union(values) | Type::Intersection(values) => values
            .iter()
            .any(|value| type_contains_hash_key_param(value, type_param)),
        _ => false,
    }
}

pub(crate) fn collect_reachable_module_generic_call_sites(
    function: &HirFunction,
    module_function_names: &HashSet<&str>,
) -> Vec<GenericCallSite> {
    collect_lexical_generic_effects(function, module_function_names).module_calls
}

pub(crate) fn reachable_direct_type_param_bounds(
    function: &HirFunction,
    type_param: &str,
    module_function_names: &HashSet<&str>,
) -> HashSet<FunctionTypeParamBound> {
    collect_lexical_generic_effects(function, module_function_names)
        .reachable_functions
        .iter()
        .flat_map(|reachable| direct_type_param_bounds(type_param, &reachable.body))
        .collect()
}

fn collect_lexical_generic_effects(
    function: &HirFunction,
    module_function_names: &HashSet<&str>,
) -> LexicalGenericEffects {
    let mut effects = LexicalGenericEffects {
        reachable_functions: Vec::new(),
        module_calls: Vec::new(),
    };
    let mut scopes = Vec::new();
    let mut visited = HashSet::new();
    collect_lexical_scope_effects(
        function,
        "root",
        module_function_names,
        &mut scopes,
        &mut visited,
        &mut effects,
    );
    effects
}

fn collect_lexical_scope_effects(
    function: &HirFunction,
    lexical_identity: &str,
    module_function_names: &HashSet<&str>,
    scopes: &mut Vec<LexicalScope>,
    visited: &mut HashSet<String>,
    effects: &mut LexicalGenericEffects,
) {
    if !visited.insert(lexical_identity.to_string()) {
        return;
    }
    effects.reachable_functions.push(function.clone());

    let mut nested_functions = HashMap::new();
    let mut on_stmt = |stmt: &HirStmt| {
        if let HirStmt::NestedFunction { func, .. } = stmt {
            nested_functions.insert(
                func.name.clone(),
                (format!("{lexical_identity}/{}", func.name), func.clone()),
            );
        }
    };
    let mut on_expr = |_expr: &HirExpr| {};
    crate::hir_analysis::traversal::walk_stmts(
        &function.body,
        crate::hir_analysis::traversal::TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    let mut shadowed_names =
        crate::hir_analysis::queries::collect_locally_defined_vars(&function.body);
    shadowed_names.extend(function.params.iter().map(|param| param.name.clone()));
    scopes.push(LexicalScope {
        nested_functions,
        shadowed_names,
    });

    let mut direct_calls = Vec::new();
    let mut on_stmt = |_stmt: &HirStmt| {};
    let mut on_expr = |expr: &HirExpr| match expr {
        HirExpr::Call { func, args, ty, .. } => direct_calls.push(GenericCallSite {
            function: crate::stmt_support_emitter::canonical_plain_call_name_for_ir(func)
                .to_string(),
            explicit_type_args: Vec::new(),
            argument_types: args.iter().map(|argument| argument.ty().clone()).collect(),
            result_type: ty.clone(),
        }),
        HirExpr::GenericCall {
            func,
            type_args,
            args,
            ty,
            ..
        } => direct_calls.push(GenericCallSite {
            function: crate::stmt_support_emitter::canonical_plain_call_name_for_ir(func)
                .to_string(),
            explicit_type_args: type_args.clone(),
            argument_types: args.iter().map(|argument| argument.ty().clone()).collect(),
            result_type: ty.clone(),
        }),
        _ => {}
    };
    crate::hir_analysis::traversal::walk_stmts(
        &function.body,
        crate::hir_analysis::traversal::TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );

    let mut nested_targets = Vec::new();
    for call in direct_calls {
        match resolve_lexical_call(&call.function, scopes) {
            LexicalCall::Nested(identity, target) => nested_targets.push((identity, target)),
            LexicalCall::Shadowed => {}
            LexicalCall::Module if module_function_names.contains(call.function.as_str()) => {
                effects.module_calls.push(call);
            }
            LexicalCall::Module => {}
        }
    }
    for (identity, target) in nested_targets {
        collect_lexical_scope_effects(
            &target,
            &identity,
            module_function_names,
            scopes,
            visited,
            effects,
        );
    }
    scopes.pop();
}

enum LexicalCall {
    Nested(String, HirFunction),
    Shadowed,
    Module,
}

fn resolve_lexical_call(name: &str, scopes: &[LexicalScope]) -> LexicalCall {
    for scope in scopes.iter().rev() {
        if let Some(function) = scope.nested_functions.get(name) {
            return LexicalCall::Nested(function.0.clone(), function.1.clone());
        }
        if scope.shadowed_names.contains(name) {
            return LexicalCall::Shadowed;
        }
    }
    LexicalCall::Module
}

pub(crate) fn forwarded_type_params(
    caller_type_params: &[String],
    callee: &HirFunction,
    call: &GenericCallSite,
    callee_type_param: &str,
) -> HashSet<String> {
    let mut forwarded = HashSet::new();
    if let Some(index) = callee
        .type_params
        .iter()
        .position(|candidate| candidate == callee_type_param)
        && let Some(actual) = call.explicit_type_args.get(index)
    {
        collect_actual_caller_params(actual, caller_type_params, &mut forwarded);
    }
    for (formal, actual) in callee.params.iter().zip(&call.argument_types) {
        collect_corresponding_type_params(
            &formal.ty,
            actual,
            callee_type_param,
            caller_type_params,
            &mut forwarded,
        );
    }
    collect_corresponding_type_params(
        &callee.return_type,
        &call.result_type,
        callee_type_param,
        caller_type_params,
        &mut forwarded,
    );
    forwarded
}

fn collect_corresponding_type_params(
    formal: &Type,
    actual: &Type,
    callee_type_param: &str,
    caller_type_params: &[String],
    forwarded: &mut HashSet<String>,
) {
    let formal = formal.resolve_alias();
    let actual = actual.resolve_alias();
    if matches!(formal, Type::TypeVar(name) if name == callee_type_param) {
        collect_actual_caller_params(actual, caller_type_params, forwarded);
        return;
    }

    match (formal, actual) {
        (Type::List(left), Type::List(right))
        | (Type::Set(left), Type::Set(right))
        | (Type::Iterable(left), Type::Iterable(right))
        | (Type::Iterator(left), Type::Iterator(right))
        | (Type::PythonBuffer(left), Type::PythonBuffer(right))
        | (Type::PythonDlpackTensor(left), Type::PythonDlpackTensor(right))
        | (Type::Awaitable(left), Type::Awaitable(right))
        | (Type::Failure(left), Type::Failure(right))
        | (Type::TimeoutResult(left), Type::TimeoutResult(right))
        | (Type::Newtype { inner: left, .. }, Type::Newtype { inner: right, .. }) => {
            collect_corresponding_type_params(
                left,
                right,
                callee_type_param,
                caller_type_params,
                forwarded,
            );
        }
        (Type::Dict(lk, lv), Type::Dict(rk, rv))
        | (Type::Result(lk, lv), Type::Result(rk, rv))
        | (Type::Task(lk, lv), Type::Task(rk, rv))
        | (Type::TaskResult(lk, lv), Type::TaskResult(rk, rv))
        | (Type::Coroutine(lk, lv), Type::Coroutine(rk, rv))
        | (Type::Select2(lk, lv), Type::Select2(rk, rv))
        | (Type::BlockingTask(lk, lv), Type::BlockingTask(rk, rv))
        | (Type::JoinSet(lk, lv), Type::JoinSet(rk, rv))
        | (Type::AsyncIterator(lk, lv), Type::AsyncIterator(rk, rv))
        | (Type::AsyncGenerator(lk, lv), Type::AsyncGenerator(rk, rv)) => {
            collect_corresponding_type_params(
                lk,
                rk,
                callee_type_param,
                caller_type_params,
                forwarded,
            );
            collect_corresponding_type_params(
                lv,
                rv,
                callee_type_param,
                caller_type_params,
                forwarded,
            );
        }
        (Type::Tuple(left), Type::Tuple(right))
        | (Type::Union(left), Type::Union(right))
        | (Type::Intersection(left), Type::Intersection(right))
            if left.len() == right.len() =>
        {
            for (formal, actual) in left.iter().zip(right) {
                collect_corresponding_type_params(
                    formal,
                    actual,
                    callee_type_param,
                    caller_type_params,
                    forwarded,
                );
            }
        }
        (
            Type::Class {
                identity: left_identity,
                type_args: left,
                name: left_name,
                ..
            },
            Type::Class {
                identity: right_identity,
                type_args: right,
                name: right_name,
                ..
            },
        ) if left_identity.as_ref().unwrap_or(left_name)
            == right_identity.as_ref().unwrap_or(right_name)
            && left.len() == right.len() =>
        {
            for (formal, actual) in left.iter().zip(right) {
                collect_corresponding_type_params(
                    formal,
                    actual,
                    callee_type_param,
                    caller_type_params,
                    forwarded,
                );
            }
        }
        _ => {}
    }
}

fn collect_actual_caller_params(
    actual: &Type,
    caller_type_params: &[String],
    forwarded: &mut HashSet<String>,
) {
    forwarded.extend(
        caller_type_params
            .iter()
            .filter(|candidate| RustEmitter::type_mentions_type_param(actual, candidate))
            .cloned(),
    );
}

#[cfg(test)]
#[path = "function_generic_bounds_tests.rs"]
mod tests;
