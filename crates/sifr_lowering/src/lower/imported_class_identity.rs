use sifr_type_system::{FunctionType, Type};

pub(super) fn class_type_for_import(class_ty: &Type, module: &str, local: &str) -> Type {
    if module.starts_with("sifr.") || module.starts_with("_sifr.") {
        return class_ty.clone();
    }
    let Type::Class { name, .. } = class_ty.resolve_alias() else {
        return class_ty.clone();
    };
    rename_class_identity(class_ty, name, local)
}

fn rename_function_type(function: &FunctionType, source: &str, local: &str) -> FunctionType {
    FunctionType {
        params: function
            .params
            .iter()
            .map(|(name, ty, convention)| {
                (
                    name.clone(),
                    rename_class_identity(ty, source, local),
                    *convention,
                )
            })
            .collect(),
        return_type: Box::new(rename_class_identity(&function.return_type, source, local)),
    }
}

fn rename_class_identity(ty: &Type, source: &str, local: &str) -> Type {
    match ty {
        Type::List(inner) => Type::List(Box::new(rename_class_identity(inner, source, local))),
        Type::Set(inner) => Type::Set(Box::new(rename_class_identity(inner, source, local))),
        Type::Iterable(inner) => {
            Type::Iterable(Box::new(rename_class_identity(inner, source, local)))
        }
        Type::Iterator(inner) => {
            Type::Iterator(Box::new(rename_class_identity(inner, source, local)))
        }
        Type::PythonBuffer(inner) => {
            Type::PythonBuffer(Box::new(rename_class_identity(inner, source, local)))
        }
        Type::Dict(key, value) => Type::Dict(
            Box::new(rename_class_identity(key, source, local)),
            Box::new(rename_class_identity(value, source, local)),
        ),
        Type::Tuple(items) => Type::Tuple(
            items
                .iter()
                .map(|item| rename_class_identity(item, source, local))
                .collect(),
        ),
        Type::Union(items) => Type::Union(
            items
                .iter()
                .map(|item| rename_class_identity(item, source, local))
                .collect(),
        ),
        Type::Intersection(items) => Type::Intersection(
            items
                .iter()
                .map(|item| rename_class_identity(item, source, local))
                .collect(),
        ),
        Type::Callable(params, conventions, result) => Type::Callable(
            params
                .iter()
                .map(|param| rename_class_identity(param, source, local))
                .collect(),
            conventions.clone(),
            Box::new(rename_class_identity(result, source, local)),
        ),
        Type::AsyncCallable(params, conventions, result) => Type::AsyncCallable(
            params
                .iter()
                .map(|param| rename_class_identity(param, source, local))
                .collect(),
            conventions.clone(),
            Box::new(rename_class_identity(result, source, local)),
        ),
        Type::Result(ok, err) => Type::Result(
            Box::new(rename_class_identity(ok, source, local)),
            Box::new(rename_class_identity(err, source, local)),
        ),
        Type::Coroutine(ok, err) => Type::Coroutine(
            Box::new(rename_class_identity(ok, source, local)),
            Box::new(rename_class_identity(err, source, local)),
        ),
        Type::Task(ok, err) => Type::Task(
            Box::new(rename_class_identity(ok, source, local)),
            Box::new(rename_class_identity(err, source, local)),
        ),
        Type::TaskResult(ok, err) => Type::TaskResult(
            Box::new(rename_class_identity(ok, source, local)),
            Box::new(rename_class_identity(err, source, local)),
        ),
        Type::BlockingTask(ok, err) => Type::BlockingTask(
            Box::new(rename_class_identity(ok, source, local)),
            Box::new(rename_class_identity(err, source, local)),
        ),
        Type::JoinSet(ok, err) => Type::JoinSet(
            Box::new(rename_class_identity(ok, source, local)),
            Box::new(rename_class_identity(err, source, local)),
        ),
        Type::Failure(err) => Type::Failure(Box::new(rename_class_identity(err, source, local))),
        Type::Select2(first, second) => Type::Select2(
            Box::new(rename_class_identity(first, source, local)),
            Box::new(rename_class_identity(second, source, local)),
        ),
        Type::TimeoutResult(err) => {
            Type::TimeoutResult(Box::new(rename_class_identity(err, source, local)))
        }
        Type::Awaitable(result) => {
            Type::Awaitable(Box::new(rename_class_identity(result, source, local)))
        }
        Type::AsyncIterator(item, err) => Type::AsyncIterator(
            Box::new(rename_class_identity(item, source, local)),
            Box::new(rename_class_identity(err, source, local)),
        ),
        Type::AsyncGenerator(item, err) => Type::AsyncGenerator(
            Box::new(rename_class_identity(item, source, local)),
            Box::new(rename_class_identity(err, source, local)),
        ),
        Type::Alias {
            name,
            type_args,
            body,
        } => Type::Alias {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(|arg| rename_class_identity(arg, source, local))
                .collect(),
            body: Box::new(rename_class_identity(body, source, local)),
        },
        Type::Function(function) => Type::Function(rename_function_type(function, source, local)),
        Type::AsyncFunction(function) => {
            Type::AsyncFunction(rename_function_type(function, source, local))
        }
        Type::Newtype { name, inner } => Type::Newtype {
            name: name.clone(),
            inner: Box::new(rename_class_identity(inner, source, local)),
        },
        Type::Class {
            name,
            fields,
            methods,
            parent_class,
        } => Type::Class {
            name: if name == source {
                local.to_string()
            } else {
                name.clone()
            },
            fields: fields
                .iter()
                .map(|(name, ty)| (name.clone(), rename_class_identity(ty, source, local)))
                .collect(),
            methods: methods
                .iter()
                .map(|(name, function)| {
                    (name.clone(), rename_function_type(function, source, local))
                })
                .collect(),
            parent_class: parent_class.clone(),
        },
        _ => ty.clone(),
    }
}
