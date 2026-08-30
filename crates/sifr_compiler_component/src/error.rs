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
    pub const fn code(self) -> &'static str {
        match self {
            Self::Registration => "SIFR-COMPONENT-0001",
            Self::Integrity => "SIFR-COMPONENT-0002",
            Self::ProtocolVersion => "SIFR-COMPONENT-0003",
            Self::ProtocolEnvelope => "SIFR-COMPONENT-0004",
            Self::Capability => "SIFR-COMPONENT-0005",
            Self::ResourceLimit => "SIFR-COMPONENT-0006",
            Self::Execution => "SIFR-COMPONENT-0007",
            Self::Cache => "SIFR-COMPONENT-0008",
            Self::DiagnosticRegistry => "SIFR-COMPONENT-0009",
        }
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
