use crate::ast::{
    CreateTableStatement, Expression, ExpressionKind, PostgresStatement, TableConstraint,
};
use crate::diagnostic::{PostgresDiagnostic, PostgresDiagnosticCode};
use sha2::{Digest, Sha256};
use sifr_sql_contract::{
    ObjectId, SchemaObject, SchemaObjectKind, SchemaSourceLocation, SemanticValue,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct TableConstraintInput<'a> {
    pub document: &'a str,
    pub statement: &'a PostgresStatement,
    pub table: &'a CreateTableStatement,
    pub table_identity: &'a ObjectId,
    pub column_ids: &'a [ObjectId],
    pub primary_key: &'a BTreeSet<String>,
    pub unique_sets: &'a BTreeSet<Vec<String>>,
}

pub(crate) fn add_table_constraints(
    input: &TableConstraintInput<'_>,
    objects: &mut BTreeMap<ObjectId, SchemaObject>,
) -> Result<Vec<ObjectId>, PostgresDiagnostic> {
    let mut writer = ConstraintWriter {
        document: input.document,
        statement: input.statement,
        table_identity: input.table_identity,
        column_ids: input.column_ids,
        objects,
    };
    let mut identities = Vec::new();
    if !input.primary_key.is_empty() {
        let columns = input.primary_key.iter().cloned().collect::<Vec<_>>();
        identities.push(writer.add_constraint(
            constraint_name(input.table_identity, "pkey"),
            SchemaObjectKind::PrimaryKey,
            columns_semantic(&columns),
            column_dependencies(input.table_identity, input.column_ids, &columns),
        ));
    }
    for columns in input.unique_sets {
        identities.push(writer.add_constraint(
            stable_constraint_name(input.table_identity, "unique", columns),
            SchemaObjectKind::UniqueConstraint,
            columns_semantic(columns),
            column_dependencies(input.table_identity, input.column_ids, columns),
        ));
    }
    for column in &input.table.columns {
        if let Some((relation, referenced)) = &column.references {
            identities.push(writer.add_foreign_key(
                std::slice::from_ref(&column.name),
                relation,
                referenced,
            ));
        }
        for expression in &column.checks {
            identities.push(writer.add_check(expression)?);
        }
    }
    for constraint in &input.table.constraints {
        match constraint {
            TableConstraint::ForeignKey {
                columns,
                relation,
                referenced,
            } => identities.push(writer.add_foreign_key(columns, relation, referenced)),
            TableConstraint::Check { expression } => identities.push(writer.add_check(expression)?),
            TableConstraint::PrimaryKey { .. } | TableConstraint::Unique { .. } => {}
        }
    }
    Ok(identities)
}

struct ConstraintWriter<'a> {
    document: &'a str,
    statement: &'a PostgresStatement,
    table_identity: &'a ObjectId,
    column_ids: &'a [ObjectId],
    objects: &'a mut BTreeMap<ObjectId, SchemaObject>,
}

impl ConstraintWriter<'_> {
    fn add_check(&mut self, expression: &Expression) -> Result<ObjectId, PostgresDiagnostic> {
        let serialized = crate::canonical_postgres_ast_json(expression).map_err(|_| {
            PostgresDiagnostic::at_sql(
                PostgresDiagnosticCode::TypeMismatch,
                "cannot serialize PostgreSQL CHECK expression",
                expression.span.start,
                expression.span.end,
            )
        })?;
        let columns = check_columns(expression).into_iter().collect::<Vec<_>>();
        Ok(self.add_constraint(
            stable_constraint_name(
                self.table_identity,
                "check",
                std::slice::from_ref(&serialized),
            ),
            SchemaObjectKind::CheckConstraint,
            BTreeMap::from([(
                "provider-expression".to_string(),
                SemanticValue::Text(serialized),
            )]),
            column_dependencies(self.table_identity, self.column_ids, &columns),
        ))
    }

    fn add_foreign_key(
        &mut self,
        columns: &[String],
        relation: &[String],
        referenced: &[String],
    ) -> ObjectId {
        let referenced_relation = ObjectId::new(qualified_name(relation));
        let mut dependencies = column_dependencies(self.table_identity, self.column_ids, columns);
        dependencies.insert(referenced_relation.clone());
        self.add_constraint(
            stable_constraint_name(
                self.table_identity,
                "fkey",
                &columns
                    .iter()
                    .chain(relation)
                    .chain(referenced)
                    .cloned()
                    .collect::<Vec<_>>(),
            ),
            SchemaObjectKind::ForeignKey,
            BTreeMap::from([
                ("columns".to_string(), string_list(columns)),
                (
                    "referenced-relation".to_string(),
                    SemanticValue::Text(referenced_relation.as_str().to_string()),
                ),
                ("referenced-columns".to_string(), string_list(referenced)),
            ]),
            dependencies,
        )
    }

    fn add_constraint(
        &mut self,
        identity: ObjectId,
        kind: SchemaObjectKind,
        semantic: BTreeMap<String, SemanticValue>,
        mut dependencies: BTreeSet<ObjectId>,
    ) -> ObjectId {
        dependencies.insert(self.table_identity.clone());
        self.objects.insert(
            identity.clone(),
            SchemaObject {
                identity: identity.clone(),
                kind,
                semantic,
                dependencies,
                source: Some(SchemaSourceLocation {
                    document: self.document.to_string(),
                    start: self.statement.span.start,
                    end: self.statement.span.end,
                }),
            },
        );
        identity
    }
}

