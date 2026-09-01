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
            window,
            filter,
            ..
        } => {
            window.is_none()
                && (*aggregate_star
                    || name.last().is_some_and(|name| {
                        matches!(name.as_str(), "count" | "sum" | "min" | "max" | "avg")
                    })
                    || arguments.iter().any(expression_has_aggregate))
                || filter.as_deref().is_some_and(expression_has_aggregate)
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
        ExpressionKind::Array { elements }
        | ExpressionKind::Coalesce {
            arguments: elements,
        } => elements.iter().any(expression_has_aggregate),
        ExpressionKind::Case {
            operand,
            branches,
            fallback,
        } => {
            operand.as_deref().is_some_and(expression_has_aggregate)
                || branches.iter().any(|branch| {
                    expression_has_aggregate(&branch.condition)
                        || expression_has_aggregate(&branch.result)
                })
                || fallback.as_deref().is_some_and(expression_has_aggregate)
        }
        // A subquery has its own aggregation level. Its aggregates never change
        // the cardinality of the containing SELECT.
        ExpressionKind::Subquery { .. }
        | ExpressionKind::Exists { .. }
        | ExpressionKind::SubqueryComparison { .. } => false,
        _ => false,
    }
}

pub(crate) fn expression_has_window(expression: &Expression) -> bool {
    match &expression.kind {
        ExpressionKind::Function {
            arguments,
            filter,
            window,
            ..
        } => {
            window.is_some()
                || arguments.iter().any(expression_has_window)
                || filter.as_deref().is_some_and(expression_has_window)
        }
        ExpressionKind::Cast { expression, .. }
        | ExpressionKind::Unary { expression, .. }
        | ExpressionKind::NullTest { expression, .. } => expression_has_window(expression),
        ExpressionKind::Binary { left, right, .. } => {
            expression_has_window(left) || expression_has_window(right)
        }
        ExpressionKind::InList {
            expression, values, ..
        } => expression_has_window(expression) || values.iter().any(expression_has_window),
        ExpressionKind::BooleanList { expressions, .. } => {
            expressions.iter().any(expression_has_window)
        }
        ExpressionKind::Array { elements }
        | ExpressionKind::Coalesce {
            arguments: elements,
        } => elements.iter().any(expression_has_window),
        ExpressionKind::Case {
            operand,
            branches,
            fallback,
        } => {
            operand.as_deref().is_some_and(expression_has_window)
                || branches.iter().any(|branch| {
                    expression_has_window(&branch.condition)
                        || expression_has_window(&branch.result)
                })
                || fallback.as_deref().is_some_and(expression_has_window)
        }
        ExpressionKind::Subquery { .. }
        | ExpressionKind::Exists { .. }
        | ExpressionKind::SubqueryComparison { .. } => false,
        _ => false,
    }
}

pub(crate) fn window_references(expression: &Expression, output: &mut Vec<String>) {
    match &expression.kind {
        ExpressionKind::Function {
            arguments,
            filter,
            window,
            ..
        } => {
            if let Some(reference) = window.as_ref().and_then(|window| window.reference.as_ref()) {
                output.push(reference.clone());
            }
            for argument in arguments {
                window_references(argument, output);
            }
            if let Some(filter) = filter {
                window_references(filter, output);
            }
        }
        ExpressionKind::Cast { expression, .. }
        | ExpressionKind::Unary { expression, .. }
        | ExpressionKind::NullTest { expression, .. } => window_references(expression, output),
        ExpressionKind::Binary { left, right, .. } => {
            window_references(left, output);
            window_references(right, output);
        }
        ExpressionKind::InList {
            expression, values, ..
        } => {
            window_references(expression, output);
            for value in values {
                window_references(value, output);
            }
        }
        ExpressionKind::BooleanList { expressions, .. }
        | ExpressionKind::Array {
            elements: expressions,
        }
        | ExpressionKind::Coalesce {
            arguments: expressions,
        } => {
            for expression in expressions {
                window_references(expression, output);
            }
        }
        ExpressionKind::Case {
            operand,
            branches,
            fallback,
        } => {
            if let Some(operand) = operand {
                window_references(operand, output);
            }
            for branch in branches {
                window_references(&branch.condition, output);
                window_references(&branch.result, output);
            }
            if let Some(fallback) = fallback {
                window_references(fallback, output);
            }
        }
        _ => {}
    }
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
    if !implicit && is_numeric(source) && is_numeric(target) {
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
