use super::cli_model_and_entrypoint::{
    diagnostic_with_code, DiagnosticFormat, EXIT_USER_DIAGNOSTIC,
};
use super::diagnostic_rendering_and_run::render_diagnostics;
use std::path::Path;
use std::process::Command;

pub(super) fn load_into_runtime(
    package_root: &Path,
    environment_digest: &str,
    runtime: &mut sifr_driver::PackagePythonRuntime,
    diagnostic_format: DiagnosticFormat,
) -> Result<(), i32> {
    let artifact_path = package_root.join(sifr_package::PYTHON_CERTIFICATIONS_FILE);
    if !artifact_path.is_file() {
        return Ok(());
    }
    match sifr_package::load_python_certifications(package_root, environment_digest) {
        Ok(artifact) => {
            if let Err(reason) = validate_installed_distributions(
                runtime,
                artifact
                    .arrow
                    .iter()
                    .flat_map(|certification| certification.distributions.iter())
                    .chain(
                        artifact
                            .dlpack
                            .iter()
                            .flat_map(|certification| certification.distributions.iter()),
                    ),
            ) {
                render_diagnostics(
                    &[diagnostic_with_code(
                        format!("invalid Python certification artifact: {reason}"),
                        sifr_diagnostics::DiagnosticCode::PYZC_INVALID_DECLARATION,
                    )],
                    diagnostic_format,
                );
                return Err(EXIT_USER_DIAGNOSTIC);
            }
            runtime.set_arrow_certifications(artifact.arrow);
            runtime.set_dlpack_certifications(artifact.dlpack);
            Ok(())
        }
        Err(reason) => {
            render_diagnostics(
                &[diagnostic_with_code(
                    format!("invalid Python certification artifact: {reason}"),
                    sifr_diagnostics::DiagnosticCode::PYZC_INVALID_DECLARATION,
                )],
                diagnostic_format,
            );
            Err(EXIT_USER_DIAGNOSTIC)
        }
    }
}

fn validate_installed_distributions<'a>(
    runtime: &sifr_driver::PackagePythonRuntime,
    distributions: impl IntoIterator<Item = &'a sifr_package::ArrowCertifiedDistribution>,
) -> Result<(), String> {
    let expected = distributions
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    validate_distribution_versions(expected, |distribution| {
        let output = Command::new(runtime.interpreter())
            .args([
                "-I",
                "-c",
                "import importlib.metadata,sys; print(importlib.metadata.version(sys.argv[1]))",
                &distribution.name,
            ])
            .output()
            .map_err(|error| {
                format!(
                    "could not inspect certified Python distribution '{}': {error}",
                    distribution.name
                )
            })?;
        if !output.status.success() {
            return Err(format!(
                "could not inspect certified Python distribution '{}'",
                distribution.name
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    })
}

fn validate_distribution_versions<'a>(
    expected: impl IntoIterator<Item = &'a sifr_package::ArrowCertifiedDistribution>,
    mut installed_version: impl FnMut(
        &sifr_package::ArrowCertifiedDistribution,
    ) -> Result<String, String>,
) -> Result<(), String> {
    for distribution in expected {
        if installed_version(distribution)? != distribution.version {
            return Err(format!(
                "certified Python distribution '{}=={}' does not match the selected environment",
                distribution.name, distribution.version
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_distribution_versions;
    use sifr_package::ArrowCertifiedDistribution;

    #[test]
    fn certified_distribution_versions_fail_closed_on_drift_and_probe_failure() {
        let expected = [ArrowCertifiedDistribution {
            name: "pyarrow".to_string(),
            version: "22.0.0".to_string(),
        }];
        validate_distribution_versions(&expected, |_| Ok("22.0.0".to_string()))
            .expect("matching installed version should pass");
        assert!(
            validate_distribution_versions(&expected, |_| Ok("23.0.0".to_string()))
                .expect_err("version drift must fail")
                .contains("does not match")
        );
        assert_eq!(
            validate_distribution_versions(&expected, |_| Err("probe failed".to_string()))
                .expect_err("probe failure must fail"),
            "probe failed"
        );
    }
}
