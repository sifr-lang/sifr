use sifr_type_system::Type;
use std::collections::HashMap;

pub(in crate::lower) fn infer_type_var_bindings(
    param_ty: &Type,
    arg_ty: &Type,
    bindings: &mut HashMap<String, Type>,
) {
    match (param_ty, arg_ty) {
        (Type::TypeVar(name), concrete) => {
            if !bindings.contains_key(name) {
                bindings.insert(name.clone(), concrete.clone());
            }
        }
        (Type::Iterable(p_elem), arg) => {
            if let Some(arg_elem) = arg.iterable_element_type() {
                infer_type_var_bindings(p_elem, &arg_elem, bindings);
            }
        }
        (Type::Iterator(p_elem), arg) => {
            if let Type::Iterator(a_elem) = arg.resolve_alias() {
                infer_type_var_bindings(p_elem, a_elem, bindings);
            }
        }
        (Type::List(p_elem), Type::List(a_elem)) | (Type::Set(p_elem), Type::Set(a_elem)) => {
            infer_type_var_bindings(p_elem, a_elem, bindings);
        }
        (Type::Dict(pk, pv), Type::Dict(ak, av)) => {
            infer_type_var_bindings(pk, ak, bindings);
            infer_type_var_bindings(pv, av, bindings);
        }
        (Type::Tuple(p_elems), Type::Tuple(a_elems)) if p_elems.len() == a_elems.len() => {
            for (p, a) in p_elems.iter().zip(a_elems.iter()) {
                infer_type_var_bindings(p, a, bindings);
            }
        }
        (Type::Union(param_members), other) => {
            let has_none_branch = param_members
                .iter()
                .any(|member| matches!(member.resolve_alias(), Type::None));
            if has_none_branch {
                if !matches!(other.resolve_alias(), Type::None) {
                    for member in param_members {
                        if !matches!(member.resolve_alias(), Type::None) {
                            infer_type_var_bindings(member, other, bindings);
                        }
                    }
                }
                return;
            }
            for member in param_members {
                if other.is_assignable_to(member) || member.is_assignable_to(other) {
                    infer_type_var_bindings(member, other, bindings);
                }
            }
        }
        (param, Type::Union(arg_members)) => {
            for member in arg_members {
                if matches!(member.resolve_alias(), Type::None) {
                    continue;
                }
                infer_type_var_bindings(param, member, bindings);
            }
        }
        (Type::Result(p_ok, p_err), Type::Result(a_ok, a_err)) => {
            infer_type_var_bindings(p_ok, a_ok, bindings);
            infer_type_var_bindings(p_err, a_err, bindings);
        }
        (Type::Coroutine(p_ok, p_err), Type::Coroutine(a_ok, a_err))
        | (Type::Task(p_ok, p_err), Type::Task(a_ok, a_err))
        | (Type::TaskResult(p_ok, p_err), Type::TaskResult(a_ok, a_err))
        | (Type::Select2(p_ok, p_err), Type::Select2(a_ok, a_err))
        | (Type::BlockingTask(p_ok, p_err), Type::BlockingTask(a_ok, a_err))
        | (Type::AsyncIterator(p_ok, p_err), Type::AsyncIterator(a_ok, a_err))
        | (Type::AsyncGenerator(p_ok, p_err), Type::AsyncGenerator(a_ok, a_err)) => {
            infer_type_var_bindings(p_ok, a_ok, bindings);
            infer_type_var_bindings(p_err, a_err, bindings);
        }
        (Type::TimeoutResult(p_err), Type::TimeoutResult(a_err)) => {
            infer_type_var_bindings(p_err, a_err, bindings);
        }
        (Type::Failure(p_err), Type::Failure(a_err)) => {
            infer_type_var_bindings(p_err, a_err, bindings);
        }
        (Type::Awaitable(p_result), Type::Awaitable(a_result)) => {
            infer_type_var_bindings(p_result, a_result, bindings);
        }
        (Type::Callable(p_params, _, p_ret), Type::Callable(a_params, _, a_ret))
            if p_params.len() == a_params.len() =>
        {
            for (p_param, a_param) in p_params.iter().zip(a_params.iter()) {
                infer_type_var_bindings(p_param, a_param, bindings);
            }
            infer_type_var_bindings(p_ret, a_ret, bindings);
        }
        (Type::Callable(p_params, _, p_ret), Type::Function(a_ft))
            if p_params.len() == a_ft.params.len() =>
        {
            for (p_param, (_, a_param, _)) in p_params.iter().zip(a_ft.params.iter()) {
                infer_type_var_bindings(p_param, a_param, bindings);
            }
            infer_type_var_bindings(p_ret, &a_ft.return_type, bindings);
        }
        (Type::Function(p_ft), Type::Callable(a_params, _, a_ret))
            if p_ft.params.len() == a_params.len() =>
        {
            for ((_, p_param, _), a_param) in p_ft.params.iter().zip(a_params.iter()) {
                infer_type_var_bindings(p_param, a_param, bindings);
            }
            infer_type_var_bindings(&p_ft.return_type, a_ret, bindings);
        }
        (
            Type::Alias {
                name: p_name,
                type_args: p_args,
                body: p_body,
            },
            Type::Alias {
                name: a_name,
                type_args: a_args,
                body: a_body,
            },
        ) if p_name == a_name && p_args.len() == a_args.len() => {
            for (p_arg, a_arg) in p_args.iter().zip(a_args.iter()) {
                infer_type_var_bindings(p_arg, a_arg, bindings);
            }
            infer_type_var_bindings(p_body, a_body, bindings);
        }
        (Type::Alias { body, .. }, other) => {
            infer_type_var_bindings(body, other, bindings);
        }
        (other, Type::Alias { body, .. }) => {
            infer_type_var_bindings(other, body, bindings);
        }
        (
            Type::Class {
                name: p_name,
                fields: p_fields,
                ..
            },
            Type::Class {
                name: a_name,
                fields: a_fields,
                ..
            },
        ) if p_name == a_name && p_fields.len() == a_fields.len() => {
            for ((_, p_ty), (_, a_ty)) in p_fields.iter().zip(a_fields.iter()) {
                infer_type_var_bindings(p_ty, a_ty, bindings);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::infer_type_var_bindings;
    use sifr_type_system::Type;
    use std::collections::HashMap;

    #[test]
    fn infers_iterable_typevar_from_list_argument() {
        let param = Type::Iterable(Box::new(Type::TypeVar("T".to_string())));
        let arg = Type::List(Box::new(Type::Int));
        let mut bindings = HashMap::new();
        infer_type_var_bindings(&param, &arg, &mut bindings);
        assert_eq!(bindings.get("T"), Some(&Type::Int));
    }

    #[test]
    fn infers_nested_iterable_typevar_from_list_of_lists() {
        let param = Type::Iterable(Box::new(Type::Iterable(Box::new(Type::TypeVar(
            "T".to_string(),
        )))));
        let arg = Type::List(Box::new(Type::List(Box::new(Type::Int))));
        let mut bindings = HashMap::new();
        infer_type_var_bindings(&param, &arg, &mut bindings);
        assert_eq!(bindings.get("T"), Some(&Type::Int));
    }

    #[test]
    fn infers_iterable_typevar_from_iterator_argument() {
        let param = Type::Iterable(Box::new(Type::TypeVar("T".to_string())));
        let arg = Type::Iterator(Box::new(Type::Str));
        let mut bindings = HashMap::new();
        infer_type_var_bindings(&param, &arg, &mut bindings);
        assert_eq!(bindings.get("T"), Some(&Type::Str));
    }

    #[test]
    fn infers_typevar_from_optional_union_parameter_non_none_branch() {
        let param = Type::Union(vec![
            Type::List(Box::new(Type::TypeVar("T".to_string()))),
            Type::None,
        ]);
        let arg = Type::List(Box::new(Type::Int));
        let mut bindings = HashMap::new();
        infer_type_var_bindings(&param, &arg, &mut bindings);
        assert_eq!(bindings.get("T"), Some(&Type::Int));
    }

    #[test]
    fn optional_union_parameter_does_not_bind_typevar_from_none_argument() {
        let param = Type::Union(vec![
            Type::List(Box::new(Type::TypeVar("T".to_string()))),
            Type::None,
        ]);
        let arg = Type::None;
        let mut bindings = HashMap::new();
        infer_type_var_bindings(&param, &arg, &mut bindings);
        assert!(bindings.is_empty());
    }
}
