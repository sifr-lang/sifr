use crate::queries::{FoldingRange, InlayHint, SelectionRange, SemanticToken};
use ruff_text_size::{TextRange, TextSize};
use sifr_syntax::{SourceText as SyntaxSourceText, TextPosition};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EditorToken {
    pub kind: String,
    pub text: String,
    pub range: TextRange,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EditorFacts {
    pub source: String,
    pub tokens: Vec<EditorToken>,
}

impl EditorFacts {
    #[must_use]
    pub(crate) fn token_at_position(&self, position: &TextPosition) -> Option<&EditorToken> {
        let offset = SyntaxSourceText::new(self.source.clone()).byte_offset(position)?;
        self.tokens
            .iter()
            .filter(|token| token.range.start() <= offset && offset <= token.range.end())
            .min_by_key(|token| token.range.len())
    }

    #[must_use]
    pub(crate) fn identifier_at_position(&self, position: &TextPosition) -> Option<&EditorToken> {
        self.token_at_position(position)
            .filter(|token| is_identifier_token(token))
    }

    #[must_use]
    pub(crate) fn tokens_named(&self, name: &str) -> Vec<EditorToken> {
        self.tokens
            .iter()
            .filter(|token| token.text == name && is_identifier_token(token))
            .cloned()
            .collect()
    }

    #[must_use]
    pub(crate) fn semantic_tokens(&self, range: Option<TextRange>) -> Vec<SemanticToken> {
        self.tokens
            .iter()
            .filter(|token| range.is_none_or(|requested| requested.contains_range(token.range)))
            .filter_map(|token| {
                Some(SemanticToken {
                    range: token.range,
                    token_type: semantic_token_kind(token)?,
                    modifiers: semantic_token_modifiers(token),
                })
            })
            .collect()
    }

    #[must_use]
    pub(crate) fn folding_ranges(&self) -> Vec<FoldingRange> {
        let line_ranges = line_ranges(&self.source);
        let lines = self.source.lines().collect::<Vec<_>>();
        let mut ranges = Vec::new();
        for (line_index, line) in lines.iter().enumerate() {
            if !line.trim_end().ends_with(':') {
                continue;
            }
            let base_indent = indent_width(line);
            let mut end_line = line_index;
            for (candidate_index, candidate) in lines.iter().enumerate().skip(line_index + 1) {
                if candidate.trim().is_empty() {
                    end_line = candidate_index;
                    continue;
                }
                if indent_width(candidate) <= base_indent {
                    break;
                }
                end_line = candidate_index;
            }
            if end_line > line_index {
                ranges.push(FoldingRange {
                    range: TextRange::new(
                        line_ranges[line_index].start(),
                        line_ranges[end_line].end(),
                    ),
                });
            }
        }
        ranges
    }

    #[must_use]
    pub(crate) fn selection_ranges(&self, positions: &[TextPosition]) -> Vec<SelectionRange> {
        let source = SyntaxSourceText::new(self.source.clone());
        let full_range = full_range(&self.source).unwrap_or_else(empty_range);
        positions
            .iter()
            .filter_map(|position| {
                let token = self.token_at_position(position)?;
                let offset = source.byte_offset(position)?;
                let line = line_range_for_offset(&self.source, offset)?;
                Some(SelectionRange {
                    range: token.range,
                    parent: Some(Box::new(SelectionRange {
                        range: line,
                        parent: Some(Box::new(SelectionRange {
                            range: full_range,
                            parent: None,
                        })),
                    })),
                })
            })
            .collect()
    }

    #[must_use]
    pub(crate) fn inlay_hints(&self, range: Option<TextRange>) -> Vec<InlayHint> {
        let source = SyntaxSourceText::new(self.source.clone());
        let mut hints = Vec::new();
        for token in &self.tokens {
            if !is_identifier_token(token)
                || !range.is_none_or(|requested| requested.contains_range(token.range))
            {
                continue;
            }
            let after = usize::try_from(token.range.end().to_u32()).ok();
            let Some(after) = after else {
                continue;
            };
            let line_end = self.source[after..]
                .find('\n')
                .map_or(self.source.len(), |offset| after + offset);
            let suffix = &self.source[after..line_end];
            let Some(annotation) = suffix.strip_prefix(": ") else {
                continue;
            };
            let annotation = annotation
                .split(['=', ',', ')'])
                .next()
                .unwrap_or_default()
                .trim();
            if annotation.is_empty() {
                continue;
            }
            if let Some(position) = source.text_position(token.range.end()) {
                hints.push(InlayHint {
                    position,
                    label: format!(": {annotation}"),
                });
            }
        }
        hints
    }
}

#[must_use]
pub(crate) fn is_identifier_token(token: &EditorToken) -> bool {
    is_identifier_text(&token.text)
        && (token.kind.contains("Name")
            || token.kind.contains("Identifier")
            || token.kind.contains("NonLogicalNewline"))
}

#[must_use]
pub(crate) fn is_identifier_text(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        && !is_keyword(text)
}

#[must_use]
pub(crate) fn full_range(source: &str) -> Option<TextRange> {
    Some(TextRange::new(
        TextSize::new(0),
        TextSize::new(u32::try_from(source.len()).ok()?),
    ))
}

#[must_use]
pub(crate) fn line_end_insert_range(source: &str, range: TextRange) -> Option<TextRange> {
    let end = usize::try_from(range.end().to_u32()).ok()?;
    let line_end = source[end..]
        .find('\n')
        .map_or(source.len(), |offset| end + offset);
    let line_end = TextSize::new(u32::try_from(line_end).ok()?);
    Some(TextRange::new(line_end, line_end))
}

fn semantic_token_kind(token: &EditorToken) -> Option<String> {
    if is_keyword(&token.text) {
        return Some("keyword".to_string());
    }
    if token.kind.contains("String") {
        return Some("string".to_string());
    }
    if token.kind.contains("Number") || token.text.chars().all(|ch| ch.is_ascii_digit()) {
        return Some("number".to_string());
    }
    if is_identifier_token(token) {
        return Some(identifier_semantic_kind(&token.text));
    }
    if token.text.chars().any(|ch| "+-*/%=<>!&|.^~".contains(ch)) {
        return Some("operator".to_string());
    }
    None
}

fn semantic_token_modifiers(token: &EditorToken) -> Vec<String> {
    if token.text.starts_with("mut_") || token.text == "mut" {
        vec!["mutable".to_string()]
    } else {
        Vec::new()
    }
}

fn identifier_semantic_kind(text: &str) -> String {
    if text.chars().next().is_some_and(char::is_uppercase) {
        "type".to_string()
    } else {
        "variable".to_string()
    }
}

fn line_ranges(source: &str) -> Vec<TextRange> {
    let mut ranges = Vec::new();
    let mut start = 0usize;
    for line in source.split_inclusive('\n') {
        let end = start + line.len();
        if let (Ok(start), Ok(end)) = (u32::try_from(start), u32::try_from(end)) {
            ranges.push(TextRange::new(TextSize::new(start), TextSize::new(end)));
        }
        start = end;
    }
    if source.is_empty() || !source.ends_with('\n') {
        if let (Ok(start), Ok(offset)) = (u32::try_from(start), u32::try_from(source.len())) {
            ranges.push(TextRange::new(TextSize::new(start), TextSize::new(offset)));
        }
    }
    ranges
}

fn line_range_for_offset(source: &str, offset: TextSize) -> Option<TextRange> {
    line_ranges(source)
        .into_iter()
        .find(|range| range.start() <= offset && offset <= range.end())
}

fn indent_width(line: &str) -> usize {
    line.chars()
        .take_while(|ch| *ch == ' ' || *ch == '\t')
        .count()
}

fn empty_range() -> TextRange {
    TextRange::new(TextSize::new(0), TextSize::new(0))
}

fn is_keyword(text: &str) -> bool {
    matches!(
        text,
        "and"
            | "as"
            | "async"
            | "await"
            | "break"
            | "class"
            | "continue"
            | "def"
            | "elif"
            | "else"
            | "enum"
            | "except"
            | "False"
            | "finally"
            | "for"
            | "from"
            | "if"
            | "import"
            | "in"
            | "is"
            | "match"
            | "mut"
            | "None"
            | "not"
            | "or"
            | "own"
            | "pass"
            | "raise"
            | "return"
            | "True"
            | "try"
            | "while"
            | "with"
            | "yield"
    )
}
