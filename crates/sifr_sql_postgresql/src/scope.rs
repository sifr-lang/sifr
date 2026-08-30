use crate::analysis::{PostgresAnalysisError, ResultFact, ScopeBinding, ScopeFrame, TypeFact};
use crate::ast::Expression;
use crate::catalog::{CatalogColumn, CatalogRelation, PostgresCatalog};
use crate::diagnostic::PostgresDiagnosticCode;
use sifr_sql_contract::ObjectId;

pub(crate) fn resolve_column(
    catalog: &PostgresCatalog,
    path: &[String],
    frames: &[ScopeFrame],
    expression: &Expression,
) -> Result<TypeFact, PostgresAnalysisError> {
    let column_name = path.last().ok_or_else(|| {
        PostgresAnalysisError::new(
            PostgresDiagnosticCode::UnknownColumn,
            "empty PostgreSQL column reference",
            expression,
        )
    })?;
    let qualifier = (path.len() > 1).then(|| path.get(path.len() - 2)).flatten();
    for frame in frames.iter().rev() {
        let matches = frame
            .bindings
            .iter()
            .filter(|binding| qualifier.is_none_or(|value| &binding.alias == value))
            .filter_map(|binding| {
                binding
                    .columns
                    .get(column_name)
                    .map(|column| (binding, column))
            })
            .collect::<Vec<_>>();
        if matches.len() > 1 {
            return Err(PostgresAnalysisError::new(
                PostgresDiagnosticCode::AmbiguousColumn,
                format!("PostgreSQL column '{column_name}' is ambiguous"),
                expression,
            ));
        }
        if let Some((binding, column)) = matches.first() {
            return Ok(TypeFact {
                database_type: column.database_type.clone(),
                nullable: column.nullable,
                source_object: binding.relation.as_ref().map(|_| column.identity.clone()),
                name_hint: Some(column.name.clone()),
            });
        }
    }
    let mut error = PostgresAnalysisError::new(
        PostgresDiagnosticCode::UnknownColumn,
        format!("unknown PostgreSQL column '{}'", path.join(".")),
        expression,
    );
    if let Some(binding) = frames
        .iter()
        .rev()
        .flat_map(|frame| &frame.bindings)
        .find(|binding| qualifier.is_none_or(|value| &binding.alias == value))
        && let Some(relation) = &binding.relation
        && let Some(source) = catalog
            .object(relation)
            .and_then(|object| object.source.as_ref())
    {
        error.diagnostic =
            error
                .diagnostic
                .with_schema_span(source.document.clone(), source.start, source.end);
    }
    Err(error)
}

pub(crate) fn frame_for_results(fields: &[ResultFact]) -> ScopeFrame {
    ScopeFrame {
        bindings: vec![ScopeBinding {
            alias: "<result>".to_string(),
            relation: None,
            columns: fields
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    (
                        field.name.clone(),
                        CatalogColumn {
                            identity: ObjectId::new(format!("result.{index}")),
                            name: field.name.clone(),
                            database_type: field.database_type.clone(),
                            nullable: field.nullable,
                            has_default: false,
                            generated: false,
                            source: None,
                        },
                    )
                })
                .collect(),
        }],
    }
}

pub(crate) fn binding_for_relation(
    relation: &CatalogRelation,
    alias: Option<&str>,
) -> ScopeBinding {
    ScopeBinding {
        alias: alias.map(str::to_string).unwrap_or_else(|| {
            relation
                .identity
                .as_str()
                .rsplit('.')
                .next()
                .unwrap_or("relation")
                .to_string()
        }),
        relation: Some(relation.identity.clone()),
        columns: relation.columns.clone(),
    }
}
