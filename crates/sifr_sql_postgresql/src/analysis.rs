use crate::ast::{
    Expression, ExpressionKind, FromItem, JoinKind, SelectItem, SelectStatement, SetOperator,
    StatementKind,
};
use crate::catalog::{CatalogColumn, PostgresCatalog};
use crate::diagnostic::{PostgresDiagnostic, PostgresDiagnosticCode};
use crate::raw_adapter::{PostgresParseError, PostgresParser};
use crate::scope::{binding_for_relation, frame_for_results, resolve_column};
use crate::semantic_helpers::{
    expression_has_aggregate, integer_type, integer64_type, is_numeric, limit_is_one, text_type,
    type_fact, unique_result_names,
};
use sifr_sql_contract::{
    Cardinality, DatabaseType, DialectSemantics, EffectContract, Nullability, ObjectId,
    ProviderAnalysis, ProviderAnalysisError, ProviderParameter, ProviderResultField, QueryEffect,
    canonical_read_type_with_nullability_in,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub struct PostgresAnalyzer<P> {
    parser: P,
    catalog: PostgresCatalog,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostgresAnalysisError {
    pub diagnostic: PostgresDiagnostic,
}

impl PostgresAnalysisError {
    pub(crate) fn new(
        code: PostgresDiagnosticCode,
        message: impl Into<String>,
        expression: &Expression,
    ) -> Self {
        Self {
            diagnostic: PostgresDiagnostic::at_sql(
                code,
                message,
                expression.span.start,
                expression.span.end,
            ),
        }
    }

    pub(crate) fn at_start(code: PostgresDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            diagnostic: PostgresDiagnostic::at_sql(code, message, 0, 1),
        }
    }

    fn with_sifr_span(mut self, document: &str, start: u32, end: u32) -> Self {
        self.diagnostic = self
            .diagnostic
            .with_sifr_span(document.to_string(), start, end);
        self
    }
}

impl fmt::Display for PostgresAnalysisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.diagnostic.message)
    }
}

impl std::error::Error for PostgresAnalysisError {}

impl From<PostgresParseError> for PostgresAnalysisError {
    fn from(value: PostgresParseError) -> Self {
        Self {
            diagnostic: value.diagnostic,
        }
    }
}

impl<P: PostgresParser> PostgresAnalyzer<P> {
    #[must_use]
    pub fn new(parser: P, catalog: PostgresCatalog) -> Self {
        Self { parser, catalog }
    }

    pub fn analyze_query(&self, source: &str) -> Result<ProviderAnalysis, PostgresAnalysisError> {
        self.analyze_query_with_sifr_span(source, "sifr://unknown", 0, 0)
    }

    pub fn analyze_query_with_sifr_span(
        &self,
        source: &str,
        sifr_document: &str,
        sifr_start: u32,
        sifr_end: u32,
    ) -> Result<ProviderAnalysis, PostgresAnalysisError> {
        self.analyze_query_inner(source)
            .map_err(|error| error.with_sifr_span(sifr_document, sifr_start, sifr_end))
    }

