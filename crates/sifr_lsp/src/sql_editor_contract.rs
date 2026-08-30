#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SqlEditorPerformanceBudget {
    pub id: &'static str,
    pub maximum_ms: u64,
}

pub(crate) const SQL_EDITOR_PERFORMANCE_BUDGETS: [SqlEditorPerformanceBudget; 5] = [
    SqlEditorPerformanceBudget {
        id: "perf.lsp.sql.completion",
        maximum_ms: 200,
    },
    SqlEditorPerformanceBudget {
        id: "perf.lsp.sql.hover",
        maximum_ms: 100,
    },
    SqlEditorPerformanceBudget {
        id: "perf.lsp.sql.navigation",
        maximum_ms: 500,
    },
    SqlEditorPerformanceBudget {
        id: "perf.lsp.sql.diagnostics",
        maximum_ms: 250,
    },
    SqlEditorPerformanceBudget {
        id: "perf.lsp.sql.format",
        maximum_ms: 500,
    },
];

pub(crate) fn budget_for_method(method: &str) -> Option<SqlEditorPerformanceBudget> {
    let suffix = match method {
        "textDocument/completion" => "completion",
        "textDocument/hover" => "hover",
        "textDocument/definition"
        | "textDocument/declaration"
        | "textDocument/typeDefinition"
        | "textDocument/references"
        | "textDocument/prepareRename"
        | "textDocument/rename" => "navigation",
        "textDocument/diagnostic" | "workspace/diagnostic" => "diagnostics",
        "textDocument/formatting" | "textDocument/rangeFormatting" => "format",
        _ => return None,
    };
    SQL_EDITOR_PERFORMANCE_BUDGETS
        .iter()
        .find(|budget| budget.id.ends_with(suffix))
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn sql_editor_budgets_have_stable_unique_names() {
        let names = SQL_EDITOR_PERFORMANCE_BUDGETS
            .iter()
            .map(|budget| budget.id)
            .collect::<BTreeSet<_>>();
        assert_eq!(names.len(), SQL_EDITOR_PERFORMANCE_BUDGETS.len());
        assert!(
            SQL_EDITOR_PERFORMANCE_BUDGETS
                .iter()
                .all(|budget| budget.id.starts_with("perf.lsp.sql.") && budget.maximum_ms > 0)
        );
    }
}
