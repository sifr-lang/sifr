//! Type-variable substitution across nested nominal declaration scopes.

use crate::{make_union, FunctionType, Type};
use std::collections::HashMap;

/// Substitute type variables without declaration-scope metadata.
pub fn substitute_type_vars(ty: &Type, bindings: &HashMap<String, Type>) -> Type {
    substitute_type_vars_with_class_scopes(ty, bindings, &|_, _| Some(Vec::new()))
}

/// Substitute free type variables while rebinding each nested class's declared parameters.
pub fn substitute_type_vars_with_class_scopes<F>(
    ty: &Type,
    bindings: &HashMap<String, Type>,
    class_type_params: &F,
) -> Type
where
    F: Fn(Option<&str>, &str) -> Option<Vec<String>>,
{
    let recurse =
        |ty: &Type| substitute_type_vars_with_class_scopes(ty, bindings, class_type_params);
    let substitute_function = |function: &FunctionType| FunctionType {
        receiver: function.receiver,
        params: function
            .params
            .iter()
            .map(|(name, ty, convention)| (name.clone(), recurse(ty), *convention))
            .collect(),
        return_type: Box::new(recurse(&function.return_type)),
    };

    match ty {
        Type::TypeVar(name) => bindings.get(name).cloned().unwrap_or_else(|| ty.clone()),
        Type::List(value) => Type::List(Box::new(recurse(value))),
        Type::Set(value) => Type::Set(Box::new(recurse(value))),
        Type::Iterable(value) => Type::Iterable(Box::new(recurse(value))),
        Type::Iterator(value) => Type::Iterator(Box::new(recurse(value))),
        Type::PythonBuffer(value) => Type::PythonBuffer(Box::new(recurse(value))),
        Type::PythonDlpackTensor(value) => Type::PythonDlpackTensor(Box::new(recurse(value))),
        Type::Dict(key, value) => Type::Dict(Box::new(recurse(key)), Box::new(recurse(value))),
        Type::Tuple(values) => Type::Tuple(values.iter().map(recurse).collect()),
        Type::Union(values) => make_union(values.iter().map(recurse).collect()),
        Type::Callable(params, conventions, result) => Type::Callable(
            params.iter().map(recurse).collect(),
            conventions.clone(),
            Box::new(recurse(result)),
        ),
        Type::AsyncCallable(params, conventions, result) => Type::AsyncCallable(
            params.iter().map(recurse).collect(),
            conventions.clone(),
            Box::new(recurse(result)),
        ),
        Type::Result(ok, error) => Type::Result(Box::new(recurse(ok)), Box::new(recurse(error))),
        Type::Coroutine(ok, error) => {
            Type::Coroutine(Box::new(recurse(ok)), Box::new(recurse(error)))
        }
        Type::Task(ok, error) => Type::Task(Box::new(recurse(ok)), Box::new(recurse(error))),
        Type::TaskResult(ok, error) => {
            Type::TaskResult(Box::new(recurse(ok)), Box::new(recurse(error)))
        }
        Type::Failure(error) => Type::Failure(Box::new(recurse(error))),
        Type::Select2(first, second) => {
            Type::Select2(Box::new(recurse(first)), Box::new(recurse(second)))
        }
        Type::TimeoutResult(error) => Type::TimeoutResult(Box::new(recurse(error))),
        Type::BlockingTask(ok, error) => {
            Type::BlockingTask(Box::new(recurse(ok)), Box::new(recurse(error)))
        }
        Type::Awaitable(result) => Type::Awaitable(Box::new(recurse(result))),
        Type::AsyncIterator(item, error) => {
            Type::AsyncIterator(Box::new(recurse(item)), Box::new(recurse(error)))
        }
        Type::AsyncGenerator(item, error) => {
            Type::AsyncGenerator(Box::new(recurse(item)), Box::new(recurse(error)))
        }
        Type::JoinSet(ok, error) => Type::JoinSet(Box::new(recurse(ok)), Box::new(recurse(error))),
        Type::Alias {
            name,
            type_args,
            body,
        } => Type::Alias {
            name: name.clone(),
            type_args: type_args.iter().map(recurse).collect(),
            body: Box::new(recurse(body)),
        },
        Type::Function(function) => Type::Function(substitute_function(function)),
        Type::AsyncFunction(function) => Type::AsyncFunction(substitute_function(function)),
        Type::Class {
            identity,
            type_args,
            name,
            fields,
            methods,
            parent_class,
        } => {
            let type_args = type_args.iter().map(recurse).collect::<Vec<_>>();
            let mut nested_bindings = bindings.clone();
            if let Some(declared_params) = class_type_params(identity.as_deref(), name) {
                for parameter in &declared_params {
                    nested_bindings.remove(parameter);
                }
                nested_bindings.extend(declared_params.into_iter().zip(type_args.iter().cloned()));
            } else {
                nested_bindings.clear();
            }
            let nested = |ty: &Type| {
                substitute_type_vars_with_class_scopes(ty, &nested_bindings, class_type_params)
            };
            Type::Class {
                identity: identity.clone(),
                type_args,
                name: name.clone(),
                fields: fields
                    .iter()
                    .map(|(field, ty)| (field.clone(), nested(ty)))
                    .collect(),
                methods: methods
                    .iter()
                    .map(|(method, function)| {
                        (
                            method.clone(),
                            FunctionType {
                                receiver: function.receiver,
                                params: function
                                    .params
                                    .iter()
                                    .map(|(name, ty, convention)| {
                                        (name.clone(), nested(ty), *convention)
                                    })
                                    .collect(),
                                return_type: Box::new(nested(&function.return_type)),
                            },
                        )
                    })
                    .collect(),
                parent_class: parent_class.clone(),
            }
        }
        _ => ty.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::substitute_type_vars_with_class_scopes;
    use crate::Type;
    use std::collections::HashMap;

    #[test]
    fn nested_class_parameters_shadow_same_named_outer_bindings() {
        let nested = Type::Class {
            identity: Some("fixture.Inner".to_string()),
            type_args: vec![Type::Str],
            name: "Inner".to_string(),
            fields: vec![("value".to_string(), Type::TypeVar("T".to_string()))],
            methods: Vec::new(),
            parent_class: None,
        };
        let bindings = HashMap::from([("T".to_string(), Type::Int)]);

        let substituted =
            substitute_type_vars_with_class_scopes(&nested, &bindings, &|identity, name| {
                assert_eq!(identity, Some("fixture.Inner"));
                assert_eq!(name, "Inner");
                Some(vec!["T".to_string()])
            });
        let Type::Class { fields, .. } = substituted else {
            panic!("nested type must stay nominal");
        };
        assert_eq!(fields[0].1, Type::Str);
    }

    #[test]
    fn unresolved_nested_scope_does_not_capture_an_outer_binding() {
        let nested = Type::Class {
            identity: Some("missing.Inner".to_string()),
            type_args: Vec::new(),
            name: "Inner".to_string(),
            fields: vec![("value".to_string(), Type::TypeVar("T".to_string()))],
            methods: Vec::new(),
            parent_class: None,
        };
        let bindings = HashMap::from([("T".to_string(), Type::Int)]);

        let substituted = substitute_type_vars_with_class_scopes(&nested, &bindings, &|_, _| None);
        let Type::Class { fields, .. } = substituted else {
            panic!("nested type must stay nominal");
        };
        assert_eq!(fields[0].1, Type::TypeVar("T".to_string()));
    }
}
