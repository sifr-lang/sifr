use super::{lower_source_with_stdlib_collections, DiagnosticCode, HirStmt, Type};
use crate::lower::builtin_calls::{DEFAULTDICT_LIST_ALIAS, DEFAULTDICT_SET_ALIAS};

fn binding_and_constructor_types(source: &str, binding: &str) -> (Type, Type) {
    let module = lower_source_with_stdlib_collections(source).expect("source should lower");
    let function = module
        .functions
        .iter()
        .find(|function| function.name == "solve")
        .expect("solve should lower");
    function
        .body
        .iter()
        .find_map(|stmt| match stmt {
            HirStmt::Let {
                name, ty, value, ..
            } if name == binding => Some((ty.clone(), value.ty().clone())),
            _ => None,
        })
        .expect("binding should lower")
}

fn defaultdict_type(alias: &str, key: Type, value: Type) -> Type {
    Type::alias(alias, Type::Dict(Box::new(key), Box::new(value)))
}

#[test]
fn defaultdict_set_uses_later_add_shape_before_first_read() {
    let source = "from sifr.collections import defaultdict\n\ndef solve(cells: list[tuple[int, str]]) -> bool:\n    rows = defaultdict(set)\n    for row, cell in cells:\n        if cell in rows[row]:\n            return False\n        rows[row].add(cell)\n    return True\n";
    let expected = defaultdict_type(
        DEFAULTDICT_SET_ALIAS,
        Type::Int,
        Type::Set(Box::new(Type::Str)),
    );
    let (binding_ty, constructor_ty) = binding_and_constructor_types(source, "rows");
    assert_eq!(binding_ty, expected);
    assert_eq!(constructor_ty, expected);
}

#[test]
fn defaultdict_set_infers_tuple_keys_and_string_elements() {
    let source = "from sifr.collections import defaultdict\n\ndef solve(cell: str) -> int:\n    squares = defaultdict(set)\n    square = (1, 2)\n    if cell in squares[square]:\n        return 0\n    squares[square].add(cell)\n    return len(squares)\n";
    let expected = defaultdict_type(
        DEFAULTDICT_SET_ALIAS,
        Type::Tuple(vec![Type::Int, Type::Int]),
        Type::Set(Box::new(Type::Str)),
    );
    let (binding_ty, constructor_ty) = binding_and_constructor_types(source, "squares");
    assert_eq!(binding_ty, expected);
    assert_eq!(constructor_ty, expected);
}

#[test]
fn defaultdict_list_uses_later_append_shape() {
    let source = "from sifr.collections import defaultdict\n\ndef solve(word: str) -> int:\n    groups = defaultdict(list)\n    groups[1].append(word)\n    return len(groups[1])\n";
    let expected = defaultdict_type(
        DEFAULTDICT_LIST_ALIAS,
        Type::Int,
        Type::List(Box::new(Type::Str)),
    );
    let (binding_ty, constructor_ty) = binding_and_constructor_types(source, "groups");
    assert_eq!(binding_ty, expected);
    assert_eq!(constructor_ty, expected);
}

#[test]
fn conflicting_defaultdict_set_elements_report_container_conflict() {
    let source = "from sifr.collections import defaultdict\n\ndef solve():\n    values = defaultdict(set)\n    values[1].add(\"x\")\n    values[1].add(2)\n";
    let errors =
        lower_source_with_stdlib_collections(source).expect_err("conflicting elements should fail");
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT)
            && error.message == "set element type conflict: expected 'str', got 'int'"
    }));
}

#[test]
fn conflicting_defaultdict_keys_report_container_conflict() {
    let source = "from sifr.collections import defaultdict\n\ndef solve():\n    values = defaultdict(set)\n    values[1].add(\"x\")\n    values[\"other\"].add(\"y\")\n";
    let errors =
        lower_source_with_stdlib_collections(source).expect_err("conflicting keys should fail");
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT)
            && error.message == "defaultdict key type conflict: expected 'int', got 'str'"
    }));
}

#[test]
fn sibling_defaultdict_bindings_keep_independent_shapes() {
    let source = "from sifr.collections import defaultdict\n\ndef solve(flag: bool) -> int:\n    if flag:\n        values = defaultdict(set)\n        values[1].add(\"x\")\n        return len(values)\n    values = defaultdict(set)\n    values[\"key\"].add(2)\n    return len(values)\n";
    assert!(lower_source_with_stdlib_collections(source).is_ok());
}
