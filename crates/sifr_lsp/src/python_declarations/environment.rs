use sifr_analysis::SourceProvider;
use sifr_compiler_services::PackagePythonRuntime;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, RenderedDiagnostic};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Debug)]
pub(super) struct EnvironmentSnapshot {
    pub(super) runtime: Option<PackagePythonRuntime>,
    pub(super) diagnostics: Vec<RenderedDiagnostic>,
}

pub(super) fn resolve_package_python_environment(
    package_root: &Path,
    required_import_roots: &[String],
    provider: &mut impl SourceProvider,
) -> EnvironmentSnapshot {
    if !provider.is_file(&package_root.join("Cargo.toml")) {
        return EnvironmentSnapshot {
            runtime: None,
            diagnostics: Vec::new(),
        };
    }
    match resolve_package_python_environment_inner(package_root, required_import_roots, provider) {
        Ok(snapshot) => snapshot,
        Err(diagnostics) => EnvironmentSnapshot {
            runtime: None,
            diagnostics,
        },
    }
}

fn resolve_package_python_environment_inner(
    package_root: &Path,
    required_import_roots: &[String],
    provider: &mut impl SourceProvider,
) -> Result<EnvironmentSnapshot, Vec<RenderedDiagnostic>> {
    let session = sifr_package::PackageSession::discover(
        sifr_package::PackageSessionOptions {
            current_dir: package_root.to_path_buf(),
            lock_mode: sifr_package::CargoLockMode::Frozen,
        },
        provider,
    )
    .map_err(|error| vec![sifr_compiler_services::render_package_diagnostic(error)])?;
    let snapshot = match sifr_package::load_package_graph_snapshot(
        &session.workspace_root,
        sifr_package::CargoLockMode::Frozen,
        provider,
    ) {
        Ok(snapshot) => snapshot,
        Err(failure) if missing_lockfile_frozen_failure(&failure) => {
            return Ok(EnvironmentSnapshot {
                runtime: None,
                diagnostics: Vec::new(),
            });
        }
        Err(failure) => return Err(render_package_diagnostics(failure.into_diagnostics())),
    };
    let package_id = session.package_id(&snapshot.graph).ok_or_else(|| {
        vec![diagnostic_with_code(
            DiagnosticCode::PACKAGE_METADATA_PARSE,
            "current Sifr package is missing from the Cargo package graph".to_string(),
            "repair Cargo package metadata before requesting Python editor status".to_string(),
        )]
    })?;
    let mut requirements = required_import_roots
        .iter()
        .map(|root| sifr_package::PythonRequirementContribution {
            root: root.clone(),
            package_id: package_id.clone(),
            kind: sifr_package::PythonRequirementKind::Declaration,
            source: "language-server compiler plan".to_string(),
        })
        .collect::<Vec<_>>();
    let bridge = sifr_package::resolve_python_bridge_graph(&snapshot.graph, &package_id)
        .map_err(render_package_diagnostics)?;
    requirements.extend(bridge.requirements);
    let allow_deferral = session
        .runnable_app_paths()
        .map_err(|error| vec![sifr_compiler_services::render_package_diagnostic(error)])?
        .is_empty();
    let resolution = sifr_package::resolve_python_environment_for_check(
        &snapshot.graph,
        &package_id,
        &requirements,
        allow_deferral,
    )
    .map_err(render_package_diagnostics)?;
    let sifr_package::PythonEnvironmentResolution::Resolved(resolved) = resolution else {
        return Ok(EnvironmentSnapshot {
            runtime: None,
            diagnostics: Vec::new(),
        });
    };
    let request = sifr_package::PythonEnvironmentProbeRequest::from(&resolved);
    let probe = sifr_package::probe_python_environment(&request)
        .map_err(|error| vec![sifr_compiler_services::render_package_diagnostic(error)])?;
    let digest = sifr_package::digest_python_environment_probe(&request, &probe)
        .map_err(|error| {
            vec![diagnostic_with_code(
                DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
                format!("could not serialize Python environment identity: {error}"),
                "retry the language-server request".to_string(),
            )]
        })?
        .hex;
    let mut runtime = PackagePythonRuntime::from_probe(
        &request,
        &probe,
        digest.clone(),
        resolved.required_imports,
        resolved.trusted_imports,
        resolved.trusted_native_imports,
    )
    .map_err(|error| {
        vec![diagnostic_with_code(
            DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
            format!("could not serialize Python authoring environment identity: {error}"),
            "retry the language-server request".to_string(),
        )]
    })?;
    let mut diagnostics = Vec::new();
    let binding_path = package_root.join(sifr_package::PYTHON_BINDINGS_FILE);
    if binding_path.is_file() {
        match sifr_package::load_python_bindings(
            package_root,
            runtime.authoring_environment_digest(),
        ) {
            Ok(artifact) => match serde_json::to_string(&artifact.bindings) {
                Ok(identity) => runtime.set_binding_identity(identity),
                Err(error) => diagnostics.push(diagnostic_with_code(
                    DiagnosticCode::PYCONV_UNSUPPORTED_DECLARATION_TYPE,
                    format!("could not fingerprint Python bindings: {error}"),
                    "rerun `sifr python bind --check`".to_string(),
                )),
            },
            Err(reason) => diagnostics.push(diagnostic_with_code(
                DiagnosticCode::PYCONV_UNSUPPORTED_DECLARATION_TYPE,
                format!("invalid Python binding artifact: {reason}"),
                "rerun `sifr python bind --check`".to_string(),
            )),
        }
    }
    let certification_path = package_root.join(sifr_package::PYTHON_CERTIFICATIONS_FILE);
    if certification_path.is_file() {
        match sifr_package::load_python_certifications(package_root, &digest) {
            Ok(artifact) => {
                match sifr_compiler_services::validate_certification_distributions(
                    &runtime, &artifact,
                ) {
                    Ok(()) => {
                        runtime.set_arrow_certifications(artifact.arrow);
                        runtime.set_dlpack_certifications(artifact.dlpack);
                    }
                    Err(reason) => diagnostics.push(diagnostic_with_code(
                        DiagnosticCode::PYZC_INVALID_DECLARATION,
                        format!("invalid Python certification artifact: {reason}"),
                        "rerun `sifr python certify --check`".to_string(),
                    )),
                }
            }
            Err(reason) => diagnostics.push(diagnostic_with_code(
                DiagnosticCode::PYZC_INVALID_DECLARATION,
                format!("invalid Python certification artifact: {reason}"),
                "rerun `sifr python certify --check`".to_string(),
            )),
        }
    }
    Ok(EnvironmentSnapshot {
        runtime: diagnostics.is_empty().then_some(runtime),
        diagnostics,
    })
}

