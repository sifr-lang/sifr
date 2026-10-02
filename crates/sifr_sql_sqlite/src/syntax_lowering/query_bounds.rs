//! Closed literal row bounds. Expressions and parameters remain conservative.
use super::{Keyword, Token, find_top_level_keyword};

pub(super) fn row_bounds(tokens: &[Token]) -> (Option<u64>, Option<u64>, bool) {
    let Some(index) = find_top_level_keyword(tokens, Keyword::Limit, 0) else {
        return (
            None,
            None,
            find_top_level_keyword(tokens, Keyword::Offset, 0).is_none(),
        );
    };
    let clause = &tokens[index + 1..];
    let clause = clause.strip_suffix(&[Token::Semicolon]).unwrap_or(clause);
    let (limit, offset) = match clause {
        [Token::Number(limit)] => (limit.parse().ok(), Some(0)),
        [
            Token::Number(limit),
            Token::Keyword(Keyword::Offset),
            Token::Number(offset),
        ] => (limit.parse().ok(), offset.parse().ok()),
        [Token::Number(offset), Token::Comma, Token::Number(limit)] => {
            (limit.parse().ok(), offset.parse().ok())
        }
        _ => return (None, None, false),
    };
    (limit, offset, limit.is_some() && offset.is_some())
}
