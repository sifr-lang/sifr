use std::fmt;
use std::marker::PhantomData;

use super::{ShapeIdentity, SlotTableIdentity, StaticProgramIdentity};
use crate::interop::GeneratedGlueToken;

pub const STATIC_PROGRAM_ENVELOPE_VERSION: u32 = 1;
pub const STATIC_PROGRAM_FORMAT_VERSION: u32 = 2;
pub const STRUCTURAL_BRIDGE_CONTRACT_VERSION: u32 = 1;

/// Borrowed compiler-emitted value produced by package const specialization.
///
/// This closed view is allocation-free. Consumers can traverse the verified
/// result without parsing its canonical byte representation.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StaticProgramValue {
    None,
    Bool(bool),
    Integer(&'static str),
    FloatBits(u64),
    String(&'static str),
    Bytes(&'static [u8]),
    Tuple(&'static [Self]),
    List(&'static [Self]),
    Record(&'static [(&'static str, Self)]),
    CallableIdentity {
        module: &'static str,
        owner: Option<&'static str>,
        symbol: &'static str,
        generic_arguments: &'static [&'static str],
        signature: &'static str,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StaticProgramHeader {
    envelope_version: u32,
    format_version: u32,
    structural_contract_version: u32,
    bridge_contract_version: u32,
    identity: StaticProgramIdentity,
    shape_identity: ShapeIdentity,
    slot_table_identity: Option<SlotTableIdentity>,
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
        slot_table_identity: Option<SlotTableIdentity>,
        _token: GeneratedGlueToken,
    ) -> Self {
        Self {
            envelope_version: STATIC_PROGRAM_ENVELOPE_VERSION,
            format_version,
            structural_contract_version,
            bridge_contract_version,
            identity,
            shape_identity,
            slot_table_identity,
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

    #[must_use]
    pub const fn slot_table_identity(&self) -> Option<SlotTableIdentity> {
        self.slot_table_identity
    }
}

/// Immutable compiler-emitted static data associated with one concrete Sifr type.
#[derive(Debug)]
pub struct StaticProgram<T> {
    header: StaticProgramHeader,
    bytes: &'static [u8],
    value: StaticProgramValue,
    _type: PhantomData<fn() -> T>,
}

/// Implemented only by compiler-emitted concrete structural types with retained static data.
pub trait StaticProgramType:
    super::StructuralType + super::StructuralConstruct + super::StructuralProject + Sized + 'static
{
    fn static_program() -> &'static StaticProgram<Self>;
}

impl<T> StaticProgram<T> {
    #[doc(hidden)]
    #[must_use]
    pub const fn __from_compiler(
        header: StaticProgramHeader,
        bytes: &'static [u8],
        value: StaticProgramValue,
        _token: GeneratedGlueToken,
    ) -> Self {
        Self {
            header,
            bytes,
            value,
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

    #[must_use]
    pub const fn value(&self) -> &StaticProgramValue {
        &self.value
    }

    pub fn verify_envelope(
        &self,
        format_version: u32,
        structural_contract_version: u32,
        bridge_contract_version: u32,
        identity: StaticProgramIdentity,
        shape_identity: ShapeIdentity,
        slot_table_identity: Option<SlotTableIdentity>,
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
        if self.header.slot_table_identity != slot_table_identity {
            return Err(StaticProgramEnvelopeError::SlotTableIdentity);
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
    SlotTableIdentity,
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
            Self::SlotTableIdentity => "static program method-slot table identity mismatch",
        })
    }
}

impl std::error::Error for StaticProgramEnvelopeError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interop::__generated_glue;
    use crate::interop::structural::{
        primitive, slot_table_identity, static_program_identity, SlotContextModeIdentity,
        StructuralConstruct, StructuralProject,
    };

    #[test]
    fn static_program_types_are_structurally_constructible_and_projectable() {
        #[allow(dead_code)]
        fn require_structural<T: StructuralConstruct + StructuralProject>() {}
        #[allow(dead_code)]
        fn require_static<T: StaticProgramType>() {
            require_structural::<T>();
        }
    }

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
            None,
            __generated_glue::token(),
        );
        let program = StaticProgram::<String>::__from_compiler(
            header,
            b"program",
            StaticProgramValue::Record(&[(
                "nodes",
                StaticProgramValue::List(&[StaticProgramValue::Integer("7")]),
            )]),
            __generated_glue::token(),
        );
        assert_eq!(
            program.verify_envelope(3, 1, 1, identity, shape, None),
            Ok(())
        );
        assert_eq!(
            program.value(),
            &StaticProgramValue::Record(&[(
                "nodes",
                StaticProgramValue::List(&[StaticProgramValue::Integer("7")]),
            )])
        );
        assert_eq!(
            program.verify_envelope(4, 1, 1, identity, shape, None),
            Err(StaticProgramEnvelopeError::FormatVersion)
        );
        let unexpected_slots = slot_table_identity(
            identity,
            primitive("method-slot-no-context"),
            SlotContextModeIdentity::None,
            &[],
        );
        assert_eq!(
            program.verify_envelope(3, 1, 1, identity, shape, Some(unexpected_slots)),
            Err(StaticProgramEnvelopeError::SlotTableIdentity)
        );
    }
}
