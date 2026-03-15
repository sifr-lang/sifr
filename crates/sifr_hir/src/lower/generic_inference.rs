use sifr_type_system::Type;
use std::collections::HashMap;

pub(super) fn infer_type_var_bindings(
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
        (Type::Result(p_ok, p_err), Type::Result(a_ok, a_err)) => {
            infer_type_var_bindings(p_ok, a_ok, bindings);
            infer_type_var_bindings(p_err, a_err, bindings);
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
