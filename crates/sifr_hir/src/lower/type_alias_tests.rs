use crate::{lower_module, HirModule, HirStmt, LoweringError};
use sifr_python_parser::parse_module;
use sifr_type_system::Type;

fn lower_source(source: &str) -> Result<HirModule, Vec<LoweringError>> {
    let parsed = parse_module(source).expect("parse failed");
    lower_module(parsed.suite()).map(|result| result.module)
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
    let result = lower_source("type Bad = Bad\n\ndef main():\n    print(\"ok\")\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "ill-formed recursive type alias 'Bad': recursion must cross an indirection boundary"
    }));
}

#[test]
fn test_recursive_generic_tuple_alias_is_rejected() {
    let result =
        lower_source("type AlsoBad[T] = tuple[AlsoBad[T], T]\n\ndef main():\n    print(\"ok\")\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "ill-formed recursive generic alias 'AlsoBad[T]': recursion must cross an indirection boundary"
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
    }));
}
