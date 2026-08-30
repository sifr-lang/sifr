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
        ExpressionKind::InList {
            expression, values, ..
        } => expression_has_aggregate(expression) || values.iter().any(expression_has_aggregate),
        ExpressionKind::BooleanList { expressions, .. } => {
            expressions.iter().any(expression_has_aggregate)
        }
        ExpressionKind::Subquery { query }
        | ExpressionKind::Exists { query }
        | ExpressionKind::SubqueryComparison { query, .. } => query
            .targets
            .iter()
            .any(|target| expression_has_aggregate(&target.expression)),
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
        canonical_database_type(ty),
        DatabaseType::Integer { .. }
            | DatabaseType::Decimal { .. }
            | DatabaseType::Float32
            | DatabaseType::Float64
    )
}

pub(crate) fn is_exact_numeric(ty: &DatabaseType) -> bool {
    matches!(
        canonical_database_type(ty),
        DatabaseType::Integer { .. } | DatabaseType::Decimal { .. }
    )
}

pub(crate) fn arithmetic_result(
    operator: &str,
    left: &DatabaseType,
    right: &DatabaseType,
) -> Option<DatabaseType> {
    if !matches!(operator, "+" | "-" | "*" | "/" | "%")
        || !is_numeric(left)
        || !is_numeric(right)
        || (operator == "%" && (!is_exact_numeric(left) || !is_exact_numeric(right)))
    {
        return None;
    }
    if left == right || can_builtin_cast(right, left, true) {
        return Some(left.clone());
    }
    can_builtin_cast(left, right, true).then(|| right.clone())
}

pub(crate) fn can_builtin_cast(
    source: &DatabaseType,
    target: &DatabaseType,
    implicit: bool,
) -> bool {
    if source == target {
        return true;
    }
    let source = canonical_database_type(source);
    let target = canonical_database_type(target);
    if source == target {
        return true;
    }
    if implicit && integer_widens(source, target) {
        return true;
    }
    matches!(
        (source, target, implicit),
        (
            DatabaseType::Integer { .. },
            DatabaseType::Decimal { .. } | DatabaseType::Float32 | DatabaseType::Float64,
            true
        ) | (
            DatabaseType::Integer { .. }
                | DatabaseType::Decimal { .. }
                | DatabaseType::Float32
                | DatabaseType::Float64,
            DatabaseType::Text { .. },
            false
        )
    )
}

fn integer_widens(source: &DatabaseType, target: &DatabaseType) -> bool {
    let (
        DatabaseType::Integer {
            sign: source_sign,
            width: source_width,
        },
        DatabaseType::Integer {
            sign: target_sign,
            width: target_width,
        },
    ) = (source, target)
    else {
        return false;
    };
    source_sign == target_sign
        && integer_width_rank(*source_width) < integer_width_rank(*target_width)
}

fn integer_width_rank(width: IntegerWidth) -> u8 {
    match width {
        IntegerWidth::Bits8 => 0,
        IntegerWidth::Bits16 => 1,
        IntegerWidth::Bits32 => 2,
        IntegerWidth::Bits64 => 3,
    }
}

pub(crate) fn is_textual(ty: &DatabaseType) -> bool {
    matches!(canonical_database_type(ty), DatabaseType::Text { .. })
}

pub(crate) fn canonical_database_type(mut ty: &DatabaseType) -> &DatabaseType {
    while let DatabaseType::Named { canonical, .. } = ty {
        ty = canonical;
    }
    ty
}
