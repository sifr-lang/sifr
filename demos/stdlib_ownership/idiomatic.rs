use std::cmp::Reverse;
use std::collections::BTreeMap;

fn sift_down(heap: &mut [i64], mut root: usize) {
    let len = heap.len();
    loop {
        let left = (root * 2) + 1;
        let right = left + 1;
        let mut smallest = root;

        if left < len && heap[left] < heap[smallest] {
            smallest = left;
        }
        if right < len && heap[right] < heap[smallest] {
            smallest = right;
        }
        if smallest == root {
            break;
        }

        heap.swap(root, smallest);
        root = smallest;
    }
}

fn sift_up(heap: &mut [i64], mut index: usize) {
    while index > 0 {
        let parent = (index - 1) / 2;
        if heap[parent] <= heap[index] {
            break;
        }
        heap.swap(parent, index);
        index = parent;
    }
}

fn heapify(data: &mut [i64]) {
    for index in (0..data.len() / 2).rev() {
        sift_down(data, index);
    }
}

fn heappush(heap: &mut Vec<i64>, item: i64) {
    heap.push(item);
    let last = heap.len() - 1;
    sift_up(heap, last);
}

fn heappop(heap: &mut Vec<i64>) -> Option<i64> {
    match heap.pop() {
        None => None,
        Some(last) if heap.is_empty() => Some(last),
        Some(last) => {
            let smallest = std::mem::replace(&mut heap[0], last);
            sift_down(heap, 0);
            Some(smallest)
        }
    }
}

fn nsmallest(count: usize, values: &[i64]) -> Vec<i64> {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted.into_iter().take(count).collect()
}

fn nlargest(count: usize, values: &[i64]) -> Vec<i64> {
    let mut sorted = values.to_vec();
    sorted.sort_unstable_by_key(|value| Reverse(*value));
    sorted.into_iter().take(count).collect()
}

fn bisect_left(values: &[i64], needle: i64) -> usize {
    values.partition_point(|value| *value < needle)
}

fn bisect_right(values: &[i64], needle: i64) -> usize {
    values.partition_point(|value| *value <= needle)
}

fn insort_left(values: &mut Vec<i64>, needle: i64) {
    let index = bisect_left(values, needle);
    values.insert(index, needle);
}

fn insort_right(values: &mut Vec<i64>, needle: i64) {
    let index = bisect_right(values, needle);
    values.insert(index, needle);
}

fn chain<'a>(left: &'a [i64], right: &'a [i64]) -> impl Iterator<Item = i64> + 'a {
    left.iter().chain(right).copied()
}

#[derive(Default)]
struct Counter {
    counts: BTreeMap<String, i64>,
}

impl Counter {
    fn from_list<T: AsRef<str>>(values: &[T]) -> Self {
        let mut counter = Self::default();
        for value in values {
            counter.increment(value.as_ref());
        }
        counter
    }

    fn get(&self, key: &str) -> i64 {
        self.counts.get(key).copied().unwrap_or(0)
    }

    fn total(&self) -> i64 {
        self.counts.values().sum()
    }

    fn increment(&mut self, key: &str) {
        *self.counts.entry(key.to_string()).or_insert(0) += 1;
    }

    fn most_common(&self, count: usize) -> Vec<(String, i64)> {
        let mut entries = self
            .counts
            .iter()
            .map(|(key, value)| (key.clone(), *value))
            .collect::<Vec<_>>();
        entries.sort_by_key(|(key, value)| (Reverse(*value), key.clone()));
        entries.into_iter().take(count).collect()
    }

    fn keys(&self) -> Vec<String> {
        self.counts.keys().cloned().collect()
    }
}

fn demo_heapq() {
    println!("=== Section 1: heapq with mut params ===");

    let mut data = vec![5, 3, 8, 1, 2, 7, 4];
    heapify(&mut data);
    println!("heapified (min at root):");
    println!("{}", data[0]);

    heappush(&mut data, 0);
    println!("after push(0), new min:");
    println!("{}", data[0]);

    if let Some(popped) = heappop(&mut data) {
        println!("popped:");
        println!("{popped}");
    }

    println!("remaining size:");
    println!("{}", data.len());

    let items = vec![9, 3, 7, 1, 5, 6, 2, 8, 4];
    println!("3 smallest:");
    println!("{:?}", nsmallest(3, &items));
    println!("3 largest:");
    println!("{:?}", nlargest(3, &items));
    println!("items still valid, length:");
    println!("{}", items.len());
}

fn demo_bisect() {
    println!("=== Section 2: bisect insort with mut params ===");

    let mut sorted_ints = vec![1, 3, 5, 7, 9];
    println!("insert 6 at position (left):");
    println!("{}", bisect_left(&sorted_ints, 6));
    println!("insert after 5 at position (right):");
    println!("{}", bisect_right(&sorted_ints, 5));

    insort_left(&mut sorted_ints, 6);
    println!("after insort_left(6):");
    println!("{sorted_ints:?}");

    let mut duplicates = vec![1, 2, 2, 3];
    insort_right(&mut duplicates, 2);
    println!("after insort_right(2) with duplicates:");
    println!("{duplicates:?}");

    insort_left(&mut sorted_ints, 0);
    insort_right(&mut sorted_ints, 10);
    println!("after more inserts:");
    println!("{sorted_ints:?}");
}

fn demo_itertools() {
    println!("=== Section 3: itertools chain ===");

    let left = vec![1, 2, 3];
    let right = vec![4, 5, 6];
    let result = chain(&left, &right).collect::<Vec<_>>();
    println!("chain (borrow both):");
    println!("{result:?}");
    println!("a still usable:");
    println!("{}", left.len());
    println!("b still usable:");
    println!("{}", right.len());

    let x = vec![10, 20, 30];
    let y = vec![40, 50, 60];
    let combined = chain(&x, &y).collect::<Vec<_>>();
    println!("chain result:");
    println!("{combined:?}");
}

fn demo_counter() {
    println!("=== Section 4: Counter with native dict[str, int] ===");

    let words = ["apple", "banana", "apple", "cherry", "banana", "apple"];
    let mut counter = Counter::from_list(&words);

    println!("apple count:");
    println!("{}", counter.get("apple"));
    println!("banana count:");
    println!("{}", counter.get("banana"));
    println!("missing key returns 0:");
    println!("{}", counter.get("missing"));
    println!("total elements:");
    println!("{}", counter.total());

    counter.increment("cherry");
    counter.increment("cherry");
    println!("cherry after 2 increments:");
    println!("{}", counter.get("cherry"));

    println!("top 1 most common:");
    println!("{:?}", counter.most_common(1));
    println!("unique keys count:");
    println!("{}", counter.keys().len());
}

fn main() {
    demo_heapq();
    demo_bisect();
    demo_itertools();
    demo_counter();
    println!("=== borrow_stdlib demo complete ===");
}
