use super::{HirExpr, HirStmt, Type, lower_source};

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
