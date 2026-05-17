#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionCandidate {
    pub label: String,
    pub kind: String,
    pub detail: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::{evaluate_completion_ranking, rank_completion_candidates, CompletionCandidate};

    fn candidate(label: &str) -> CompletionCandidate {
        CompletionCandidate {
            label: label.to_string(),
            kind: "function".to_string(),
            detail: None,
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
}
