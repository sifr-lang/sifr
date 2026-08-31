use mysql_async::Error;
use sifr_sql_runtime::{
    ConstraintKind, RetryClassification, SafeSqlIdentifier, SqlError, SqlErrorKind,
    SqlErrorMetadata,
};

pub(crate) fn configuration_error() -> SqlError {
    SqlError::new(SqlErrorKind::Configuration)
}

pub(crate) fn provider_error() -> SqlError {
    SqlError::new(SqlErrorKind::Provider)
}

pub(crate) fn map_mysql_error(error: &Error) -> SqlError {
    match error {
        Error::Server(server) => {
            let (kind, retry, constraint_kind) = classify_vendor_code(server.code);
            let metadata = SqlErrorMetadata {
                vendor_code: Some(i64::from(server.code)),
                constraint_kind,
                constraint_identity: extract_quoted_identity(&server.message),
                retry,
                ..SqlErrorMetadata::default()
            };
            SqlError::with_metadata(kind, metadata).unwrap_or_else(|failure| failure)
        }
        Error::Io(_) => SqlError::new(SqlErrorKind::Connection),
        Error::Driver(_) => SqlError::new(SqlErrorKind::Provider),
        Error::Url(_) => configuration_error(),
        Error::Other(_) => SqlError::new(SqlErrorKind::Provider),
    }
}

fn classify_vendor_code(code: u16) -> (SqlErrorKind, RetryClassification, Option<ConstraintKind>) {
    match code {
        1045 => (
            SqlErrorKind::Authentication,
            RetryClassification::Never,
            None,
        ),
        1062 => constraint(ConstraintKind::Unique),
        1048 => constraint(ConstraintKind::NotNull),
        1451 | 1452 => constraint(ConstraintKind::ForeignKey),
        3819 => constraint(ConstraintKind::Check),
        1205 => (
            SqlErrorKind::Timeout,
            RetryClassification::RetryTransaction,
            None,
        ),
        1213 => (
            SqlErrorKind::Deadlock,
            RetryClassification::RetryTransaction,
            None,
        ),
        1317 => (SqlErrorKind::Cancelled, RetryClassification::Never, None),
        2002 | 2003 | 2006 | 2013 => (
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

fn extract_quoted_identity(message: &str) -> Option<SafeSqlIdentifier> {
    let start = message.find('`')? + 1;
    let end = message[start..].find('`')? + start;
    SafeSqlIdentifier::new(&message[start..end]).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendor_codes_have_stable_retry_and_constraint_meaning() {
        assert_eq!(classify_vendor_code(1062).2, Some(ConstraintKind::Unique));
        assert_eq!(
            classify_vendor_code(1213).1,
            RetryClassification::RetryTransaction
        );
        assert_eq!(
            classify_vendor_code(2006).1,
            RetryClassification::RetryConnection
        );
    }
}
