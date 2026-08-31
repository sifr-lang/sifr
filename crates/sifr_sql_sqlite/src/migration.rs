use crate::component::sqlite_capabilities;
use crate::parser::SqliteParser;
use crate::schema::{SqliteSchemaOptions, normalize_sqlite_documents};
use sifr_sql_contract::{
    DdlReflection, DdlRisk, MigrationCompileError, MigrationCompileErrorKind, MigrationDialect,
    SchemaIr,
};
use std::collections::BTreeSet;

pub struct SqliteMigrationDialect {
    parser: SqliteParser,
    options: SqliteSchemaOptions,
}

impl SqliteMigrationDialect {
    #[must_use]
    pub fn new(parser: SqliteParser, options: SqliteSchemaOptions) -> Self {
        Self { parser, options }
    }
}

impl MigrationDialect for SqliteMigrationDialect {
    fn family(&self) -> &'static str {
        "sqlite"
    }

    fn server_version(&self) -> &str {
        // The public contract borrows this value, so use the canonical static
        // supported-series name selected by the parser.
        match (
            self.parser.series().major,
            self.parser.series().minor,
            self.parser.series().patch,
        ) {
            (3, 53, 2) => "3.53.2",
            _ => "unsupported",
        }
    }

    fn capabilities(&self) -> &BTreeSet<String> {
        static CAPABILITIES: std::sync::OnceLock<BTreeSet<String>> = std::sync::OnceLock::new();
        CAPABILITIES.get_or_init(sqlite_capabilities)
    }

    fn reflect_ddl(
        &self,
        input: &SchemaIr,
        statement: &str,
    ) -> Result<DdlReflection, MigrationCompileError> {
        if input.dialect.family != "sqlite" {
            return Err(reflection_error(
                "SQLite migration reflection requires a SQLite schema",
            ));
        }
        let parsed = self
            .parser
            .parse(statement)
            .map_err(|error| reflection_error(error.to_string()))?;
        if parsed.len() != 1 {
            return Err(reflection_error(
                "a SQLite DDL migration step must contain one statement",
            ));
        }
        if !matches!(
            parsed[0].kind,
            crate::ast::SqliteStatementKind::CreateTable(_)
                | crate::ast::SqliteStatementKind::CreateView(_)
                | crate::ast::SqliteStatementKind::CreateIndex(_)
        ) {
            return Ok(DdlReflection::Opaque);
        }
        let output = normalize_sqlite_documents(
            input.provider.clone(),
            &self.parser,
            &self.options,
            vec![(
                "sifr://sqlite/migration-ddl".to_string(),
                statement.to_string(),
            )],
        )
        .map_err(|error| reflection_error(error.to_string()))?;
        let reflected = sifr_sql_contract::normalize_schema(
            input.provider.clone(),
            output.dialect,
            output.documents,
        )
        .map_err(|error| reflection_error(error.to_string()))?;
        let mut schema = input.clone();
        for (identity, object) in reflected.objects {
            schema.objects.insert(identity, object);
        }
        Ok(DdlReflection::Reflected {
            schema,
            risk: reflected_risk(statement),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SqliteDdlExecutionClass {
    RequiresBoundary {
        recovery_reason: &'static str,
        online: bool,
    },
}

#[must_use]
pub fn classify_migration_ddl(_statement: &str) -> SqliteDdlExecutionClass {
    let online = false;
    SqliteDdlExecutionClass::RequiresBoundary {
        recovery_reason: "SQLite table rebuilds need a savepoint, foreign-key check, and explicit recovery point",
        online,
    }
}

fn reflected_risk(statement: &str) -> DdlRisk {
    let normalized = statement.to_ascii_uppercase();
    let lock_risks = BTreeSet::from(["sqlite-metadata-lock".to_string()]);
    let mut data_rewrites = BTreeSet::new();
    if normalized.starts_with("ALTER TABLE") {
        data_rewrites.insert("sqlite-table-rebuild".to_string());
    }
    DdlRisk {
        lock_risks,
        data_rewrites,
    }
}

fn reflection_error(message: impl Into<String>) -> MigrationCompileError {
    MigrationCompileError::new(MigrationCompileErrorKind::DdlReflection, message)
}
