use crate::analysis::ScopeFrame;
use crate::ast::{Expression, ExpressionKind, SelectStatement, SetOperator};
use crate::catalog::PostgresCatalog;
use sifr_sql_contract::Cardinality;

pub(crate) fn make_frame_nullable(frame: &mut ScopeFrame) {
    for binding in &mut frame.bindings {
        for column in binding.columns.values_mut() {
            column.nullable = true;
        }
    }
}

pub(crate) fn set_cardinality(
    operator: SetOperator,
    all: bool,
    left: Cardinality,
    right: Cardinality,
) -> Cardinality {
    if operator == SetOperator::Union && all {
        return add_cardinalities(left, right);
    }
    match operator {
        SetOperator::Union if left == Cardinality::ZERO => distinct_cardinality(right),
        SetOperator::Union if right == Cardinality::ZERO => distinct_cardinality(left),
        SetOperator::Union => distinct_cardinality(add_cardinalities(left, right)),
        SetOperator::Intersect if left == Cardinality::ZERO || right == Cardinality::ZERO => {
            Cardinality::ZERO
        }
        SetOperator::Intersect => Cardinality::MANY,
        SetOperator::Except if left == Cardinality::ZERO => Cardinality::ZERO,
        SetOperator::Except => match left {
            Cardinality::Empty => Cardinality::BOTTOM,
            Cardinality::Interval { maximum, .. } => {
                Cardinality::new(0, maximum).unwrap_or(Cardinality::MANY)
            }
        },
    }
}

fn distinct_cardinality(cardinality: Cardinality) -> Cardinality {
    match cardinality {
        Cardinality::Empty => Cardinality::BOTTOM,
        Cardinality::Interval { minimum, maximum } => {
            Cardinality::new(minimum.min(1), maximum).unwrap_or(Cardinality::MANY)
        }
    }
}

fn add_cardinalities(left: Cardinality, right: Cardinality) -> Cardinality {
    match (left, right) {
        (Cardinality::Empty, value) | (value, Cardinality::Empty) => value,
        (
            Cardinality::Interval {
                minimum: left_minimum,
                maximum: left_maximum,
            },
            Cardinality::Interval {
                minimum: right_minimum,
                maximum: right_maximum,
            },
        ) => Cardinality::new(
            left_minimum.saturating_add(right_minimum),
            match (left_maximum, right_maximum) {
                (Some(left), Some(right)) => left.checked_add(right),
                _ => None,
            },
        )
        .unwrap_or(Cardinality::MANY),
    }
}

pub(crate) fn apply_limit_and_offset(
    cardinality: Cardinality,
    limit: Option<&Expression>,
    offset: Option<&Expression>,
) -> Cardinality {
    let offset = integer_literal(offset).unwrap_or(0);
    let limit = integer_literal(limit);
    match cardinality {
        Cardinality::Empty => Cardinality::BOTTOM,
        Cardinality::Interval { minimum, maximum } => {
            let minimum = minimum.saturating_sub(offset);
            let maximum = maximum.map(|value| value.saturating_sub(offset));
            let minimum = limit.map_or(minimum, |limit| minimum.min(limit));
            let maximum = match (maximum, limit) {
                (Some(value), Some(limit)) => Some(value.min(limit)),
                (None, Some(limit)) => Some(limit),
                (value, None) => value,
            };
            Cardinality::new(minimum, maximum).unwrap_or(Cardinality::MANY)
        }
    }
}

fn integer_literal(expression: Option<&Expression>) -> Option<u64> {
    let ExpressionKind::Integer { value } = &expression?.kind else {
        return None;
    };
    value.parse().ok()
}

pub(crate) fn unique_predicate_cardinality(
    select: &SelectStatement,
    frames: &[ScopeFrame],
    catalog: &PostgresCatalog,
) -> bool {
    let Some(predicate) = &select.predicate else {
        return false;
    };
    let relation_bindings = frames
        .last()
        .into_iter()
        .flat_map(|frame| &frame.bindings)
        .filter_map(|binding| {
            binding
                .relation
                .as_ref()
                .map(|identity| (binding, identity))
        })
        .collect::<Vec<_>>();
    relation_bindings.len() == 1
        && relation_bindings.iter().any(|(binding, identity)| {
            let constrained = equality_columns(predicate, &binding.alias);
            catalog.relation_by_id(identity).is_some_and(|relation| {
                (!relation.primary_key.is_empty() && relation.primary_key.is_subset(&constrained))
                    || relation
                        .unique_sets
                        .iter()
                        .any(|columns| columns.iter().all(|column| constrained.contains(column)))
            })
        })
}

