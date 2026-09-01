use crate::ast::{MysqlCreateTable, MysqlStatementKind};
use crate::component::mysql_capabilities;
use crate::lower_hex;
use crate::parser::{MysqlParseError, MysqlParser};
use crate::types::mysql_type;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sifr_sql_contract::{
    DialectIdentity, ObjectId, ProviderIdentity, SchemaDocument, SchemaDocumentKind,
    SchemaNormalizationOutput, SchemaObject, SchemaObjectKind, SchemaSourceLocation, SemanticValue,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MysqlSchemaOptions {
    pub default_database: String,
    pub default_character_set: String,
    pub default_collation: String,
    pub sql_modes: BTreeSet<String>,
    pub extensions: BTreeSet<String>,
}

impl MysqlSchemaOptions {
    pub fn validate(&self) -> Result<(), MysqlParseError> {
        for (label, value) in [
            ("database", self.default_database.as_str()),
            ("character set", self.default_character_set.as_str()),
            ("collation", self.default_collation.as_str()),
        ] {
            if !canonical_identifier(value) {
                return Err(MysqlParseError {
                    offset: 0,
                    message: format!("MySQL default {label} is invalid"),
                });
            }
        }
        if self.sql_modes.iter().any(|mode| {
            mode.is_empty()
                || !mode
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte == b'_')
        }) {
            return Err(MysqlParseError {
                offset: 0,
                message: "MySQL SQL mode is invalid".to_string(),
            });
        }
        Ok(())
    }
}

