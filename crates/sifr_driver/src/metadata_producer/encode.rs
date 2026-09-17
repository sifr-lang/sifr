use super::{Encoder, Result, wire};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

pub(super) trait Encode<T> {
    fn encode(&self, cx: &mut Encoder) -> Result<T>;
}
impl<T: ?Sized, U> Encode<U> for Box<T>
where
    T: Encode<U>,
{
    fn encode(&self, cx: &mut Encoder) -> Result<U> {
        (**self).encode(cx)
    }
}
impl<T: ?Sized, U> Encode<U> for std::sync::Arc<T>
where
    T: Encode<U>,
{
    fn encode(&self, cx: &mut Encoder) -> Result<U> {
        (**self).encode(cx)
    }
}
impl<T: ?Sized, U> Encode<U> for &T
where
    T: Encode<U>,
{
    fn encode(&self, cx: &mut Encoder) -> Result<U> {
        (**self).encode(cx)
    }
}
impl<T, U> Encode<Option<U>> for Option<T>
where
    T: Encode<U>,
{
    fn encode(&self, cx: &mut Encoder) -> Result<Option<U>> {
        self.as_ref().map(|v| v.encode(cx)).transpose()
    }
}
impl<T, U> Encode<Vec<U>> for [T]
where
    T: Encode<U>,
{
    fn encode(&self, cx: &mut Encoder) -> Result<Vec<U>> {
        self.iter().map(|v| v.encode(cx)).collect()
    }
}
impl<T, U> Encode<Vec<U>> for Vec<T>
where
    T: Encode<U>,
{
    fn encode(&self, cx: &mut Encoder) -> Result<Vec<U>> {
        self.as_slice().encode(cx)
    }
}
impl<A, B, X, Y> Encode<(X, Y)> for (A, B)
where
    A: Encode<X>,
    B: Encode<Y>,
{
    fn encode(&self, cx: &mut Encoder) -> Result<(X, Y)> {
        Ok((self.0.encode(cx)?, self.1.encode(cx)?))
    }
}
impl<A, B, C, X, Y, Z> Encode<(X, Y, Z)> for (A, B, C)
where
    A: Encode<X>,
    B: Encode<Y>,
    C: Encode<Z>,
{
    fn encode(&self, cx: &mut Encoder) -> Result<(X, Y, Z)> {
        Ok((self.0.encode(cx)?, self.1.encode(cx)?, self.2.encode(cx)?))
    }
}
impl<K: Encode<X> + Ord + Hash + Eq, V: Encode<Y>, X: Ord, Y> Encode<BTreeMap<X, Y>>
    for HashMap<K, V>
{
    fn encode(&self, cx: &mut Encoder) -> Result<BTreeMap<X, Y>> {
        let mut entries = self.iter().collect::<Vec<_>>();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        entries
            .into_iter()
            .map(|(k, v)| Ok((k.encode(cx)?, v.encode(cx)?)))
            .collect()
    }
}
impl<K: Encode<X> + Ord, V: Encode<Y>, X: Ord, Y> Encode<BTreeMap<X, Y>> for BTreeMap<K, V> {
    fn encode(&self, cx: &mut Encoder) -> Result<BTreeMap<X, Y>> {
        self.iter()
            .map(|(k, v)| Ok((k.encode(cx)?, v.encode(cx)?)))
            .collect()
    }
}
impl<T: Encode<U> + Ord + Hash + Eq, U: Ord> Encode<BTreeSet<U>> for HashSet<T> {
    fn encode(&self, cx: &mut Encoder) -> Result<BTreeSet<U>> {
        let mut values = self.iter().collect::<Vec<_>>();
        values.sort();
        values.into_iter().map(|v| v.encode(cx)).collect()
    }
}
impl<T: Encode<U> + Ord, U: Ord> Encode<BTreeSet<U>> for BTreeSet<T> {
    fn encode(&self, cx: &mut Encoder) -> Result<BTreeSet<U>> {
        self.iter().map(|v| v.encode(cx)).collect()
    }
}
macro_rules! scalar { ($($t:ty),*) => {$ (impl Encode<$t> for $t { fn encode(&self,_:&mut Encoder)->Result<$t>{Ok(*self)} })*}; }
scalar!(u8, u32, u64, i32, i64, bool, char);
impl Encode<u32> for usize {
    fn encode(&self, _: &mut Encoder) -> Result<u32> {
        u32::try_from(*self).map_err(|_| wire::MetadataError("index exceeds wire range".into()))
    }
}
impl Encode<u64> for f64 {
    fn encode(&self, _: &mut Encoder) -> Result<u64> {
        Ok(self.to_bits())
    }
}
impl Encode<wire::Ref<wire::Text>> for str {
    fn encode(&self, cx: &mut Encoder) -> Result<wire::Ref<wire::Text>> {
        cx.records.intern(&wire::Text {
            value: self.to_owned(),
        })
    }
}
impl Encode<wire::Ref<wire::Text>> for String {
    fn encode(&self, cx: &mut Encoder) -> Result<wire::Ref<wire::Text>> {
        self.as_str().encode(cx)
    }
}

impl<const N: usize> Encode<[u8; N]> for [u8; N] {
    fn encode(&self, _: &mut Encoder) -> Result<[u8; N]> {
        Ok(*self)
    }
}

// The frontend's try-error collection is a set. Its display-name sort can tie
// for distinct views of one nominal type; canonicalize by the complete record.
pub(super) fn error_type_set(
    values: &[sifr_type_system::Type],
    cx: &mut Encoder,
) -> Result<Vec<wire::Ref<wire::Type>>> {
    let mut references: Vec<wire::Ref<wire::Type>> = values.encode(cx)?;
    references.sort();
    Ok(references)
}
impl<T: Encode<U>, U> Encode<Vec<U>> for sifr_type_system::SharedVec<T> {
    fn encode(&self, cx: &mut Encoder) -> Result<Vec<U>> {
        self.as_slice().encode(cx)
    }
}
