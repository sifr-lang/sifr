use super::*;

#[test]
fn test_check_package_project_rejects_private_dependency_module() {
    let dir = mktemp_dir("package_private_rejection");
    let app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    let json = production_package(&dir, "json", "sifr-demo-json", "demo_json");
    write_package_source(
        &app,
        "main.sifr",
        "from demo_json.parse import parse_json\n\n\
def main():\n    assert parse_json() == 1\n",
    );
    write_package_source(&json, "__init__.sifr", "from .parse import parse_json\n");
    write_package_source(
        &json,
        "parse.sifr",
        "def parse_json() -> int:\n    return 1\n",
    );
    let graph = package_graph(
        &dir,
        &[&app, &json],
        &[package_edge(&app, "demo_json", &json)],
    );
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let entrypoint = package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));

    let errors = check_package_project(
        &crate::CompilerContext::for_test(),
        &entrypoint,
        &mut sifr_frontend::DiskSourceProvider::new(),
    );

    assert!(
        errors
            .iter()
            .any(|error| error.code == DiagnosticCode::PACKAGE_PRIVATE_MODULE_ACCESS.code()),
        "private dependency module should be rejected: {errors:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_check_package_project_reclassifies_unresolved_bare_stdlib_import() {
    let dir = mktemp_dir("package_bare_stdlib_import");
    let app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    write_package_source(
        &app,
        "main.sifr",
        "from math import sqrt\n\n\
def main():\n    pass\n",
    );
    let graph = package_graph(&dir, &[&app], &[]);
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let entrypoint = package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));

    let errors = check_package_project(
        &crate::CompilerContext::for_test(),
        &entrypoint,
        &mut sifr_frontend::DiskSourceProvider::new(),
    );

    let diagnostic = errors
        .iter()
        .find(|error| error.code == DiagnosticCode::IMPORT_BARE_STDLIB.code())
        .unwrap_or_else(|| panic!("bare stdlib diagnostic should be emitted: {errors:?}"));
    assert_eq!(
        diagnostic.message,
        "bare stdlib import 'math'; Sifr stdlib lives under 'sifr.*'"
    );
    assert_eq!(diagnostic.args["bare_module"], "math".into());
    assert_eq!(diagnostic.args["suggested_module"], "sifr.math".into());
    assert_eq!(diagnostic.args["imported_names"], "sqrt".into());
    assert_eq!(
        diagnostic.help.as_deref(),
        Some("use 'from sifr.math import sqrt'")
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_check_package_project_rejects_transitive_dependency_import() {
    let dir = mktemp_dir("package_transitive_rejection");
    let app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    let mid = production_package(&dir, "mid", "sifr-demo-mid", "demo_mid");
    let json = production_package(&dir, "json", "sifr-demo-json", "demo_json");
    write_package_source(
        &app,
        "main.sifr",
        "from demo_json import parse_json\n\n\
def main():\n    assert parse_json() == 1\n",
    );
    write_package_source(&mid, "__init__.sifr", "from demo_json import parse_json\n");
    write_package_source(&json, "__init__.sifr", "from .parse import parse_json\n");
    write_package_source(
        &json,
        "parse.sifr",
        "def parse_json() -> int:\n    return 1\n",
    );
    let graph = package_graph(
        &dir,
        &[&app, &mid, &json],
        &[
            package_edge(&app, "demo_mid", &mid),
            package_edge(&mid, "demo_json", &json),
        ],
    );
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let entrypoint = package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));

    let errors = check_package_project(
        &crate::CompilerContext::for_test(),
        &entrypoint,
        &mut sifr_frontend::DiskSourceProvider::new(),
    );

    assert!(
        errors
            .iter()
            .any(|error| error.code == DiagnosticCode::PACKAGE_UNDECLARED_DIRECT_IMPORT.code()),
        "transitive dependency import should be rejected: {errors:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_check_package_project_reports_internal_reexport_cycles() {
    let dir = mktemp_dir("package_reexport_cycle");
    let app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    let cycle = production_package(&dir, "cycle", "sifr-demo-cycle", "demo_cycle");
    write_package_source(
        &app,
        "main.sifr",
        "from demo_cycle import value\n\n\
def main():\n    assert value() == 1\n",
    );
    write_package_source(&cycle, "__init__.sifr", "from .a import value\n");
    write_package_source(&cycle, "a.sifr", "from .b import value\n");
    write_package_source(&cycle, "b.sifr", "from .a import value\n");
    let graph = package_graph(
        &dir,
        &[&app, &cycle],
        &[package_edge(&app, "demo_cycle", &cycle)],
    );
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let entrypoint = package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));

    let errors = check_package_project(
        &crate::CompilerContext::for_test(),
        &entrypoint,
        &mut sifr_frontend::DiskSourceProvider::new(),
    );

    assert!(
        errors
            .iter()
            .any(|error| error.code == DiagnosticCode::IMPORT_CYCLE.code()
                && error.spans.iter().any(|span| span.is_primary)
                && error.spans.iter().filter(|span| !span.is_primary).count() >= 2),
        "package re-export cycle should be reported: {errors:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}
