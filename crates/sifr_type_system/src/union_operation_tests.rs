use crate::{
    Type, intersect_with_union, remove_none_from_union, subtract_from_union, union_contains,
    union_contains_none,
};

#[test]
fn subtracts_a_member_from_a_union() {
    let union = Type::Union(vec![Type::Int, Type::Str]);
    assert_eq!(subtract_from_union(&union, &Type::Int), Type::Str);
}

#[test]
fn subtracts_none_from_an_optional() {
    let optional = Type::Union(vec![Type::None, Type::Str]);
    assert_eq!(remove_none_from_union(&optional), Type::Str);
}

#[test]
fn subtracts_only_the_outer_none_from_a_nested_optional_union() {
    let payload = crate::make_union(vec![Type::Int, Type::Str, Type::None]);
    let nested = Type::Union(vec![payload.clone(), Type::None]);
    assert_eq!(remove_none_from_union(&nested), payload);
}

#[test]
fn intersects_a_union_with_a_member() {
    let union = Type::Union(vec![Type::Bool, Type::Int, Type::Str]);
    assert_eq!(intersect_with_union(&union, &Type::Int), Type::Int);
}

#[test]
fn reports_union_membership() {
    let union = Type::Union(vec![Type::Int, Type::Str]);
    assert!(union_contains(&union, &Type::Int));
    assert!(union_contains(&union, &Type::Str));
    assert!(!union_contains(&union, &Type::Bool));
}

#[test]
fn reports_none_membership() {
    let optional = Type::Union(vec![Type::None, Type::Str]);
    assert!(union_contains_none(&optional));

    let union = Type::Union(vec![Type::Int, Type::Str]);
    assert!(!union_contains_none(&union));
}

#[test]
fn reports_assignability_to_union_members() {
    let members = vec![Type::Int, Type::Str];
    assert!(Type::Int.is_assignable_to(&Type::Union(members.clone())));
    assert!(Type::Str.is_assignable_to(&Type::Union(members.clone())));
    assert!(!Type::Bool.is_assignable_to(&Type::Union(members)));
}

#[test]
fn intersects_a_literal_with_its_union_base() {
    let union = Type::Union(vec![Type::Int, Type::Str]);
    assert_eq!(
        intersect_with_union(&union, &Type::LiteralInt(42)),
        Type::Int
    );
}
