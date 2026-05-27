use crate::SuppressionComplexity;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

impl LineRange {
    fn contains(self, line: usize) -> bool {
        self.start <= line && line <= self.end
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuppressionDirective {
    pub line: usize,
    pub rules: Vec<String>,
    used_rules: Vec<String>,
    attached_range: LineRange,
}

impl SuppressionDirective {
    pub fn is_used_for(&self, rule: &str) -> bool {
        self.used_rules.iter().any(|used| used == rule)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParserAwareSuppressions {
    directives: Vec<SuppressionDirective>,
}

impl ParserAwareSuppressions {
    pub fn new(source: &str, ignore_suppressions: bool) -> Self {
        if ignore_suppressions {
            return Self {
                directives: Vec::new(),
            };
        }
        let statement_ranges = statement_ranges(source);
        let directives = parse_directives(source)
            .into_iter()
            .map(|mut directive| {
                directive.attached_range = statement_ranges
                    .iter()
                    .copied()
                    .find(|range| range.contains(directive.line))
                    .unwrap_or(LineRange {
                        start: directive.line,
                        end: directive.line,
                    });
                directive
            })
            .collect();
        Self { directives }
    }

    pub fn directives(&self) -> &[SuppressionDirective] {
        &self.directives
    }

    pub fn mark_suppressed(
        &mut self,
        diagnostic_line: usize,
        rule: &str,
        complexity: SuppressionComplexity,
    ) -> bool {
        let Some(directive) = self
            .directives
            .iter_mut()
            .find(|directive| directive_applies(directive, diagnostic_line, rule, complexity))
        else {
            return false;
        };
        if !directive.used_rules.iter().any(|used| used == rule) {
            directive.used_rules.push(rule.to_string());
        }
        true
    }
}

fn directive_applies(
    directive: &SuppressionDirective,
    diagnostic_line: usize,
    rule: &str,
    complexity: SuppressionComplexity,
) -> bool {
    if !directive.rules.iter().any(|candidate| candidate == rule) {
        return false;
    }
    match complexity {
        SuppressionComplexity::PhysicalLine => directive.line == diagnostic_line,
        SuppressionComplexity::SingleNode
        | SuppressionComplexity::StatementRange
        | SuppressionComplexity::SymbolWorkspace => {
            directive.attached_range.contains(diagnostic_line)
        }
    }
}

fn parse_directives(source: &str) -> Vec<SuppressionDirective> {
    let mut suppressions = Vec::new();
    for (line_index, line) in source.lines().enumerate() {
        let Some(comment_start) = line.find('#') else {
            continue;
        };
        let comment = &line[comment_start..];
        let Some(ignore_start) = comment.find("sifr: ignore") else {
            continue;
        };
        let suffix = &comment[ignore_start + "sifr: ignore".len()..];
        let rules = if let Some(stripped) = suffix.strip_prefix('[') {
            stripped
                .split_once(']')
                .map(|(rule_list, _)| {
                    rule_list
                        .split(',')
                        .map(str::trim)
                        .filter(|rule| !rule.is_empty())
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        suppressions.push(SuppressionDirective {
            line: line_index,
            rules,
            used_rules: Vec::new(),
            attached_range: LineRange {
                start: line_index,
                end: line_index,
            },
        });
    }
    suppressions
}

fn statement_ranges(source: &str) -> Vec<LineRange> {
    let mut ranges = Vec::new();
    let mut start = 0usize;
    let mut depth = 0i32;
    let mut saw_code = false;
    for (line_index, line) in source.lines().enumerate() {
        let code = line.split_once('#').map_or(line, |(code, _)| code);
        if !code.trim().is_empty() {
            saw_code = true;
        }
        depth = update_depth(depth, code);
        if saw_code && depth == 0 && !code.trim_end().ends_with('\\') {
            ranges.push(LineRange {
                start,
                end: line_index,
            });
            start = line_index.saturating_add(1);
            saw_code = false;
        }
    }
    let total_lines = source.lines().count();
    if start < total_lines {
        ranges.push(LineRange {
            start,
            end: total_lines.saturating_sub(1),
        });
    }
    ranges
}

fn update_depth(mut depth: i32, code: &str) -> i32 {
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for character in code.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '(' | '[' | '{' => depth = depth.saturating_add(1),
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    depth.max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statement_range_suppression_attaches_to_multiline_construct() {
        let source = "value = call(\n    first,\n    second,  # sifr: ignore[demo-rule]\n)\n";
        let mut suppressions = ParserAwareSuppressions::new(source, false);
        assert!(suppressions.mark_suppressed(
            0,
            "demo-rule",
            SuppressionComplexity::StatementRange
        ));
    }

    #[test]
    fn physical_line_suppression_stays_line_local() {
        let source = "value = call(\n    first,  # sifr: ignore[demo-rule]\n)\n";
        let mut suppressions = ParserAwareSuppressions::new(source, false);
        assert!(!suppressions.mark_suppressed(0, "demo-rule", SuppressionComplexity::PhysicalLine));
        assert!(suppressions.mark_suppressed(1, "demo-rule", SuppressionComplexity::PhysicalLine));
    }
}
