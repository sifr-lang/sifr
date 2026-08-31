use mysql_async::{Conn, Opts, prelude::Queryable};
use sifr_sql_contract::{DialectIdentity, ProviderIdentity, SchemaIr, normalize_schema};
use sifr_sql_mysql::{
    MysqlParser, MysqlSchemaOptions, MysqlServerSeries, normalize_mysql_documents,
};
use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MysqlCatalogError {
    pub message: String,
}

impl fmt::Display for MysqlCatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for MysqlCatalogError {}

pub async fn pull_live_catalog(
    connection_url: &str,
    provider: ProviderIdentity,
    dialect: DialectIdentity,
) -> Result<SchemaIr, MysqlCatalogError> {
    let opts = Opts::from_url(connection_url)
        .map_err(|_| catalog_error("MySQL catalog connection configuration is invalid"))?;
    let mut connection = Conn::new(opts)
        .await
        .map_err(|_| catalog_error("cannot connect to the MySQL catalog"))?;
    let result = pull_live_catalog_from_connection(&mut connection, provider, dialect).await;
    let _disconnect = connection.disconnect().await;
    result
}

pub async fn pull_live_catalog_from_connection(
    connection: &mut Conn,
    provider: ProviderIdentity,
    dialect: DialectIdentity,
) -> Result<SchemaIr, MysqlCatalogError> {
    if dialect.family != "mysql" {
        return Err(catalog_error(
            "MySQL catalog pull requires a MySQL dialect identity",
        ));
    }
    let metadata: Option<(String, String, String, String, String)> = connection
        .query_first(
            "SELECT VERSION(), DATABASE(), @@session.sql_mode, \
             @@character_set_database, @@collation_database",
        )
        .await
        .map_err(|_| catalog_error("cannot read MySQL catalog settings"))?;
    let Some((version, database, modes, character_set, collation)) = metadata else {
        return Err(catalog_error("MySQL catalog settings are incomplete"));
    };
    let series = series_from_version(&version)?;
    if series.version() != dialect.server_version {
        return Err(catalog_error(format!(
            "MySQL server series {} does not match profile series {}",
            series.version(),
            dialect.server_version
        )));
    }
    let sql_modes = modes
        .split(',')
        .filter(|mode| !mode.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    verify_dialect_settings(&dialect, &sql_modes, &character_set, &collation)?;
    let before = table_names(connection, &database).await?;
    let mut documents = Vec::with_capacity(before.len());
    for table in &before {
        let statement = format!("SHOW CREATE TABLE {}", quote_identifier(table));
        let row: Option<(String, String)> = connection
            .query_first(statement)
            .await
            .map_err(|_| catalog_error(format!("cannot inspect MySQL table '{table}'")))?;
        let Some((_, ddl)) = row else {
            return Err(catalog_error(format!(
                "MySQL table '{table}' disappeared during catalog pull"
            )));
        };
        documents.push((format!("mysql://live-catalog/{table}"), ddl));
    }
    if documents.is_empty() {
        documents.push(("mysql://live-catalog/metadata".to_string(), String::new()));
    }
    if before != table_names(connection, &database).await? {
        return Err(catalog_error(
            "MySQL catalog changed during introspection; retry the schema pull",
        ));
    }
    let options = MysqlSchemaOptions {
        default_database: database,
        default_character_set: character_set,
        default_collation: collation.clone(),
        sql_modes: sql_modes.clone(),
        extensions: dialect.features.clone(),
    };
    let parser = MysqlParser::new(series, sql_modes, collation)
        .map_err(|error| catalog_error(error.to_string()))?;
    let output = normalize_mysql_documents(provider.clone(), &parser, &options, documents)
        .map_err(|error| catalog_error(error.to_string()))?;
    normalize_schema(provider, output.dialect, output.documents)
        .map_err(|error| catalog_error(error.to_string()))
}

async fn table_names(
    connection: &mut Conn,
    database: &str,
) -> Result<Vec<String>, MysqlCatalogError> {
    let mut names = connection
        .exec_map(
            "SELECT TABLE_NAME FROM information_schema.TABLES \
             WHERE TABLE_SCHEMA = ? AND TABLE_TYPE = 'BASE TABLE' \
             AND TABLE_NAME <> 'sifr_migration_ledger' ORDER BY TABLE_NAME",
            (database,),
            |name: String| name,
        )
        .await
        .map_err(|_| catalog_error("cannot list MySQL catalog tables"))?;
    names.sort();
    Ok(names)
}

fn series_from_version(version: &str) -> Result<MysqlServerSeries, MysqlCatalogError> {
    let mut numbers = version.split(['.', '-']);
    let major = numbers.next().and_then(|part| part.parse::<u16>().ok());
    let minor = numbers.next().and_then(|part| part.parse::<u16>().ok());
    match (major, minor) {
        (Some(8), Some(4)) => Ok(MysqlServerSeries::new(8, 4)),
        (Some(9), Some(7)) => Ok(MysqlServerSeries::new(9, 7)),
        (Some(26), Some(7)) => Ok(MysqlServerSeries::new(26, 7)),
        _ => Err(catalog_error(format!(
            "MySQL server version '{version}' is unsupported"
        ))),
    }
}

fn verify_dialect_settings(
    dialect: &DialectIdentity,
    modes: &BTreeSet<String>,
    character_set: &str,
    collation: &str,
) -> Result<(), MysqlCatalogError> {
    let observed = modes
        .iter()
        .cloned()
        .chain([
            format!("character-set:{character_set}"),
            format!("collation:{collation}"),
        ])
        .collect::<BTreeSet<_>>();
    if observed == dialect.modes {
        Ok(())
    } else {
        Err(catalog_error(
            "MySQL live SQL mode, character set, or collation differs from the profile",
        ))
    }
}

fn quote_identifier(value: &str) -> String {
    format!("`{}`", value.replace('`', "``"))
}

fn catalog_error(message: impl Into<String>) -> MysqlCatalogError {
    MysqlCatalogError {
        message: message.into(),
    }
}
