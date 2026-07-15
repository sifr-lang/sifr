use super::Type;

pub(super) fn has_conflicting_class_specializations(ty: &Type) -> bool {
    let Type::Union(members) = ty.resolve_alias() else {
        return false;
    };
    for (index, left) in members.iter().enumerate() {
        let Type::Class {
            identity: left_identity,
            name: left_name,
            ..
        } = left.resolve_alias()
        else {
            continue;
        };
        let left_identity = left_identity.as_ref().unwrap_or(left_name);
        for right in &members[index + 1..] {
            let Type::Class {
                identity: right_identity,
                name: right_name,
                ..
            } = right.resolve_alias()
            else {
                continue;
            };
            let right_identity = right_identity.as_ref().unwrap_or(right_name);
            if left_identity == right_identity && left != right {
                return true;
            }
        }
    }
    false
}
