use crate::cli_model_and_entrypoint::{
    diagnostic_with_code, DiagnosticFormat, EXIT_SUCCESS, EXIT_USER_DIAGNOSTIC,
};
use crate::diagnostic_rendering_and_run::render_diagnostics;
use crate::python_cli::{installed_distribution_version, validated_fixture, CertificationContext};
use serde::Deserialize;
use std::io::{self, Write as _};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct DlpackFixtureEvidence {
    target: String,
    producer_module: String,
    producer_type: String,
    distributions: Vec<sifr_package::ArrowCertifiedDistribution>,
    device: sifr_package::DlpackCertifiedDevice,
    stream_policy: sifr_package::DlpackCertifiedStreamPolicy,
    pointer_identity_verified: bool,
    exact_deleter_count: u64,
    copy_performed: bool,
    within_run_assertions: bool,
}

pub(crate) fn certify_dlpack(
    context: &CertificationContext,
    target: &str,
    fixture: &Path,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let (fixture, relative) = match validated_fixture(context, fixture) {
        Ok(value) => value,
        Err(reason) => return fail(reason, diagnostic_format),
    };
    let evidence = match run_fixture(context, &fixture, target) {
        Ok(evidence) => evidence,
        Err(reason) => return fail(reason, diagnostic_format),
    };
    if let Err(reason) = validate_evidence(context, target, &evidence) {
        return fail(reason, diagnostic_format);
    }
    let fixture_digest = match sifr_package::arrow_fixture_digest(&fixture) {
        Ok(digest) => digest,
        Err(reason) => return fail(reason, diagnostic_format),
    };
    let certification = sifr_package::DlpackCertification {
        target: target.to_string(),
        fixture: relative.to_string_lossy().replace('\\', "/"),
        fixture_digest,
        producer_module: evidence.producer_module,
        producer_type: evidence.producer_type,
        distributions: evidence.distributions,
        device: evidence.device,
        stream_policy: evidence.stream_policy,
        pointer_identity_verified: evidence.pointer_identity_verified,
        exact_deleter_count: evidence.exact_deleter_count,
        copy_performed: evidence.copy_performed,
        within_run_assertions: evidence.within_run_assertions,
    };
    let artifact_path = context
        .package_root
        .join(sifr_package::PYTHON_CERTIFICATIONS_FILE);
    let mut artifact = if artifact_path.is_file() {
        match sifr_package::load_python_certifications_for_dlpack_update(
            &context.package_root,
            &context.environment_digest,
            target,
        ) {
            Ok(artifact) => artifact,
            Err(reason) => return fail(reason, diagnostic_format),
        }
    } else {
        sifr_package::PythonCertificationArtifact {
            schema_version: sifr_package::PYTHON_CERTIFICATION_SCHEMA_VERSION,
            environment_digest: context.environment_digest.clone(),
            arrow: Vec::new(),
            dlpack: Vec::new(),
        }
    };
    artifact.dlpack.retain(|existing| existing.target != target);
    artifact.dlpack.push(certification);
    artifact
        .dlpack
        .sort_by(|left, right| left.target.cmp(&right.target));
    match sifr_package::write_python_certifications(&context.package_root, &artifact) {
        Ok(path) => {
            let _ = writeln!(
                io::stdout(),
                "certified DLPack target '{target}' in {}",
                path.display()
            );
            EXIT_SUCCESS
        }
        Err(reason) => fail(reason, diagnostic_format),
    }
}

pub(crate) fn check_dlpack_certification(
    context: &CertificationContext,
    certification: &sifr_package::DlpackCertification,
) -> Result<(), String> {
    let fixture = sifr_package::arrow_fixture_path(&context.package_root, &certification.fixture)?;
    let evidence = run_fixture(context, &fixture, &certification.target)?;
    validate_evidence(context, &certification.target, &evidence)?;
    if evidence.target != certification.target
        || evidence.producer_module != certification.producer_module
        || evidence.producer_type != certification.producer_type
        || evidence.distributions != certification.distributions
        || evidence.device != certification.device
        || evidence.stream_policy != certification.stream_policy
        || evidence.pointer_identity_verified != certification.pointer_identity_verified
        || evidence.exact_deleter_count != certification.exact_deleter_count
        || evidence.copy_performed != certification.copy_performed
        || evidence.within_run_assertions != certification.within_run_assertions
    {
        return Err(format!(
            "DLPack certification evidence changed for '{}'",
            certification.target
        ));
    }
    Ok(())
}

fn run_fixture(
    context: &CertificationContext,
    fixture: &Path,
    target: &str,
) -> Result<DlpackFixtureEvidence, String> {
    let output = Command::new(&context.interpreter)
        .arg("-I")
        .arg(fixture)
        .arg(target)
        .current_dir(&context.package_root)
        .output()
        .map_err(|error| format!("could not execute DLPack certification fixture: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "DLPack certification fixture failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("DLPack certification fixture returned invalid JSON: {error}"))
}

fn validate_evidence(
    context: &CertificationContext,
    target: &str,
    evidence: &DlpackFixtureEvidence,
) -> Result<(), String> {
    if target.trim().is_empty()
        || evidence.target != target
        || evidence.producer_module.trim().is_empty()
        || evidence.producer_type.trim().is_empty()
    {
        return Err("DLPack certification identities must be non-empty".to_string());
    }
    if evidence.copy_performed
        || !evidence.pointer_identity_verified
        || !evidence.within_run_assertions
    {
        return Err(format!(
            "DLPack fixture for '{target}' did not prove a within-run no-copy transfer"
        ));
    }
    if evidence.exact_deleter_count != 1 {
        return Err(format!(
            "DLPack fixture for '{target}' did not prove exactly one deleter call"
        ));
    }
    if evidence.distributions.is_empty() {
        return Err(format!(
            "DLPack fixture for '{target}' did not report an exact distribution"
        ));
    }
    for distribution in &evidence.distributions {
        let installed = installed_distribution_version(context, &distribution.name)?;
        if installed != distribution.version {
            return Err(format!(
                "DLPack fixture for '{target}' reported distribution '{}=={}', but the selected environment contains '{}=={}'",
                distribution.name, distribution.version, distribution.name, installed
            ));
        }
    }
    Ok(())
}

fn fail(reason: impl Into<String>, diagnostic_format: DiagnosticFormat) -> i32 {
    render_diagnostics(
        &[diagnostic_with_code(
            reason,
            sifr_diagnostics::DiagnosticCode::PYZC_INVALID_DECLARATION,
        )],
        diagnostic_format,
    );
    EXIT_USER_DIAGNOSTIC
}
