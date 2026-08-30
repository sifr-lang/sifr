use crate::analysis::{PostgresAnalysisError, ScopeFrame};
use crate::ast::{FromItem, JoinKind, SelectStatement};
use crate::diagnostic::PostgresDiagnosticCode;
use crate::semantic_helpers::expression_has_window;
use std::collections::BTreeSet;

pub(crate) fn validate_locking(
    select: &SelectStatement,
    frames: &[ScopeFrame],
    aggregate: bool,
) -> Result<(), PostgresAnalysisError> {
    if aggregate
        || !select.group_by.is_empty()
        || select.having.is_some()
        || !select.windows.is_empty()
        || select
            .targets
            .iter()
            .any(|target| expression_has_window(&target.expression))
    {
        return Err(PostgresAnalysisError::at_start(
            PostgresDiagnosticCode::InvalidResult,
            "PostgreSQL row locking cannot apply to grouped, aggregate, or window results",
        ));
    }
    let available = frames
        .last()
        .into_iter()
        .flat_map(|frame| &frame.bindings)
        .map(|binding| binding.alias.clone())
        .collect::<BTreeSet<_>>();
    let nullable = select
        .from
        .iter()
        .flat_map(nullable_join_aliases)
        .collect::<BTreeSet<_>>();
    for lock in &select.locking {
        if lock.relations.is_empty() && !nullable.is_empty() {
            return Err(PostgresAnalysisError::at_start(
                PostgresDiagnosticCode::InvalidResult,
                "row locking across an outer join needs an OF list for non-nullable relations",
            ));
        }
        for relation in &lock.relations {
            let alias = relation.rsplit('.').next().unwrap_or(relation);
            if !available.contains(alias) {
                return Err(PostgresAnalysisError::at_start(
                    PostgresDiagnosticCode::UnknownRelation,
                    format!("row locking names unknown relation '{relation}'"),
                ));
            }
            if nullable.contains(alias) {
                return Err(PostgresAnalysisError::at_start(
                    PostgresDiagnosticCode::InvalidResult,
                    format!("row locking cannot target nullable outer-join relation '{relation}'"),
                ));
            }
        }
    }
    Ok(())
}

fn nullable_join_aliases(item: &FromItem) -> Vec<String> {
    let FromItem::Join {
        join, left, right, ..
    } = item
    else {
        return Vec::new();
    };
    let mut aliases = nullable_join_aliases(left);
    aliases.extend(nullable_join_aliases(right));
    if matches!(join, JoinKind::Right | JoinKind::Full) {
        aliases.extend(relation_aliases(left));
    }
    if matches!(join, JoinKind::Left | JoinKind::Full) {
        aliases.extend(relation_aliases(right));
    }
    aliases
}

fn relation_aliases(item: &FromItem) -> Vec<String> {
    match item {
        FromItem::Relation { name, alias, .. } => alias
            .clone()
            .or_else(|| name.last().cloned())
            .into_iter()
            .collect(),
        FromItem::Subquery { alias, .. } => vec![alias.clone()],
        FromItem::Join { left, right, .. } => {
            let mut aliases = relation_aliases(left);
            aliases.extend(relation_aliases(right));
            aliases
        }
    }
}
