use crate::analysis::{AnalysisContext, PostgresAnalysisError, StarExpansion, type_error};
use crate::ast::StatementKind;
use crate::catalog::PostgresCatalog;
use crate::diagnostic::PostgresDiagnostic;
use crate::raw_adapter::PostgresParser;
use sifr_sql_contract::{
    DialectSemantics, EffectContract, Nullability, ProviderAnalysis, ProviderAnalysisError,
    ProviderDiagnosticSpan, ProviderParameter, ProviderResultField, ProviderSemanticDiagnostic,
    canonical_read_type_with_nullability_in,
};
use std::collections::BTreeSet;

pub struct PostgresAnalyzer<P> {
    parser: P,
    catalog: PostgresCatalog,
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
                crate::PostgresDiagnosticCode::UnsupportedCoreSyntax,
                "a reusable PostgreSQL query must contain exactly one statement",
            ));
        }
        let statement = &statements[0];
        let mut context = AnalysisContext::new(&self.catalog);
        let mut analyzed = match &statement.kind {
            StatementKind::Select(select) => {
                context
                    .required_capabilities
                    .insert("sql.query.select".to_string());
                context.analyze_select(select, Vec::new())?
            }
            StatementKind::Insert(insert) => {
                context
                    .required_capabilities
                    .insert("sql.query.insert".to_string());
                context.analyze_insert(insert)?
            }
            StatementKind::Update(update) => {
                context
                    .required_capabilities
                    .insert("sql.query.update".to_string());
                context.analyze_update(update)?
            }
            StatementKind::Delete(delete) => {
                context
                    .required_capabilities
                    .insert("sql.query.delete".to_string());
                context.analyze_delete(delete)?
            }
            _ => {
                return Err(PostgresAnalysisError::at_start(
                    crate::PostgresDiagnosticCode::UnsupportedCoreSyntax,
                    "DDL is valid only in a schema profile source",
                ));
            }
        };
        let explicit_source = expand_private_stars(source, context.star_expansions.values())?;
        if !context.star_expansions.is_empty() {
            analyzed.flags.insert("expanded-select-star".to_string());
        }
        let mut required_capabilities = context.required_capabilities.clone();
        let mut accessed_objects = context.accessed_objects.clone();
        let parameter_types = context.finish_parameters()?;
        if !parameter_types.is_empty() {
            required_capabilities.insert("sql.bind.parameters".to_string());
        }
        let used_types = parameter_types
            .iter()
            .map(|(_, database_type)| database_type)
            .chain(analyzed.fields.iter().map(|field| &field.database_type))
            .collect::<BTreeSet<_>>();
        let codecs = self
            .catalog
            .types
            .codec_registry_for(used_types)
            .map_err(|error| type_error(error.to_string()))?;
        let parameters = parameter_types
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
        accessed_objects.extend(analyzed.referenced.iter().cloned());
        accessed_objects.extend(analyzed.affected.iter().cloned());
        accessed_objects.extend(
            result_fields
                .iter()
                .filter_map(|field| field.source_object.clone()),
        );
        let analysis = ProviderAnalysis {
            server_profile: self.catalog.types.server_profile().to_string(),
            normalized_statement: self.parser.normalize(&explicit_source)?,
            parameters,
            result_fields,
            cardinality: analyzed.cardinality,
            effects: EffectContract::new(analyzed.effect, analyzed.referenced, analyzed.affected)
                .map_err(|error| type_error(error.to_string()))?,
            accessed_objects,
            semantic_flags: analyzed.flags,
            required_capabilities,
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

fn expand_private_stars<'a>(
    source: &str,
    expansions: impl IntoIterator<Item = &'a StarExpansion>,
) -> Result<String, PostgresAnalysisError> {
    let mut expansions = expansions.into_iter().collect::<Vec<_>>();
    expansions.sort_by_key(|expansion| std::cmp::Reverse((expansion.start, expansion.end)));
    let mut output = source.to_string();
    let mut prior_start = source.len();
    for expansion in expansions {
        let start = usize::try_from(expansion.start).unwrap_or(usize::MAX);
        let end = usize::try_from(expansion.end).unwrap_or(usize::MAX);
        if start >= end
            || end > prior_start
            || end > output.len()
            || !output.is_char_boundary(start)
            || !output.is_char_boundary(end)
            || expansion.columns.is_empty()
        {
            return Err(PostgresAnalysisError::at_start(
                crate::PostgresDiagnosticCode::InvalidResult,
                "SELECT * source span cannot be expanded safely",
            ));
        }
        let qualifier = expansion
            .qualifier
            .as_ref()
            .map(|value| format!("{}.", quote_identifier(value)))
            .unwrap_or_default();
        let replacement = expansion
            .columns
            .iter()
            .map(|column| format!("{qualifier}{}", quote_identifier(column)))
            .collect::<Vec<_>>()
            .join(", ");
        output.replace_range(start..end, &replacement);
        prior_start = start;
    }
    Ok(output)
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
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
            .map_err(|error| provider_diagnostic(error.diagnostic))
    }
}

fn provider_diagnostic(diagnostic: PostgresDiagnostic) -> ProviderAnalysisError {
    fn span(value: crate::diagnostic::PostgresDiagnosticSpan) -> ProviderDiagnosticSpan {
        ProviderDiagnosticSpan {
            kind: match value.kind {
                crate::diagnostic::PostgresSpanKind::Sifr => "sifr",
                crate::diagnostic::PostgresSpanKind::VirtualSql => "virtual-sql",
                crate::diagnostic::PostgresSpanKind::Schema => "schema",
            }
            .to_string(),
            document: value.document,
            start: value.start,
            end: value.end,
            label: value.label,
        }
    }
    ProviderAnalysisError::Diagnostic(Box::new(ProviderSemanticDiagnostic {
        code: diagnostic.code.as_str().to_string(),
        message: diagnostic.message,
        primary: span(diagnostic.primary),
        related: diagnostic.related.into_iter().map(span).collect(),
    }))
}
