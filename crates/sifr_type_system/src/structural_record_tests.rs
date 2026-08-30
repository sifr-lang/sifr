use crate::{OwnershipKind, StructuralRecordType, Type};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn record(fields: &[(&str, Type)]) -> Type {
    Type::StructuralRecord(StructuralRecordType::new(
        fields
            .iter()
            .map(|(name, ty)| ((*name).to_string(), ty.clone()))
            .collect(),
    ))
}

fn hash(ty: &Type) -> u64 {
    let mut hasher = DefaultHasher::new();
    ty.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn identity_and_hash_ignore_declaration_order() {
    let first = record(&[("id", Type::Int), ("email", Type::Str)]);
    let second = record(&[("email", Type::Str), ("id", Type::Int)]);
    assert_eq!(first, second);
    assert_eq!(hash(&first), hash(&second));
    assert_eq!(
        first.canonical_identity_key(),
        second.canonical_identity_key()
    );
}

#[test]
fn canonical_identity_is_constant_for_every_field_permutation() {
    let permutations = [
        ["active", "email", "id"],
        ["active", "id", "email"],
        ["email", "active", "id"],
        ["email", "id", "active"],
        ["id", "active", "email"],
        ["id", "email", "active"],
    ];
    let field_type = |name: &str| match name {
        "active" => Type::Bool,
        "email" => Type::Str,
        "id" => Type::Int,
        _ => unreachable!("the property fixture has a closed field set"),
    };
    let records = permutations.map(|names| {
        record(
            &names
                .map(|name| (name, field_type(name)))
                .into_iter()
                .collect::<Vec<_>>(),
        )
    });
    let expected_identity = records[0].canonical_identity_key();
    let expected_hash = hash(&records[0]);
    for record in &records[1..] {
        assert_eq!(record.canonical_identity_key(), expected_identity);
        assert_eq!(hash(record), expected_hash);
        assert_eq!(record, &records[0]);
    }
}

#[test]
fn width_subtyping_exists_only_in_shared_borrow_relation() {
    let wider = record(&[("id", Type::Int), ("email", Type::Str)]);
    let narrower = record(&[("id", Type::Int)]);
    assert!(!wider.is_assignable_to(&narrower));
    assert!(wider.is_shared_borrow_assignable_to(&narrower));
    assert!(!narrower.is_shared_borrow_assignable_to(&wider));
}

#[test]
fn record_capabilities_are_the_intersection_of_field_capabilities() {
    let copy = record(&[("id", Type::Int), ("active", Type::Bool)]);
    assert_eq!(copy.ownership(), OwnershipKind::Copy);
    assert!(copy.supports_derived_clone());
    assert!(copy.supports_structural_equality());
    assert!(copy.supports_hash_key());
    assert!(copy.supports_total_order());

    let moved = record(&[("id", Type::Int), ("email", Type::Str)]);
    assert_eq!(moved.ownership(), OwnershipKind::Move);
    assert!(moved.supports_derived_clone());
}

#[test]
fn mutable_containers_remain_invariant_over_width_related_records() {
    let wider = record(&[("id", Type::Int), ("email", Type::Str)]);
    let narrower = record(&[("id", Type::Int)]);
    assert!(!Type::List(Box::new(wider)).is_assignable_to(&Type::List(Box::new(narrower))));
}

#[test]
fn width_related_record_union_is_detectable() {
    let wider = record(&[("id", Type::Int), ("email", Type::Str)]);
    let narrower = record(&[("id", Type::Int)]);
    let union = crate::make_union(vec![wider, narrower]);
    assert!(union.has_width_related_structural_records());
}
