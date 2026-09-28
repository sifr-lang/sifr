use super::package_diagnostics::render_package_diagnostic;
use super::{
    PythonAuthoringCancelled, PythonAuthoringRuntime, validate_certification_distributions,
};
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, RenderedDiagnostic};
use sifr_frontend::SourceProvider;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct PythonEditorEnvironment {
    pub runtime: Option<PythonAuthoringRuntime>,
    pub diagnostics: Vec<RenderedDiagnostic>,
}

pub enum PythonEditorResolutionError {
    Diagnostics(Vec<RenderedDiagnostic>),
    Cancelled,
}

impl From<Vec<RenderedDiagnostic>> for PythonEditorResolutionError {
    fn from(diagnostics: Vec<RenderedDiagnostic>) -> Self {
        Self::Diagnostics(diagnostics)
    }
}

fn check_cancelled(cancelled: &impl Fn() -> bool) -> Result<(), PythonEditorResolutionError> {
    if cancelled() {
        Err(PythonEditorResolutionError::Cancelled)
    } else {
        Ok(())
    }
}

pub fn resolve_editor_environment(
    package_root: &Path,
    required_import_roots: &[String],
    provider: &mut impl SourceProvider,
    cancelled: impl Fn() -> bool,
) -> Result<PythonEditorEnvironment, PythonAuthoringCancelled> {
    if cancelled() {
        return Err(PythonAuthoringCancelled);
    }
    if !provider.is_file(&package_root.join("Cargo.toml")) {
        return Ok(PythonEditorEnvironment {
            runtime: None,
            diagnostics: Vec::new(),
        });
    }
    match resolve_package_python_environment_inner(
        package_root,
        required_import_roots,
        provider,
        &cancelled,
    ) {
        Ok(snapshot) => Ok(snapshot),
        Err(PythonEditorResolutionError::Diagnostics(diagnostics)) => Ok(PythonEditorEnvironment {
            runtime: None,
            diagnostics,
        }),
        Err(PythonEditorResolutionError::Cancelled) => Err(PythonAuthoringCancelled),
    }
}

fn resolve_package_python_environment_inner(
    package_root: &Path,
    required_import_roots: &[String],
    provider: &mut impl SourceProvider,
    cancelled: &impl Fn() -> bool,
) -> Result<PythonEditorEnvironment, PythonEditorResolutionError> {
    check_cancelled(cancelled)?;
    let session = sifr_package::PackageSession::discover(
        sifr_package::PackageSessionOptions {
            current_dir: package_root.to_path_buf(),
            lock_mode: sifr_package::CargoLockMode::Frozen,
        },
        provider,
    )
    .map_err(|error| vec![render_package_diagnostic(error)])?;
    check_cancelled(cancelled)?;
    let snapshot = match sifr_package::load_package_graph_snapshot(
        &session.workspace_root,
        sifr_package::CargoLockMode::Frozen,
        provider,
    ) {
        Ok(snapshot) => snapshot,
        Err(failure) if missing_lockfile_frozen_failure(&failure) => {
            return Ok(PythonEditorEnvironment {
                runtime: None,
                diagnostics: Vec::new(),
            });
        }
        Err(failure) => return Err(render_package_diagnostics(failure.into_diagnostics()).into()),
    };
    resolve_editor_environment_from_snapshot(
        package_root,
        &session,
        &snapshot,
        required_import_roots,
        cancelled,
    )
}

/// Resolve editor status from the caller-supplied, authoritative package graph.
pub fn resolve_editor_environment_from_snapshot(
    package_root: &Path,
    session: &sifr_package::PackageSession,
    snapshot: &sifr_package::PackageGraphSnapshot,
    required_import_roots: &[String],
    cancelled: &impl Fn() -> bool,
) -> Result<PythonEditorEnvironment, PythonEditorResolutionError> {
    check_cancelled(cancelled)?;
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
        .map_err(|error| vec![render_package_diagnostic(error)])?
        .is_empty();
    let resolution = sifr_package::resolve_python_environment_for_check(
        &snapshot.graph,
        &package_id,
        &requirements,
        allow_deferral,
    )
    .map_err(render_package_diagnostics)?;
    let sifr_package::PythonEnvironmentResolution::Resolved(resolved) = resolution else {
        return Ok(PythonEditorEnvironment {
            runtime: None,
            diagnostics: Vec::new(),
        });
    };
    let request = sifr_package::PythonEnvironmentProbeRequest::from(&resolved);
    check_cancelled(cancelled)?;
    let probe = sifr_package::probe_python_environment(&request)
        .map_err(|error| vec![render_package_diagnostic(error)])?;
    let mut runtime = PythonAuthoringRuntime::from_probe(&request, &probe).map_err(|error| {
        vec![diagnostic_with_code(
            DiagnosticCode::PYENV_PROBE_FAILED,
            error,
            "check Python environment paths".to_string(),
        )]
    })?;
    let mut diagnostics = Vec::new();
    check_cancelled(cancelled)?;
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
    check_cancelled(cancelled)?;
    let certification_path = package_root.join(sifr_package::PYTHON_CERTIFICATIONS_FILE);
    if certification_path.is_file() {
        match sifr_package::load_python_certifications(package_root, runtime.environment_digest()) {
            Ok(artifact) => match validate_certification_distributions(&runtime, &artifact) {
                Ok(()) => {
                    if let Err(error) = runtime
                        .set_arrow_certifications(artifact.arrow)
                        .and_then(|()| runtime.set_dlpack_certifications(artifact.dlpack))
                    {
                        diagnostics.push(diagnostic_with_code(
                            DiagnosticCode::PYZC_INVALID_DECLARATION,
                            error,
                            "rerun `sifr python certify --check`".to_string(),
                        ));
                    }
                }
                Err(reason) => diagnostics.push(diagnostic_with_code(
                    DiagnosticCode::PYZC_INVALID_DECLARATION,
                    format!("invalid Python certification artifact: {reason}"),
                    "rerun `sifr python certify --check`".to_string(),
                )),
            },
            Err(reason) => diagnostics.push(diagnostic_with_code(
                DiagnosticCode::PYZC_INVALID_DECLARATION,
                format!("invalid Python certification artifact: {reason}"),
                "rerun `sifr python certify --check`".to_string(),
            )),
        }
    }
    check_cancelled(cancelled)?;
    Ok(PythonEditorEnvironment {
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
        .map(render_package_diagnostic)
        .collect()
}

pub fn python_environment_selection(
    root: &Path,
    provider: &mut impl SourceProvider,
) -> Option<sifr_package::PythonEnvironmentSelection> {
    let session = sifr_package::PackageSession::discover(
        sifr_package::PackageSessionOptions {
            current_dir: root.to_path_buf(),
            lock_mode: sifr_package::CargoLockMode::Frozen,
        },
        provider,
    )
    .ok()?;
    let manifest = session.manifest?;
    sifr_package::select_root_python_environment(root, &manifest.python)
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