fn equality_columns(expression: &Expression, alias: &str) -> std::collections::BTreeSet<String> {
    match &expression.kind {
        ExpressionKind::BooleanList {
            and: true,
            expressions,
        } => expressions
            .iter()
            .flat_map(|expression| equality_columns(expression, alias))
            .collect(),
        ExpressionKind::Binary {
            operator,
            left,
            right,
        } if operator == "=" => [(&left.kind, &right.kind), (&right.kind, &left.kind)]
            .into_iter()
            .find_map(|(column, value)| match (column, value) {
                (
                    ExpressionKind::Column { path },
                    ExpressionKind::Parameter { .. }
                    | ExpressionKind::Integer { .. }
                    | ExpressionKind::Float { .. }
                    | ExpressionKind::String { .. }
                    | ExpressionKind::Boolean { .. },
                ) if path.len() == 1 || path.first().is_some_and(|name| name == alias) => {
                    path.last().cloned()
                }
                _ => None,
            })
            .into_iter()
            .collect(),
        _ => std::collections::BTreeSet::new(),
    }
}

pub(crate) fn group_expression_valid(expression: &Expression, group_by: &[Expression]) -> bool {
    group_by.iter().any(|group| group.kind == expression.kind)
        || matches!(
            expression.kind,
            ExpressionKind::Integer { .. }
                | ExpressionKind::Float { .. }
                | ExpressionKind::String { .. }
                | ExpressionKind::Boolean { .. }
                | ExpressionKind::Null
        )
        || match &expression.kind {
            ExpressionKind::Cast { expression, .. }
            | ExpressionKind::Unary { expression, .. }
            | ExpressionKind::NullTest { expression, .. } => {
                group_expression_valid(expression, group_by)
            }
            ExpressionKind::Binary { left, right, .. } => {
                group_expression_valid(left, group_by) && group_expression_valid(right, group_by)
            }
            ExpressionKind::Function {
                arguments,
                filter,
                window,
                ..
            } => {
                arguments
                    .iter()
                    .all(|argument| group_expression_valid(argument, group_by))
                    && filter
                        .as_deref()
                        .is_none_or(|value| group_expression_valid(value, group_by))
                    && window.is_none()
            }
            ExpressionKind::Case {
                operand,
                branches,
                fallback,
            } => {
                operand
                    .as_deref()
                    .is_none_or(|value| group_expression_valid(value, group_by))
                    && branches.iter().all(|branch| {
                        group_expression_valid(&branch.condition, group_by)
                            && group_expression_valid(&branch.result, group_by)
                    })
                    && fallback
                        .as_deref()
                        .is_none_or(|value| group_expression_valid(value, group_by))
            }
            _ => false,
        }
}

pub(crate) fn group_expression_functionally_dependent(
    expression: &Expression,
    group_by: &[Expression],
    frames: &[ScopeFrame],
    catalog: &PostgresCatalog,
) -> bool {
    match &expression.kind {
        ExpressionKind::Column { path } => frames
            .last()
            .into_iter()
            .flat_map(|frame| &frame.bindings)
            .filter(|binding| {
                binding.columns.contains_key(path.last().map(String::as_str).unwrap_or_default())
                    && (path.len() == 1
                        || path
                            .get(path.len().saturating_sub(2))
                            .is_some_and(|qualifier| qualifier == &binding.alias))
            })
            .any(|binding| {
                binding
                    .relation
                    .as_ref()
                    .and_then(|identity| catalog.relation_by_id(identity))
                    .is_some_and(|relation| {
                        !relation.primary_key.is_empty()
                            && relation.primary_key.iter().all(|key| {
                                group_by.iter().any(|group| {
                                    matches!(
                                        &group.kind,
                                        ExpressionKind::Column { path }
                                            if path.last() == Some(key)
                                                && (path.len() == 1
                                                    || path
                                                        .get(path.len().saturating_sub(2))
                                                        .is_some_and(|qualifier| qualifier == &binding.alias))
                                    )
                                })
                            })
                    })
            }),
        ExpressionKind::Cast { expression, .. }
        | ExpressionKind::Unary { expression, .. }
        | ExpressionKind::NullTest { expression, .. } => {
            group_expression_functionally_dependent(expression, group_by, frames, catalog)
        }
        ExpressionKind::Binary { left, right, .. } => {
            group_expression_functionally_dependent(left, group_by, frames, catalog)
                && group_expression_functionally_dependent(right, group_by, frames, catalog)
        }
        ExpressionKind::Function { arguments, .. } => arguments.iter().all(|argument| {
            group_expression_functionally_dependent(argument, group_by, frames, catalog)
        }),
        _ => false,
    }
}
