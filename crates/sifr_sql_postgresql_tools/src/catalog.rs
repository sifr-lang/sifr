use rustls::ClientConfig;
use rustls_platform_verifier::ConfigVerifierExt;
use serde_json::Value;
use sifr_sql_contract::{
    DialectIdentity, ObjectId, ProviderIdentity, SchemaDocument, SchemaDocumentKind, SchemaIr,
    SchemaObject, SchemaObjectKind, SemanticValue, normalize_schema,
};
use std::collections::BTreeMap;
use std::fmt;
use tokio_postgres::config::SslMode;
use tokio_postgres::{Client, Config, NoTls, Row};
use tokio_postgres_rustls::MakeRustlsConnect;

const CATALOG_QUERY: &str = include_str!("postgresql_catalog.sql");
const MULTIRANGE_QUERY: &str = include_str!("postgresql_multirange.sql");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostgresCatalogError {
    pub message: String,
}

impl fmt::Display for PostgresCatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for PostgresCatalogError {}

pub async fn pull_live_catalog(
    connection_url: &str,
    provider: ProviderIdentity,
    expected_dialect: DialectIdentity,
) -> Result<SchemaIr, PostgresCatalogError> {
    let expected_server_major = expected_major(&expected_dialect)?;
    let config = connection_url
        .parse::<Config>()
        .map_err(|_| catalog_error("PostgreSQL connection configuration is invalid"))?;
    let ssl_mode = config.get_ssl_mode();
    let (client, connection_task) = if ssl_mode == SslMode::Disable {
        let (client, connection) = config
            .connect(NoTls)
            .await
            .map_err(|_| catalog_error("cannot connect to the PostgreSQL catalog"))?;
        (
            client,
            tokio::spawn(async move { connection.await.map_err(|_| ()) }),
        )
    } else {
        let tls = ClientConfig::with_platform_verifier()
            .map_err(|_| catalog_error("cannot initialize platform TLS verification"))?;
        let (client, connection) = config
            .connect(MakeRustlsConnect::new(tls))
            .await
            .map_err(|_| catalog_error("cannot connect to the PostgreSQL catalog with TLS"))?;
        (
            client,
            tokio::spawn(async move { connection.await.map_err(|_| ()) }),
        )
    };
    let result = pull_from_client(&client, provider, expected_dialect, expected_server_major).await;
    drop(client);
    let _ = connection_task.await;
    result
}

async fn pull_from_client(
    client: &Client,
    provider: ProviderIdentity,
    expected_dialect: DialectIdentity,
    expected_server_major: u16,
) -> Result<SchemaIr, PostgresCatalogError> {
    client
        .batch_execute("BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .await
        .map_err(|_| catalog_error("cannot start a consistent PostgreSQL catalog snapshot"))?;
    let version = client
        .query_one("SHOW server_version_num", &[])
        .await
        .map_err(|_| catalog_error("cannot read the PostgreSQL server version"))?
        .try_get::<_, String>(0)
        .map_err(|_| catalog_error("PostgreSQL returned invalid server version metadata"))?;
    let major = parse_server_major(&version)?;
    if major != expected_server_major {
        return Err(catalog_error(format!(
            "PostgreSQL server major {major} does not match profile major {expected_server_major}"
        )));
    }
    let mut rows = client
        .query(CATALOG_QUERY, &[])
        .await
        .map_err(|failure| postgres_error("cannot introspect the PostgreSQL catalog", &failure))?;
    if major >= 14 {
        rows.extend(
            client
                .query(MULTIRANGE_QUERY, &[])
                .await
                .map_err(|failure| {
                    postgres_error("cannot introspect PostgreSQL multiranges", &failure)
                })?,
        );
    }
    client
        .batch_execute("COMMIT")
        .await
        .map_err(|_| catalog_error("cannot finish the PostgreSQL catalog snapshot"))?;
    schema_from_rows(provider, expected_dialect, rows)
}

fn schema_from_rows(
    provider: ProviderIdentity,
    dialect: DialectIdentity,
    rows: Vec<Row>,
) -> Result<SchemaIr, PostgresCatalogError> {
    let mut objects = Vec::with_capacity(rows.len());
    for row in rows {
        let identity = row
            .try_get::<_, String>("identity")
            .map_err(|_| incomplete("object identity"))?;
        let kind = parse_kind(
            &row.try_get::<_, String>("object_kind")
                .map_err(|_| incomplete("object kind"))?,
        )?;
        let semantic_json = row
            .try_get::<_, String>("semantic_json")
            .map_err(|_| incomplete("object semantics"))?;
        let dependencies_json = row
            .try_get::<_, String>("dependencies_json")
            .map_err(|_| incomplete("object dependencies"))?;
        let semantic = value_map(
            serde_json::from_str(&semantic_json).map_err(|_| incomplete("object semantic JSON"))?,
        )?;
        let dependencies = serde_json::from_str::<Vec<String>>(&dependencies_json)
            .map_err(|_| incomplete("object dependency JSON"))?
            .into_iter()
            .map(ObjectId::new)
            .collect();
        objects.push(SchemaObject {
            identity: ObjectId::new(identity),
            kind,
            semantic,
            dependencies,
            source: None,
        });
    }
    crate::normalization::normalize_catalog_objects(expected_major(&dialect)?, &mut objects)?;
    normalize_schema(
        provider,
        dialect,
        [SchemaDocument {
            kind: SchemaDocumentKind::ProviderMetadata,
            document: "postgresql://live-catalog".to_string(),
            objects,
        }],
    )
    .map_err(|failure| {
        catalog_error(format!(
            "PostgreSQL catalog metadata is incomplete: {failure}"
        ))
    })
}

fn expected_major(dialect: &DialectIdentity) -> Result<u16, PostgresCatalogError> {
    if dialect.family != "postgresql" {
        return Err(catalog_error(
            "PostgreSQL catalog pull requires a PostgreSQL dialect identity",
        ));
    }
    dialect
        .server_version
        .split('.')
        .next()
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|major| *major >= 10)
        .ok_or_else(|| incomplete("profile server major"))
}

