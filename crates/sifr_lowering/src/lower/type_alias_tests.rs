use crate::{HirDiagnostic, HirModule, HirStmt, lower_module};
use ruff_text_size::{TextRange, TextSize};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;
use sifr_type_system::{FixedIntType, Type};

fn lower_source(source: &str) -> Result<HirModule, Vec<HirDiagnostic>> {
    let parsed = parse_module(source).expect("parse failed");
    lower_module(parsed.suite()).map(|result| result.module)
}

fn range_for_after(source: &str, after: &str, needle: &str) -> TextRange {
    let after_start = source.find(after).expect("anchor should exist");
    let relative_start = source[after_start..]
        .find(needle)
        .expect("needle should exist after anchor");
    let start = u32::try_from(after_start + relative_start).expect("fixture offset must fit u32");
    let needle_len = u32::try_from(needle.len()).expect("fixture length must fit u32");
    TextRange::new(TextSize::new(start), TextSize::new(start + needle_len))
}

#[test]
fn test_forward_type_alias_resolves_independent_of_declaration_order() {
    let result = lower_source(
        "type Payload = Response\ntype Response = list[int]\n\ndef main():\n    data: Payload = [1, 2, 3]\n    print(len(data))\n",
    );
    assert!(
        result.is_ok(),
        "forward type aliases should resolve deterministically"
    );
}

#[test]
fn test_class_alias_cannot_supply_parent_identity() {
    let source = "class Parent:\n    value: int\n\ntype ParentAlias = Parent\n\nclass Child(ParentAlias):\n    pass\n";
    let errors = lower_source(source).expect_err("class alias base must fail before inheritance");
    assert!(errors.iter().any(|error| {
        error.message == "invalid base class for 'Child': parent type 'ParentAlias' is not a class"
            && error.code == Some(DiagnosticCode::CLASS_INVALID_BASE)
            && error.primary_range == Some(range_for_after(source, "class Child(", "ParentAlias"))
    }));
}

#[test]
fn test_recursive_type_alias_name_resolves_without_unknown_type_error() {
    let result = lower_source(
        "type Json = None | bool | int | float | str | list[Json] | dict[str, Json]\n\ndef main():\n    print(\"ok\")\n",
    );
    assert!(
        result.is_ok(),
        "recursive alias names should be predeclared before alias body resolution"
    );
}

#[test]
fn test_mutual_recursive_alias_accepts_cycle_with_container_boundary() {
    let result = lower_source(
        "type Node = Branch\ntype Branch = list[Node]\n\ndef main():\n    print(\"ok\")\n",
    );
    assert!(
        result.is_ok(),
        "recursive SCCs should be accepted when every cycle crosses a container boundary"
    );
}

#[test]
fn test_naked_recursive_alias_is_rejected() {
    let source = "type Bad = Bad\n\ndef main():\n    print(\"ok\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "ill-formed recursive type alias 'Bad': recursion must cross an indirection boundary"
            && error.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)
            && error.primary_range == Some(range_for_after(source, "= ", "Bad"))
    }));
}

#[test]
fn test_mutual_naked_recursive_alias_is_rejected() {
    let source = "type Left = Right\ntype Right = Left\n\ndef main():\n    print(\"ok\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "ill-formed recursive type alias 'Left': recursion must cross an indirection boundary"
            && error.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)
            && error.primary_range == Some(range_for_after(source, "= ", "Right"))
    }));
}

#[test]
fn test_recursive_generic_tuple_alias_is_rejected() {
    let source = "type AlsoBad[T] = tuple[AlsoBad[T], T]\n\ndef main():\n    print(\"ok\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "ill-formed recursive generic alias 'AlsoBad[T]': recursion must cross an indirection boundary"
            && error.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)
            && error.primary_range == Some(range_for_after(source, "= ", "tuple[AlsoBad[T], T]"))
    }));
}

#[test]
fn test_unresolved_type_alias_dependency_still_errors() {
    let result = lower_source("type Payload = Missing\n\ndef main():\n    print(\"ok\")\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("unknown type: 'Missing'")),
        "missing type names outside the alias predeclaration set should still error"
    );
}

#[test]
fn test_reserved_integer_width_annotations_have_int_code() {
    let source = "type Wide = int128\n\ndef take(value: uint128) -> int:\n    return 0\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();

    assert!(errors.iter().any(|error| {
        error.message == "reserved integer width name 'int128' is not supported yet"
            && error.code == Some(DiagnosticCode::INT_RESERVED_WIDTH_NAME)
            && error.primary_range == Some(range_for_after(source, "= ", "int128"))
    }));
    assert!(errors.iter().any(|error| {
        error.message == "reserved integer width name 'uint128' is not supported yet"
            && error.code == Some(DiagnosticCode::INT_RESERVED_WIDTH_NAME)
            && error.primary_range == Some(range_for_after(source, "value: ", "uint128"))
    }));
    assert!(
        errors
            .iter()
            .all(|error| error.code != Some(DiagnosticCode::NAME_UNKNOWN_TYPE)),
        "reserved width names should not fall through to generic unknown-type diagnostics"
    );
}

