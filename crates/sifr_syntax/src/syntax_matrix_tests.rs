use crate::{parse_module, parse_module_raw};
use serde_json::{json, Value};
use sifr_diagnostics::DiagnosticArg;
use sifr_python_ast::Stmt;
use std::collections::BTreeSet;
use std::path::PathBuf;

fn syntax_matrix() -> Value {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let matrix_path = manifest_dir
        .join("../..")
        .join("verification/areas/core_language/data/syntax_parser_lexer_matrix.json");
    let raw = std::fs::read_to_string(&matrix_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", matrix_path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("invalid {}: {err}", matrix_path.display()))
}

fn array_field<'a>(value: &'a Value, field: &str) -> &'a [Value] {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_else(|| panic!("matrix field '{field}' must be an array"))
}

fn string_field<'a>(value: &'a Value, field: &str) -> &'a str {
    value
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("matrix field '{field}' must be a string"))
}

fn optional_string_field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(Value::as_str)
}

fn string_array_field<'a>(value: &'a Value, field: &str) -> Vec<&'a str> {
    array_field(value, field)
        .iter()
        .map(|item| {
            item.as_str()
                .unwrap_or_else(|| panic!("matrix field '{field}' must contain strings"))
        })
        .collect()
}

fn token_kinds(source: &str, id: &str) -> Vec<String> {
    parse_module(source, Some(id))
        .unwrap_or_else(|diagnostics| panic!("{id} should parse: {diagnostics:#?}"))
        .tokens()
        .iter()
        .map(|token| token.kind.as_str().to_string())
        .collect()
}

fn statement_kind(stmt: &Stmt) -> &'static str {
    match stmt {
        Stmt::FunctionDef(_) => "FunctionDef",
        Stmt::ClassDef(_) => "ClassDef",
        Stmt::Return(_) => "Return",
        Stmt::Delete(_) => "Delete",
        Stmt::TypeAlias(_) => "TypeAlias",
        Stmt::Assign(_) => "Assign",
        Stmt::AugAssign(_) => "AugAssign",
        Stmt::AnnAssign(_) => "AnnAssign",
        Stmt::For(_) => "For",
        Stmt::While(_) => "While",
        Stmt::If(_) => "If",
        Stmt::With(_) => "With",
        Stmt::Match(_) => "Match",
        Stmt::Raise(_) => "Raise",
        Stmt::Try(_) => "Try",
        Stmt::Assert(_) => "Assert",
        Stmt::Import(_) => "Import",
        Stmt::ImportFrom(_) => "ImportFrom",
        Stmt::Global(_) => "Global",
        Stmt::Nonlocal(_) => "Nonlocal",
        Stmt::Expr(_) => "Expr",
        Stmt::Pass(_) => "Pass",
        Stmt::Break(_) => "Break",
        Stmt::Continue(_) => "Continue",
        Stmt::IpyEscapeCommand(_) => "IpyEscapeCommand",
    }
}

fn statement_shape(stmt: &Stmt) -> Value {
    let nested = match stmt {
        Stmt::FunctionDef(function_def) => Some(statement_tree(&function_def.body)),
        Stmt::ClassDef(class_def) => Some(statement_tree(&class_def.body)),
        Stmt::For(for_stmt) => Some(statement_tree(&for_stmt.body)),
        Stmt::While(while_stmt) => Some(statement_tree(&while_stmt.body)),
        Stmt::If(if_stmt) => Some(statement_tree(&if_stmt.body)),
        Stmt::With(with_stmt) => Some(statement_tree(&with_stmt.body)),
        Stmt::Match(match_stmt) => Some(
            match_stmt
                .cases
                .iter()
                .flat_map(|case| statement_tree(&case.body))
                .collect::<Vec<_>>(),
        ),
        Stmt::Try(try_stmt) => Some(statement_tree(&try_stmt.body)),
        _ => None,
    };
    if let Some(body) = nested.filter(|body| !body.is_empty()) {
        json!({
            "kind": statement_kind(stmt),
            "body": body,
        })
    } else {
        json!({
            "kind": statement_kind(stmt),
        })
    }
}

fn statement_tree(stmts: &[Stmt]) -> Vec<Value> {
    stmts.iter().map(statement_shape).collect()
}

#[test]
fn positive_parser_matrix_cases_parse_and_expose_required_tokens() {
    let matrix = syntax_matrix();
    for case in array_field(&matrix, "positive_parse_cases") {
        let id = string_field(case, "id");
        let source = string_field(case, "source");
        let kinds = token_kinds(source, id);
        for required in string_array_field(case, "required_token_kinds") {
            assert!(
                kinds.iter().any(|kind| kind == required),
                "{id} missing required token kind {required}; got {kinds:?}"
            );
        }
    }
}

