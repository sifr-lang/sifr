use crate::analysis::{PostgresAnalysisError, ResultFact, TypeFact};
use crate::ast::{Expression, ExpressionKind};
use crate::diagnostic::PostgresDiagnosticCode;
use sifr_sql_contract::{DatabaseType, IntegerSign, IntegerWidth};
use std::collections::BTreeSet;

pub(crate) fn unique_result_names(fields: &mut [ResultFact]) -> Result<(), PostgresAnalysisError> {
    let mut names = BTreeSet::new();
    for field in fields {
        if !names.insert(field.name.clone()) {
            return Err(PostgresAnalysisError::at_start(
                PostgresDiagnosticCode::InvalidResult,
                format!("PostgreSQL result name '{}' is duplicated", field.name),
            ));
        }
    }
    Ok(())
}

pub(crate) fn expression_has_aggregate(expression: &Expression) -> bool {
    match &expression.kind {
        ExpressionKind::Function {
            name,
            arguments,
            aggregate_star,
        } => {
            *aggregate_star
                || name.last().is_some_and(|name| {
                    matches!(name.as_str(), "count" | "sum" | "min" | "max" | "avg")
                })
                || arguments.iter().any(expression_has_aggregate)
        }
        ExpressionKind::Cast { expression, .. }
        | ExpressionKind::Unary { expression, .. }
        | ExpressionKind::NullTest { expression, .. } => expression_has_aggregate(expression),
        ExpressionKind::Binary { left, right, .. } => {
            expression_has_aggregate(left) || expression_has_aggregate(right)
        }
        ExpressionKind::BooleanList { expressions, .. } => {
            expressions.iter().any(expression_has_aggregate)
        }
        _ => false,
    }
}

pub(crate) fn limit_is_one(expression: Option<&Expression>) -> bool {
    expression.is_some_and(
        |expression| matches!(&expression.kind, ExpressionKind::Integer { value } if value == "1"),
    )
}

pub(crate) fn type_fact(database_type: DatabaseType, nullable: bool) -> TypeFact {
    TypeFact {
        database_type,
        nullable,
        source_object: None,
        name_hint: None,
    }
}

pub(crate) fn integer_type() -> DatabaseType {
    DatabaseType::Integer {
        sign: IntegerSign::Signed,
        width: IntegerWidth::Bits32,
    }
}

pub(crate) fn integer64_type() -> DatabaseType {
    DatabaseType::Integer {
        sign: IntegerSign::Signed,
        width: IntegerWidth::Bits64,
    }
}

pub(crate) fn text_type() -> DatabaseType {
    DatabaseType::Text {
        fixed: false,
        max_characters: None,
    }
}

pub(crate) fn is_numeric(ty: &DatabaseType) -> bool {
    matches!(
        ty,
        DatabaseType::Integer { .. }
            | DatabaseType::Decimal { .. }
            | DatabaseType::Float32
            | DatabaseType::Float64
    )
}
