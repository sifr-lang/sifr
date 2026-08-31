use rusqlite::Connection;
use sifr_sql_contract::{DialectIdentity, ProviderIdentity, SchemaIr, normalize_schema};
use sifr_sql_sqlite::{
    SUPPORTED_SQLITE_SERIES, SqliteParser, SqliteSchemaOptions, normalize_sqlite_catalog_documents,
};
use std::collections::BTreeSet;
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
    let connection = Connection::open(path).map_err(|_| error("cannot open the SQLite catalog"))?;
    pull_live_catalog_from_connection(&connection, provider, dialect)
}

pub fn pull_live_catalog_from_connection(
    connection: &Connection,
    provider: ProviderIdentity,
    dialect: DialectIdentity,
) -> Result<SchemaIr, SqliteCatalogError> {
    if dialect.family != "sqlite" || dialect.server_version != "3.53.2" {
        return Err(error(
            "catalog reflection requires the qualified SQLite 3.53.2 dialect",
        ));
    }
    connection
        .pragma_update(None, "trusted_schema", false)
        .map_err(|_| error("cannot secure SQLite catalog reflection"))?;
    let schemas = database_names(connection)?;
    let mut sources = Vec::new();
    for schema in &schemas {
        let schema_sql = quote_identifier(schema);
        let query = format!(
            "SELECT sql FROM {schema_sql}.sqlite_schema WHERE sql IS NOT NULL \
             AND name NOT LIKE 'sqlite_%' AND name != 'sifr_migration_ledger' \
             ORDER BY type, name"
        );
        let mut statement = connection
            .prepare(&query)
            .map_err(|_| error("cannot prepare SQLite schema reflection"))?;
        let definitions = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|_| error("cannot read SQLite schema"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| error("SQLite schema row is malformed"))?;
        if !definitions.is_empty() {
            sources.push((
                schema.clone(),
                format!("sifr://sqlite/live-catalog/{schema}"),
                definitions.join(";\n"),
            ));
        }
    }
    let parser = SqliteParser::new(SUPPORTED_SQLITE_SERIES[0], dialect.modes.clone())
        .map_err(|failure| error(failure.to_string()))?;
    let attached_schemas = schemas
        .iter()
        .filter(|name| name.as_str() != "main")
        .cloned()
        .collect::<BTreeSet<_>>();
    let options = SqliteSchemaOptions {
        default_schema: "main".to_string(),
        compile_flags: dialect.modes.clone(),
        attached_schemas,
        required_features: dialect.features.clone(),
        extensions: BTreeSet::new(),
    };
    if sources.is_empty() {
        sources.push((
            "main".to_string(),
            "sifr://sqlite/live-catalog/main".to_string(),
            String::new(),
        ));
    }
    let output = normalize_sqlite_catalog_documents(provider.clone(), &parser, &options, sources)
        .map_err(|failure| error(failure.to_string()))?;
    normalize_schema(provider, dialect, output.documents)
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

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn error(message: impl Into<String>) -> SqliteCatalogError {
    SqliteCatalogError {
        message: message.into(),
    }
}
