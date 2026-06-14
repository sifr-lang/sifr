//! Stable Sifr-facing syntax wrapper over the Sifr Ruff fork.
//!
//! This crate owns parser, token, and source-position entrypoints that other
//! Sifr crates can depend on without taking a direct dependency on raw Ruff
//! parser APIs.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

use ruff_text_size::{Ranged as _, TextRange};
use sifr_diagnostics::{
    ChildSeverity, DiagnosticArg, DiagnosticBuilder, DiagnosticCode, DiagnosticSink,
    RenderedDiagnostic, Severity, SourceMap, SourceSpan,
};
use sifr_python_ast::token::TokenKind;
use sifr_python_ast::{ModModule, PythonVersion, Stmt};
use sifr_python_parser::{
    parse_unchecked, InterpolatedStringErrorType, LexicalErrorType, Mode, ParseError,
    ParseErrorType, ParseOptions, Parsed, UnsupportedSyntaxError,
};
pub use sifr_source::{SourceText, TextPosition, TextRangeUtf};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct ParsedModule {
    suite: Vec<Stmt>,
    tokens: Vec<SyntaxToken>,
}

impl ParsedModule {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            suite: Vec::new(),
            tokens: Vec::new(),
        }
    }

    #[must_use]
    pub fn suite(&self) -> &[Stmt] {
        &self.suite
    }

    #[must_use]
    pub fn into_suite(self) -> Vec<Stmt> {
        self.suite
    }

    #[must_use]
    pub fn tokens(&self) -> &[SyntaxToken] {
        &self.tokens
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxToken {
    pub kind: SyntaxTokenKind,
    pub range: TextRange,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxTokenKind(String);

impl SyntaxTokenKind {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<TokenKind> for SyntaxTokenKind {
    fn from(value: TokenKind) -> Self {
        Self(format!("{value:?}"))
    }
}

pub fn parse_module(
    source: &str,
    context: Option<&str>,
) -> Result<ParsedModule, Vec<RenderedDiagnostic>> {
    let parsed = parse_module_raw(source, context)?;
    let tokens = parsed
        .tokens()
        .iter()
        .map(|token| SyntaxToken {
            kind: token.kind().into(),
            range: token.range(),
        })
        .collect();
    Ok(ParsedModule {
        suite: parsed.into_suite(),
        tokens,
    })
}

pub fn parse_module_suite(
    source: &str,
    context: Option<&str>,
) -> Result<Vec<Stmt>, Vec<RenderedDiagnostic>> {
    parse_module_raw(source, context).map(Parsed::into_suite)
}

pub fn parse_module_raw(
    source: &str,
    context: Option<&str>,
) -> Result<Parsed<ModModule>, Vec<RenderedDiagnostic>> {
    let parsed = parse_unchecked(
        source,
        ParseOptions::from(Mode::Module).with_target_version(PythonVersion::latest_ty()),
    );
    let Some(parsed) = parsed.try_into_module() else {
        return Err(vec![diagnostic_with_code(
            "internal compiler error: module parse mode did not produce a module syntax tree",
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        )]);
    };
    if parsed.has_invalid_syntax() {
        return Err(parsed
            .errors()
            .iter()
            .map(|error| parse_error_diagnostic(source, error, context))
            .collect());
    }
    if !parsed.unsupported_syntax_errors().is_empty() {
        return Err(parsed
            .unsupported_syntax_errors()
            .iter()
            .map(|error| unsupported_syntax_diagnostic(source, error, context))
            .collect());
    }
    Ok(parsed)
}

fn diagnostic_with_code(message: impl Into<String>, code: DiagnosticCode) -> RenderedDiagnostic {
    RenderedDiagnostic {
        code: code.code().to_string(),
        severity: code.declared_severity(),
        message: message.into(),
        message_template: "{message}".to_string(),
        args: BTreeMap::from([("message".to_string(), DiagnosticArg::String(String::new()))]),
        url: code.docs_url(),
        spans: Vec::new(),
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    }
}

fn parse_error_diagnostic(
    source: &str,
    error: &ParseError,
    context: Option<&str>,
) -> RenderedDiagnostic {
    parse_diagnostic(
        parse_error_details(&error.error),
        source,
        error.location,
        context,
    )
}

fn unsupported_syntax_diagnostic(
    source: &str,
    error: &UnsupportedSyntaxError,
    context: Option<&str>,
) -> RenderedDiagnostic {
    parse_diagnostic(
        ParseDiagnosticDetails {
            code: DiagnosticCode::PARSE_UNSUPPORTED_SYNTAX,
            template: "unsupported syntax: {syntax_kind}",
            arg_name: "syntax_kind",
            arg_value: error.to_string(),
            parser_category: "unsupported_syntax",
        },
        source,
        error.range(),
        context,
    )
}

struct ParseDiagnosticDetails {
    code: DiagnosticCode,
    template: &'static str,
    arg_name: &'static str,
    arg_value: String,
    parser_category: &'static str,
}

fn parse_diagnostic(
    details: ParseDiagnosticDetails,
    source: &str,
    range: TextRange,
    context: Option<&str>,
) -> RenderedDiagnostic {
    let display_path = context.unwrap_or("main");
    let mut source_map = SourceMap::new();
    let source_id = source_map.register_source(display_path, source);
    let span = match SourceSpan::new_validated(&source_map, source_id, range) {
        Ok(span) => span,
        Err(error) => {
            return diagnostic_with_code(
                format!("internal compiler error: invalid parser diagnostic span: {error:?}"),
                DiagnosticCode::INTERNAL_COMPILER_PANIC,
            );
        }
    };
    let mut builder = DiagnosticBuilder::source(details.code, Severity::Error, span)
        .message_template(details.template)
        .arg(details.arg_name, details.arg_value)
        .arg("parser_category", details.parser_category);
    if let Some(label) = context {
        builder = builder.child(ChildSeverity::Note, format!("while parsing {label}"));
    }
    let diagnostic = builder.build();
    let mut sink = DiagnosticSink::new();
    let _ = sink.emit_error(diagnostic);
    match sifr_diagnostics::render::render_sink(&sink, &source_map) {
        Ok(mut envelope) if envelope.diagnostics.len() == 1 => envelope.diagnostics.remove(0),
        Ok(_) => diagnostic_with_code(
            "internal compiler error: parser diagnostic renderer emitted an unexpected diagnostic count",
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ),
        Err(error) => diagnostic_with_code(
            format!("internal compiler error: invalid parser diagnostic span: {error:?}"),
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ),
    }
}

fn parse_error_details(error: &ParseErrorType) -> ParseDiagnosticDetails {
    match error {
        ParseErrorType::Lexical(reason) => lexical_or_string_details(reason),
        ParseErrorType::FStringError(reason) => interpolated_string_details("f_string", reason),
        ParseErrorType::TStringError(reason) => interpolated_string_details("t_string", reason),
        ParseErrorType::UnexpectedIndentation => layout_details("unexpected indentation"),
        ParseErrorType::SimpleStatementsOnSameLine => {
            layout_details("simple statements must be separated")
        }
        ParseErrorType::SimpleAndCompoundStatementOnSameLine => {
            layout_details("compound statement shares a line with a simple statement")
        }
        ParseErrorType::InvalidAssignmentTarget => {
            invalid_target_details("assignment target", "invalid_assignment_target")
        }
        ParseErrorType::InvalidNamedAssignmentTarget => {
            invalid_target_details("named assignment target", "invalid_named_assignment_target")
        }
        ParseErrorType::InvalidAnnotatedAssignmentTarget => invalid_target_details(
            "annotated assignment target",
            "invalid_annotated_assignment_target",
        ),
        ParseErrorType::InvalidAugmentedAssignmentTarget => invalid_target_details(
            "augmented assignment target",
            "invalid_augmented_assignment_target",
        ),
        ParseErrorType::InvalidDeleteTarget => {
            invalid_target_details("delete target", "invalid_delete_target")
        }
        ParseErrorType::EmptyDeleteTargets => declaration_list_details("delete statement"),
        ParseErrorType::InvalidStarredExpressionUsage => {
            invalid_target_details("starred expression", "invalid_starred_expression")
        }
        ParseErrorType::DuplicateKeywordArgumentError(name) => invalid_call_details(
            format!("duplicate keyword argument {name:?}"),
            "duplicate_keyword_argument",
        ),
        ParseErrorType::PositionalAfterKeywordArgument => invalid_call_details(
            "positional argument after keyword argument",
            "positional_after_keyword_argument",
        ),
        ParseErrorType::PositionalAfterKeywordUnpacking => invalid_call_details(
            "positional argument after keyword unpacking",
            "positional_after_keyword_unpacking",
        ),
        ParseErrorType::InvalidArgumentUnpackingOrder => invalid_call_details(
            "iterable unpacking after keyword unpacking",
            "invalid_argument_unpacking_order",
        ),
        ParseErrorType::IterableUnpackingInComprehension => invalid_call_details(
            "iterable unpacking in comprehension",
            "iterable_unpacking_in_comprehension",
        ),
        ParseErrorType::EmptyGlobalNames => declaration_list_details("global statement"),
        ParseErrorType::EmptyNonlocalNames => declaration_list_details("nonlocal statement"),
        ParseErrorType::EmptyImportNames => declaration_list_details("import statement"),
        ParseErrorType::EmptyTypeParams => declaration_list_details("type parameter list"),
        ParseErrorType::ParamAfterVarKeywordParam => {
            declaration_list_details("parameter after var-keyword parameter")
        }
        ParseErrorType::NonDefaultParamAfterDefaultParam => {
            declaration_list_details("non-default parameter after default parameter")
        }
        ParseErrorType::VarParameterWithDefault => {
            declaration_list_details("var parameter default")
        }
        ParseErrorType::ExpectedKeywordParam => {
            declaration_list_details("keyword-only parameter list")
        }
        ParseErrorType::InvalidStarPatternUsage => {
            invalid_pattern_details("star pattern outside a sequence pattern")
        }
        ParseErrorType::ExpectedRealNumber => {
            invalid_pattern_details("expected real number in complex literal pattern")
        }
        ParseErrorType::ExpectedImaginaryNumber => {
            invalid_pattern_details("expected imaginary number in complex literal pattern")
        }
        ParseErrorType::UnexpectedTokenAfterAsync(token) => unsupported_details(
            format!("async statement cannot be followed by {token}"),
            "unexpected_token_after_async",
        ),
        ParseErrorType::UnexpectedIpythonEscapeCommand => unsupported_details(
            "IPython escape command in module mode",
            "unexpected_ipython_escape_command",
        ),
        ParseErrorType::ExpectedToken { expected, found } => expected_details(
            format!("{expected}; found {found}"),
            "expected_token_or_recovery",
        ),
        ParseErrorType::ExpectedExpression => expected_details("expression", "expected_expression"),
        ParseErrorType::UnexpectedExpressionToken => {
            expected_details("expression terminator", "unexpected_expression_token")
        }
        ParseErrorType::EmptySlice => expected_details("index or slice expression", "empty_slice"),
        ParseErrorType::UnparenthesizedNamedExpression => expected_details(
            "parenthesized named expression",
            "unparenthesized_named_expression",
        ),
        ParseErrorType::UnparenthesizedTupleExpression => expected_details(
            "parenthesized tuple expression",
            "unparenthesized_tuple_expression",
        ),
        ParseErrorType::UnparenthesizedGeneratorExpression => expected_details(
            "parenthesized generator expression",
            "unparenthesized_generator_expression",
        ),
        ParseErrorType::InvalidLambdaExpressionUsage => expected_details(
            "valid lambda expression context",
            "invalid_lambda_expression_usage",
        ),
        ParseErrorType::InvalidYieldExpressionUsage => expected_details(
            "valid yield expression context",
            "invalid_yield_expression_usage",
        ),
        ParseErrorType::OtherError(message) => {
            expected_details(recovery_expected(message), "parser_recovery")
        }
    }
}

fn recovery_expected(message: &str) -> String {
    if let Some(stripped) = message
        .strip_prefix("Expected ")
        .or_else(|| message.strip_prefix("expected "))
    {
        return stripped.to_string();
    }
    format!("recovery: {message}")
}

fn expected_details(
    expected: impl Into<String>,
    parser_category: &'static str,
) -> ParseDiagnosticDetails {
    ParseDiagnosticDetails {
        code: DiagnosticCode::PARSE_EXPECTED_TOKEN_OR_RECOVERY,
        template: "syntax error: expected {expected}",
        arg_name: "expected",
        arg_value: expected.into(),
        parser_category,
    }
}

fn lexical_or_string_details(reason: &LexicalErrorType) -> ParseDiagnosticDetails {
    ParseDiagnosticDetails {
        code: DiagnosticCode::PARSE_LEXICAL_OR_STRING,
        template: "lexical error: {reason}",
        arg_name: "reason",
        arg_value: reason.to_string(),
        parser_category: match reason {
            LexicalErrorType::StringError
            | LexicalErrorType::UnclosedStringError
            | LexicalErrorType::UnicodeError
            | LexicalErrorType::MissingUnicodeLbrace
            | LexicalErrorType::MissingUnicodeRbrace
            | LexicalErrorType::FStringError(_)
            | LexicalErrorType::TStringError(_)
            | LexicalErrorType::InvalidByteLiteral => "lexical_string",
            LexicalErrorType::IndentationError => "lexical_indentation",
            LexicalErrorType::UnrecognizedToken { .. } => "lexical_unrecognized_token",
            LexicalErrorType::LineContinuationError => "lexical_line_continuation",
            LexicalErrorType::Eof => "lexical_eof",
            LexicalErrorType::OtherError(_) => "lexical_other",
        },
    }
}

fn interpolated_string_details(
    string_kind: &'static str,
    reason: &InterpolatedStringErrorType,
) -> ParseDiagnosticDetails {
    ParseDiagnosticDetails {
        code: DiagnosticCode::PARSE_LEXICAL_OR_STRING,
        template: "lexical error: {reason}",
        arg_name: "reason",
        arg_value: format!("{string_kind}: {reason}"),
        parser_category: "lexical_string",
    }
}

fn layout_details(reason: impl Into<String>) -> ParseDiagnosticDetails {
    ParseDiagnosticDetails {
        code: DiagnosticCode::PARSE_LAYOUT,
        template: "invalid layout: {reason}",
        arg_name: "reason",
        arg_value: reason.into(),
        parser_category: "layout",
    }
}

fn invalid_target_details(
    target_kind: impl Into<String>,
    parser_category: &'static str,
) -> ParseDiagnosticDetails {
    ParseDiagnosticDetails {
        code: DiagnosticCode::PARSE_INVALID_TARGET,
        template: "invalid target: {target_kind}",
        arg_name: "target_kind",
        arg_value: target_kind.into(),
        parser_category,
    }
}

fn invalid_call_details(
    reason: impl Into<String>,
    parser_category: &'static str,
) -> ParseDiagnosticDetails {
    ParseDiagnosticDetails {
        code: DiagnosticCode::PARSE_INVALID_CALL_ARGUMENTS,
        template: "invalid call arguments: {reason}",
        arg_name: "reason",
        arg_value: reason.into(),
        parser_category,
    }
}

fn declaration_list_details(declaration_kind: impl Into<String>) -> ParseDiagnosticDetails {
    ParseDiagnosticDetails {
        code: DiagnosticCode::PARSE_MALFORMED_DECLARATION_LIST,
        template: "malformed declaration list: {declaration_kind}",
        arg_name: "declaration_kind",
        arg_value: declaration_kind.into(),
        parser_category: "declaration_list",
    }
}

fn invalid_pattern_details(reason: impl Into<String>) -> ParseDiagnosticDetails {
    ParseDiagnosticDetails {
        code: DiagnosticCode::PARSE_INVALID_PATTERN,
        template: "invalid match pattern: {reason}",
        arg_name: "reason",
        arg_value: reason.into(),
        parser_category: "match_pattern",
    }
}

fn unsupported_details(
    syntax_kind: impl Into<String>,
    parser_category: &'static str,
) -> ParseDiagnosticDetails {
    ParseDiagnosticDetails {
        code: DiagnosticCode::PARSE_UNSUPPORTED_SYNTAX,
        template: "unsupported syntax: {syntax_kind}",
        arg_name: "syntax_kind",
        arg_value: syntax_kind.into(),
        parser_category,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        expected_details, parse_diagnostic, parse_module, parse_module_raw, DiagnosticArg,
        SourceText, TextPosition,
    };
    use ruff_text_size::{TextRange, TextSize};
    use serde_json::Value;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    #[test]
    fn parse_module_exposes_suite_and_tokens() {
        let parsed = parse_module("def main():\n    return 1\n", Some("main"))
            .expect("valid module should parse");

        assert_eq!(parsed.suite().len(), 1);
        assert!(parsed
            .tokens()
            .iter()
            .any(|token| token.kind.as_str() == "Def"));
    }

    #[test]
    fn source_text_converts_utf8_positions() {
        let source = SourceText::new("a\nbc\n");

        assert_eq!(
            source.byte_offset(&TextPosition {
                line: 1,
                character: 1
            }),
            Some(ruff_text_size::TextSize::new(3))
        );
        assert_eq!(
            source.text_position(ruff_text_size::TextSize::new(3)),
            Some(TextPosition {
                line: 1,
                character: 1
            })
        );
    }

    #[test]
    fn parser_diagnostic_uses_parse_error_location_span() {
        let source = "def main():\nprint('bad indent')\n";
        let mut diagnostics =
            parse_module_raw(source, Some("bad_indent.sifr")).expect_err("source must fail");
        let diagnostic = diagnostics.remove(0);

        assert_eq!(diagnostic.code, "SIFR-PARSE-0002");
        let span = diagnostic.spans.first().expect("primary span");
        assert_eq!(span.file.as_deref(), Some("bad_indent.sifr"));
        assert_eq!(span.byte_start, 12);
        assert_eq!(span.line, Some(2));
        assert_eq!(span.column, Some(1));
        assert_eq!(span.lines[0].text, "print('bad indent')");
        assert_eq!(span.lines[0].highlight_start, 1);
    }

    #[test]
    fn parser_diagnostic_columns_are_utf8_character_based() {
        let source = "name = '🦀'\ndef main():\nprint('bad indent')\n";
        let mut diagnostics =
            parse_module_raw(source, Some("utf8.sifr")).expect_err("source must fail");
        let diagnostic = diagnostics.remove(0);
        let span = diagnostic.spans.first().expect("primary span");

        assert_eq!(
            span.byte_start,
            u32::try_from(source.find("print").expect("print offset")).unwrap()
        );
        assert_eq!(span.line, Some(3));
        assert_eq!(span.column, Some(1));
        assert_eq!(span.lines[0].text, "print('bad indent')");
    }

    #[test]
    fn parser_diagnostic_preserves_crlf_source_text_in_json_span() {
        let source = "def main():\r\nprint('bad indent')\r\n";
        let mut diagnostics =
            parse_module_raw(source, Some("crlf.sifr")).expect_err("source must fail");
        let diagnostic = diagnostics.remove(0);
        let span = diagnostic.spans.first().expect("primary span");

        assert_eq!(span.line, Some(2));
        assert_eq!(span.column, Some(1));
        assert_eq!(span.lines[0].text, "print('bad indent')\r");
    }

    #[test]
    fn parser_diagnostic_renders_zero_length_eof_span() {
        let diagnostic = parse_diagnostic(
            expected_details("end of file", "parser_recovery"),
            "",
            TextRange::new(TextSize::new(0), TextSize::new(0)),
            Some("empty.sifr"),
        );
        let span = diagnostic.spans.first().expect("primary span");

        assert_eq!(diagnostic.code, "SIFR-PARSE-0002");
        assert_eq!(span.byte_start, 0);
        assert_eq!(span.byte_end, 0);
        assert_eq!(span.line, Some(1));
        assert_eq!(span.column, Some(1));
        assert_eq!(span.lines[0].text, "");
        assert_eq!(span.lines[0].highlight_start, 1);
        assert_eq!(span.lines[0].highlight_end, 1);
    }

    #[test]
    fn parser_diagnostic_invalid_span_becomes_internal_error() {
        let diagnostic = parse_diagnostic(
            expected_details("identifier", "parser_recovery"),
            "x\n",
            TextRange::new(TextSize::new(9), TextSize::new(9)),
            Some("invalid.sifr"),
        );

        assert_eq!(diagnostic.code, "SIFR-INTERNAL-0001");
        assert!(diagnostic
            .message
            .contains("invalid parser diagnostic span"));
        assert!(diagnostic.spans.is_empty());
    }

    #[test]
    fn unsupported_syntax_diagnostic_uses_ruff_range() {
        let mut diagnostics = parse_module_raw("lazy import value\n", Some("lazy.sifr"))
            .expect_err("source must fail");
        let diagnostic = diagnostics.remove(0);
        let span = diagnostic.spans.first().expect("primary span");

        assert_eq!(diagnostic.code, "SIFR-PARSE-0009");
        assert_eq!(span.byte_start, 0);
        assert_eq!(span.byte_end, 4);
        assert_eq!(span.line, Some(1));
        assert_eq!(span.column, Some(1));
        assert_eq!(span.lines[0].text, "lazy import value");
    }

    mod syntax_matrix_tests {
        use super::*;

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
                if let Some(expected_category) =
                    optional_string_field(case, "expected_parser_category")
                {
                    let category = diagnostic.args.get("parser_category");
                    assert!(
                        matches!(category, Some(DiagnosticArg::String(actual)) if actual == expected_category),
                        "{id} expected parser_category={expected_category:?}, got {category:?}"
                    );
                }
                if let Some(expected_message) =
                    optional_string_field(case, "expected_message_contains")
                {
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
    }
}
