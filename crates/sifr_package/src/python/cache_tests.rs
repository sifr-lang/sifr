use super::test_support::{request, valid_probe};
use crate::{
    PackageBuildCacheInputs, PythonDistributionProbe, digest_package_build_cache_inputs,
    digest_python_authoring_environment_probe, digest_python_environment_probe,
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
fn authoring_environment_digest_ignores_entrypoint_import_selection() {
    let mut first_request = request();
    first_request.required_imports = vec!["numpy".to_string()];
    let mut second_request = first_request.clone();
    second_request.required_imports = vec!["pandas".to_string()];
    let mut first_probe = valid_probe();
    first_probe.imports[0].root = "numpy".to_string();
    let mut second_probe = first_probe.clone();
    second_probe.imports[0].root = "pandas".to_string();

    assert_eq!(
        digest_python_authoring_environment_probe(&first_request, &first_probe),
        digest_python_authoring_environment_probe(&second_request, &second_probe),
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
    abi_changed.soabi = Some("cpython-314t-darwin".to_string());

    assert_ne!(
        digest_python_environment_probe(&request, &first),
        digest_python_environment_probe(&request, &distribution_changed)
    );
    assert_ne!(
        digest_python_environment_probe(&request, &first),
        digest_python_environment_probe(&request, &abi_changed)
    );
    let package_key = |probe| {
        let python_probe_digest = digest_python_environment_probe(&request, probe)
            .unwrap()
            .hex;
        digest_package_build_cache_inputs(&PackageBuildCacheInputs {
            python_probe_digest: Some(python_probe_digest),
            ..PackageBuildCacheInputs::default()
        })
    };
    assert_ne!(package_key(&first), package_key(&distribution_changed));
    assert_ne!(package_key(&first), package_key(&abi_changed));
}

#[test]
fn python_probe_identity_binds_each_request_and_observation_field() {
    let request = request();
    let probe = valid_probe();
    let original = digest_python_environment_probe(&request, &probe);
    let check_request = |modify: fn(&mut crate::PythonEnvironmentProbeRequest)| {
        let mut next = request.clone();
        modify(&mut next);
        assert_ne!(original, digest_python_environment_probe(&next, &probe));
    };
    check_request(|x| x.venv_root = "/other".into());
    check_request(|x| x.interpreter = "/other/python".into());
    check_request(|x| x.pyproject = Some("".into()));
    check_request(|x| x.pyproject = Some("pyproject.toml".into()));
    check_request(|x| x.lock = Some("".into()));
    check_request(|x| x.lock = Some("uv.lock".into()));
    check_request(|x| x.required_imports.push("numpy".into()));
    check_request(|x| x.declared_imports.clear());
    check_request(|x| x.native_imports.push("numpy".into()));

    let check_probe = |modify: fn(&mut crate::PythonEnvironmentProbe)| {
        let mut next = probe.clone();
        modify(&mut next);
        assert_ne!(original, digest_python_environment_probe(&request, &next));
    };
    check_probe(|x| x.implementation_name = "PyPy".into());
    check_probe(|x| x.implementation_version = "3.15".into());
    check_probe(|x| x.cpython_version_tuple.push(8));
    check_probe(|x| x.executable = "/other/python".into());
    check_probe(|x| x.sys_prefix = "/other".into());
    check_probe(|x| x.sys_base_prefix = "/other".into());
    check_probe(|x| x.site_packages.push("/other/site".into()));
    check_probe(|x| x.sys_path.push("/other".into()));
    check_probe(|x| x.soabi = None);
    check_probe(|x| x.extension_suffixes.push(".pyd".into()));
    check_probe(|x| x.pointer_width = 32);
    check_probe(|x| x.platform = "linux".into());
    check_probe(|x| x.machine = "x86_64".into());
    check_probe(|x| x.libpython = None);
    check_probe(|x| x.free_threaded = true);
    check_probe(|x| x.imports[0].root = "other".into());
    check_probe(|x| x.imports[0].ok = false);
    check_probe(|x| x.imports[0].origin = Some("/other".into()));
    check_probe(|x| {
        x.imports[0].distributions.push(PythonDistributionProbe {
            name: "pkg".into(),
            version: "1".into(),
        })
    });
    check_probe(|x| x.imports[0].error = Some("error".into()));
    check_probe(|x| x.native_imports.push(x.imports[0].clone()));
    check_probe(|x| x.pyproject_digest = Some("digest".into()));
    check_probe(|x| x.uv_lock_digest = Some("digest".into()));
}

#[test]
fn authoring_probe_domain_is_distinct_from_empty_full_probe() {
    let mut request = request();
    request.required_imports.clear();
    request.declared_imports.clear();
    request.native_imports.clear();
    let mut probe = valid_probe();
    probe.imports.clear();
    probe.native_imports.clear();
    assert_ne!(
        digest_python_authoring_environment_probe(&request, &probe),
        digest_python_environment_probe(&request, &probe),
    );
}
