use rusqlite::Connection;
use serde_json::to_string;
use sifr_sql_contract::{
    DatabaseType, DialectIdentity, ObjectId, ProviderIdentity, SchemaDocument, SchemaDocumentKind,
    SchemaIr, SchemaObject, SchemaObjectKind, SemanticValue, SqliteStorageClass,
};
use sifr_sql_sqlite::{SqliteAffinity, affinity};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqliteCatalogError {
    pub message: String,
}

impl fmt::Display for SqliteCatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SqliteCatalogError {}

pub async fn pull_live_catalog(
    path: &str,
    provider: ProviderIdentity,
    dialect: DialectIdentity,
) -> Result<SchemaIr, SqliteCatalogError> {
    pull_live_catalog_from_path(Path::new(path), provider, dialect)
}

pub fn pull_live_catalog_from_path(
    path: &Path,
    provider: ProviderIdentity,
    dialect: DialectIdentity,
) -> Result<SchemaIr, SqliteCatalogError> {
    if dialect.family != "sqlite" || dialect.server_version != "3.53.2" {
        return Err(error(
            "catalog reflection requires the qualified SQLite 3.53.2 dialect",
        ));
    }
    let connection = Connection::open(path).map_err(|_| error("cannot open the SQLite catalog"))?;
    pull_live_catalog_from_connection(&connection, provider, dialect)
}

pub fn pull_live_catalog_from_connection(
    connection: &Connection,
    provider: ProviderIdentity,
    dialect: DialectIdentity,
) -> Result<SchemaIr, SqliteCatalogError> {
    connection
        .pragma_update(None, "trusted_schema", false)
        .map_err(|_| error("cannot secure SQLite catalog reflection"))?;
    let mut objects = BTreeMap::new();
    for schema in database_names(connection)? {
        let namespace = ObjectId::new(&schema);
        objects.insert(
            namespace.clone(),
            SchemaObject {
                identity: namespace,
                kind: SchemaObjectKind::Namespace,
                semantic: BTreeMap::new(),
                dependencies: BTreeSet::new(),
                source: None,
            },
        );
        reflect_schema(connection, &schema, &mut objects)?;
    }
    sifr_sql_contract::normalize_schema(
        provider,
        dialect,
        vec![SchemaDocument {
            kind: SchemaDocumentKind::ProviderMetadata,
            document: "sifr://sqlite/live-catalog".to_string(),
            objects: objects.into_values().collect(),
        }],
    )
    .map_err(|failure| error(failure.to_string()))
}

fn database_names(connection: &Connection) -> Result<Vec<String>, SqliteCatalogError> {
    let mut statement = connection
        .prepare("PRAGMA database_list")
        .map_err(|_| error("cannot prepare SQLite database reflection"))?;
    let names = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|_| error("cannot read SQLite database list"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| error("SQLite database list is malformed"))?;
    Ok(names.into_iter().filter(|name| name != "temp").collect())
}

fn reflect_schema(
    connection: &Connection,
    schema: &str,
    objects: &mut BTreeMap<ObjectId, SchemaObject>,
) -> Result<(), SqliteCatalogError> {
    let schema_sql = quote_identifier(schema);
    let query = format!(
        "SELECT name, type, sql FROM {schema_sql}.sqlite_schema \
         WHERE name NOT LIKE 'sqlite_%' ORDER BY type, name"
    );
    let mut statement = connection
        .prepare(&query)
        .map_err(|_| error("cannot prepare SQLite schema reflection"))?;
    let entries = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })
        .map_err(|_| error("cannot read SQLite schema"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| error("SQLite schema row is malformed"))?;
    for (name, kind, definition) in entries {
        match kind.as_str() {
            "table" => reflect_table(connection, schema, &name, definition, objects)?,
            "view" => {
                insert_named_object(schema, &name, SchemaObjectKind::View, definition, objects);
            }
            "index" => insert_index(schema, &name, definition, objects),
            "trigger" => insert_named_object(
                schema,
                &name,
                SchemaObjectKind::Trigger,
                definition,
                objects,
            ),
            _ => return Err(error("SQLite returned an unknown schema object kind")),
        }
    }
    Ok(())
}

