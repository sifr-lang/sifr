use crate::{GetSize, GetSizeTracker};

impl GetSize for compact_str::CompactString {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        let size = if self.is_heap_allocated() {
            self.capacity()
        } else {
            0
        };

        (size, tracker)
    }
}
