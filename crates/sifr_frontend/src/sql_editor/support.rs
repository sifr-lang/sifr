use super::{SqlEditorSymbol, SqlEditorToken, SqlEditorTokenKind};
use crate::TemplateDocumentView;
use ruff_text_size::{TextRange, TextSize};
use sifr_sql_contract::{Cardinality, SchemaObjectKind, SemanticValue};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const SQL_KEYWORDS: &[&str] = &[
    "ALL",
    "AND",
    "AS",
    "ASC",
    "BY",
    "DELETE",
    "DESC",
    "DISTINCT",
    "EXISTS",
    "FROM",
    "FULL",
    "GROUP",
    "HAVING",
    "INNER",
    "INSERT",
    "INTO",
    "IS",
    "JOIN",
    "LEFT",
    "LIMIT",
    "NOT",
    "NULL",
    "OFFSET",
    "ON",
    "OR",
    "ORDER",
    "OUTER",
    "RETURNING",
    "RIGHT",
    "SELECT",
    "SET",
    "UPDATE",
    "VALUES",
    "WHERE",
    "WITH",
];

pub(super) fn lex_sql(template: &TemplateDocumentView) -> Vec<SqlEditorToken> {
    let source = template.source.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0_usize;
    let mut hole = 0_usize;
    while index < source.len() {
        let byte = source[index];
        if byte.is_ascii_whitespace() {
            index += 1;
            continue;
        }
        let start = index;
        let kind = if byte == 0xef && source.get(index..index + 3) == Some("\u{fffc}".as_bytes()) {
            index += 3;
            let current = hole;
            hole += 1;
            SqlEditorTokenKind::Hole { index: current }
        } else if byte == b'\'' || byte == b'"' {
            let quote = byte;
            index += 1;
            while index < source.len() {
                if source[index] == quote {
                    index += 1;
                    if source.get(index) == Some(&quote) {
                        index += 1;
                        continue;
                    }
                    break;
                }
                index += 1;
            }
            SqlEditorTokenKind::String
        } else if byte.is_ascii_digit() {
            index += 1;
            while source
                .get(index)
                .is_some_and(|next| next.is_ascii_digit() || *next == b'.')
            {
                index += 1;
            }
            SqlEditorTokenKind::Number
        } else if byte.is_ascii_alphabetic() || byte == b'_' {
            index += 1;
            while source
                .get(index)
                .is_some_and(|next| next.is_ascii_alphanumeric() || *next == b'_')
            {
                index += 1;
            }
            let text = &template.source[start..index];
            if SQL_KEYWORDS.contains(&text.to_ascii_uppercase().as_str()) {
                SqlEditorTokenKind::Keyword
            } else {
                SqlEditorTokenKind::Identifier
            }
        } else {
            index += 1;
            while source.get(index).is_some_and(|next| {
                !next.is_ascii_whitespace()
                    && !next.is_ascii_alphanumeric()
                    && *next != b'_'
                    && *next != b'\''
                    && *next != b'"'
                    && *next != 0xef
            }) {
                index += 1;
            }
            SqlEditorTokenKind::Operator
        };
        let Ok(start) = u32::try_from(start) else {
            break;
        };
        let Ok(end) = u32::try_from(index) else {
            break;
        };
        tokens.push(SqlEditorToken {
            text: template.source[usize::try_from(start).unwrap_or_default()..index].to_string(),
            virtual_range: TextRange::new(TextSize::new(start), TextSize::new(end)),
            kind,
        });
    }
    tokens
}

pub(super) fn infer_cardinality(tokens: &[SqlEditorToken]) -> String {
    let has_limit_one = tokens
        .windows(2)
        .any(|pair| pair[0].text.eq_ignore_ascii_case("limit") && pair[1].text == "1");
    let aggregate = tokens.iter().any(|token| {
        matches!(token.kind, SqlEditorTokenKind::Identifier)
            && matches!(
                token.text.to_ascii_lowercase().as_str(),
                "count" | "sum" | "avg"
            )
    });
    if has_limit_one || aggregate {
        "zero-or-one".to_string()
    } else {
        "many".to_string()
    }
}

pub(super) fn qualifier_before(tokens: &[SqlEditorToken], offset: TextSize) -> Option<String> {
    let before = tokens
        .iter()
        .take_while(|token| token.virtual_range.end() <= offset)
        .collect::<Vec<_>>();
    let [identifier, dot] = before.as_slice().get(before.len().saturating_sub(2)..)? else {
        return None;
    };
    (identifier.kind == SqlEditorTokenKind::Identifier && dot.text == ".")
        .then(|| identifier.text.clone())
}

pub(super) fn relation_aliases(tokens: &[SqlEditorToken]) -> BTreeMap<String, String> {
    let mut aliases = BTreeMap::new();
    let mut index = 0;
    while index < tokens.len() {
        if !matches!(
            tokens[index].text.to_ascii_uppercase().as_str(),
            "FROM" | "JOIN" | "UPDATE" | "INTO"
        ) {
            index += 1;
            continue;
        }
        index += 1;
        let Some(first) = tokens
            .get(index)
            .filter(|token| token.kind == SqlEditorTokenKind::Identifier)
        else {
            continue;
        };
        let mut relation = first.text.clone();
        index += 1;
        while tokens.get(index).is_some_and(|token| token.text == ".")
            && tokens
                .get(index + 1)
                .is_some_and(|token| token.kind == SqlEditorTokenKind::Identifier)
        {
            relation.push('.');
            relation.push_str(&tokens[index + 1].text);
            index += 2;
        }
        if tokens
            .get(index)
            .is_some_and(|token| token.text.eq_ignore_ascii_case("AS"))
        {
            index += 1;
        }
        if let Some(alias) = tokens.get(index).filter(|token| {
            token.kind == SqlEditorTokenKind::Identifier
                && !SQL_KEYWORDS.contains(&token.text.to_ascii_uppercase().as_str())
        }) {
            aliases.insert(alias.text.clone(), relation.clone());
        }
        aliases
            .entry(relation.rsplit('.').next().unwrap_or(&relation).to_string())
            .or_insert(relation);
    }
    aliases
}

