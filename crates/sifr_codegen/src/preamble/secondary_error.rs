use crate::RustItem;

pub(crate) fn build_secondary_error_type_items(include_cleanup_evidence: bool) -> Vec<RustItem> {
    let mut source = r#"#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    }"#
    .to_string();
    if include_cleanup_evidence {
        source.push_str(
            r#"

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
    }"#,
        );
    }
    source.push_str(
        r#"
}

impl std::fmt::Display for SecondaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.message, f)
    }
}

impl std::error::Error for SecondaryError {}"#,
    );
    vec![RustItem::Attr(source)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(include_cleanup_evidence: bool) -> String {
        let items: [RustItem; 1] = build_secondary_error_type_items(include_cleanup_evidence)
            .try_into()
            .expect("one secondary error support item");
        let [RustItem::Attr(source)] = items else {
            panic!("secondary error support must be one source item");
        };
        source
    }

    #[test]
    fn synchronous_secondary_error_has_no_async_runtime_reference() {
        let source = source(false);
        syn::parse_file(&source).expect("valid standalone error items");
        assert!(!source.contains("sifr_runtime"));
        assert!(!source.contains("from_async_cleanup"));
        assert!(source.contains("fn new(message: String)"));
        for field in [
            "message",
            "kind",
            "location",
            "resource",
            "operation",
            "budget_millis",
        ] {
            assert!(source.contains(&format!("    {field}:")));
        }
    }

    #[test]
    fn async_conversion_stays_in_the_secondary_error_constructor_impl() {
        let source = source(true);
        let file = syn::parse_file(&source).expect("valid async error items");
        let inherent = file
            .items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Impl(item) if item.trait_.is_none() => Some(item),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(inherent.len(), 1);
        let methods = inherent[0]
            .items
            .iter()
            .filter_map(|item| match item {
                syn::ImplItem::Fn(item) => Some(item.sig.ident.to_string()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(methods, ["new", "from_async_cleanup"]);
        assert!(source.contains("AsyncCleanupEvidence::CleanupFailed"));
        assert!(source.contains("AsyncCleanupEvidence::CleanupTimedOut"));
    }
}
