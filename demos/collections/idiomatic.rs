use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

#[derive(Debug, Clone)]
struct Counter<T> {
    counts: HashMap<T, i64>,
    encounter_order: Vec<T>,
}

impl<T> Counter<T>
where
    T: Clone + Eq + Hash,
{
    fn new() -> Self {
        Self {
            counts: HashMap::new(),
            encounter_order: Vec::new(),
        }
    }

    fn from_slice(values: &[T]) -> Self {
        let mut counter = Self::new();
        for value in values {
            counter.increment(value.clone());
        }
        counter
    }

    fn increment(&mut self, value: T) {
        let is_new = !self.counts.contains_key(&value);
        let count = self.counts.entry(value.clone()).or_insert(0);
        *count += 1;
        if is_new {
            self.encounter_order.push(value);
        }
    }

    fn get(&self, value: &T) -> i64 {
        self.counts.get(value).copied().unwrap_or(0)
    }

    fn most_common(&self, limit: usize) -> Vec<(T, i64)> {
        let mut entries: Vec<_> = self
            .encounter_order
            .iter()
            .enumerate()
            .map(|(index, value)| (index, value.clone(), self.get(value)))
            .collect();

        entries.sort_by(|left, right| right.2.cmp(&left.2).then_with(|| left.0.cmp(&right.0)));

        entries
            .into_iter()
            .take(limit)
            .map(|(_, value, count)| (value, count))
            .collect()
    }
}

fn from_list<T>(values: &[T]) -> Counter<T>
where
    T: Clone + Eq + Hash,
{
    Counter::from_slice(values)
}

fn set_from_list<T>(values: &[T]) -> Vec<T>
where
    T: Clone + Eq + Hash,
{
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for value in values {
        if seen.insert(value.clone()) {
            result.push(value.clone());
        }
    }

    result
}

fn set_union<T>(left: &[T], right: &[T]) -> Vec<T>
where
    T: Clone + Eq + Hash,
{
    let mut result = set_from_list(left);
    let mut seen: HashSet<_> = result.iter().cloned().collect();

    for value in right {
        if seen.insert(value.clone()) {
            result.push(value.clone());
        }
    }

    result
}

fn set_intersection<T>(left: &[T], right: &[T]) -> Vec<T>
where
    T: Clone + Eq + Hash,
{
    let right_values: HashSet<_> = right.iter().cloned().collect();
    let mut emitted = HashSet::new();
    let mut result = Vec::new();

    for value in left {
        if right_values.contains(value) && emitted.insert(value.clone()) {
            result.push(value.clone());
        }
    }

    result
}

fn set_len<T>(values: &[T]) -> usize {
    values.len()
}

#[derive(Debug, Clone)]
struct Deque<T> {
    values: VecDeque<T>,
    maxlen: Option<usize>,
}

impl<T> Deque<T> {
    fn new(maxlen: Option<usize>) -> Self {
        Self {
            values: VecDeque::new(),
            maxlen,
        }
    }

    fn append(&mut self, value: T) {
        if let Some(limit) = self.maxlen {
            if limit == 0 {
                return;
            }
            if self.values.len() >= limit {
                self.values.pop_front();
            }
        }

        self.values.push_back(value);
    }

    fn popleft(&mut self) -> Option<T> {
        self.values.pop_front()
    }

    fn pop(&mut self) -> Option<T> {
        self.values.pop_back()
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

fn collect_set_and_counter_actual() -> Vec<bool> {
    let left = set_from_list(&[1, 2, 3]);
    let right = set_from_list(&[3, 4, 5]);
    let counts = from_list(
        &["x", "y", "x", "z", "x", "y"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>(),
    );

    vec![
        set_len(&set_union(&left, &right)) == 5,
        set_len(&set_intersection(&left, &right)) == 1,
        counts.get(&"x".to_string()) == 3,
        counts.most_common(2) == vec![("x".to_string(), 3), ("y".to_string(), 2)],
    ]
}

fn collect_deque_actual() -> Vec<bool> {
    let mut deque = Deque::new(Some(2));
    deque.append(10);
    deque.append(20);
    deque.append(30);

    let first_check = deque.len() == 2 && deque.popleft() == Some(20);
    let _ = deque.pop();

    vec![first_check, deque.pop().is_none()]
}

fn append_all(target: &mut Vec<bool>, values: &[bool]) {
    target.extend_from_slice(values);
}

fn main() {
    let mut actual = Vec::new();
    append_all(&mut actual, &collect_set_and_counter_actual());
    append_all(&mut actual, &collect_deque_actual());

    assert_bool_vector_eq(&actual, &[true, true, true, true, true, true]);
    println!("collections collections parity demo: pass");
}
