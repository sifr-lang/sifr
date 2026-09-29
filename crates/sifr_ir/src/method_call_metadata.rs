//! Source method-call authority, ranges, and checked mutable places.

use crate::{BindingId, CallableIdentity};
use ruff_text_size::TextRange;

/// Resolved dispatch owner for a source method call. `Unclassified` is an
/// explicit transition state until lowering assigns a declaration in H02a1;
/// consumers must never interpret it as builtin dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodAuthority {
    Unclassified,
    BuiltinIntrinsic { declaration: CallableIdentity },
    Protocol { declaration: CallableIdentity },
    LocalNominal { declaration: CallableIdentity },
    InheritedNominal { declaration: CallableIdentity },
    Imported { declaration: CallableIdentity },
    RustAdapted { declaration: CallableIdentity },
}

/// Source ranges retained for ownership diagnostics on source method calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodCallSource {
    pub call_range: TextRange,
    pub receiver_range: TextRange,
    pub arg_ranges: Vec<TextRange>,
}

/// Stable identity of a field projection used by ownership-place analysis.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldIdentity {
    pub declaring_class: String,
    pub field: String,
}

/// A projection from a binding root to checked storage.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlaceProjection {
    Field(FieldIdentity),
}

/// A checked source storage place.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Place {
    pub root: BindingId,
    pub projections: Vec<PlaceProjection>,
}

/// Proven target shape for a mutable method receiver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutableReceiverTarget {
    Place(Place),
    OwnedTemporary,
    /// Compiler-owned indexed container mutation with a separately audited
    /// lowering. The base place is retained for exclusivity checks.
    SpecializedIndexedStorage(Place),
}

/// Proven target shape for a mutable call argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutableArgumentTarget {
    Place(Place),
    OwnedTemporary,
}
