use super::cli_model_and_entrypoint::{
    DiagnosticFormat, EXIT_USER_DIAGNOSTIC, diagnostic_with_code,
};
use super::diagnostic_rendering_and_run::render_diagnostics;
use std::path::Path;

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
            if let Err(reason) =
                sifr_driver::validate_certification_distributions(runtime, &artifact)
            {
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
