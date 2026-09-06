use sifr_type_system::{FunctionType, OwnershipKind, ParamConvention, Type, make_union};
use std::collections::HashMap;

pub(super) fn imported_constructor_function_type(class_ty: &Type) -> Option<FunctionType> {
    let Type::Class {
        fields, methods, ..
    } = class_ty.resolve_alias()
    else {
        return None;
    };
    if let Some((_, constructor)) = methods.iter().find(|(name, _)| name == "new") {
        return Some(FunctionType {
            receiver: None,
            params: constructor.params.clone(),
            return_type: Box::new(class_ty.clone()),
        });
    }
    let params = fields
        .iter()
        .map(|(name, ty)| {
            let convention = if ty.contains_affine_resource()
                || matches!(ty, Type::TypeVar(_))
                || ty.ownership() == OwnershipKind::Copy
            {
                ParamConvention::own()
            } else {
                ParamConvention::borrow()
            };
            (name.clone(), ty.clone(), convention)
        })
        .collect();
    Some(FunctionType {
        receiver: None,
        params,
        return_type: Box::new(class_ty.clone()),
    })
}

pub(super) fn complete_class_type_with_identity(
    class_types: &HashMap<String, Type>,
    identity: &str,
) -> Option<Type> {
    class_types
        .values()
        .filter(|candidate| {
            matches!(
                candidate.resolve_alias(),
                Type::Class {
                    identity: Some(candidate_identity),
                    ..
                } if candidate_identity == identity
            )
        })
        .max_by_key(|candidate| match candidate.resolve_alias() {
            Type::Class {
                fields, methods, ..
            } => (methods.len(), fields.len()),
            _ => (0, 0),
        })
        .cloned()
}