fn missing_lockfile_frozen_failure(failure: &sifr_package::PackageGraphLoadFailure) -> bool {
    if failure.plan.current_dir.join("Cargo.lock").exists() {
        return false;
    }
    let sifr_package::PackageGraphLoadFailureKind::Command { output, .. } = &failure.kind else {
        return false;
    };
    let output = output.to_ascii_lowercase();
    output.contains("lock file")
        && (output.contains("needs to be updated")
            || output.contains("cannot create")
            || output.contains("could not be updated"))
}

fn render_package_diagnostics(
    diagnostics: Vec<sifr_package::PackageDiagnostic>,
) -> Vec<RenderedDiagnostic> {
    diagnostics
        .into_iter()
        .map(sifr_compiler_services::render_package_diagnostic)
        .collect()
}

fn diagnostic_with_code(code: DiagnosticCode, message: String, help: String) -> RenderedDiagnostic {
    let mut args = BTreeMap::new();
    args.insert(
        "message".to_string(),
        DiagnosticArg::String(message.clone()),
    );
    RenderedDiagnostic {
        code: code.code().to_string(),
        severity: code.declared_severity(),
        message,
        message_template: "{message}".to_string(),
        args,
        url: code.docs_url(),
        spans: Vec::new(),
        children: Vec::new(),
        help: Some(help),
        suggestions: Vec::new(),
    }
}
