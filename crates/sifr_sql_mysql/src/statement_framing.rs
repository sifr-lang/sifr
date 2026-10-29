//! Statement boundaries and admission; detailed SQL syntax belongs to lowering.
use crate::lexer::{Keyword, SpannedToken, Token};
use crate::parser::{MysqlParseError, RawStatement, parse_error};

// Declaration order is observable in diagnostics and is deliberately retained.
const STARTERS: [Keyword; 9] = [
    Keyword::Select,
    Keyword::With,
    Keyword::Insert,
    Keyword::Replace,
    Keyword::Update,
    Keyword::Delete,
    Keyword::Create,
    Keyword::Alter,
    Keyword::Drop,
];

pub(crate) fn statements(
    input: impl IntoIterator<Item = SpannedToken>,
) -> Result<Vec<RawStatement>, MysqlParseError> {
    let mut completed = Vec::new();
    let mut current: Option<RawStatement> = None;
    for item in input {
        let (start, token, end) = item.map_err(|error| parse_error(error.offset, error.message))?;
        if token == Token::Semicolon
            && let Some(statement) = current.take()
        {
            completed.push(statement);
            continue;
        }
        if let Some(statement) = current.as_mut() {
            statement.tokens.push(token);
            statement.span.end = u32::try_from(end).unwrap_or(u32::MAX);
        } else if matches!(&token, Token::Keyword(keyword) if STARTERS.contains(keyword)) {
            current = Some(RawStatement::new(start, token, Vec::new(), end));
        } else {
            let expected = STARTERS
                .iter()
                .map(|keyword| format!("\"{}\"", keyword.text()))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(parse_error(
                start,
                format!(
                    "unexpected MySQL token '{}'; expected {expected}",
                    token.normalized()
                ),
            ));
        }
    }
    if let Some(statement) = current {
        completed.push(statement);
    }
    Ok(completed)
}

#[cfg(test)]
#[path = "../tests/unit/statement_framing.rs"]
mod tests;
