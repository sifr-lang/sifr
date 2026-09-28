use super::project_codegen::GeneratedBinaryProject;
use super::python_runtime::PackagePythonRuntime;
use crate::diagnostics::diagnostic_with_code;
#[cfg(test)]
use sifr_compiler_services::python::validate_signature;
pub use sifr_compiler_services::python::{
    PythonInteropPlanDiagnostic, PythonTargetInspection, PythonTargetParameter,
    apply_python_target_inspection, inspect_python_target, probe_python_interop_plan,
};
use sifr_compiler_services::python::{
    mark_embedded_bridge_targets, validate_protocol_certifications_for_plan,
};
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};

pub(super) fn apply_python_interop_metadata(
    generated: GeneratedBinaryProject,
    runtime: Option<&PackagePythonRuntime>,
) -> Result<GeneratedBinaryProject, Vec<RenderedDiagnostic>> {
    apply_python_interop_metadata_with_policy(generated, runtime, false)
}

pub(super) fn apply_python_interop_metadata_for_check(
    generated: GeneratedBinaryProject,
    runtime: Option<&PackagePythonRuntime>,
) -> Result<GeneratedBinaryProject, Vec<RenderedDiagnostic>> {
    apply_python_interop_metadata_with_policy(generated, runtime, true)
}

fn apply_python_interop_metadata_with_policy(
    mut generated: GeneratedBinaryProject,
    runtime: Option<&PackagePythonRuntime>,
    allow_deferred: bool,
) -> Result<GeneratedBinaryProject, Vec<RenderedDiagnostic>> {
    if generated.interop.python.declarations.is_empty() {
        return Ok(generated);
    }
    let Some(runtime) = runtime else {
        mark_embedded_bridge_targets(&mut generated.interop.python);
        if allow_deferred {
            return Ok(generated);
        }
        return Err(vec![diagnostic_with_code(
            "Python declarations require a root-selected Python environment",
            DiagnosticCode::PYENV_MISSING_SELECTION,
        )]);
    };

    let mut diagnostics =
        validate_protocol_certifications_for_plan(&generated.interop.python, runtime)
            .into_iter()
            .map(|diagnostic| diagnostic.diagnostic)
            .collect::<Vec<_>>();
    diagnostics.extend(probe_python_interop_plan(
        &mut generated.interop.python,
        runtime.interpreter(),
    ));
    if diagnostics.is_empty() {
        Ok(generated)
    } else {
        Err(diagnostics)
    }
}

#[cfg(test)]
#[path = "python_interop_tests.rs"]
mod tests;
