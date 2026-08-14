use super::Type;

pub(super) fn option_member_type(ty: &Type) -> Option<Type> {
    ty.optional_member_type()
}
