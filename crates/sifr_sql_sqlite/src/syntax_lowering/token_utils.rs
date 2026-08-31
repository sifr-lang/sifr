use crate::lexer::{Keyword, Token};
use crate::parser::{SqliteParseError, is_keyword, parse_error};

pub(super) fn find_keyword(tokens: &[Token], keyword: Keyword, start: usize) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, token)| is_keyword(token, keyword).then_some(index))
}

pub(super) fn find_top_level_keyword(
    tokens: &[Token],
    keyword: Keyword,
    start: usize,
) -> Option<usize> {
    let mut depth = 0_u32;
    for (index, token) in tokens.iter().enumerate().skip(start) {
        match token {
            Token::LeftParen => depth = depth.saturating_add(1),
            Token::RightParen => depth = depth.saturating_sub(1),
            _ if depth == 0 && is_keyword(token, keyword) => return Some(index),
            _ => {}
        }
    }
    None
}

pub(super) fn split_top_level<'tokens>(
    tokens: &'tokens [Token],
    separator: &Token,
) -> Vec<&'tokens [Token]> {
    let mut parts = Vec::new();
    let mut depth = 0_u32;
    let mut start = 0;
    for (index, token) in tokens.iter().enumerate() {
        match token {
            Token::LeftParen => depth = depth.saturating_add(1),
            Token::RightParen => depth = depth.saturating_sub(1),
            _ if depth == 0 && token == separator => {
                parts.push(&tokens[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    if start < tokens.len() {
        parts.push(&tokens[start..]);
    }
    parts
}

pub(super) fn identifier_path(tokens: &[Token], start: usize) -> Option<(Vec<String>, usize)> {
    let mut path = vec![tokens.get(start)?.identifier()?.to_string()];
    let mut cursor = start + 1;
    while tokens.get(cursor) == Some(&Token::Dot) {
        path.push(tokens.get(cursor + 1)?.identifier()?.to_string());
        cursor += 2;
    }
    Some((path, cursor))
}

pub(super) fn matching_right(tokens: &[Token], open: usize) -> Option<usize> {
    let mut depth = 0_u32;
    for (index, token) in tokens.iter().enumerate().skip(open) {
        match token {
            Token::LeftParen => depth = depth.saturating_add(1),
            Token::RightParen => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

pub(super) fn validate_parentheses(
    tokens: &[Token],
    offset: usize,
) -> Result<(), SqliteParseError> {
    let mut depth = 0_i64;
    for token in tokens {
        match token {
            Token::LeftParen => depth += 1,
            Token::RightParen => {
                depth -= 1;
                if depth < 0 {
                    return Err(parse_error(offset, "unexpected closing parenthesis"));
                }
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err(parse_error(offset, "unclosed parenthesis"));
    }
    Ok(())
}

pub(super) fn key_columns(tokens: &[Token], start: usize) -> Result<Vec<String>, SqliteParseError> {
    let open = tokens
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, token)| (*token == Token::LeftParen).then_some(index))
        .ok_or_else(|| parse_error(0, "key needs a column list"))?;
    let close = matching_right(tokens, open)
        .ok_or_else(|| parse_error(0, "key column list is not closed"))?;
    split_top_level(&tokens[open + 1..close], &Token::Comma)
        .into_iter()
        .map(single_identifier)
        .collect()
}

pub(super) fn single_identifier(tokens: &[Token]) -> Result<String, SqliteParseError> {
    tokens
        .first()
        .and_then(Token::identifier)
        .map(str::to_string)
        .ok_or_else(|| parse_error(0, "expected an identifier"))
}

pub(super) fn optional_key_name(tokens: &[Token], start: usize) -> Option<String> {
    tokens
        .iter()
        .skip(start)
        .take_while(|token| **token != Token::LeftParen)
        .find_map(Token::identifier)
        .map(str::to_string)
}

pub(super) fn option_value(tokens: &[Token], keyword: Keyword) -> Option<String> {
    let index = find_keyword(tokens, keyword, 0)?;
    tokens
        .get(index + 1)
        .filter(|token| **token != Token::Operator("=".to_string()))
        .or_else(|| tokens.get(index + 2))
        .and_then(|token| token_word(Some(token)))
}

pub(super) fn value_after(tokens: &[Token], keyword: Keyword) -> Option<String> {
    let index = find_keyword(tokens, keyword, 0)?;
    tokens.get(index + 1).map(Token::normalized)
}

pub(super) fn token_word(token: Option<&Token>) -> Option<String> {
    match token? {
        Token::Identifier(value) | Token::QuotedIdentifier(value) => Some(value.clone()),
        Token::Keyword(keyword) => Some(keyword.text().to_ascii_lowercase()),
        Token::String(value) | Token::Number(value) => Some(value.clone()),
        _ => None,
    }
}

pub(super) fn contains_sequence(tokens: &[Token], keywords: &[Keyword]) -> bool {
    tokens.windows(keywords.len()).any(|window| {
        window
            .iter()
            .zip(keywords)
            .all(|(token, keyword)| is_keyword(token, *keyword))
    })
}
