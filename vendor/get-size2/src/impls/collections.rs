use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::hash::BuildHasher;

use crate::{GetSize, GetSizeTracker};

macro_rules! impl_size_set {
    ($name:ident) => {
        impl<T> GetSize for $name<T>
        where
            T: GetSize,
        {
            fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
                let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), elem| {
                    let (elem_size, tracker) = T::get_heap_size_with_tracker(elem, tracker);
                    (size + elem_size, tracker)
                });

                let allocation_size = self.capacity() * T::get_stack_size();
                (size + allocation_size, tracker)
            }
        }
    };
}

macro_rules! impl_size_set_no_capacity {
    ($name:ident) => {
        impl<T> GetSize for $name<T>
        where
            T: GetSize,
        {
            fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
                let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), elem| {
                    // We assume that values are held inside the heap.
                    let (elem_size, tracker) = T::get_size_with_tracker(elem, tracker);
                    (size + elem_size, tracker)
                });

                (size, tracker)
            }
        }
    };
}

impl_size_set_no_capacity!(BTreeSet);
impl_size_set!(BinaryHeap);
impl_size_set_no_capacity!(LinkedList);
impl_size_set!(VecDeque);

impl<K, V> GetSize for BTreeMap<K, V>
where
    K: GetSize,
    V: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        self.iter()
            .fold((0, tracker), |(size, tracker), (key, value)| {
                let (key_size, tracker) = K::get_size_with_tracker(key, tracker);
                let (value_size, tracker) = V::get_size_with_tracker(value, tracker);
                (size + key_size + value_size, tracker)
            })
    }
}

impl<K, V, S: BuildHasher> GetSize for HashMap<K, V, S>
where
    K: GetSize,
    V: GetSize,
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

impl<T, S: BuildHasher> GetSize for HashSet<T, S>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), elem| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(elem, tracker);
            (size + elem_size, tracker)
        });

        let allocation_size = self.capacity() * T::get_stack_size();
        (size + allocation_size, tracker)
    }
}

impl_size_set!(Vec);

macro_rules! impl_size_tuple {
    ($($t:ident, $T:ident),+) => {
        impl<$($T,)*> GetSize for ($($T,)*)
        where
            $(
                $T: GetSize,
            )*
        {
            #[allow(unused_mut, reason = "the macro supports a variadic number of elements")]
            #[expect(clippy::allow_attributes, reason = "the macro supports a variadic number of elements")]
            fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, mut tracker: Tr) -> (usize, Tr) {
                let mut total = 0;
                let mut elem_size;

                let ($($t,)*) = self;
                $(
                    (elem_size, tracker) = <$T>::get_heap_size_with_tracker($t, tracker);
                    total += elem_size;
                )*

                (total, tracker)
            }
        }
    }
}

macro_rules! execute_tuple_macro_16 {
    ($name:ident) => {
        $name!(v1, V1);
        $name!(v1, V1, v2, V2);
        $name!(v1, V1, v2, V2, v3, V3);
        $name!(v1, V1, v2, V2, v3, V3, v4, V4);
        $name!(v1, V1, v2, V2, v3, V3, v4, V4, v5, V5);
        $name!(v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6);
        $name!(v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7);
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10, v11,
            V11
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10, v11,
            V11, v12, V12
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10, v11,
            V11, v12, V12, v13, V13
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10, v11,
            V11, v12, V12, v13, V13, v14, V14
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10, v11,
            V11, v12, V12, v13, V13, v14, V14, v15, V15
        );
        $name!(
            v1, V1, v2, V2, v3, V3, v4, V4, v5, V5, v6, V6, v7, V7, v8, V8, v9, V9, v10, V10, v11,
            V11, v12, V12, v13, V13, v14, V14, v15, V15, v16, V16
        );
    };
}

execute_tuple_macro_16!(impl_size_tuple);

impl<T, const SIZE: usize> GetSize for [T; SIZE]
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        self.iter().fold((0, tracker), |(size, tracker), element| {
            // The array stack size already accounts for the stack size of the elements of the array.
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        })
    }
}
