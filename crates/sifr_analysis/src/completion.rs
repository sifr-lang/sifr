use sifr_frontend::FileId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionCandidate {
    pub label: String,
    pub kind: String,
    pub detail: Option<String>,
    pub symbol_file: Option<FileId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionRankingResult {
    pub candidates: Vec<CompletionCandidate>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionEvaluation {
    pub query: String,
    pub expected_top_label: String,
    pub actual_top_label: Option<String>,
    pub passed: bool,
}

pub(crate) fn rust_interop_completion_candidates(
    source: &str,
    position: &sifr_syntax::TextPosition,
) -> Vec<CompletionCandidate> {
    let source_text = sifr_syntax::SourceText::new(source.to_string());
    let Some(offset) = source_text.byte_offset(position) else {
        return Vec::new();
    };
    let Ok(offset) = usize::try_from(offset.to_u32()) else {
        return Vec::new();
    };
    let line_start = source[..offset].rfind('\n').map_or(0, |index| index + 1);
    let prefix = source[line_start..offset].trim_start();
    if !prefix.contains('(') && is_rust_interop_decorator_prefix(prefix) {
        return rust_interop_decorator_candidates();
    }

    let Some(context) = rust_interop_completion_context(source, offset) else {
        return Vec::new();
    };
    if !context.policy_keys_available {
        return Vec::new();
    }
    rust_interop_policy_key_candidates(context.decorator)
}

pub fn rank_completion_candidates(
    query: &str,
    mut candidates: Vec<CompletionCandidate>,
) -> CompletionRankingResult {
    candidates.sort_by(|left, right| {
        completion_score(query, right)
            .cmp(&completion_score(query, left))
            .then_with(|| left.label.cmp(&right.label))
            .then_with(|| left.kind.cmp(&right.kind))
    });
    CompletionRankingResult { candidates }
}

#[must_use]
pub fn evaluate_completion_ranking(
    query: &str,
    expected_top_label: &str,
    candidates: Vec<CompletionCandidate>,
) -> CompletionEvaluation {
    let ranked = rank_completion_candidates(query, candidates);
    let actual_top_label = ranked
        .candidates
        .first()
        .map(|candidate| candidate.label.clone());
    CompletionEvaluation {
        query: query.to_string(),
        expected_top_label: expected_top_label.to_string(),
        passed: actual_top_label.as_deref() == Some(expected_top_label),
        actual_top_label,
    }
}

fn completion_score(query: &str, candidate: &CompletionCandidate) -> u8 {
    if query.is_empty() {
        return 1;
    }
    if candidate.label == query {
        return 4;
    }
    if candidate.label.starts_with(query) {
        return 3;
    }
    if candidate.label.contains(query) {
        return 2;
    }
    0
}

fn rust_interop_decorator_candidates() -> Vec<CompletionCandidate> {
    [
        ("rust", "Rust interop decorator root"),
        ("rust.async", "Rust async bridge contract decorator"),
        ("rust.opaque", "Rust opaque handle contract decorator"),
        ("rust.zero_copy", "Rust zero-copy contract decorator"),
        ("rust.view", "Rust borrowed view contract decorator"),
        ("rust.callback", "Rust callback contract decorator"),
    ]
    .into_iter()
    .map(|(label, detail)| CompletionCandidate {
        label: label.to_string(),
        kind: "decorator".to_string(),
        detail: Some(detail.to_string()),
        symbol_file: None,
    })
    .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RustInteropCompletionDecorator {
    Function,
    Async,
    Opaque,
    ZeroCopy,
    View,
    Callback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RustInteropCompletionContext {
    decorator: RustInteropCompletionDecorator,
    policy_keys_available: bool,
}

fn is_rust_interop_decorator_prefix(prefix: &str) -> bool {
    let Some(name) = prefix.strip_prefix('@') else {
        return false;
    };
    [
        "rust",
        "rust.async",
        "rust.opaque",
        "rust.zero_copy",
        "rust.view",
        "rust.callback",
    ]
    .into_iter()
    .any(|candidate| candidate.starts_with(name))
}

fn rust_interop_completion_context(
    source: &str,
    offset: usize,
) -> Option<RustInteropCompletionContext> {
    let before_cursor = &source[..offset];
    let mut search_end = before_cursor.len();
    while let Some(index) = before_cursor[..search_end].rfind("@rust") {
        if !is_decorator_start_boundary(source, index) {
            search_end = index;
            continue;
        }
        let after_rust = &before_cursor[index + "@rust".len()..];
        let Some((decorator, after_decorator)) = parse_decorator_suffix(after_rust) else {
            search_end = index;
            continue;
        };
        let after_decorator = after_decorator.trim_start();
        let Some(arguments) = after_decorator.strip_prefix('(') else {
            search_end = index;
            continue;
        };
        let Some(policy_keys_available) = policy_keys_available_before_cursor(decorator, arguments)
        else {
            search_end = index;
            continue;
        };
        return Some(RustInteropCompletionContext {
            decorator,
            policy_keys_available,
        });
    }
    None
}

fn is_decorator_start_boundary(source: &str, index: usize) -> bool {
    if index == 0 {
        return true;
    }
    source[..index]
        .chars()
        .next_back()
        .is_none_or(char::is_whitespace)
}

fn parse_decorator_suffix(after_rust: &str) -> Option<(RustInteropCompletionDecorator, &str)> {
    [
        (".zero_copy", RustInteropCompletionDecorator::ZeroCopy),
        (".callback", RustInteropCompletionDecorator::Callback),
        (".opaque", RustInteropCompletionDecorator::Opaque),
        (".async", RustInteropCompletionDecorator::Async),
        (".view", RustInteropCompletionDecorator::View),
    ]
    .into_iter()
    .find_map(|(suffix, decorator)| {
        after_rust
            .strip_prefix(suffix)
            .map(|remaining| (decorator, remaining))
    })
    .or(Some((RustInteropCompletionDecorator::Function, after_rust)))
}

fn policy_keys_available_before_cursor(
    decorator: RustInteropCompletionDecorator,
    arguments: &str,
) -> Option<bool> {
    let mut depth = 1_i32;
    let mut saw_function_target_separator = false;
    for character in arguments.chars() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth <= 0 {
                    return None;
                }
            }
            ',' if depth == 1 => saw_function_target_separator = true,
            _ => {}
        }
    }
    Some(decorator != RustInteropCompletionDecorator::Function || saw_function_target_separator)
}

fn rust_interop_policy_key_candidates(
    decorator: RustInteropCompletionDecorator,
) -> Vec<CompletionCandidate> {
    match decorator {
        RustInteropCompletionDecorator::Function => &[("panic", "Rust panic boundary policy")][..],
        RustInteropCompletionDecorator::Async => &[(
            "thread_affinity",
            "Rust runtime or OS-thread affinity policy",
        )],
        RustInteropCompletionDecorator::Opaque => &[
            ("type", "Rust opaque handle target type"),
            ("send", "Rust Send bridge policy"),
            ("sync", "Rust Sync bridge policy"),
            ("clone", "Rust opaque handle clone policy"),
            ("close", "Rust opaque handle close policy"),
            ("borrow", "Rust opaque method receiver policy"),
            (
                "thread_affinity",
                "Rust runtime or OS-thread affinity policy",
            ),
        ],
        RustInteropCompletionDecorator::ZeroCopy => &[
            ("owner", "Rust zero-copy owner binding"),
            ("view", "Rust zero-copy view target type"),
        ],
        RustInteropCompletionDecorator::View => &[
            ("owner", "Rust borrowed view owner binding"),
            ("lifetime", "Rust borrowed view lifetime policy"),
            ("mutability", "Rust borrowed view mutability policy"),
            ("send", "Rust Send bridge policy"),
            ("sync", "Rust Sync bridge policy"),
            ("data", "Rust advanced data view kind"),
            ("schema", "Rust Arrow schema path"),
            ("dtype", "Rust tensor dtype"),
            ("rank", "Rust tensor rank"),
            ("shape", "Rust tensor shape"),
            ("layout", "Rust tensor layout"),
            ("strides", "Rust tensor strides"),
            ("device", "Rust tensor device"),
            ("ownership", "Rust advanced data ownership policy"),
            ("protocol", "Rust DLPack protocol path"),
        ],
        RustInteropCompletionDecorator::Callback => &[
            ("backpressure", "Rust callback backpressure policy"),
            ("overflow", "Rust callback overflow policy"),
            ("shutdown", "Rust callback shutdown policy"),
        ],
    }
    .iter()
    .map(|(label, detail)| CompletionCandidate {
        label: label.to_string(),
        kind: "property".to_string(),
        detail: Some(detail.to_string()),
        symbol_file: None,
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        CompletionCandidate, evaluate_completion_ranking, rank_completion_candidates,
        rust_interop_completion_candidates,
    };
    use sifr_syntax::TextPosition;

    fn candidate(label: &str) -> CompletionCandidate {
        CompletionCandidate {
            label: label.to_string(),
            kind: "function".to_string(),
            detail: None,
            symbol_file: None,
        }
    }

    #[test]
    fn completion_ranking_prefers_exact_then_prefix_then_substring() {
        let ranked = rank_completion_candidates(
            "map",
            vec![
                candidate("remap"),
                candidate("mapper"),
                candidate("map"),
                candidate("zip"),
            ],
        );

        let labels = ranked
            .candidates
            .into_iter()
            .map(|candidate| candidate.label)
            .collect::<Vec<_>>();

        assert_eq!(labels, vec!["map", "mapper", "remap", "zip"]);
    }

    #[test]
    fn completion_evaluation_records_top_candidate_quality() {
        let evaluation = evaluate_completion_ranking(
            "hel",
            "helper",
            vec![candidate("shell"), candidate("helper")],
        );

        assert!(evaluation.passed);
        assert_eq!(evaluation.actual_top_label.as_deref(), Some("helper"));
    }

    #[test]
    fn rust_interop_completion_suggests_decorator_paths_on_rust_decorator_line() {
        let source = "@rust.\ndef main():\n    return 1\n";
        let candidates = rust_interop_completion_candidates(
            source,
            &TextPosition {
                line: 0,
                character: 6,
            },
        );
        let labels = candidates
            .iter()
            .map(|candidate| candidate.label.as_str())
            .collect::<Vec<_>>();

        assert!(labels.contains(&"rust.async"));
        assert!(labels.contains(&"rust.opaque"));
        assert!(labels.contains(&"rust.zero_copy"));
        assert!(labels.contains(&"rust.view"));
        assert!(labels.contains(&"rust.callback"));
    }

    #[test]
    fn rust_interop_completion_suggests_policy_keys_inside_decorator_call() {
        let source = "@rust(callback_target, )\ndef main():\n    return 1\n";
        let candidates = rust_interop_completion_candidates(
            source,
            &TextPosition {
                line: 0,
                character: 22,
            },
        );
        let labels = candidates
            .iter()
            .map(|candidate| candidate.label.as_str())
            .collect::<Vec<_>>();

        assert_eq!(labels, vec!["panic"]);
    }

    #[test]
    fn rust_interop_completion_uses_callback_policy_keys_only() {
        let source = "@rust.callback(back)\ndef main():\n    return 1\n";
        let candidates = rust_interop_completion_candidates(
            source,
            &TextPosition {
                line: 0,
                character: 19,
            },
        );
        let labels = candidates
            .iter()
            .map(|candidate| candidate.label.as_str())
            .collect::<Vec<_>>();

        assert_eq!(labels, vec!["backpressure", "overflow", "shutdown"]);
        assert!(!labels.contains(&"lifetime"));
        assert!(!labels.contains(&"panic"));
    }

    #[test]
    fn rust_interop_completion_uses_opaque_policy_keys_only() {
        let source = "@rust.opaque(type=bridge.Token, )\nclass Token:\n    pass\n";
        let candidates = rust_interop_completion_candidates(
            source,
            &TextPosition {
                line: 0,
                character: 32,
            },
        );
        let labels = candidates
            .iter()
            .map(|candidate| candidate.label.as_str())
            .collect::<Vec<_>>();

        assert!(labels.contains(&"type"));
        assert!(labels.contains(&"thread_affinity"));
        assert!(!labels.contains(&"backpressure"));
        assert!(!labels.contains(&"owner"));
    }

    #[test]
    fn rust_interop_completion_supports_multiline_view_policy_keys() {
        let source = "@rust.view(\n    owner=input,\n    schema=\n)\ndef main():\n    return 1\n";
        let candidates = rust_interop_completion_candidates(
            source,
            &TextPosition {
                line: 2,
                character: 11,
            },
        );
        let labels = candidates
            .iter()
            .map(|candidate| candidate.label.as_str())
            .collect::<Vec<_>>();

        assert!(labels.contains(&"lifetime"));
        assert!(labels.contains(&"schema"));
        assert!(labels.contains(&"protocol"));
        assert!(!labels.contains(&"backpressure"));
    }

    #[test]
    fn rust_interop_completion_waits_for_function_target_separator() {
        let before_target_separator = "@rust()";
        assert!(
            rust_interop_completion_candidates(
                before_target_separator,
                &TextPosition {
                    line: 0,
                    character: 6,
                },
            )
            .is_empty()
        );

        let after_target_separator = "@rust(bridge.hash, )";
        let labels = rust_interop_completion_candidates(
            after_target_separator,
            &TextPosition {
                line: 0,
                character: 19,
            },
        )
        .into_iter()
        .map(|candidate| candidate.label)
        .collect::<Vec<_>>();

        assert_eq!(labels, vec!["panic"]);
    }

    #[test]
    fn rust_interop_completion_ignores_non_decorator_lines() {
        let source = "value = rust\n";
        let candidates = rust_interop_completion_candidates(
            source,
            &TextPosition {
                line: 0,
                character: 10,
            },
        );

        assert!(candidates.is_empty());
    }
}
