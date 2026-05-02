use crate::diagnostics::RenderedDiagnostic;
use sifr_diagnostics::render::RenderedDiagnosticChild;
use sifr_diagnostics::{ChildSeverity, DiagnosticArg, DiagnosticCode};
use sifr_python_ast::{ModModule, PythonVersion};
use sifr_python_parser::{
    parse_unchecked, InterpolatedStringErrorType, LexicalErrorType, Mode, ParseError,
    ParseErrorType, ParseOptions, Parsed, UnsupportedSyntaxError,
};
use std::collections::BTreeMap;

pub(crate) fn parse_module_with_diagnostics(
    source: &str,
    context: Option<&str>,
) -> Result<Parsed<ModModule>, Vec<RenderedDiagnostic>> {
    let parsed = parse_unchecked(
        source,
        // Sifr owns the latest Python-derived syntax surface pre-1.0; do not
        // reject syntax only because Ruff's default target is older.
        ParseOptions::from(Mode::Module).with_target_version(PythonVersion::latest_ty()),
    );
    let Some(parsed) = parsed.try_into_module() else {
        unreachable!("module parse mode must produce a module syntax tree");
    };
    if parsed.has_invalid_syntax() {
        // Grammar errors are primary. Ruff version diagnostics are only useful
        // after the parser has produced a syntactically valid module.
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
    let message = render_template(details.template, details.arg_name, &details.arg_value);
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
        severity: details.code.declared_severity(),
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

fn render_template(template: &str, arg_name: &str, arg_value: &str) -> String {
    template.replace(&format!("{{{arg_name}}}"), arg_value)
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
    parser_category: &'static str,
    reason: &InterpolatedStringErrorType,
) -> ParseDiagnosticDetails {
    ParseDiagnosticDetails {
        code: DiagnosticCode::PARSE_LEXICAL_OR_STRING,
        template: "lexical error: {reason}",
        arg_name: "reason",
        arg_value: reason.to_string(),
        parser_category,
    }
}

fn layout_details(reason: impl Into<String>) -> ParseDiagnosticDetails {
    ParseDiagnosticDetails {
        code: DiagnosticCode::PARSE_LAYOUT,
        template: "invalid source layout: {reason}",
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
        template: "invalid target syntax: {target_kind}",
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
        template: "invalid call argument syntax: {reason}",
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
        parser_category: "malformed_declaration_list",
    }
}

fn invalid_pattern_details(reason: impl Into<String>) -> ParseDiagnosticDetails {
    ParseDiagnosticDetails {
        code: DiagnosticCode::PARSE_INVALID_PATTERN,
        template: "invalid match pattern syntax: {reason}",
        arg_name: "reason",
        arg_value: reason.into(),
        parser_category: "invalid_pattern",
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
