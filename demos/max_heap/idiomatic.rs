use std::collections::BinaryHeap;

#[derive(Debug, Clone)]
struct MaxHeap<T: Ord> {
    values: BinaryHeap<T>,
}

impl<T: Ord> MaxHeap<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            values: iter.into_iter().collect(),
        }
    }
}

fn heapify_max<T: Ord, I>(values: I) -> MaxHeap<T>
where
    I: IntoIterator<Item = T>,
{
    MaxHeap::from_iter(values)
}

fn heappop_max<T: Ord>(heap: &mut MaxHeap<T>) -> Option<T> {
    heap.values.pop()
}

fn heapreplace_max<T: Ord>(heap: &mut MaxHeap<T>, item: T) -> Option<T> {
    let removed = heap.values.pop();
    heap.values.push(item);
    removed
}

fn drain(heap: &mut MaxHeap<i64>) -> Vec<i64> {
    let mut result = Vec::new();
    while let Some(value) = heappop_max(heap) {
        result.push(value);
    }
    result
}

fn main() {
    let mut stones = heapify_max([2, 7, 4, 1, 8, 1]);
    println!("{:?}", drain(&mut stones));

    let mut probe = heapify_max([4, 10, 7]);
    let _ = heapreplace_max(&mut probe, 6);
    println!("{:?}", drain(&mut probe));
}
