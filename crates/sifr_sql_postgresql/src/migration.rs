use crate::catalog::ddl_document;
use crate::{PostgresDiagnosticCode, PostgresParser, PostgresTypeRegistry};
use sifr_sql_contract::{
    DdlReflection, DdlRisk, MigrationCompileError, MigrationCompileErrorKind, MigrationDialect,
    SchemaIr,
};
use std::collections::BTreeSet;

pub struct PostgresMigrationDialect<P> {
    parser: P,
    server_version: String,
    capabilities: BTreeSet<String>,
}

impl<P> PostgresMigrationDialect<P> {
    #[must_use]
    pub fn new(
        parser: P,
        server_version: impl Into<String>,
        capabilities: BTreeSet<String>,
    ) -> Self {
        Self {
            parser,
            server_version: server_version.into(),
            capabilities,
        }
    }
}

impl<P: PostgresParser> MigrationDialect for PostgresMigrationDialect<P> {
    fn family(&self) -> &'static str {
        "postgresql"
    }

    fn server_version(&self) -> &str {
        &self.server_version
    }

    fn capabilities(&self) -> &BTreeSet<String> {
        &self.capabilities
    }

    fn reflect_ddl(
        &self,
        input: &SchemaIr,
        statement: &str,
    ) -> Result<DdlReflection, MigrationCompileError> {
        if input.dialect.family != "postgresql" {
            return Err(reflection_error(
                "PostgreSQL migration reflection requires a PostgreSQL schema",
            ));
        }
        let expected_major = self
            .server_version
            .split('.')
            .next()
            .and_then(|value| value.parse::<u16>().ok())
            .ok_or_else(|| reflection_error("PostgreSQL server version is invalid"))?;
        if expected_major != self.parser.server_major() {
            return Err(reflection_error(
                "PostgreSQL migration parser and server majors differ",
            ));
        }
        let statements = match self.parser.parse(statement) {
            Ok(statements) => statements,
            Err(failure)
                if failure.diagnostic.code == PostgresDiagnosticCode::UnsupportedCoreSyntax =>
            {
                return Ok(DdlReflection::Opaque);
            }
            Err(failure) => return Err(reflection_error(failure.to_string())),
        };
        if statements.len() != 1 {
            return Err(reflection_error(
                "a PostgreSQL DDL migration step must contain one statement",
            ));
        }
        let document = ddl_document(
            "sifr://migration-ddl",
            &statements,
            &PostgresTypeRegistry::new(expected_major),
            &input.objects,
        )
        .map_err(|failure| reflection_error(failure.message))?;
        if document.objects.is_empty() {
            return Ok(DdlReflection::Opaque);
        }
        let mut schema = input.clone();
        for object in document.objects {
            if schema
                .objects
                .insert(object.identity.clone(), object)
                .is_some()
            {
                return Err(reflection_error(
                    "PostgreSQL DDL replaces an existing schema identity",
                ));
            }
        }
        let risk = reflected_risk(statement);
        Ok(DdlReflection::Reflected { schema, risk })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PostgresDdlExecutionClass {
    Transactional,
    RequiresAutocommit { recovery_reason: &'static str },
}

#[must_use]
pub fn classify_migration_ddl(statement: &str) -> PostgresDdlExecutionClass {
    let tokens = leading_sql_tokens(statement);
    let first = tokens.first().map(String::as_str);
    let second = tokens.get(1).map(String::as_str);
    let create_or_drop_nontransactional = matches!(first, Some("CREATE" | "DROP"))
        && matches!(second, Some("DATABASE" | "TABLESPACE" | "SUBSCRIPTION"));
    let index_concurrently = matches!(first, Some("CREATE" | "DROP"))
        && tokens
            .iter()
            .position(|token| token == "INDEX")
            .and_then(|index| tokens.get(index + 1))
            .is_some_and(|token| token == "CONCURRENTLY");
    let refresh_concurrently = tokens.starts_with(&[
        "REFRESH".to_string(),
        "MATERIALIZED".to_string(),
        "VIEW".to_string(),
        "CONCURRENTLY".to_string(),
    ]);
    if create_or_drop_nontransactional
        || index_concurrently
        || (first == Some("REINDEX") && tokens.iter().any(|token| token == "CONCURRENTLY"))
        || refresh_concurrently
        || tokens.starts_with(&["ALTER".to_string(), "SYSTEM".to_string()])
        || tokens.starts_with(&["ALTER".to_string(), "SUBSCRIPTION".to_string()])
        || matches!(first, Some("VACUUM" | "CLUSTER"))
    {
        PostgresDdlExecutionClass::RequiresAutocommit {
            recovery_reason: "PostgreSQL requires this DDL outside a transaction",
        }
    } else {
        PostgresDdlExecutionClass::Transactional
    }
}

fn leading_sql_tokens(statement: &str) -> Vec<String> {
    let bytes = statement.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() && tokens.len() < 12 {
        if bytes[index].is_ascii_whitespace() || bytes[index].is_ascii_punctuation() {
            if bytes[index] == b'-' && bytes.get(index + 1) == Some(&b'-') {
                index += 2;
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
                continue;
            }
            if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'*') {
                index += 2;
                let mut depth = 1_u32;
                while index < bytes.len() && depth > 0 {
                    if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'*') {
                        depth = depth.saturating_add(1);
                        index += 2;
                    } else if bytes[index] == b'*' && bytes.get(index + 1) == Some(&b'/') {
                        depth = depth.saturating_sub(1);
                        index += 2;
                    } else {
                        index += 1;
                    }
                }
                continue;
            }
            index += 1;
            continue;
        }
        if bytes[index].is_ascii_alphabetic() || bytes[index] == b'_' {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            tokens.push(statement[start..index].to_ascii_uppercase());
            continue;
        }
        index += 1;
    }
    tokens
}

fn reflected_risk(statement: &str) -> DdlRisk {
    let normalized = statement.to_ascii_uppercase();
    let mut lock_risks = BTreeSet::new();
    let mut data_rewrites = BTreeSet::new();
    if normalized.contains("CREATE TABLE")
        || normalized.contains("CREATE INDEX")
        || normalized.contains("CREATE MATERIALIZED VIEW")
    {
        lock_risks.insert("postgresql-schema-lock".to_string());
    }
    if normalized.contains("CREATE MATERIALIZED VIEW") {
        data_rewrites.insert("materialized-view-population".to_string());
    }
    DdlRisk {
        lock_risks,
        data_rewrites,
    }
}

fn reflection_error(message: impl Into<String>) -> MigrationCompileError {
    MigrationCompileError::new(MigrationCompileErrorKind::DdlReflection, message)
}
