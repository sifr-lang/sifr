use super::{Decoder, Result, wire};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;
pub trait Decode<W>: Sized {
    fn decode(value: &W, cx: &mut Decoder) -> Result<Self>;
}
impl<T: Decode<W>, W> Decode<W> for Box<T> {
    fn decode(value: &W, cx: &mut Decoder) -> Result<Self> {
        T::decode(value, cx).map(Box::new)
    }
}
impl<T: Decode<W>, W> Decode<W> for std::sync::Arc<T> {
    fn decode(value: &W, cx: &mut Decoder) -> Result<Self> {
        T::decode(value, cx).map(std::sync::Arc::new)
    }
}
impl<T: Decode<W>, W> Decode<Option<W>> for Option<T> {
    fn decode(value: &Option<W>, cx: &mut Decoder) -> Result<Self> {
        value.as_ref().map(|v| T::decode(v, cx)).transpose()
    }
}
impl<T: Decode<W>, W> Decode<Vec<W>> for Vec<T> {
    fn decode(value: &Vec<W>, cx: &mut Decoder) -> Result<Self> {
        value.iter().map(|v| T::decode(v, cx)).collect()
    }
}
impl<A: Decode<X>, B: Decode<Y>, X, Y> Decode<(X, Y)> for (A, B) {
    fn decode(v: &(X, Y), cx: &mut Decoder) -> Result<Self> {
        Ok((A::decode(&v.0, cx)?, B::decode(&v.1, cx)?))
    }
}
impl<A: Decode<X>, B: Decode<Y>, C: Decode<Z>, X, Y, Z> Decode<(X, Y, Z)> for (A, B, C) {
    fn decode(v: &(X, Y, Z), cx: &mut Decoder) -> Result<Self> {
        Ok((
            A::decode(&v.0, cx)?,
            B::decode(&v.1, cx)?,
            C::decode(&v.2, cx)?,
        ))
    }
}
impl<K: Decode<X> + Hash + Eq, V: Decode<Y>, X, Y> Decode<BTreeMap<X, Y>> for HashMap<K, V> {
    fn decode(v: &BTreeMap<X, Y>, cx: &mut Decoder) -> Result<Self> {
        v.iter()
            .map(|(k, v)| Ok((K::decode(k, cx)?, V::decode(v, cx)?)))
            .collect()
    }
}
impl<K: Decode<X> + Ord, V: Decode<Y>, X, Y> Decode<BTreeMap<X, Y>> for BTreeMap<K, V> {
    fn decode(v: &BTreeMap<X, Y>, cx: &mut Decoder) -> Result<Self> {
        v.iter()
            .map(|(k, v)| Ok((K::decode(k, cx)?, V::decode(v, cx)?)))
            .collect()
    }
}
impl<T: Decode<W> + Hash + Eq, W> Decode<BTreeSet<W>> for HashSet<T> {
    fn decode(v: &BTreeSet<W>, cx: &mut Decoder) -> Result<Self> {
        v.iter().map(|v| T::decode(v, cx)).collect()
    }
}
impl<T: Decode<W> + Ord, W> Decode<BTreeSet<W>> for BTreeSet<T> {
    fn decode(v: &BTreeSet<W>, cx: &mut Decoder) -> Result<Self> {
        v.iter().map(|v| T::decode(v, cx)).collect()
    }
}
macro_rules! scalar {($($t:ty),*)=>{$(impl Decode<$t> for $t {fn decode(v:&$t,_:&mut Decoder)->Result<Self>{Ok(*v)}})*};}
scalar!(u8, u32, u64, i32, i64, bool, char);
impl Decode<u32> for usize {
    fn decode(v: &u32, _: &mut Decoder) -> Result<Self> {
        Ok(*v as usize)
    }
}
impl Decode<u64> for f64 {
    fn decode(v: &u64, _: &mut Decoder) -> Result<Self> {
        Ok(Self::from_bits(*v))
    }
}
impl Decode<wire::Ref<wire::Text>> for String {
    fn decode(v: &wire::Ref<wire::Text>, cx: &mut Decoder) -> Result<Self> {
        Ok(cx.store.get(*v)?.value.clone())
    }
}
impl Decode<wire::Ref<wire::Text>> for num_bigint::BigInt {
    fn decode(v: &wire::Ref<wire::Text>, cx: &mut Decoder) -> Result<Self> {
        String::decode(v, cx)?
            .parse()
            .map_err(|_| wire::MetadataError("invalid integer metadata".into()))
    }
}
impl<const N: usize> Decode<[u8; N]> for [u8; N] {
    fn decode(v: &[u8; N], _: &mut Decoder) -> Result<Self> {
        Ok(*v)
    }
}
impl<K: Decode<X> + Hash + Eq, V: Decode<Y>, X, Y> Decode<Vec<(X, Y)>> for HashMap<K, V> {
    fn decode(v: &Vec<(X, Y)>, cx: &mut Decoder) -> Result<Self> {
        v.iter()
            .map(|(k, v)| Ok((K::decode(k, cx)?, V::decode(v, cx)?)))
            .collect()
    }
}
impl<T: Decode<W>, W> Decode<Vec<W>> for sifr_type_system::SharedVec<T> {
    fn decode(value: &Vec<W>, cx: &mut Decoder) -> Result<Self> {
        value.iter().map(|v| T::decode(v, cx)).collect()
    }
}
