use crate::ast::{MysqlStatement, SqlSpan};
use crate::lexer::{Keyword, LexError, Lexer, Token};
use crate::syntax_lowering::lower_statement;
use crate::types::{MysqlServerSeries, SUPPORTED_MYSQL_SERIES};
use lalrpop_util::{ParseError, lalrpop_mod};
use std::collections::BTreeSet;
use std::fmt;

lalrpop_mod!(
    #[allow(
        clippy::all,
        clippy::pedantic,
        clippy::nursery,
        clippy::restriction,
        unreachable_pub
    )]
    mysql
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RawStatement {
    pub(crate) tokens: Vec<Token>,
    pub(crate) span: SqlSpan,
}

impl RawStatement {
    pub(crate) fn new(start: usize, head: Token, tail: Vec<Token>, end: usize) -> Self {
        let mut tokens = Vec::with_capacity(tail.len() + 1);
        tokens.push(head);
        tokens.extend(tail);
        Self {
            tokens,
            span: SqlSpan {
                start: u32::try_from(start).unwrap_or(u32::MAX),
                end: u32::try_from(end).unwrap_or(u32::MAX),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MysqlParseError {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for MysqlParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at byte {}", self.message, self.offset)
    }
}

impl std::error::Error for MysqlParseError {}

#[derive(Clone, Debug)]
pub struct MysqlParser {
    series: MysqlServerSeries,
    sql_modes: BTreeSet<String>,
    default_character_set: String,
    default_collation: String,
}

impl MysqlParser {
    pub fn new(
        series: MysqlServerSeries,
        sql_modes: impl IntoIterator<Item = impl Into<String>>,
        default_character_set: impl Into<String>,
        default_collation: impl Into<String>,
    ) -> Result<Self, MysqlParseError> {
        if !SUPPORTED_MYSQL_SERIES.contains(&series) {
            return Err(parse_error(0, "unsupported MySQL server series"));
        }
        let sql_modes = sql_modes
            .into_iter()
            .map(Into::into)
            .map(|mode: String| mode.to_ascii_uppercase())
            .collect::<BTreeSet<_>>();
        if sql_modes.iter().any(|mode| {
            mode.is_empty()
                || !mode
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte == b'_')
        }) {
            return Err(parse_error(0, "invalid MySQL SQL mode"));
        }
        let default_character_set = default_character_set.into().to_ascii_lowercase();
        if !valid_character_set(&default_character_set) {
            return Err(parse_error(0, "invalid MySQL default character set"));
        }
        let default_collation = default_collation.into().to_ascii_lowercase();
        if !valid_collation(&default_collation) {
            return Err(parse_error(0, "invalid MySQL default collation"));
        }
        Ok(Self {
            series,
            sql_modes,
            default_character_set,
            default_collation,
        })
    }

    #[must_use]
    pub fn series(&self) -> MysqlServerSeries {
        self.series
    }

    #[must_use]
    pub fn sql_modes(&self) -> &BTreeSet<String> {
        &self.sql_modes
    }

    #[must_use]
    pub fn default_collation(&self) -> &str {
        &self.default_collation
    }

    #[must_use]
    pub fn default_character_set(&self) -> &str {
        &self.default_character_set
    }

    pub fn parse(&self, source: &str) -> Result<Vec<MysqlStatement>, MysqlParseError> {
        if source.len() > u32::MAX as usize {
            return Err(parse_error(0, "MySQL source exceeds the component limit"));
        }
        let raw = mysql::StatementsParser::new()
            .parse(Lexer::with_ansi_quotes(
                source,
                self.sql_modes.contains("ANSI_QUOTES"),
            ))
            .map_err(map_parse_error)?;
        if raw.is_empty() {
            return Err(parse_error(0, "MySQL source contains no statement"));
        }
        raw.into_iter()
            .map(|statement| lower_statement(&statement, self))
            .collect()
    }

    pub fn normalize(&self, source: &str) -> Result<String, MysqlParseError> {
        self.parse(source)?;
        let tokens = Lexer::with_ansi_quotes(source, self.sql_modes.contains("ANSI_QUOTES"))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| parse_error(error.offset, error.message))?
            .into_iter()
            .map(|(_, token, _)| token)
            .collect::<Vec<_>>();
        Ok(normalize_tokens(&tokens))
    }
}

fn map_parse_error(error: ParseError<usize, Token, LexError>) -> MysqlParseError {
    match error {
        ParseError::InvalidToken { location } => parse_error(location, "invalid MySQL token"),
        ParseError::UnrecognizedEof { location, expected } => parse_error(
            location,
            format!(
                "incomplete MySQL statement; expected {}",
                expected.join(", ")
            ),
        ),
        ParseError::UnrecognizedToken { token, expected } => parse_error(
            token.0,
            format!(
                "unexpected MySQL token '{}'; expected {}",
                token.1.normalized(),
                expected.join(", ")
            ),
        ),
        ParseError::ExtraToken { token } => parse_error(
            token.0,
            format!("extra MySQL token '{}'", token.1.normalized()),
        ),
        ParseError::User { error } => parse_error(error.offset, error.message),
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

pub(crate) fn parse_error(offset: usize, message: impl Into<String>) -> MysqlParseError {
    MysqlParseError {
        offset,
        message: message.into(),
    }
}

pub(crate) fn is_keyword(token: &Token, keyword: Keyword) -> bool {
    *token == Token::Keyword(keyword)
}

fn valid_collation(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn valid_character_set(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
