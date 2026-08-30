use crate::RustItem;

pub(super) fn build_secondary_error_type_items() -> Vec<RustItem> {
    vec![RustItem::Attr(
        r#"#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum __SifrSecondaryErrorKind {
    Message,
    CleanupFailed,
    CleanupTimedOut,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SecondaryError {
    message: String,
    kind: __SifrSecondaryErrorKind,
    location: String,
    resource: String,
    operation: String,
    budget_millis: u64,
}

impl SecondaryError {
    fn new(message: String) -> Self {
        Self {
            message,
            kind: __SifrSecondaryErrorKind::Message,
            location: String::new(),
            resource: String::new(),
            operation: String::new(),
            budget_millis: 0,
        }
    }

    fn from_async_cleanup(
        evidence: ::sifr_runtime::async_cleanup::AsyncCleanupEvidence,
    ) -> Self {
        match evidence {
            ::sifr_runtime::async_cleanup::AsyncCleanupEvidence::CleanupFailed {
                error,
                location,
                resource,
                operation,
                budget,
            } => Self {
                message: error,
                kind: __SifrSecondaryErrorKind::CleanupFailed,
                location,
                resource,
                operation,
                budget_millis: u64::try_from(budget.as_millis()).unwrap_or(u64::MAX),
            },
            ::sifr_runtime::async_cleanup::AsyncCleanupEvidence::CleanupTimedOut {
                location,
                resource,
                operation,
                budget,
            } => Self {
                message: "asynchronous cleanup timed out".to_string(),
                kind: __SifrSecondaryErrorKind::CleanupTimedOut,
                location,
                resource,
                operation,
                budget_millis: u64::try_from(budget.as_millis()).unwrap_or(u64::MAX),
            },
        }
    }
}

impl std::fmt::Display for SecondaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.message, f)
    }
}

impl std::error::Error for SecondaryError {}"#
            .to_string(),
    )]
}