fn reflect_table(
    connection: &Connection,
    schema: &str,
    table: &str,
    definition: Option<String>,
    objects: &mut BTreeMap<ObjectId, SchemaObject>,
) -> Result<(), SqliteCatalogError> {
    let table_identity = format!("{schema}.{table}");
    let table_literal = quote_pragma_argument(table);
    let schema_sql = quote_identifier(schema);
    let query = format!("PRAGMA {schema_sql}.table_xinfo({table_literal})");
    let mut statement = connection
        .prepare(&query)
        .map_err(|_| error("cannot inspect SQLite table"))?;
    let columns = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)? != 0,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
            ))
        })
        .map_err(|_| error("cannot read SQLite table columns"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| error("SQLite column metadata is malformed"))?;
    let strict = definition
        .as_deref()
        .is_some_and(|sql| sql.trim_end().to_ascii_uppercase().ends_with("STRICT"));
    let without_rowid = definition
        .as_deref()
        .is_some_and(|sql| sql.to_ascii_uppercase().contains("WITHOUT ROWID"));
    let mut column_ids = Vec::new();
    let mut primary_key = Vec::new();
    for (name, declared_type, not_null, default, key_position, hidden) in columns {
        let identity = ObjectId::new(format!("{table_identity}.{name}"));
        column_ids.push(identity.clone());
        if key_position > 0 {
            primary_key.push((key_position, name.clone()));
        }
        let database_type = database_type(&declared_type, strict);
        let generated = hidden == 2 || hidden == 3;
        let rowid_alias =
            !without_rowid && declared_type.eq_ignore_ascii_case("INTEGER") && key_position == 1;
        objects.insert(
            identity.clone(),
            SchemaObject {
                identity,
                kind: if rowid_alias {
                    SchemaObjectKind::IdentityColumn
                } else {
                    SchemaObjectKind::Column
                },
                semantic: BTreeMap::from([
                    ("name".to_string(), SemanticValue::Text(name)),
                    (
                        "database-type".to_string(),
                        SemanticValue::Text(
                            to_string(&database_type)
                                .map_err(|_| error("cannot encode SQLite type"))?,
                        ),
                    ),
                    (
                        "sqlite-type".to_string(),
                        SemanticValue::Text(declared_type.clone()),
                    ),
                    (
                        "affinity".to_string(),
                        SemanticValue::Text(
                            format!("{:?}", affinity(&declared_type)).to_ascii_lowercase(),
                        ),
                    ),
                    ("nullable".to_string(), SemanticValue::Bool(!not_null)),
                    ("generated".to_string(), SemanticValue::Bool(generated)),
                    (
                        "generated-stored".to_string(),
                        SemanticValue::Bool(hidden == 3),
                    ),
                    (
                        "default".to_string(),
                        SemanticValue::Text(default.unwrap_or_default()),
                    ),
                ]),
                dependencies: BTreeSet::from([ObjectId::new(&table_identity)]),
                source: None,
            },
        );
    }
    primary_key.sort_by_key(|(position, _)| *position);
    let primary_names = primary_key
        .into_iter()
        .map(|(_, name)| name)
        .collect::<Vec<_>>();
    if !primary_names.is_empty() {
        let identity = ObjectId::new(format!("{table_identity}.primary_key"));
        objects.insert(
            identity.clone(),
            SchemaObject {
                identity,
                kind: SchemaObjectKind::PrimaryKey,
                semantic: BTreeMap::from([("columns".to_string(), string_list(&primary_names))]),
                dependencies: BTreeSet::from([ObjectId::new(&table_identity)]),
                source: None,
            },
        );
    }
    objects.insert(
        ObjectId::new(&table_identity),
        SchemaObject {
            identity: ObjectId::new(&table_identity),
            kind: SchemaObjectKind::Table,
            semantic: BTreeMap::from([
                (
                    "columns".to_string(),
                    SemanticValue::List(
                        column_ids
                            .iter()
                            .map(|id| SemanticValue::Text(id.as_str().to_string()))
                            .collect(),
                    ),
                ),
                (
                    "primary-key".to_string(),
                    SemanticValue::Set(
                        primary_names.into_iter().map(SemanticValue::Text).collect(),
                    ),
                ),
                ("strict".to_string(), SemanticValue::Bool(strict)),
                (
                    "without-rowid".to_string(),
                    SemanticValue::Bool(without_rowid),
                ),
                (
                    "definition".to_string(),
                    SemanticValue::Text(definition.unwrap_or_default()),
                ),
            ]),
            dependencies: BTreeSet::from([ObjectId::new(schema)]),
            source: None,
        },
    );
    reflect_foreign_keys(connection, schema, table, &table_identity, objects)?;
    Ok(())
}

