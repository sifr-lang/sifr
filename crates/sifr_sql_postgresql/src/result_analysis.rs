use crate::analysis::{
    AnalysisContext, PostgresAnalysisError, ResultFact, ScopeFrame, StarExpansion,
};
use crate::ast::{Expression, ExpressionKind, SelectItem};
use crate::diagnostic::PostgresDiagnosticCode;
use sifr_sql_contract::DatabaseType;

impl AnalysisContext<'_> {
    pub(crate) fn result_fields(
        &mut self,
        targets: &[SelectItem],
        frames: &[ScopeFrame],
    ) -> Result<Vec<ResultFact>, PostgresAnalysisError> {
        self.result_fields_with_expected(targets, frames, None)
    }

    pub(crate) fn result_fields_with_expected(
        &mut self,
        targets: &[SelectItem],
        frames: &[ScopeFrame],
        expected_fields: Option<&[DatabaseType]>,
    ) -> Result<Vec<ResultFact>, PostgresAnalysisError> {
        let star_count = targets
            .iter()
            .filter(|target| matches!(target.expression.kind, ExpressionKind::Star { .. }))
            .count();
        if star_count == 0 && expected_fields.is_some_and(|fields| fields.len() != targets.len()) {
            return Err(PostgresAnalysisError::at_start(
                PostgresDiagnosticCode::TypeMismatch,
                "SELECT result width does not match its assignment context",
            ));
        }
        let mut output = Vec::new();
        for target in targets {
            if let ExpressionKind::Star { qualifier } = &target.expression.kind {
                let first = output.len();
                expand_star(target, qualifier, frames, &mut output)?;
                let expansion = StarExpansion {
                    start: target.expression.span.start,
                    end: target.expression.span.end,
                    qualifier: qualifier.last().cloned(),
                    columns: output[first..]
                        .iter()
                        .map(|field| field.name.clone())
                        .collect(),
                };
                self.star_expansions
                    .entry((expansion.start, expansion.end))
                    .or_insert(expansion);
                continue;
            }
            let index = output.len();
            let expected = expected_fields.and_then(|fields| fields.get(index));
            let fact = self.infer(&target.expression, frames, expected)?;
            let name = target
                .alias
                .clone()
                .or(fact.name_hint)
                .or_else(|| stable_expression_name(&target.expression))
                .ok_or_else(|| {
                    PostgresAnalysisError::new(
                        PostgresDiagnosticCode::InvalidResult,
                        "PostgreSQL result expression needs an explicit stable AS alias",
                        &target.expression,
                    )
                })?;
            output.push(ResultFact {
                name,
                database_type: fact.database_type,
                nullable: fact.nullable,
                source_object: fact.source_object,
            });
        }
        if expected_fields.is_some_and(|fields| fields.len() != output.len()) {
            return Err(PostgresAnalysisError::at_start(
                PostgresDiagnosticCode::TypeMismatch,
                "expanded SELECT result width does not match its assignment context",
            ));
        }
        Ok(output)
    }

    pub(crate) fn value_fields(
        &mut self,
        rows: &[Vec<Expression>],
        frames: &[ScopeFrame],
    ) -> Result<Vec<ResultFact>, PostgresAnalysisError> {
        let Some(first) = rows.first() else {
            return Ok(Vec::new());
        };
        let mut facts = first
            .iter()
            .map(|expression| self.infer(expression, frames, None))
            .collect::<Result<Vec<_>, _>>()?;
        for row in rows.iter().skip(1) {
            if row.len() != facts.len() {
                return Err(PostgresAnalysisError::at_start(
                    PostgresDiagnosticCode::TypeMismatch,
                    "VALUES rows have different widths",
                ));
            }
            for (expression, fact) in row.iter().zip(&mut facts) {
                let next = self.infer(expression, frames, Some(&fact.database_type))?;
                fact.nullable |= next.nullable;
            }
        }
        Ok(facts
            .into_iter()
            .enumerate()
            .map(|(index, fact)| ResultFact {
                name: format!("column_{}", index + 1),
                database_type: fact.database_type,
                nullable: fact.nullable,
                source_object: None,
            })
            .collect())
    }

    pub(crate) fn require_boolean(
        &mut self,
        expression: &Expression,
        frames: &[ScopeFrame],
    ) -> Result<(), PostgresAnalysisError> {
        let fact = self.infer(expression, frames, Some(&DatabaseType::Boolean))?;
        if fact.database_type != DatabaseType::Boolean {
            return Err(PostgresAnalysisError::new(
                PostgresDiagnosticCode::TypeMismatch,
                "PostgreSQL predicate must have boolean type",
                expression,
            ));
        }
        Ok(())
    }
}

fn expand_star(
    target: &SelectItem,
    qualifier: &[String],
    frames: &[ScopeFrame],
    output: &mut Vec<ResultFact>,
) -> Result<(), PostgresAnalysisError> {
    let mut matched = false;
    for binding in frames.last().into_iter().flat_map(|frame| &frame.bindings) {
        if !qualifier.is_empty() && qualifier.last().is_none_or(|name| name != &binding.alias) {
            continue;
        }
        matched = true;
        for column_name in &binding.column_order {
            let Some(column) = binding.columns.get(column_name) else {
                continue;
            };
            output.push(ResultFact {
                name: column.name.clone(),
                database_type: column.database_type.clone(),
                nullable: column.nullable,
                source_object: Some(column.identity.clone()),
            });
        }
    }
    if !matched {
        return Err(PostgresAnalysisError::new(
            PostgresDiagnosticCode::UnknownRelation,
            "SELECT * qualifier does not name a relation in scope",
            &target.expression,
        ));
    }
    Ok(())
}

fn stable_expression_name(expression: &Expression) -> Option<String> {
    match &expression.kind {
        ExpressionKind::Function { name, .. } => name.last().cloned(),
        _ => None,
    }
}