fn value_map(value: Value) -> Result<BTreeMap<String, SemanticValue>, PostgresCatalogError> {
    let Value::Object(values) = value else {
        return Err(incomplete("object semantic map"));
    };
    values
        .into_iter()
        .map(|(key, value)| Ok((key, semantic_value(value)?)))
        .collect()
}

fn semantic_value(value: Value) -> Result<SemanticValue, PostgresCatalogError> {
    match value {
        Value::Bool(value) => Ok(SemanticValue::Bool(value)),
        Value::Number(value) => value
            .as_i64()
            .map(SemanticValue::Signed)
            .or_else(|| value.as_u64().map(SemanticValue::Unsigned))
            .ok_or_else(|| incomplete("integer semantic value")),
        Value::String(value) => Ok(SemanticValue::Text(value)),
        Value::Array(values) => values
            .into_iter()
            .map(semantic_value)
            .collect::<Result<Vec<_>, _>>()
            .map(SemanticValue::List),
        Value::Object(values) => values
            .into_iter()
            .map(|(key, value)| Ok((key, semantic_value(value)?)))
            .collect::<Result<BTreeMap<_, _>, _>>()
            .map(SemanticValue::Map),
        Value::Null => Err(incomplete("non-null semantic value")),
    }
}

fn parse_server_major(version: &str) -> Result<u16, PostgresCatalogError> {
    version
        .parse::<u32>()
        .ok()
        .map(|value| value / 10_000)
        .and_then(|value| u16::try_from(value).ok())
        .filter(|major| *major >= 10)
        .ok_or_else(|| incomplete("server version number"))
}

fn parse_kind(value: &str) -> Result<SchemaObjectKind, PostgresCatalogError> {
    match value {
        "namespace" => Ok(SchemaObjectKind::Namespace),
        "table" => Ok(SchemaObjectKind::Table),
        "column" => Ok(SchemaObjectKind::Column),
        "primary-key" => Ok(SchemaObjectKind::PrimaryKey),
        "unique-constraint" => Ok(SchemaObjectKind::UniqueConstraint),
        "foreign-key" => Ok(SchemaObjectKind::ForeignKey),
        "check-constraint" => Ok(SchemaObjectKind::CheckConstraint),
        "index" => Ok(SchemaObjectKind::Index),
        "sequence" => Ok(SchemaObjectKind::Sequence),
        "identity-column" => Ok(SchemaObjectKind::IdentityColumn),
        "view" => Ok(SchemaObjectKind::View),
        "materialized-view" => Ok(SchemaObjectKind::MaterializedView),
        "enum" => Ok(SchemaObjectKind::Enum),
        "domain" => Ok(SchemaObjectKind::Domain),
        "composite" => Ok(SchemaObjectKind::Composite),
        "array" => Ok(SchemaObjectKind::Array),
        "range" => Ok(SchemaObjectKind::Range),
        "function" => Ok(SchemaObjectKind::Function),
        "operator" => Ok(SchemaObjectKind::Operator),
        "cast" => Ok(SchemaObjectKind::Cast),
        "collation" => Ok(SchemaObjectKind::Collation),
        "extension" => Ok(SchemaObjectKind::Extension),
        "trigger" => Ok(SchemaObjectKind::Trigger),
        "server-capability" => Ok(SchemaObjectKind::ServerCapability),
        "dialect-metadata" => Ok(SchemaObjectKind::DialectMetadata),
        _ => Err(incomplete("known object kind")),
    }
}

fn incomplete(field: &str) -> PostgresCatalogError {
    catalog_error(format!("PostgreSQL catalog is missing required {field}"))
}

fn catalog_error(message: impl Into<String>) -> PostgresCatalogError {
    PostgresCatalogError {
        message: message.into(),
    }
}

fn postgres_error(operation: &str, failure: &tokio_postgres::Error) -> PostgresCatalogError {
    let code = failure
        .code()
        .map_or_else(|| "unknown".to_string(), |code| code.code().to_string());
    catalog_error(format!("{operation} (SQLSTATE {code})"))
}
