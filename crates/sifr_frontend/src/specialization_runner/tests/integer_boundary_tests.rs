use super::*;

#[test]
fn non_package_integer_boundary_fixture_fails_closed_for_missing_or_unsafe_policy() {
    let missing = errors(compile(
        "fixture.boundaries",
        r#"
@json_integer_boundary("count", None, "default", None, None)
class Counter:
    count: int
"#,
        &ExternalDefs::default(),
    ));
    assert_eq!(missing[0].code, "SIFR-INT-0009");
    assert_eq!(
        missing[0].args["path"],
        DiagnosticArg::String("fixture.boundaries.Counter.count".to_string())
    );

    let unsafe_web = errors(compile(
        "fixture.boundaries",
        r#"
@json_integer_boundary("count", "web", "number", None, None)
class Counter:
    count: int
"#,
        &ExternalDefs::default(),
    ));
    assert_eq!(unsafe_web[0].code, "SIFR-INT-0009");

    let safe = compile(
        "fixture.boundaries",
        r#"
@json_integer_boundary("count", "web", "number", -2147483648, 2147483647)
class Counter:
    count: int32
"#,
        &ExternalDefs::default(),
    );
    assert!(safe.is_ok());
}
