use super::Type;

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

pub(super) fn result_int_to_sifr_int_rust_type(ty: &Type) -> crate::RustType {
    let Type::Result(_, err_ty) = ty else {
        return crate::RustType::Named(ty.rust_type());
    };
    crate::RustType::Result(
        Box::new(crate::RustType::Named("SifrInt".to_string())),
        Box::new(crate::sifr_type_to_rust_type(err_ty)),
    )
}
