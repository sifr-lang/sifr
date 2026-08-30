use crate::parse_module;
use ruff_text_size::Ranged;
use sifr_python_ast::{Expr, InterpolatedStringElement, Stmt};

fn parsed_template(source: &str) -> sifr_python_ast::ExprTString {
    let parsed = parse_module(source, Some("template.sifr")).expect("template should parse");
    let Stmt::Assign(assign) = &parsed.suite()[0] else {
        panic!("expected assignment");
    };
    let Expr::TString(template) = assign.value.as_ref() else {
        panic!("expected template string AST");
    };
    template.clone()
}

#[test]
fn template_ast_retains_static_segments_holes_and_ranges() {
    let template = parsed_template("result = t\"left {first} middle {second!r:>8} right\"\n");
    let elements = template.value.elements().collect::<Vec<_>>();
    assert_eq!(elements.len(), 5);
    assert!(matches!(elements[0], InterpolatedStringElement::Literal(_)));
    assert!(matches!(
        elements[1],
        InterpolatedStringElement::Interpolation(_)
    ));
    assert!(matches!(elements[2], InterpolatedStringElement::Literal(_)));
    assert!(matches!(
        elements[3],
        InterpolatedStringElement::Interpolation(_)
    ));
    assert!(matches!(elements[4], InterpolatedStringElement::Literal(_)));
    assert!(
        elements
            .windows(2)
            .all(|pair| pair[0].range().end() <= pair[1].range().start())
    );
}

#[test]
fn multiline_raw_template_retains_decoded_and_raw_escape_meanings() {
    let ordinary = parsed_template("result = t\"\"\"line\\n{value}\nend\"\"\"\n");
    let raw = parsed_template("result = rt\"\"\"line\\n{value}\nend\"\"\"\n");
    let ordinary_text = ordinary
        .value
        .elements()
        .filter_map(|element| match element {
            InterpolatedStringElement::Literal(literal) => Some(literal.value.as_ref()),
            InterpolatedStringElement::Interpolation(_) => None,
        })
        .collect::<String>();
    let raw_text = raw
        .value
        .elements()
        .filter_map(|element| match element {
            InterpolatedStringElement::Literal(literal) => Some(literal.value.as_ref()),
            InterpolatedStringElement::Interpolation(_) => None,
        })
        .collect::<String>();
    assert!(ordinary_text.starts_with("line\n"));
    assert!(raw_text.starts_with("line\\n"));
}

#[test]
fn malformed_template_reports_parser_recovery_diagnostic() {
    let diagnostics = parse_module("result = t\"missing {value\"\n", Some("broken.sifr"))
        .expect_err("unclosed template must fail");
    assert!(!diagnostics.is_empty());
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.code.is_empty())
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| !diagnostic.spans.is_empty())
    );
}
