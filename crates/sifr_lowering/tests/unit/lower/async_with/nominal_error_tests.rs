use super::*;

#[test]
fn task_scope_failure_matches_builtin_and_rejects_user_shadow() {
    let produced = scope_failure_type();
    let Type::Class { identity, .. } = &produced else {
        panic!("scope failure class")
    };
    assert_eq!(identity.as_deref(), Some("sifr.builtin.ScopeFailure"));
    let mut shadow = produced.clone();
    if let Type::Class { identity, .. } = &mut shadow {
        *identity = Some("user.ScopeFailure".to_string());
    }
    assert!(!produced.is_assignable_to(&shadow));
    assert_eq!(
        sifr_type_system::class_rust_name(identity.as_deref(), "ScopeFailure"),
        "ScopeFailure"
    );
}