    fn analyze_query_inner(&self, source: &str) -> Result<ProviderAnalysis, PostgresAnalysisError> {
        let statements = self.parser.parse(source)?;
        if statements.len() != 1 {
            return Err(PostgresAnalysisError::at_start(
                PostgresDiagnosticCode::UnsupportedCoreSyntax,
                "a reusable PostgreSQL query must contain exactly one statement",
            ));
        }
        let statement = &statements[0];
        let mut context = AnalysisContext::new(&self.catalog);
        let analyzed = match &statement.kind {
            StatementKind::Select(select) => context.analyze_select(select, Vec::new())?,
            StatementKind::Insert(insert) => context.analyze_insert(insert)?,
            StatementKind::Update(update) => context.analyze_update(update)?,
            StatementKind::Delete(delete) => context.analyze_delete(delete)?,
            _ => {
                return Err(PostgresAnalysisError::at_start(
                    PostgresDiagnosticCode::UnsupportedCoreSyntax,
                    "DDL is valid only in a schema profile source",
                ));
            }
        };
        let codecs = self
            .catalog
            .types
            .codec_registry()
            .map_err(|error| type_error(error.to_string()))?;
        let parameters = context
            .finish_parameters()?
            .into_iter()
            .map(|(slot, database_type)| {
                Ok(ProviderParameter {
                    slot,
                    codec: self
                        .catalog
                        .types
                        .codec_identity(&database_type)
                        .map_err(|error| type_error(error.to_string()))?,
                    database_type,
                    nullability: Nullability::NonNull,
                })
            })
            .collect::<Result<Vec<_>, PostgresAnalysisError>>()?;
        let result_fields = analyzed
            .fields
            .into_iter()
            .map(|field| {
                let nullability = if field.nullable {
                    Nullability::Nullable
                } else {
                    Nullability::NonNull
                };
                Ok(ProviderResultField {
                    name: field.name,
                    sifr_type: canonical_read_type_with_nullability_in(
                        &field.database_type,
                        nullability,
                        &codecs,
                    )
                    .map_err(|error| type_error(error.to_string()))?,
                    codec: self
                        .catalog
                        .types
                        .codec_identity(&field.database_type)
                        .map_err(|error| type_error(error.to_string()))?,
                    database_type: field.database_type,
                    nullability,
                    source_object: field.source_object,
                })
            })
            .collect::<Result<Vec<_>, PostgresAnalysisError>>()?;
        let analysis = ProviderAnalysis {
            server_profile: self.catalog.types.server_profile().to_string(),
            normalized_statement: self.parser.normalize(source)?,
            parameters,
            result_fields,
            cardinality: analyzed.cardinality,
            effects: EffectContract::new(analyzed.effect, analyzed.referenced, analyzed.affected)
                .map_err(|error| type_error(error.to_string()))?,
            semantic_flags: analyzed.flags,
        };
        analysis
            .validate(&codecs)
            .map_err(|error| type_error(error.to_string()))?;
        Ok(analysis)
    }

    #[must_use]
    pub fn catalog(&self) -> &PostgresCatalog {
        &self.catalog
    }
}

impl<P: PostgresParser> DialectSemantics for PostgresAnalyzer<P> {
    fn family(&self) -> &'static str {
        "postgresql"
    }

    fn analyze(
        &self,
        schema_fingerprint: &str,
        source: &str,
    ) -> Result<ProviderAnalysis, ProviderAnalysisError> {
        if schema_fingerprint != self.catalog.schema_fingerprint {
            return Err(ProviderAnalysisError::InvalidDialectSemantics);
        }
        self.analyze_query(source)
            .map_err(|_| ProviderAnalysisError::InvalidDialectSemantics)
    }
}

#[derive(Clone)]
pub(crate) struct ScopeBinding {
    pub(crate) alias: String,
    pub(crate) relation: Option<ObjectId>,
    pub(crate) columns: BTreeMap<String, CatalogColumn>,
}

#[derive(Clone, Default)]
pub(crate) struct ScopeFrame {
    pub(crate) bindings: Vec<ScopeBinding>,
}

#[derive(Clone)]
pub(crate) struct TypeFact {
    pub(crate) database_type: DatabaseType,
    pub(crate) nullable: bool,
    pub(crate) source_object: Option<ObjectId>,
    pub(crate) name_hint: Option<String>,
}

#[derive(Clone)]
pub(crate) struct ResultFact {
    pub(crate) name: String,
    pub(crate) database_type: DatabaseType,
    pub(crate) nullable: bool,
    pub(crate) source_object: Option<ObjectId>,
}

pub(crate) struct AnalyzedStatement {
    pub(crate) fields: Vec<ResultFact>,
    pub(crate) cardinality: Cardinality,
    pub(crate) effect: QueryEffect,
    pub(crate) referenced: BTreeSet<ObjectId>,
    pub(crate) affected: BTreeSet<ObjectId>,
    pub(crate) flags: BTreeSet<String>,
}

pub(crate) struct AnalysisContext<'a> {
    pub(crate) catalog: &'a PostgresCatalog,
    parameters: BTreeMap<u32, DatabaseType>,
    pub(crate) referenced: BTreeSet<ObjectId>,
}

impl<'a> AnalysisContext<'a> {
    fn new(catalog: &'a PostgresCatalog) -> Self {
        Self {
            catalog,
            parameters: BTreeMap::new(),
            referenced: BTreeSet::new(),
        }
    }

