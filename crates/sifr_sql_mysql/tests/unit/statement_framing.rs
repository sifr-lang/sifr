use super::statements;
use crate::lexer::{Keyword, LexError, Lexer, Token};

const EXPECTED: &str = "\"SELECT\", \"WITH\", \"INSERT\", \"REPLACE\", \"UPDATE\", \"DELETE\", \"CREATE\", \"ALTER\", \"DROP\"";

#[test]
fn statement_boundaries_preserve_quoted_separators_tokens_and_spans() {
    let source = "  SELECT 'a;b'; UPDATE t SET x=2; ";
    let result = statements(Lexer::new(source)).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!((result[0].span.start, result[0].span.end), (2, 14));
    assert_eq!(
        result[0].tokens,
        vec![Token::Keyword(Keyword::Select), Token::String("a;b".into())]
    );
    assert_eq!((result[1].span.start, result[1].span.end), (16, 32));
    assert_eq!(
        result[1].tokens.first(),
        Some(&Token::Keyword(Keyword::Update))
    );
    assert_eq!(result[1].tokens.last(), Some(&Token::Number("2".into())));
    assert!(
        result
            .iter()
            .all(|statement| !statement.tokens.contains(&Token::Semicolon))
    );
}

#[test]
fn framing_accepts_all_supported_heads_and_leaves_syntax_to_lowering() {
    for source in [
        "SELECT", "WITH", "INSERT", "REPLACE", "UPDATE", "DELETE", "CREATE", "ALTER", "DROP",
    ] {
        let result = statements(Lexer::new(source)).unwrap();
        assert_eq!(result.len(), 1, "{source}");
        assert_eq!(result[0].span.end, u32::try_from(source.len()).unwrap());
        assert_eq!(result[0].tokens.len(), 1);
    }
    assert!(statements(Lexer::new("SELECT ( 1")).is_ok());
}

#[test]
fn empty_input_and_optional_final_separator_have_distinct_contracts() {
    for source in ["", "  ", "-- comment\n", "/* comment */"] {
        assert!(statements(Lexer::new(source)).unwrap().is_empty());
    }
    for source in ["SELECT 1", "SELECT 1;", "SELECT 1; -- trailing comment"] {
        let result = statements(Lexer::new(source)).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!((result[0].span.start, result[0].span.end), (0, 8));
    }
    for (source, offset, token) in [
        (";", 0, ";"),
        ("SELECT 1;;", 9, ";"),
        ("TABLE t", 0, "TABLE"),
        ("SELECT 1; TABLE t", 10, "TABLE"),
        ("name", 0, "name"),
    ] {
        let error = statements(Lexer::new(source)).unwrap_err();
        assert_eq!(error.offset, offset, "{source}");
        assert_eq!(
            error.message,
            format!("unexpected MySQL token '{token}'; expected {EXPECTED}")
        );
    }
}

#[test]
fn lexical_errors_preserve_precedence_without_reading_past_a_syntax_error() {
    let lexical = LexError {
        offset: 7,
        message: "owned lexical diagnostic".into(),
    };
    let error = statements([
        Ok((0, Token::Keyword(Keyword::Select), 6)),
        Err(lexical.clone()),
    ])
    .unwrap_err();
    assert_eq!(
        (error.offset, error.message),
        (lexical.offset, lexical.message)
    );
    let mut consumed = 0;
    let input = [
        Ok((0, Token::Keyword(Keyword::Table), 5)),
        Err(LexError {
            offset: 6,
            message: "must not replace earlier error".into(),
        }),
    ];
    let error = statements(input.into_iter().inspect(|_| consumed += 1)).unwrap_err();
    assert_eq!(consumed, 1);
    assert_eq!(error.offset, 0);
    assert!(error.message.starts_with("unexpected MySQL token 'TABLE'"));
}

#[test]
fn framing_retains_ansi_quote_mode_and_byte_offsets() {
    let source = "SELECT \"é\"";
    for (ansi, expected) in [
        (false, Token::String("é".into())),
        (true, Token::QuotedIdentifier("é".into())),
    ] {
        let result = statements(Lexer::with_ansi_quotes(source, ansi)).unwrap();
        assert_eq!(result[0].tokens[1], expected);
        assert_eq!((result[0].span.start, result[0].span.end), (0, 11));
    }
}
