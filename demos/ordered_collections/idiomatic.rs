use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, VecDeque};

fn most_common(items: &[&str]) -> Vec<(String, i64)> {
    let mut counts = HashMap::new();
    let mut first_seen = HashMap::new();

    for (index, item) in items.iter().enumerate() {
        *counts.entry(*item).or_insert(0_i64) += 1;
        first_seen.entry(*item).or_insert(index);
    }

    let mut pairs: Vec<(String, i64, usize)> = counts
        .into_iter()
        .map(|(item, count)| (item.to_string(), count, first_seen[item]))
        .collect();
    pairs.sort_by_key(|(_, count, index)| (Reverse(*count), *index));
    pairs
        .into_iter()
        .map(|(item, count, _)| (item, count))
        .collect()
}

fn rotate_right_once(values: &mut VecDeque<i64>) {
    if let Some(last) = values.pop_back() {
        values.push_front(last);
    }
}

fn appendleft_bounded(values: &mut VecDeque<i64>, value: i64, maxlen: usize) {
    if values.len() == maxlen {
        values.pop_back();
    }
    values.push_front(value);
}

fn insort(values: &mut Vec<i64>, value: i64) {
    let index = values.partition_point(|existing| *existing <= value);
    values.insert(index, value);
}

fn bisect(values: &[i64], value: i64) -> usize {
    values.partition_point(|existing| *existing <= value)
}

fn heapify(values: Vec<i64>) -> BinaryHeap<Reverse<i64>> {
    values.into_iter().map(Reverse).collect()
}

fn heappushpop(heap: &mut BinaryHeap<Reverse<i64>>, value: i64) -> i64 {
    match heap.peek().map(|entry| entry.0) {
        Some(smallest) if value > smallest => {
            if let Some(popped) = heap.pop() {
                heap.push(Reverse(value));
                popped.0
            } else {
                value
            }
        }
        _ => value,
    }
}

fn heapreplace(heap: &mut BinaryHeap<Reverse<i64>>, value: i64) -> Option<i64> {
    let popped = heap.pop()?;
    heap.push(Reverse(value));
    Some(popped.0)
}

fn main() {
    println!("{:?}", most_common(&["delta", "alpha", "delta", "beta"]));

    let mut queue = VecDeque::from([1_i64, 2, 3]);
    rotate_right_once(&mut queue);
    appendleft_bounded(&mut queue, 0, 4);
    println!("{:?}", queue.into_iter().collect::<Vec<_>>());

    let mut ordered = vec![1_i64, 3, 5];
    insort(&mut ordered, 4);
    println!("{}", bisect(&ordered, 4));

    let mut heap = heapify(vec![1_i64, 3, 5]);
    println!("{}", heappushpop(&mut heap, 2));
    if let Some(replaced) = heapreplace(&mut heap, 4) {
        println!("{replaced}");
    }
}