#[test]
fn test_nested_reserved_integer_width_annotations_have_int_code() {
    let source = "type BadMap = dict[str, uint128]\n\ndef bad() -> list[int128]:\n    return []\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();

    assert!(errors.iter().any(|error| {
        error.message == "reserved integer width name 'uint128' is not supported yet"
            && error.code == Some(DiagnosticCode::INT_RESERVED_WIDTH_NAME)
            && error.primary_range == Some(range_for_after(source, "str, ", "uint128"))
    }));
    assert!(errors.iter().any(|error| {
        error.message == "reserved integer width name 'int128' is not supported yet"
            && error.code == Some(DiagnosticCode::INT_RESERVED_WIDTH_NAME)
            && error.primary_range == Some(range_for_after(source, "list[", "int128"))
    }));
    assert!(
        errors
            .iter()
            .all(|error| error.code != Some(DiagnosticCode::NAME_UNKNOWN_TYPE)),
        "nested reserved width names should not fall through to generic unknown-type diagnostics"
    );
}

#[test]
fn test_fixed_width_integer_annotations_resolve_in_hir_signatures() {
    let result = lower_source(
        "def widths(a: int8, b: int16, c: int32, d: int64, e: uint8, f: uint16, g: uint32, h: uint64, i: isize, j: usize) -> usize:\n    return j\n",
    )
    .expect("fixed-width integer annotations should lower");

    let widths = result
        .functions
        .iter()
        .find(|function| function.name == "widths")
        .expect("widths function missing");

    let actual: Vec<Type> = widths.params.iter().map(|param| param.ty.clone()).collect();
    assert_eq!(
        actual,
        vec![
            Type::FixedInt(FixedIntType::I8),
            Type::FixedInt(FixedIntType::I16),
            Type::FixedInt(FixedIntType::I32),
            Type::FixedInt(FixedIntType::I64),
            Type::FixedInt(FixedIntType::U8),
            Type::FixedInt(FixedIntType::U16),
            Type::FixedInt(FixedIntType::U32),
            Type::FixedInt(FixedIntType::U64),
            Type::FixedInt(FixedIntType::ISize),
            Type::FixedInt(FixedIntType::USize),
        ]
    );
    assert_eq!(widths.return_type, Type::FixedInt(FixedIntType::USize));
}

#[test]
fn test_recursive_alias_annotation_preserves_symbolic_self_reference() {
    let result = lower_source(
        "type Json = None | bool | int | float | str | list[Json] | dict[str, Json]\n\ndef main():\n    value: Json = None\n",
    )
    .expect("recursive alias should lower");

    let main_fn = result
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main function missing");
    let HirStmt::Let { ty, .. } = &main_fn.body[0] else {
        panic!("expected first main statement to be a let binding");
    };

    let Type::Alias { name, body, .. } = ty else {
        panic!("expected recursive alias annotation to retain alias identity");
    };
    assert_eq!(name, "Json");
    let Type::Union(members) = body.as_ref() else {
        panic!("expected Json alias body to remain a union");
    };
    assert!(members.iter().any(|member| matches!(member, Type::None)));
    assert!(members.iter().any(|member| {
        matches!(
            member,
            Type::List(elem)
                if matches!(
                    elem.as_ref(),
                    Type::Alias {
                        name,
                        type_args,
                        body,
                    } if name == "Json" && type_args.is_empty() && matches!(body.as_ref(), Type::Unknown)
                )
        )
    }));
}

#[test]
fn test_recursive_generic_alias_preserves_specialized_type_arguments() {
    let result = lower_source(
        "type Node[T] = T | list[Node[T]]\n\ndef main():\n    leaf: Node[int] = 1\n    branch: Node[int] = [leaf]\n",
    )
    .expect("recursive generic alias should lower");

    let main_fn = result
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main function missing");
    let HirStmt::Let { ty, .. } = &main_fn.body[0] else {
        panic!("expected first main statement to be a let binding");
    };

    let Type::Alias {
        name,
        type_args,
        body,
    } = ty
    else {
        panic!("expected Node[int] annotation to retain alias identity");
    };
    assert_eq!(name, "Node");
    assert_eq!(type_args, &vec![Type::Int]);
    let Type::Union(members) = body.as_ref() else {
        panic!("expected Node[int] alias body to remain a union");
    };
    assert!(members.iter().any(|member| matches!(member, Type::Int)));
    assert!(members.iter().any(|member| {
        matches!(
            member,
            Type::List(elem)
                if matches!(
                    elem.as_ref(),
                    Type::Alias {
                        name,
                        type_args,
                        body,
                    } if name == "Node" && type_args == &vec![Type::Int] && matches!(body.as_ref(), Type::Unknown)
                )
        )
    }));
}

#[test]
fn test_generic_type_alias_wrong_arity_still_errors() {
    let result = lower_source(
        "type Pair[T] = tuple[T, T]\n\ndef main():\n    value: Pair[int, str] = (1, 2)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "generic type alias 'Pair' expects 1 type argument(s), got 2"
            && error.code == Some(DiagnosticCode::TYPE_INVALID_ANNOTATION)
    }));
}
