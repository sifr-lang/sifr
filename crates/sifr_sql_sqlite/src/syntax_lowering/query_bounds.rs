//! Closed literal row bounds. Expressions and parameters remain conservative.
use super::{Keyword, Token, find_top_level_keyword, split_top_level};

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
    let (limit, offset) = match split_top_level(clause, &Token::Comma).as_slice() {
        [offset, limit] => (literal_bound(limit), literal_bound(offset)),
        [bounds] => {
            if let Some(offset) = find_top_level_keyword(bounds, Keyword::Offset, 0) {
                (
                    literal_bound(&bounds[..offset]),
                    literal_bound(&bounds[offset + 1..]),
                )
            } else {
                (literal_bound(bounds), Some(0))
            }
        }
        _ => return (None, None, false),
    };
    (limit, offset, limit.is_some() && offset.is_some())
}

fn literal_bound(tokens: &[Token]) -> Option<u64> {
    match tokens {
        [Token::Number(value)] => value.parse().ok(),
        _ => None,
    }
}
