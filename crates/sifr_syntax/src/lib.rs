//! Stable Sifr-facing syntax wrapper over the Sifr Ruff fork.
//!
//! This crate owns parser, token, and source-position entrypoints that other
//! Sifr crates can depend on without taking a direct dependency on raw Ruff
//! parser APIs.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

use ruff_text_size::{Ranged as _, TextRange, TextSize};
use sifr_diagnostics::render::RenderedDiagnosticChild;
use sifr_diagnostics::{
    ChildSeverity, DiagnosticArg, DiagnosticCode, RenderedDiagnostic, Severity,
};
use sifr_python_ast::token::TokenKind;
use sifr_python_ast::{ModModule, PythonVersion, Stmt};
use sifr_python_parser::{
    parse_unchecked, InterpolatedStringErrorType, LexicalErrorType, Mode, ParseError,
    ParseErrorType, ParseOptions, Parsed, UnsupportedSyntaxError,
};
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextPosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextRangeUtf {
    pub start: TextPosition,
    pub end: TextPosition,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceText {
    text: String,
    line_starts: Vec<usize>,
}

impl SourceText {
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let mut line_starts = vec![0];
        for (idx, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(idx + 1);
            }
        }
        Self { text, line_starts }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn byte_offset(&self, position: &TextPosition) -> Option<TextSize> {
        let line = usize::try_from(position.line).ok()?;
        let character = usize::try_from(position.character).ok()?;
        let line_start = *self.line_starts.get(line)?;
        let line_end = self
            .line_starts
            .get(line + 1)
            .copied()
            .unwrap_or(self.text.len());
        let offset = line_start.checked_add(character)?;
        (offset <= line_end && self.text.is_char_boundary(offset))
            .then(|| u32::try_from(offset).ok().map(TextSize::new))
            .flatten()
    }

    #[must_use]
    pub fn text_position(&self, offset: TextSize) -> Option<TextPosition> {
        let offset = usize::try_from(offset.to_u32()).ok()?;
        if offset > self.text.len() || !self.text.is_char_boundary(offset) {
            return None;
        }
        let line_index = match self.line_starts.binary_search(&offset) {
            Ok(index) => index,
            Err(index) => index.checked_sub(1)?,
        };
        let line_start = *self.line_starts.get(line_index)?;
        Some(TextPosition {
            line: u32::try_from(line_index).ok()?,
            character: u32::try_from(offset - line_start).ok()?,
        })
    }

    #[must_use]
    pub fn text_range(&self, range: TextRange) -> Option<TextRangeUtf> {
        Some(TextRangeUtf {
            start: self.text_position(range.start())?,
            end: self.text_position(range.end())?,
        })
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
            .map(|error| parse_error_diagnostic(error, context))
            .collect());
    }
    if !parsed.unsupported_syntax_errors().is_empty() {
        return Err(parsed
            .unsupported_syntax_errors()
            .iter()
            .map(|error| unsupported_syntax_diagnostic(error, context))
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

fn parse_error_diagnostic(error: &ParseError, context: Option<&str>) -> RenderedDiagnostic {
    parse_diagnostic(parse_error_details(&error.error), context)
}

fn unsupported_syntax_diagnostic(
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

fn parse_diagnostic(details: ParseDiagnosticDetails, context: Option<&str>) -> RenderedDiagnostic {
    let message = details
        .template
        .replace(&format!("{{{}}}", details.arg_name), &details.arg_value);
    let mut args = BTreeMap::new();
    args.insert(
        details.arg_name.to_string(),
        DiagnosticArg::String(details.arg_value),
    );
    args.insert(
        "parser_category".to_string(),
        DiagnosticArg::String(details.parser_category.to_string()),
    );
    let children = context
        .map(|label| RenderedDiagnosticChild {
            severity: ChildSeverity::Note,
            message: format!("while parsing {label}"),
        })
        .into_iter()
        .collect();
    RenderedDiagnostic {
        code: details.code.code().to_string(),
        severity: Severity::Error,
        message,
        message_template: details.template.to_string(),
        args,
        url: details.code.docs_url(),
        spans: Vec::new(),
        children,
        help: None,
        suggestions: Vec::new(),
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
    use super::{parse_module, SourceText, TextPosition};

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
}
