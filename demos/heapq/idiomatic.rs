use std::cmp::Reverse;
use std::collections::BinaryHeap;

fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

#[derive(Debug, Clone)]
struct MinHeap<T: Ord> {
    values: BinaryHeap<Reverse<T>>,
}

impl<T: Ord> MinHeap<T> {
    fn new() -> Self {
        Self {
            values: BinaryHeap::new(),
        }
    }

    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            values: iter.into_iter().map(Reverse).collect(),
        }
    }
}

fn heapify<T: Ord, I>(values: I) -> MinHeap<T>
where
    I: IntoIterator<Item = T>,
{
    MinHeap::from_iter(values)
}

fn heappush<T: Ord>(heap: &mut MinHeap<T>, item: T) {
    heap.values.push(Reverse(item));
}

fn heappop<T: Ord>(heap: &mut MinHeap<T>) -> Option<T> {
    heap.values.pop().map(|Reverse(value)| value)
}

fn nsmallest<T: Ord + Clone>(n: usize, values: &[T]) -> Vec<T> {
    let mut result = values.to_vec();
    result.sort();
    result.truncate(n);
    result
}

fn nlargest<T: Ord + Clone>(n: usize, values: &[T]) -> Vec<T> {
    let mut result = values.to_vec();
    result.sort_by(|left, right| right.cmp(left));
    result.truncate(n);
    result
}

fn collect_actual() -> Vec<bool> {
    let mut heap = MinHeap::new();
    heappush(&mut heap, 5);
    heappush(&mut heap, 1);
    heappush(&mut heap, 3);

    let first = heappop(&mut heap);
    let second = heappop(&mut heap);

    let mut from_values = heapify([4, 2, 7, 1, 5]);
    let top = heappop(&mut from_values);

    let items = vec![9, 3, 7, 1, 5];
    let mut empty_heap = MinHeap::<i64>::new();

    vec![
        first == Some(1),
        second == Some(3),
        top == Some(1),
        nsmallest(3, &items) == vec![1, 3, 5],
        nlargest(2, &items) == vec![9, 7],
        heappop(&mut empty_heap).is_none(),
        items == vec![9, 3, 7, 1, 5],
    ]
}

fn main() {
    let actual = collect_actual();
    assert_bool_vector_eq(&actual, &[true, true, true, true, true, true, true]);
    println!("heapq heapq parity demo: pass");
}
