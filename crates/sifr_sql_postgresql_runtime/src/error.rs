use sifr_sql_runtime::{
    ConstraintKind, RetryClassification, SafeSqlIdentifier, SqlError, SqlErrorKind,
    SqlErrorMetadata, SqlState,
};

pub(crate) fn configuration_error() -> SqlError {
    SqlError::new(SqlErrorKind::Configuration)
}

pub(crate) fn provider_error() -> SqlError {
    SqlError::new(SqlErrorKind::Provider)
}

pub(crate) fn map_postgres_error(error: &tokio_postgres::Error) -> SqlError {
    let Some(database) = error.as_db_error() else {
        return SqlError::new(if error.is_closed() {
            SqlErrorKind::Connection
        } else {
            SqlErrorKind::Provider
        });
    };
    let state = database.code().code();
    let (kind, retry, constraint_kind) = classify_state(state);
    let metadata = SqlErrorMetadata {
        sql_state: SqlState::new(state).ok(),
        vendor_code: None,
        constraint_kind,
        constraint_identity: database.constraint().and_then(safe_identifier),
        table_identity: database.table().and_then(safe_identifier),
        columns: database
            .column()
            .and_then(safe_identifier)
            .into_iter()
            .collect(),
        retry,
        resource_limit: None,
        cardinality: None,
    };
    SqlError::with_metadata(kind, metadata).unwrap_or_else(|error| error)
}

fn classify_state(state: &str) -> (SqlErrorKind, RetryClassification, Option<ConstraintKind>) {
    match state {
        "40001" => (
            SqlErrorKind::Serialization,
            RetryClassification::RetryTransaction,
            None,
        ),
        "40P01" => (
            SqlErrorKind::Deadlock,
            RetryClassification::RetryTransaction,
            None,
        ),
        "57014" => (SqlErrorKind::Cancelled, RetryClassification::Never, None),
        "23505" => constraint(ConstraintKind::Unique),
        "23503" => constraint(ConstraintKind::ForeignKey),
        "23514" => constraint(ConstraintKind::Check),
        "23502" => constraint(ConstraintKind::NotNull),
        "23P01" => constraint(ConstraintKind::Exclusion),
        value if value.starts_with("23") => constraint(ConstraintKind::ProviderSpecific),
        value if value.starts_with("28") => (
            SqlErrorKind::Authentication,
            RetryClassification::Never,
            None,
        ),
        value if value.starts_with("08") => (
            SqlErrorKind::Connection,
            RetryClassification::RetryConnection,
            None,
        ),
        _ => (SqlErrorKind::Provider, RetryClassification::Never, None),
    }
}

fn constraint(kind: ConstraintKind) -> (SqlErrorKind, RetryClassification, Option<ConstraintKind>) {
    (
        SqlErrorKind::Constraint,
        RetryClassification::Never,
        Some(kind),
    )
}

fn safe_identifier(value: &str) -> Option<SafeSqlIdentifier> {
    SafeSqlIdentifier::new(value).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlstate_classification_is_stable_and_retry_specific() {
        assert_eq!(
            classify_state("40001"),
            (
                SqlErrorKind::Serialization,
                RetryClassification::RetryTransaction,
                None,
            )
        );
        assert_eq!(
            classify_state("40P01"),
            (
                SqlErrorKind::Deadlock,
                RetryClassification::RetryTransaction,
                None,
            )
        );
        assert_eq!(
            classify_state("23505"),
            (
                SqlErrorKind::Constraint,
                RetryClassification::Never,
                Some(ConstraintKind::Unique),
            )
        );
        assert_eq!(
            classify_state("08006").1,
            RetryClassification::RetryConnection
        );
    }
}
