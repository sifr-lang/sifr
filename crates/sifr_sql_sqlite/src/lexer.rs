use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Keyword {
    Select,
    With,
    Insert,
    Replace,
    Update,
    Delete,
    Create,
    Alter,
    Drop,
    Table,
    View,
    Index,
    Trigger,
    Unique,
    Primary,
    Key,
    Foreign,
    References,
    Check,
    Into,
    Values,
    From,
    Where,
    Join,
    Left,
    Right,
    Inner,
    Cross,
    On,
    As,
    Set,
    And,
    Or,
    Not,
    Null,
    Default,
    Generated,
    Always,
    Virtual,
    Stored,
    AutoIncrement,
    Unsigned,
    Zerofill,
    Character,
    Charset,
    Collate,
    Constraint,
    Ignore,
    Duplicate,
    Group,
    By,
    Having,
    Order,
    Limit,
    Offset,
    Distinct,
    Window,
    Over,
    For,
    Returning,
    Union,
    All,
    Strict,
    Without,
    Rowid,
    Rollback,
    Abort,
    Fail,
    Do,
    Nothing,
    Conflict,
    Desc,
    If,
    Exists,
}

impl Keyword {
    #[must_use]
    pub fn text(self) -> &'static str {
        match self {
            Self::Select => "SELECT",
            Self::With => "WITH",
            Self::Insert => "INSERT",
            Self::Replace => "REPLACE",
            Self::Update => "UPDATE",
            Self::Delete => "DELETE",
            Self::Create => "CREATE",
            Self::Alter => "ALTER",
            Self::Drop => "DROP",
            Self::Table => "TABLE",
            Self::View => "VIEW",
            Self::Index => "INDEX",
            Self::Trigger => "TRIGGER",
            Self::Unique => "UNIQUE",
            Self::Primary => "PRIMARY",
            Self::Key => "KEY",
            Self::Foreign => "FOREIGN",
            Self::References => "REFERENCES",
            Self::Check => "CHECK",
            Self::Into => "INTO",
            Self::Values => "VALUES",
            Self::From => "FROM",
            Self::Where => "WHERE",
            Self::Join => "JOIN",
            Self::Left => "LEFT",
            Self::Right => "RIGHT",
            Self::Inner => "INNER",
            Self::Cross => "CROSS",
            Self::On => "ON",
            Self::As => "AS",
            Self::Set => "SET",
            Self::And => "AND",
            Self::Or => "OR",
            Self::Not => "NOT",
            Self::Null => "NULL",
            Self::Default => "DEFAULT",
            Self::Generated => "GENERATED",
            Self::Always => "ALWAYS",
            Self::Virtual => "VIRTUAL",
            Self::Stored => "STORED",
            Self::AutoIncrement => "AUTOINCREMENT",
            Self::Unsigned => "UNSIGNED",
            Self::Zerofill => "ZEROFILL",
            Self::Character => "CHARACTER",
            Self::Charset => "CHARSET",
            Self::Collate => "COLLATE",
            Self::Constraint => "CONSTRAINT",
            Self::Ignore => "IGNORE",
            Self::Duplicate => "DUPLICATE",
            Self::Group => "GROUP",
            Self::By => "BY",
            Self::Having => "HAVING",
            Self::Order => "ORDER",
            Self::Limit => "LIMIT",
            Self::Offset => "OFFSET",
            Self::Distinct => "DISTINCT",
            Self::Window => "WINDOW",
            Self::Over => "OVER",
            Self::For => "FOR",
            Self::Returning => "RETURNING",
            Self::Union => "UNION",
            Self::All => "ALL",
            Self::Strict => "STRICT",
            Self::Without => "WITHOUT",
            Self::Rowid => "ROWID",
            Self::Rollback => "ROLLBACK",
            Self::Abort => "ABORT",
            Self::Fail => "FAIL",
            Self::Do => "DO",
            Self::Nothing => "NOTHING",
            Self::Conflict => "CONFLICT",
            Self::Desc => "DESC",
            Self::If => "IF",
            Self::Exists => "EXISTS",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Token {
    Keyword(Keyword),
    Identifier(String),
    QuotedIdentifier(String),
    String(String),
    Number(String),
    Parameter(String),
    Comma,
    Dot,
    LeftParen,
    RightParen,
    Operator(String),
    Semicolon,
}

impl Token {
    #[must_use]
    pub fn normalized(&self) -> String {
        match self {
            Self::Keyword(keyword) => keyword.text().to_string(),
            Self::Identifier(value) => value.to_ascii_lowercase(),
            Self::QuotedIdentifier(value) => format!("`{}`", value.replace('`', "``")),
            Self::String(_) => "?literal".to_string(),
            Self::Number(value) => value.clone(),
            Self::Parameter(value) => value.clone(),
            Self::Comma => ",".to_string(),
            Self::Dot => ".".to_string(),
            Self::LeftParen => "(".to_string(),
            Self::RightParen => ")".to_string(),
            Self::Operator(value) => value.clone(),
            Self::Semicolon => ";".to_string(),
        }
    }

    #[must_use]
    pub fn identifier(&self) -> Option<&str> {
        match self {
            Self::Identifier(value) | Self::QuotedIdentifier(value) => Some(value),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LexError {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for LexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at byte {}", self.message, self.offset)
    }
}

impl std::error::Error for LexError {}

pub type SpannedToken = Result<(usize, Token, usize), LexError>;

pub fn tokenize(source: &str) -> Vec<SpannedToken> {
    Lexer::new(source).collect()
}

pub(crate) struct Lexer<'a> {
    source: &'a str,
    offset: usize,
    failed: bool,
    ansi_quotes: bool,
}

impl<'a> Lexer<'a> {
    #[must_use]
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            source,
            offset: 0,
            failed: false,
            ansi_quotes: true,
        }
    }

    #[must_use]
    pub(crate) fn with_ansi_quotes(source: &'a str, ansi_quotes: bool) -> Self {
        Self {
            source,
            offset: 0,
            failed: false,
            ansi_quotes,
        }
    }

    fn skip_layout(&mut self) -> Result<(), LexError> {
        loop {
            let remainder = &self.source[self.offset..];
            let trimmed = remainder.trim_start_matches(char::is_whitespace);
            self.offset += remainder.len() - trimmed.len();
            let remainder = &self.source[self.offset..];
            if remainder.starts_with("--") {
                self.offset += remainder.find('\n').unwrap_or(remainder.len());
                continue;
            }
            if remainder.starts_with("/*") {
                let Some(end) = remainder.find("*/") else {
                    return Err(self.error("unterminated block comment"));
                };
                self.offset += end + 2;
                continue;
            }
            return Ok(());
        }
    }

    fn scan_quoted(&mut self, quote: u8, identifier: bool) -> Result<Token, LexError> {
        let start = self.offset;
        self.offset += 1;
        let mut value = String::new();
        while self.offset < self.source.len() {
            let bytes = self.source.as_bytes();
            let byte = bytes[self.offset];
            if byte == quote {
                if bytes.get(self.offset + 1) == Some(&quote) {
                    value.push(char::from(quote));
                    self.offset += 2;
                    continue;
                }
                self.offset += 1;
                return Ok(if identifier {
                    Token::QuotedIdentifier(value)
                } else {
                    Token::String(value)
                });
            }
            if byte == b'\\' && !identifier {
                let Some(next) = bytes.get(self.offset + 1).copied() else {
                    return Err(self.error("unterminated string escape"));
                };
                value.push(char::from(next));
                self.offset += 2;
                continue;
            }
            let character = self.source[self.offset..]
                .chars()
                .next()
                .ok_or_else(|| self.error("invalid UTF-8 boundary"))?;
            value.push(character);
            self.offset += character.len_utf8();
        }
        self.offset = start;
        Err(self.error("unterminated quoted value"))
    }

    fn scan_bracket_identifier(&mut self) -> Result<Token, LexError> {
        let start = self.offset;
        self.offset += 1;
        let Some(end) = self.source[self.offset..].find(']') else {
            self.offset = start;
            return Err(self.error("unterminated bracket identifier"));
        };
        let value = self.source[self.offset..self.offset + end].to_string();
        self.offset += end + 1;
        Ok(Token::QuotedIdentifier(value))
    }

    fn error(&self, message: impl Into<String>) -> LexError {
        LexError {
            offset: self.offset,
            message: message.into(),
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = SpannedToken;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed || self.offset >= self.source.len() {
            return None;
        }
        if let Err(error) = self.skip_layout() {
            self.failed = true;
            return Some(Err(error));
        }
        if self.offset >= self.source.len() {
            return None;
        }
        let start = self.offset;
        let byte = self.source.as_bytes()[start];
        let token = match byte {
            b'`' => self.scan_quoted(b'`', true),
            b'[' => self.scan_bracket_identifier(),
            b'\'' => self.scan_quoted(byte, false),
            b'"' => self.scan_quoted(byte, self.ansi_quotes),
            b'?' => {
                self.offset += 1;
                while self
                    .source
                    .as_bytes()
                    .get(self.offset)
                    .is_some_and(u8::is_ascii_digit)
                {
                    self.offset += 1;
                }
                Ok(Token::Parameter(
                    self.source[start..self.offset].to_string(),
                ))
            }
            b':' | b'@' | b'$'
                if self
                    .source
                    .as_bytes()
                    .get(start + 1)
                    .is_some_and(u8::is_ascii_alphabetic) =>
            {
                self.offset += 2;
                while self
                    .source
                    .as_bytes()
                    .get(self.offset)
                    .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                {
                    self.offset += 1;
                }
                Ok(Token::Parameter(
                    self.source[start..self.offset].to_string(),
                ))
            }
            b',' => {
                self.offset += 1;
                Ok(Token::Comma)
            }
            b'.' => {
                self.offset += 1;
                Ok(Token::Dot)
            }
            b'(' => {
                self.offset += 1;
                Ok(Token::LeftParen)
            }
            b')' => {
                self.offset += 1;
                Ok(Token::RightParen)
            }
            b';' => {
                self.offset += 1;
                Ok(Token::Semicolon)
            }
            byte if byte.is_ascii_digit() => {
                self.offset += 1;
                while self.source.as_bytes().get(self.offset).is_some_and(|byte| {
                    byte.is_ascii_digit() || matches!(byte, b'.' | b'e' | b'E' | b'+' | b'-')
                }) {
                    self.offset += 1;
                }
                Ok(Token::Number(self.source[start..self.offset].to_string()))
            }
            byte if byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$') => {
                self.offset += 1;
                while self
                    .source
                    .as_bytes()
                    .get(self.offset)
                    .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
                {
                    self.offset += 1;
                }
                let value = &self.source[start..self.offset];
                Ok(keyword(value)
                    .map_or_else(|| Token::Identifier(value.to_string()), Token::Keyword))
            }
            byte if matches!(
                byte,
                b'=' | b'<' | b'>' | b'!' | b'+' | b'-' | b'*' | b'/' | b'%' | b'|' | b'&' | b'~'
            ) =>
            {
                self.offset += 1;
                if self.source.as_bytes().get(self.offset).is_some_and(|next| {
                    matches!(
                        (byte, *next),
                        (b'<' | b'>' | b'!', b'=')
                            | (b'<', b'>' | b'<')
                            | (b'>' | b'-', b'>')
                            | (b'|', b'|')
                    )
                }) {
                    self.offset += 1;
                    if byte == b'-' && self.source.as_bytes().get(self.offset) == Some(&b'>') {
                        self.offset += 1;
                    }
                }
                Ok(Token::Operator(self.source[start..self.offset].to_string()))
            }
            _ => Err(self.error("unsupported SQLite token")),
        };
        match token {
            Ok(token) => Some(Ok((start, token, self.offset))),
            Err(error) => {
                self.failed = true;
                Some(Err(error))
            }
        }
    }
}

fn keyword(value: &str) -> Option<Keyword> {
    Some(match value.to_ascii_uppercase().as_str() {
        "SELECT" => Keyword::Select,
        "WITH" => Keyword::With,
        "INSERT" => Keyword::Insert,
        "REPLACE" => Keyword::Replace,
        "UPDATE" => Keyword::Update,
        "DELETE" => Keyword::Delete,
        "CREATE" => Keyword::Create,
        "ALTER" => Keyword::Alter,
        "DROP" => Keyword::Drop,
        "TABLE" => Keyword::Table,
        "VIEW" => Keyword::View,
        "INDEX" => Keyword::Index,
        "TRIGGER" => Keyword::Trigger,
        "UNIQUE" => Keyword::Unique,
        "PRIMARY" => Keyword::Primary,
        "KEY" => Keyword::Key,
        "FOREIGN" => Keyword::Foreign,
        "REFERENCES" => Keyword::References,
        "CHECK" => Keyword::Check,
        "INTO" => Keyword::Into,
        "VALUES" => Keyword::Values,
        "FROM" => Keyword::From,
        "WHERE" => Keyword::Where,
        "JOIN" => Keyword::Join,
        "LEFT" => Keyword::Left,
        "RIGHT" => Keyword::Right,
        "INNER" => Keyword::Inner,
        "CROSS" => Keyword::Cross,
        "ON" => Keyword::On,
        "AS" => Keyword::As,
        "SET" => Keyword::Set,
        "AND" => Keyword::And,
        "OR" => Keyword::Or,
        "NOT" => Keyword::Not,
        "NULL" => Keyword::Null,
        "DEFAULT" => Keyword::Default,
        "GENERATED" => Keyword::Generated,
        "ALWAYS" => Keyword::Always,
        "VIRTUAL" => Keyword::Virtual,
        "STORED" => Keyword::Stored,
        "AUTOINCREMENT" => Keyword::AutoIncrement,
        "UNSIGNED" => Keyword::Unsigned,
        "ZEROFILL" => Keyword::Zerofill,
        "CHARACTER" => Keyword::Character,
        "CHARSET" => Keyword::Charset,
        "COLLATE" => Keyword::Collate,
        "CONSTRAINT" => Keyword::Constraint,
        "IGNORE" => Keyword::Ignore,
        "DUPLICATE" => Keyword::Duplicate,
        "GROUP" => Keyword::Group,
        "BY" => Keyword::By,
        "HAVING" => Keyword::Having,
        "ORDER" => Keyword::Order,
        "LIMIT" => Keyword::Limit,
        "OFFSET" => Keyword::Offset,
        "DISTINCT" => Keyword::Distinct,
        "WINDOW" => Keyword::Window,
        "OVER" => Keyword::Over,
        "FOR" => Keyword::For,
        "RETURNING" => Keyword::Returning,
        "UNION" => Keyword::Union,
        "ALL" => Keyword::All,
        "STRICT" => Keyword::Strict,
        "WITHOUT" => Keyword::Without,
        "ROWID" => Keyword::Rowid,
        "ROLLBACK" => Keyword::Rollback,
        "ABORT" => Keyword::Abort,
        "FAIL" => Keyword::Fail,
        "DO" => Keyword::Do,
        "NOTHING" => Keyword::Nothing,
        "CONFLICT" => Keyword::Conflict,
        "DESC" => Keyword::Desc,
        "IF" => Keyword::If,
        "EXISTS" => Keyword::Exists,
        _ => return None,
    })
}
