use std::collections::{BTreeMap, VecDeque};

fn take<T: Clone>(count: usize, values: &[T]) -> Vec<T> {
    values.iter().take(count).cloned().collect()
}

fn flatten<T: Clone>(nested: &[Vec<T>]) -> Vec<T> {
    nested
        .iter()
        .flat_map(|items| items.iter().cloned())
        .collect()
}

fn accumulate<T, F>(values: &[T], combine: F) -> Vec<T>
where
    T: Clone,
    F: Fn(&T, &T) -> T,
{
    let mut result = Vec::new();
    for value in values {
        let next = match result.last() {
            Some(previous) => combine(previous, value),
            None => value.clone(),
        };
        result.push(next);
    }
    result
}

fn dropwhile<T: Clone, F>(values: &[T], predicate: F) -> Vec<T>
where
    F: Fn(&T) -> bool,
{
    let mut dropping = true;
    values
        .iter()
        .filter_map(|value| {
            if dropping && predicate(value) {
                None
            } else {
                dropping = false;
                Some(value.clone())
            }
        })
        .collect()
}

fn takewhile<T: Clone, F>(values: &[T], predicate: F) -> Vec<T>
where
    F: Fn(&T) -> bool,
{
    values.iter().cloned().take_while(predicate).collect()
}

fn filterfalse<T: Clone, F>(values: &[T], predicate: F) -> Vec<T>
where
    F: Fn(&T) -> bool,
{
    values
        .iter()
        .filter(|value| !predicate(value))
        .cloned()
        .collect()
}

fn compress<T: Clone>(values: &[T], selectors: &[bool]) -> Vec<T> {
    values
        .iter()
        .zip(selectors.iter())
        .filter_map(|(value, keep)| keep.then_some(value.clone()))
        .collect()
}

fn zip_longest<T: Clone>(left: &[T], right: &[T], fill: T) -> Vec<Vec<T>> {
    let max_len = left.len().max(right.len());
    (0..max_len)
        .map(|index| {
            vec![
                left.get(index).cloned().unwrap_or_else(|| fill.clone()),
                right.get(index).cloned().unwrap_or_else(|| fill.clone()),
            ]
        })
        .collect()
}

fn reduce<T, F>(values: &[T], initial: T, reducer: F) -> T
where
    T: Clone,
    F: Fn(T, T) -> T,
{
    values.iter().cloned().fold(initial, reducer)
}

#[derive(Clone, Debug)]
struct Counter<T>
where
    T: Ord + Clone,
{
    counts: BTreeMap<T, i64>,
}

impl<T> Counter<T>
where
    T: Ord + Clone,
{
    fn from_list(values: &[T]) -> Self {
        let mut counts = BTreeMap::new();
        for value in values {
            *counts.entry(value.clone()).or_insert(0) += 1;
        }
        Self { counts }
    }

    fn get(&self, value: &T) -> i64 {
        *self.counts.get(value).unwrap_or(&0)
    }

    fn total(&self) -> i64 {
        self.counts.values().sum()
    }

    fn most_common(&self, n: usize) -> Vec<(T, i64)> {
        let mut entries = self
            .counts
            .iter()
            .map(|(key, value)| (key.clone(), *value))
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
        entries.into_iter().take(n).collect()
    }
}

impl<T> std::ops::Add for Counter<T>
where
    T: Ord + Clone,
{
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        for (key, value) in rhs.counts {
            *self.counts.entry(key).or_insert(0) += value;
        }
        self
    }
}

#[derive(Clone, Debug)]
struct Deque<T> {
    values: VecDeque<T>,
}

impl<T: Clone> Deque<T> {
    fn new() -> Self {
        Self {
            values: VecDeque::new(),
        }
    }

    fn append(&mut self, value: T) {
        self.values.push_back(value);
    }

    fn appendleft(&mut self, value: T) {
        self.values.push_front(value);
    }

    fn to_list(&self) -> Vec<T> {
        self.values.iter().cloned().collect()
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

fn main() {
    println!("=== Generic chain ===");
    let ints = [1_i64, 2]
        .iter()
        .copied()
        .chain([3_i64, 4].iter().copied())
        .collect::<Vec<_>>();
    println!("{ints:?}");
    let strs = ["a", "b"]
        .iter()
        .copied()
        .chain(["c", "d"].iter().copied())
        .map(str::to_string)
        .collect::<Vec<_>>();
    println!("{strs:?}");

    println!("=== Generic take ===");
    println!("{:?}", take(3, &[10_i64, 20, 30, 40, 50]));
    println!(
        "{:?}",
        take(
            2,
            &["hello".to_string(), "world".to_string(), "foo".to_string()]
        )
    );

    println!("=== Generic flatten ===");
    println!("{:?}", flatten(&[vec![1_i64, 2], vec![3, 4], vec![5]]));

    println!("=== Generic accumulate ===");
    println!("{:?}", accumulate(&[1_i64, 2, 3, 4, 5], |a, b| a + b));
    println!("{:?}", accumulate(&[1.0_f64, 2.5, 3.5], |a, b| a + b));

    println!("=== Predicate-based dropwhile ===");
    let data = [1_i64, 3, 7, 2, 8];
    println!("{:?}", dropwhile(&data, |value| *value < 5));

    println!("=== Predicate-based takewhile ===");
    println!("{:?}", takewhile(&data, |value| *value < 5));

    println!("=== Predicate-based filterfalse ===");
    println!(
        "{:?}",
        filterfalse(&[1_i64, 2, 3, 4, 5, 6], |value| value % 2 == 0)
    );

    println!("=== Generic heapq ===");
    let mut items = vec![9_i64, 3, 7, 1, 5];
    items.sort_unstable();
    println!("{:?}", take(3, &items));
    items.reverse();
    println!("{:?}", take(2, &items));

    println!("=== Generic Counter[T] ===");
    let words = vec![
        "apple".to_string(),
        "banana".to_string(),
        "apple".to_string(),
        "cherry".to_string(),
        "banana".to_string(),
        "apple".to_string(),
    ];
    let c = Counter::from_list(&words);
    println!("{}", c.get(&"apple".to_string()));
    println!("{}", c.total());
    println!("{:?}", c.most_common(2));
    let ci = Counter::from_list(&[1_i64, 2, 2, 3, 3, 3]);
    println!("{}", ci.get(&3));
    let combined = c.clone() + Counter::from_list(&["banana".to_string(), "date".to_string()]);
    println!("{}", combined.get(&"banana".to_string()));

    println!("=== Generic deque[T] ===");
    let mut d = Deque::new();
    d.append("first".to_string());
    d.append("second".to_string());
    d.appendleft("zero".to_string());
    println!("{:?}", d.to_list());
    println!("{}", d.len());

    println!("=== Generic reduce ===");
    let sentence = reduce(
        &["hello".to_string(), " ".to_string(), "world".to_string()],
        "".to_string(),
        |a, b| a + &b,
    );
    println!("{sentence}");

    println!("=== Generic compress ===");
    let data_c = vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
        "d".to_string(),
        "e".to_string(),
    ];
    println!("{:?}", compress(&data_c, &[true, false, true, false, true]));

    println!("=== Generic zip_longest ===");
    for pair in zip_longest(
        &["a".to_string(), "b".to_string(), "c".to_string()],
        &["x".to_string(), "y".to_string()],
        "-".to_string(),
    ) {
        println!("{pair:?}");
    }

    println!("=== Generic shuffle ===");
    let mut shuffled = vec!["a", "b", "c", "d", "e"];
    shuffled.rotate_left(2);
    println!("{}", shuffled.len());
}
