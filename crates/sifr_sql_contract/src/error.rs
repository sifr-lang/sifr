use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaContractErrorKind {
    InvalidProfile,
    InvalidProvider,
    InvalidSchema,
    DuplicateObject,
    MissingDependency,
    AmbiguousSymbol,
    UnknownSymbol,
    ReservedExport,
    IncompatibleSchema,
    Serialization,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaContractError {
    pub kind: SchemaContractErrorKind,
    pub message: String,
}

impl SchemaContractError {
    #[must_use]
    pub fn new(kind: SchemaContractErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for SchemaContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SchemaContractError {}