#[test]
fn negative_parser_matrix_cases_emit_stable_diagnostics() {
    let matrix = syntax_matrix();
    for case in array_field(&matrix, "negative_parse_cases") {
        let id = string_field(case, "id");
        let source = string_field(case, "source");
        let diagnostics = match parse_module_raw(source, Some(id)) {
            Ok(_) => panic!("{id} should fail to parse"),
            Err(diagnostics) => diagnostics,
        };
        let expected_code = string_field(case, "expected_code");
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == expected_code)
            .unwrap_or_else(|| {
                panic!("{id} missing expected code {expected_code}: {diagnostics:#?}")
            });
        if let Some(expected_category) = optional_string_field(case, "expected_parser_category") {
            let category = diagnostic.args.get("parser_category");
            assert!(
                matches!(category, Some(DiagnosticArg::String(actual)) if actual == expected_category),
                "{id} expected parser_category={expected_category:?}, got {category:?}"
            );
        }
        if let Some(expected_message) = optional_string_field(case, "expected_message_contains") {
            assert!(
                diagnostic.message.contains(expected_message),
                "{id} expected message containing {expected_message:?}, got {:?}",
                diagnostic.message
            );
        }
    }
}

#[test]
fn lexer_token_matrix_preserves_kinds_and_byte_spans() {
    let matrix = syntax_matrix();
    for case in array_field(&matrix, "token_stream_cases") {
        let id = string_field(case, "id");
        let source = string_field(case, "source");
        let parsed = parse_module(source, Some(id))
            .unwrap_or_else(|diagnostics| panic!("{id} should parse: {diagnostics:#?}"));
        let kinds = parsed
            .tokens()
            .iter()
            .map(|token| token.kind.as_str().to_string())
            .collect::<Vec<_>>();
        for required in string_array_field(case, "required_token_kinds") {
            assert!(
                kinds.iter().any(|kind| kind == required),
                "{id} missing required token kind {required}; got {kinds:?}"
            );
        }
        for assertion in array_field(case, "span_assertions") {
            let kind = string_field(assertion, "kind");
            let occurrence = assertion
                .get("occurrence")
                .and_then(Value::as_u64)
                .and_then(|value| usize::try_from(value).ok())
                .filter(|value| *value > 0)
                .unwrap_or_else(|| panic!("{id}/{kind} occurrence must be positive"));
            let token = parsed
                .tokens()
                .iter()
                .filter(|token| token.kind.as_str() == kind)
                .nth(occurrence - 1)
                .unwrap_or_else(|| {
                    panic!("{id} missing occurrence {occurrence} for token kind {kind}")
                });
            let expected_start = assertion
                .get("byte_start")
                .and_then(Value::as_u64)
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or_else(|| panic!("{id}/{kind} byte_start must be a u32"));
            let expected_end = assertion
                .get("byte_end")
                .and_then(Value::as_u64)
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or_else(|| panic!("{id}/{kind} byte_end must be a u32"));
            assert_eq!(
                token.range.start().to_u32(),
                expected_start,
                "{id}/{kind} byte_start changed"
            );
            assert_eq!(
                token.range.end().to_u32(),
                expected_end,
                "{id}/{kind} byte_end changed"
            );
        }
    }
}

#[test]
fn parsed_source_shape_snapshots_match_sifr_owned_boundary() {
    let matrix = syntax_matrix();
    for case in array_field(&matrix, "shape_snapshots") {
        let id = string_field(case, "id");
        let source = string_field(case, "source");
        let expected_tree = case
            .get("expected_statement_tree")
            .unwrap_or_else(|| panic!("{id} must declare expected_statement_tree"));
        let parsed = parse_module(source, Some(id))
            .unwrap_or_else(|diagnostics| panic!("{id} should parse: {diagnostics:#?}"));
        let actual_tree = Value::Array(statement_tree(parsed.suite()));
        assert_eq!(
            &actual_tree, expected_tree,
            "{id} parsed-source statement shape snapshot changed"
        );
    }
}

#[test]
fn syntax_matrix_has_no_positive_negative_source_contradictions() {
    let matrix = syntax_matrix();
    assert!(
        !array_field(&matrix, "contradiction_checks").is_empty(),
        "syntax matrix must declare contradiction checks"
    );
    let positive_sources = array_field(&matrix, "positive_parse_cases")
        .iter()
        .map(|case| string_field(case, "source"))
        .collect::<BTreeSet<_>>();
    let negative_cases = array_field(&matrix, "negative_parse_cases");
    let negative_sources = negative_cases
        .iter()
        .map(|case| string_field(case, "source"))
        .collect::<BTreeSet<_>>();
    for case in negative_cases {
        let id = string_field(case, "id");
        let source = string_field(case, "source");
        assert!(
            !positive_sources.contains(source),
            "{id} appears in both positive and negative parser matrices"
        );
    }
    for case in array_field(&matrix, "positive_parse_cases") {
        let id = string_field(case, "id");
        let source = string_field(case, "source");
        assert!(
            !negative_sources.contains(source),
            "{id} appears in both positive and negative parser matrices"
        );
    }
}
