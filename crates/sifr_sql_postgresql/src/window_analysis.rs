use crate::analysis::{AnalysisContext, PostgresAnalysisError, ScopeFrame};
use crate::ast::{Expression, SelectStatement};
use crate::diagnostic::PostgresDiagnosticCode;
use crate::semantic_helpers::{expression_has_window, window_references};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn validate_named_windows(
    context: &mut AnalysisContext<'_>,
    select: &SelectStatement,
    frames: &[ScopeFrame],
) -> Result<BTreeSet<String>, PostgresAnalysisError> {
    let mut references = BTreeMap::<String, Option<String>>::new();
    for window in &select.windows {
        if references
            .insert(window.name.clone(), window.specification.reference.clone())
            .is_some()
        {
            return Err(PostgresAnalysisError::at_start(
                PostgresDiagnosticCode::InvalidResult,
                format!("named window '{}' is duplicated", window.name),
            ));
        }
        for expression in &window.specification.partition_by {
            reject_window_expression(expression, "WINDOW")?;
            context.infer(expression, frames, None)?;
        }
        for order in &window.specification.order_by {
            reject_window_expression(&order.expression, "WINDOW")?;
            context.infer(&order.expression, frames, None)?;
        }
        for offset in [
            window.specification.start_offset.as_deref(),
            window.specification.end_offset.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            reject_window_expression(offset, "WINDOW")?;
            context.infer(offset, frames, None)?;
        }
    }
    for name in references.keys() {
        let mut current = Some(name.as_str());
        let mut path = BTreeSet::new();
        while let Some(value) = current {
            if !path.insert(value.to_string()) {
                return Err(PostgresAnalysisError::at_start(
                    PostgresDiagnosticCode::InvalidResult,
                    format!("named window '{name}' has a cyclic reference"),
                ));
            }
            current = references
                .get(value)
                .ok_or_else(|| {
                    PostgresAnalysisError::at_start(
                        PostgresDiagnosticCode::InvalidResult,
                        format!("named window '{value}' does not exist"),
                    )
                })?
                .as_deref();
        }
    }
    Ok(references.into_keys().collect())
}

pub(crate) fn reject_window_expression(
    expression: &Expression,
    clause: &str,
) -> Result<(), PostgresAnalysisError> {
    if expression_has_window(expression) {
        return Err(PostgresAnalysisError::new(
            PostgresDiagnosticCode::InvalidResult,
            format!("a PostgreSQL window function is not valid in {clause}"),
            expression,
        ));
    }
    Ok(())
}

pub(crate) fn validate_window_references(
    expression: &Expression,
    names: &BTreeSet<String>,
) -> Result<(), PostgresAnalysisError> {
    let mut references = Vec::new();
    window_references(expression, &mut references);
    if let Some(reference) = references
        .into_iter()
        .find(|reference| !names.contains(reference))
    {
        return Err(PostgresAnalysisError::new(
            PostgresDiagnosticCode::InvalidResult,
            format!("named window '{reference}' does not exist"),
            expression,
        ));
    }
    Ok(())
}
