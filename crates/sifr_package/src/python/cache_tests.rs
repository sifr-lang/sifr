use super::test_support::{request, valid_probe};
use crate::digest_python_environment_probe;

#[test]
fn python_probe_digest_includes_canonical_required_roots() {
    let mut first = request();
    first.required_imports = vec!["numpy".to_string()];
    let mut second = first.clone();
    second.required_imports = vec!["pandas".to_string()];
    let probe = valid_probe();

    assert_ne!(
        digest_python_environment_probe(&first, &probe),
        digest_python_environment_probe(&second, &probe),
        "derived canonical roots must participate in Python build cache identity"
    );
}