fn reflect_foreign_keys(
    connection: &Connection,
    schema: &str,
    table: &str,
    table_identity: &str,
    objects: &mut BTreeMap<ObjectId, SchemaObject>,
) -> Result<(), SqliteCatalogError> {
    let query = format!(
        "PRAGMA {}.foreign_key_list({})",
        quote_identifier(schema),
        quote_pragma_argument(table)
    );
    let mut statement = connection
        .prepare(&query)
        .map_err(|_| error("cannot inspect SQLite foreign keys"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })
        .map_err(|_| error("cannot read SQLite foreign keys"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| error("SQLite foreign-key metadata is malformed"))?;
    let mut groups: BTreeMap<i64, (String, Vec<String>, Vec<String>)> = BTreeMap::new();
    for (id, referenced, from, to) in rows {
        let group = groups
            .entry(id)
            .or_insert_with(|| (referenced, Vec::new(), Vec::new()));
        group.1.push(from);
        group.2.push(to.unwrap_or_default());
    }
    for (id, (referenced, from, to)) in groups {
        let identity = ObjectId::new(format!("{table_identity}.foreign_key_{id}"));
        let referenced_identity = format!("{schema}.{referenced}");
        objects.insert(
            identity.clone(),
            SchemaObject {
                identity,
                kind: SchemaObjectKind::ForeignKey,
                semantic: BTreeMap::from([
                    ("columns".to_string(), string_list(&from)),
                    (
                        "referenced-table".to_string(),
                        SemanticValue::Text(referenced_identity.clone()),
                    ),
                    ("referenced-columns".to_string(), string_list(&to)),
                ]),
                dependencies: BTreeSet::from([
                    ObjectId::new(table_identity),
                    ObjectId::new(referenced_identity),
                ]),
                source: None,
            },
        );
    }
    Ok(())
}

fn insert_named_object(
    schema: &str,
    name: &str,
    kind: SchemaObjectKind,
    definition: Option<String>,
    objects: &mut BTreeMap<ObjectId, SchemaObject>,
) {
    let identity = ObjectId::new(format!("{schema}.{name}"));
    objects.insert(
        identity.clone(),
        SchemaObject {
            identity,
            kind,
            semantic: BTreeMap::from([(
                "definition".to_string(),
                SemanticValue::Text(definition.unwrap_or_default()),
            )]),
            dependencies: BTreeSet::from([ObjectId::new(schema)]),
            source: None,
        },
    );
}

fn insert_index(
    schema: &str,
    name: &str,
    definition: Option<String>,
    objects: &mut BTreeMap<ObjectId, SchemaObject>,
) {
    insert_named_object(schema, name, SchemaObjectKind::Index, definition, objects);
}

fn database_type(declared: &str, strict: bool) -> DatabaseType {
    if strict {
        match declared.to_ascii_uppercase().as_str() {
            "INT" | "INTEGER" => DatabaseType::Integer {
                sign: sifr_sql_contract::IntegerSign::Signed,
                width: sifr_sql_contract::IntegerWidth::Bits64,
            },
            "REAL" => DatabaseType::Float64,
            "TEXT" => DatabaseType::Text {
                fixed: false,
                max_characters: None,
            },
            "BLOB" => DatabaseType::Binary { max_bytes: None },
            _ => dynamic_type(),
        }
    } else {
        match affinity(declared) {
            SqliteAffinity::Integer => DatabaseType::Integer {
                sign: sifr_sql_contract::IntegerSign::Signed,
                width: sifr_sql_contract::IntegerWidth::Bits64,
            },
            SqliteAffinity::Real => DatabaseType::Float64,
            SqliteAffinity::Text => DatabaseType::Text {
                fixed: false,
                max_characters: None,
            },
            SqliteAffinity::Blob => DatabaseType::Binary { max_bytes: None },
            SqliteAffinity::Numeric => dynamic_type(),
        }
    }
}

fn dynamic_type() -> DatabaseType {
    DatabaseType::SqliteDynamic {
        storage_classes: BTreeSet::from([
            SqliteStorageClass::Integer,
            SqliteStorageClass::Real,
            SqliteStorageClass::Text,
            SqliteStorageClass::Blob,
            SqliteStorageClass::Null,
        ]),
    }
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn quote_pragma_argument(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn string_list(values: &[String]) -> SemanticValue {
    SemanticValue::List(values.iter().cloned().map(SemanticValue::Text).collect())
}

fn error(message: impl Into<String>) -> SqliteCatalogError {
    SqliteCatalogError {
        message: message.into(),
    }
}
