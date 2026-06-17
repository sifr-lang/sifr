use crate::{build_cached_project, build_project, check_project, emit_project, CompileResult};
use sifr_diagnostics::DiagnosticCode;

pub(super) fn mktemp_dir(name: &str) -> std::path::PathBuf {
    let unique = format!(
        "sifr_project_build_{name}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&dir).expect("temp dir should be created");
    dir
}

#[test]
fn test_check_project_resolves_valid_local_imports() {
    let dir = mktemp_dir("check_positive");
    std::fs::write(
        dir.join("main.sifr"),
        r#"
from helper import area

def main():
    print(area(2.0))
"#,
    )
    .expect("main module should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        r#"
from sifr.math import pi

def area(radius: float) -> float:
    return pi * radius * radius
"#,
    )
    .expect("helper module should be written");

    let errors = check_project(&dir.join("main.sifr"));
    assert!(
        errors.is_empty(),
        "check_project should succeed: {errors:?}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_check_project_reports_primary_span_for_ranged_hir_diagnostic() {
    let dir = mktemp_dir("check_project_primary_span");
    let main_file = dir.join("main.sifr");
    std::fs::write(&main_file, "def main() -> None:\n    if 1:\n        pass\n")
        .expect("main module should be written");

    let errors = check_project(&main_file);
    let diagnostic = errors
        .iter()
        .find(|error| error.code == DiagnosticCode::FLOW_INVALID_CONDITION_TYPE.code())
        .expect("expected invalid condition diagnostic");
    let primary = diagnostic
        .spans
        .iter()
        .find(|span| span.is_primary)
        .expect("expected primary span");

    let expected_file = main_file.display().to_string();
    assert_eq!(primary.file.as_deref(), Some(expected_file.as_str()));
    assert_eq!(primary.line, Some(2));
    assert_eq!(primary.column, Some(8));
    assert_eq!(primary.end_line, Some(2));
    assert_eq!(primary.end_column, Some(9));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_check_project_resolves_workspace_source_import_for_non_main_entry() {
    let dir = mktemp_dir("workspace_check_non_main");
    std::fs::create_dir_all(dir.join("cases")).expect("cases dir should be created");
    std::fs::create_dir_all(dir.join("lib")).expect("lib dir should be created");
    std::fs::write(dir.join("sifr.toml"), "[source]\nroots = [\"lib\"]\n")
        .expect("manifest should be written");
    std::fs::write(
        dir.join("cases/app.sifr"),
        "from helper import value\n\ndef main():\n    print(value())\n",
    )
    .expect("entry should be written");
    std::fs::write(
        dir.join("lib/helper.sifr"),
        "def value() -> int:\n    return 42\n",
    )
    .expect("helper should be written");

    let errors = check_project(&dir.join("cases/app.sifr"));

    assert!(
        errors.is_empty(),
        "workspace check should succeed: {errors:?}"
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_build_project_materializes_dotted_workspace_modules() {
    let dir = mktemp_dir("workspace_dotted_build");
    let main_file = dir.join("cases/app.sifr");
    let build_out = dir.join("build_out");
    std::fs::create_dir_all(dir.join("cases")).expect("cases dir should be created");
    std::fs::create_dir_all(dir.join("lib/helpers")).expect("helpers dir should be created");
    std::fs::write(dir.join("sifr.toml"), "[source]\nroots = [\"lib\"]\n")
        .expect("manifest should be written");
    std::fs::write(
        &main_file,
        "from helpers.value import answer\n\ndef main():\n    print(answer())\n",
    )
    .expect("entry should be written");
    std::fs::write(
        dir.join("lib/helpers/value.sifr"),
        "def answer() -> int:\n    return 42\n",
    )
    .expect("helper should be written");

    let binary = build_project(&main_file, &build_out)
        .expect("workspace dotted project should build successfully");

    assert!(binary.exists());
    let src_dir = build_out.join("sifr_output/src");
    let main_rs = std::fs::read_to_string(src_dir.join("main.rs")).expect("main.rs should exist");
    let helpers_mod =
        std::fs::read_to_string(src_dir.join("helpers/mod.rs")).expect("helpers mod should exist");
    assert!(main_rs.contains("mod helpers;"));
    assert_eq!(helpers_mod, "pub mod value;\n");
    assert!(src_dir.join("helpers/value.rs").is_file());

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_build_project_preserves_imported_class_constructors_and_signatures() {
    let dir = mktemp_dir("workspace_imported_class_codegen");
    let main_file = dir.join("cases/app.sifr");
    let build_out = dir.join("build_out");
    std::fs::create_dir_all(dir.join("cases")).expect("cases dir should be created");
    std::fs::create_dir_all(dir.join("lib/helpers")).expect("helpers dir should be created");
    std::fs::write(dir.join("sifr.toml"), "[source]\nroots = [\"lib\"]\n")
        .expect("manifest should be written");
    std::fs::write(
        &main_file,
        r#"
from helpers.nodes import NodeBag, LinkedNode, node_value

def main():
    bag = NodeBag()
    assert bag.count() == 0
    head = LinkedNode(1, LinkedNode(2, None))
    assert node_value(head) == 1
"#,
    )
    .expect("entry should be written");
    std::fs::write(
        dir.join("lib/helpers/nodes.sifr"),
        r#"
class LinkedNode:
    val: int
    next: LinkedNode | None

    def __init__(self, val: int = 0, next: LinkedNode | None = None):
        self.val = val
        self.next = next

class NodeBag:
    items: list[int]

    def __init__(self):
        self.items = []

    def count(self) -> int:
        return len(self.items)

def node_value(node: LinkedNode | None) -> int:
    if node is None:
        return 0
    return node.val
"#,
    )
    .expect("helper should be written");

    let binary = build_project(&main_file, &build_out)
        .expect("imported class constructors should build successfully");

    assert!(binary.exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_emit_project_includes_workspace_support_modules() {
    let dir = mktemp_dir("workspace_emit");
    let main_file = dir.join("cases/app.sifr");
    std::fs::create_dir_all(dir.join("cases")).expect("cases dir should be created");
    std::fs::create_dir_all(dir.join("lib/helpers")).expect("helpers dir should be created");
    std::fs::write(dir.join("sifr.toml"), "[source]\nroots = [\"lib\"]\n")
        .expect("manifest should be written");
    std::fs::write(
        &main_file,
        "from helpers.value import answer\n\ndef main():\n    print(answer())\n",
    )
    .expect("entry should be written");
    std::fs::write(
        dir.join("lib/helpers/value.sifr"),
        "def answer() -> int:\n    return 7\n",
    )
    .expect("helper should be written");

    let emitted = emit_project(&main_file);

    let CompileResult::Success { rust_source } = emitted else {
        panic!("workspace project emit should succeed");
    };
    assert!(rust_source.contains("// src/main.rs"));
    assert!(rust_source.contains("mod helpers;"));
    assert!(rust_source.contains("// src/helpers/value.rs"));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_cached_project_invalidates_when_workspace_helper_changes() {
    let dir = mktemp_dir("workspace_cache_invalidation");
    let main_file = dir.join("cases/app.sifr");
    let helper_file = dir.join("lib/helpers/value.sifr");
    std::fs::create_dir_all(dir.join("cases")).expect("cases dir should be created");
    std::fs::create_dir_all(dir.join("lib/helpers")).expect("helpers dir should be created");
    std::fs::write(dir.join("sifr.toml"), "[source]\nroots = [\"lib\"]\n")
        .expect("manifest should be written");
    std::fs::write(
        &main_file,
        "from helpers.value import answer\n\ndef main():\n    print(answer())\n",
    )
    .expect("entry should be written");
    std::fs::write(&helper_file, "def answer() -> int:\n    return 10\n")
        .expect("helper should be written");

    let first = build_cached_project(&main_file).expect("first workspace build should succeed");
    std::fs::write(&helper_file, "def answer() -> int:\n    return 11\n")
        .expect("helper should be updated");
    let second = build_cached_project(&main_file).expect("second workspace build should succeed");

    assert!(!first.build_report().cache_hit());
    assert!(!second.build_report().cache_hit());
    assert_ne!(first.binary_path(), second.binary_path());

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_check_project_ignores_unrelated_non_closure_parse_errors() {
    let dir = mktemp_dir("check_ignore");
    std::fs::create_dir_all(&dir).expect("project dir should be created");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import value\n\ndef main():\n    print(value())\n",
    )
    .expect("main module should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "def value() -> int:\n    return 42\n",
    )
    .expect("helper module should be written");
    std::fs::write(dir.join("unrelated_bad.sifr"), "def unrelated(:\n")
        .expect("unrelated sibling should be written");

    let errors = check_project(&dir.join("main.sifr"));
    assert!(
        errors.is_empty(),
        "unrelated sibling parse errors should not affect check_project: {errors:?}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_check_project_reports_reachable_parse_errors_in_import_closure() {
    let dir = mktemp_dir("check_reachable");
    std::fs::create_dir_all(&dir).expect("project dir should be created");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import value\n\ndef main():\n    print(value())\n",
    )
    .expect("main module should be written");
    std::fs::write(dir.join("helper.sifr"), "def value(:\n")
        .expect("helper module should be written");
    std::fs::write(
        dir.join("unrelated_ok.sifr"),
        "def spare() -> int:\n    return 1\n",
    )
    .expect("unrelated module should be written");

    let errors = check_project(&dir.join("main.sifr"));
    assert!(
        errors.iter().any(|e| e.code == "SIFR-PARSE-0002"
            && e.children
                .iter()
                .any(|child| child.message == "while parsing helper")),
        "reachable parse errors must still fail check_project: {errors:?}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_check_project_error_messages_match_build_project() {
    let dir = mktemp_dir("check_build_parity");
    std::fs::create_dir_all(&dir).expect("project dir should be created");
    std::fs::write(
        dir.join("main.sifr"),
        r#"
from helper import broken

def main():
    print(broken())
"#,
    )
    .expect("main module should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        r#"
def broken() -> int:
    return "bad"
"#,
    )
    .expect("helper module should be written");

    let check_errors = check_project(&dir.join("main.sifr"));
    let build_errors = build_project(&dir.join("main.sifr"), &dir.join("build_out"))
        .err()
        .expect("build_project should fail with same frontend error");

    let check_messages: Vec<String> = check_errors
        .iter()
        .map(crate::diagnostics::diagnostic_legacy_display)
        .collect();
    let build_messages: Vec<String> = build_errors
        .iter()
        .map(crate::diagnostics::diagnostic_legacy_display)
        .collect();
    assert_eq!(check_messages, build_messages);
    assert!(build_messages
        .iter()
        .any(|m| m.contains("[helper] return type mismatch")));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_build_project_includes_support_module_required_features_in_manifest() {
    let dir = mktemp_dir("manifest_positive");
    let main_file = dir.join("main.sifr");
    let build_out = dir.join("build_out");
    std::fs::write(
        &main_file,
        "from helper import helper\n\ndef main():\n    print(helper())\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "def helper() -> bigint:\n    return bigint(42)\n",
    )
    .expect("helper should be written");

    let binary = build_project(&main_file, &build_out)
        .expect("project build should succeed with support-module required crates");
    assert!(binary.exists());

    let cargo_toml = std::fs::read_to_string(build_out.join("sifr_output").join("Cargo.toml"))
        .expect("cargo manifest should be written");
    assert!(cargo_toml.contains("num-bigint = \"0.4.6\""));
    assert!(cargo_toml.contains("num-traits = \"0.2.19\""));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_build_project_manifest_ignores_unreachable_required_features() {
    let dir = mktemp_dir("manifest_negative");
    let main_file = dir.join("main.sifr");
    let build_out = dir.join("build_out");
    std::fs::write(
        &main_file,
        "from helper import helper\n\ndef main():\n    print(helper())\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "def helper() -> int:\n    return 1\n",
    )
    .expect("helper should be written");
    std::fs::write(
        dir.join("unused_bigint.sifr"),
        "def unused() -> bigint:\n    return bigint(99)\n",
    )
    .expect("unused helper should be written");

    let binary = build_project(&main_file, &build_out)
        .expect("project build should ignore unreachable dependency metadata");
    assert!(binary.exists());

    let cargo_toml = std::fs::read_to_string(build_out.join("sifr_output").join("Cargo.toml"))
        .expect("cargo manifest should be written");
    assert!(!cargo_toml.contains("num-bigint = \"0.4.6\""));
    assert!(!cargo_toml.contains("num-traits = \"0.2.19\""));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_build_project_includes_reachable_support_module_stdlib_crates_in_manifest() {
    let dir = mktemp_dir("stdlib_positive");
    let main_file = dir.join("main.sifr");
    let build_out = dir.join("build_out");
    std::fs::write(
        &main_file,
        "from helper import render\n\ndef main():\n    print(render())\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "from sifr.tomllib import TomlValue, loads\n\n\
def render() -> str:\n    try:\n        parsed: TomlValue = loads(\"name = \\\"fixture-five\\\"\\nvalue = 5\")\n        return \"ok\"\n    except TOMLDecodeError as e:\n        return e.message\n",
    )
    .expect("helper should be written");

    let binary = build_project(&main_file, &build_out)
        .expect("project build should succeed with support-module stdlib dependencies");
    assert!(binary.exists());

    let cargo_toml = std::fs::read_to_string(build_out.join("sifr_output").join("Cargo.toml"))
        .expect("cargo manifest should be written");
    assert!(cargo_toml.contains("toml = { version = \"1.1.2\", features = [\"preserve_order\"] }"));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_build_project_manifest_ignores_unreachable_support_module_stdlib_crates() {
    let dir = mktemp_dir("stdlib_negative");
    let main_file = dir.join("main.sifr");
    let build_out = dir.join("build_out");
    std::fs::write(
        &main_file,
        "from helper import helper\n\ndef main():\n    print(helper())\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "def helper() -> int:\n    return 1\n",
    )
    .expect("helper should be written");
    std::fs::write(
        dir.join("unused_json.sifr"),
        "from sifr.tomllib import loads\n\n\
def unused() -> str:\n    try:\n        parsed: str = loads(\"name = \\\"unused\\\"\\nvalue = 1\")\n        return parsed\n    except TOMLDecodeError as e:\n        return e.message\n",
    )
    .expect("unused helper should be written");

    let binary = build_project(&main_file, &build_out)
        .expect("project build should ignore unreachable stdlib dependency metadata");
    assert!(binary.exists());

    let cargo_toml = std::fs::read_to_string(build_out.join("sifr_output").join("Cargo.toml"))
        .expect("cargo manifest should be written");
    assert!(!cargo_toml.contains("toml = { version = \"1.1.2\", features = [\"preserve_order\"] }"));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_build_project_includes_transitive_dependency_closure_in_manifest() {
    let dir = mktemp_dir("transitive_positive");
    let main_file = dir.join("main.sifr");
    let build_out = dir.join("build_out");
    std::fs::write(
        &main_file,
        "from helper import render\n\ndef main():\n    print(render())\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "from formatter import render_value\n\n\
def render() -> str:\n    return render_value()\n",
    )
    .expect("helper should be written");
    std::fs::write(
        dir.join("formatter.sifr"),
        "def render_value() -> str:\n    value: bigint = bigint(7)\n    return str(value)\n",
    )
    .expect("formatter should be written");

    let binary = build_project(&main_file, &build_out)
        .expect("project build should include transitive dependency closure");
    assert!(binary.exists());

    let cargo_toml = std::fs::read_to_string(build_out.join("sifr_output").join("Cargo.toml"))
        .expect("cargo manifest should be written");
    assert!(cargo_toml.contains("num-bigint = \"0.4.6\""));
    assert!(cargo_toml.contains("num-traits = \"0.2.19\""));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_build_project_manifest_ignores_unreachable_transitive_dependency_chain() {
    let dir = mktemp_dir("transitive_negative");
    let main_file = dir.join("main.sifr");
    let build_out = dir.join("build_out");
    std::fs::write(
        &main_file,
        "from helper import helper\n\ndef main():\n    print(helper())\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "def helper() -> int:\n    return 1\n",
    )
    .expect("helper should be written");
    std::fs::write(
        dir.join("unused_chain.sifr"),
        "from unused_formatter import render_value\n\n\
def unused() -> str:\n    return render_value()\n",
    )
    .expect("unused chain root should be written");
    std::fs::write(
        dir.join("unused_formatter.sifr"),
        "def render_value() -> str:\n    value: bigint = bigint(9)\n    return str(value)\n",
    )
    .expect("unused chain leaf should be written");

    let binary = build_project(&main_file, &build_out)
        .expect("project build should ignore unreachable transitive dependency chains");
    assert!(binary.exists());

    let cargo_toml = std::fs::read_to_string(build_out.join("sifr_output").join("Cargo.toml"))
        .expect("cargo manifest should be written");
    assert!(!cargo_toml.contains("num-bigint = \"0.4.6\""));
    assert!(!cargo_toml.contains("num-traits = \"0.2.19\""));

    let _ = std::fs::remove_dir_all(dir);
}
