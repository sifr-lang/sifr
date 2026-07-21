use super::{infer_type_var_bindings, LowerCtx};
use sifr_diagnostics::DiagnosticCode;
use sifr_type_system::{type_check_binary_op, type_check_comparison, type_check_unary_op, Type};
use std::collections::{HashMap, HashSet};

fn type_mentions_param(ty: &Type, param: &str) -> bool {
    match ty.resolve_alias() {
        Type::TypeVar(name) => name == param,
        Type::List(inner)
        | Type::Set(inner)
        | Type::Iterable(inner)
        | Type::Iterator(inner)
        | Type::PythonBuffer(inner)
        | Type::PythonDlpackTensor(inner)
        | Type::Awaitable(inner)
        | Type::Failure(inner)
        | Type::TimeoutResult(inner)
        | Type::Newtype { inner, .. } => type_mentions_param(inner, param),
        Type::Dict(left, right)
        | Type::Result(left, right)
        | Type::Task(left, right)
        | Type::TaskResult(left, right)
        | Type::Coroutine(left, right)
        | Type::Select2(left, right)
        | Type::BlockingTask(left, right)
        | Type::JoinSet(left, right)
        | Type::AsyncIterator(left, right)
        | Type::AsyncGenerator(left, right) => {
            type_mentions_param(left, param) || type_mentions_param(right, param)
        }
        Type::Tuple(items) | Type::Union(items) | Type::Intersection(items) => {
            items.iter().any(|item| type_mentions_param(item, param))
        }
        Type::Callable(params, _, result) | Type::AsyncCallable(params, _, result) => {
            params.iter().any(|item| type_mentions_param(item, param))
                || type_mentions_param(result, param)
        }
        Type::Function(function) | Type::AsyncFunction(function) => {
            function
                .params
                .iter()
                .any(|(_, item, _)| type_mentions_param(item, param))
                || type_mentions_param(&function.return_type, param)
        }
        Type::Class {
            fields, methods, ..
        } => {
            fields
                .iter()
                .any(|(_, item)| type_mentions_param(item, param))
                || methods.iter().any(|(_, method)| {
                    method
                        .params
                        .iter()
                        .any(|(_, item, _)| type_mentions_param(item, param))
                        || type_mentions_param(&method.return_type, param)
                })
        }
        _ => false,
    }
}

pub(in crate::lower) fn record_current_method_requirements(
    ctx: &mut LowerCtx,
    operand_types: &[&Type],
    requirements: &[&str],
) {
    let (Some(class), Some(method)) = (ctx.current_class.clone(), ctx.current_method.clone())
    else {
        return;
    };
    let Some(params) = ctx.class_declared_type_params.get(&class).cloned() else {
        return;
    };
    for param in params {
        if !operand_types
            .iter()
            .any(|ty| type_mentions_param(ty, &param))
        {
            continue;
        }
        ctx.generic_method_requirements
            .entry(class.clone())
            .or_default()
            .entry(method.clone())
            .or_default()
            .entry(param)
            .or_default()
            .extend(
                requirements
                    .iter()
                    .map(|requirement| (*requirement).to_string()),
            );
    }
}

pub(in crate::lower) fn record_current_method_dependency(
    ctx: &mut LowerCtx,
    receiver_is_current_class: bool,
    called_method: &str,
) {
    if !receiver_is_current_class {
        return;
    }
    let (Some(class), Some(method)) = (ctx.current_class.clone(), ctx.current_method.clone())
    else {
        return;
    };
    ctx.generic_method_dependencies
        .entry(class)
        .or_default()
        .entry(method)
        .or_default()
        .insert(called_method.to_string());
}

pub(in crate::lower) fn close_generic_method_requirements(ctx: &mut LowerCtx) {
    close_method_dependencies(ctx);
    add_ordering_supertrait_requirements(ctx);
    close_method_dependencies(ctx);
    for (class, methods) in ctx.generic_method_requirements.clone() {
        for (method, by_param) in methods {
            let encoded = by_param
                .into_iter()
                .map(|(param, requirements)| {
                    let mut requirements = requirements.into_iter().collect::<Vec<_>>();
                    requirements.sort();
                    (param, requirements)
                })
                .collect();
            ctx.type_param_bounds
                .insert(format!("{class}.{method}"), encoded);
        }
    }
}

