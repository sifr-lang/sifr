use sifr_frontend::{
    FrontendContext, ModuleCheckDecision, ProjectRoot, SourcePath,
    persistence::{CapturingSourceProvider, CompletedCheck, SemanticInputs},
};
use std::path::Path;

/// The same narrow body proof applies after current package resolution.
/// SQL, Python and native authorities continue through their live owners.
pub(super) fn restore(
    record: &CompletedCheck,
    file: &Path,
    inputs: &SemanticInputs,
    provider: &mut CapturingSourceProvider<'_>,
    compiler: &sifr_identity::CompilerIdentity,
    defs: sifr_lowering::ExternalDefs,
    package: Option<&crate::PackageEntrypoint>,
) -> Option<Vec<ModuleCheckDecision>> {
    let package_identity = package.map(super::package_context::identity);
    if package_identity.as_ref().is_some_and(Option::is_none)
        || package_identity
            .flatten()
            .as_deref()
            .unwrap_or("manifestless-owner-v1")
            != inputs.package_and_lock
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
    let frontend = if let Some(package) = package {
        // Reuse the ordinary package resolver, including visibility, ownership,
        // direct dependency scopes, aliases and relative-import rewriting.
        let resolved = crate::project::parse_package_import_closure_source_project(
            &package.graph,
            &package.source_map,
            &package.package_id,
            &package.main_file,
            crate::project::DiscoveryDiagnosticStyle::ModuleName,
            provider,
        )
        .ok()?;
        let entry_name = resolved.entry_module_name;
        let mut modules = resolved
            .parsed_modules
            .into_iter()
            .map(|(name, module)| sifr_frontend::ResolvedCheckModule {
                name: if name == entry_name {
                    "main".into()
                } else {
                    name
                },
                source: sifr_frontend::persistence::CapturedSource {
                    path: module.display_path.into(),
                    text: module.source,
                },
                suite: module.suite,
            })
            .collect::<Vec<_>>();
        modules.sort_by(|a, b| a.name.cmp(&b.name));
        FrontendContext::from_resolved_check_modules(root, modules, defs)?
    } else {
        let sources = record
            .result
            .resolution
            .ready()?
            .sources
            .iter()
            .filter(|source| source.path.extension().is_some_and(|ext| ext == "sifr"))
            .map(|source| {
                use sifr_frontend::SourceProvider;
                Some(sifr_frontend::persistence::CapturedSource {
                    path: source.path.clone(),
                    text: provider.read_file(&source.path).ok()?.as_str().to_owned(),
                })
            })
            .collect::<Option<Vec<_>>>()?;
        FrontendContext::from_resolved_sources(root, sources, defs)?
    };
    let mut frontend = frontend.with_compiler_identity(compiler.clone());
    let modules = frontend.restore_completed_checks(record, inputs, provider, true)?;
    modules
        .iter()
        .any(|module| module.action == "restored")
        .then_some(modules)
}
