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
            if let Err(reason) = validate_installed_distributions(runtime, &artifact.arrow) {
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

fn validate_installed_distributions(
    runtime: &sifr_driver::PackagePythonRuntime,
    certifications: &[sifr_package::ArrowCertification],
) -> Result<(), String> {
    let expected = certifications
        .iter()
        .flat_map(|certification| certification.distributions.iter())
        .collect::<std::collections::BTreeSet<_>>();
    for distribution in expected {
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
        let installed = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !output.status.success() || installed != distribution.version {
            return Err(format!(
                "certified Python distribution '{}=={}' does not match the selected environment",
                distribution.name, distribution.version
            ));
        }
    }
    Ok(())
}
