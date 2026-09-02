use crate::check_and_package_commands::declaration_python_requirements;
use crate::cli_model_and_entrypoint::{
    DiagnosticFormat, EXIT_USER_DIAGNOSTIC, diagnostic_with_code, package_diagnostic,
};
use crate::diagnostic_rendering_and_run::{current_session_package_id, render_diagnostics};
use crate::package_graph_context::load_package_graph_context;
use crate::package_session_cli::package_session_for_cwd;
use sifr_frontend::DiskSourceProvider;
use std::path::PathBuf;

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
    package_python_runtime_from_resolved(graph, package_id, resolved, true, diagnostic_format)
        .map(Some)
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
                true,
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
    load_certifications: bool,
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
    let digest = match sifr_package::digest_python_environment_probe(&request, &probe) {
        Ok(digest) => digest.hex,
        Err(error) => {
            return Err(render_python_identity_failure(
                format!("could not serialize the Python environment identity: {error}"),
                diagnostic_format,
            ));
        }
    };
    let mut runtime = sifr_driver::PackagePythonRuntime::from_probe(
        &request,
        &probe,
        digest.clone(),
        resolved.required_imports,
        resolved.trusted_imports,
        resolved.trusted_native_imports,
    )
    .map_err(|error| {
        render_python_identity_failure(
            format!("could not serialize the Python authoring environment identity: {error}"),
            diagnostic_format,
        )
    })?;
    let package_root = graph
        .packages
        .get(package_id)
        .map(|package| package.package_root.as_path());
    if load_certifications {
        if let Some(package_root) = package_root {
            crate::package_python_certifications::load_into_runtime(
                package_root,
                &digest,
                &mut runtime,
                diagnostic_format,
            )?;
            let authoring_digest = runtime.authoring_environment_digest().to_string();
            load_python_bindings_into_runtime(
                package_root,
                &authoring_digest,
                &mut runtime,
                diagnostic_format,
            )?;
        }
    }
    Ok(runtime)
}

fn render_python_identity_failure(reason: String, diagnostic_format: DiagnosticFormat) -> i32 {
    render_diagnostics(
        &[diagnostic_with_code(
            reason,
            sifr_diagnostics::DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
        )],
        diagnostic_format,
    );
    EXIT_USER_DIAGNOSTIC
}

fn load_python_bindings_into_runtime(
    package_root: &std::path::Path,
    environment_digest: &str,
    runtime: &mut sifr_driver::PackagePythonRuntime,
    diagnostic_format: DiagnosticFormat,
) -> Result<(), i32> {
    let artifact_path = package_root.join(sifr_package::PYTHON_BINDINGS_FILE);
    if !artifact_path.is_file() {
        return Ok(());
    }
    match sifr_package::load_python_bindings(package_root, environment_digest) {
        Ok(artifact) => {
            if let Err(reason) = sifr_driver::validate_binding_distributions(runtime, &artifact) {
                render_diagnostics(
                    &[binding_diagnostic(format!(
                        "invalid Python binding artifact: {reason}"
                    ))],
                    diagnostic_format,
                );
                return Err(EXIT_USER_DIAGNOSTIC);
            }
            let identity = serde_json::to_string(&artifact.bindings).map_err(|error| {
                render_diagnostics(
                    &[binding_diagnostic(format!(
                        "could not fingerprint Python bindings: {error}"
                    ))],
                    diagnostic_format,
                );
                EXIT_USER_DIAGNOSTIC
            })?;
            runtime.set_binding_identity(identity);
            Ok(())
        }
        Err(reason) => {
            render_diagnostics(
                &[binding_diagnostic(format!(
                    "invalid Python binding artifact: {reason}"
                ))],
                diagnostic_format,
            );
            Err(EXIT_USER_DIAGNOSTIC)
        }
    }
}

pub(super) struct PythonAuthoringContext {
    pub package_root: PathBuf,
    pub runtime: sifr_driver::PackagePythonRuntime,
}

pub(super) fn package_python_authoring_context(
    lock_mode: sifr_package::CargoLockMode,
    additional_import_roots: &[String],
    diagnostic_format: DiagnosticFormat,
) -> Result<PythonAuthoringContext, i32> {
    let mut provider = DiskSourceProvider::new();
    let session = package_session_for_cwd(lock_mode, &mut provider).map_err(|error| {
        render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
        crate::cli_model_and_entrypoint::EXIT_USAGE_OR_CONFIG
    })?;
    if session.manifest_less_mode {
        render_diagnostics(
            &[binding_diagnostic(
                "Python authoring commands require a Sifr package",
            )],
            diagnostic_format,
        );
        return Err(crate::cli_model_and_entrypoint::EXIT_USAGE_OR_CONFIG);
    }
    let graph_context =
        load_package_graph_context(&session, lock_mode, diagnostic_format, &mut provider)?
            .ok_or(crate::cli_model_and_entrypoint::EXIT_USAGE_OR_CONFIG)?;
    let package_id = current_session_package_id(&session, &graph_context.graph)
        .ok_or(crate::cli_model_and_entrypoint::EXIT_USAGE_OR_CONFIG)?;
    let package = graph_context
        .graph
        .packages
        .get(&package_id)
        .ok_or(crate::cli_model_and_entrypoint::EXIT_USAGE_OR_CONFIG)?;
    let mut requirements =
        declaration_python_requirements(&graph_context.source_map, None, &mut provider);
    requirements.extend(additional_import_roots.iter().map(|root| {
        sifr_package::PythonRequirementContribution {
            root: root.clone(),
            package_id: package_id.clone(),
            kind: sifr_package::PythonRequirementKind::Declaration,
            source: "sifr python authoring request".to_string(),
        }
    }));
    let bridge_graph = sifr_package::resolve_python_bridge_graph(&graph_context.graph, &package_id)
        .map_err(|errors| {
            render_diagnostics(
                &errors
                    .into_iter()
                    .map(package_diagnostic)
                    .collect::<Vec<_>>(),
                diagnostic_format,
            );
            EXIT_USER_DIAGNOSTIC
        })?;
    requirements.extend(bridge_graph.requirements);
    requirements.sort();
    requirements.dedup();
    let resolved = sifr_package::resolve_python_environment_with_requirements(
        &graph_context.graph,
        &package_id,
        &requirements,
    )
    .map_err(|errors| {
        render_diagnostics(
            &errors
                .into_iter()
                .map(package_diagnostic)
                .collect::<Vec<_>>(),
            diagnostic_format,
        );
        EXIT_USER_DIAGNOSTIC
    })?
    .ok_or_else(|| {
        render_diagnostics(
            &[binding_diagnostic(
                "Python authoring requires a selected package Python environment",
            )],
            diagnostic_format,
        );
        crate::cli_model_and_entrypoint::EXIT_USER_DIAGNOSTIC
    })?;
    let runtime = package_python_runtime_from_resolved(
        &graph_context.graph,
        &package_id,
        resolved,
        false,
        diagnostic_format,
    )?;
    Ok(PythonAuthoringContext {
        package_root: package.package_root.clone(),
        runtime,
    })
}

fn binding_diagnostic(message: impl Into<String>) -> sifr_diagnostics::RenderedDiagnostic {
    diagnostic_with_code(
        message,
        sifr_diagnostics::DiagnosticCode::PYCONV_UNSUPPORTED_DECLARATION_TYPE,
    )
}
