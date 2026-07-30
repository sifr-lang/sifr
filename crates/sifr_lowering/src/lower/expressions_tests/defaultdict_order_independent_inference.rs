use super::{lower_source_with_stdlib_collections, DiagnosticCode, HirExpr, HirStmt, Type};
use crate::lower::builtin_calls::{DEFAULTDICT_LIST_ALIAS, DEFAULTDICT_SET_ALIAS};
use crate::MutableReceiverTarget;

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
fn defaultdict_iterable_mutators_carry_checked_backing_storage_places() {
    let source = "from sifr.collections import defaultdict\n\ndef solve() -> int:\n    lists = defaultdict(list)\n    lists[2].append(7)\n    lists[1].extend(lists[2])\n    sets = defaultdict(set)\n    sets[2].add(7)\n    sets[1].update(sets[2])\n    return len(lists[1]) + len(sets[1])\n";
    let module = lower_source_with_stdlib_collections(source).expect("source should lower");
    let function = module
        .functions
        .iter()
        .find(|function| function.name == "solve")
        .expect("solve should lower");
    let mut checked_methods = Vec::new();
    for statement in &function.body {
        let HirStmt::Expr {
            expr:
                HirExpr::MethodCall {
                    method,
                    receiver_target,
                    ..
                },
        } = statement
        else {
            continue;
        };
        if matches!(method.as_str(), "extend" | "update") {
            assert!(
                matches!(
                    receiver_target,
                    Some(MutableReceiverTarget::SpecializedIndexedStorage(_))
                ),
                "{method} should retain a checked defaultdict backing-storage place"
            );
            checked_methods.push(method.as_str());
        }
    }
    assert_eq!(checked_methods, ["extend", "update"]);
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
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].code,
        Some(DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT)
    );
    assert_eq!(
        errors[0].message,
        "set element type conflict: expected 'str', got 'int'"
    );
}

#[test]
fn conflicting_defaultdict_keys_report_container_conflict() {
    let source = "from sifr.collections import defaultdict\n\ndef solve():\n    values = defaultdict(set)\n    values[1].add(\"x\")\n    values[\"other\"].add(\"y\")\n";
    let errors =
        lower_source_with_stdlib_collections(source).expect_err("conflicting keys should fail");
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].code,
        Some(DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT)
    );
    assert_eq!(
        errors[0].message,
        "defaultdict key type conflict: expected 'int', got 'str'"
    );
}

#[test]
fn sibling_defaultdict_bindings_keep_independent_declaration_types() {
    let source = "from sifr.collections import defaultdict\n\ndef solve(flag: bool) -> int:\n    if flag:\n        values = defaultdict(set)\n        values[1].add(\"x\")\n        return len(values)\n    values = defaultdict(set)\n    values[\"key\"].add(2)\n    return len(values)\n";
    let module = lower_source_with_stdlib_collections(source).expect("source should lower");
    let function = module
        .functions
        .iter()
        .find(|function| function.name == "solve")
        .expect("solve should lower");
    let HirStmt::If { then_body, .. } = &function.body[0] else {
        panic!("first statement should remain the conditional");
    };
    let HirStmt::Let {
        ty: nested_ty,
        value: nested_value,
        ..
    } = &then_body[0]
    else {
        panic!("nested declaration should lower as a let");
    };
    let HirStmt::Let {
        ty: direct_ty,
        value: direct_value,
        ..
    } = &function.body[1]
    else {
        panic!("direct declaration should lower as a let");
    };
    assert_eq!(
        nested_ty,
        &defaultdict_type(
            DEFAULTDICT_SET_ALIAS,
            Type::Int,
            Type::Set(Box::new(Type::Str)),
        )
    );
    assert_eq!(nested_value.ty(), nested_ty);
    assert_eq!(
        direct_ty,
        &defaultdict_type(
            DEFAULTDICT_SET_ALIAS,
            Type::Any,
            Type::Set(Box::new(Type::Any)),
        )
    );
    assert_eq!(direct_value.ty(), direct_ty);
}

