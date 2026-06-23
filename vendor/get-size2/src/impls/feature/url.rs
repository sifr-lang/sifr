use crate::{GetSize, GetSizeTracker};

impl GetSize for url::Url {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.as_str().len(), tracker)
    }
}
