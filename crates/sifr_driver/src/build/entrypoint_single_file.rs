use super::super::frontend::FrontendCompiled;
use super::super::project::ProjectCompilation;
use super::super::stdlib::StdlibCompiled;
use crate::diagnostics::RenderedDiagnostic;

pub(super) fn into_frontend(
    stdlib: StdlibCompiled,
    mut project_lowering: ProjectCompilation,
) -> Result<FrontendCompiled, Vec<RenderedDiagnostic>> {
    let lowering_result = project_lowering
        .lowering_results
        .remove("main")
        .ok_or_else(|| {
            vec![crate::diagnostics::diagnostic_with_code(
                "internal error: frontend compilation missing 'main' lowering result",
                sifr_diagnostics::DiagnosticCode::INTERNAL_COMPILER_PANIC,
            )]
        })?;
    Ok(FrontendCompiled {
        stdlib,
        lowering_result,
    })
}
