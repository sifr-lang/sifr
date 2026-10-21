use super::super::frontend::FrontendCompiled;
use super::super::project::ProjectLowering;
use super::super::stdlib::StdlibCompiled;
use crate::diagnostics::RenderedDiagnostic;
pub(super) fn into_frontend(
    stdlib: std::sync::Arc<StdlibCompiled>,
    project_lowering: ProjectLowering,
) -> Result<FrontendCompiled, Vec<RenderedDiagnostic>> {
    let lowering_result = sifr_semantic_services::single_file::into_lowering(project_lowering)?;
    Ok(FrontendCompiled {
        stdlib,
        lowering_result,
    })
}
