use crate::{NarrowingCondition, Type, make_union, narrow_type};

fn sample_types() -> Vec<Type> {
    vec![
        Type::None,
        Type::Bool,
        Type::Int,
        Type::Float,
        Type::Str,
        Type::Bytes,
    ]
}

#[test]
fn semantic_property_union_normalization_is_idempotent_and_permutation_invariant() {
    let samples = sample_types();
    for first in &samples {
        for second in &samples {
            for third in &samples {
                let forward = make_union(vec![first.clone(), second.clone(), third.clone()]);
                let reverse = make_union(vec![third.clone(), second.clone(), first.clone()]);
                let nested = make_union(vec![
                    first.clone(),
                    make_union(vec![second.clone(), third.clone(), first.clone()]),
                ]);

                assert_eq!(forward, reverse, "union order changed canonical identity");
                assert_eq!(forward, nested, "nested union did not normalize");
                assert_eq!(make_union(vec![forward.clone()]), forward);
            }
        }
    }
}

#[test]
fn semantic_property_narrowing_partitions_closed_unions() {
    let samples = sample_types();
    for selected in &samples {
        let original = make_union(samples.clone());
        let condition = NarrowingCondition::IsInstance("value".to_string(), selected.clone());
        let when_true = narrow_type(&original, &condition, true);
        let when_false = narrow_type(&original, &condition, false);

        assert_eq!(make_union(vec![when_true, when_false]), original);
        assert_eq!(
            narrow_type(
                &original,
                &NarrowingCondition::Not(Box::new(condition.clone())),
                true,
            ),
            narrow_type(&original, &condition, false),
        );
    }
}
