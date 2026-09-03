use super::{Type, type_queries::parent_chain_contains};
use std::collections::HashSet;

impl Type {
    /// Whether a generated Rust aggregate containing this type may derive
    /// `Eq`. Recursive classes are accepted when every reachable field has an
    /// equality-total representation.
    #[must_use]
    pub fn supports_derived_eq(&self) -> bool {
        self.supports_derived_eq_inner(&mut HashSet::new())
    }

    fn supports_derived_eq_inner(
        &self,
        visiting_classes: &mut HashSet<(String, Vec<Self>)>,
    ) -> bool {
        match self.resolve_alias() {
            Self::Int
            | Self::FixedInt(_)
            | Self::Bool
            | Self::Str
            | Self::Bytes
            | Self::None
            | Self::Range
            | Self::LiteralInt(_)
            | Self::LiteralStr(_)
            | Self::LiteralBool(_)
            | Self::Enum { .. }
            | Self::Decimal
            | Self::BigDecimal
            | Self::TypeVar(_) => true,
            Self::List(element) => element.supports_derived_eq_inner(visiting_classes),
            Self::Set(element) => element.supports_hash_key_inner(visiting_classes),
            Self::Dict(key, value) => {
                key.supports_derived_eq_inner(visiting_classes)
                    && value.supports_derived_eq_inner(visiting_classes)
            }
            Self::Tuple(elements) | Self::Union(elements) => elements
                .iter()
                .all(|element| element.supports_derived_eq_inner(visiting_classes)),
            Self::StructuralRecord(record) => record
                .fields()
                .iter()
                .all(|field| field.ty().supports_derived_eq_inner(visiting_classes)),
            Self::Result(ok, error) => {
                ok.supports_derived_eq_inner(visiting_classes)
                    && error.supports_derived_eq_inner(visiting_classes)
            }
            Self::Newtype { inner, .. } => inner.supports_derived_eq_inner(visiting_classes),
            Self::Class {
                fields,
                methods,
                parent_class,
                ..
            } => {
                if parent_chain_contains(parent_class.as_deref(), "NonSend")
                    || methods.iter().any(|(method, _)| method == "__eq__")
                {
                    return false;
                }
                let Some(key) = self.class_recursion_key() else {
                    return false;
                };
                if !visiting_classes.insert(key.clone()) {
                    return true;
                }
                let supports = fields
                    .iter()
                    .all(|(_, field)| field.supports_derived_eq_inner(visiting_classes));
                visiting_classes.remove(&key);
                supports
            }
            _ => false,
        }
    }

    /// Whether the generated representation implements Rust's total ordering
    /// traits. Structural records require every field to have total order.
    #[must_use]
    pub fn supports_total_order(&self) -> bool {
        match self.resolve_alias() {
            Self::Int
            | Self::FixedInt(_)
            | Self::Bool
            | Self::Str
            | Self::Bytes
            | Self::None
            | Self::Range
            | Self::LiteralInt(_)
            | Self::LiteralStr(_)
            | Self::LiteralBool(_)
            | Self::Decimal
            | Self::BigDecimal => true,
            Self::List(element) => element.supports_total_order(),
            Self::Tuple(elements) => elements.iter().all(Self::supports_total_order),
            Self::StructuralRecord(record) => record
                .fields()
                .iter()
                .all(|field| field.ty().supports_total_order()),
            _ => false,
        }
    }

    /// Whether Rust aggregate generation may derive conditional `Eq + Hash`.
    ///
    /// Type variables are accepted because the generated implementation adds
    /// the required bounds. Containers whose Rust representation is never
    /// hashable remain rejected.
    #[must_use]
    pub fn supports_derived_hash(&self) -> bool {
        self.supports_derived_hash_inner(&mut HashSet::new())
    }

