#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
pub struct ExtendedRecord { pub scalar: i32, pub text: Option<&'static str> }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
pub struct ExtendedTuple(pub i32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
pub struct ExtendedUnit;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
pub enum ExtendedEnum { #[default] Empty, Tuple(i32), Record { scalar: i32 } }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
pub struct Lifetime<'a> { pub value: Option<&'a str> }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct TwoLifetimes<'a: 'b, 'b, T: 'a> { pub first: &'a T, pub second: &'b T, pub static_ref: &'static str }
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
pub struct TestOnly<'a> { pub value: Option<&'a str> }

// More than five fields selects the compiler aggregate Debug formatter.
#[derive(Debug)]
pub struct AggregateRecord { pub a: i32, pub b: i32, pub c: i32, pub d: i32, pub e: i32, pub f: i32 }
#[derive(Debug)]
pub struct AggregateTuple(pub i32, pub i32, pub i32, pub i32, pub i32, pub i32);
#[derive(Debug)]
pub enum AggregateEnum {
    Record { a: i32, b: i32, c: i32, d: i32, e: i32, f: i32 },
    Tuple(i32, i32, i32, i32, i32, i32),
    Unit,
    Fixed(i32),
}

pub trait ObjectBase<'a, T, const N: usize>: std::fmt::Debug { type Output; }
pub trait ObjectChild<'a, T, const N: usize>: ObjectBase<'a, T, N> {}
pub trait BoundObject<'a>: std::fmt::Debug { type Output; }
#[derive(Debug)]
pub struct ObjectShapes<'a> {
    pub parameterized: &'a (dyn ObjectChild<'a, u8, 7, Output=u16> + Send + Sync + 'a),
    pub auto_only: std::marker::PhantomData<&'a (dyn Send + Sync + 'a)>,
    pub static_object: &'a (dyn std::fmt::Debug + 'static),
}

pub mod included_contracts { include!("included_contracts.rs"); }

pub fn local_left() {
    #[derive(Default)]
    struct LocalShape { flag: bool }
    let _ = LocalShape::default();
}
pub fn local_right() {
    #[derive(Default)]
    struct LocalShape { flag: bool }
    let _ = LocalShape::default();
}
