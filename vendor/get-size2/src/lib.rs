#![doc = include_str!("./lib.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use get_size_derive2::*;

mod impls;
mod tracker;
pub use tracker::*;
#[cfg(test)]
mod tests;

/// Determines how many bytes the object occupies inside the heap.
pub fn heap_size<T: GetSize>(value: &T) -> usize {
    value.get_heap_size()
}

/// Determine the size in bytes an object occupies inside RAM.
pub trait GetSize: Sized {
    /// Determines how may bytes this object occupies inside the stack.
    ///
    /// The default implementation uses [`std::mem::size_of`] and should work for almost all types.
    #[must_use]
    fn get_stack_size() -> usize {
        std::mem::size_of::<Self>()
    }

    /// Determines how many bytes this object occupies inside the heap.
    ///
    /// The default implementation simply delegates to [`get_heap_size_with_tracker`](Self::get_heap_size_with_tracker)
    /// with a noop tracker. This method is not meant to be implemented directly, and only exists for convenience.
    fn get_heap_size(&self) -> usize {
        let tracker = NoTracker::new(true);
        Self::get_heap_size_with_tracker(self, tracker).0
    }

    /// Determines how many bytes this object occupies inside the heap while using a `tracker`.
    ///
    /// The default implementation returns 0, assuming the object is fully allocated on the stack.
    /// It must be adjusted as appropriate for objects which hold data inside the heap.
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (0, tracker)
    }

    /// Determines the total size of the object.
    ///
    /// The default implementation simply adds up the results of [`get_stack_size`](Self::get_stack_size)
    /// and [`get_heap_size`](Self::get_heap_size) and is not meant to be changed.
    fn get_size(&self) -> usize {
        Self::get_stack_size() + GetSize::get_heap_size(self)
    }

    /// Determines the total size of the object while using a `tracker`.
    ///
    /// The default implementation simply adds up the results of [`get_stack_size`](Self::get_stack_size)
    /// and [`get_heap_size_with_tracker`](Self::get_heap_size_with_tracker) and is not meant to
    /// be changed.
    fn get_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        let stack_size = Self::get_stack_size();
        let (heap_size, tracker) = Self::get_heap_size_with_tracker(self, tracker);
        (stack_size + heap_size, tracker)
    }
}
