use std::convert::Infallible;
use std::marker::{PhantomData, PhantomPinned};
use std::num::{
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
};
use std::sync::atomic::{
    AtomicBool, AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize, AtomicU8, AtomicU16,
    AtomicU32, AtomicU64, AtomicUsize, Ordering,
};
use std::time::{Duration, Instant, SystemTime};

use crate::GetSize;

impl GetSize for () {}
impl GetSize for bool {}
impl GetSize for u8 {}
impl GetSize for u16 {}
impl GetSize for u32 {}
impl GetSize for u64 {}
impl GetSize for u128 {}
impl GetSize for usize {}
impl GetSize for NonZeroU8 {}
impl GetSize for NonZeroU16 {}
impl GetSize for NonZeroU32 {}
impl GetSize for NonZeroU64 {}
impl GetSize for NonZeroU128 {}
impl GetSize for NonZeroUsize {}
impl GetSize for i8 {}
impl GetSize for i16 {}
impl GetSize for i32 {}
impl GetSize for i64 {}
impl GetSize for i128 {}
impl GetSize for isize {}
impl GetSize for NonZeroI8 {}
impl GetSize for NonZeroI16 {}
impl GetSize for NonZeroI32 {}
impl GetSize for NonZeroI64 {}
impl GetSize for NonZeroI128 {}
impl GetSize for NonZeroIsize {}
impl GetSize for f32 {}
impl GetSize for f64 {}
impl GetSize for char {}

impl GetSize for AtomicBool {}
impl GetSize for AtomicI8 {}
impl GetSize for AtomicI16 {}
impl GetSize for AtomicI32 {}
impl GetSize for AtomicI64 {}
impl GetSize for AtomicIsize {}
impl GetSize for AtomicU8 {}
impl GetSize for AtomicU16 {}
impl GetSize for AtomicU32 {}
impl GetSize for AtomicU64 {}
impl GetSize for AtomicUsize {}
impl GetSize for Ordering {}
impl GetSize for std::cmp::Ordering {}

impl GetSize for Infallible {}
impl<T> GetSize for PhantomData<T> {}
impl GetSize for PhantomPinned {}

impl GetSize for Instant {}
impl GetSize for Duration {}
impl GetSize for SystemTime {}
