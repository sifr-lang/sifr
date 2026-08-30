use super::{HirExpr, Type};

pub(super) fn is_none_like_result_value(value: &HirExpr) -> bool {
    matches!(value, HirExpr::NoneLiteral)
        || matches!(
            crate::resolve_alias_type_for_plain_call(value.ty()),
            Type::None
        )
        || matches!(
            value,
            HirExpr::OkWrap { value, .. }
                if matches!(
                    crate::resolve_alias_type_for_plain_call(value.ty()),
                    Type::None
                )
        )
}

pub(super) fn is_result_int_division_error_type(ty: &Type) -> bool {
    let Type::Result(ok_ty, err_ty) = ty else {
        return false;
    };
    matches!(
        crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
        Type::Int | Type::LiteralInt(_)
    ) && matches!(
        crate::resolve_alias_type_for_plain_call(err_ty.as_ref()),
        Type::Class { name, .. } if name == "DivisionError"
    )
}

pub(super) fn integer_arithmetic_error_union(ty: &Type) -> Option<(&Type, &Type, &Type)> {
    let Type::Result(ok_ty, err_ty) = ty else {
        return None;
    };
    if !matches!(
        crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
        Type::Int | Type::LiteralInt(_)
    ) {
        return None;
    }
    let Type::Union(members) = crate::resolve_alias_type_for_plain_call(err_ty.as_ref()) else {
        return None;
    };
    let value_error = members.iter().find(|member| {
        matches!(
            crate::resolve_alias_type_for_plain_call(member),
            Type::Class { name, .. } if name == "ValueError"
        )
    })?;
    let limit_error = members.iter().find(|member| {
        matches!(
            crate::resolve_alias_type_for_plain_call(member),
            Type::Class { name, .. } if name == "ArithmeticLimitError"
        )
    })?;
    Some((err_ty.as_ref(), value_error, limit_error))
}

pub(super) fn integer_division_error_union(ty: &Type) -> Option<(&Type, &Type, &Type, &Type)> {
    let Type::Result(ok_ty, err_ty) = ty else {
        return None;
    };
    if !matches!(
        crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
        Type::Float
    ) {
        return None;
    }
    let Type::Union(members) = crate::resolve_alias_type_for_plain_call(err_ty.as_ref()) else {
        return None;
    };
    let find = |expected: &str| {
        members.iter().find(|member| {
            matches!(
                crate::resolve_alias_type_for_plain_call(member),
                Type::Class { name, .. } if name == expected
            )
        })
    };
    Some((
        err_ty.as_ref(),
        find("DivisionError")?,
        find("FloatOverflowError")?,
        find("FloatPrecisionLossError")?,
    ))
}

pub(super) fn result_error_member<'a>(
    result_ty: &'a Type,
    expected: &str,
) -> Option<(&'a Type, &'a Type)> {
    let Type::Result(_, error_ty) = result_ty.resolve_alias() else {
        return None;
    };
    let resolved_error = crate::resolve_alias_type_for_plain_call(error_ty.as_ref());
    if matches!(resolved_error, Type::Class { name, .. } if name == expected) {
        return Some((error_ty.as_ref(), error_ty.as_ref()));
    }
    let Type::Union(members) = resolved_error else {
        return None;
    };
    let member = members.iter().find(|member| {
        matches!(
            crate::resolve_alias_type_for_plain_call(member),
            Type::Class { name, .. } if name == expected
        )
    })?;
    Some((error_ty.as_ref(), member))
}

pub(crate) fn integer_float_conversion_error_union(ty: &Type) -> Option<(&Type, &Type, &Type)> {
    let Type::Result(ok_ty, err_ty) = ty else {
        return None;
    };
    if !matches!(
        crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
        Type::Float
    ) {
        return None;
    }
    let Type::Union(members) = crate::resolve_alias_type_for_plain_call(err_ty.as_ref()) else {
        return None;
    };
    let find = |expected: &str| {
        members.iter().find(|member| {
            matches!(
                crate::resolve_alias_type_for_plain_call(member),
                Type::Class { name, .. } if name == expected
            )
        })
    };
    Some((
        err_ty.as_ref(),
        find("FloatOverflowError")?,
        find("FloatPrecisionLossError")?,
    ))
}

pub(super) fn result_int_to_sifr_int_rust_type(ty: &Type) -> crate::RustType {
    let Type::Result(_, err_ty) = ty else {
        return crate::sifr_type_to_rust_type(ty);
    };
    crate::RustType::Result(
        Box::new(crate::RustType::Named("SifrInt".to_string())),
        Box::new(crate::sifr_type_to_rust_type(err_ty)),
    )
}
