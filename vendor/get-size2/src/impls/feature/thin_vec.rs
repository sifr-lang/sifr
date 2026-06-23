use crate::{GetSize, GetSizeTracker};

impl<T> GetSize for thin_vec::ThinVec<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        if self.capacity() == 0 {
            // If it's the singleton we might not be a heap pointer.
            return (0, tracker);
        }

        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        let metadata_size = std::mem::size_of::<usize>() * 2; // Capacity and length.
        let allocation_size = self.capacity() * T::get_stack_size();
        (size + metadata_size + allocation_size, tracker)
    }
}
