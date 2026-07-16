use sifr_type_system::Type;

/// Collect all `TypeVar` names used in a type.
pub(in crate::lower) fn collect_type_vars(ty: &Type, vars: &mut Vec<String>) {
    match ty {
        Type::TypeVar(name) => {
            if !vars.contains(name) {
                vars.push(name.clone());
            }
        }
        Type::List(elem) | Type::Set(elem) | Type::Iterable(elem) | Type::Iterator(elem) => {
            collect_type_vars(elem, vars);
        }
        Type::Dict(k, v) => {
            collect_type_vars(k, vars);
            collect_type_vars(v, vars);
        }
        Type::Tuple(elems) => {
            for e in elems {
                collect_type_vars(e, vars);
            }
        }
        Type::Union(members) => {
            for m in members {
                collect_type_vars(m, vars);
            }
        }
        Type::Result(ok, err) => {
            collect_type_vars(ok, vars);
            collect_type_vars(err, vars);
        }
        Type::Coroutine(ok, err)
        | Type::Task(ok, err)
        | Type::TaskResult(ok, err)
        | Type::Select2(ok, err)
        | Type::BlockingTask(ok, err)
        | Type::JoinSet(ok, err)
        | Type::AsyncIterator(ok, err)
        | Type::AsyncGenerator(ok, err) => {
            collect_type_vars(ok, vars);
            collect_type_vars(err, vars);
        }
        Type::Failure(err) | Type::PythonBuffer(err) => collect_type_vars(err, vars),
        Type::TimeoutResult(err) => collect_type_vars(err, vars),
        Type::Awaitable(result) => collect_type_vars(result, vars),
        Type::Alias {
            type_args, body, ..
        } => {
            for arg in type_args {
                collect_type_vars(arg, vars);
            }
            collect_type_vars(body, vars);
        }
        Type::Function(ft) | Type::AsyncFunction(ft) => {
            for (_, param_ty, _) in &ft.params {
                collect_type_vars(param_ty, vars);
            }
            collect_type_vars(&ft.return_type, vars);
        }
        Type::Class {
            type_args,
            fields,
            methods,
            ..
        } => {
            for type_arg in type_args {
                collect_type_vars(type_arg, vars);
            }
            for (_, field_ty) in fields {
                collect_type_vars(field_ty, vars);
            }
            for (_, method_ft) in methods {
                for (_, param_ty, _) in &method_ft.params {
                    collect_type_vars(param_ty, vars);
                }
                collect_type_vars(&method_ft.return_type, vars);
            }
        }
        Type::Callable(params, _, ret) | Type::AsyncCallable(params, _, ret) => {
            for p in params {
                collect_type_vars(p, vars);
            }
            collect_type_vars(ret, vars);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::collect_type_vars;
    use sifr_type_system::Type;

    #[test]
    fn collects_typevars_from_nominal_class_arguments() {
        let ty = Type::Class {
            identity: Some("pkg.Node".to_string()),
            type_args: vec![Type::TypeVar("T".to_string())],
            name: "Node".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        };
        let mut vars = Vec::new();
        collect_type_vars(&ty, &mut vars);
        assert_eq!(vars, vec!["T".to_string()]);
    }
}
