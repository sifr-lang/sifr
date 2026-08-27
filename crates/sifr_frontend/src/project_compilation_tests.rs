use crate::{
    FrontendContext, FrontendDiagnosticStyle, FrontendProjectModule, compute_project_compile_order,
    parse_source,
};
use sifr_lowering::{ExternalDefs, LoweringOptions};
use std::collections::BTreeMap;

fn project_module(path: &str, source: &str) -> FrontendProjectModule {
    FrontendProjectModule {
        suite: parse_source(source, Some(path)).expect("project fixture should parse"),
        source: source.to_string(),
        display_path: path.to_string(),
    }
}

fn project_modules() -> BTreeMap<String, FrontendProjectModule> {
    BTreeMap::from([
        (
            "main".to_string(),
            project_module(
                "/project/main.sifr",
                "from helper import answer\n\ndef main() -> int:\n    value = answer()\n    reveal_type(value)\n    return value\n",
            ),
        ),
        (
            "helper".to_string(),
            project_module(
                "/project/helper.sifr",
                "def answer() -> int:\n    value = 42\n    reveal_type(value)\n    return value\n",
            ),
        ),
    ])
}

#[test]
fn project_compilation_product_and_analysis_share_one_lowered_snapshot() {
    let modules = project_modules();
    let mut context = FrontendContext::load_project_modules(&modules, ExternalDefs::default())
        .expect("project context should load");
    let graph = context.module_graph();
    let compilation = context
        .compile_project(FrontendDiagnosticStyle::Bare, &LoweringOptions::default())
        .expect("project should compile");

    assert_eq!(compilation.compile_order, ["helper", "main"]);
    assert_eq!(compilation.lowering_results.len(), 2);
    assert_eq!(compilation.hir_modules.len(), 2);
    assert_eq!(compilation.flow_graphs.len(), 2);

    for node in graph.modules {
        let module_name = node
            .canonical_path
            .as_path()
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("fixture module should have a UTF-8 file stem");
        let query_hir = context.lower_module(node.id).into_value().hir;
        let compiled_hir = compilation
            .hir_modules
            .get(module_name)
            .expect("compiled module should exist");
        assert_eq!(format!("{query_hir:?}"), format!("{compiled_hir:?}"));
    }

    let query_diagnostics = context.diagnostics_for_project().into_value().diagnostics;
    let product_diagnostics = compilation
        .compile_order
        .iter()
        .filter_map(|module_name| compilation.module_diagnostics.get(module_name))
        .flat_map(|diagnostics| {
            diagnostics
                .rendered_warnings
                .iter()
                .chain(&diagnostics.rendered_reveal_types)
                .cloned()
        })
        .collect::<Vec<_>>();
    assert_eq!(query_diagnostics, product_diagnostics);
}

#[test]
fn project_compile_order_is_dependency_safe_and_insertion_independent() {
    let modules = project_modules();
    let reversed = modules
        .iter()
        .rev()
        .map(|(name, module)| (name.clone(), module.clone()))
        .collect();

    assert_eq!(
        compute_project_compile_order(&modules).expect("project should be acyclic"),
        ["helper", "main"]
    );
    assert_eq!(
        compute_project_compile_order(&reversed).expect("project should be acyclic"),
        ["helper", "main"]
    );
}

#[test]
fn project_compile_order_reports_a_source_anchored_canonical_cycle() {
    let modules = BTreeMap::from([
        (
            "a".to_string(),
            project_module(
                "/project/a.sifr",
                "from b import value_b\n\ndef value_a() -> int:\n    return value_b()\n",
            ),
        ),
        (
            "b".to_string(),
            project_module(
                "/project/b.sifr",
                "from a import value_a\n\ndef value_b() -> int:\n    return value_a()\n",
            ),
        ),
    ]);

    let errors = compute_project_compile_order(&modules)
        .expect_err("cyclic project should not have a compile order");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].message, "circular import detected: a -> b -> a");
    assert!(errors[0].spans.iter().any(|span| span.is_primary));
}

#[test]
fn query_graph_and_project_product_share_relative_dependency_resolution() {
    let modules = BTreeMap::from([
        (
            "main".to_string(),
            project_module(
                "/project/main.sifr",
                "from pkg.consumer import value\n\ndef main() -> int:\n    return value()\n",
            ),
        ),
        (
            "pkg.consumer".to_string(),
            project_module(
                "/project/pkg/consumer.sifr",
                "from .helper import answer\n\ndef value() -> int:\n    return answer()\n",
            ),
        ),
        (
            "pkg.helper".to_string(),
            project_module(
                "/project/pkg/helper.sifr",
                "def answer() -> int:\n    return 42\n",
            ),
        ),
    ]);
    let mut context = FrontendContext::load_project_modules(&modules, ExternalDefs::default())
        .expect("relative-import project should load");

    assert_eq!(context.module_graph().edges.len(), 2);
    let compilation = context
        .compile_project(FrontendDiagnosticStyle::Bare, &LoweringOptions::default())
        .expect("relative-import project should compile");
    assert_eq!(
        compilation.compile_order,
        ["pkg.helper", "pkg.consumer", "main"]
    );
}