fn close_method_dependencies(ctx: &mut LowerCtx) {
    loop {
        let snapshot = ctx.generic_method_requirements.clone();
        let mut changed = false;
        for (class, methods) in ctx.generic_method_dependencies.clone() {
            for (method, dependencies) in methods {
                for dependency in dependencies {
                    let Some(inherited) = snapshot
                        .get(&class)
                        .and_then(|requirements| requirements.get(&dependency))
                    else {
                        continue;
                    };
                    let target = ctx
                        .generic_method_requirements
                        .entry(class.clone())
                        .or_default()
                        .entry(method.clone())
                        .or_default();
                    for (param, requirements) in inherited {
                        let entry = target.entry(param.clone()).or_default();
                        let before = entry.len();
                        entry.extend(requirements.iter().cloned());
                        changed |= entry.len() != before;
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
}

fn add_ordering_supertrait_requirements(ctx: &mut LowerCtx) {
    let snapshot = ctx.generic_method_requirements.clone();
    for (class, methods) in snapshot {
        if !methods.contains_key("__lt__") {
            continue;
        }
        let equality_requirements = if let Some(requirements) = methods.get("__eq__") {
            requirements.clone()
        } else {
            let Some(Type::Class { fields, .. }) = ctx.class_types.get(&class) else {
                continue;
            };
            ctx.class_declared_type_params
                .get(&class)
                .into_iter()
                .flatten()
                .filter(|param| {
                    fields
                        .iter()
                        .any(|(_, field)| type_mentions_param(field, param))
                })
                .map(|param| (param.clone(), HashSet::from(["PartialEq".to_string()])))
                .collect()
        };
        let ordering = ctx
            .generic_method_requirements
            .entry(class)
            .or_default()
            .entry("__lt__".to_string())
            .or_default();
        for (param, requirements) in equality_requirements {
            ordering.entry(param).or_default().extend(requirements);
        }
    }
}

pub(in crate::lower) fn import_generic_method_requirements(
    ctx: &mut LowerCtx,
    module_bounds: &HashMap<String, HashMap<String, Vec<String>>>,
    source_class: &str,
    local_class: &str,
) {
    let prefix = format!("{source_class}.");
    for (owner, by_param) in module_bounds {
        let Some(method) = owner.strip_prefix(&prefix) else {
            continue;
        };
        let requirements: HashMap<String, HashSet<String>> = by_param
            .iter()
            .map(|(param, requirements)| (param.clone(), requirements.iter().cloned().collect()))
            .collect();
        let mut class_identities = HashSet::from([local_class.to_string()]);
        if let Some(Type::Class { name, .. }) =
            ctx.class_types.get(local_class).map(Type::resolve_alias)
        {
            class_identities.insert(name.clone());
        }
        for class_identity in class_identities {
            ctx.generic_method_requirements
                .entry(class_identity)
                .or_default()
                .insert(method.to_string(), requirements.clone());
        }
    }
}

fn concrete_type_bindings(
    class_name: &str,
    concrete: &Type,
    ctx: &LowerCtx,
) -> HashMap<String, Type> {
    let mut bindings = HashMap::new();
    if let Some(template) = ctx.class_types.get(class_name).or_else(|| {
        ctx.class_types.values().find(|candidate| {
            matches!(candidate.resolve_alias(), Type::Class { name, .. } if name == class_name)
        })
    }) {
        infer_type_var_bindings(template, concrete, &mut bindings);
    }
    bindings
}

fn operator_method_for_requirement(requirement: &str) -> Option<&'static str> {
    match requirement {
        "PartialEq" => Some("__eq__"),
        "PartialOrd" => Some("__lt__"),
        "Neg" => Some("__neg__"),
        _ => None,
    }
}

fn class_operator_supported(
    ty: &Type,
    method: &str,
    ctx: &LowerCtx,
    visiting: &mut HashSet<String>,
) -> bool {
    let Type::Class { name, methods, .. } = ty.resolve_alias() else {
        return false;
    };
    if !methods.iter().any(|(candidate, _)| candidate == method) {
        return false;
    }
    let visit_key = format!("{}::{method}", ty.display_name());
    if !visiting.insert(visit_key.clone()) {
        return true;
    }
    let supported = ctx
        .generic_method_requirements
        .get(name)
        .and_then(|requirements| requirements.get(method))
        .is_none_or(|requirements| {
            let bindings = concrete_type_bindings(name, ty, ctx);
            requirements.iter().all(|(param, required)| {
                bindings.get(param).is_none_or(|concrete| {
                    required.iter().all(|requirement| {
                        requirement_supported_inner(concrete, requirement, ctx, visiting)
                    })
                })
            })
        });
    visiting.remove(&visit_key);
    supported
}

fn requirement_supported_inner(
    ty: &Type,
    requirement: &str,
    ctx: &LowerCtx,
    visiting: &mut HashSet<String>,
) -> bool {
    if let Some(method) = operator_method_for_requirement(requirement) {
        if class_operator_supported(ty, method, ctx, visiting) {
            return true;
        }
    }
    match requirement {
        "Clone" => ty.supports_derived_clone(),
        "PartialEq" => super::type_bounds::supports_structural_equality_in_context(ty, ctx),
        "PartialOrd" => type_check_comparison(ty, "<", ty).is_ok(),
        "Add" => type_check_binary_op(ty, "+", ty).is_ok(),
        "Sub" => type_check_binary_op(ty, "-", ty).is_ok(),
        "Mul" => type_check_binary_op(ty, "*", ty).is_ok(),
        "Div" => type_check_binary_op(ty, "/", ty).is_ok(),
        "Rem" => type_check_binary_op(ty, "%", ty).is_ok(),
        "Neg" => type_check_unary_op("-", ty).is_ok(),
        _ => false,
    }
}

fn requirement_supported(ty: &Type, requirement: &str, ctx: &LowerCtx) -> bool {
    requirement_supported_inner(ty, requirement, ctx, &mut HashSet::new())
}

pub(in crate::lower) fn validate_generic_method_specialization(
    class_name: &str,
    concrete_class: &Type,
    method: &str,
    range: ruff_text_size::TextRange,
    ctx: &mut LowerCtx,
) -> bool {
    let Some(requirements) = ctx
        .generic_method_requirements
        .get(class_name)
        .and_then(|methods| methods.get(method))
        .cloned()
    else {
        return true;
    };
    let bindings = concrete_type_bindings(class_name, concrete_class, ctx);
    let mut failures = Vec::new();
    for (param, required) in requirements {
        let Some(concrete) = bindings.get(&param) else {
            continue;
        };
        if matches!(
            concrete.resolve_alias(),
            Type::TypeVar(_) | Type::Any | Type::Unknown
        ) {
            continue;
        }
        let mut missing = required
            .iter()
            .filter(|requirement| !requirement_supported(concrete, requirement, ctx))
            .cloned()
            .collect::<Vec<_>>();
        missing.sort();
        if !missing.is_empty() {
            failures.push(format!(
                "{param}='{}' lacks {}",
                concrete.display_name(),
                missing.join(" + ")
            ));
        }
    }
    if failures.is_empty() {
        return true;
    }
    failures.sort();
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_MISMATCH,
        format!(
            "{class_name}.{method}() is unavailable for this specialization because {}",
            failures.join(", ")
        ),
        range,
    );
    false
}

pub(in crate::lower) fn requirement_names_for_binary_operator(
    op: &str,
) -> Option<[&'static str; 2]> {
    let operation = match op {
        "+" => "Add",
        "-" => "Sub",
        "*" => "Mul",
        "/" | "//" => "Div",
        "%" => "Rem",
        _ => return None,
    };
    Some(["Clone", operation])
}

pub(in crate::lower) fn requirement_names_for_comparison(ops: &[String]) -> HashSet<&'static str> {
    let mut requirements = HashSet::from(["Clone"]);
    if ops.iter().any(|op| matches!(op.as_str(), "==" | "!=")) {
        requirements.insert("PartialEq");
    }
    if ops
        .iter()
        .any(|op| matches!(op.as_str(), "<" | "<=" | ">" | ">="))
    {
        requirements.insert("PartialOrd");
    }
    requirements
}
