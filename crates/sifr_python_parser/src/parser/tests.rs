use crate::{parse, parse_expression, parse_module, Mode, ParseErrorType};
use sifr_python_ast::{AstParamConvention, Stmt};

#[test]
fn test_modes() {
    let source = "a[0][1][2][3][4]";

    assert!(parse(source, Mode::Expression).is_ok());
    assert!(parse(source, Mode::Module).is_ok());
}

#[test]
fn test_expr_mode_invalid_syntax1() {
    let source = "first second";
    let error = parse_expression(source).unwrap_err();

    insta::assert_debug_snapshot!(error);
}

#[test]
fn test_expr_mode_invalid_syntax2() {
    let source = r"first

second
";
    let error = parse_expression(source).unwrap_err();

    insta::assert_debug_snapshot!(error);
}

#[test]
fn test_expr_mode_invalid_syntax3() {
    let source = r"first

second

third
";
    let error = parse_expression(source).unwrap_err();

    insta::assert_debug_snapshot!(error);
}

#[test]
fn test_expr_mode_valid_syntax() {
    let source = "first

";
    let parsed = parse_expression(source).unwrap();

    insta::assert_debug_snapshot!(parsed.expr());
}

#[test]
fn test_unicode_aliases() {
    // https://github.com/RustPython/RustPython/issues/4566
    let source = r#"x = "\N{BACKSPACE}another cool trick""#;
    let suite = parse_module(source).unwrap().into_suite();

    insta::assert_debug_snapshot!(suite);
}

#[test]
fn test_ipython_escape_commands() {
    let parsed = parse(
        r"
# Normal Python code
(
    a
    %
    b
)

# Dynamic object info
??a.foo
?a.foo
?a.foo?
??a.foo()??

# Line magic
%timeit a = b
%timeit foo(b) % 3
%alias showPath pwd && ls -a
%timeit a =\
  foo(b); b = 2
%matplotlib --inline
%matplotlib \
    --inline

# System shell access
!pwd && ls -a | sed 's/^/\    /'
!pwd \
  && ls -a | sed 's/^/\\    /'
!!cd /Users/foo/Library/Application\ Support/

# Let's add some Python code to make sure that earlier escapes were handled
# correctly and that we didn't consume any of the following code as a result
# of the escapes.
def foo():
    return (
        a
        !=
        b
    )

# Transforms into `foo(..)`
/foo 1 2
;foo 1 2
,foo 1 2

# Indented escape commands
for a in range(5):
    !ls

p1 = !pwd
p2: str = !pwd
foo = %foo \
    bar

% foo
foo = %foo  # comment

# Help end line magics
foo?
foo.bar??
foo.bar.baz?
foo[0]??
foo[0][1]?
foo.bar[0].baz[1]??
foo.bar[0].baz[2].egg??
"
        .trim(),
        Mode::Ipython,
    )
    .unwrap();
    insta::assert_debug_snapshot!(parsed.syntax());
}

fn parse_function(source: &str) -> sifr_python_ast::StmtFunctionDef {
    let suite = parse_module(source).expect("parse failed").into_suite();
    let stmt = suite
        .into_iter()
        .next()
        .expect("expected function definition");
    match stmt {
        Stmt::FunctionDef(function) => function,
        other => panic!("expected function definition, got {other:?}"),
    }
}

#[test]
fn test_parameter_modifiers_normalize_both_source_orders() {
    let own_mut = parse_function("def f(own mut items: list[int]):\n    return items\n");
    let mut_own = parse_function("def f(mut own items: list[int]):\n    return items\n");

    let own_mut_param = &own_mut.parameters.args[0].parameter;
    let mut_own_param = &mut_own.parameters.args[0].parameter;

    assert_eq!(own_mut_param.convention, AstParamConvention::own_mut());
    assert_eq!(mut_own_param.convention, AstParamConvention::own_mut());
}

#[test]
fn test_duplicate_mut_parameter_modifier_is_rejected() {
    let error = parse_module("def f(mut mut items: list[int]):\n    return items\n").unwrap_err();

    assert!(matches!(
        error.error,
        ParseErrorType::OtherError(message) if message == "duplicate `mut` parameter modifier"
    ));
}

#[test]
fn test_duplicate_own_parameter_modifier_is_rejected() {
    let error = parse_module("def f(own own items: list[int]):\n    return items\n").unwrap_err();

    assert!(matches!(
        error.error,
        ParseErrorType::OtherError(message) if message == "duplicate `own` parameter modifier"
    ));
}

#[test]
fn test_soft_keyword_parameter_names_still_parse_without_modifier_context() {
    let function = parse_function("def f(mut: int, own: int) -> int:\n    return mut + own\n");

    assert_eq!(
        function.parameters.args[0].parameter.convention,
        AstParamConvention::borrow()
    );
    assert_eq!(
        function.parameters.args[1].parameter.convention,
        AstParamConvention::borrow()
    );
}
