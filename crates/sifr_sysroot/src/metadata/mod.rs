//! Private indexed stdlib wire schema. Version 2 compresses the physical container; normal commands validate its complete digest before opening.
//!
//! Every non-scalar payload reference names a typed directory record, including
//! strings and type occurrences. Wire records never contain live HIR or `Type`.
//! Float literals carry IEEE bits; source positions are packaged-source relative;
//! binding slots are scoped by a stable declaration binder. No projection into
//! owned compiler values is provided here (the DX.7 provider owns that boundary).
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;

mod container;
mod decode;
mod directory;
mod physical;
mod store;
mod validation;
pub use container::{Compatibility, Limits, MetadataEncoder};
pub use store::MetadataStore;
mod hir_nodes;
pub use hir_nodes::*;
mod hir_expr_refs;
mod hir_nodes_refs;
mod specialization_metadata;
pub use specialization_metadata::*;
mod rust_interop;
mod specialization_metadata_refs;
pub use rust_interop::*;
mod python_interop;
mod rust_interop_refs;
pub use python_interop::*;
mod python_interop_refs;
mod template_strings;
pub use template_strings::*;
mod sql_queries;
mod template_strings_refs;
pub use sql_queries::*;
mod sql_migrations;
mod sql_queries_refs;
pub use sql_migrations::*;
mod sql_migrations_refs;
mod type_records;
pub use type_records::*;
mod container_records;
mod type_records_refs;
pub use container_records::*;
mod container_records_refs;
mod semantic_records;
pub use semantic_records::*;
mod semantic_records_refs;
pub const KIND_COUNT: u16 = 134;

pub type RecordId = [u8; 32];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataError(pub String);
impl std::fmt::Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid stdlib metadata: {}; rebuild development metadata or select a matching installed toolchain",
            self.0
        )
    }
}
impl std::error::Error for MetadataError {}
fn err(message: impl Into<String>) -> MetadataError {
    MetadataError(message.into())
}
pub type Result<T> = std::result::Result<T, MetadataError>;

/// A typed, artifact-local reference. It is never a compiler nominal identity by
/// itself: the store and resolved package/declaration identity remain required.
#[derive(Debug)]
pub struct Ref<T> {
    id: RecordId,
    marker: PhantomData<fn() -> T>,
}
impl<T> Serialize for Ref<T> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        let text = id_text(&self.id);
        serializer.serialize_str(&text)
    }
}
impl<'de, T> Deserialize<'de> for Ref<T> {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        if text.len() != 64
            || !text
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(serde::de::Error::custom(
                "expected canonical 32-byte record ID",
            ));
        }
        let mut id = [0; 32];
        for (i, pair) in text.as_bytes().as_chunks::<2>().0.iter().enumerate() {
            let digit = |b: u8| if b <= b'9' { b - b'0' } else { b - b'a' + 10 };
            id[i] = digit(pair[0]) * 16 + digit(pair[1]);
        }
        Ok(Self {
            id,
            marker: PhantomData,
        })
    }
}
impl<T> Copy for Ref<T> {}
impl<T> Clone for Ref<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> PartialEq for Ref<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for Ref<T> {}
impl<T> PartialOrd for Ref<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Ord for Ref<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}
impl<T: Record> Ref<T> {
    /// Length-delimited anchors must include package/version/source, module and
    /// declaration/binder path where applicable. Content records use `intern`.
    pub fn anchor(parts: &[&[u8]]) -> Self {
        let mut hash = Sha256::new();
        hash.update(b"sifrmeta-anchor-v1");
        hash.update(T::KIND.to_le_bytes());
        for part in parts {
            hash.update((part.len() as u64).to_le_bytes());
            hash.update(part);
        }
        Self {
            id: hash.finalize().into(),
            marker: PhantomData,
        }
    }
    #[must_use]
    pub fn id(self) -> RecordId {
        self.id
    }
}

mod sealed {
    pub trait Sealed {}
}

pub trait Record:
    sealed::Sealed
    + References
    + Serialize
    + serde::de::DeserializeOwned
    + std::fmt::Debug
    + Send
    + Sync
    + 'static
{
    const KIND: u16;
}
pub trait References {
    fn references(&self, out: &mut Vec<(RecordId, u16)>);
}
impl<T: Record> References for Ref<T> {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        out.push((self.id, T::KIND));
    }
}
macro_rules! scalar_refs { ($($t:ty),*) => {$(impl References for $t { fn references(&self, _: &mut Vec<(RecordId,u16)>) {} })*}; }
scalar_refs!(String, char, bool, u8, u32, u64, i32, i64);
impl<const N: usize> References for [u8; N] {
    fn references(&self, _: &mut Vec<(RecordId, u16)>) {}
}
impl<T: References> References for Vec<T> {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        for v in self {
            v.references(out);
        }
    }
}
impl<T: References> References for Option<T> {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        if let Some(v) = self {
            v.references(out);
        }
    }
}
impl<T: References> References for BTreeSet<T> {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        for v in self {
            v.references(out);
        }
    }
}
impl<K: References, V: References> References for BTreeMap<K, V> {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        for (k, v) in self {
            k.references(out);
            v.references(out);
        }
    }
}
impl<A: References, B: References> References for (A, B) {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        self.0.references(out);
        self.1.references(out);
    }
}
impl<A: References, B: References, C: References> References for (A, B, C) {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        self.0.references(out);
        self.1.references(out);
        self.2.references(out);
    }
}

#[cfg(test)]
mod tests;

fn id_text(id: &RecordId) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(64);
    for byte in id {
        text.push(char::from(HEX[usize::from(byte >> 4)]));
        text.push(char::from(HEX[usize::from(byte & 15)]));
    }
    text
}

pub type TypeParameterBounds = BTreeMap<Ref<Text>, BTreeMap<Ref<Text>, Vec<Ref<Text>>>>;
