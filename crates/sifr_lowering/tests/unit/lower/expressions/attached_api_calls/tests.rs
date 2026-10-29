use super::provisional_owner_bound_is_deferred;

#[test]
fn provisional_static_owner_bounds_have_exact_parity() {
    assert!(provisional_owner_bound_is_deferred(
        true,
        "T",
        "T",
        "StaticProgram"
    ));
    assert!(provisional_owner_bound_is_deferred(
        true,
        "T",
        "T",
        "MethodSlots"
    ));
    assert!(!provisional_owner_bound_is_deferred(
        true,
        "T",
        "T",
        "Structural"
    ));
    assert!(!provisional_owner_bound_is_deferred(
        true,
        "Context",
        "T",
        "MethodSlots"
    ));
    assert!(!provisional_owner_bound_is_deferred(
        false,
        "T",
        "T",
        "MethodSlots"
    ));
}
