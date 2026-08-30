use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentErrorKind {
    Registration,
    Integrity,
    ProtocolVersion,
    ProtocolEnvelope,
    Capability,
    ResourceLimit,
    Execution,
    Cache,
    DiagnosticRegistry,
}

impl ComponentErrorKind {
    pub const ALL: [Self; 9] = [
        Self::Registration,
        Self::Integrity,
        Self::ProtocolVersion,
        Self::ProtocolEnvelope,
        Self::Capability,
        Self::ResourceLimit,
        Self::Execution,
        Self::Cache,
        Self::DiagnosticRegistry,
    ];

    #[must_use]
    pub const fn diagnostic_code(self) -> sifr_diagnostics::DiagnosticCode {
        match self {
            Self::Registration => sifr_diagnostics::DiagnosticCode::COMPONENT_REGISTRATION,
            Self::Integrity => sifr_diagnostics::DiagnosticCode::COMPONENT_INTEGRITY,
            Self::ProtocolVersion => sifr_diagnostics::DiagnosticCode::COMPONENT_PROTOCOL_VERSION,
            Self::ProtocolEnvelope => sifr_diagnostics::DiagnosticCode::COMPONENT_PROTOCOL_ENVELOPE,
            Self::Capability => sifr_diagnostics::DiagnosticCode::COMPONENT_CAPABILITY,
            Self::ResourceLimit => sifr_diagnostics::DiagnosticCode::COMPONENT_RESOURCE_LIMIT,
            Self::Execution => sifr_diagnostics::DiagnosticCode::COMPONENT_EXECUTION,
            Self::Cache => sifr_diagnostics::DiagnosticCode::COMPONENT_CACHE,
            Self::DiagnosticRegistry => {
                sifr_diagnostics::DiagnosticCode::COMPONENT_DIAGNOSTIC_REGISTRY
            }
        }
    }

    #[must_use]
    pub const fn code(self) -> &'static str {
        self.diagnostic_code().code()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentError {
    pub kind: ComponentErrorKind,
    pub message: String,
}

impl ComponentError {
    #[must_use]
    pub fn new(kind: ComponentErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.kind.code()
    }
}

impl Display for ComponentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code(), self.message)
    }
}

impl std::error::Error for ComponentError {}
