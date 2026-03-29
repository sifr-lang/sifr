use std::collections::BTreeMap;

#[derive(Clone, Debug)]
struct Counter<T: Ord + Clone> {
    counts: BTreeMap<T, i64>,
}

impl<T: Ord + Clone> Counter<T> {
    fn new(counts: BTreeMap<T, i64>) -> Self {
        Self { counts }
    }

    fn get(&self, key: &T) -> i64 {
        self.counts.get(key).copied().unwrap_or(0)
    }

    fn total(&self) -> i64 {
        self.counts.values().sum()
    }

    fn increment(&mut self, key: T) {
        *self.counts.entry(key).or_insert(0) += 1;
    }

    fn most_common(&self, count: usize) -> Vec<(T, i64)> {
        let mut entries = self
            .counts
            .iter()
            .map(|(key, value)| (key.clone(), *value))
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
        entries.into_iter().take(count).collect()
    }
}

fn from_list<T: Ord + Clone>(values: &[T]) -> Counter<T> {
    let mut counter = Counter::new(BTreeMap::new());
    for value in values {
        counter.increment(value.clone());
    }
    counter
}

fn main() {
    let words = vec![
        "apple".to_string(),
        "banana".to_string(),
        "apple".to_string(),
        "cherry".to_string(),
        "banana".to_string(),
        "apple".to_string(),
    ];
    let mut counter = from_list(&words);

    println!("{}", counter.get(&"apple".to_string()));
    println!("{}", counter.get(&"banana".to_string()));
    println!("{}", counter.get(&"cherry".to_string()));
    println!("{}", counter.get(&"missing".to_string()));
    println!("{}", counter.total());
    println!("{:?}", counter.most_common(2));

    counter.increment("banana".to_string());
    counter.increment("banana".to_string());
    println!("{}", counter.get(&"banana".to_string()));
    println!("{}", counter.total());

    counter.increment("date".to_string());
    println!("{}", counter.get(&"date".to_string()));
    println!("{}", counter.total());

    let direct = Counter::new(BTreeMap::from([
        ("x".to_string(), 10),
        ("y".to_string(), 20),
    ]));
    println!("{}", direct.get(&"x".to_string()));
    println!("{}", direct.get(&"y".to_string()));
    println!("{}", direct.total());
}
