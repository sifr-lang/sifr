//! Type-variable substitution across nested nominal declaration scopes.

use crate::{FunctionType, Type, make_union};
use std::collections::{HashMap, HashSet};

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
        Type::Template(values) => Type::Template(values.iter().map(recurse).collect()),
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

/// Return whether substitution keeps every union's immediate member topology.
///
/// Rust generic storage retains the declaration's union nesting. A substituted
/// union member cannot expand into another union or collapse into a sibling.
pub fn substitution_preserves_union_structure(ty: &Type, bindings: &HashMap<String, Type>) -> bool {
    substitution_preserves_union_structure_with_class_scopes(ty, bindings, &|_, _| None)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnionStructureClassScope {
    pub type_params: Vec<String>,
    pub member_types: Vec<Type>,
}

/// Check union topology and rebind nested class declarations through one resolver.
pub fn substitution_preserves_union_structure_with_class_scopes<F>(
    ty: &Type,
    bindings: &HashMap<String, Type>,
    class_scope: &F,
) -> bool
where
    F: Fn(Option<&str>, &str) -> Option<UnionStructureClassScope>,
{
    preserves_union_structure(ty, bindings, class_scope, &mut HashSet::new())
}

fn function_preserves_union_structure<F>(
    function: &FunctionType,
    bindings: &HashMap<String, Type>,
    class_scope: &F,
    visiting: &mut HashSet<String>,
) -> bool
where
    F: Fn(Option<&str>, &str) -> Option<UnionStructureClassScope>,
{
    function
        .params
        .iter()
        .all(|(_, ty, _)| preserves_union_structure(ty, bindings, class_scope, visiting))
        && preserves_union_structure(&function.return_type, bindings, class_scope, visiting)
}

fn preserves_union_structure<F>(
    ty: &Type,
    bindings: &HashMap<String, Type>,
    class_scope: &F,
    visiting: &mut HashSet<String>,
) -> bool
where
    F: Fn(Option<&str>, &str) -> Option<UnionStructureClassScope>,
{
    let mut recurse = |ty: &Type| preserves_union_structure(ty, bindings, class_scope, visiting);

    match ty {
        Type::List(value)
        | Type::Set(value)
        | Type::Iterable(value)
        | Type::Iterator(value)
        | Type::PythonBuffer(value)
        | Type::PythonDlpackTensor(value)
        | Type::Failure(value)
        | Type::Awaitable(value) => recurse(value),
        Type::Dict(key, value)
        | Type::Result(key, value)
        | Type::Coroutine(key, value)
        | Type::Task(key, value)
        | Type::TaskResult(key, value)
        | Type::Select2(key, value)
        | Type::BlockingTask(key, value)
        | Type::AsyncIterator(key, value)
        | Type::AsyncGenerator(key, value)
        | Type::JoinSet(key, value) => recurse(key) && recurse(value),
        Type::TimeoutResult(value) => recurse(value),
        Type::Tuple(values) | Type::Template(values) => values.iter().all(&mut recurse),
        Type::Union(values) => {
            let substituted = values
                .iter()
                .map(|value| substitute_type_vars(value, bindings))
                .collect::<Vec<_>>();
            !substituted
                .iter()
                .any(|value| matches!(value.resolve_alias(), Type::Union(_)))
                && matches!(make_union(substituted), Type::Union(canonical) if canonical.len() == values.len())
                && values.iter().all(&mut recurse)
        }
        Type::Callable(params, _, result) | Type::AsyncCallable(params, _, result) => {
            params.iter().all(&mut recurse) && recurse(result)
        }
        Type::Alias {
            type_args, body, ..
        } => type_args.iter().all(&mut recurse) && recurse(body),
        Type::Function(function) | Type::AsyncFunction(function) => {
            function_preserves_union_structure(function, bindings, class_scope, visiting)
        }
        Type::Class {
            identity,
            type_args,
            name,
            ..
        } => {
            if !type_args.iter().all(&mut recurse) {
                return false;
            }
            let Some(scope) = class_scope(identity.as_deref(), name) else {
                return true;
            };
            let key = identity.as_deref().unwrap_or(name).to_string();
            if !visiting.insert(key.clone()) {
                return true;
            }
            let concrete_args = type_args
                .iter()
                .map(|argument| substitute_type_vars(argument, bindings))
                .collect::<Vec<_>>();
            if scope.type_params.len() != concrete_args.len() {
                visiting.remove(&key);
                return false;
            }
            let nested_bindings = scope
                .type_params
                .into_iter()
                .zip(concrete_args)
                .collect::<HashMap<_, _>>();
            let preserved = scope.member_types.iter().all(|member| {
                preserves_union_structure(member, &nested_bindings, class_scope, visiting)
            });
            visiting.remove(&key);
            preserved
        }
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        UnionStructureClassScope, substitute_type_vars_with_class_scopes,
        substitution_preserves_union_structure,
        substitution_preserves_union_structure_with_class_scopes,
    };
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

    #[test]
    fn union_structure_rejects_expansion_and_collapse_after_substitution() {
        let template = Type::Union(vec![Type::None, Type::TypeVar("T".to_string())]);

        assert!(substitution_preserves_union_structure(
            &template,
            &HashMap::from([("T".to_string(), Type::Str)]),
        ));
        assert!(!substitution_preserves_union_structure(
            &template,
            &HashMap::from([(
                "T".to_string(),
                crate::make_union(vec![Type::None, Type::Str]),
            )]),
        ));
        assert!(!substitution_preserves_union_structure(
            &template,
            &HashMap::from([("T".to_string(), Type::None)]),
        ));
    }

    #[test]
    fn union_structure_does_not_capture_nested_class_fields() {
        let nested = Type::Class {
            identity: Some("models.Inner".to_string()),
            type_args: vec![Type::Str],
            name: "Inner".to_string(),
            fields: vec![(
                "value".to_string(),
                Type::Union(vec![Type::None, Type::TypeVar("T".to_string())]),
            )],
            methods: Vec::new(),
            parent_class: None,
        };
        let outer_optional = crate::make_union(vec![Type::None, Type::Int]);

        assert!(substitution_preserves_union_structure(
            &nested,
            &HashMap::from([("T".to_string(), outer_optional)]),
        ));
    }

    #[test]
    fn scoped_union_structure_detects_a_transitive_nested_expansion() {
        let nested = Type::Class {
            identity: Some("models.Inner".to_string()),
            type_args: vec![Type::TypeVar("T".to_string())],
            name: "Inner".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        };
        let outer_optional = crate::make_union(vec![Type::None, Type::Str]);
        let preserved = substitution_preserves_union_structure_with_class_scopes(
            &nested,
            &HashMap::from([("T".to_string(), outer_optional)]),
            &|identity, _| {
                (identity == Some("models.Inner")).then(|| UnionStructureClassScope {
                    type_params: vec!["U".to_string()],
                    member_types: vec![Type::Union(vec![
                        Type::None,
                        Type::TypeVar("U".to_string()),
                    ])],
                })
            },
        );

        assert!(!preserved);
    }

    #[test]
    fn scoped_union_structure_rejects_parameter_arity_mismatch() {
        let nested = Type::Class {
            identity: Some("models.Inner".to_string()),
            type_args: vec![Type::Str],
            name: "Inner".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        };
        let preserved = substitution_preserves_union_structure_with_class_scopes(
            &nested,
            &HashMap::new(),
            &|_, _| {
                Some(UnionStructureClassScope {
                    type_params: vec!["T".to_string(), "U".to_string()],
                    member_types: Vec::new(),
                })
            },
        );

        assert!(!preserved);
    }

    #[test]
    fn generic_alias_arguments_and_body_use_the_same_outer_binding() {
        let alias = Type::Alias {
            name: "Boxed".to_string(),
            type_args: vec![Type::TypeVar("T".to_string())],
            body: Box::new(Type::List(Box::new(Type::TypeVar("T".to_string())))),
        };
        let substituted = substitute_type_vars_with_class_scopes(
            &alias,
            &HashMap::from([("T".to_string(), Type::Int)]),
            &|_, _| None,
        );

        assert_eq!(
            substituted,
            Type::Alias {
                name: "Boxed".to_string(),
                type_args: vec![Type::Int],
                body: Box::new(Type::List(Box::new(Type::Int))),
            }
        );
    }
}
