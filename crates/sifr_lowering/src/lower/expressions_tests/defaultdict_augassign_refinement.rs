use super::{lower_source_with_stdlib_collections, DiagnosticCode, HirStmt, Type};

fn defaultdict_binding_types(source: &str, binding: &str) -> (Type, Type) {
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
        .expect("defaultdict binding should lower")
}

fn defaultdict_int_type(key_ty: Type) -> Type {
    Type::alias(
        "__sifr_defaultdict_int",
        Type::Dict(Box::new(key_ty), Box::new(Type::Int)),
    )
}

#[test]
fn defaultdict_int_augassign_refines_variable_string_key_at_declaration() {
    let source = "from sifr.collections import defaultdict\n\ndef solve(tasks: list[str]) -> int:\n    counts = defaultdict(int)\n    for task in tasks:\n        counts[task] += 1\n    total = 0\n    for _, value in counts.items():\n        total += value\n    return total\n";
    let expected = defaultdict_int_type(Type::Str);
    let (binding_ty, constructor_ty) = defaultdict_binding_types(source, "counts");
    assert_eq!(binding_ty, expected);
    assert_eq!(constructor_ty, expected);
}

#[test]
fn defaultdict_int_augassign_refines_variable_integer_key_at_declaration() {
    let source = "from sifr.collections import defaultdict\n\ndef solve(items: list[int]) -> int:\n    counts = defaultdict(int)\n    for item in items:\n        counts[item] += 1\n    return len(counts)\n";
    let expected = defaultdict_int_type(Type::Int);
    let (binding_ty, constructor_ty) = defaultdict_binding_types(source, "counts");
    assert_eq!(binding_ty, expected);
    assert_eq!(constructor_ty, expected);
}

#[test]
fn defaultdict_int_literal_key_widens_to_mutable_base_type() {
    let source = "from sifr.collections import defaultdict\n\ndef solve() -> int:\n    counts = defaultdict(int)\n    counts[\"steps\"] += 1\n    counts[\"later\"] += 2\n    return counts[\"steps\"]\n";
    let expected = defaultdict_int_type(Type::Str);
    let (binding_ty, constructor_ty) = defaultdict_binding_types(source, "counts");
    assert_eq!(binding_ty, expected);
    assert_eq!(constructor_ty, expected);
}

#[test]
fn defaultdict_int_conflicting_augassign_key_is_rejected() {
    let source = "from sifr.collections import defaultdict\n\ndef solve():\n    counts = defaultdict(int)\n    counts[\"steps\"] += 1\n    counts[2] += 1\n";
    let errors = lower_source_with_stdlib_collections(source)
        .expect_err("conflicting defaultdict keys should fail");
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message
                == "dict subscript assignment key type 'int' is not compatible with dict key type 'str'"
    }));
}

#[test]
fn initialized_defaultdict_int_keeps_declared_key_type() {
    let source = "from sifr.collections import defaultdict\n\ndef solve(seed: dict[str, int]):\n    counts = defaultdict(int, seed)\n    counts[1] += 1\n";
    let errors = lower_source_with_stdlib_collections(source)
        .expect_err("initialized defaultdict key type should not be replaced");
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message
                == "dict subscript assignment key type 'int' is not compatible with dict key type 'str'"
    }));
}
