use crate::{GetSize, GetSizeTracker};

impl GetSize for bytes::Bytes {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.len(), tracker)
    }
}

impl GetSize for bytes::BytesMut {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.len(), tracker)
    }
}
