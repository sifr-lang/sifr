pub use crate::analysis_types::PostgresAnalysisError;
pub(crate) use crate::analysis_types::{
    AnalysisContext, AnalyzedStatement, ResultFact, ScopeBinding, ScopeFrame, StarExpansion,
    TypeFact,
};
use crate::ast::{Expression, ExpressionKind, SelectStatement, SetOperator};
use crate::cardinality_analysis::{
    apply_limit_and_offset, group_expression_functionally_dependent, group_expression_valid,
    set_cardinality, unique_predicate_cardinality,
};
use crate::diagnostic::PostgresDiagnosticCode;
use crate::locking_analysis::validate_locking;
use crate::nullability_analysis::refine_for_null_test;
use crate::scope::{frame_for_results, resolve_column};
use crate::semantic_helpers::{
    expression_has_aggregate, expression_has_window, integer_type, integer64_type, is_numeric,
    text_type, type_fact, unique_result_names,
};
use crate::window_analysis::{
    reject_window_expression, validate_named_windows, validate_window_references,
};
use sifr_sql_contract::{Cardinality, DatabaseType, Nullability, QueryEffect};
use std::collections::BTreeSet;

impl AnalysisContext<'_> {
    pub(crate) fn analyze_select(
        &mut self,
        select: &SelectStatement,
        outer: Vec<ScopeFrame>,
    ) -> Result<AnalyzedStatement, PostgresAnalysisError> {
        self.analyze_select_with_expected(select, outer, None)
    }

    pub(crate) fn analyze_select_with_expected(
        &mut self,
        select: &SelectStatement,
        outer: Vec<ScopeFrame>,
        expected_fields: Option<&[DatabaseType]>,
    ) -> Result<AnalyzedStatement, PostgresAnalysisError> {
        self.required_capabilities
            .insert("sql.query.select".to_string());
        if !select.common_tables.is_empty() {
            self.required_capabilities
                .insert("sql.query.common-table-expression".to_string());
        }
        let outer = self.scopes_with_ctes(select, outer)?;
        if let Some(set) = &select.set_operation {
            self.required_capabilities
                .insert("sql.query.set-operation".to_string());
            if !select.locking.is_empty() {
                return Err(PostgresAnalysisError::at_start(
                    PostgresDiagnosticCode::InvalidResult,
                    "PostgreSQL row locking cannot apply to a set operation",
                ));
            }
            let left =
                self.analyze_select_with_expected(&set.left, outer.clone(), expected_fields)?;
            let right = self.analyze_select_with_expected(&set.right, outer, expected_fields)?;
            if left.fields.len() != right.fields.len() {
                return Err(PostgresAnalysisError::at_start(
                    PostgresDiagnosticCode::TypeMismatch,
                    "PostgreSQL set operands return different column counts",
                ));
            }
            let mut fields = Vec::with_capacity(left.fields.len());
            for (left, right) in left.fields.into_iter().zip(right.fields) {
                if left.database_type != right.database_type {
                    return Err(PostgresAnalysisError::at_start(
                        PostgresDiagnosticCode::TypeMismatch,
                        "PostgreSQL set operands return incompatible column types",
                    ));
                }
                fields.push(ResultFact {
                    nullable: left.nullable || right.nullable,
                    ..left
                });
            }
            let mut referenced = left.referenced;
            referenced.extend(right.referenced);
            unique_result_names(&mut fields)?;
            let result_scope = frame_for_results(&fields);
            for order in &select.order_by {
                self.infer(&order.expression, std::slice::from_ref(&result_scope), None)?;
            }
            if let Some(limit) = &select.limit {
                self.infer(limit, &[], Some(&integer64_type()))?;
            }
            if let Some(offset) = &select.offset {
                self.infer(offset, &[], Some(&integer64_type()))?;
            }
            let mut flags = left.flags;
            flags.extend(right.flags);
            flags.insert(
                match set.operator {
                    SetOperator::Union => "set-union",
                    SetOperator::Intersect => "set-intersect",
                    SetOperator::Except => "set-except",
                }
                .to_string(),
            );
            if !select.order_by.is_empty() {
                flags.insert("deterministic-order".to_string());
            }
            if !select.common_tables.is_empty() {
                flags.insert("common-table-expression".to_string());
            }
            return Ok(AnalyzedStatement {
                fields,
                cardinality: apply_limit_and_offset(
                    set_cardinality(set.operator, set.all, left.cardinality, right.cardinality),
                    select.limit.as_ref(),
                    select.offset.as_ref(),
                ),
                effect: QueryEffect::Read,
                referenced,
                affected: BTreeSet::new(),
                flags,
            });
        }
        let mut frames = outer;
        let frame = self.scope_from(&select.from, &frames)?;
        frames.push(frame);
        if let Some(predicate) = &select.predicate {
            reject_window_expression(predicate, "WHERE")?;
            self.require_boolean(predicate, &frames)?;
        }
        let window_names = validate_named_windows(self, select, &frames)?;
        for target in &select.targets {
            validate_window_references(&target.expression, &window_names)?;
        }
        let mut fields = if select.values.is_empty() {
            self.result_fields_with_expected(&select.targets, &frames, expected_fields)?
        } else {
            self.value_fields(&select.values, &frames)?
        };
        unique_result_names(&mut fields)?;
        let mut alias_frames = frames.clone();
        alias_frames.push(frame_for_results(&fields));
        for expression in &select.group_by {
            reject_window_expression(expression, "GROUP BY")?;
            self.infer(expression, &alias_frames, None)?;
        }
        if let Some(having) = &select.having {
            reject_window_expression(having, "HAVING")?;
            self.require_boolean(having, &frames)?;
        }
        for order in &select.order_by {
            validate_window_references(&order.expression, &window_names)?;
            self.infer(&order.expression, &alias_frames, None)?;
        }
        if let Some(limit) = &select.limit {
            self.infer(limit, &[], Some(&integer64_type()))?;
        }
        if let Some(offset) = &select.offset {
            self.infer(offset, &[], Some(&integer64_type()))?;
        }
        let aggregate = select
            .targets
            .iter()
            .any(|target| expression_has_aggregate(&target.expression));
        if aggregate || !select.group_by.is_empty() {
            self.required_capabilities
                .insert("sql.query.aggregate".to_string());
            for target in &select.targets {
                if !expression_has_aggregate(&target.expression)
                    && !group_expression_valid(&target.expression, &select.group_by)
                    && !group_expression_functionally_dependent(
                        &target.expression,
                        &select.group_by,
                        &frames,
                        self.catalog,
                    )
                {
                    return Err(PostgresAnalysisError::new(
                        PostgresDiagnosticCode::InvalidResult,
                        "a non-aggregate SELECT expression must appear in GROUP BY",
                        &target.expression,
                    ));
                }
            }
        }
        let base_cardinality = if !select.values.is_empty() {
            let rows = u64::try_from(select.values.len()).unwrap_or(u64::MAX);
            Cardinality::new(rows, Some(rows)).unwrap_or(Cardinality::MANY)
        } else if aggregate && select.group_by.is_empty() {
            if select.having.is_some() {
                Cardinality::AT_MOST_ONE
            } else {
                Cardinality::EXACTLY_ONE
            }
        } else if select.from.is_empty() {
            if select.predicate.is_some() {
                Cardinality::AT_MOST_ONE
            } else {
                Cardinality::EXACTLY_ONE
            }
        } else if unique_predicate_cardinality(select, &frames, self.catalog) {
            Cardinality::AT_MOST_ONE
        } else {
            Cardinality::MANY
        };
        let cardinality = apply_limit_and_offset(
            base_cardinality,
            select.limit.as_ref(),
            select.offset.as_ref(),
        );
        let mut flags = if select.order_by.is_empty() {
            BTreeSet::new()
        } else {
            BTreeSet::from(["deterministic-order".to_string()])
        };
        if !select.common_tables.is_empty() {
            self.required_capabilities
                .insert("sql.query.common-table-expression".to_string());
            flags.insert("common-table-expression".to_string());
        }
        if !select.locking.is_empty() {
            self.required_capabilities
                .insert("sql.query.row-locking".to_string());
            validate_locking(select, &frames, aggregate)?;
            flags.insert("row-locking".to_string());
        }
        if !select.windows.is_empty()
            || select
                .targets
                .iter()
                .any(|target| expression_has_window(&target.expression))
        {
            self.required_capabilities
                .insert("sql.query.window".to_string());
            flags.insert("window-function".to_string());
        }
        if select
            .targets
            .iter()
            .any(|target| matches!(target.expression.kind, ExpressionKind::Star { .. }))
        {
            flags.insert("expanded-select-star".to_string());
        }
        Ok(AnalyzedStatement {
            fields,
            cardinality,
            effect: QueryEffect::Read,
            referenced: self.referenced.clone(),
            affected: BTreeSet::new(),
            flags,
        })
    }

    fn scopes_with_ctes(
        &mut self,
        select: &SelectStatement,
        mut frames: Vec<ScopeFrame>,
    ) -> Result<Vec<ScopeFrame>, PostgresAnalysisError> {
        let mut cte_frame = ScopeFrame::default();
        let mut names = BTreeSet::new();
        for cte in &select.common_tables {
            if !names.insert(cte.name.clone()) {
                return Err(PostgresAnalysisError::at_start(
                    PostgresDiagnosticCode::InvalidResult,
                    format!("CTE '{}' is declared more than once", cte.name),
                ));
            }
            let mut cte_scopes = frames.clone();
            if !cte_frame.bindings.is_empty() {
                cte_scopes.push(cte_frame.clone());
            }
            if select.recursive
                && let Some(set) = &cte.query.set_operation
            {
                let anchor = self.analyze_select(&set.left, cte_scopes.clone())?;
                let anchor_names = cte_names(cte, &anchor.fields)?;
                cte_frame.bindings.push(ScopeBinding::derived(
                    &cte.name,
                    anchor_names,
                    anchor.fields,
                ));
            }
            if select.recursive {
                cte_scopes.clone_from(&frames);
                cte_scopes.push(cte_frame.clone());
            }
            let analyzed = self.analyze_select(&cte.query, cte_scopes)?;
            let names = cte_names(cte, &analyzed.fields)?;
            if select.recursive
                && cte_frame
                    .bindings
                    .last()
                    .is_some_and(|binding| binding.alias == cte.name)
            {
                cte_frame.bindings.pop();
            }
            cte_frame
                .bindings
                .push(ScopeBinding::derived(&cte.name, names, analyzed.fields));
        }
        if !cte_frame.bindings.is_empty() {
            frames.push(cte_frame);
        }
        Ok(frames)
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn infer(
        &mut self,
        expression: &Expression,
        frames: &[ScopeFrame],
        expected: Option<&DatabaseType>,
    ) -> Result<TypeFact, PostgresAnalysisError> {
        match &expression.kind {
            ExpressionKind::Star { .. } => Err(PostgresAnalysisError::new(
                PostgresDiagnosticCode::InvalidResult,
                "wildcard '*' is valid only as a SELECT projection item; this wildcard is inside an expression, so replace it with an explicit column or literal",
                expression,
            )),
            ExpressionKind::Column { path } => {
                let fact = resolve_column(self.catalog, path, frames, expression)?;
                if let Some(object) = &fact.source_object {
                    self.accessed_objects.insert(object.clone());
                }
                Ok(fact)
            }
            ExpressionKind::Parameter { number } => {
                let expected = expected.ok_or_else(|| {
                    PostgresAnalysisError::new(
                        PostgresDiagnosticCode::InvalidParameter,
                        format!("cannot infer one exact PostgreSQL type for ${number}"),
                        expression,
                    )
                })?;
                if let Some(previous) = self.parameters.insert(*number, expected.clone())
                    && previous != *expected
                {
                    return Err(PostgresAnalysisError::new(
                        PostgresDiagnosticCode::InvalidParameter,
                        format!("PostgreSQL parameter ${number} has incompatible constraints"),
                        expression,
                    ));
                }
                Ok(type_fact(expected.clone(), false))
            }
            ExpressionKind::Integer { .. } => Ok(type_fact(integer_type(), false)),
            ExpressionKind::Float { .. } => Ok(type_fact(DatabaseType::Float64, false)),
            // PostgreSQL string constants start with the pseudo-type `unknown`.
            // Resolve them to an exact contextual type when one exists; use
            // text only when the expression has no stronger context.
            ExpressionKind::String { .. } => Ok(type_fact(
                expected.cloned().unwrap_or_else(text_type),
                false,
            )),
            ExpressionKind::Boolean { .. } => Ok(type_fact(DatabaseType::Boolean, false)),
            ExpressionKind::Null => {
                expected
                    .cloned()
                    .map(|ty| type_fact(ty, true))
                    .ok_or_else(|| {
                        PostgresAnalysisError::new(
                            PostgresDiagnosticCode::TypeMismatch,
                            "untyped NULL needs an exact PostgreSQL context",
                            expression,
                        )
                    })
            }
            ExpressionKind::Cast {
                expression: inner,
                ty,
            } => {
                let target = self.catalog.types.resolve(ty).ok_or_else(|| {
                    PostgresAnalysisError::new(
                        PostgresDiagnosticCode::TypeMismatch,
                        format!("unknown PostgreSQL type '{}'", ty.display()),
                        expression,
                    )
                })?;
                let source = self.infer(inner, frames, Some(&target.database_type))?;
                if !self
                    .catalog
                    .can_cast(&source.database_type, &target.database_type, false)
                {
                    return Err(PostgresAnalysisError::new(
                        PostgresDiagnosticCode::TypeMismatch,
                        "PostgreSQL cast does not exist",
                        expression,
                    ));
                }
                Ok(TypeFact {
                    database_type: target.database_type.clone(),
                    nullable: source.nullable,
                    source_object: None,
                    name_hint: None,
                })
            }
            ExpressionKind::Binary {
                operator,
                left,
                right,
            } => {
                if matches!(operator.as_str(), "=" | "<>") {
                    self.required_capabilities
                        .insert("sql.expression.equality".to_string());
                }
                self.infer_binary(operator, left, right, frames, expression)
            }
            ExpressionKind::InList {
                expression: left,
                values,
                ..
            } => {
                self.required_capabilities
                    .insert("sql.expression.equality".to_string());
                let Some(first) = values.first() else {
                    return Err(PostgresAnalysisError::new(
                        PostgresDiagnosticCode::TypeMismatch,
                        "PostgreSQL IN list cannot be empty",
                        expression,
                    ));
                };
                let left_is_untyped = matches!(
                    left.kind,
                    ExpressionKind::Parameter { .. } | ExpressionKind::Null
                );
                let (left_fact, first_fact) = if left_is_untyped {
                    let first_fact = self.infer(first, frames, None)?;
                    let left_fact = self.infer(left, frames, Some(&first_fact.database_type))?;
                    (left_fact, first_fact)
                } else {
                    let left_fact = self.infer(left, frames, None)?;
                    let first_fact = self.infer(first, frames, Some(&left_fact.database_type))?;
                    (left_fact, first_fact)
                };
                let mut nullable = left_fact.nullable || first_fact.nullable;
                for value in values.iter().skip(1) {
                    let fact = self.infer(value, frames, Some(&left_fact.database_type))?;
                    if !self
                        .catalog
                        .can_cast(&fact.database_type, &left_fact.database_type, true)
                    {
                        return Err(PostgresAnalysisError::new(
                            PostgresDiagnosticCode::TypeMismatch,
                            "PostgreSQL IN value has an incompatible type",
                            value,
                        ));
                    }
                    nullable |= fact.nullable;
                }
                Ok(type_fact(DatabaseType::Boolean, nullable))
            }
            ExpressionKind::Unary {
                operator,
                expression: inner,
            } => {
                let fact = self.infer(inner, frames, expected)?;
                if operator == "NOT" && fact.database_type != DatabaseType::Boolean {
                    return Err(PostgresAnalysisError::new(
                        PostgresDiagnosticCode::TypeMismatch,
                        "NOT needs a boolean operand",
                        expression,
                    ));
                }
                if matches!(operator.as_str(), "+" | "-") && !is_numeric(&fact.database_type) {
                    return Err(PostgresAnalysisError::new(
                        PostgresDiagnosticCode::UnknownOperator,
                        format!(
                            "PostgreSQL unary {operator} needs a numeric operand, found {:?}",
                            fact.database_type
                        ),
                        expression,
                    ));
                }
                Ok(fact)
            }
            ExpressionKind::BooleanList { expressions, .. } => {
                let mut nullable = false;
                for child in expressions {
                    let fact = self.infer(child, frames, Some(&DatabaseType::Boolean))?;
                    if fact.database_type != DatabaseType::Boolean {
                        return Err(PostgresAnalysisError::new(
                            PostgresDiagnosticCode::TypeMismatch,
                            "boolean expression contains a non-boolean value",
                            child,
                        ));
                    }
                    nullable |= fact.nullable;
                }
                Ok(type_fact(DatabaseType::Boolean, nullable))
            }
            ExpressionKind::Function {
                name,
                arguments,
                aggregate_star,
                distinct,
                filter,
                window,
            } => {
                let short = name.last().map(String::as_str).unwrap_or_default();
                let window_only = matches!(
                    short,
                    "row_number"
                        | "rank"
                        | "dense_rank"
                        | "lag"
                        | "lead"
                        | "first_value"
                        | "last_value"
                );
                let aggregate_function = matches!(short, "count" | "sum" | "avg" | "min" | "max")
                    || self
                        .catalog
                        .functions(name)
                        .iter()
                        .any(|function| function.aggregate);
                if aggregate_function {
                    self.required_capabilities
                        .insert("sql.query.aggregate".to_string());
                }
                if window.is_some() {
                    self.required_capabilities
                        .insert("sql.query.window".to_string());
                }
                if window_only && window.is_none() {
                    return Err(PostgresAnalysisError::new(
                        PostgresDiagnosticCode::InvalidResult,
                        format!("PostgreSQL window function '{short}' needs an OVER clause"),
                        expression,
                    ));
                }
                if window.is_some() && !window_only && !aggregate_function {
                    return Err(PostgresAnalysisError::new(
                        PostgresDiagnosticCode::InvalidResult,
                        format!("PostgreSQL function '{short}' cannot use an OVER clause"),
                        expression,
                    ));
                }
                if (*distinct || filter.is_some()) && !aggregate_function {
                    return Err(PostgresAnalysisError::new(
                        PostgresDiagnosticCode::InvalidResult,
                        format!("PostgreSQL function '{short}' cannot use aggregate modifiers"),
                        expression,
                    ));
                }
                if let Some(filter) = filter {
                    self.require_boolean(filter, frames)?;
                }
                if let Some(window) = window {
                    for item in &window.partition_by {
                        self.infer(item, frames, None)?;
                    }
                    for item in &window.order_by {
                        self.infer(&item.expression, frames, None)?;
                    }
                    if let Some(offset) = &window.start_offset {
                        self.infer(offset, frames, None)?;
                    }
                    if let Some(offset) = &window.end_offset {
                        self.infer(offset, frames, None)?;
                    }
                }
                self.infer_function(name, arguments, *aggregate_star, frames, expression)
            }
            ExpressionKind::Array { elements } => {
                let Some(first) = elements.first() else {
                    return Err(PostgresAnalysisError::new(
                        PostgresDiagnosticCode::TypeMismatch,
                        "an empty PostgreSQL array needs an explicit cast",
                        expression,
                    ));
                };
                let first = self.infer(first, frames, None)?;
                let mut element_nullable = first.nullable;
                for element in &elements[1..] {
                    let next = self.infer(element, frames, Some(&first.database_type))?;
                    element_nullable |= next.nullable;
                }
                Ok(type_fact(
                    DatabaseType::Array {
                        element: Box::new(first.database_type),
                        dimensions: Some(1),
                        element_nullability: if element_nullable {
                            Nullability::Nullable
                        } else {
                            Nullability::NonNull
                        },
                        preserves_lower_bounds: true,
                    },
                    false,
                ))
            }
            ExpressionKind::Case {
                operand,
                branches,
                fallback,
            } => {
                self.required_capabilities
                    .insert("sql.expression.case".to_string());
                let operand_fact = operand
                    .as_deref()
                    .map(|value| self.infer(value, frames, None))
                    .transpose()?;
                let mut result: Option<TypeFact> = None;
                for branch in branches {
                    if let Some(operand) = &operand_fact {
                        self.infer(&branch.condition, frames, Some(&operand.database_type))?;
                    } else {
                        self.require_boolean(&branch.condition, frames)?;
                    }
                    let branch_frames =
                        refine_for_null_test(self.catalog, frames, &branch.condition, true);
                    let fact = self.infer(
                        &branch.result,
                        &branch_frames,
                        result.as_ref().map(|fact| &fact.database_type).or(expected),
                    )?;
                    if let Some(current) = &mut result {
                        if current.database_type != fact.database_type {
                            return Err(PostgresAnalysisError::new(
                                PostgresDiagnosticCode::TypeMismatch,
                                "CASE branches have incompatible PostgreSQL types",
                                &branch.result,
                            ));
                        }
                        current.nullable |= fact.nullable;
                    } else {
                        result = Some(fact);
                    }
                }
                if let Some(fallback) = fallback {
                    let fallback_frames = if branches.len() == 1 {
                        refine_for_null_test(self.catalog, frames, &branches[0].condition, false)
                    } else {
                        frames.to_vec()
                    };
                    let fact = self.infer(
                        fallback,
                        &fallback_frames,
                        result.as_ref().map(|fact| &fact.database_type).or(expected),
                    )?;
                    if let Some(current) = &mut result {
                        if current.database_type != fact.database_type {
                            return Err(PostgresAnalysisError::new(
                                PostgresDiagnosticCode::TypeMismatch,
                                "CASE fallback has an incompatible PostgreSQL type",
                                fallback,
                            ));
                        }
                        current.nullable |= fact.nullable;
                    } else {
                        result = Some(fact);
                    }
                } else if let Some(current) = &mut result {
                    current.nullable = true;
                }
                result.ok_or_else(|| {
                    PostgresAnalysisError::new(
                        PostgresDiagnosticCode::TypeMismatch,
                        "CASE has no result expression",
                        expression,
                    )
                })
            }
            ExpressionKind::Coalesce { arguments } => {
                let Some(first) = arguments.first() else {
                    return Err(PostgresAnalysisError::new(
                        PostgresDiagnosticCode::TypeMismatch,
                        "COALESCE needs an argument",
                        expression,
                    ));
                };
                let mut result = self.infer(first, frames, expected)?;
                let mut all_nullable = result.nullable;
                for argument in &arguments[1..] {
                    let next = self.infer(argument, frames, Some(&result.database_type))?;
                    if next.database_type != result.database_type {
                        return Err(PostgresAnalysisError::new(
                            PostgresDiagnosticCode::TypeMismatch,
                            "COALESCE arguments have incompatible PostgreSQL types",
                            argument,
                        ));
                    }
                    all_nullable &= next.nullable;
                }
                result.nullable = all_nullable;
                Ok(result)
            }
            ExpressionKind::NullTest {
                expression: inner, ..
            } => {
                self.infer(inner, frames, None)?;
                Ok(type_fact(DatabaseType::Boolean, false))
            }
            ExpressionKind::Subquery { query } => {
                self.required_capabilities
                    .insert("sql.query.subquery".to_string());
                let analyzed = self.analyze_select(query, frames.to_vec())?;
                if analyzed.fields.len() != 1 {
                    return Err(PostgresAnalysisError::new(
                        PostgresDiagnosticCode::InvalidResult,
                        "scalar subquery must return exactly one column",
                        expression,
                    ));
                }
                let field = &analyzed.fields[0];
                Ok(TypeFact {
                    database_type: field.database_type.clone(),
                    nullable: field.nullable || analyzed.cardinality != Cardinality::EXACTLY_ONE,
                    source_object: field.source_object.clone(),
                    name_hint: Some(field.name.clone()),
                })
            }
            ExpressionKind::Exists { query } => {
                self.required_capabilities
                    .insert("sql.query.subquery".to_string());
                self.analyze_select(query, frames.to_vec())?;
                Ok(type_fact(DatabaseType::Boolean, false))
            }
            ExpressionKind::SubqueryComparison {
                operator,
                left,
                query,
                ..
            } => {
                self.required_capabilities
                    .insert("sql.query.subquery".to_string());
                let analyzed = self.analyze_select(query, frames.to_vec())?;
                if analyzed.fields.len() != 1 {
                    return Err(PostgresAnalysisError::new(
                        PostgresDiagnosticCode::InvalidResult,
                        "quantified subquery must return exactly one column",
                        expression,
                    ));
                }
                let field = &analyzed.fields[0];
                let left = self.infer(left, frames, Some(&field.database_type))?;
                if !matches!(operator.as_str(), "=" | "<>" | "<" | ">" | "<=" | ">=")
                    || !(self
                        .catalog
                        .can_cast(&left.database_type, &field.database_type, true)
                        || self
                            .catalog
                            .can_cast(&field.database_type, &left.database_type, true))
                {
                    return Err(PostgresAnalysisError::new(
                        PostgresDiagnosticCode::UnknownOperator,
                        "quantified subquery comparison has incompatible operands",
                        expression,
                    ));
                }
                Ok(type_fact(
                    DatabaseType::Boolean,
                    left.nullable || field.nullable,
                ))
            }
            ExpressionKind::Default => {
                expected
                    .cloned()
                    .map(|ty| type_fact(ty, false))
                    .ok_or_else(|| {
                        PostgresAnalysisError::new(
                            PostgresDiagnosticCode::InvalidWrite,
                            "DEFAULT is valid only in an assignment context",
                            expression,
                        )
                    })
            }
        }
    }
}

pub(crate) fn write_error(message: impl Into<String>) -> PostgresAnalysisError {
    PostgresAnalysisError::at_start(PostgresDiagnosticCode::InvalidWrite, message)
}

pub(crate) fn type_error(message: impl Into<String>) -> PostgresAnalysisError {
    PostgresAnalysisError::at_start(PostgresDiagnosticCode::TypeMismatch, message)
}

fn cte_names(
    cte: &crate::ast::CommonTableExpression,
    fields: &[ResultFact],
) -> Result<Vec<String>, PostgresAnalysisError> {
    if cte.columns.is_empty() {
        return Ok(fields.iter().map(|field| field.name.clone()).collect());
    }
    if cte.columns.len() != fields.len() {
        return Err(PostgresAnalysisError::at_start(
            PostgresDiagnosticCode::InvalidResult,
            format!("CTE '{}' column list has the wrong width", cte.name),
        ));
    }
    Ok(cte.columns.clone())
}
