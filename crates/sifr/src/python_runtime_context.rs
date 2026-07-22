use crate::cli_model_and_entrypoint::{package_diagnostic, DiagnosticFormat, EXIT_USER_DIAGNOSTIC};
use crate::diagnostic_rendering_and_run::render_diagnostics;

pub(super) fn package_python_runtime(
    graph: &sifr_package::SifrPackageGraph,
    package_id: &sifr_package::SifrPackageId,
    derived: &[sifr_package::PythonRequirementContribution],
    diagnostic_format: DiagnosticFormat,
) -> Result<Option<sifr_driver::PackagePythonRuntime>, i32> {
    let resolved = match sifr_package::resolve_python_environment_with_requirements(
        graph, package_id, derived,
    ) {
        Ok(resolved) => resolved,
        Err(errors) => {
            let diagnostics = errors
                .into_iter()
                .map(package_diagnostic)
                .collect::<Vec<_>>();
            render_diagnostics(&diagnostics, diagnostic_format);
            return Err(EXIT_USER_DIAGNOSTIC);
        }
    };
    let Some(resolved) = resolved else {
        return Ok(None);
    };
    package_python_runtime_from_resolved(graph, package_id, resolved, diagnostic_format).map(Some)
}

pub(super) struct PackagePythonCheckRuntime {
    pub runtime: Option<sifr_driver::PackagePythonRuntime>,
    pub deferral: Option<sifr_package::DeferredPythonEnvironment>,
}

pub(super) fn package_python_runtime_for_check(
    graph: &sifr_package::SifrPackageGraph,
    package_id: &sifr_package::SifrPackageId,
    derived: &[sifr_package::PythonRequirementContribution],
    allow_final_application_deferral: bool,
    diagnostic_format: DiagnosticFormat,
) -> Result<PackagePythonCheckRuntime, i32> {
    let resolution = sifr_package::resolve_python_environment_for_check(
        graph,
        package_id,
        derived,
        allow_final_application_deferral,
    )
    .map_err(|errors| {
        let diagnostics = errors
            .into_iter()
            .map(package_diagnostic)
            .collect::<Vec<_>>();
        render_diagnostics(&diagnostics, diagnostic_format);
        EXIT_USER_DIAGNOSTIC
    })?;
    match resolution {
        sifr_package::PythonEnvironmentResolution::NotRequired => Ok(PackagePythonCheckRuntime {
            runtime: None,
            deferral: None,
        }),
        sifr_package::PythonEnvironmentResolution::DeferredToFinalApplication(deferral) => {
            Ok(PackagePythonCheckRuntime {
                runtime: None,
                deferral: Some(deferral),
            })
        }
        sifr_package::PythonEnvironmentResolution::Resolved(resolved) => {
            let runtime = package_python_runtime_from_resolved(
                graph,
                package_id,
                resolved,
                diagnostic_format,
            )?;
            Ok(PackagePythonCheckRuntime {
                runtime: Some(runtime),
                deferral: None,
            })
        }
    }
}

fn package_python_runtime_from_resolved(
    graph: &sifr_package::SifrPackageGraph,
    package_id: &sifr_package::SifrPackageId,
    resolved: sifr_package::ResolvedPythonEnvironment,
    diagnostic_format: DiagnosticFormat,
) -> Result<sifr_driver::PackagePythonRuntime, i32> {
    let request = sifr_package::PythonEnvironmentProbeRequest::from(&resolved);
    let probe = match sifr_package::probe_python_environment(&request) {
        Ok(probe) => probe,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return Err(EXIT_USER_DIAGNOSTIC);
        }
    };
    let digest = sifr_package::digest_python_environment_probe(&request, &probe).hex;
    let mut runtime = sifr_driver::PackagePythonRuntime::from_probe(
        &request,
        &probe,
        digest.clone(),
        resolved.required_imports,
        resolved.trusted_imports,
        resolved.trusted_native_imports,
    );
    let package_root = graph
        .packages
        .get(package_id)
        .map(|package| package.package_root.as_path());
    if let Some(package_root) = package_root {
        crate::package_python_certifications::load_into_runtime(
            package_root,
            &digest,
            &mut runtime,
            diagnostic_format,
        )?;
    }
    Ok(runtime)
}
