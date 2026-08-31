use crate::ast::{SqliteCreateTable, SqliteStatementKind};
use crate::component::sqlite_capabilities;
use crate::lower_hex;
use crate::parser::{SqliteParseError, SqliteParser};
use crate::types::sqlite_type;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sifr_sql_contract::{
    DialectIdentity, ObjectId, ProviderIdentity, SchemaDocument, SchemaDocumentKind,
    SchemaNormalizationOutput, SchemaObject, SchemaObjectKind, SchemaSourceLocation, SemanticValue,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteSchemaOptions {
    pub default_schema: String,
    pub compile_flags: BTreeSet<String>,
    pub attached_schemas: BTreeSet<String>,
    pub required_features: BTreeSet<String>,
    pub extensions: BTreeSet<String>,
}

impl SqliteSchemaOptions {
    pub fn validate(&self) -> Result<(), SqliteParseError> {
        if self.default_schema != "main" || self.attached_schemas.contains("main") {
            return Err(SqliteParseError {
                offset: 0,
                message:
                    "SQLite uses 'main' as the default schema and attached names must be distinct"
                        .to_string(),
            });
        }
        if self
            .attached_schemas
            .iter()
            .any(|name| !canonical_identifier(name) || name == "temp")
        {
            return Err(SqliteParseError {
                offset: 0,
                message: "SQLite attached schema name is invalid or reserved".to_string(),
            });
        }
        if !self.compile_flags.is_empty() {
            return Err(SqliteParseError {
                offset: 0,
                message: "SQLite compile flags differ from the qualified bundled build".to_string(),
            });
        }
        Ok(())
    }
}

pub fn normalize_sqlite_documents(
    provider: ProviderIdentity,
    parser: &SqliteParser,
    options: &SqliteSchemaOptions,
    documents: Vec<(String, String)>,
) -> Result<SchemaNormalizationOutput, SqliteParseError> {
    options.validate()?;
    if parser.compile_flags() != &options.compile_flags {
        return Err(SqliteParseError {
            offset: 0,
            message: "SQLite parser settings and schema options differ".to_string(),
        });
    }
    if documents.is_empty() {
        return Err(SqliteParseError {
            offset: 0,
            message: "SQLite schema normalization needs at least one document".to_string(),
        });
    }
    let metadata = metadata_document(options).objects;
    let mut normalized = Vec::with_capacity(documents.len());
    let mut seen = BTreeSet::new();
    for (document_index, (document, source)) in documents.into_iter().enumerate() {
        if document.is_empty() || !seen.insert(document.clone()) {
            return Err(SqliteParseError {
                offset: 0,
                message: "SQLite schema document identity is empty or duplicated".to_string(),
            });
        }
        let statements = if source.trim().is_empty() {
            Vec::new()
        } else {
            parser.parse(&source)?
        };
        let mut objects = if document_index == 0 {
            metadata.clone()
        } else {
            Vec::new()
        };
        for statement in statements {
            match statement.kind {
                SqliteStatementKind::CreateTable(table) => {
                    objects.extend(table_objects(&document, &table, options)?);
                }
                SqliteStatementKind::CreateView(view) => {
                    let identity = qualify(&view.name, &options.default_schema);
                    objects.push(SchemaObject {
                        identity: ObjectId::new(&identity),
                        kind: SchemaObjectKind::View,
                        semantic: BTreeMap::from([(
                            "definition".to_string(),
                            SemanticValue::Text(view.definition),
                        )]),
                        dependencies: BTreeSet::from([ObjectId::new(namespace(&identity))]),
                        source: Some(SchemaSourceLocation {
                            document: document.clone(),
                            start: statement.span.start,
                            end: statement.span.end,
                        }),
                    });
                }
                SqliteStatementKind::CreateIndex(index) => {
                    let relation = index.relation.ok_or_else(|| SqliteParseError {
                        offset: statement.span.start as usize,
                        message: "CREATE INDEX needs an owning table".to_string(),
                    })?;
                    let table = qualify(&relation, &options.default_schema);
                    let name = index.name.last().cloned().ok_or_else(|| SqliteParseError {
                        offset: statement.span.start as usize,
                        message: "CREATE INDEX needs a name".to_string(),
                    })?;
                    let identity = format!("{table}.index_{name}");
                    objects.push(SchemaObject {
                        identity: ObjectId::new(identity),
                        kind: SchemaObjectKind::Index,
                        semantic: BTreeMap::from([(
                            "definition".to_string(),
                            SemanticValue::Text(index.definition),
                        )]),
                        dependencies: BTreeSet::from([ObjectId::new(table)]),
                        source: Some(SchemaSourceLocation {
                            document: document.clone(),
                            start: statement.span.start,
                            end: statement.span.end,
                        }),
                    });
                }
                _ => {
                    return Err(SqliteParseError {
                        offset: statement.span.start as usize,
                        message: "schema sources accept only SQLite CREATE statements".to_string(),
                    });
                }
            }
        }
        normalized.push(SchemaDocument {
            kind: SchemaDocumentKind::SqlDdl,
            document,
            objects,
        });
    }
    let dialect = DialectIdentity {
        family: "sqlite".to_string(),
        server_version: parser.series().version(),
        modes: options.compile_flags.clone(),
        features: options
            .extensions
            .union(&options.required_features)
            .cloned()
            .collect(),
    };
    sifr_sql_contract::normalize_schema(provider, dialect.clone(), normalized.clone()).map_err(
        |error| SqliteParseError {
            offset: 0,
            message: error.to_string(),
        },
    )?;
    Ok(SchemaNormalizationOutput {
        dialect,
        capabilities: sqlite_capabilities(),
        documents: normalized,
    })
}

fn metadata_document(options: &SqliteSchemaOptions) -> SchemaDocument {
    let database = ObjectId::new(&options.default_schema);
    let mode_identity = ObjectId::new("sqlite.dialect.settings");
    SchemaDocument {
        kind: SchemaDocumentKind::ProviderMetadata,
        document: "sifr://sqlite/profile-metadata".to_string(),
        objects: vec![
            SchemaObject {
                identity: database,
                kind: SchemaObjectKind::Namespace,
                semantic: BTreeMap::new(),
                dependencies: BTreeSet::new(),
                source: None,
            },
            SchemaObject {
                identity: mode_identity,
                kind: SchemaObjectKind::DialectMetadata,
                semantic: BTreeMap::from([
                    (
                        "compile-flags".to_string(),
                        SemanticValue::Set(
                            options
                                .compile_flags
                                .iter()
                                .cloned()
                                .map(SemanticValue::Text)
                                .collect(),
                        ),
                    ),
                    (
                        "minimum-version".to_string(),
                        SemanticValue::Text("3.53.2".to_string()),
                    ),
                ]),
                dependencies: BTreeSet::new(),
                source: None,
            },
        ],
    }
}

fn table_objects(
    document: &str,
    table: &SqliteCreateTable,
    options: &SqliteSchemaOptions,
) -> Result<Vec<SchemaObject>, SqliteParseError> {
    let table_identity = qualify(&table.name, &options.default_schema);
    let mut objects = Vec::new();
    let mut columns = Vec::new();
    for column in &table.columns {
        let identity = ObjectId::new(format!("{table_identity}.{}", column.name));
        let ty = sqlite_type(&column.ty, &identity, table.strict)
            .map_err(|message| SqliteParseError { offset: 0, message })?;
        let database_type = serde_json::to_string(&ty.database).map_err(|_| SqliteParseError {
            offset: 0,
            message: "cannot serialize canonical SQLite type".to_string(),
        })?;
        columns.push(identity.clone());
        objects.push(SchemaObject {
            identity,
            kind: if column.auto_increment {
                SchemaObjectKind::IdentityColumn
            } else {
                SchemaObjectKind::Column
            },
            semantic: BTreeMap::from([
                ("name".to_string(), SemanticValue::Text(column.name.clone())),
                (
                    "database-type".to_string(),
                    SemanticValue::Text(database_type),
                ),
                (
                    "sqlite-type".to_string(),
                    SemanticValue::Text(ty.declared_name),
                ),
                ("nullable".to_string(), SemanticValue::Bool(column.nullable)),
                (
                    "affinity".to_string(),
                    SemanticValue::Text(format!("{:?}", ty.affinity).to_ascii_lowercase()),
                ),
                (
                    "generated".to_string(),
                    SemanticValue::Bool(column.generated.is_some()),
                ),
                (
                    "generated-expression".to_string(),
                    SemanticValue::Text(
                        column
                            .generated
                            .as_ref()
                            .map_or_else(String::new, |value| value.expression.clone()),
                    ),
                ),
                (
                    "collation".to_string(),
                    SemanticValue::Text(
                        column
                            .collation
                            .clone()
                            .unwrap_or_else(|| "binary".to_string()),
                    ),
                ),
            ]),
            dependencies: BTreeSet::from([ObjectId::new(&table_identity)]),
            source: Some(SchemaSourceLocation {
                document: document.to_string(),
                start: 0,
                end: 0,
            }),
        });
    }
    let mut constraint_ids = Vec::new();
    if !table.primary_key.is_empty() {
        let identity = ObjectId::new(format!("{table_identity}.primary_key"));
        constraint_ids.push(identity.clone());
        objects.push(key_object(
            identity,
            SchemaObjectKind::PrimaryKey,
            &table.primary_key,
            &table_identity,
            document,
        ));
    }
    for unique in &table.unique_keys {
        let identity = constraint_identity(&table_identity, "unique", &unique.columns);
        constraint_ids.push(identity.clone());
        objects.push(key_object(
            identity,
            SchemaObjectKind::UniqueConstraint,
            &unique.columns,
            &table_identity,
            document,
        ));
    }
    for foreign in &table.foreign_keys {
        let mut signature = foreign.columns.clone();
        signature.push(qualify(&foreign.referenced_table, &options.default_schema));
        signature.extend(foreign.referenced_columns.clone());
        let identity = constraint_identity(&table_identity, "foreign_key", &signature);
        constraint_ids.push(identity.clone());
        let referenced_table = qualify(&foreign.referenced_table, &options.default_schema);
        objects.push(SchemaObject {
            identity,
            kind: SchemaObjectKind::ForeignKey,
            semantic: BTreeMap::from([
                ("columns".to_string(), string_list(&foreign.columns)),
                (
                    "referenced-table".to_string(),
                    SemanticValue::Text(referenced_table.clone()),
                ),
                (
                    "referenced-columns".to_string(),
                    string_list(&foreign.referenced_columns),
                ),
            ]),
            dependencies: BTreeSet::from([
                ObjectId::new(&table_identity),
                ObjectId::new(referenced_table),
            ]),
            source: Some(source(document)),
        });
    }
    for check in &table.checks {
        let identity = constraint_identity(&table_identity, "check", std::slice::from_ref(check));
        constraint_ids.push(identity.clone());
        objects.push(SchemaObject {
            identity,
            kind: SchemaObjectKind::CheckConstraint,
            semantic: BTreeMap::from([(
                "expression".to_string(),
                SemanticValue::Text(check.clone()),
            )]),
            dependencies: BTreeSet::from([ObjectId::new(&table_identity)]),
            source: Some(source(document)),
        });
    }
    let unique_sets = table
        .unique_keys
        .iter()
        .map(|key| string_list(&key.columns))
        .collect();
    objects.push(SchemaObject {
        identity: ObjectId::new(&table_identity),
        kind: SchemaObjectKind::Table,
        semantic: BTreeMap::from([
            (
                "columns".to_string(),
                SemanticValue::List(
                    columns
                        .iter()
                        .map(|identity| SemanticValue::Text(identity.as_str().to_string()))
                        .collect(),
                ),
            ),
            ("primary-key".to_string(), string_set(&table.primary_key)),
            ("unique-sets".to_string(), SemanticValue::List(unique_sets)),
            (
                "constraints".to_string(),
                SemanticValue::List(
                    constraint_ids
                        .iter()
                        .map(|identity| SemanticValue::Text(identity.as_str().to_string()))
                        .collect(),
                ),
            ),
            ("strict".to_string(), SemanticValue::Bool(table.strict)),
            (
                "without-rowid".to_string(),
                SemanticValue::Bool(table.without_rowid),
            ),
            (
                "rowid-alias".to_string(),
                SemanticValue::Bool(table.columns.iter().any(|column| column.auto_increment)),
            ),
        ]),
        dependencies: BTreeSet::from([ObjectId::new(namespace(&table_identity))]),
        source: Some(source(document)),
    });
    Ok(objects)
}

fn key_object(
    identity: ObjectId,
    kind: SchemaObjectKind,
    columns: &[String],
    table: &str,
    document: &str,
) -> SchemaObject {
    SchemaObject {
        identity,
        kind,
        semantic: BTreeMap::from([("columns".to_string(), string_list(columns))]),
        dependencies: BTreeSet::from([ObjectId::new(table)]),
        source: Some(source(document)),
    }
}

fn constraint_identity(table: &str, kind: &str, signature: &[String]) -> ObjectId {
    let mut digest = Sha256::new();
    digest.update(kind.as_bytes());
    for value in signature {
        digest.update([0]);
        digest.update(value.as_bytes());
    }
    let hex = lower_hex(&digest.finalize());
    ObjectId::new(format!("{table}.{kind}_{}", &hex[..16]))
}

fn qualify(path: &[String], default_database: &str) -> String {
    if path.len() > 1 {
        path.join(".")
    } else {
        format!(
            "{default_database}.{}",
            path.first().map_or("unknown", String::as_str)
        )
    }
}

fn namespace(identity: &str) -> String {
    identity.split('.').next().unwrap_or("unknown").to_string()
}

fn string_list(values: &[String]) -> SemanticValue {
    SemanticValue::List(values.iter().cloned().map(SemanticValue::Text).collect())
}

fn string_set(values: &[String]) -> SemanticValue {
    SemanticValue::Set(values.iter().cloned().map(SemanticValue::Text).collect())
}

fn source(document: &str) -> SchemaSourceLocation {
    SchemaSourceLocation {
        document: document.to_string(),
        start: 0,
        end: 0,
    }
}

fn canonical_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}
