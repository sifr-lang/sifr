use crate::{GetSize, GetSizeTracker};

impl<K, V, H> GetSize for hashbrown::HashMap<K, V, H>
where
    K: GetSize + Eq + std::hash::Hash,
    V: GetSize,
    H: std::hash::BuildHasher,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self
            .iter()
            .fold((0, tracker), |(size, tracker), (key, value)| {
                let (key_size, tracker) = K::get_heap_size_with_tracker(key, tracker);
                let (value_size, tracker) = V::get_heap_size_with_tracker(value, tracker);
                (size + key_size + value_size, tracker)
            });

        (size + self.allocation_size(), tracker)
    }
}

impl<T, H> GetSize for hashbrown::HashSet<T, H>
where
    T: GetSize + Eq + std::hash::Hash,
    H: std::hash::BuildHasher,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        (size + self.allocation_size(), tracker)
    }
}

impl<T> GetSize for hashbrown::HashTable<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        (size + self.allocation_size(), tracker)
    }
}
