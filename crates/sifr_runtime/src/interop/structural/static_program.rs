use std::fmt;
use std::marker::PhantomData;

use super::{ShapeIdentity, StaticProgramIdentity};
use crate::interop::GeneratedGlueToken;

pub const STATIC_PROGRAM_ENVELOPE_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StaticProgramHeader {
    envelope_version: u32,
    format_version: u32,
    structural_contract_version: u32,
    bridge_contract_version: u32,
    identity: StaticProgramIdentity,
    shape_identity: ShapeIdentity,
}

impl StaticProgramHeader {
    #[doc(hidden)]
    #[must_use]
    pub const fn __from_compiler(
        format_version: u32,
        structural_contract_version: u32,
        bridge_contract_version: u32,
        identity: StaticProgramIdentity,
        shape_identity: ShapeIdentity,
        _token: GeneratedGlueToken,
    ) -> Self {
        Self {
            envelope_version: STATIC_PROGRAM_ENVELOPE_VERSION,
            format_version,
            structural_contract_version,
            bridge_contract_version,
            identity,
            shape_identity,
        }
    }

    #[must_use]
    pub const fn format_version(&self) -> u32 {
        self.format_version
    }

    #[must_use]
    pub const fn structural_contract_version(&self) -> u32 {
        self.structural_contract_version
    }

    #[must_use]
    pub const fn bridge_contract_version(&self) -> u32 {
        self.bridge_contract_version
    }

    #[must_use]
    pub const fn identity(&self) -> StaticProgramIdentity {
        self.identity
    }

    #[must_use]
    pub const fn shape_identity(&self) -> ShapeIdentity {
        self.shape_identity
    }
}

/// Immutable compiler-emitted static data associated with one concrete Sifr type.
#[derive(Debug)]
pub struct StaticProgram<T> {
    header: StaticProgramHeader,
    bytes: &'static [u8],
    _type: PhantomData<fn() -> T>,
}

/// Implemented only by compiler-emitted concrete types with retained static data.
pub trait StaticProgramType: super::StructuralType + Sized + 'static {
    fn static_program() -> &'static StaticProgram<Self>;
}

impl<T> StaticProgram<T> {
    #[doc(hidden)]
    #[must_use]
    pub const fn __from_compiler(
        header: StaticProgramHeader,
        bytes: &'static [u8],
        _token: GeneratedGlueToken,
    ) -> Self {
        Self {
            header,
            bytes,
            _type: PhantomData,
        }
    }

    #[must_use]
    pub const fn header(&self) -> &StaticProgramHeader {
        &self.header
    }

    #[must_use]
    pub const fn bytes(&self) -> &'static [u8] {
        self.bytes
    }

    pub fn verify_envelope(
        &self,
        format_version: u32,
        structural_contract_version: u32,
        bridge_contract_version: u32,
        identity: StaticProgramIdentity,
        shape_identity: ShapeIdentity,
    ) -> Result<(), StaticProgramEnvelopeError> {
        if self.header.envelope_version != STATIC_PROGRAM_ENVELOPE_VERSION {
            return Err(StaticProgramEnvelopeError::EnvelopeVersion);
        }
        if self.header.format_version != format_version {
            return Err(StaticProgramEnvelopeError::FormatVersion);
        }
        if self.header.structural_contract_version != structural_contract_version {
            return Err(StaticProgramEnvelopeError::StructuralContract);
        }
        if self.header.bridge_contract_version != bridge_contract_version {
            return Err(StaticProgramEnvelopeError::BridgeContract);
        }
        if self.header.identity != identity {
            return Err(StaticProgramEnvelopeError::Identity);
        }
        if self.header.shape_identity != shape_identity {
            return Err(StaticProgramEnvelopeError::ShapeIdentity);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StaticProgramEnvelopeError {
    EnvelopeVersion,
    FormatVersion,
    StructuralContract,
    BridgeContract,
    Identity,
    ShapeIdentity,
}

impl fmt::Display for StaticProgramEnvelopeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EnvelopeVersion => "static program envelope version mismatch",
            Self::FormatVersion => "static program format version mismatch",
            Self::StructuralContract => "static program structural contract mismatch",
            Self::BridgeContract => "static program bridge contract mismatch",
            Self::Identity => "static program identity mismatch",
            Self::ShapeIdentity => "static program shape identity mismatch",
        })
    }
}

impl std::error::Error for StaticProgramEnvelopeError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interop::__generated_glue;
    use crate::interop::structural::{primitive, static_program_identity};

    #[test]
    fn sealed_program_verifies_complete_envelope() {
        let shape = primitive("str");
        let identity = static_program_identity(1, [("value", b"program".as_slice())]);
        let header = StaticProgramHeader::__from_compiler(
            3,
            1,
            1,
            identity,
            shape,
            __generated_glue::token(),
        );
        let program =
            StaticProgram::<String>::__from_compiler(header, b"program", __generated_glue::token());
        assert_eq!(program.verify_envelope(3, 1, 1, identity, shape), Ok(()));
        assert_eq!(
            program.verify_envelope(4, 1, 1, identity, shape),
            Err(StaticProgramEnvelopeError::FormatVersion)
        );
    }
}
