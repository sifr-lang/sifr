use super::{DiagnosticCode, HirStmt, Type, lower_source};

fn local_binding(source: &str, name: &str) -> (Type, Type) {
    let module = lower_source(source).expect("source should lower");
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
                name: binding_name,
                ty,
                value,
                ..
            } if binding_name == name => Some((ty.clone(), value.ty().clone())),
            _ => None,
        })
        .expect("binding should lower")
}

#[test]
fn empty_plain_dict_uses_later_subscript_write_at_declaration() {
    let source = "def solve(nums: list[int]):\n    seen = {}\n    for i, n in enumerate(nums):\n        seen[n] = i\n";
    let expected = Type::Dict(Box::new(Type::Int), Box::new(Type::Int));
    let (binding_ty, literal_ty) = local_binding(source, "seen");
    assert_eq!(binding_ty, expected);
    assert_eq!(literal_ty, expected);
}

#[test]
fn empty_plain_dict_inference_is_independent_of_read_before_write_order() {
    let source = "def solve(nums: list[int], target: int) -> list[int]:\n    seen = {}\n    for i, n in enumerate(nums):\n        diff = target - n\n        if diff in seen:\n            return [seen[diff], i]\n        seen[n] = i\n    return []\n";
    let expected = Type::Dict(Box::new(Type::Int), Box::new(Type::Int));
    let (binding_ty, literal_ty) = local_binding(source, "seen");
    assert_eq!(binding_ty, expected);
    assert_eq!(literal_ty, expected);
}

#[test]
fn conflicting_plain_dict_writes_keep_deterministic_container_diagnostic() {
    let source = "def solve():\n    data = {}\n    data[1] = 10\n    data[\"x\"] = 20\n";
    let errors = lower_source(source).expect_err("conflicting writes should fail");
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT)
            && error.message
                == "empty literal type conflict for 'data': expected key 'int' and value 'int', got key 'str' and value 'int'"
    }));
}

#[test]
fn widening_numeric_dict_writes_keep_container_conflict_diagnostic() {
    let source = "def solve():\n    data = {}\n    data[1] = 4\n    data[2] = 2.5\n";
    let errors = lower_source(source).expect_err("widening writes should fail");
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT)
            && error.message
                == "empty literal type conflict for 'data': expected key 'int' and value 'int', got key 'int' and value 'float'"
    }));
}

#[test]
fn widening_class_dict_writes_keep_container_conflict_diagnostic() {
    let source = "class Base:\n    value: int\n\nclass Derived(Base):\n    extra: int\n\ndef solve():\n    data = {}\n    data[1] = Derived(1, 2)\n    data[2] = Base(2)\n";
    let errors = lower_source(source).expect_err("widening writes should fail");
    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT))
    );
}

#[test]
fn widening_unhashable_key_reports_one_capability_error_and_the_conflict() {
    let source = "def solve():\n    data = {}\n    data[1] = \"a\"\n    data[2.5] = \"b\"\n";
    let errors = lower_source(source).expect_err("unhashable widening key should fail");
    let hash_errors = errors
        .iter()
        .filter(|error| {
            error.message
                == "dict subscript assignment requires a key type with generated Rust Eq + Hash traits, unavailable for 'float'"
        })
        .count();
    assert_eq!(hash_errors, 1);
    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT))
    );
}

#[test]
fn missing_key_augassign_before_plain_dict_write_stays_unsupported() {
    let source = "def solve(words: list[str]) -> int:\n    counts = {}\n    for word in words:\n        counts[word] += 1\n    counts[\"seed\"] = 0\n    return len(counts)\n";
    let errors = lower_source(source).expect_err("missing-key augassign should remain unsupported");
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
            && error.message == "unsupported operand type(s) for +: 'Any' and 'int'"
    }));
}

#[test]
fn empty_plain_dict_without_concrete_evidence_remains_dynamic() {
    let source = "def solve():\n    data = {}\n    print(len(data))\n";
    let expected = Type::Dict(Box::new(Type::Any), Box::new(Type::Any));
    let (binding_ty, literal_ty) = local_binding(source, "data");
    assert_eq!(binding_ty, expected);
    assert_eq!(literal_ty, expected);
}

#[test]
fn empty_plain_dict_hint_does_not_cross_sibling_scope_binding() {
    let source = "def solve(flag: bool) -> int:\n    if flag:\n        data = {}\n        data[\"k\"] = 1\n        return len(data)\n    data = {}\n    data[2] = 3\n    return len(data)\n";
    assert!(lower_source(source).is_ok());
}

#[test]
fn compatible_sibling_scope_hint_does_not_retype_later_binding() {
    let source = "def solve(flag: bool) -> int:\n    if flag:\n        data = {}\n        data[1] = 2.5\n        return len(data)\n    data = {}\n    data[3] = 4\n    return len(data)\n";
    let module = lower_source(source).expect("source should lower");
    let function = module
        .functions
        .iter()
        .find(|function| function.name == "solve")
        .expect("solve should lower");
    let HirStmt::Let {
        name, ty, value, ..
    } = &function.body[1]
    else {
        panic!("later data binding should be a direct let");
    };
    assert_eq!(name, "data");
    let expected = Type::Dict(Box::new(Type::Int), Box::new(Type::Int));
    assert_eq!(ty, &expected);
    assert_eq!(value.ty(), &expected);
}

#[test]
fn preexisting_nested_function_hint_keeps_type_mismatch_diagnostic() {
    let source = "def solve():\n    data = {}\n    def nested() -> int:\n        return 1\n    data[1] = nested()\n    data[\"x\"] = 2\n";
    let errors = lower_source(source).expect_err("conflicting writes should fail");
    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH))
    );
    assert!(
        !errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT))
    );
}

#[test]
fn nested_function_block_keeps_concrete_dict_declaration_and_literal_hir() {
    let source = "def solve():\n    data = {}\n    def nested() -> int:\n        return 1\n    data[1] = nested()\n";
    let expected = Type::Dict(Box::new(Type::Int), Box::new(Type::Int));
    let (binding_ty, literal_ty) = local_binding(source, "data");
    assert_eq!(binding_ty, expected);
    assert_eq!(literal_ty, expected);
}

#[test]
fn loop_local_dict_hint_does_not_retype_later_function_binding() {
    let source = "def solve(items: list[int]) -> int:\n    total = 0\n    for item in items:\n        data = {}\n        data[item] = item\n        total += len(data)\n    data = {}\n    data[\"a\"] = 1\n    return total + len(data)\n";
    assert!(lower_source(source).is_ok());
}
