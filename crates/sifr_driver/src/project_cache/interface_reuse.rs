use sifr_frontend::{
    FrontendContext, ModuleCheckDecision, ProjectRoot, SourcePath,
    persistence::{CapturingSourceProvider, CompletedCheck, SemanticInputs},
};
use std::path::Path;

/// The initial proven boundary supports ordinary manifestless source graphs.
/// Package, SQL, Python and component authorities continue through their owners.
pub(super) fn restore(
    record: &CompletedCheck,
    file: &Path,
    inputs: &SemanticInputs,
    provider: &mut CapturingSourceProvider<'_>,
    compiler: &sifr_identity::CompilerIdentity,
    defs: sifr_lowering::ExternalDefs,
) -> Option<Vec<ModuleCheckDecision>> {
    if inputs.package_and_lock != "manifestless-owner-v1"
        || &record.result.inputs.semantic_inputs != inputs
        || record.result.inputs.source.path != file
        || record.result.resolution.ready()?.sources.len() < 2
    {
        return None;
    }
    let root = ProjectRoot {
        root: SourcePath::new(file.parent()?.to_owned()),
        entrypoint: SourcePath::new(file.to_owned()),
    };
    let sources = record
        .result
        .resolution
        .ready()?
        .sources
        .iter()
        .map(|source| {
            use sifr_frontend::SourceProvider;
            Some(sifr_frontend::persistence::CapturedSource {
                path: source.path.clone(),
                text: provider.read_file(&source.path).ok()?.as_str().to_owned(),
            })
        })
        .collect::<Option<Vec<_>>>()?;
    let mut frontend = FrontendContext::from_resolved_sources(root, sources, defs)?
        .with_compiler_identity(compiler.clone());
    let modules = frontend.restore_completed_checks(record, inputs, provider, true)?;
    modules
        .iter()
        .any(|module| module.action == "restored")
        .then_some(modules)
}
