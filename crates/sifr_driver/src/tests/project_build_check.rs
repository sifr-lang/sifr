use crate::{CompileResult, build_cached_project, build_project, check_project, emit_project};
use sifr_diagnostics::{DiagnosticCode, render_compact_diagnostics};

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

    let errors = check_project(
        &dir.join("main.sifr"),
        &mut sifr_frontend::DiskSourceProvider::new(),
    );
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

    let errors = check_project(&main_file, &mut sifr_frontend::DiskSourceProvider::new());
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
    std::fs::write(dir.join("sifr.toml"), "[source]\nroot = \"lib\"\n")
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

    let errors = check_project(
        &dir.join("cases/app.sifr"),
        &mut sifr_frontend::DiskSourceProvider::new(),
    );

    assert!(
        errors.is_empty(),
        "workspace check should succeed: {errors:?}"
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_project_materializes_dotted_workspace_modules() {
    let dir = mktemp_dir("workspace_dotted_build");
    let main_file = dir.join("cases/app.sifr");
    let build_out = dir.join("build_out");
    std::fs::create_dir_all(dir.join("cases")).expect("cases dir should be created");
    std::fs::create_dir_all(dir.join("lib/helpers")).expect("helpers dir should be created");
    std::fs::write(dir.join("sifr.toml"), "[source]\nroot = \"lib\"\n")
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

    let binary = build_project(
        &main_file,
        &build_out,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
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
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_project_preserves_imported_class_constructors_and_signatures() {
    let dir = mktemp_dir("workspace_imported_class_codegen");
    let main_file = dir.join("cases/app.sifr");
    let build_out = dir.join("build_out");
    std::fs::create_dir_all(dir.join("cases")).expect("cases dir should be created");
    std::fs::create_dir_all(dir.join("lib/helpers")).expect("helpers dir should be created");
    std::fs::write(dir.join("sifr.toml"), "[source]\nroot = \"lib\"\n")
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

    let binary = build_project(
        &main_file,
        &build_out,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("imported class constructors should build successfully");

    assert!(binary.exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_project_keeps_aliased_same_name_generic_classes_distinct() {
    let dir = mktemp_dir("workspace_aliased_same_name_generics");
    let main_file = dir.join("cases/app.sifr");
    let build_out = dir.join("build_out");
    std::fs::create_dir_all(dir.join("cases")).expect("cases dir should be created");
    std::fs::create_dir_all(dir.join("lib/helpers")).expect("helpers dir should be created");
    std::fs::write(dir.join("sifr.toml"), "[source]\nroot = \"lib\"\n")
        .expect("manifest should be written");
    std::fs::write(
        &main_file,
        r#"
from helpers.factories import make_left
from helpers.types import Right as RightBox
from helpers.types import Left as LeftBox
from helpers.factories import make_right
from helpers.roots import LeftRoot as RootAlias
from helpers.leaves import LeftLeaf as LeafAlias

def left_value(value: LeftBox[int]) -> int:
    return value.value

def right_value(value: RightBox[int]) -> int:
    return value.value

def leaf_end(value: LeafAlias) -> int:
    return value.end

def main():
    left: LeftBox[int] = make_left()
    right: RightBox[int] = make_right()
    assert left_value(left) == 7
    assert right_value(right) == 3
    root: RootAlias = RootAlias(9)
    assert root.base == 9
    leaf: LeafAlias = LeafAlias(1, 2, 3)
    assert leaf_end(leaf) == 3
"#,
    )
    .expect("entry should be written");
    std::fs::write(
        dir.join("lib/helpers/left.sifr"),
        r#"
class Box[T]:
    value: T

    def same(self, other: T) -> bool:
        return self.value == other

def make() -> Box[int]:
    return Box(7)

class Root:
    base: int

class Mid(Root):
    middle: int

    def __init__(self, base: int, middle: int):
        super().__init__(base)
        self.middle = middle

class Leaf(Mid):
    end: int

    def __init__(self, base: int, middle: int, end: int):
        super().__init__(base, middle)
        self.end = end

"#,
    )
    .expect("left helper should be written");
    std::fs::write(
        dir.join("lib/helpers/right.sifr"),
        r#"
class Box[T]:
    value: T

    def negated(self) -> T:
        return -self.value

def make() -> Box[int]:
    return Box(3)
"#,
    )
    .expect("right helper should be written");
    std::fs::write(
        dir.join("lib/helpers/types.sifr"),
        r#"
from helpers.right import Box as Right
from helpers.left import Box as Left
"#,
    )
    .expect("type facade should be written");
    std::fs::write(
        dir.join("lib/helpers/factories.sifr"),
        "from helpers.left import make as make_left\nfrom helpers.right import make as make_right\n",
    )
    .expect("factory facade should be written");
    std::fs::write(
        dir.join("lib/helpers/roots.sifr"),
        "from helpers.left import Root as LeftRoot\n",
    )
    .expect("root facade should be written");
    std::fs::write(
        dir.join("lib/helpers/leaves.sifr"),
        "from helpers.left import Leaf as LeftLeaf\n",
    )
    .expect("leaf facade should be written");

    let binary = build_project(
        &main_file,
        &build_out,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("aliased same-name generic classes should build successfully");

    assert!(binary.exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_check_project_preserves_aliased_import_ancestry() {
    let dir = mktemp_dir("workspace_aliased_ancestry");
    std::fs::write(
        dir.join("main.sifr"),
        r#"
from roots import Root as R
from children import Child as C

def base_of(value: R) -> int:
    return value.base

def main():
    child: C = C(1, 2, 3)
    assert base_of(child) == 1
"#,
    )
    .expect("entry should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        r#"
class Root:
    base: int

class Parent(Root):
    middle: int

    def __init__(self, base: int, middle: int):
        super().__init__(base)
        self.middle = middle

class Child(Parent):
    extra: int

    def __init__(self, base: int, middle: int, extra: int):
        super().__init__(base, middle)
        self.extra = extra
"#,
    )
    .expect("helper should be written");
    std::fs::write(dir.join("roots.sifr"), "from helper import Root\n")
        .expect("root facade should be written");
    std::fs::write(dir.join("children.sifr"), "from helper import Child\n")
        .expect("child facade should be written");

    let errors = check_project(
        &dir.join("main.sifr"),
        &mut sifr_frontend::DiskSourceProvider::new(),
    );
    assert!(
        errors.is_empty(),
        "aliased ancestry should check: {errors:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_check_project_rejects_cross_reexported_same_name_classes() {
    let dir = mktemp_dir("workspace_reexported_same_name_classes");
    std::fs::write(
        dir.join("main.sifr"),
        r#"
from types import Left as L
from factories import make_right
from types import Right as R
from factories import make_left

def main():
    left: L[int] = make_left()
    right: R[int] = make_right()
    invalid: L[int] = right
"#,
    )
    .expect("entry should be written");
    std::fs::write(
        dir.join("left.sifr"),
        "class Box[T]:\n    value: T\n\ndef make() -> Box[int]:\n    return Box(1)\n",
    )
    .expect("left helper should be written");
    std::fs::write(
        dir.join("right.sifr"),
        "class Box[T]:\n    value: T\n\ndef make() -> Box[int]:\n    return Box(2)\n",
    )
    .expect("right helper should be written");
    std::fs::write(
        dir.join("types.sifr"),
        "from right import Box as Right\nfrom left import Box as Left\n",
    )
    .expect("type facade should be written");
    std::fs::write(
        dir.join("factories.sifr"),
        "from left import make as make_left\nfrom right import make as make_right\n",
    )
    .expect("factory facade should be written");

    let errors = check_project(
        &dir.join("main.sifr"),
        &mut sifr_frontend::DiskSourceProvider::new(),
    );
    assert!(
        errors.iter().any(|error| {
            error.code == DiagnosticCode::TYPE_MISMATCH.code()
                && error.message.contains("expected 'L[int]', got 'R[int]'")
        }),
        "cross-reexport assignment should be rejected: {errors:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_check_project_rejects_reexported_generic_method_bound_violation() {
    let dir = mktemp_dir("workspace_reexported_generic_method_bounds");
    std::fs::write(
        dir.join("main.sifr"),
        r#"
from facade import PublicBox as Box

class Local(NonSend):
    pass

def invalid(value: Box[Local], other: Local) -> bool:
    return value.same(other)
"#,
    )
    .expect("entry should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "class Box[T]:\n    value: T\n\n    def same(self, other: T) -> bool:\n        return self.value == other\n",
    )
    .expect("helper should be written");
    std::fs::write(
        dir.join("facade.sifr"),
        "from helper import Box as PublicBox\n",
    )
    .expect("facade should be written");

    let errors = check_project(
        &dir.join("main.sifr"),
        &mut sifr_frontend::DiskSourceProvider::new(),
    );
    assert!(
        errors.iter().any(|error| {
            error.code == DiagnosticCode::TYPE_MISMATCH.code()
                && error.message.contains("Box.same() is unavailable")
                && error.message.contains("lacks Clone + PartialEq")
        }),
        "re-exported generic method bounds should reject invalid specialization: {errors:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_emit_project_includes_workspace_support_modules() {
    let dir = mktemp_dir("workspace_emit");
    let main_file = dir.join("cases/app.sifr");
    std::fs::create_dir_all(dir.join("cases")).expect("cases dir should be created");
    std::fs::create_dir_all(dir.join("lib/helpers")).expect("helpers dir should be created");
    std::fs::write(dir.join("sifr.toml"), "[source]\nroot = \"lib\"\n")
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

    let emitted = emit_project(&main_file, &mut sifr_frontend::DiskSourceProvider::new());

    let CompileResult::Success { rust_source } = emitted else {
        panic!("workspace project emit should succeed");
    };
    assert!(rust_source.contains("// src/main.rs"));
    assert!(rust_source.contains("mod helpers;"));
    assert!(rust_source.contains("// src/helpers/value.rs"));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_cached_project_invalidates_when_workspace_helper_changes() {
    let dir = mktemp_dir("workspace_cache_invalidation");
    let main_file = dir.join("cases/app.sifr");
    let helper_file = dir.join("lib/helpers/value.sifr");
    std::fs::create_dir_all(dir.join("cases")).expect("cases dir should be created");
    std::fs::create_dir_all(dir.join("lib/helpers")).expect("helpers dir should be created");
    std::fs::write(dir.join("sifr.toml"), "[source]\nroot = \"lib\"\n")
        .expect("manifest should be written");
    std::fs::write(
        &main_file,
        "from helpers.value import answer\n\ndef main():\n    print(answer())\n",
    )
    .expect("entry should be written");
    std::fs::write(&helper_file, "def answer() -> int:\n    return 10\n")
        .expect("helper should be written");

    let first = build_cached_project(&main_file, &mut sifr_frontend::DiskSourceProvider::new())
        .expect("first workspace build should succeed");
    std::fs::write(&helper_file, "def answer() -> int:\n    return 11\n")
        .expect("helper should be updated");
    let second = build_cached_project(&main_file, &mut sifr_frontend::DiskSourceProvider::new())
        .expect("second workspace build should succeed");

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

    let errors = check_project(
        &dir.join("main.sifr"),
        &mut sifr_frontend::DiskSourceProvider::new(),
    );
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

    let errors = check_project(
        &dir.join("main.sifr"),
        &mut sifr_frontend::DiskSourceProvider::new(),
    );
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

    let check_errors = check_project(
        &dir.join("main.sifr"),
        &mut sifr_frontend::DiskSourceProvider::new(),
    );
    let build_errors = build_project(
        &dir.join("main.sifr"),
        &dir.join("build_out"),
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect_err("build_project should fail with same frontend error");

    let check_messages = render_compact_diagnostics(&check_errors);
    let build_messages = render_compact_diagnostics(&build_errors);
    assert_eq!(check_messages, build_messages);
    assert!(build_messages.contains("return type mismatch"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
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

    let binary = build_project(
        &main_file,
        &build_out,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("project build should succeed with support-module stdlib dependencies");
    assert!(binary.exists());

    let cargo_toml = std::fs::read_to_string(build_out.join("sifr_output").join("Cargo.toml"))
        .expect("cargo manifest should be written");
    assert!(cargo_toml.contains("sifr_stdlib = { path = "));
    assert!(cargo_toml.contains("default-features = false"));
    assert!(
        cargo_toml
            .lines()
            .any(|line| line.starts_with("sifr_stdlib = ") && line.contains("\"toml\""))
    );
    assert!(!cargo_toml.contains("toml = { version"));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
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

    let binary = build_project(
        &main_file,
        &build_out,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("project build should ignore unreachable stdlib dependency metadata");
    assert!(binary.exists());

    let cargo_toml = std::fs::read_to_string(build_out.join("sifr_output").join("Cargo.toml"))
        .expect("cargo manifest should be written");
    assert!(
        !cargo_toml.contains("toml = { version = \"1.1.4\", features = [\"preserve_order\"] }")
    );

    let _ = std::fs::remove_dir_all(dir);
}
