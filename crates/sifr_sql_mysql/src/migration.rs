use crate::component::mysql_capabilities;
use crate::parser::MysqlParser;
use crate::schema::{MysqlSchemaOptions, normalize_mysql_documents};
use sifr_sql_contract::{
    DdlReflection, DdlRisk, MigrationCompileError, MigrationCompileErrorKind, MigrationDialect,
    SchemaIr,
};
use std::collections::BTreeSet;

pub struct MysqlMigrationDialect {
    parser: MysqlParser,
    options: MysqlSchemaOptions,
}

impl MysqlMigrationDialect {
    #[must_use]
    pub fn new(parser: MysqlParser, options: MysqlSchemaOptions) -> Self {
        Self { parser, options }
    }
}

impl MigrationDialect for MysqlMigrationDialect {
    fn family(&self) -> &'static str {
        "mysql"
    }

    fn server_version(&self) -> &str {
        // The public contract borrows this value, so use the canonical static
        // supported-series name selected by the parser.
        match (self.parser.series().major, self.parser.series().minor) {
            (8, 4) => "8.4",
            (9, 7) => "9.7",
            (26, 7) => "26.7",
            _ => "unsupported",
        }
    }

    fn capabilities(&self) -> &BTreeSet<String> {
        static CAPABILITIES: std::sync::OnceLock<BTreeSet<String>> = std::sync::OnceLock::new();
        CAPABILITIES.get_or_init(mysql_capabilities)
    }

    fn reflect_ddl(
        &self,
        input: &SchemaIr,
        statement: &str,
    ) -> Result<DdlReflection, MigrationCompileError> {
        if input.dialect.family != "mysql" {
            return Err(reflection_error(
                "MySQL migration reflection requires a MySQL schema",
            ));
        }
        let parsed = self
            .parser
            .parse(statement)
            .map_err(|error| reflection_error(error.to_string()))?;
        if parsed.len() != 1 {
            return Err(reflection_error(
                "a MySQL DDL migration step must contain one statement",
            ));
        }
        if !matches!(
            parsed[0].kind,
            crate::ast::MysqlStatementKind::CreateTable(_)
                | crate::ast::MysqlStatementKind::CreateView(_)
                | crate::ast::MysqlStatementKind::CreateIndex(_)
        ) {
            return Ok(DdlReflection::Opaque);
        }
        let output = normalize_mysql_documents(
            input.provider.clone(),
            &self.parser,
            &self.options,
            vec![(
                "sifr://mysql/migration-ddl".to_string(),
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
pub enum MysqlDdlExecutionClass {
    RequiresBoundary {
        recovery_reason: &'static str,
        online: bool,
    },
}

#[must_use]
pub fn classify_migration_ddl(statement: &str) -> MysqlDdlExecutionClass {
    let normalized = statement.to_ascii_uppercase();
    let online = normalized.contains("ALGORITHM=INSTANT")
        || normalized.contains("ALGORITHM=INPLACE")
        || normalized.contains("LOCK=NONE");
    MysqlDdlExecutionClass::RequiresBoundary {
        recovery_reason: "MySQL DDL can commit implicitly and needs an explicit recovery point",
        online,
    }
}

fn reflected_risk(statement: &str) -> DdlRisk {
    let normalized = statement.to_ascii_uppercase();
    let mut lock_risks = BTreeSet::from(["mysql-metadata-lock".to_string()]);
    let mut data_rewrites = BTreeSet::new();
    if normalized.contains("ALGORITHM=COPY")
        || normalized.starts_with("ALTER TABLE") && !normalized.contains("ALGORITHM=INSTANT")
    {
        data_rewrites.insert("mysql-table-rebuild".to_string());
    }
    if normalized.contains("LOCK=NONE") {
        lock_risks.remove("mysql-metadata-lock");
        lock_risks.insert("mysql-online-metadata-lock".to_string());
    }
    DdlRisk {
        lock_risks,
        data_rewrites,
    }
}

fn reflection_error(message: impl Into<String>) -> MigrationCompileError {
    MigrationCompileError::new(MigrationCompileErrorKind::DdlReflection, message)
}
