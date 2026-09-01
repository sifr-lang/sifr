use crate::ast::{CreateTableStatement, PostgresStatement, TableConstraint};
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
    }
    for constraint in &input.table.constraints {
        match constraint {
            TableConstraint::ForeignKey {
                columns,
                relation,
                referenced,
            } => identities.push(writer.add_foreign_key(columns, relation, referenced)),
            TableConstraint::Check { expression } => {
                let serialized = crate::canonical_postgres_ast_json(expression).map_err(|_| {
                    PostgresDiagnostic::at_sql(
                        PostgresDiagnosticCode::TypeMismatch,
                        "cannot serialize PostgreSQL CHECK expression",
                        expression.span.start,
                        expression.span.end,
                    )
                })?;
                identities.push(writer.add_constraint(
                    stable_constraint_name(
                        input.table_identity,
                        "check",
                        std::slice::from_ref(&serialized),
                    ),
                    SchemaObjectKind::CheckConstraint,
                    BTreeMap::from([(
                        "provider-expression".to_string(),
                        SemanticValue::Text(serialized),
                    )]),
                    BTreeSet::from([input.table_identity.clone()]),
                ));
            }
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
