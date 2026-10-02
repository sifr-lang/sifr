//! View fingerprints bind names with the query analyzer; spelling and source
//! locations are not identities. Binding aliases and scope depths retain the
//! distinction between separate uses of the same relation (including self joins).
use crate::analysis::{AnalysisContext, PostgresAnalysisError, ScopeFrame};
use crate::ast::{Expression, FromItem, SelectItem, SelectStatement};
use crate::catalog::PostgresCatalog;
use crate::diagnostic::PostgresDiagnosticCode;
use crate::scope::resolve_column_binding;
use serde::Serialize;
use serde_json::{Value, json};
use sifr_sql_contract::ObjectId;
use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct ViewBindings {
    replacements: BTreeMap<String, Value>,
}

impl ViewBindings {
    pub(crate) fn record_star(
        &mut self,
        target: &SelectItem,
        qualifier: &[String],
        frames: &[ScopeFrame],
    ) -> Result<(), PostgresAnalysisError> {
        let mut targets = Vec::new();
        for (depth, frame) in frames.iter().enumerate().rev().take(1) {
            for binding in &frame.bindings {
                if qualifier.last().is_some_and(|name| name != &binding.alias) {
                    continue;
                }
                if qualifier.len() > 1
                    && binding.relation.as_ref().map(ObjectId::as_str)
                        != Some(qualifier.join(".").as_str())
                {
                    return Err(invalid_view());
                }
                for name in &binding.column_order {
                    let column = binding.columns.get(name).ok_or_else(invalid_view)?;
                    targets.push(json!({"expression": {"kind": {"kind": "bound_column",
                        "identity": column.identity.as_str(), "binding": binding.alias, "scope": depth}}, "alias": null}));
                }
            }
        }
        self.insert(target, Value::Array(targets))
    }

    fn insert(
        &mut self,
        source: &impl Serialize,
        resolved: Value,
    ) -> Result<(), PostgresAnalysisError> {
        let key = serde_json::to_string(&serde_json::to_value(source).map_err(|_| invalid_view())?)
            .map_err(|_| invalid_view())?;
        if let Some(previous) = self.replacements.get(&key)
            && previous != &resolved
        {
            return Err(invalid_view());
        }
        self.replacements.insert(key, resolved);
        Ok(())
    }

    pub(crate) fn record_column(
        &mut self,
        catalog: &PostgresCatalog,
        path: &[String],
        frames: &[ScopeFrame],
        expression: &Expression,
    ) -> Result<(), PostgresAnalysisError> {
        let (binding, column, depth) = resolve_column_binding(catalog, path, frames, expression)?;
        // The general expression resolver accepts a relation qualifier. A view
        // fingerprint must additionally verify an explicit schema qualification
        // instead of dropping it and accidentally equating different schemas.
        if path.len() > 2 {
            let relation = binding.relation.as_ref().ok_or_else(invalid_view)?;
            let expected = path[..path.len() - 1].join(".");
            if path.len() != 3 || relation.as_str() != expected {
                return Err(PostgresAnalysisError::new(
                    PostgresDiagnosticCode::UnknownColumn,
                    "view column schema qualification does not match its resolved relation",
                    expression,
                ));
            }
        }
        self.insert(
            expression,
            json!({
                "kind": {"kind": "bound_column", "identity": column.identity.as_str(),
                    "binding": binding.alias, "scope": depth},
            }),
        )
    }

    pub(crate) fn record_relation(
        &mut self,
        item: &FromItem,
        identity: &ObjectId,
    ) -> Result<(), PostgresAnalysisError> {
        let FromItem::Relation { alias, .. } = item else {
            return Err(invalid_view());
        };
        self.insert(
            item,
            json!({"kind": "relation", "name": identity.as_str(), "alias": alias}),
        )
    }

