use std::collections::{BTreeMap, HashSet};

use sifr_ir::HirFunction;
use sifr_type_system::{FunctionType, Type, is_crate_root_rust_nominal_identity};

use crate::HirModule;

pub(super) fn collect_module_nominals(
    module: &HirModule,
    declarations: &mut BTreeMap<String, HashSet<String>>,
    builtin_types: &mut BTreeMap<String, Type>,
) {
    for function in &module.functions {
        collect_hir_function_nominals(function, declarations, builtin_types);
    }
    for class in &module.classes {
        for (_, field) in &class.fields {
            collect_shared_nominals(field, declarations, builtin_types);
        }
        if let Some(parent) = &class.parent_type {
            collect_shared_nominals(parent, declarations, builtin_types);
        }
        for method in &class.methods {
            collect_hir_function_nominals(method, declarations, builtin_types);
        }
        for (_, operator) in &class.operator_impls {
            collect_hir_function_nominals(operator, declarations, builtin_types);
        }
    }
    for (_, ty, _) in &module.constants {
        collect_shared_nominals(ty, declarations, builtin_types);
    }
}

pub(super) fn collect_shared_nominals(
    ty: &Type,
    declarations: &mut BTreeMap<String, HashSet<String>>,
    builtin_types: &mut BTreeMap<String, Type>,
) {
    match ty.resolve_alias() {
        class @ Type::Class {
            identity,
            type_args,
            name,
            fields,
            methods,
            ..
        } => {
            if is_compiler_builtin_error(identity.as_deref(), name)
                && sifr_type_system::io_error_kind(name).is_none()
            {
                builtin_types
                    .entry(name.clone())
                    .or_insert_with(|| class.clone());
            }
            if !class.is_python_object_contract() && !class.is_python_resource_identity_contract() {
                collect_nominal_identity(identity.as_deref(), declarations);
            }
            for type_arg in type_args {
                collect_shared_nominals(type_arg, declarations, builtin_types);
            }
            if identity.as_deref().is_some_and(|identity| {
                identity.starts_with("sifr.") || identity.starts_with("_sifr.")
            }) {
                for (_, field) in fields {
                    collect_shared_nominals(field, declarations, builtin_types);
                }
                for (_, method) in methods {
                    collect_function_nominals(method, declarations, builtin_types);
                }
            }
        }
        Type::Protocol {
            identity, methods, ..
        } => {
            collect_nominal_identity(identity.as_deref(), declarations);
            for (_, method) in methods {
                collect_function_nominals(method, declarations, builtin_types);
            }
        }
        Type::Newtype {
            identity, inner, ..
        } => {
            collect_nominal_identity(identity.as_deref(), declarations);
            collect_shared_nominals(inner, declarations, builtin_types);
        }
        Type::Enum { identity, .. } => collect_nominal_identity(identity.as_deref(), declarations),
        Type::List(inner)
        | Type::Set(inner)
        | Type::Iterable(inner)
        | Type::Iterator(inner)
        | Type::Awaitable(inner)
        | Type::Failure(inner)
        | Type::TimeoutResult(inner)
        | Type::PythonBuffer(inner)
        | Type::PythonDlpackTensor(inner) => {
            collect_shared_nominals(inner, declarations, builtin_types);
        }
        Type::Dict(left, right)
        | Type::Result(left, right)
        | Type::Coroutine(left, right)
        | Type::Task(left, right)
        | Type::TaskResult(left, right)
        | Type::Select2(left, right)
        | Type::BlockingTask(left, right)
        | Type::JoinSet(left, right)
        | Type::AsyncIterator(left, right)
        | Type::AsyncGenerator(left, right) => {
            collect_shared_nominals(left, declarations, builtin_types);
            collect_shared_nominals(right, declarations, builtin_types);
        }
        Type::Tuple(items) | Type::Union(items) | Type::Intersection(items) => {
            for item in items {
                collect_shared_nominals(item, declarations, builtin_types);
            }
        }
        Type::Function(function) | Type::AsyncFunction(function) => {
            collect_function_nominals(function, declarations, builtin_types);
        }
        Type::Callable(parameters, _, result) | Type::AsyncCallable(parameters, _, result) => {
            for parameter in parameters {
                collect_shared_nominals(parameter, declarations, builtin_types);
            }
            collect_shared_nominals(result, declarations, builtin_types);
        }
        _ => {}
    }
}

fn collect_function_nominals(
    function: &FunctionType,
    declarations: &mut BTreeMap<String, HashSet<String>>,
    builtin_types: &mut BTreeMap<String, Type>,
) {
    for (_, parameter, _) in &function.params {
        collect_shared_nominals(parameter, declarations, builtin_types);
    }
    collect_shared_nominals(&function.return_type, declarations, builtin_types);
}

fn collect_hir_function_nominals(
    function: &HirFunction,
    declarations: &mut BTreeMap<String, HashSet<String>>,
    builtin_types: &mut BTreeMap<String, Type>,
) {
    for parameter in &function.params {
        collect_shared_nominals(&parameter.ty, declarations, builtin_types);
    }
    collect_shared_nominals(&function.return_type, declarations, builtin_types);
    for ty in crate::hir_analysis::queries::collect_let_declared_types(&function.body) {
        collect_shared_nominals(&ty, declarations, builtin_types);
    }
}

fn collect_nominal_identity(
    identity: Option<&str>,
    declarations: &mut BTreeMap<String, HashSet<String>>,
) {
    let Some(identity) = identity else {
        return;
    };
    if is_crate_root_rust_nominal_identity(identity)
        || (!identity.starts_with("sifr.") && !identity.starts_with("_sifr."))
    {
        return;
    }
    let Some((module, name)) = identity.rsplit_once('.') else {
        return;
    };
    if is_compiler_builtin_error(Some(identity), name) {
        return;
    }
    declarations
        .entry(module.to_string())
        .or_default()
        .insert(name.to_string());
}

pub(super) fn is_compiler_builtin_error(identity: Option<&str>, name: &str) -> bool {
    crate::BUILTIN_ERROR_CLASSES.contains(&name)
        && identity.is_some_and(|identity| {
            identity.starts_with("sifr.builtin.")
                || sifr_type_system::is_global_rust_nominal_identity(identity)
        })
}
