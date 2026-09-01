use super::{HirExpr, HirStmt, Type, lower_source};
use sifr_type_system::ReceiverConvention;

#[test]
fn record_constructor_and_field_access_lower_with_exact_types() {
    let module = lower_source(
        r#"
type User = {email: str, id: int}

def main():
    user: User = User(id=1, email="a@example.com")
    value: int = user.id
"#,
    )
    .expect("structural record source should lower");
    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main should exist");
    assert!(matches!(
        &main.body[0],
        HirStmt::Let {
            value: HirExpr::ConstructorCall { ty, .. },
            ..
        } if matches!(ty.resolve_alias(), Type::StructuralRecord(_))
    ));
    assert!(matches!(
        &main.body[1],
        HirStmt::Let {
            value: HirExpr::FieldAccess { ty: Type::Int, .. },
            ..
        }
    ));
}

#[test]
fn nullable_field_access_keeps_the_exact_optional_type() {
    let module = lower_source(
        r#"
type User = {id: int, nickname: str | None}

def nickname(user: User) -> str | None:
    return user.nickname
"#,
    )
    .expect("nullable record fields should lower");
    let Type::Union(members) = module.functions[0].return_type.resolve_alias() else {
        panic!("the nullable field must remain a union");
    };
    assert!(members.contains(&Type::None));
    assert!(members.contains(&Type::Str));
}

#[test]
fn owned_projection_consumes_the_source() {
    let errors = lower_source(
        r#"
type Wide = {id: int, email: str}
type Narrow = {id: int}

def main():
    wide: Wide = Wide(id=1, email="a@example.com")
    narrow: Narrow = wide.project[Narrow]()
    print(wide.email)
"#,
    )
    .expect_err("projection source must be unavailable after the move");
    assert!(errors.iter().any(|error| error.message.contains("moved")));
}

#[test]
fn width_subtyping_is_rejected_for_owned_parameters() {
    let errors = lower_source(
        r#"
type Wide = {id: int, email: str}
type Narrow = {id: int}

def consume(own value: Narrow):
    pass

def main():
    wide: Wide = Wide(id=1, email="a@example.com")
    consume(wide)
"#,
    )
    .expect_err("owned parameters must not apply record width subtyping");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("expected 'Narrow'"))
    );
}

#[test]
fn record_constructor_reports_missing_and_unknown_fields() {
    let missing = lower_source(
        r#"
type User = {id: int, email: str}
def main():
    user: User = User(id=1)
"#,
    )
    .expect_err("missing fields must fail");
    assert!(
        missing
            .iter()
            .any(|error| error.message.contains("missing field"))
    );

    let unknown = lower_source(
        r#"
type User = {id: int}
def main():
    user: User = User(id=1, email="no")
"#,
    )
    .expect_err("unknown fields must fail");
    assert!(
        unknown
            .iter()
            .any(|error| error.message.contains("has no field 'email'"))
    );
}

#[test]
fn mapping_pattern_is_named_record_destructuring() {
    let module = lower_source(
        r#"
type Pair = {left: int, right: int}

def total(pair: Pair) -> int:
    match pair:
        case {"right": right, "left": left}:
            return left + right
    return 0
"#,
    )
    .expect("named record pattern should lower");
    assert!(matches!(
        &module.functions[0].body[0],
        HirStmt::Match { arms, .. }
            if matches!(&arms[0].pattern, sifr_ir::HirPattern::Class { fields, .. }
                if fields.iter().map(|(name, _)| name.as_str()).eq(["right", "left"]))
    ));
}

#[test]
fn generic_records_specialize_construction_fields_and_function_calls() {
    let module = lower_source(
        r#"
type Boxed[T] = {value: T}

def unwrap[T](box: Boxed[T]) -> T:
    return box.value

def main():
    box: Boxed[int] = Boxed[int](value=9)
    value: int = unwrap(box)
"#,
    )
    .expect("generic structural records should lower");
    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main should exist");
    assert!(matches!(
        &main.body[0],
        HirStmt::Let {
            value: HirExpr::ConstructorCall { ty, .. },
            ..
        } if matches!(ty.resolve_alias(), Type::StructuralRecord(record)
            if record.field("value").is_some_and(|field| field.ty() == &Type::Int))
    ));
}

#[test]
fn width_related_record_unions_are_rejected() {
    let errors = lower_source(
        r#"
type Wide = {email: str, id: int}
type Narrow = {id: int}

def choose(value: Wide | Narrow):
    pass
"#,
    )
    .expect_err("width-related records must not form a union");
    assert!(errors.iter().any(|error| {
        error
            .message
            .contains("project to one shape or add a tag field")
    }));
}

