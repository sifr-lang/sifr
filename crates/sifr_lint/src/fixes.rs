use crate::{FixAvailability, LintOptions, rule_metadata};
use sifr_diagnostics::render::RenderedDiagnosticSuggestion;
use sifr_diagnostics::{DiagnosticArg, RenderedDiagnostic, SuggestionApplicability};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnsafeFixPolicy {
    Disabled,
    Hint,
    Enabled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixOptions {
    pub fixable: Vec<String>,
    pub extend_fixable: Vec<String>,
    pub unfixable: Vec<String>,
    pub extend_unfixable: Vec<String>,
    pub unsafe_fixes: UnsafeFixPolicy,
}

impl Default for FixOptions {
    fn default() -> Self {
        Self {
            fixable: Vec::new(),
            extend_fixable: Vec::new(),
            unfixable: Vec::new(),
            extend_unfixable: Vec::new(),
            unsafe_fixes: UnsafeFixPolicy::Hint,
        }
    }
}

impl From<&LintOptions> for FixOptions {
    fn from(options: &LintOptions) -> Self {
        Self {
            fixable: options.fixable.clone(),
            extend_fixable: options.extend_fixable.clone(),
            unfixable: options.unfixable.clone(),
            extend_unfixable: options.extend_unfixable.clone(),
            unsafe_fixes: options.unsafe_fixes,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceEdit {
    pub byte_start: u32,
    pub byte_end: u32,
    pub replacement: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LintFix {
    pub rule_id: String,
    pub message: String,
    pub applicability: SuggestionApplicability,
    pub edits: Vec<SourceEdit>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FixedSource {
    pub fixed_source: String,
    pub diagnostics: Vec<RenderedDiagnostic>,
    pub remaining_diagnostics: Vec<RenderedDiagnostic>,
    pub applied_fixes: Vec<LintFix>,
    pub skipped_conflicting_fixes: usize,
}

pub fn fix_source(
    source: &str,
    file: Option<&std::path::Path>,
    options: &LintOptions,
) -> FixedSource {
    let diagnostics = crate::lint_source(source, file, options).diagnostics;
    let fixes = collect_fixes(&diagnostics, &FixOptions::from(options));
    let application = apply_fixes(source, fixes);
    let remaining_diagnostics = crate::lint_source(&application.fixed_source, file, options)
        .diagnostics
        .into_iter()
        .filter(|diagnostic| {
            diagnostic
                .args
                .get("rule")
                .and_then(string_arg)
                .is_none_or(|rule| !fix_rule_allowed(rule, options))
        })
        .collect();
    FixedSource {
        fixed_source: application.fixed_source,
        diagnostics,
        remaining_diagnostics,
        applied_fixes: application.applied_fixes,
        skipped_conflicting_fixes: application.skipped_conflicting_fixes,
    }
}

pub fn collect_fixes(diagnostics: &[RenderedDiagnostic], options: &FixOptions) -> Vec<LintFix> {
    let mut fixes = Vec::new();
    for diagnostic in diagnostics {
        let Some(rule_id) = diagnostic.args.get("rule").and_then(string_arg) else {
            continue;
        };
        if !fix_rule_allowed_by_options(rule_id, options) {
            continue;
        }
        for suggestion in &diagnostic.suggestions {
            if suggestion_allowed(suggestion, options.unsafe_fixes) {
                fixes.push(LintFix {
                    rule_id: rule_id.to_string(),
                    message: suggestion.message.clone(),
                    applicability: suggestion.applicability,
                    edits: suggestion
                        .edits
                        .iter()
                        .map(|edit| SourceEdit {
                            byte_start: edit.span.byte_start,
                            byte_end: edit.span.byte_end,
                            replacement: edit.replacement.clone(),
                        })
                        .collect(),
                });
            }
        }
    }
    fixes
}

/// Apply fixes that were collected from an existing lint result.
///
/// Analysis clients use this entrypoint to preserve the canonical frontend HIR
/// that produced the diagnostics instead of starting another frontend run.
pub fn apply_collected_fixes(source: &str, fixes: Vec<LintFix>) -> String {
    apply_fixes(source, fixes).fixed_source
}

pub fn fix_rule_allowed(rule_id: &str, options: &LintOptions) -> bool {
    fix_rule_allowed_by_options(rule_id, &FixOptions::from(options))
}

fn fix_rule_allowed_by_options(rule_id: &str, options: &FixOptions) -> bool {
    let Some(metadata) = rule_metadata(rule_id) else {
        return false;
    };
    if metadata.fix_availability == FixAvailability::None {
        return false;
    }
    if options
        .unfixable
        .iter()
        .chain(options.extend_unfixable.iter())
        .any(|selector| crate::rule_selector_matches(selector, metadata))
    {
        return false;
    }
    let has_allow_list = !options.fixable.is_empty() || !options.extend_fixable.is_empty();
    !has_allow_list
        || options
            .fixable
            .iter()
            .chain(options.extend_fixable.iter())
            .any(|selector| crate::rule_selector_matches(selector, metadata))
}

fn suggestion_allowed(
    suggestion: &RenderedDiagnosticSuggestion,
    unsafe_policy: UnsafeFixPolicy,
) -> bool {
    match suggestion.applicability {
        SuggestionApplicability::MachineApplicable => true,
        SuggestionApplicability::MaybeIncorrect => unsafe_policy == UnsafeFixPolicy::Enabled,
        SuggestionApplicability::HasPlaceholders | SuggestionApplicability::Unspecified => false,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FixApplication {
    fixed_source: String,
    applied_fixes: Vec<LintFix>,
    skipped_conflicting_fixes: usize,
}

fn apply_fixes(source: &str, fixes: Vec<LintFix>) -> FixApplication {
    let mut ordered = fixes;
    ordered.sort_by_key(|fix| {
        let first = fix
            .edits
            .iter()
            .map(|edit| (edit.byte_start, edit.byte_end))
            .min()
            .unwrap_or((u32::MAX, u32::MAX));
        (first.0, first.1, fix.rule_id.clone())
    });

    let mut accepted_ranges: Vec<(u32, u32)> = Vec::new();
    let mut accepted = Vec::new();
    let mut skipped_conflicting_fixes = 0usize;

    for fix in ordered {
        if fix.edits.iter().any(|edit| {
            invalid_range(source, edit)
                || accepted_ranges
                    .iter()
                    .any(|range| ranges_overlap((edit.byte_start, edit.byte_end), *range))
        }) {
            skipped_conflicting_fixes = skipped_conflicting_fixes.saturating_add(1);
            continue;
        }
        accepted_ranges.extend(
            fix.edits
                .iter()
                .map(|edit| (edit.byte_start, edit.byte_end)),
        );
        accepted.push(fix);
    }

    let mut edits = accepted
        .iter()
        .flat_map(|fix| fix.edits.iter())
        .collect::<Vec<_>>();
    edits.sort_by_key(|edit| std::cmp::Reverse((edit.byte_start, edit.byte_end)));

    let mut fixed_source = source.to_string();
    for edit in edits {
        let start = usize::try_from(edit.byte_start).unwrap_or(usize::MAX);
        let end = usize::try_from(edit.byte_end).unwrap_or(usize::MAX);
        fixed_source.replace_range(start..end, &edit.replacement);
    }

    FixApplication {
        fixed_source,
        applied_fixes: accepted,
        skipped_conflicting_fixes,
    }
}

fn invalid_range(source: &str, edit: &SourceEdit) -> bool {
    let Ok(start) = usize::try_from(edit.byte_start) else {
        return true;
    };
    let Ok(end) = usize::try_from(edit.byte_end) else {
        return true;
    };
    start > end
        || end > source.len()
        || !source.is_char_boundary(start)
        || !source.is_char_boundary(end)
}

fn ranges_overlap(left: (u32, u32), right: (u32, u32)) -> bool {
    left.0 < right.1 && right.0 < left.1
}

fn string_arg(arg: &DiagnosticArg) -> Option<&str> {
    match arg {
        DiagnosticArg::String(value) => Some(value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trailing_whitespace_fix_is_idempotent() {
        let options = LintOptions::default();
        let first = fix_source("def main():  \n    return 1\n", None, &options);
        assert_eq!(first.applied_fixes.len(), 1);
        assert_eq!(first.fixed_source, "def main():\n    return 1\n");
        assert!(first.remaining_diagnostics.is_empty());

        let second = fix_source(&first.fixed_source, None, &options);
        assert!(second.applied_fixes.is_empty());
        assert_eq!(second.fixed_source, first.fixed_source);
    }

    #[test]
    fn fix_conflicts_skip_later_overlapping_groups() {
        let fixes = vec![
            LintFix {
                rule_id: "a".to_string(),
                message: "first".to_string(),
                applicability: SuggestionApplicability::MachineApplicable,
                edits: vec![SourceEdit {
                    byte_start: 0,
                    byte_end: 2,
                    replacement: "x".to_string(),
                }],
            },
            LintFix {
                rule_id: "b".to_string(),
                message: "second".to_string(),
                applicability: SuggestionApplicability::MachineApplicable,
                edits: vec![SourceEdit {
                    byte_start: 1,
                    byte_end: 3,
                    replacement: "y".to_string(),
                }],
            },
        ];
        let fixed = apply_fixes("abcd", fixes);
        assert_eq!(fixed.fixed_source, "xcd");
        assert_eq!(fixed.applied_fixes.len(), 1);
        assert_eq!(fixed.skipped_conflicting_fixes, 1);
    }
}
