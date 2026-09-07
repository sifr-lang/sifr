use super::generate_rust_from_source;

#[test]
fn nested_capture_call_result_refines_outer_list_codegen() {
    let generated = generate_rust_from_source(
        "def solve() -> list[str]:\n    called = []\n    def add(value: str):\n        called.append(\"\".join([value]))\n    add(\"x\")\n    return called\n",
    );
    assert!(
        generated.contains("let mut called: Vec<String> = Vec::<String>::new();"),
        "{generated}"
    );
    assert!(
        generated.contains("called.push(vec![value.to_owned()].join(\"\"));"),
        "{generated}"
    );
    assert!(
        !generated.contains("let mut called: Vec<Box<dyn"),
        "{generated}"
    );
}

#[test]
fn multilevel_nested_capture_refines_outer_list_codegen() {
    let generated = generate_rust_from_source(
        "def solve() -> list[str]:\n    called = []\n    def middle():\n        def add(value: str):\n            called.append(\"\".join([value]))\n        add(\"x\")\n    middle()\n    return called\n",
    );
    assert!(
        generated.contains("let mut called: Vec<String> = Vec::<String>::new();"),
        "{generated}"
    );
    assert!(
        !generated.contains("let mut called: Vec<Box<dyn"),
        "{generated}"
    );
}

#[test]
fn nested_same_named_lists_keep_independent_codegen_types() {
    let generated = generate_rust_from_source(
        "def solve() -> list[int]:\n    values = []\n    def add() -> int:\n        values = []\n        values.append(\"inner\")\n        return len(values)\n    values.append(1)\n    assert add() == 1\n    return values\n",
    );
    assert!(
        generated.contains("let mut values: Vec<SifrInt> = vec![];"),
        "{generated}"
    );
    assert!(
        generated.contains("let mut values: Vec<String> = Vec::<String>::new();"),
        "{generated}"
    );
}
