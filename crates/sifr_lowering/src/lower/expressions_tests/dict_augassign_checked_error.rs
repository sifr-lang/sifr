use super::{DiagnosticCode, HirStmt, Type, lower_source, lower_source_with_stdlib_collections};

#[test]
fn plain_dict_augassign_requires_checked_error_handling() {
    let source = "def solve(mut values: dict[int, int], key: int):\n    values[key] += 3\n";
    let errors = lower_source(source).expect_err("fallible dict augassign should require try");
    assert!(errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::RESULT_UNUSED_VALUE)
                && error.message
                == "augmented subscript assignment may fail with 'KeyError'; handle it inside try/except"
    }), "unexpected diagnostics: {errors:#?}");
}

#[test]
fn handled_plain_dict_augassign_carries_key_error_in_hir() {
    let source = "def solve():\n    values: dict[int, int] = {1: 2}\n    try:\n        values[1] += 3\n    except KeyError:\n        pass\n";
    let module = lower_source(source).expect("handled dict augassign should lower");
    let function = module
        .functions
        .iter()
        .find(|function| function.name == "solve")
        .expect("solve should lower");
    let HirStmt::TryExcept {
        body,
        body_error_types,
        ..
    } = &function.body[1]
    else {
        panic!("expected try/except");
    };
    assert!(matches!(
        body_error_types.as_slice(),
        [Type::Class { name, .. }] if name == "KeyError"
    ));
    assert!(matches!(
        &body[0],
        HirStmt::SubscriptAugAssign {
            failure: Some(Type::Class { name, .. }),
            ..
        } if name == "KeyError"
    ));
}

#[test]
fn proven_present_plain_dict_key_has_no_key_failure() {
    let source = "def solve(mut values: dict[str, int], key: str):\n    if key in values:\n        values[key] += 1\n";
    let module = lower_source(source).expect("guarded dict augassign should lower");
    let function = module
        .functions
        .iter()
        .find(|function| function.name == "solve")
        .expect("solve should lower");
    let HirStmt::If { then_body, .. } = &function.body[0] else {
        panic!("expected membership guard");
    };
    assert!(matches!(
        &then_body[0],
        HirStmt::SubscriptAugAssign { failure: None, .. }
    ));
}

#[test]
fn preceding_subscript_assignment_proves_key_presence() {
    let source = "def solve(mut values: dict[str, int], key: str):\n    values[key] = 0\n    values[key] += 1\n";
    let module = lower_source(source).expect("assigned dict key should lower");
    let function = module
        .functions
        .iter()
        .find(|function| function.name == "solve")
        .expect("solve should lower");
    assert!(matches!(
        &function.body[1],
        HirStmt::SubscriptAugAssign { failure: None, .. }
    ));
}

#[test]
fn annotated_defaultdict_keeps_factory_semantics_and_declared_shape() {
    let source = "from sifr.collections import defaultdict\n\ndef solve():\n    values: dict[str, int] = defaultdict(int)\n    values[\"missing\"] += 3\n";
    let module = lower_source_with_stdlib_collections(source)
        .expect("annotated defaultdict augassign should lower without try");
    let function = module
        .functions
        .iter()
        .find(|function| function.name == "solve")
        .expect("solve should lower");
    assert!(matches!(
        &function.body[0],
        HirStmt::Let {
            ty: Type::Alias { name, body, .. },
            ..
        } if name == "__sifr_defaultdict_int"
            && matches!(body.as_ref(), Type::Dict(key, value) if key.as_ref() == &Type::Str && value.as_ref() == &Type::Int)
    ));
    assert!(matches!(
        &function.body[1],
        HirStmt::SubscriptAugAssign {
            object_ty: Type::Alias { name, .. },
            failure: None,
            ..
        } if name == "__sifr_defaultdict_int"
    ));
}