#[test]
fn inferred_return_and_branch_unions_reject_width_related_records() {
    let return_errors = lower_source(
        r#"
type Wide = {email: str, id: int}
type Narrow = {id: int}

def choose(flag: bool):
    if flag:
        return Wide(email="dev@sifr.dev", id=1)
    return Narrow(id=1)
"#,
    )
    .expect_err("return inference must reject width-related record unions");
    assert!(return_errors.iter().any(|error| {
        error
            .message
            .contains("cannot infer a union of width-related record shapes")
    }));

    let branch_errors = lower_source(
        r#"
type Wide = {email: str, id: int}
type Narrow = {id: int}

def main():
    if True:
        value: Wide = Wide(email="dev@sifr.dev", id=1)
    else:
        value: Narrow = Narrow(id=1)
    print(value)
"#,
    )
    .expect_err("branch inference must reject width-related record unions");
    assert!(branch_errors.iter().any(|error| {
        error
            .message
            .contains("branches cannot infer 'value' as a union")
    }));
}

#[test]
fn callable_record_field_lowers_as_a_shared_receiver_call() {
    let module = lower_source(
        r#"
type Transform = {apply: Callable[[int], int]}

def apply(transform: Transform, value: int) -> int:
    return transform.apply(value)
"#,
    )
    .expect("callable structural-record fields should lower");
    let HirStmt::Return {
        value:
            Some(HirExpr::MethodCall {
                method,
                receiver_convention,
                ty,
                ..
            }),
    } = &module.functions[0].body[0]
    else {
        panic!("expected callable field method call");
    };
    assert_eq!(method, "apply");
    assert_eq!(*receiver_convention, Some(ReceiverConvention::SharedBorrow));
    assert_eq!(ty, &Type::Int);
}

#[test]
fn async_callable_record_field_lowers_to_a_coroutine() {
    let module = lower_source(
        r#"
type Runner = {apply: AsyncCallable[[str], str]}

async def apply(runner: Runner, own value: str) -> str:
    return await runner.apply(value)
"#,
    )
    .expect("async callable structural-record fields should lower");
    let HirStmt::Return {
        value: Some(HirExpr::Await { value, .. }),
    } = &module.functions[0].body[0]
    else {
        panic!("expected awaited callable field");
    };
    assert!(matches!(
        value.as_ref(),
        HirExpr::MethodCall {
            method,
            ty: Type::Coroutine(ok, _),
            ..
        } if method == "apply" && ok.as_ref() == &Type::Str
    ));
}

#[test]
fn callable_record_field_validates_arity_and_argument_types() {
    let arity_errors = lower_source(
        r#"
type Transform = {apply: Callable[[int], int]}

def apply(transform: Transform) -> int:
    return transform.apply()
"#,
    )
    .expect_err("callable record-field arity must be checked");
    assert!(arity_errors.iter().any(|error| {
        error
            .message
            .contains("(callable field) takes 1 argument(s), got 0")
    }));

    let type_errors = lower_source(
        r#"
type Transform = {apply: Callable[[int], int]}

def apply(transform: Transform) -> int:
    return transform.apply("wrong")
"#,
    )
    .expect_err("callable record-field argument types must be checked");
    assert!(type_errors.iter().any(|error| {
        error.message.contains("argument 1 of")
            && error
                .message
                .contains(".apply(): expected 'int', got 'str'")
    }));
}

#[test]
fn record_field_call_distinguishes_missing_and_non_callable_fields() {
    let missing = lower_source(
        r#"
type Transform = {apply: Callable[[int], int]}

def apply(transform: Transform) -> int:
    return transform.missing(1)
"#,
    )
    .expect_err("missing record fields must be diagnosed");
    assert!(
        missing
            .iter()
            .any(|error| error.message.contains("has no field 'missing'"))
    );

    let non_callable = lower_source(
        r#"
type Value = {number: int}

def read(value: Value) -> int:
    return value.number()
"#,
    )
    .expect_err("non-callable record fields must be diagnosed");
    assert!(non_callable.iter().any(|error| {
        error.message.contains("field 'number' of record type")
            && error.message.contains("is not callable (type: 'int')")
    }));
}

#[test]
fn async_callable_record_field_consumes_owned_arguments() {
    let errors = lower_source(
        r#"
type Runner = {apply: AsyncCallable[[str], str]}

async def apply(runner: Runner, own value: str) -> str:
    result: str = await runner.apply(value)
    print(value)
    return result
"#,
    )
    .expect_err("async callable record fields must consume owned arguments");
    assert!(errors.iter().any(|error| error.message.contains("moved")));
}
