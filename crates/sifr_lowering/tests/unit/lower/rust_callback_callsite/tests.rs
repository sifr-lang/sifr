
use super::*;

#[test]
fn unresolved_retained_capture_has_explicit_contract_reason() {
    let ctx = LowerCtx::new();
    let violation =
        retained_capture_violation("unresolved", &Type::Unknown, &ctx, &mut HashSet::new())
            .expect("an unresolved retained capture must be rejected");

    assert_eq!(violation.path, "unresolved");
    assert!(violation.ty.is_unknown());
    assert_eq!(
        violation.reason,
        "has a type that could not be resolved for retained callback verification"
    );
}
