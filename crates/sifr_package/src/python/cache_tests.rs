use super::test_support::{request, valid_probe};
use crate::{
    digest_package_build_cache_inputs, digest_python_environment_probe, PackageBuildCacheInputs,
    PythonDistributionProbe,
};

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

#[test]
fn python_probe_digest_includes_resolved_distribution_versions_and_abi() {
    let request = request();
    let mut first = valid_probe();
    first.imports[0].distributions = vec![PythonDistributionProbe {
        name: "demo-dist".to_string(),
        version: "1.0.0".to_string(),
    }];
    let mut distribution_changed = first.clone();
    distribution_changed.imports[0].distributions[0].version = "1.0.1".to_string();
    let mut abi_changed = first.clone();
    abi_changed.soabi = Some("cpython-314-darwin".to_string());

    assert_ne!(
        digest_python_environment_probe(&request, &first),
        digest_python_environment_probe(&request, &distribution_changed)
    );
    assert_ne!(
        digest_python_environment_probe(&request, &first),
        digest_python_environment_probe(&request, &abi_changed)
    );
    let package_key = |probe| {
        let python_probe_digest = digest_python_environment_probe(&request, probe).hex;
        digest_package_build_cache_inputs(&PackageBuildCacheInputs {
            python_probe_digest: Some(python_probe_digest),
            ..PackageBuildCacheInputs::default()
        })
    };
    assert_ne!(package_key(&first), package_key(&distribution_changed));
    assert_ne!(package_key(&first), package_key(&abi_changed));
}
