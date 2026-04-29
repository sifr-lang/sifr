use crate::model::Severity;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticCode {
    code: &'static str,
    docs_slug: &'static str,
    declared_severity: Severity,
}

impl DiagnosticCode {
    pub const INTERNAL_COMPILER_PANIC: Self =
        Self::new("SIFR-INTERNAL-0001", "SIFR-INTERNAL-0001", Severity::Error);
    pub const NAME_UNDEFINED_VARIABLE: Self =
        Self::new("SIFR-NAME-0001", "SIFR-NAME-0001", Severity::Error);
    pub const TYPE_ASSIGNMENT_MISMATCH: Self =
        Self::new("SIFR-TYPE-0002", "SIFR-TYPE-0002", Severity::Error);
    #[cfg(test)]
    pub(crate) const TEST_NOTE: Self =
        Self::new("SIFR-INTERNAL-9999", "SIFR-INTERNAL-9999", Severity::Note);

    const fn new(code: &'static str, docs_slug: &'static str, declared_severity: Severity) -> Self {
        Self {
            code,
            docs_slug,
            declared_severity,
        }
    }

    #[must_use]
    pub const fn code(self) -> &'static str {
        self.code
    }

    #[must_use]
    pub const fn declared_severity(self) -> Severity {
        self.declared_severity
    }

    #[must_use]
    pub fn docs_url(self) -> String {
        format!("https://sifr.sh/docs/errors/{}", self.docs_slug)
    }
}

#[cfg(test)]
mod tests {
    use super::DiagnosticCode;

    #[test]
    fn docs_url_is_derived_from_code() {
        assert_eq!(
            DiagnosticCode::NAME_UNDEFINED_VARIABLE.docs_url(),
            "https://sifr.sh/docs/errors/SIFR-NAME-0001"
        );
    }
}
