use super::{lower_source, lower_source_with_stdlib_collections, DiagnosticCode, HirStmt, Type};
use crate::lower::builtin_calls::{DEFAULTDICT_LIST_ALIAS, DEFAULTDICT_SET_ALIAS};
use crate::lower::defaultdict_refinement::refine_defaultdict_int_augassign_key;
use crate::lower::LowerCtx;

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

#[test]
fn nested_function_shadowing_keeps_independent_defaultdict_key_types() {
    let source = "from sifr.collections import defaultdict\n\ndef solve(words: list[str], nums: list[int]) -> int:\n    counts = defaultdict(int)\n    def helper() -> int:\n        counts = defaultdict(int)\n        for n in nums:\n            counts[n] += 1\n        return len(counts)\n    for word in words:\n        counts[word] += 1\n    return len(counts) + helper()\n";
    let module = lower_source_with_stdlib_collections(source).expect("source should lower");
    let function = module
        .functions
        .iter()
        .find(|function| function.name == "solve")
        .expect("solve should lower");
    let outer_ty = function
        .body
        .iter()
        .find_map(|stmt| match stmt {
            HirStmt::Let { name, ty, .. } if name == "counts" => Some(ty),
            _ => None,
        })
        .expect("outer counts should lower");
    let inner_ty = function
        .body
        .iter()
        .find_map(|stmt| match stmt {
            HirStmt::NestedFunction { func, .. } if func.name == "helper" => {
                func.body.iter().find_map(|stmt| match stmt {
                    HirStmt::Let { name, ty, .. } if name == "counts" => Some(ty),
                    _ => None,
                })
            }
            _ => None,
        })
        .expect("inner counts should lower");
    assert_eq!(outer_ty, &defaultdict_int_type(Type::Str));
    assert_eq!(inner_ty, &defaultdict_int_type(Type::Int));
}

#[test]
fn late_nested_shadow_does_not_clear_enclosing_defaultdict_patch() {
    let source = "from sifr.collections import defaultdict\n\ndef solve(words: list[str], nums: list[int]) -> int:\n    counts = defaultdict(int)\n    if len(words) > 0:\n        for word in words:\n            counts[word] += 1\n        def helper() -> int:\n            counts = defaultdict(int)\n            for value in nums:\n                counts[value] += 1\n            return len(counts)\n        return len(counts) + helper()\n    return 0\n";
    let (binding_ty, value_ty) = defaultdict_binding_types(source, "counts");
    let expected = defaultdict_int_type(Type::Str);
    assert_eq!(binding_ty, expected);
    assert_eq!(value_ty, expected);
}

#[test]
fn defaultdict_list_and_set_aliases_are_not_retyped_by_int_augassign_refiner() {
    let mut ctx = LowerCtx::new();
    for (alias, value_ty) in [
        (DEFAULTDICT_LIST_ALIAS, Type::List(Box::new(Type::Any))),
        (DEFAULTDICT_SET_ALIAS, Type::Set(Box::new(Type::Any))),
    ] {
        let original = Type::alias(alias, Type::Dict(Box::new(Type::Any), Box::new(value_ty)));
        let actual =
            refine_defaultdict_int_augassign_key("items", original.clone(), &Type::Str, &mut ctx);
        assert_eq!(actual, original);
    }
}

#[test]
fn unhashable_defaultdict_int_augassign_key_reports_one_capability_error() {
    let source = "from sifr.collections import defaultdict\n\ndef solve():\n    counts = defaultdict(int)\n    counts[1.5] += 1\n";
    let errors =
        lower_source_with_stdlib_collections(source).expect_err("float key should be rejected");
    let capability_errors = errors
        .iter()
        .filter(|error| {
            error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message
                    == "dict augmented subscript assignment requires a key type with generated Rust Eq + Hash traits, unavailable for 'float'"
        })
        .count();
    assert_eq!(capability_errors, 1);
}

#[test]
fn plain_dict_missing_key_augassign_remains_rejected() {
    let source = "def solve(words: list[str]):\n    counts = {}\n    for word in words:\n        counts[word] += 1\n";
    let errors = lower_source(source).expect_err("plain dict missing-key augassign should fail");
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
            && error.message == "unsupported operand type(s) for +: 'Any' and 'int'"
    }));
}
