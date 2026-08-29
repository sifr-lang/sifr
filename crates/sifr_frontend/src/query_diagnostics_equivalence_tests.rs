use crate::{
    DiskSourceProvider, DocumentVersion, FrontendContext, FrontendInput, FrontendMode, ModuleId,
    ProjectRoot, SourcePath, SourceText,
};
use sifr_diagnostics::RenderedDiagnostic;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[test]
fn single_file_edit_queries_match_clean_contexts() {
    let success_before = "def main():\n    value: int = 1\n";
    let success_after = "def main():\n    value: int = 2\n";
    let type_error = "def main():\n    value: str = 1\n";
    let main = ModuleId(0);
    let mut context = single_file_context(success_before);

    assert_eq!(
        diagnostics_for_module(&mut context, main),
        clean_single_file_diagnostics(success_before),
    );

    update_module(&mut context, main, success_after, 2);
    assert_eq!(
        diagnostics_for_module(&mut context, main),
        clean_single_file_diagnostics(success_after),
        "success-preserving edit must match a clean context",
    );

    update_module(&mut context, main, type_error, 3);
    assert_eq!(
        diagnostics_for_module(&mut context, main),
        clean_single_file_diagnostics(type_error),
        "diagnostic-introducing edit must match a clean context",
    );

    update_module(&mut context, main, success_after, 4);
    assert_eq!(
        diagnostics_for_module(&mut context, main),
        clean_single_file_diagnostics(success_after),
        "diagnostic-fixing edit must match a clean context",
    );
}

#[test]
fn project_edit_queries_match_clean_contexts() {
    let mut sources = BTreeMap::from([
        (
            "api.sifr",
            "def public_value() -> int:\n    return 2\n".to_string(),
        ),
        (
            "helper.sifr",
            "def value() -> int:\n    return 1\n".to_string(),
        ),
        (
            "main.sifr",
            "from api import public_value\nfrom helper import value\n\n\
             def main():\n    total: int = value() + public_value()\n    assert total == 3\n"
                .to_string(),
        ),
    ]);
    let project = temp_project_dir("project_edit_equivalence");
    write_project(&project, &sources);
    let mut context = load_project(&project);

    assert_eq!(
        project_diagnostics(&mut context),
        clean_project_diagnostics(&sources)
    );
    assert_eq!(edge_summary(&context), clean_project_edges(&sources));

    let helper = module_by_stem(&context, "helper");
    sources.insert(
        "helper.sifr",
        "def value() -> int:\n    return 10\n".to_string(),
    );
    update_module(&mut context, helper, sources["helper.sifr"].as_str(), 2);
    assert_eq!(
        project_diagnostics(&mut context),
        clean_project_diagnostics(&sources),
        "success-preserving helper edit must match a clean project",
    );

    let api = module_by_stem(&context, "api");
    sources.insert(
        "api.sifr",
        "def public_value() -> str:\n    return \"changed\"\n".to_string(),
    );
    update_module(&mut context, api, sources["api.sifr"].as_str(), 3);
    assert_eq!(
        project_diagnostics(&mut context),
        clean_project_diagnostics(&sources),
        "public API diagnostic edit must match a clean project",
    );

    sources.insert(
        "api.sifr",
        "def public_value() -> int:\n    return 30\n".to_string(),
    );
    update_module(&mut context, api, sources["api.sifr"].as_str(), 4);
    assert_eq!(
        project_diagnostics(&mut context),
        clean_project_diagnostics(&sources),
        "diagnostic recovery edit must match a clean project",
    );

    let main = module_by_stem(&context, "main");
    sources.insert(
        "main.sifr",
        "from api import public_value\n\n\
         def main():\n    total: int = public_value()\n    assert total == 30\n"
            .to_string(),
    );
    update_module(&mut context, main, sources["main.sifr"].as_str(), 5);
    assert_eq!(
        project_diagnostics(&mut context),
        clean_project_diagnostics(&sources),
        "project graph dependency edit diagnostics must match a clean project",
    );
    assert_eq!(
        edge_summary(&context),
        clean_project_edges(&sources),
        "project graph dependency edit edges must match a clean project",
    );
}