    fn supports_derived_hash_inner(
        &self,
        visiting_classes: &mut HashSet<(String, Vec<Self>)>,
    ) -> bool {
        match self.resolve_alias() {
            Self::TypeVar(_) => true,
            Self::Tuple(elements) | Self::Union(elements) => elements
                .iter()
                .all(|element| element.supports_derived_hash_inner(visiting_classes)),
            Self::StructuralRecord(record) => record
                .fields()
                .iter()
                .all(|field| field.ty().supports_derived_hash_inner(visiting_classes)),
            Self::Result(ok, error) => {
                ok.supports_derived_hash_inner(visiting_classes)
                    && error.supports_derived_hash_inner(visiting_classes)
            }
            Self::Newtype { inner, .. } => inner.supports_derived_hash_inner(visiting_classes),
            Self::Class {
                fields,
                methods,
                parent_class,
                ..
            } => {
                if parent_chain_contains(parent_class.as_deref(), "NonSend")
                    || methods.iter().any(|(method, _)| method == "__eq__")
                {
                    return false;
                }
                let Some(key) = self.class_recursion_key() else {
                    return false;
                };
                if !visiting_classes.insert(key.clone()) {
                    return true;
                }
                let supports = fields
                    .iter()
                    .all(|(_, field)| field.supports_derived_hash_inner(visiting_classes));
                visiting_classes.remove(&key);
                supports
            }
            Self::Template(_) => false,
            _ => self.supports_hash_key(),
        }
    }

    /// Whether the generated Rust representation implements `Eq + Hash` and
    /// can therefore be used as a `HashSet` element or `HashMap` key.
    #[must_use]
    pub fn supports_hash_key(&self) -> bool {
        self.supports_hash_key_inner(&mut HashSet::new())
    }

    pub(super) fn supports_hash_key_inner(
        &self,
        visiting_classes: &mut HashSet<(String, Vec<Self>)>,
    ) -> bool {
        match self.resolve_alias() {
            Self::Int
            | Self::FixedInt(_)
            | Self::Bool
            | Self::Str
            | Self::Bytes
            | Self::None
            | Self::Range
            | Self::LiteralInt(_)
            | Self::LiteralStr(_)
            | Self::LiteralBool(_)
            | Self::Enum { .. }
            | Self::Decimal => true,
            Self::Tuple(elements) | Self::Union(elements) => elements
                .iter()
                .all(|element| element.supports_hash_key_inner(visiting_classes)),
            Self::StructuralRecord(record) => record
                .fields()
                .iter()
                .all(|field| field.ty().supports_hash_key_inner(visiting_classes)),
            Self::Result(ok, error) => {
                ok.supports_hash_key_inner(visiting_classes)
                    && error.supports_hash_key_inner(visiting_classes)
            }
            Self::Class {
                fields,
                methods,
                parent_class,
                type_args,
                ..
            } => {
                if parent_chain_contains(parent_class.as_deref(), "NonSend")
                    || methods.iter().any(|(method, _)| method == "__eq__")
                {
                    return false;
                }
                let Some(key) = self.class_recursion_key() else {
                    return false;
                };
                if !visiting_classes.insert(key.clone()) {
                    return true;
                }
                let supports = fields
                    .iter()
                    .all(|(_, field)| field.supports_hash_key_inner(visiting_classes))
                    && type_args.iter().all(|argument| {
                        matches!(argument.resolve_alias(), Self::TypeVar(_))
                            || argument.supports_hash_key_inner(visiting_classes)
                    });
                visiting_classes.remove(&key);
                supports
            }
            Self::Newtype { inner, .. } => inner.supports_hash_key_inner(visiting_classes),
            Self::Template(_) => false,
            _ => false,
        }
    }

    /// Whether the generated Rust representation implements `Debug`.
    #[must_use]
    pub fn supports_debug_formatting(&self) -> bool {
        self.supports_debug_formatting_inner(&mut HashSet::new())
    }

