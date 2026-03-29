use std::cmp::Reverse;
use std::collections::BinaryHeap;

fn heappush(heap: &mut BinaryHeap<Reverse<i64>>, value: i64) {
    heap.push(Reverse(value));
}

fn heappop(heap: &mut BinaryHeap<Reverse<i64>>) -> Option<i64> {
    heap.pop().map(|value| value.0)
}

fn drain_sorted(values: &[i64]) -> Vec<i64> {
    let mut heap = BinaryHeap::new();
    let mut order = Vec::new();

    for value in values {
        heappush(&mut heap, *value);
    }

    while let Some(item) = heappop(&mut heap) {
        order.push(item);
    }

    order
}

fn main() {
    assert_eq!(format!("{:?}", drain_sorted(&[5, 1, 3])), "[1, 3, 5]");

    let mut heap = BinaryHeap::new();
    heappush(&mut heap, 7);
    assert_eq!(heappop(&mut heap), Some(7));
    assert_eq!(heappop(&mut heap), None);
    println!("heap_option_drain: ok");
}
