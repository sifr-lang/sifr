use super::*;
#[test]
fn identity_encoding_is_unambiguous_and_domain_separated() {
    let a = CompilerBuildId::from_records([("ab", b"c".as_slice())]);
    let b = CompilerBuildId::from_records([("a", b"bc".as_slice())]);
    assert_ne!(a, b);
    assert_ne!(
        a.as_str(),
        MetadataId::from_records([("ab", b"c".as_slice())]).as_str()
    );
}
#[test]
fn compiled_test_tokens_are_order_independent_and_overrides_fail_closed() {
    let a = CompilerIdentity::for_test(vec![("b", "2"), ("a", "1")], "unit");
    assert_eq!(
        a,
        CompilerIdentity::for_test(vec![("a", "1"), ("b", "2")], "unit")
    );
    assert!(
        a.validate_override(&CompilerIdentity::for_test(vec![("a", "changed")], "unit"))
            .is_err()
    );
    assert_ne!(
        a,
        CompilerIdentity::for_test(vec![("b", "2"), ("a", "1")], "integration")
    );
    assert!(
        a.validate_override(&CompilerIdentity::product(a.as_str()).unwrap())
            .is_err()
    );
}
