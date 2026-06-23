use std::cell::RefCell;
use std::sync::{Mutex, OnceLock, RwLock};

use crate::{GetSize, GetSizeTracker};

impl<T> GetSize for Mutex<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        // We assume that a `Mutex` holds its data on the stack.
        let guard = self
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        T::get_heap_size_with_tracker(&*guard, tracker)
    }
}

impl<T> GetSize for RwLock<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        // We assume that a `RwLock` holds its data on the stack.
        let guard = self
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        T::get_heap_size_with_tracker(&*guard, tracker)
    }
}

impl<T> GetSize for RefCell<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        // We assume that a `RefCell` holds its data on the stack.
        // Use try_borrow to avoid panicking if the RefCell is already mutably borrowed
        match self.try_borrow() {
            Ok(borrowed) => T::get_heap_size_with_tracker(&*borrowed, tracker),
            Err(_) => {
                // If the RefCell is already mutably borrowed, we cannot safely access it.
                // Return 0 for heap size to avoid panic, though this is a rare edge case.
                (0, tracker)
            }
        }
    }
}

impl<T> GetSize for OnceLock<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        // We assume that a `OnceLock` holds its data on the stack.
        match self.get() {
            None => (0, tracker),
            Some(value) => T::get_heap_size_with_tracker(value, tracker),
        }
    }
}