fn check_columns(expression: &Expression) -> BTreeSet<String> {
    let mut columns = BTreeSet::new();
    collect_check_columns(expression, &mut columns);
    columns
}

fn collect_check_columns(expression: &Expression, columns: &mut BTreeSet<String>) {
    match &expression.kind {
        ExpressionKind::Column { path } => {
            if let Some(name) = path.last() {
                columns.insert(name.clone());
            }
        }
        ExpressionKind::Cast { expression, .. }
        | ExpressionKind::Unary { expression, .. }
        | ExpressionKind::NullTest { expression, .. } => {
            collect_check_columns(expression, columns);
        }
        ExpressionKind::Binary { left, right, .. } => {
            collect_check_columns(left, columns);
            collect_check_columns(right, columns);
        }
        ExpressionKind::InList {
            expression, values, ..
        } => {
            collect_check_columns(expression, columns);
            for value in values {
                collect_check_columns(value, columns);
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
                collect_check_columns(expression, columns);
            }
        }
        ExpressionKind::Function {
            arguments,
            filter,
            window,
            ..
        } => {
            for argument in arguments {
                collect_check_columns(argument, columns);
            }
            if let Some(filter) = filter {
                collect_check_columns(filter, columns);
            }
            if let Some(window) = window {
                for expression in &window.partition_by {
                    collect_check_columns(expression, columns);
                }
                for order in &window.order_by {
                    collect_check_columns(&order.expression, columns);
                }
                if let Some(offset) = &window.start_offset {
                    collect_check_columns(offset, columns);
                }
                if let Some(offset) = &window.end_offset {
                    collect_check_columns(offset, columns);
                }
            }
        }
        ExpressionKind::Case {
            operand,
            branches,
            fallback,
        } => {
            if let Some(operand) = operand {
                collect_check_columns(operand, columns);
            }
            for branch in branches {
                collect_check_columns(&branch.condition, columns);
                collect_check_columns(&branch.result, columns);
            }
            if let Some(fallback) = fallback {
                collect_check_columns(fallback, columns);
            }
        }
        ExpressionKind::SubqueryComparison { left, .. } => {
            collect_check_columns(left, columns);
        }
        ExpressionKind::Star { .. }
        | ExpressionKind::Parameter { .. }
        | ExpressionKind::Integer { .. }
        | ExpressionKind::Float { .. }
        | ExpressionKind::String { .. }
        | ExpressionKind::Boolean { .. }
        | ExpressionKind::Null
        | ExpressionKind::Subquery { .. }
        | ExpressionKind::Exists { .. }
        | ExpressionKind::Default => {}
    }
}

fn columns_semantic(columns: &[String]) -> BTreeMap<String, SemanticValue> {
    BTreeMap::from([("columns".to_string(), string_list(columns))])
}

fn column_dependencies(
    table_identity: &ObjectId,
    column_ids: &[ObjectId],
    columns: &[String],
) -> BTreeSet<ObjectId> {
    column_ids
        .iter()
        .filter(|identity| {
            identity
                .as_str()
                .rsplit('.')
                .next()
                .is_some_and(|name| columns.iter().any(|column| column == name))
        })
        .cloned()
        .chain(std::iter::once(table_identity.clone()))
        .collect()
}

fn constraint_name(table: &ObjectId, suffix: &str) -> ObjectId {
    let mut segments = table.as_str().rsplitn(2, '.');
    let relation = segments.next().unwrap_or("relation");
    let namespace = segments.next().unwrap_or("public");
    ObjectId::new(format!("{namespace}.{relation}_{suffix}"))
}

fn stable_constraint_name(table: &ObjectId, kind: &str, signature: &[String]) -> ObjectId {
    let mut digest = Sha256::new();
    digest.update(kind.as_bytes());
    for value in signature {
        digest.update([0]);
        digest.update(value.as_bytes());
    }
    let encoded = crate::component::lower_hex(&digest.finalize());
    constraint_name(table, &format!("{kind}_{}", &encoded[..16]))
}

fn qualified_name(name: &[String]) -> String {
    if name.len() > 1 {
        name.join(".")
    } else {
        format!(
            "public.{}",
            name.first().map(String::as_str).unwrap_or("unknown")
        )
    }
}

fn string_list(values: &[String]) -> SemanticValue {
    SemanticValue::List(values.iter().cloned().map(SemanticValue::Text).collect())
}