pub(super) fn symbol_allowed_in_fragment(
    symbol: &SqlEditorSymbol,
    relations: &BTreeSet<String>,
) -> bool {
    if symbol.kind == "relation" {
        return relations
            .iter()
            .any(|relation| relation_matches(&symbol.name, relation));
    }
    if symbol.kind == "column" {
        return symbol.name.rsplit_once('.').is_some_and(|(owner, _)| {
            relations
                .iter()
                .any(|relation| relation_matches(owner, relation))
        });
    }
    true
}

pub(super) fn symbol_matches_relation(symbol: &SqlEditorSymbol, relation: &str) -> bool {
    symbol.kind != "column"
        || symbol
            .name
            .rsplit_once('.')
            .is_some_and(|(owner, _)| relation_matches(owner, relation))
}

fn relation_matches(left: &str, right: &str) -> bool {
    left == right || left.ends_with(&format!(".{right}")) || right.ends_with(&format!(".{left}"))
}

pub(super) fn virtual_text(source: &str, range: TextRange) -> Option<&str> {
    source.get(
        usize::try_from(range.start().to_u32()).ok()?
            ..usize::try_from(range.end().to_u32()).ok()?,
    )
}

pub(super) fn ranges_overlap(left: TextRange, right: TextRange) -> bool {
    if left.is_empty() {
        return right.start() <= left.start() && left.start() <= right.end();
    }
    if right.is_empty() {
        return left.start() <= right.start() && right.start() <= left.end();
    }
    left.start() < right.end() && right.start() < left.end()
}

pub(super) fn inferred_symbol(
    token: &SqlEditorToken,
    tokens: &[SqlEditorToken],
) -> SqlEditorSymbol {
    let index = tokens
        .iter()
        .position(|candidate| candidate.virtual_range == token.virtual_range)
        .unwrap_or_default();
    let previous = index
        .checked_sub(1)
        .and_then(|previous| tokens.get(previous))
        .map(|token| token.text.to_ascii_uppercase());
    let kind = if previous
        .as_deref()
        .is_some_and(|keyword| matches!(keyword, "FROM" | "JOIN" | "UPDATE" | "INTO"))
    {
        "relation"
    } else if tokens.get(index + 1).is_some_and(|next| next.text == "(") {
        "function"
    } else {
        "column"
    };
    SqlEditorSymbol {
        name: token.text.clone(),
        kind: kind.to_string(),
        database_type: None,
        sifr_type: None,
        nullable: None,
        definition_document: None,
        definition_range: None,
    }
}

pub(super) fn contains(range: TextRange, offset: TextSize) -> bool {
    if range.is_empty() {
        range.start() == offset
    } else {
        range.start() <= offset && offset < range.end()
    }
}

pub(super) fn schema_kind_label(kind: SchemaObjectKind) -> &'static str {
    match kind {
        SchemaObjectKind::Table | SchemaObjectKind::View | SchemaObjectKind::MaterializedView => {
            "relation"
        }
        SchemaObjectKind::Column => "column",
        SchemaObjectKind::Function => "function",
        SchemaObjectKind::Operator => "operator",
        SchemaObjectKind::Enum
        | SchemaObjectKind::Domain
        | SchemaObjectKind::Composite
        | SchemaObjectKind::Array
        | SchemaObjectKind::Range => "type",
        _ => "schema-object",
    }
}

pub(super) fn semantic_text(
    semantic: &BTreeMap<String, SemanticValue>,
    name: &str,
) -> Option<String> {
    match semantic.get(name) {
        Some(SemanticValue::Text(value)) => Some(value.clone()),
        _ => None,
    }
}

pub(super) fn semantic_display(
    semantic: &BTreeMap<String, SemanticValue>,
    name: &str,
) -> Option<String> {
    semantic.get(name).map(|value| match value {
        SemanticValue::Text(value) => value.clone(),
        other => format!("{other:?}"),
    })
}

pub(super) fn cardinality_label(cardinality: Cardinality) -> String {
    match cardinality {
        Cardinality::Empty => "empty".to_string(),
        Cardinality::Interval {
            minimum: 0,
            maximum: Some(1),
        } => "zero-or-one".to_string(),
        Cardinality::Interval {
            minimum: 1,
            maximum: Some(1),
        } => "exactly-one".to_string(),
        Cardinality::Interval {
            minimum: 1,
            maximum: None,
        } => "one-or-more".to_string(),
        Cardinality::Interval { minimum, maximum } => format!(
            "{minimum}..{}",
            maximum.map_or_else(|| "many".to_string(), |value| value.to_string())
        ),
    }
}

pub(super) fn semantic_bool(
    semantic: &BTreeMap<String, SemanticValue>,
    name: &str,
) -> Option<bool> {
    match semantic.get(name) {
        Some(SemanticValue::Bool(value)) => Some(*value),
        _ => None,
    }
}
