use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use crate::interop::HandleStateError;

use super::{
    ConstructToken, NodeId, ShapeIdentity, StructuralConstruct, StructuralContractError,
    StructuralProject, StructuralSource, StructuralType, StructuralVisitor,
};

/// Package-owned construction and projection for one native structural value.
///
/// The mapping type and native value type are both selected by a checked Sifr
/// declaration. Implementations remain in the native package; generated code
/// only names this trait and [`MappedValue`].
pub trait StructuralMapping<T> {
    fn shape_identity() -> ShapeIdentity;

    fn nominal_identity() -> Option<&'static str> {
        None
    }

    fn structural_construct_at<S: StructuralSource>(
        source: &mut S,
        node: NodeId,
        token: ConstructToken,
    ) -> Result<T, StructuralContractError>;

    fn structural_project<'value, V: StructuralVisitor<'value>>(
        value: &'value T,
        visitor: &mut V,
    ) -> Result<(), V::Error>;
}

/// Stable runtime carrier for a package-mapped native structural value.
///
/// Unlike an opaque resource handle, this value has no closed or poisoned
/// state. The handle-shaped accessors let existing generated Rust method glue
/// borrow or consume the native value without exposing its representation.
///
/// The mapping marker has no runtime state and uses `PhantomData<fn() -> M>`.
/// Consequently, `Send` and `Sync` follow `T` alone. A package must encode any
/// thread-safety restriction in the stored native value, not in `M`.
pub struct MappedValue<T, M> {
    value: T,
    _mapping: PhantomData<fn() -> M>,
}

impl<T, M> MappedValue<T, M> {
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self {
            value,
            _mapping: PhantomData,
        }
    }

    pub const fn inner_ref(&self) -> Result<&T, HandleStateError> {
        Ok(&self.value)
    }

    pub const fn inner_mut(&mut self) -> Result<&mut T, HandleStateError> {
        Ok(&mut self.value)
    }

    pub fn into_inner(self) -> Result<T, HandleStateError> {
        Ok(self.value)
    }
}

impl<T: Clone, M> Clone for MappedValue<T, M> {
    fn clone(&self) -> Self {
        Self::new(self.value.clone())
    }
}

impl<T: fmt::Debug, M> fmt::Debug for MappedValue<T, M> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(formatter)
    }
}

impl<T: PartialEq, M> PartialEq for MappedValue<T, M> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq, M> Eq for MappedValue<T, M> {}

impl<T: Hash, M> Hash for MappedValue<T, M> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T, M: StructuralMapping<T>> StructuralType for MappedValue<T, M> {
    fn shape_identity() -> ShapeIdentity {
        M::shape_identity()
    }

    fn nominal_identity() -> Option<&'static str> {
        M::nominal_identity()
    }
}

impl<T, M: StructuralMapping<T>> StructuralConstruct for MappedValue<T, M> {
    fn structural_construct_at<S: StructuralSource>(
        source: &mut S,
        node: NodeId,
        token: ConstructToken,
    ) -> Result<Self, StructuralContractError> {
        M::structural_construct_at(source, node, token).map(Self::new)
    }
}

impl<T, M: StructuralMapping<T>> StructuralProject for MappedValue<T, M> {
    fn structural_project<'value, V: StructuralVisitor<'value>>(
        &'value self,
        visitor: &mut V,
    ) -> Result<(), V::Error> {
        M::structural_project(&self.value, visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interop::structural::{
        primitive, structural_construct, ArenaNode, StructuralArena, StructuralKind,
        StructuralScalar, StructuralScalarRef,
    };

    struct StringMapping;

    impl StructuralMapping<String> for StringMapping {
        fn shape_identity() -> ShapeIdentity {
            primitive("str")
        }

        fn structural_construct_at<S: StructuralSource>(
            source: &mut S,
            node: NodeId,
            token: ConstructToken,
        ) -> Result<String, StructuralContractError> {
            String::structural_construct_at(source, node, token)
        }

        fn structural_project<'value, V: StructuralVisitor<'value>>(
            value: &'value String,
            visitor: &mut V,
        ) -> Result<(), V::Error> {
            value.structural_project(visitor)
        }
    }

    #[derive(Default)]
    struct StringVisitor {
        value: Option<String>,
    }

    impl<'value> StructuralVisitor<'value> for StringVisitor {
        type Error = StructuralContractError;

        fn enter(
            &mut self,
            _event: super::super::StructuralEnter<'value>,
        ) -> Result<super::super::VisitControl, Self::Error> {
            Err(StructuralContractError::KindMismatch)
        }

        fn edge(&mut self, _edge: super::super::StructuralEdge<'value>) -> Result<(), Self::Error> {
            Err(StructuralContractError::KindMismatch)
        }

        fn scalar(&mut self, scalar: StructuralScalarRef<'value>) -> Result<(), Self::Error> {
            let StructuralScalarRef::String(value) = scalar else {
                return Err(StructuralContractError::ScalarMismatch);
            };
            self.value = Some(value.to_string());
            Ok(())
        }

        fn exit(&mut self, _kind: StructuralKind) -> Result<(), Self::Error> {
            Err(StructuralContractError::KindMismatch)
        }
    }

    #[test]
    fn mapped_value_constructs_projects_and_rejects_shape_mismatch() {
        type Value = MappedValue<String, StringMapping>;
        let arena = StructuralArena::seal(
            Value::shape_identity(),
            NodeId::new(0),
            vec![ArenaNode::scalar(
                StructuralKind::String,
                StructuralScalar::String("mapped".to_string()),
            )],
        )
        .expect("mapped source should seal");
        let value = structural_construct::<Value, _>(arena).expect("mapping should construct");
        assert_eq!(
            value.inner_ref().expect("mapped value is always open"),
            "mapped"
        );

        let mut visitor = StringVisitor::default();
        value
            .structural_project(&mut visitor)
            .expect("mapping should project");
        assert_eq!(visitor.value.as_deref(), Some("mapped"));

        let mismatch = StructuralArena::seal(
            primitive("bytes"),
            NodeId::new(0),
            vec![ArenaNode::scalar(
                StructuralKind::String,
                StructuralScalar::String("mapped".to_string()),
            )],
        )
        .expect("mismatched source should still seal");
        assert_eq!(
            structural_construct::<Value, _>(mismatch),
            Err(StructuralContractError::ShapeMismatch)
        );
    }

    #[test]
    fn mapped_value_preserves_send_and_sync_backstops() {
        struct NonThreadSafeMarker {
            _not_send_sync: *mut (),
        }

        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<MappedValue<String, StringMapping>>();
        assert_send_sync::<MappedValue<String, NonThreadSafeMarker>>();
    }
}
