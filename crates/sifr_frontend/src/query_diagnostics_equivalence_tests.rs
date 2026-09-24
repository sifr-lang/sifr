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
        clean_project_diagnostics(&sources, &project)
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
        clean_project_diagnostics(&sources, &project),
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
        clean_project_diagnostics(&sources, &project),
        "public API diagnostic edit must match a clean project",
    );

    sources.insert(
        "api.sifr",
        "def public_value() -> int:\n    return 30\n".to_string(),
    );
    update_module(&mut context, api, sources["api.sifr"].as_str(), 4);
    assert_eq!(
        project_diagnostics(&mut context),
        clean_project_diagnostics(&sources, &project),
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
        clean_project_diagnostics(&sources, &project),
        "project graph dependency edit diagnostics must match a clean project",
    );
    assert_eq!(
        edge_summary(&context),
        clean_project_edges(&sources),
        "project graph dependency edit edges must match a clean project",
    );
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

fn clean_project_diagnostics(
    sources: &BTreeMap<&'static str, String>,
    reference_path: &Path,
) -> Vec<RenderedDiagnostic> {
    let dir = temp_project_dir("clean_project_diagnostics");
    write_project(&dir, sources);
    let mut context = load_project(&dir);
    let mut diagnostics = project_diagnostics(&mut context);
    // These equivalent snapshots live in distinct temporary directories.
    for diagnostic in &mut diagnostics {
        for span in &mut diagnostic.spans {
            if let Some(file) = &span.file {
                if let Ok(relative) = Path::new(file).strip_prefix(&dir) {
                    span.file = Some(reference_path.join(relative).to_string_lossy().into_owned());
                }
            }
        }
    }
    diagnostics
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

fn product_diagnostics_for_snapshot(
    dir: &Path,
    sources: &BTreeMap<&'static str, String>,
) -> Vec<RenderedDiagnostic> {
    let parsed = sources
        .iter()
        .map(|(filename, source)| {
            let name = filename.trim_end_matches(".sifr").to_string();
            let suite = crate::parse_source(source, Some(&name)).expect("fixture parses");
            (name, suite)
        })
        .collect::<BTreeMap<_, _>>();
    let paths = sources
        .keys()
        .map(|filename| {
            (
                filename.trim_end_matches(".sifr").to_string(),
                dir.join(filename),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let inputs = parsed
        .iter()
        .map(|(name, suite)| {
            let filename = format!("{name}.sifr");
            (
                name.clone(),
                crate::FrontendProductInput {
                    suite,
                    source: &sources[filename.as_str()],
                    display_path: paths[name].to_str().expect("UTF-8 fixture path"),
                    source_backed: true,
                },
            )
        })
        .collect();
    crate::compile_frontend_product(
        &inputs,
        sifr_lowering::ExternalDefs::default(),
        crate::FrontendDiagnosticStyle::ModulePrefixed,
        &sifr_lowering::LoweringOptions::default(),
    )
    .err()
    .expect("snapshot must fail")
}

#[test]
fn source_backed_cycles_match_product_and_analysis() {
    for (name, sources, cycle, related_count) in [
        (
            "self_cycle",
            BTreeMap::from([("main.sifr", "from main import value\n".to_string())]),
            "main -> main",
            1,
        ),
        (
            "multi_cycle",
            BTreeMap::from([
                ("main.sifr", "from a import value\n".to_string()),
                ("a.sifr", "from b import value\n".to_string()),
                ("b.sifr", "from a import value\n".to_string()),
            ]),
            "a -> b -> a",
            2,
        ),
    ] {
        let dir = temp_project_dir(name);
        write_project(&dir, &sources);
        let product = product_diagnostics_for_snapshot(&dir, &sources);
        let mut context = load_project(&dir);
        let analysis = project_diagnostics(&mut context);
        assert_eq!(
            analysis, product,
            "CLI product and analysis must use one snapshot"
        );
        assert_eq!(analysis.len(), 1);
        assert_eq!(
            analysis[0].code,
            sifr_diagnostics::DiagnosticCode::IMPORT_CYCLE.code()
        );
        assert!(analysis[0].message.contains(cycle));
        assert_eq!(
            analysis[0]
                .spans
                .iter()
                .filter(|span| !span.is_primary)
                .count(),
            related_count
        );
        assert!(analysis[0].spans.iter().all(|span| span.file.is_some()));
    }
}

#[test]
fn cycle_edits_invalidate_diagnostics_and_broken_imports_do_not_claim_cycles() {
    let mut sources = BTreeMap::from([
        ("main.sifr", "from a import value\n".to_string()),
        ("a.sifr", "value: int = 1\n".to_string()),
    ]);
    let dir = temp_project_dir("cycle_edits");
    write_project(&dir, &sources);
    let mut context = load_project(&dir);
    assert!(project_diagnostics(&mut context).is_empty());

    let a = module_by_stem(&context, "a");
    sources.insert(
        "a.sifr",
        "from main import value\nvalue: int = 1\n".to_string(),
    );
    update_module(&mut context, a, &sources["a.sifr"], 2);
    let cycle = project_diagnostics(&mut context);
    assert_eq!(cycle, product_diagnostics_for_snapshot(&dir, &sources));
    assert_eq!(cycle.len(), 1);
    assert_eq!(
        cycle[0].code,
        sifr_diagnostics::DiagnosticCode::IMPORT_CYCLE.code()
    );

    sources.insert("a.sifr", "value: int = 1\n".to_string());
    update_module(&mut context, a, &sources["a.sifr"], 3);
    assert!(project_diagnostics(&mut context).is_empty());

    sources.insert("main.sifr", "from missing import value\n".to_string());
    let main = module_by_stem(&context, "main");
    update_module(&mut context, main, &sources["main.sifr"], 4);
    let broken = project_diagnostics(&mut context);
    assert!(!broken.is_empty());
    assert!(broken.iter().all(|diagnostic| {
        diagnostic.code != sifr_diagnostics::DiagnosticCode::IMPORT_CYCLE.code()
    }));
}

#[test]
fn successful_product_preserves_exports_flow_and_analysis_diagnostics() {
    let sources = BTreeMap::from([
        (
            "main.sifr",
            "from helper import value\n\ndef main() -> int:\n    reveal_type(value())\n    return value()\n"
                .to_string(),
        ),
        ("helper.sifr", "def value() -> int:\n    return 1\n".to_string()),
    ]);
    let dir = temp_project_dir("product_analysis_equivalence");
    write_project(&dir, &sources);
    let parsed = sources
        .iter()
        .map(|(filename, source)| {
            let name = filename.trim_end_matches(".sifr").to_string();
            (
                name.clone(),
                crate::parse_source(source, Some(&name)).expect("fixture parses"),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let paths = sources
        .keys()
        .map(|filename| {
            (
                filename.trim_end_matches(".sifr").to_string(),
                dir.join(filename),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let inputs = parsed
        .iter()
        .map(|(name, suite)| {
            let filename = format!("{name}.sifr");
            (
                name.clone(),
                crate::FrontendProductInput {
                    suite,
                    source: &sources[filename.as_str()],
                    display_path: paths[name].to_str().expect("UTF-8 fixture path"),
                    source_backed: true,
                },
            )
        })
        .collect();
    let product = crate::compile_frontend_product(
        &inputs,
        sifr_lowering::ExternalDefs::default(),
        crate::FrontendDiagnosticStyle::ModulePrefixed,
        &sifr_lowering::LoweringOptions::default(),
    )
    .expect("source snapshot should compile");
    assert_eq!(product.compile_order, vec!["helper", "main"]);
    assert_eq!(product.hir_modules.len(), 2);
    assert_eq!(product.flow_graphs.len(), 2);
    assert!(
        product
            .external_defs
            .functions
            .get("helper")
            .is_some_and(|functions| functions.contains_key("value"))
    );
    let product_diagnostics = product
        .compile_order
        .iter()
        .flat_map(|name| {
            let module = &product.module_diagnostics[name];
            module
                .rendered_warnings
                .iter()
                .chain(&module.rendered_reveal_types)
                .cloned()
        })
        .collect::<Vec<_>>();
    let mut context = load_project(&dir);
    assert_eq!(project_diagnostics(&mut context), product_diagnostics);
}
