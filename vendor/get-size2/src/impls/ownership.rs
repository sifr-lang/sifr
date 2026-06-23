use std::borrow::Cow;
use std::rc::{Rc, Weak as RcWeak};
use std::sync::{Arc, Weak as ArcWeak};

use crate::{GetSize, GetSizeTracker};

impl<T> GetSize for Cow<'_, T>
where
    T: ToOwned + ?Sized,
    T::Owned: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        match self {
            Self::Borrowed(_borrowed) => (0, tracker),
            Self::Owned(owned) => <T::Owned>::get_heap_size_with_tracker(owned, tracker),
        }
    }
}

impl<T> GetSize for &[T] where T: GetSize {}

impl<T> GetSize for &T {}
impl<T> GetSize for &mut T {}
impl<T> GetSize for *const T {}
impl<T> GetSize for *mut T {}

impl<T> GetSize for Box<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        T::get_size_with_tracker(&**self, tracker)
    }
}

impl<T> GetSize for Rc<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, mut tracker: Tr) -> (usize, Tr) {
        if tracker.track(Self::as_ptr(self)) {
            T::get_size_with_tracker(&**self, tracker)
        } else {
            (0, tracker)
        }
    }
}

impl<T> GetSize for RcWeak<T> {}

impl<T> GetSize for Arc<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, mut tracker: Tr) -> (usize, Tr) {
        if tracker.track(Self::as_ptr(self)) {
            T::get_size_with_tracker(&**self, tracker)
        } else {
            (0, tracker)
        }
    }
}

impl<T> GetSize for ArcWeak<T> {}

impl<T> GetSize for Option<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        match self {
            None => (0, tracker),
            Some(value) => T::get_heap_size_with_tracker(value, tracker),
        }
    }
}

impl<T, E> GetSize for Result<T, E>
where
    T: GetSize,
    E: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        // The result's stack size already accounts for the values stack size.
        match self {
            Ok(value) => T::get_heap_size_with_tracker(value, tracker),
            Err(err) => E::get_heap_size_with_tracker(err, tracker),
        }
    }
}
