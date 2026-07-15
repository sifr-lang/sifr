use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

pub(super) fn class_aliases_for_import(
    module: &str,
    module_classes: Option<&HashMap<String, Type>>,
    names: &[String],
    aliases: &[(String, String)],
) -> HashMap<String, String> {
    if module.starts_with("sifr.") || module.starts_with("_sifr.") {
        return HashMap::new();
    }
    let Some(classes) = module_classes else {
        return HashMap::new();
    };
    names
        .iter()
        .filter(|name| classes.contains_key(*name))
        .map(|name| {
            let local = aliases
                .iter()
                .find(|(source, _)| source == name)
                .map_or_else(|| name.clone(), |(_, local)| local.clone());
            (name.clone(), local)
        })
        .collect()
}

pub(super) fn type_for_import(
    ty: &Type,
    module: &str,
    class_aliases: &HashMap<String, String>,
) -> Type {
    if module.starts_with("sifr.") || module.starts_with("_sifr.") {
        ty.clone()
    } else {
        rename_class_identities(ty, class_aliases)
    }
}

pub(super) fn function_type_for_import(
    function: &FunctionType,
    module: &str,
    class_aliases: &HashMap<String, String>,
) -> FunctionType {
    if module.starts_with("sifr.") || module.starts_with("_sifr.") {
        function.clone()
    } else {
        rename_function_type(function, class_aliases)
    }
}

fn rename_function_type(
    function: &FunctionType,
    class_aliases: &HashMap<String, String>,
) -> FunctionType {
    FunctionType {
        params: function
            .params
            .iter()
            .map(|(name, ty, convention)| {
                (
                    name.clone(),
                    rename_class_identities(ty, class_aliases),
                    *convention,
                )
            })
            .collect(),
        return_type: Box::new(rename_class_identities(
            &function.return_type,
            class_aliases,
        )),
    }
}

fn rename_class_identities(ty: &Type, class_aliases: &HashMap<String, String>) -> Type {
    match ty {
        Type::List(inner) => Type::List(Box::new(rename_class_identities(inner, class_aliases))),
        Type::Set(inner) => Type::Set(Box::new(rename_class_identities(inner, class_aliases))),
        Type::Iterable(inner) => {
            Type::Iterable(Box::new(rename_class_identities(inner, class_aliases)))
        }
        Type::Iterator(inner) => {
            Type::Iterator(Box::new(rename_class_identities(inner, class_aliases)))
        }
        Type::PythonBuffer(inner) => {
            Type::PythonBuffer(Box::new(rename_class_identities(inner, class_aliases)))
        }
        Type::Dict(key, value) => Type::Dict(
            Box::new(rename_class_identities(key, class_aliases)),
            Box::new(rename_class_identities(value, class_aliases)),
        ),
        Type::Tuple(items) => Type::Tuple(
            items
                .iter()
                .map(|item| rename_class_identities(item, class_aliases))
                .collect(),
        ),
        Type::Union(items) => Type::Union(
            items
                .iter()
                .map(|item| rename_class_identities(item, class_aliases))
                .collect(),
        ),
        Type::Intersection(items) => Type::Intersection(
            items
                .iter()
                .map(|item| rename_class_identities(item, class_aliases))
                .collect(),
        ),
        Type::Callable(params, conventions, result) => Type::Callable(
            params
                .iter()
                .map(|param| rename_class_identities(param, class_aliases))
                .collect(),
            conventions.clone(),
            Box::new(rename_class_identities(result, class_aliases)),
        ),
        Type::AsyncCallable(params, conventions, result) => Type::AsyncCallable(
            params
                .iter()
                .map(|param| rename_class_identities(param, class_aliases))
                .collect(),
            conventions.clone(),
            Box::new(rename_class_identities(result, class_aliases)),
        ),
        Type::Result(ok, err) => Type::Result(
            Box::new(rename_class_identities(ok, class_aliases)),
            Box::new(rename_class_identities(err, class_aliases)),
        ),
        Type::Coroutine(ok, err) => Type::Coroutine(
            Box::new(rename_class_identities(ok, class_aliases)),
            Box::new(rename_class_identities(err, class_aliases)),
        ),
        Type::Task(ok, err) => Type::Task(
            Box::new(rename_class_identities(ok, class_aliases)),
            Box::new(rename_class_identities(err, class_aliases)),
        ),
        Type::TaskResult(ok, err) => Type::TaskResult(
            Box::new(rename_class_identities(ok, class_aliases)),
            Box::new(rename_class_identities(err, class_aliases)),
        ),
        Type::BlockingTask(ok, err) => Type::BlockingTask(
            Box::new(rename_class_identities(ok, class_aliases)),
            Box::new(rename_class_identities(err, class_aliases)),
        ),
        Type::JoinSet(ok, err) => Type::JoinSet(
            Box::new(rename_class_identities(ok, class_aliases)),
            Box::new(rename_class_identities(err, class_aliases)),
        ),
        Type::Failure(err) => Type::Failure(Box::new(rename_class_identities(err, class_aliases))),
        Type::Select2(first, second) => Type::Select2(
            Box::new(rename_class_identities(first, class_aliases)),
            Box::new(rename_class_identities(second, class_aliases)),
        ),
        Type::TimeoutResult(err) => {
            Type::TimeoutResult(Box::new(rename_class_identities(err, class_aliases)))
        }
        Type::Awaitable(result) => {
            Type::Awaitable(Box::new(rename_class_identities(result, class_aliases)))
        }
        Type::AsyncIterator(item, err) => Type::AsyncIterator(
            Box::new(rename_class_identities(item, class_aliases)),
            Box::new(rename_class_identities(err, class_aliases)),
        ),
        Type::AsyncGenerator(item, err) => Type::AsyncGenerator(
            Box::new(rename_class_identities(item, class_aliases)),
            Box::new(rename_class_identities(err, class_aliases)),
        ),
        Type::Alias {
            name,
            type_args,
            body,
        } => Type::Alias {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(|arg| rename_class_identities(arg, class_aliases))
                .collect(),
            body: Box::new(rename_class_identities(body, class_aliases)),
        },
        Type::Function(function) => Type::Function(rename_function_type(function, class_aliases)),
        Type::AsyncFunction(function) => {
            Type::AsyncFunction(rename_function_type(function, class_aliases))
        }
        Type::Newtype { name, inner } => Type::Newtype {
            name: name.clone(),
            inner: Box::new(rename_class_identities(inner, class_aliases)),
        },
        Type::Class {
            name,
            fields,
            methods,
            parent_class,
        } => Type::Class {
            name: class_aliases
                .get(name)
                .cloned()
                .unwrap_or_else(|| name.clone()),
            fields: fields
                .iter()
                .map(|(name, ty)| (name.clone(), rename_class_identities(ty, class_aliases)))
                .collect(),
            methods: methods
                .iter()
                .map(|(name, function)| {
                    (name.clone(), rename_function_type(function, class_aliases))
                })
                .collect(),
            parent_class: parent_class.as_ref().map(|chain| {
                chain
                    .split('|')
                    .map(|parent| class_aliases.get(parent).map_or(parent, String::as_str))
                    .collect::<Vec<_>>()
                    .join("|")
            }),
        },
        _ => ty.clone(),
    }
}
