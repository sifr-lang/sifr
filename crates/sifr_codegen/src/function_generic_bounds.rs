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
struct GenericCallSite {
    function: String,
    explicit_type_args: Vec<Type>,
    argument_types: Vec<Type>,
    result_type: Type,
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
        let mut requirements = module
            .functions
            .iter()
            .filter(|function| !function.type_params.is_empty())
            .map(|function| {
                let by_param = function
                    .type_params
                    .iter()
                    .map(|type_param| {
                        let mut bounds = direct_type_param_bounds(type_param, &function.body)
                            .into_iter()
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
                for call in collect_generic_call_sites(&caller.body) {
                    let Some(callee) = functions.get(call.function.as_str()) else {
                        continue;
                    };
                    let Some(callee_requirements) = requirements.get(&callee.name).cloned() else {
                        continue;
                    };
                    for (callee_type_param, bounds) in callee_requirements {
                        let forwarded =
                            forwarded_caller_type_params(caller, callee, &call, &callee_type_param);
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

fn collect_generic_call_sites(body: &[HirStmt]) -> Vec<GenericCallSite> {
    let mut calls = Vec::new();
    let mut on_stmt = |_stmt: &HirStmt| {};
    let mut on_expr = |expr: &HirExpr| match expr {
        HirExpr::Call { func, args, ty, .. } => calls.push(GenericCallSite {
            function: func.clone(),
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
        } => calls.push(GenericCallSite {
            function: func.clone(),
            explicit_type_args: type_args.clone(),
            argument_types: args.iter().map(|argument| argument.ty().clone()).collect(),
            result_type: ty.clone(),
        }),
        _ => {}
    };
    crate::hir_analysis::traversal::walk_stmts(
        body,
        crate::hir_analysis::traversal::TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    calls
}

fn forwarded_caller_type_params(
    caller: &HirFunction,
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
        collect_actual_caller_params(actual, &caller.type_params, &mut forwarded);
    }
    for (formal, actual) in callee.params.iter().zip(&call.argument_types) {
        collect_corresponding_type_params(
            &formal.ty,
            actual,
            callee_type_param,
            &caller.type_params,
            &mut forwarded,
        );
    }
    collect_corresponding_type_params(
        &callee.return_type,
        &call.result_type,
        callee_type_param,
        &caller.type_params,
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
        _ if RustEmitter::type_mentions_type_param(formal, callee_type_param) => {
            collect_actual_caller_params(actual, caller_type_params, forwarded);
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
mod tests {
    use super::*;
    use sifr_ir::{HirParam, MethodKind};
    use sifr_type_system::ParamConvention;

    fn parameter(name: &str, ty: Type) -> HirParam {
        HirParam {
            name: name.to_string(),
            ty,
            default: None,
            keyword_only: false,
            convention: ParamConvention::own(),
        }
    }

    fn function(
        name: &str,
        type_param: &str,
        params: Vec<HirParam>,
        return_type: Type,
        body: Vec<HirStmt>,
    ) -> HirFunction {
        HirFunction {
            name: name.to_string(),
            params,
            return_type,
            body,
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: Vec::new(),
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![type_param.to_string()],
        }
    }

    fn name(name: &str, ty: Type) -> HirExpr {
        HirExpr::Name {
            name: name.to_string(),
            binding_id: None,
            ty,
        }
    }

    fn call(function: &str, args: Vec<HirExpr>, ty: Type) -> HirStmt {
        HirStmt::Expr {
            expr: HirExpr::Call {
                func: function.to_string(),
                args,
                mutable_arg_places: Vec::new(),
                ty,
            },
        }
    }

    fn module(
        functions: Vec<HirFunction>,
        type_param_bounds: HashMap<String, HashMap<String, Vec<String>>>,
    ) -> HirModule {
        HirModule {
            functions,
            classes: Vec::new(),
            imports: Vec::new(),
            constants: Vec::new(),
            generic_functions: HashMap::new(),
            type_param_bounds,
        }
    }

    #[test]
    fn source_protocol_bounds_survive_without_local_operator_use() {
        let passthrough = function(
            "passthrough",
            "T",
            vec![parameter("value", Type::TypeVar("T".to_string()))],
            Type::TypeVar("T".to_string()),
            vec![HirStmt::Return {
                value: Some(name("value", Type::TypeVar("T".to_string()))),
            }],
        );
        let bounds = HashMap::from([(
            "passthrough".to_string(),
            HashMap::from([("T".to_string(), vec!["Comparable".to_string()])]),
        )]);

        let closed =
            RustEmitter::closed_function_type_param_bounds(&module(vec![passthrough], bounds));

        assert!(
            closed["passthrough"]["T"]
                .contains(&FunctionTypeParamBound::Trait("PartialOrd".to_string()))
        );
    }

    #[test]
    fn generic_call_graph_closes_operator_display_and_hash_requirements() {
        let compared = Type::TypeVar("Compared".to_string());
        let compare = function(
            "compare",
            "Compared",
            vec![
                parameter("left", compared.clone()),
                parameter("right", compared.clone()),
            ],
            Type::Bool,
            vec![HirStmt::Return {
                value: Some(HirExpr::Compare {
                    left: Box::new(name("left", compared.clone())),
                    ops: vec!["<".to_string()],
                    comparators: vec![name("right", compared.clone())],
                    ty: Type::Bool,
                }),
            }],
        );
        let displayed = Type::TypeVar("Displayed".to_string());
        let display = function(
            "display",
            "Displayed",
            vec![parameter("value", displayed.clone())],
            Type::None,
            vec![call("print", vec![name("value", displayed)], Type::None)],
        );
        let keyed = Type::TypeVar("Key".to_string());
        let key_map = Type::Dict(Box::new(keyed), Box::new(Type::Int));
        let hash = function(
            "hash",
            "Key",
            vec![parameter("values", key_map)],
            Type::None,
            vec![HirStmt::Pass],
        );
        let forwarded = Type::TypeVar("Forwarded".to_string());
        let forwarded_map = Type::Dict(Box::new(forwarded.clone()), Box::new(Type::Int));
        let forward = function(
            "forward",
            "Forwarded",
            vec![
                parameter("value", forwarded.clone()),
                parameter("values", forwarded_map.clone()),
            ],
            Type::None,
            vec![
                call(
                    "compare",
                    vec![
                        name("value", forwarded.clone()),
                        name("value", forwarded.clone()),
                    ],
                    Type::Bool,
                ),
                call(
                    "display",
                    vec![name("value", forwarded.clone())],
                    Type::None,
                ),
                call("hash", vec![name("values", forwarded_map)], Type::None),
            ],
        );
        let outer = Type::TypeVar("Outer".to_string());
        let outer_map = Type::Dict(Box::new(outer.clone()), Box::new(Type::Int));
        let outer_forward = function(
            "outer_forward",
            "Outer",
            vec![
                parameter("value", outer.clone()),
                parameter("values", outer_map.clone()),
            ],
            Type::None,
            vec![call(
                "forward",
                vec![name("value", outer), name("values", outer_map)],
                Type::None,
            )],
        );

        let closed = RustEmitter::closed_function_type_param_bounds(&module(
            vec![compare, display, hash, forward, outer_forward],
            HashMap::new(),
        ));
        let outer_bounds = closed["outer_forward"]["Outer"]
            .iter()
            .map(|bound| bound.render_for("Outer"))
            .collect::<HashSet<_>>();

        for expected in ["PartialOrd", "std::fmt::Display", "std::hash::Hash", "Eq"] {
            assert!(outer_bounds.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn generic_call_graph_renders_output_traits_for_the_receiving_parameter() {
        let callee_type = Type::TypeVar("T".to_string());
        let add_same = function(
            "add_same",
            "T",
            vec![
                parameter("left", callee_type.clone()),
                parameter("right", callee_type.clone()),
            ],
            callee_type.clone(),
            vec![HirStmt::Return {
                value: Some(HirExpr::BinOp {
                    left: Box::new(name("left", callee_type.clone())),
                    op: "+".to_string(),
                    right: Box::new(name("right", callee_type.clone())),
                    ty: callee_type,
                }),
            }],
        );
        let caller_type = Type::TypeVar("U".to_string());
        let relay_add = function(
            "relay_add",
            "U",
            vec![
                parameter("left", caller_type.clone()),
                parameter("right", caller_type.clone()),
            ],
            caller_type.clone(),
            vec![HirStmt::Return {
                value: Some(HirExpr::Call {
                    func: "add_same".to_string(),
                    args: vec![
                        name("left", caller_type.clone()),
                        name("right", caller_type.clone()),
                    ],
                    mutable_arg_places: Vec::new(),
                    ty: caller_type,
                }),
            }],
        );
        let bounds = HashMap::from([
            (
                "add_same".to_string(),
                HashMap::from([("T".to_string(), vec!["Addable".to_string()])]),
            ),
            (
                "relay_add".to_string(),
                HashMap::from([("U".to_string(), vec!["Addable".to_string()])]),
            ),
        ]);

        let closed = RustEmitter::closed_function_type_param_bounds(&module(
            vec![add_same, relay_add],
            bounds,
        ));
        let rendered = closed["relay_add"]["U"]
            .iter()
            .map(|bound| bound.render_for("U"))
            .collect::<HashSet<_>>();

        assert!(rendered.contains("__SifrAdd"));
        assert!(rendered.iter().all(|bound| !bound.contains("Output = T")));
    }
}
