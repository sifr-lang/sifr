use super::*;
pub(crate) fn failure_group(reason: &str) -> &'static str {
    if reason.contains("sifr compilation failed") {
        "compile"
    } else if reason.contains("failed to generate grouped crate source") {
        "planning"
    } else if reason.contains("Rust compilation failed")
        || reason.contains("build log:")
        || reason.contains("missing batch artifact")
    {
        "build"
    } else if reason.contains("stdout mismatch") || reason.contains("binary exited with error") {
        "run"
    } else {
        "other"
    }
}

pub(crate) fn indent_multiline(text: &str, indent: &str) -> String {
    text.lines()
        .map(|line| format!("{indent}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn format_failures(kind: &str, cases: &[FixtureExecution]) -> String {
    let mut failures = cases
        .iter()
        .filter_map(|case| {
            case.status.as_ref().err().map(|reason| {
                (
                    case.name.clone(),
                    failure_group(reason).to_string(),
                    reason.clone(),
                )
            })
        })
        .collect::<Vec<_>>();

    if failures.is_empty() {
        return String::new();
    }

    failures.sort_by(|left, right| {
        left.1
            .cmp(&right.1)
            .then_with(|| left.0.cmp(&right.0))
            .then_with(|| left.2.cmp(&right.2))
    });

    let mut grouped: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for (name, group, reason) in failures {
        grouped.entry(group).or_default().push((name, reason));
    }

    let passed = cases.iter().filter(|case| case.status.is_ok()).count();
    let failed = cases.len().saturating_sub(passed);
    let mut sections = Vec::new();
    for (group, entries) in grouped {
        let mut rows = Vec::new();
        for (name, reason) in entries {
            rows.push(format!("- [{}]\n{}", name, indent_multiline(&reason, "  ")));
        }
        sections.push(format!(
            "[{group}] {} failure(s)\n{}",
            rows.len(),
            rows.join("\n")
        ));
    }

    format!(
        "{kind} E2E pass failures ({passed} passed, {failed} failed)\n\n{}",
        sections.join("\n\n")
    )
}

pub(crate) fn report_signature(kind: &str, report: &PassReport) -> String {
    let summary = report_summary(kind, report);
    deterministic_hash(&format!(
        "{kind}|{}|{}|{}",
        report.cases.len(),
        report.passed_count(),
        summary
    ))
}

pub(crate) fn assert_report(label: &str, report: &PassReport) {
    let summary = report_summary(label, report);
    assert!(summary.is_empty(), "{}", summary);
}

fn report_summary(label: &str, report: &PassReport) -> String {
    let mut summary = format_failures(label, &report.cases);
    let mut batches = report.batch_failures.clone();
    batches.sort();
    if !batches.is_empty() {
        summary.push_str(&format!(
            "\nOriginal native batch failures ({} unique fixtures passed, {} failed):\n{}",
            report.passed_count(),
            report.failed_count(),
            batches.join("\n")
        ));
    }
    summary
}