    fn supports_debug_formatting_inner(
        &self,
        visiting_classes: &mut HashSet<(String, Vec<Self>)>,
    ) -> bool {
        match self.resolve_alias() {
            Self::Any
            | Self::Unknown
            | Self::Function(_)
            | Self::AsyncFunction(_)
            | Self::Protocol { .. }
            | Self::Callable(..)
            | Self::AsyncCallable(..)
            | Self::Coroutine(..)
            | Self::Task(..)
            | Self::BlockingTask(..)
            | Self::JoinSet(..)
            | Self::Awaitable(_)
            | Self::Iterator(_)
            | Self::AsyncIterator(..)
            | Self::AsyncGenerator(..)
            | Self::Template(_)
            | Self::Intersection(_)
            | Self::TypeVar(_) => false,
            Self::List(element)
            | Self::Set(element)
            | Self::Iterable(element)
            | Self::Newtype { inner: element, .. } => {
                element.supports_debug_formatting_inner(visiting_classes)
            }
            Self::Dict(key, value) | Self::Result(key, value) => {
                key.supports_debug_formatting_inner(visiting_classes)
                    && value.supports_debug_formatting_inner(visiting_classes)
            }
            Self::TaskResult(ok, error) | Self::Select2(ok, error) => {
                ok.supports_debug_formatting_inner(visiting_classes)
                    && error.supports_debug_formatting_inner(visiting_classes)
            }
            Self::Failure(error) | Self::TimeoutResult(error) => {
                error.supports_debug_formatting_inner(visiting_classes)
            }
            Self::Tuple(elements) | Self::Union(elements) => elements
                .iter()
                .all(|element| element.supports_debug_formatting_inner(visiting_classes)),
            Self::StructuralRecord(record) => record
                .fields()
                .iter()
                .all(|field| field.ty().supports_debug_formatting_inner(visiting_classes)),
            Self::Class {
                fields,
                parent_class,
                type_args,
                ..
            } => {
                // Most `NonSend` classes wrap runtime resources whose emitted
                // structs intentionally do not derive `Debug`. A few package-
                // specific resources receive a bespoke derive in codegen, but
                // the type alone cannot identify those declarations, so keep
                // the shared capability query conservative.
                if parent_chain_contains(parent_class.as_deref(), "NonSend") {
                    return false;
                }
                let Some(key) = self.class_recursion_key() else {
                    return false;
                };
                if !visiting_classes.insert(key.clone()) {
                    return true;
                }
                let supports = fields
                    .iter()
                    .all(|(_, field)| field.supports_debug_formatting_inner(visiting_classes))
                    && type_args.iter().all(|argument| {
                        matches!(argument.resolve_alias(), Self::TypeVar(_))
                            || argument.supports_debug_formatting_inner(visiting_classes)
                    });
                visiting_classes.remove(&key);
                supports
            }
            _ => true,
        }
    }

    /// Whether the generated Rust representation implements `Display`.
    #[must_use]
    pub fn supports_display_formatting(&self) -> bool {
        self.supports_display_formatting_inner(&mut HashSet::new())
    }

    fn supports_display_formatting_inner(
        &self,
        visiting_classes: &mut HashSet<(String, Vec<Self>)>,
    ) -> bool {
        match self.resolve_alias() {
            Self::Int
            | Self::FixedInt(_)
            | Self::Float
            | Self::Bool
            | Self::Str
            | Self::LiteralInt(_)
            | Self::LiteralStr(_)
            | Self::LiteralBool(_)
            | Self::Enum { .. }
            | Self::Decimal
            | Self::BigDecimal => true,
            Self::Union(elements) => {
                let is_option = elements.len() == 2
                    && elements
                        .iter()
                        .any(|element| matches!(element.resolve_alias(), Self::None));
                !is_option
                    && elements.iter().all(|element| {
                        element.supports_display_formatting_inner(visiting_classes)
                            || element.supports_debug_formatting_inner(visiting_classes)
                    })
            }
            Self::StructuralRecord(record) => record.fields().iter().all(|field| {
                field
                    .ty()
                    .supports_display_formatting_inner(visiting_classes)
            }),
            Self::Class {
                name,
                fields,
                methods,
                ..
            } => {
                // `JoinItemId` is a compiler-owned runtime wrapper with a
                // bespoke `Display` implementation rather than a Sifr method.
                if name == "JoinItemId" || methods.iter().any(|(method, _)| method == "__str__") {
                    return true;
                }
                let Some(key) = self.class_recursion_key() else {
                    return false;
                };
                if fields.is_empty() || !visiting_classes.insert(key.clone()) {
                    return false;
                }
                let supports = fields
                    .iter()
                    .all(|(_, field)| field.supports_display_formatting_inner(visiting_classes));
                visiting_classes.remove(&key);
                supports
            }
            Self::Newtype { inner, .. } => {
                inner.supports_display_formatting_inner(visiting_classes)
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Type;

    #[test]
    fn derived_eq_capability_tracks_nested_float_fields() {
        assert!(Type::List(Box::new(Type::Int)).supports_derived_eq());
        assert!(!Type::Float.supports_derived_eq());
        assert!(!Type::List(Box::new(Type::Float)).supports_derived_eq());
    }
}
