use crate::session::Session;
use sifr_driver::{ApplicationProfile, CompilerContext};
use sifr_identity::CompilerIdentity;

#[test]
fn production_workspace_preserves_context() {
    // These supplied product identities deliberately differ from each other
    // and from the dependency-bound unit-test identity.
    let a = CompilerContext::new(CompilerIdentity::product(&"a".repeat(64)).unwrap())
        .with_application_profile(ApplicationProfile::Release)
        .with_project_incremental(false);
    let b = CompilerContext::new(CompilerIdentity::product(&"b".repeat(64)).unwrap());
    let first = Session::with_compiler(a.clone());
    let second = Session::with_compiler(b.clone());
    for (session, supplied) in [(&first, &a), (&second, &b)] {
        let actual = session.compiler_context();
        assert_eq!(actual.identity(), supplied.identity());
        assert!(!actual.identity().is_test());
        assert!(actual.shares_metadata_generation(supplied));
        assert_eq!(actual.application_profile(), supplied.application_profile());
        assert_eq!(actual.project_incremental(), supplied.project_incremental());
    }
    assert_ne!(
        first.compiler_context().identity(),
        second.compiler_context().identity()
    );
    assert!(
        !first
            .compiler_context()
            .shares_metadata_generation(second.compiler_context())
    );

    let expected_test =
        CompilerContext::for_test_tokens(crate::compiled_input_tokens(), "sifr_lsp-tests");
    let test_session = Session::new();
    assert!(test_session.compiler_context().identity().is_test());
    assert_eq!(
        test_session.compiler_context().identity(),
        expected_test.identity()
    );
}
