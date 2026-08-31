use crate::ast::{SqlSpan, SqliteStatement};
use crate::lexer::{Keyword, Lexer, Token};
use crate::syntax_lowering::lower_statement;
use crate::types::{SUPPORTED_SQLITE_SERIES, SqliteServerSeries};
use std::collections::BTreeSet;
use std::fmt;
use syntaqlite::{ParseOutcome, Parser};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RawStatement {
    pub(crate) tokens: Vec<Token>,
    pub(crate) span: SqlSpan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqliteParseError {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for SqliteParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at byte {}", self.message, self.offset)
    }
}

impl std::error::Error for SqliteParseError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqliteParser {
    series: SqliteServerSeries,
    compile_flags: BTreeSet<String>,
}

impl SqliteParser {
    pub fn new(
        series: SqliteServerSeries,
        compile_flags: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, SqliteParseError> {
        if !SUPPORTED_SQLITE_SERIES.contains(&series) {
            return Err(parse_error(0, "unsupported SQLite grammar version"));
        }
        let compile_flags = compile_flags
            .into_iter()
            .map(Into::into)
            .map(|flag: String| flag.to_ascii_uppercase())
            .collect::<BTreeSet<_>>();
        if !compile_flags.is_empty() {
            return Err(parse_error(
                0,
                "SQLite compile flags differ from the qualified bundled build",
            ));
        }
        Ok(Self {
            series,
            compile_flags,
        })
    }

    #[must_use]
    pub const fn series(&self) -> SqliteServerSeries {
        self.series
    }

    #[must_use]
    pub fn version(&self) -> String {
        self.series.version()
    }

    #[must_use]
    pub fn compile_flags(&self) -> &BTreeSet<String> {
        &self.compile_flags
    }

    pub fn parse(&self, source: &str) -> Result<Vec<SqliteStatement>, SqliteParseError> {
        if source.len() > u32::MAX as usize {
            return Err(parse_error(0, "SQLite source exceeds the component limit"));
        }
        validate_with_sqlite_grammar(source)?;
        let raw = split_statements(source)?;
        if raw.is_empty() {
            return Err(parse_error(0, "SQLite source contains no statement"));
        }
        raw.iter()
            .map(|statement| lower_statement(statement, self))
            .collect()
    }

    pub fn normalize(&self, source: &str) -> Result<String, SqliteParseError> {
        self.parse(source)?;
        let tokens = Lexer::with_ansi_quotes(source, true)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| parse_error(error.offset, error.message))?
            .into_iter()
            .map(|(_, token, _)| token)
            .collect::<Vec<_>>();
        Ok(normalize_tokens(&tokens))
    }
}

fn validate_with_sqlite_grammar(source: &str) -> Result<(), SqliteParseError> {
    let parser = Parser::new();
    let mut session = parser.parse(source);
    let mut statements = 0_u32;
    loop {
        match session.next() {
            ParseOutcome::Ok(_) => statements = statements.saturating_add(1),
            ParseOutcome::Err(error) => {
                let base = error.statement_base().as_doc_offset().as_u32();
                let local = error
                    .offset()
                    .map_or(0, syntaqlite::source::StmtOffset::as_u32);
                return Err(parse_error(
                    usize::try_from(base.saturating_add(local)).unwrap_or(usize::MAX),
                    format!("SQLite grammar error: {}", error.message()),
                ));
            }
            ParseOutcome::Done => break,
        }
    }
    if statements == 0 && !source.trim().is_empty() {
        return Err(parse_error(0, "SQLite source contains no statement"));
    }
    Ok(())
}

fn split_statements(source: &str) -> Result<Vec<RawStatement>, SqliteParseError> {
    let lexed = Lexer::with_ansi_quotes(source, true)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| parse_error(error.offset, error.message))?;
    let mut statements = Vec::new();
    let mut tokens = Vec::new();
    let mut start = 0_usize;
    let mut end = 0_usize;
    for (token_start, token, token_end) in lexed {
        if tokens.is_empty() {
            start = token_start;
        }
        end = token_end;
        if token == Token::Semicolon {
            if !tokens.is_empty() {
                statements.push(raw_statement(start, std::mem::take(&mut tokens), token_end));
            }
        } else {
            tokens.push(token);
        }
    }
    if !tokens.is_empty() {
        statements.push(raw_statement(start, tokens, end));
    }
    Ok(statements)
}

fn raw_statement(start: usize, tokens: Vec<Token>, end: usize) -> RawStatement {
    RawStatement {
        tokens,
        span: SqlSpan {
            start: u32::try_from(start).unwrap_or(u32::MAX),
            end: u32::try_from(end).unwrap_or(u32::MAX),
        },
    }
}

pub(crate) fn normalize_tokens(tokens: &[Token]) -> String {
    let mut output = String::new();
    let mut previous: Option<&Token> = None;
    for token in tokens {
        let no_space_before = matches!(
            token,
            Token::Comma | Token::Dot | Token::RightParen | Token::Semicolon
        );
        let no_space_after_previous = matches!(previous, Some(Token::Dot | Token::LeftParen));
        if !output.is_empty() && !no_space_before && !no_space_after_previous {
            output.push(' ');
        }
        output.push_str(&token.normalized());
        previous = Some(token);
    }
    output
}

pub(crate) fn parse_error(offset: usize, message: impl Into<String>) -> SqliteParseError {
    SqliteParseError {
        offset,
        message: message.into(),
    }
}

pub(crate) fn is_keyword(token: &Token, keyword: Keyword) -> bool {
    *token == Token::Keyword(keyword)
}