#[test]
fn lowering_inexact_index_elements_do_not_force_declaration_hints() {
    let list_source = "from sifr.collections import defaultdict\n\ndef solve(values: list[int]) -> int:\n    d = defaultdict(list)\n    d[1].append(values[0])\n    return len(d[1])\n";
    assert!(lower_source_with_stdlib_collections(list_source).is_ok());

    let set_source = "from sifr.collections import defaultdict\n\ndef solve(values: list[str]) -> int:\n    d = defaultdict(set)\n    d[1].add(values[0])\n    return len(d[1])\n";
    assert!(lower_source_with_stdlib_collections(set_source).is_ok());

    let tuple_key_source = "from sifr.collections import defaultdict\n\ndef solve(values: list[int], n: int) -> int:\n    squares = defaultdict(set)\n    key = (n, values[0])\n    squares[key].add(\"a\")\n    return len(squares)\n";
    assert!(lower_source_with_stdlib_collections(tuple_key_source).is_ok());
}

#[test]
fn lowering_inexact_call_results_and_rebindings_do_not_force_declaration_hints() {
    let get_element_source = "from sifr.collections import defaultdict\n\ndef solve(m: dict[int, int]) -> int:\n    d = defaultdict(list)\n    d[1].append(m.get(5))\n    return len(d[1])\n";
    let pop_element_source = "from sifr.collections import defaultdict\n\ndef solve(mut values: list[int]) -> int:\n    d = defaultdict(list)\n    d[1].append(values.pop())\n    return len(d[1])\n";
    let get_key_source = "from sifr.collections import defaultdict\n\ndef solve(m: dict[int, int]) -> int:\n    d = defaultdict(set)\n    k = m.get(5)\n    d[k].add(\"a\")\n    return len(d)\n";
    let loop_get_key_source = "from sifr.collections import defaultdict\n\ndef solve(m: dict[int, int]) -> int:\n    d = defaultdict(set)\n    for k in m:\n        v = m.get(k)\n        d[v].add(k)\n    return len(d)\n";
    let rebound_index_source = "from sifr.collections import defaultdict\n\ndef solve(values: list[int]) -> int:\n    d = defaultdict(list)\n    x = values[0]\n    x = 5\n    d[1].append(x)\n    return len(d[1])\n";

    for source in [
        get_element_source,
        pop_element_source,
        get_key_source,
        loop_get_key_source,
        rebound_index_source,
    ] {
        let (binding_ty, constructor_ty) = binding_and_constructor_types(source, "d");
        assert_eq!(binding_ty, constructor_ty);
        assert!(
            binding_ty.contains_unknown_or_any(),
            "lowering-inexact shape must not become a concrete declaration: {binding_ty:?}"
        );
    }
}

#[test]
fn nested_nonlocal_inexact_rebinding_does_not_force_declaration_hint() {
    let source = "from sifr.collections import defaultdict\n\ndef solve(values: list[int]) -> int:\n    d = defaultdict(list)\n    x = 5\n    def inner():\n        nonlocal x\n        x = values[0]\n    inner()\n    d[1].append(x)\n    return len(d[1])\n";
    let (binding_ty, constructor_ty) = binding_and_constructor_types(source, "d");
    assert_eq!(binding_ty, constructor_ty);
    assert!(
        binding_ty.contains_unknown_or_any(),
        "nested lowering-inexact rebinding must not become a concrete declaration: {binding_ty:?}"
    );
}

#[test]
fn tuple_key_with_unresolved_member_is_not_adopted() {
    let source = "from sifr.collections import defaultdict\n\nclass Point:\n    x: int\n\ndef solve(p: Point) -> int:\n    d = defaultdict(set)\n    key = (p.x, 1)\n    d[key].add(\"a\")\n    return len(d)\n";
    assert!(lower_source_with_stdlib_collections(source).is_ok());
}

#[test]
fn incomplete_defaultdict_nested_return_reports_missing_annotation() {
    let source = "from sifr.collections import defaultdict\n\ndef solve() -> int:\n    rows = defaultdict(set)\n    def peek(k: int):\n        return rows[k]\n    return len(peek(1))\n";
    let errors = lower_source_with_stdlib_collections(source)
        .expect_err("incomplete nested return type should be rejected");
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(
        errors[0].code,
        Some(DiagnosticCode::TYPE_MISSING_ANNOTATION)
    );
    assert_eq!(
        errors[0].message,
        "function 'peek' return type could not be inferred deterministically"
    );
}

#[test]
fn nested_defaultdict_shadow_does_not_merge_with_outer_hint() {
    let source = "from sifr.collections import defaultdict\n\ndef solve() -> int:\n    values = defaultdict(set)\n    values[1].add(\"outer\")\n    def inner() -> int:\n        values = defaultdict(set)\n        values[\"inner\"].add(2)\n        return len(values)\n    return len(values) + inner()\n";
    assert!(lower_source_with_stdlib_collections(source).is_ok());
}
