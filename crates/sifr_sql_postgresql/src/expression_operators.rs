use crate::analysis::{AnalysisContext, PostgresAnalysisError, ScopeFrame, TypeFact};
use crate::ast::{Expression, ExpressionKind};
use crate::diagnostic::PostgresDiagnosticCode;
use crate::semantic_helpers::{
    arithmetic_result, canonical_database_type, integer64_type, is_numeric, is_textual, text_type,
    type_fact,
};
use sifr_sql_contract::DatabaseType;

impl AnalysisContext<'_> {
    pub(crate) fn infer_binary(
        &mut self,
        operator: &str,
        left: &Expression,
        right: &Expression,
        frames: &[ScopeFrame],
        whole: &Expression,
    ) -> Result<TypeFact, PostgresAnalysisError> {
        let left_parameter = matches!(
            left.kind,
            ExpressionKind::Parameter { .. } | ExpressionKind::Null
        );
        let right_parameter = matches!(
            right.kind,
            ExpressionKind::Parameter { .. } | ExpressionKind::Null
        );
        let (left_fact, right_fact) = if left_parameter && !right_parameter {
            let right_fact = self.infer(right, frames, None)?;
            let left_fact = self.infer(left, frames, Some(&right_fact.database_type))?;
            (left_fact, right_fact)
        } else if right_parameter && !left_parameter {
            let left_fact = self.infer(left, frames, None)?;
            let right_fact = self.infer(right, frames, Some(&left_fact.database_type))?;
            (left_fact, right_fact)
        } else {
            (
                self.infer(left, frames, None)?,
                self.infer(right, frames, None)?,
            )
        };
        let nullable = left_fact.nullable || right_fact.nullable;
        if matches!(operator, "IS DISTINCT FROM" | "IS NOT DISTINCT FROM")
            && (left_fact.database_type == right_fact.database_type
                || self
                    .catalog
                    .can_cast(&left_fact.database_type, &right_fact.database_type, true)
                || self
                    .catalog
                    .can_cast(&right_fact.database_type, &left_fact.database_type, true))
        {
            return Ok(type_fact(DatabaseType::Boolean, false));
        }
        if matches!(operator, "=" | "<>" | "<" | ">" | "<=" | ">=")
            && (left_fact.database_type == right_fact.database_type
                || self
                    .catalog
                    .can_cast(&left_fact.database_type, &right_fact.database_type, true)
                || self
                    .catalog
                    .can_cast(&right_fact.database_type, &left_fact.database_type, true))
        {
            return Ok(type_fact(DatabaseType::Boolean, nullable));
        }
        if operator == "||"
            && is_textual(&left_fact.database_type)
            && is_textual(&right_fact.database_type)
        {
            return Ok(type_fact(text_type(), nullable));
        }
        if matches!(operator, "LIKE" | "NOT LIKE")
            && is_textual(&left_fact.database_type)
            && is_textual(&right_fact.database_type)
        {
            return Ok(type_fact(DatabaseType::Boolean, nullable));
        }
        if matches!(
            operator,
            "@>" | "<@" | "?" | "?|" | "?&" | "&&" | "-|-" | "&<" | "&>"
        ) && (matches!(
            canonical_database_type(&left_fact.database_type),
            DatabaseType::Json { .. }
        ) || matches!(
            canonical_database_type(&left_fact.database_type),
            DatabaseType::Array { .. } | DatabaseType::Range { .. }
        )) {
            return Ok(type_fact(DatabaseType::Boolean, nullable));
        }
        if matches!(operator, "->" | "#>")
            && matches!(
                canonical_database_type(&left_fact.database_type),
                DatabaseType::Json { .. }
            )
        {
            return Ok(type_fact(left_fact.database_type, true));
        }
        if matches!(operator, "->>" | "#>>")
            && matches!(
                canonical_database_type(&left_fact.database_type),
                DatabaseType::Json { .. }
            )
        {
            return Ok(type_fact(text_type(), true));
        }
        if operator == "||" && left_fact.database_type == right_fact.database_type {
            return Ok(type_fact(left_fact.database_type, nullable));
        }
        if operator == "||"
            && let (
                DatabaseType::Array {
                    element: left_element,
                    dimensions,
                    preserves_lower_bounds,
                    ..
                },
                DatabaseType::Array {
                    element: right_element,
                    ..
                },
            ) = (&left_fact.database_type, &right_fact.database_type)
            && canonical_database_type(left_element) == canonical_database_type(right_element)
        {
            return Ok(type_fact(
                DatabaseType::Array {
                    element: left_element.clone(),
                    dimensions: *dimensions,
                    element_nullability: sifr_sql_contract::Nullability::Nullable,
                    preserves_lower_bounds: *preserves_lower_bounds,
                },
                nullable,
            ));
        }
        if let Some(result) = arithmetic_result(
            operator,
            &left_fact.database_type,
            &right_fact.database_type,
        ) {
            return Ok(type_fact(result, nullable));
        }
        if let Some(found) = self.catalog.operators(operator).iter().find(|candidate| {
            self.catalog
                .can_cast(&left_fact.database_type, &candidate.left, true)
                && self
                    .catalog
                    .can_cast(&right_fact.database_type, &candidate.right, true)
        }) {
            return Ok(type_fact(found.result.clone(), nullable));
        }
        Err(PostgresAnalysisError::new(
            PostgresDiagnosticCode::UnknownOperator,
            format!("no PostgreSQL operator '{operator}' accepts these operand types"),
            whole,
        ))
    }

    pub(crate) fn infer_function(
        &mut self,
        name: &[String],
        arguments: &[Expression],
        aggregate_star: bool,
        frames: &[ScopeFrame],
        expression: &Expression,
    ) -> Result<TypeFact, PostgresAnalysisError> {
        let short = name
            .last()
            .map(String::as_str)
            .unwrap_or_default()
            .to_ascii_lowercase();
        if short == "count" && (aggregate_star || arguments.len() == 1) {
            for argument in arguments {
                self.infer(argument, frames, None)?;
            }
            return Ok(type_fact(integer64_type(), false));
        }
        if matches!(short.as_str(), "lower" | "upper") && arguments.len() == 1 {
            let argument = self.infer(&arguments[0], frames, Some(&text_type()))?;
            if !is_textual(&argument.database_type) {
                return Err(PostgresAnalysisError::new(
                    PostgresDiagnosticCode::UnknownFunction,
                    "lower/upper needs a text argument",
                    expression,
                ));
            }
            return Ok(type_fact(text_type(), argument.nullable));
        }
        if short == "now" && arguments.is_empty() {
            return Ok(type_fact(DatabaseType::Instant { precision: 6 }, false));
        }
        if matches!(short.as_str(), "row_number" | "rank" | "dense_rank") && arguments.is_empty() {
            return Ok(type_fact(integer64_type(), false));
        }
        if matches!(
            short.as_str(),
            "lag" | "lead" | "first_value" | "last_value"
        ) && !arguments.is_empty()
        {
            let argument = self.infer(&arguments[0], frames, None)?;
            for additional in &arguments[1..] {
                self.infer(additional, frames, None)?;
            }
            return Ok(type_fact(argument.database_type, true));
        }
        if matches!(short.as_str(), "sum" | "avg" | "min" | "max") && arguments.len() == 1 {
            let argument = self.infer(&arguments[0], frames, None)?;
            let result = match short.as_str() {
                "sum" => sum_type(&argument.database_type).ok_or_else(|| {
                    PostgresAnalysisError::new(
                        PostgresDiagnosticCode::UnknownFunction,
                        "sum needs a numeric argument",
                        expression,
                    )
                })?,
                "avg" => average_type(&argument.database_type).ok_or_else(|| {
                    PostgresAnalysisError::new(
                        PostgresDiagnosticCode::UnknownFunction,
                        "avg needs a numeric argument",
                        expression,
                    )
                })?,
                "min" | "max" => argument.database_type,
                _ => {
                    return Err(PostgresAnalysisError::new(
                        PostgresDiagnosticCode::UnknownFunction,
                        "unknown PostgreSQL aggregate",
                        expression,
                    ));
                }
            };
            return Ok(type_fact(result, true));
        }
        let candidates = self.catalog.functions(name);
        for candidate in candidates {
            if candidate.arguments.len() != arguments.len() {
                continue;
            }
            let mut facts = Vec::with_capacity(arguments.len());
            let mut compatible = true;
            for (argument, expected) in arguments.iter().zip(&candidate.arguments) {
                let fact = self.infer(argument, frames, Some(expected))?;
                compatible &= self.catalog.can_cast(&fact.database_type, expected, true);
                facts.push(fact);
            }
            if compatible {
                return Ok(type_fact(
                    candidate.result.clone(),
                    candidate.result_nullable
                        || (candidate.strict && facts.iter().any(|fact| fact.nullable)),
                ));
            }
        }
        Err(PostgresAnalysisError::new(
            PostgresDiagnosticCode::UnknownFunction,
            format!(
                "no PostgreSQL function '{}' matches these arguments",
                name.join(".")
            ),
            expression,
        ))
    }
}

fn sum_type(value: &DatabaseType) -> Option<DatabaseType> {
    match value {
        DatabaseType::Integer {
            width: sifr_sql_contract::IntegerWidth::Bits16 | sifr_sql_contract::IntegerWidth::Bits32,
            ..
        } => Some(integer64_type()),
        DatabaseType::Integer {
            width: sifr_sql_contract::IntegerWidth::Bits64,
            ..
        } => Some(numeric_type()),
        value if is_numeric(value) => Some(value.clone()),
        _ => None,
    }
}

fn average_type(value: &DatabaseType) -> Option<DatabaseType> {
    match value {
        DatabaseType::Integer { .. } | DatabaseType::Decimal { .. } => Some(numeric_type()),
        DatabaseType::Float32 | DatabaseType::Float64 => Some(value.clone()),
        _ => None,
    }
}

fn numeric_type() -> DatabaseType {
    DatabaseType::Decimal {
        precision: None,
        scale: None,
        representation: sifr_sql_contract::DecimalRepresentation::Numeric,
    }
}