pub fn normalize_mysql_documents(
    provider: ProviderIdentity,
    parser: &MysqlParser,
    options: &MysqlSchemaOptions,
    documents: Vec<(String, String)>,
) -> Result<SchemaNormalizationOutput, MysqlParseError> {
    options.validate()?;
    if parser.sql_modes() != &options.sql_modes
        || parser.default_character_set() != options.default_character_set
        || parser.default_collation() != options.default_collation
    {
        return Err(MysqlParseError {
            offset: 0,
            message: "MySQL parser settings and schema options differ".to_string(),
        });
    }
    if documents.is_empty() {
        return Err(MysqlParseError {
            offset: 0,
            message: "MySQL schema normalization needs at least one document".to_string(),
        });
    }
    let metadata = metadata_document(options).objects;
    let mut normalized = Vec::with_capacity(documents.len());
    let mut seen = BTreeSet::new();
    for (document_index, (document, source)) in documents.into_iter().enumerate() {
        if document.is_empty() || !seen.insert(document.clone()) {
            return Err(MysqlParseError {
                offset: 0,
                message: "MySQL schema document identity is empty or duplicated".to_string(),
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
                MysqlStatementKind::CreateTable(table) => {
                    objects.extend(table_objects(&document, &table, options)?);
                }
                MysqlStatementKind::CreateView(view) => {
                    let identity = qualify(&view.name, &options.default_database);
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
                MysqlStatementKind::CreateIndex(index) => {
                    let relation = index.relation.ok_or_else(|| MysqlParseError {
                        offset: statement.span.start as usize,
                        message: "CREATE INDEX needs an owning table".to_string(),
                    })?;
                    let table = qualify(&relation, &options.default_database);
                    let name = index.name.last().cloned().ok_or_else(|| MysqlParseError {
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
                    return Err(MysqlParseError {
                        offset: statement.span.start as usize,
                        message: "schema sources accept only MySQL CREATE statements".to_string(),
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
        family: "mysql".to_string(),
        server_version: parser.series().version(),
        modes: options
            .sql_modes
            .iter()
            .cloned()
            .chain([
                format!("character-set:{}", options.default_character_set),
                format!("collation:{}", options.default_collation),
            ])
            .collect(),
        features: options.extensions.clone(),
    };
    sifr_sql_contract::normalize_schema(provider, dialect.clone(), normalized.clone()).map_err(
        |error| MysqlParseError {
            offset: 0,
            message: error.to_string(),
        },
    )?;
    Ok(SchemaNormalizationOutput {
        dialect,
        capabilities: mysql_capabilities(),
        documents: normalized,
    })
}

fn metadata_document(options: &MysqlSchemaOptions) -> SchemaDocument {
    let database = ObjectId::new(&options.default_database);
    let charset = ObjectId::new(format!("mysql.charset.{}", options.default_character_set));
    let collation = ObjectId::new(format!("mysql.collation.{}", options.default_collation));
    let mode_identity = ObjectId::new("mysql.dialect.settings");
    SchemaDocument {
        kind: SchemaDocumentKind::ProviderMetadata,
        document: "sifr://mysql/profile-metadata".to_string(),
        objects: vec![
            SchemaObject {
                identity: database,
                kind: SchemaObjectKind::Namespace,
                semantic: BTreeMap::new(),
                dependencies: BTreeSet::new(),
                source: None,
            },
            SchemaObject {
                identity: charset.clone(),
                kind: SchemaObjectKind::CharacterSet,
                semantic: BTreeMap::from([(
                    "name".to_string(),
                    SemanticValue::Text(options.default_character_set.clone()),
                )]),
                dependencies: BTreeSet::new(),
                source: None,
            },
            SchemaObject {
                identity: collation,
                kind: SchemaObjectKind::Collation,
                semantic: BTreeMap::from([(
                    "name".to_string(),
                    SemanticValue::Text(options.default_collation.clone()),
                )]),
                dependencies: BTreeSet::from([charset]),
                source: None,
            },
            SchemaObject {
                identity: mode_identity,
                kind: SchemaObjectKind::DialectMetadata,
                semantic: BTreeMap::from([
                    (
                        "sql-modes".to_string(),
                        SemanticValue::Set(
                            options
                                .sql_modes
                                .iter()
                                .cloned()
                                .map(SemanticValue::Text)
                                .collect(),
                        ),
                    ),
                    (
                        "collation".to_string(),
                        SemanticValue::Text(options.default_collation.clone()),
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
    table: &MysqlCreateTable,
    options: &MysqlSchemaOptions,
) -> Result<Vec<SchemaObject>, MysqlParseError> {
    let table_identity = qualify(&table.name, &options.default_database);
    let table_collation = effective_table_collation(table, options)?;
    let mut objects = Vec::new();
    let mut columns = Vec::new();
    for column in &table.columns {
        let identity = ObjectId::new(format!("{table_identity}.{}", column.name));
        let ty = mysql_type(&column.ty, &identity)
            .map_err(|message| MysqlParseError { offset: 0, message })?;
        let database_type = serde_json::to_string(&ty.database).map_err(|_| MysqlParseError {
            offset: 0,
            message: "cannot serialize canonical MySQL type".to_string(),
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
                    "mysql-type".to_string(),
                    SemanticValue::Text(ty.canonical_name),
                ),
                ("nullable".to_string(), SemanticValue::Bool(column.nullable)),
                (
                    "unsigned".to_string(),
                    SemanticValue::Bool(column.ty.unsigned),
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
                            .unwrap_or_else(|| table_collation.clone()),
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
        signature.push(qualify(
            &foreign.referenced_table,
            &options.default_database,
        ));
        signature.extend(foreign.referenced_columns.clone());
        let identity = constraint_identity(&table_identity, "foreign_key", &signature);
        constraint_ids.push(identity.clone());
        let referenced_table = qualify(&foreign.referenced_table, &options.default_database);
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
            (
                "character-set".to_string(),
                SemanticValue::Text(
                    table
                        .default_character_set
                        .clone()
                        .unwrap_or_else(|| options.default_character_set.clone()),
                ),
            ),
            (
                "collation".to_string(),
                SemanticValue::Text(table_collation),
            ),
        ]),
        dependencies: BTreeSet::from([ObjectId::new(namespace(&table_identity))]),
        source: Some(source(document)),
    });
    Ok(objects)
}

fn effective_table_collation(
    table: &MysqlCreateTable,
    options: &MysqlSchemaOptions,
) -> Result<String, MysqlParseError> {
    if let Some(collation) = &table.default_collation {
        return Ok(collation.clone());
    }
    let Some(charset) = table.default_character_set.as_deref() else {
        return Ok(options.default_collation.clone());
    };
    let collation = match charset {
        "utf8mb4" => "utf8mb4_0900_ai_ci",
        "utf8mb3" | "utf8" => "utf8mb3_general_ci",
        "latin1" => "latin1_swedish_ci",
        "ascii" => "ascii_general_ci",
        "binary" => "binary",
        value if value == options.default_character_set => options.default_collation.as_str(),
        _ => {
            return Err(MysqlParseError {
                offset: 0,
                message: format!("MySQL character set '{charset}' has no locked default collation"),
            });
        }
    };
    Ok(collation.to_string())
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