    fn rewrite(&self, value: &mut Value) -> Result<(), PostgresAnalysisError> {
        let key = serde_json::to_string(value).map_err(|_| invalid_view())?;
        if let Some(replacement) = self.replacements.get(&key) {
            *value = replacement.clone();
            return Ok(());
        }
        if value
            .get("kind")
            .and_then(|kind| kind.get("kind"))
            .and_then(Value::as_str)
            .is_some_and(|kind| matches!(kind, "column" | "star"))
        {
            return Err(invalid_view());
        }
        match value {
            Value::Array(values) => {
                for value in values {
                    self.rewrite(value)?;
                }
            }
            Value::Object(values) => {
                // A wildcard is replaced by ordered, bound projection items;
                // flatten only the SELECT targets, never arbitrary SQL arrays.
                if let Some(Value::Array(targets)) = values.get_mut("targets") {
                    let mut expanded = Vec::new();
                    for target in std::mem::take(targets) {
                        let key = serde_json::to_string(&target).map_err(|_| invalid_view())?;
                        if let Some(Value::Array(replacements)) = self.replacements.get(&key) {
                            expanded.extend(replacements.clone());
                        } else {
                            expanded.push(target);
                        }
                    }
                    *targets = expanded;
                }
                if values
                    .get("alias")
                    .and_then(Value::as_str)
                    .is_some_and(|alias| {
                        values
                            .get("expression")
                            .and_then(|expression| expression.get("kind"))
                            .filter(|kind| {
                                kind.get("kind").and_then(Value::as_str) == Some("column")
                            })
                            .and_then(|kind| kind.get("path"))
                            .and_then(Value::as_array)
                            .and_then(|path| path.last())
                            .and_then(Value::as_str)
                            == Some(alias)
                    })
                {
                    values.insert("alias".to_string(), Value::Null);
                }
                if values.get("direction").and_then(Value::as_str) == Some("default") {
                    values.insert(
                        "direction".to_string(),
                        Value::String("ascending".to_string()),
                    );
                }
                if values.get("nulls").and_then(Value::as_str) == Some("default") {
                    let nulls =
                        if values.get("direction").and_then(Value::as_str) == Some("descending") {
                            "first"
                        } else {
                            "last"
                        };
                    values.insert("nulls".to_string(), Value::String(nulls.to_string()));
                }
                values.remove("span");
                for value in values.values_mut() {
                    self.rewrite(value)?;
                }
                if let Some(Value::Array(targets)) = values.get("targets") {
                    let expressions = targets
                        .iter()
                        .filter_map(|target| target.get("expression").cloned())
                        .collect::<Vec<_>>();
                    for key in ["order_by", "group_by"] {
                        if let Some(value) = values.get_mut(key) {
                            resolve_projection_references(value, &expressions)?;
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

fn resolve_projection_references(
    value: &mut Value,
    expressions: &[Value],
) -> Result<(), PostgresAnalysisError> {
    if let Some(kind) = value.get("kind")
        && kind.get("kind").and_then(Value::as_str) == Some("bound_column")
        && kind.get("binding").and_then(Value::as_str) == Some("<result>")
    {
        let index = kind
            .get("identity")
            .and_then(Value::as_str)
            .and_then(|identity| identity.strip_prefix("result."))
            .and_then(|index| index.parse::<usize>().ok())
            .ok_or_else(invalid_view)?;
        *value = expressions.get(index).ok_or_else(invalid_view)?.clone();
        return Ok(());
    }
    match value {
        Value::Array(values) => {
            for value in values {
                resolve_projection_references(value, expressions)?;
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                resolve_projection_references(value, expressions)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Analyze a parsed SELECT against the real provider catalog and serialize its
/// resolved view semantics. Invalid and ambiguous references fail closed.
pub fn canonical_postgres_view_json(
    query: &SelectStatement,
    catalog: &PostgresCatalog,
) -> Result<String, PostgresAnalysisError> {
    let mut context = AnalysisContext::new(catalog);
    context.view_bindings = Some(ViewBindings::default());
    context.analyze_select(query, Vec::new())?;
    serialize_view(query, &context)
}

pub(crate) fn serialize_view(
    query: &SelectStatement,
    context: &AnalysisContext<'_>,
) -> Result<String, PostgresAnalysisError> {
    let mut value = serde_json::to_value(query).map_err(|_| invalid_view())?;
    context
        .view_bindings
        .as_ref()
        .ok_or_else(invalid_view)?
        .rewrite(&mut value)?;
    serde_json::to_string(&value).map_err(|_| invalid_view())
}

fn invalid_view() -> PostgresAnalysisError {
    PostgresAnalysisError::at_start(
        PostgresDiagnosticCode::InvalidResult,
        "PostgreSQL view references cannot be canonicalized consistently",
    )
}