pub(super) fn complete_context_enter_return_type(
    class_types: &HashMap<String, Type>,
    context_type: &Type,
    return_type: &Type,
) -> Type {
    let Type::Class { identity, name, .. } = return_type.resolve_alias() else {
        return return_type.clone();
    };
    let returns_context_identity = match context_type.resolve_alias() {
        Type::Class {
            identity: context_identity,
            ..
        } => identity.is_some() && identity == context_identity,
        _ => false,
    };
    if returns_context_identity {
        return context_type.clone();
    }
    identity
        .as_deref()
        .and_then(|identity| complete_class_type_with_identity(class_types, identity))
        .or_else(|| class_types.get(name).cloned())
        .unwrap_or_else(|| return_type.clone())
}

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
        receiver: function.receiver,
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
        Type::PythonDlpackTensor(inner) => {
            Type::PythonDlpackTensor(Box::new(rename_class_identities(inner, class_aliases)))
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
        Type::Union(items) => make_union(
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
        Type::Newtype {
            identity,
            name,
            inner,
        } => Type::Newtype {
            identity: identity.clone(),
            name: class_aliases
                .get(name)
                .cloned()
                .unwrap_or_else(|| name.clone()),
            inner: Box::new(rename_class_identities(inner, class_aliases)),
        },
        Type::Protocol {
            identity,
            name,
            methods,
        } => Type::Protocol {
            identity: identity.clone(),
            name: class_aliases
                .get(name)
                .cloned()
                .unwrap_or_else(|| name.clone()),
            methods: methods
                .iter()
                .map(|(name, function)| {
                    (name.clone(), rename_function_type(function, class_aliases))
                })
                .collect(),
        },
        Type::Enum {
            identity,
            name,
            variants,
        } => Type::Enum {
            identity: identity.clone(),
            name: class_aliases
                .get(name)
                .cloned()
                .unwrap_or_else(|| name.clone()),
            variants: variants.clone(),
        },
        Type::Class {
            identity,
            type_args,
            name,
            fields,
            methods,
            parent_class,
        } => Type::Class {
            identity: identity.clone(),
            type_args: type_args
                .iter()
                .map(|arg| rename_class_identities(arg, class_aliases))
                .collect(),
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

pub(super) fn canonicalize_export_type(ty: &Type, local_classes: &HashMap<String, String>) -> Type {
    canonicalize_class_identities(ty, local_classes)
}

pub(super) fn canonicalize_export_type_in_place(
    ty: &mut Type,
    local_classes: &HashMap<String, String>,
) {
    set_canonical_identities(ty, local_classes);
}

pub(super) fn canonicalize_export_function_type(
    function: &FunctionType,
    local_classes: &HashMap<String, String>,
) -> FunctionType {
    FunctionType {
        receiver: function.receiver,
        params: function
            .params
            .iter()
            .map(|(name, ty, convention)| {
                (
                    name.clone(),
                    canonicalize_class_identities(ty, local_classes),
                    *convention,
                )
            })
            .collect(),
        return_type: Box::new(canonicalize_class_identities(
            &function.return_type,
            local_classes,
        )),
    }
}

fn canonicalize_class_identities(ty: &Type, local_classes: &HashMap<String, String>) -> Type {
    let mut canonicalized = ty.clone();
    set_canonical_identities(&mut canonicalized, local_classes);
    canonicalized
}

fn set_canonical_identities(ty: &mut Type, local_classes: &HashMap<String, String>) {
    match ty {
        Type::List(inner)
        | Type::Set(inner)
        | Type::Iterable(inner)
        | Type::Iterator(inner)
        | Type::PythonBuffer(inner)
        | Type::PythonDlpackTensor(inner)
        | Type::Awaitable(inner)
        | Type::Failure(inner)
        | Type::TimeoutResult(inner) => set_canonical_identities(inner, local_classes),
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
            set_canonical_identities(left, local_classes);
            set_canonical_identities(right, local_classes);
        }
        Type::Tuple(items) | Type::Intersection(items) => {
            for item in items {
                set_canonical_identities(item, local_classes);
            }
        }
        Type::Union(items) => {
            for item in &mut *items {
                set_canonical_identities(item, local_classes);
            }
            *ty = make_union(std::mem::take(items));
        }
        Type::Callable(params, _, result) | Type::AsyncCallable(params, _, result) => {
            for param in params {
                set_canonical_identities(param, local_classes);
            }
            set_canonical_identities(result, local_classes);
        }
        Type::Function(function) | Type::AsyncFunction(function) => {
            for (_, param, _) in &mut function.params {
                set_canonical_identities(param, local_classes);
            }
            set_canonical_identities(&mut function.return_type, local_classes);
        }
        Type::Alias {
            type_args, body, ..
        } => {
            for arg in type_args {
                set_canonical_identities(arg, local_classes);
            }
            set_canonical_identities(body, local_classes);
        }
        Type::Class {
            identity,
            type_args,
            name,
            fields,
            methods,
            parent_class,
        } => {
            if identity.is_none() {
                *identity = local_classes.get(name).cloned();
            }
            for arg in type_args {
                set_canonical_identities(arg, local_classes);
            }
            for (_, field) in fields {
                set_canonical_identities(field, local_classes);
            }
            for (_, method) in methods {
                for (_, param, _) in &mut method.params {
                    set_canonical_identities(param, local_classes);
                }
                set_canonical_identities(&mut method.return_type, local_classes);
            }
            if let Some(chain) = parent_class {
                if chain
                    .split('|')
                    .any(|parent| local_classes.contains_key(parent))
                {
                    *chain = chain
                        .split('|')
                        .map(|parent| {
                            local_classes
                                .get(parent)
                                // A same-named base refers to the prior declaration
                                // (notably builtin Error), never the class itself.
                                .filter(|parent_identity| {
                                    parent != "Error" && Some(*parent_identity) != identity.as_ref()
                                })
                                .map_or_else(|| parent.to_string(), Clone::clone)
                        })
                        .collect::<Vec<_>>()
                        .join("|");
                }
            }
        }
        Type::Protocol {
            identity,
            name,
            methods,
        } => {
            if identity.is_none() {
                *identity = local_classes.get(name).cloned();
            }
            for (_, method) in methods {
                for (_, param, _) in &mut method.params {
                    set_canonical_identities(param, local_classes);
                }
                set_canonical_identities(&mut method.return_type, local_classes);
            }
        }
        Type::Newtype {
            identity,
            name,
            inner,
        } => {
            if identity.is_none() {
                *identity = local_classes.get(name).cloned();
            }
            set_canonical_identities(inner, local_classes);
        }
        Type::Enum { identity, name, .. } if identity.is_none() => {
            *identity = local_classes.get(name).cloned();
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class(name: &str, identity: Option<&str>) -> Type {
        Type::Class {
            identity: identity.map(str::to_string),
            type_args: Vec::new(),
            name: name.to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        }
    }

    fn member_identities(ty: Type) -> Vec<String> {
        let Type::Union(members) = ty else {
            panic!("expected union");
        };
        members.iter().map(Type::union_variant_name).collect()
    }

    #[test]
    fn imported_alias_renaming_keeps_union_identity_order() {
        let original = make_union(vec![
            class("Alpha", Some("pkg.Alpha")),
            class("Beta", Some("pkg.Beta")),
        ]);
        let aliases = HashMap::from([("Alpha".to_string(), "Zeta".to_string())]);
        let renamed = rename_class_identities(&original, &aliases);

        assert_eq!(member_identities(original), member_identities(renamed));
    }

    #[test]
    fn canonical_identity_insertion_reorders_union_members() {
        let mut actual = Type::Union(vec![class("Zeta", None), class("Beta", None)]);
        let identities = HashMap::from([
            ("Zeta".to_string(), "pkg.Alpha".to_string()),
            ("Beta".to_string(), "pkg.Beta".to_string()),
        ]);
        set_canonical_identities(&mut actual, &identities);
        let expected = make_union(vec![
            class("Zeta", Some("pkg.Alpha")),
            class("Beta", Some("pkg.Beta")),
        ]);

        assert_eq!(actual, expected);
    }
}