#[test]
fn semantic_property_incremental_queries_match_full_recomputation() {
    let variants = [
        "def value() -> int:\n    return 1\n",
        "def value() -> int:\n    return 2\n",
        "def value() -> str:\n    return \"wrong\"\n",
        "def value() -> int:\n    return 3\n",
    ];
    let main = ModuleId(0);
    let mut context = single_file_context(variants[0]);

    for (index, source) in variants.iter().enumerate() {
        if index > 0 {
            let version = i64::try_from(index).expect("fixture edit index must fit i64") + 1;
            update_module(&mut context, main, source, version);
        }
        assert_eq!(
            diagnostics_for_module(&mut context, main),
            clean_single_file_diagnostics(source),
            "incremental diagnostics differ after edit {index}",
        );
    }
}

fn single_file_context(source: &str) -> FrontendContext {
    FrontendContext::load_single_file(FrontendInput {
        path: SourcePath::new("main.sifr"),
        source: SourceText::new(source),
        mode: FrontendMode::SingleFile,
    })
    .expect("single-file context should load")
}

fn clean_single_file_diagnostics(source: &str) -> Vec<RenderedDiagnostic> {
    let mut context = single_file_context(source);
    diagnostics_for_module(&mut context, ModuleId(0))
}

fn diagnostics_for_module(
    context: &mut FrontendContext,
    module: ModuleId,
) -> Vec<RenderedDiagnostic> {
    context
        .diagnostics_for_module(module)
        .into_value()
        .diagnostics
}

fn project_diagnostics(context: &mut FrontendContext) -> Vec<RenderedDiagnostic> {
    context.diagnostics_for_project().into_value().diagnostics
}

fn clean_project_diagnostics(sources: &BTreeMap<&'static str, String>) -> Vec<RenderedDiagnostic> {
    let dir = temp_project_dir("clean_project_diagnostics");
    write_project(&dir, sources);
    let mut context = load_project(&dir);
    project_diagnostics(&mut context)
}

fn clean_project_edges(sources: &BTreeMap<&'static str, String>) -> Vec<(String, String)> {
    let dir = temp_project_dir("clean_project_edges");
    write_project(&dir, sources);
    let context = load_project(&dir);
    edge_summary(&context)
}

fn update_module(context: &mut FrontendContext, module: ModuleId, source: &str, version: i64) {
    context
        .update_module_source(
            module,
            SourceText::new(source),
            Some(DocumentVersion::new(version)),
        )
        .expect("module update should succeed");
}

fn module_by_stem(context: &FrontendContext, stem: &str) -> ModuleId {
    context
        .module_graph()
        .modules
        .iter()
        .find(|module| {
            module
                .canonical_path
                .as_path()
                .file_stem()
                .is_some_and(|candidate| candidate == stem)
        })
        .map(|module| module.id)
        .unwrap_or_else(|| panic!("project fixture is missing module {stem:?}"))
}

fn edge_summary(context: &FrontendContext) -> Vec<(String, String)> {
    let graph = context.module_graph();
    let names_by_id = graph
        .modules
        .iter()
        .map(|module| (module.id, module_stem(&module.canonical_path)))
        .collect::<BTreeMap<_, _>>();
    let mut edges = graph
        .edges
        .iter()
        .map(|edge| {
            (
                names_by_id[&edge.importer].clone(),
                names_by_id[&edge.imported].clone(),
            )
        })
        .collect::<Vec<_>>();
    edges.sort();
    edges
}

fn module_stem(path: &SourcePath) -> String {
    path.as_path()
        .file_stem()
        .and_then(|stem| stem.to_str())
        .expect("module path should have a UTF-8 stem")
        .to_string()
}

fn load_project(dir: &Path) -> FrontendContext {
    let mut provider = DiskSourceProvider::new();
    FrontendContext::load_project(
        &ProjectRoot {
            root: SourcePath::new(dir),
            entrypoint: SourcePath::new(dir.join("main.sifr")),
        },
        &mut provider,
    )
    .expect("project context should load")
}

fn write_project(dir: &Path, sources: &BTreeMap<&'static str, String>) {
    std::fs::create_dir_all(dir).expect("project directory should be created");
    for (name, source) in sources {
        std::fs::write(dir.join(name), source).expect("project source should be written");
    }
}

fn temp_project_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "sifr_frontend_{name}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    ))
}
