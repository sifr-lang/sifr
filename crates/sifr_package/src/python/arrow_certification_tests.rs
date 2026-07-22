use super::arrow_certification::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn artifact_binds_environment_fixture_and_no_copy_evidence() {
    let root = temp_root("valid");
    fs::create_dir_all(root.join("fixtures")).expect("fixture directory");
    let fixture = root.join("fixtures/arrow.py");
    fs::write(&fixture, "print('evidence')\n").expect("fixture");
    let artifact = artifact(&fixture);

    let path = write_python_certifications(&root, &artifact).expect("artifact should write");
    assert_eq!(path, root.join(PYTHON_CERTIFICATIONS_FILE));
    assert_eq!(
        load_python_certifications(&root, "environment-a").expect("artifact should load"),
        artifact
    );
    assert_eq!(
        required_python_certification_archive_entries(&root),
        std::collections::BTreeSet::from([
            PathBuf::from(PYTHON_CERTIFICATIONS_FILE),
            PathBuf::from("fixtures/arrow.py"),
        ])
    );
    assert!(load_python_certifications(&root, "environment-b")
        .expect_err("environment mismatch must fail")
        .contains("environment digest"));

    fs::write(&fixture, "print('changed')\n").expect("fixture mutation");
    assert!(load_python_certifications(&root, "environment-a")
        .expect_err("stale fixture must fail")
        .contains("fixture digest is stale"));
    assert!(
        load_python_certifications_for_update(&root, "environment-a", "pkg.make_array")
            .expect("the target being recertified may have a stale fixture")
            .arrow
            .is_empty()
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn artifact_rejects_uncertain_copy_and_escaping_fixture() {
    let root = temp_root("invalid");
    fs::create_dir_all(root.join("fixtures")).expect("fixture directory");
    let fixture = root.join("fixtures/arrow.py");
    fs::write(&fixture, "print('evidence')\n").expect("fixture");
    let mut artifact = artifact(&fixture);
    artifact.arrow[0].copy_performed = true;
    assert!(
        validate_python_certifications(&root, "environment-a", &artifact)
            .expect_err("copy evidence must fail")
            .contains("no-copy")
    );
    assert!(safe_fixture_path(&root, "../outside.py").is_err());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn dlpack_artifact_requires_within_run_identity_and_exact_deleter() {
    let root = temp_root("dlpack");
    fs::create_dir_all(root.join("fixtures")).expect("fixture directory");
    let fixture = root.join("fixtures/dlpack.py");
    fs::write(&fixture, "print('dlpack evidence')\n").expect("fixture");
    let mut artifact = PythonCertificationArtifact {
        schema_version: PYTHON_CERTIFICATION_SCHEMA_VERSION,
        environment_digest: "environment-a".to_string(),
        arrow: Vec::new(),
        dlpack: vec![DlpackCertification {
            target: "torch.Tensor".to_string(),
            fixture: "fixtures/dlpack.py".to_string(),
            fixture_digest: fixture_digest(&fixture).expect("digest"),
            producer_module: "torch".to_string(),
            producer_type: "Tensor".to_string(),
            distributions: vec![ArrowCertifiedDistribution {
                name: "torch".to_string(),
                version: "2.12.0".to_string(),
            }],
            device: DlpackCertifiedDevice::Cpu,
            stream_policy: DlpackCertifiedStreamPolicy::None,
            pointer_identity_verified: true,
            exact_deleter_count: 1,
            copy_performed: false,
            within_run_assertions: true,
        }],
    };
    write_python_certifications(&root, &artifact).expect("DLPack artifact should write");
    assert!(required_python_certification_archive_entries(&root)
        .contains(&PathBuf::from("fixtures/dlpack.py")));

    artifact.dlpack[0].within_run_assertions = false;
    assert!(
        validate_python_certifications(&root, "environment-a", &artifact)
            .expect_err("cross-run address evidence must fail")
            .contains("within-run")
    );
    artifact.dlpack[0].within_run_assertions = true;
    artifact.dlpack[0].exact_deleter_count = 2;
    assert!(
        validate_python_certifications(&root, "environment-a", &artifact)
            .expect_err("multiple deleter calls must fail")
            .contains("exactly one deleter")
    );
    fs::remove_dir_all(root).expect("cleanup");
}

fn artifact(fixture: &Path) -> PythonCertificationArtifact {
    PythonCertificationArtifact {
        schema_version: ARROW_CERTIFICATION_SCHEMA_VERSION,
        environment_digest: "environment-a".to_string(),
        arrow: vec![ArrowCertification {
            target: "pkg.make_array".to_string(),
            kind: ArrowCertifiedKind::Array,
            fixture: "fixtures/arrow.py".to_string(),
            fixture_digest: fixture_digest(fixture).expect("digest"),
            producer_module: "pyarrow.lib".to_string(),
            producer_type: "Int64Array".to_string(),
            distributions: vec![ArrowCertifiedDistribution {
                name: "pyarrow".to_string(),
                version: "22.0.0".to_string(),
            }],
            schema_mode: ArrowCertifiedSchemaMode::Omitted,
            identity_method: ArrowCertifiedIdentityMethod::BufferAddress,
            pointer_identity_verified: true,
            exact_release_count: 1,
            copy_performed: false,
        }],
        dlpack: Vec::new(),
    }
}

fn temp_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "sifr-arrow-certification-{label}-{}-{nonce}",
        std::process::id()
    ))
}
