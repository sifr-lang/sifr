use crate::{GetSize, GetSizeTracker};

impl<K, V, S> GetSize for ordermap::OrderMap<K, V, S>
where
    K: GetSize,
    V: GetSize,
    S: std::hash::BuildHasher,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self
            .iter()
            .fold((0, tracker), |(size, tracker), (key, value)| {
                let (key_size, tracker) = K::get_heap_size_with_tracker(key, tracker);
                let (value_size, tracker) = V::get_heap_size_with_tracker(value, tracker);
                (size + key_size + value_size, tracker)
            });

        let allocation_size = self.capacity() * <(K, V)>::get_stack_size();
        (size + allocation_size, tracker)
    }
}

impl<T, S> GetSize for ordermap::OrderSet<T, S>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        let allocation_size = self.capacity() * T::get_stack_size();
        (size + allocation_size, tracker)
    }
}
