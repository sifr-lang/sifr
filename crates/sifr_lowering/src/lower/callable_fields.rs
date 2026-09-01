use sifr_type_system::{FunctionType, ParamConvention, Type};

pub(super) fn callable_field_function_type(
    object_ty: &Type,
    field_name: &str,
) -> Option<FunctionType> {
    let field_ty = object_ty.callable_field_type(field_name)?;
    let (params, conventions, return_type) = match field_ty.resolve_alias() {
        Type::Callable(params, conventions, return_type)
        | Type::AsyncCallable(params, conventions, return_type) => {
            (params, conventions, return_type)
        }
        _ => return None,
    };
    Some(FunctionType {
        receiver: None,
        params: params
            .iter()
            .enumerate()
            .map(|(index, ty)| {
                (
                    format!("arg{index}"),
                    ty.clone(),
                    conventions
                        .get(index)
                        .copied()
                        .unwrap_or_else(ParamConvention::borrow),
                )
            })
            .collect(),
        return_type: return_type.clone(),
    })
}

pub(super) fn callable_argument_is_assignable(
    source: &Type,
    target: &Type,
    convention: ParamConvention,
) -> bool {
    if convention.is_shared_borrow() {
        source.is_shared_borrow_assignable_to(target)
    } else {
        source.is_assignable_to(target)
    }
}
