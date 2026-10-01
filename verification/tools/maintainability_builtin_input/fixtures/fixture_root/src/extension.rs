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
