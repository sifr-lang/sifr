use crate::model::Severity;

use super::DiagnosticCode;

impl DiagnosticCode {
    pub const COMPONENT_REGISTRATION: Self = Self::new("SIFR-COMPONENT-0001", Severity::Error);
    pub const COMPONENT_INTEGRITY: Self = Self::new("SIFR-COMPONENT-0002", Severity::Error);
    pub const COMPONENT_PROTOCOL_VERSION: Self = Self::new("SIFR-COMPONENT-0003", Severity::Error);
    pub const COMPONENT_PROTOCOL_ENVELOPE: Self = Self::new("SIFR-COMPONENT-0004", Severity::Error);
    pub const COMPONENT_CAPABILITY: Self = Self::new("SIFR-COMPONENT-0005", Severity::Error);
    pub const COMPONENT_RESOURCE_LIMIT: Self = Self::new("SIFR-COMPONENT-0006", Severity::Error);
    pub const COMPONENT_EXECUTION: Self = Self::new("SIFR-COMPONENT-0007", Severity::Error);
    pub const COMPONENT_CACHE: Self = Self::new("SIFR-COMPONENT-0008", Severity::Error);
    pub const COMPONENT_DIAGNOSTIC_REGISTRY: Self =
        Self::new("SIFR-COMPONENT-0009", Severity::Error);
}
