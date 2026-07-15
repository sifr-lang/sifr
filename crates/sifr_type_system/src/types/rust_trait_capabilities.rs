use super::{type_queries::parent_chain_contains, Type};
use std::collections::BTreeSet;

impl Type {
    /// Whether the generated Rust representation implements `Eq + Hash` and
    /// can therefore be used as a `HashSet` element or `HashMap` key.
    #[must_use]
    pub fn supports_hash_key(&self) -> bool {
        self.supports_hash_key_inner(&mut BTreeSet::new())
    }

    pub(super) fn supports_hash_key_inner(&self, visiting_classes: &mut BTreeSet<String>) -> bool {
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
            | Self::BigInt
            | Self::Decimal => true,
            Self::Tuple(elements) | Self::Union(elements) => elements
                .iter()
                .all(|element| element.supports_hash_key_inner(visiting_classes)),
            Self::Result(ok, error) => {
                ok.supports_hash_key_inner(visiting_classes)
                    && error.supports_hash_key_inner(visiting_classes)
            }
            Self::Class {
                name,
                fields,
                methods,
                parent_class,
            } => {
                if parent_chain_contains(parent_class.as_deref(), "NonSend")
                    || methods.iter().any(|(method, _)| method == "__eq__")
                {
                    return false;
                }
                if !visiting_classes.insert(name.clone()) {
                    return true;
                }
                let supports = fields
                    .iter()
                    .all(|(_, field)| field.supports_hash_key_inner(visiting_classes));
                visiting_classes.remove(name);
                supports
            }
            Self::Newtype { inner, .. } => inner.supports_hash_key_inner(visiting_classes),
            _ => false,
        }
    }

    /// Whether the generated Rust representation implements `Debug`.
    #[must_use]
    pub fn supports_debug_formatting(&self) -> bool {
        self.supports_debug_formatting_inner(&mut BTreeSet::new())
    }

    fn supports_debug_formatting_inner(&self, visiting_classes: &mut BTreeSet<String>) -> bool {
        match self.resolve_alias() {
            Self::Any
            | Self::Unknown
            | Self::Never
            | Self::Function(_)
            | Self::AsyncFunction(_)
            | Self::Protocol { .. }
            | Self::Callable(..)
            | Self::AsyncCallable(..)
            | Self::Coroutine(..)
            | Self::Task(..)
            | Self::TaskResult(..)
            | Self::Failure(_)
            | Self::TimeoutResult(_)
            | Self::Select2(..)
            | Self::BlockingTask(..)
            | Self::JoinSet(..)
            | Self::Awaitable(_)
            | Self::Iterator(_)
            | Self::AsyncIterator(..)
            | Self::AsyncGenerator(..)
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
            Self::Tuple(elements) | Self::Union(elements) => elements
                .iter()
                .all(|element| element.supports_debug_formatting_inner(visiting_classes)),
            Self::Class {
                name,
                fields,
                parent_class,
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
                if !visiting_classes.insert(name.clone()) {
                    return true;
                }
                let supports = fields
                    .iter()
                    .all(|(_, field)| field.supports_debug_formatting_inner(visiting_classes));
                visiting_classes.remove(name);
                supports
            }
            _ => true,
        }
    }

    /// Whether the generated Rust representation implements `Display`.
    #[must_use]
    pub fn supports_display_formatting(&self) -> bool {
        self.supports_display_formatting_inner(&mut BTreeSet::new())
    }

    fn supports_display_formatting_inner(&self, visiting_classes: &mut BTreeSet<String>) -> bool {
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
            | Self::BigInt
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
            Self::Class {
                name,
                fields,
                methods,
                ..
            } => {
                if methods.iter().any(|(method, _)| method == "__str__") {
                    return true;
                }
                if fields.is_empty() || !visiting_classes.insert(name.clone()) {
                    return false;
                }
                let supports = fields
                    .iter()
                    .all(|(_, field)| field.supports_display_formatting_inner(visiting_classes));
                visiting_classes.remove(name);
                supports
            }
            Self::Newtype { inner, .. } => {
                inner.supports_display_formatting_inner(visiting_classes)
            }
            _ => false,
        }
    }
}