    fn finish_parameters(self) -> Result<Vec<(u32, DatabaseType)>, PostgresAnalysisError> {
        let mut output = Vec::with_capacity(self.parameters.len());
        for (index, (number, ty)) in self.parameters.into_iter().enumerate() {
            let expected = u32::try_from(index).unwrap_or(u32::MAX).saturating_add(1);
            if number != expected {
                return Err(PostgresAnalysisError::at_start(
                    PostgresDiagnosticCode::InvalidParameter,
                    "PostgreSQL parameters must form the contiguous sequence $1, $2, ...",
                ));
            }
            output.push((number - 1, ty));
        }
        Ok(output)
    }

    pub(crate) fn analyze_select(
        &mut self,
        select: &SelectStatement,
        outer: Vec<ScopeFrame>,
    ) -> Result<AnalyzedStatement, PostgresAnalysisError> {
        if let Some(set) = &select.set_operation {
            let left = self.analyze_select(&set.left, outer.clone())?;
            let right = self.analyze_select(&set.right, outer)?;
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
            let mut flags = BTreeSet::from([match set.operator {
                SetOperator::Union => "set-union",
                SetOperator::Intersect => "set-intersect",
                SetOperator::Except => "set-except",
            }
            .to_string()]);
            if !select.order_by.is_empty() {
                flags.insert("deterministic-order".to_string());
            }
            return Ok(AnalyzedStatement {
                fields,
                cardinality: if limit_is_one(select.limit.as_ref()) {
                    Cardinality::AT_MOST_ONE
                } else {
                    left.cardinality.join(right.cardinality)
                },
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
            self.require_boolean(predicate, &frames)?;
        }
        let mut fields = if select.values.is_empty() {
            self.result_fields(&select.targets, &frames)?
        } else {
            self.value_fields(&select.values, &frames)?
        };
        unique_result_names(&mut fields)?;
        let mut alias_frames = frames.clone();
        alias_frames.push(frame_for_results(&fields));
        for expression in &select.group_by {
            self.infer(expression, &alias_frames, None)?;
        }
        if let Some(having) = &select.having {
            self.require_boolean(having, &frames)?;
        }
        for order in &select.order_by {
            self.infer(&order.expression, &alias_frames, None)?;
        }
        if let Some(limit) = &select.limit {
            self.infer(limit, &[], Some(&integer64_type()))?;
        }
        let aggregate = select
            .targets
            .iter()
            .any(|target| expression_has_aggregate(&target.expression));
        let cardinality = if aggregate && select.group_by.is_empty() {
            Cardinality::EXACTLY_ONE
        } else if limit_is_one(select.limit.as_ref()) {
            Cardinality::AT_MOST_ONE
        } else {
            Cardinality::MANY
        };
        Ok(AnalyzedStatement {
            fields,
            cardinality,
            effect: QueryEffect::Read,
            referenced: self.referenced.clone(),
            affected: BTreeSet::new(),
            flags: if select.order_by.is_empty() {
                BTreeSet::new()
            } else {
                BTreeSet::from(["deterministic-order".to_string()])
            },
        })
    }

    fn scope_from(
        &mut self,
        items: &[FromItem],
        outer: &[ScopeFrame],
    ) -> Result<ScopeFrame, PostgresAnalysisError> {
        let mut frame = ScopeFrame::default();
        for item in items {
            self.add_from_item(item, outer, &mut frame)?;
        }
        Ok(frame)
    }

    pub(crate) fn add_from_item(
        &mut self,
        item: &FromItem,
        outer: &[ScopeFrame],
        frame: &mut ScopeFrame,
    ) -> Result<(), PostgresAnalysisError> {
        match item {
            FromItem::Relation { name, alias, .. } => {
                let relation = self
                    .catalog
                    .relation(name)
                    .map_err(|diagnostic| PostgresAnalysisError { diagnostic })?;
                self.referenced.insert(relation.identity.clone());
                frame
                    .bindings
                    .push(binding_for_relation(relation, alias.as_deref()));
            }
            FromItem::Subquery {
                query,
                alias,
                lateral,
                ..
            } => {
                let mut scopes = outer.to_vec();
                if *lateral {
                    scopes.push(frame.clone());
                }
                let analyzed = self.analyze_select(query, scopes)?;
                frame.bindings.push(ScopeBinding {
                    alias: alias.clone(),
                    relation: None,
                    columns: analyzed
                        .fields
                        .into_iter()
                        .enumerate()
                        .map(|(index, field)| {
                            let identity = ObjectId::new(format!("derived.{alias}.{index}"));
                            (
                                field.name.clone(),
                                CatalogColumn {
                                    identity,
                                    name: field.name,
                                    database_type: field.database_type,
                                    nullable: field.nullable,
                                    has_default: false,
                                    generated: false,
                                    source: None,
                                },
                            )
                        })
                        .collect(),
                });
            }
            FromItem::Join {
                join,
                left,
                right,
                condition,
                using_columns,
                ..
            } => {
                if !matches!(join, JoinKind::Inner | JoinKind::Cross) {
                    return Err(PostgresAnalysisError::at_start(
                        PostgresDiagnosticCode::UnsupportedCoreSyntax,
                        "exact outer-join nullability belongs to PostgreSQL semantic completion",
                    ));
                }
                self.add_from_item(left, outer, frame)?;
                self.add_from_item(right, outer, frame)?;
                if let Some(condition) = condition {
                    let mut scopes = outer.to_vec();
                    scopes.push(frame.clone());
                    self.require_boolean(condition, &scopes)?;
                }
                for column in using_columns {
                    let matches = frame
                        .bindings
                        .iter()
                        .filter(|binding| binding.columns.contains_key(column))
                        .count();
                    if matches != 2 {
                        return Err(PostgresAnalysisError::at_start(
                            PostgresDiagnosticCode::UnknownColumn,
                            format!("JOIN USING column '{column}' must exist on both sides"),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn result_fields(
        &mut self,
        targets: &[SelectItem],
        frames: &[ScopeFrame],
    ) -> Result<Vec<ResultFact>, PostgresAnalysisError> {
        targets
            .iter()
            .enumerate()
            .map(|(index, target)| {
                let fact = self.infer(&target.expression, frames, None)?;
                Ok(ResultFact {
                    name: target
                        .alias
                        .clone()
                        .or(fact.name_hint)
                        .unwrap_or_else(|| format!("column_{}", index + 1)),
                    database_type: fact.database_type,
                    nullable: fact.nullable,
                    source_object: fact.source_object,
                })
            })
            .collect()
    }

    fn value_fields(
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

    #[allow(clippy::too_many_lines)]
    pub(crate) fn infer(
        &mut self,
        expression: &Expression,
        frames: &[ScopeFrame],
        expected: Option<&DatabaseType>,
    ) -> Result<TypeFact, PostgresAnalysisError> {
        match &expression.kind {
            ExpressionKind::Column { path } => {
                resolve_column(self.catalog, path, frames, expression)
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
            ExpressionKind::String { .. } => Ok(type_fact(text_type(), false)),
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
                        format!("unknown PostgreSQL type '{}'", ty.join(".")),
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
            } => self.infer_binary(operator, left, right, frames, expression),
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
            } => self.infer_function(name, arguments, *aggregate_star, frames, expression),
            ExpressionKind::NullTest {
                expression: inner, ..
            } => {
                self.infer(inner, frames, None)?;
                Ok(type_fact(DatabaseType::Boolean, false))
            }
            ExpressionKind::Subquery { query } => {
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
                    nullable: true,
                    source_object: field.source_object.clone(),
                    name_hint: Some(field.name.clone()),
                })
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

    fn infer_binary(
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
            && matches!(left_fact.database_type, DatabaseType::Text { .. })
            && matches!(right_fact.database_type, DatabaseType::Text { .. })
        {
            return Ok(type_fact(text_type(), nullable));
        }
        if matches!(operator, "+" | "-" | "*" | "/")
            && left_fact.database_type == right_fact.database_type
            && is_numeric(&left_fact.database_type)
        {
            return Ok(type_fact(left_fact.database_type, nullable));
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

    fn infer_function(
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
            if !matches!(argument.database_type, DatabaseType::Text { .. }) {
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

pub(crate) fn write_error(message: impl Into<String>) -> PostgresAnalysisError {
    PostgresAnalysisError::at_start(PostgresDiagnosticCode::InvalidWrite, message)
}

fn type_error(message: impl Into<String>) -> PostgresAnalysisError {
    PostgresAnalysisError::at_start(PostgresDiagnosticCode::TypeMismatch, message)
}
