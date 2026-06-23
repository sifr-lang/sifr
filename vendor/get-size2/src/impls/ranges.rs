use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

use crate::{GetSize, GetSizeTracker};

/// Helper macro used to sum the heap sizes of specific fields.
macro_rules! impl_sum_of_fields {
    ($name:ident, $($field:ident),+) => {
        impl<I: GetSize> GetSize for $name<I> {
            #[allow(unused_mut, reason = "the macro supports a variadic number of elements")]
            #[expect(clippy::allow_attributes, reason = "the macro supports a variadic number of elements")]
            fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, mut tracker: Tr) -> (usize, Tr) {
                let mut size = 0;
                let mut elem_size;

                $(
                    (elem_size, tracker) = self.$field.get_heap_size_with_tracker(tracker);
                    size += elem_size;
                )+

                (size, tracker)
            }
        }
    };
}

impl_sum_of_fields!(Range, start, end);
impl_sum_of_fields!(RangeFrom, start);
impl_sum_of_fields!(RangeTo, end);
impl_sum_of_fields!(RangeToInclusive, end);

impl GetSize for RangeFull {}

impl<I: GetSize> GetSize for RangeInclusive<I> {
    #[inline]
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (start_size, tracker) = (*self.start()).get_heap_size_with_tracker(tracker);
        let (end_size, tracker) = (*self.end()).get_heap_size_with_tracker(tracker);
        (start_size + end_size, tracker)
    }
}
