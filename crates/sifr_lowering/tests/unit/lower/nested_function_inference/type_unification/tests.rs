
use super::*;

#[test]
fn inference_hole_replacement_recanonicalizes_and_deduplicates_unions() {
    let ty = Type::Union(vec![Type::Unknown, Type::Any]);

    assert_eq!(replace_inference_holes_with_any(ty), Type::Any);
}
