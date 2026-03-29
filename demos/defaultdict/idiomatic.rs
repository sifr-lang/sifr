use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

#[derive(Debug, Clone)]
struct DefaultDict<K, V, F>
where
    K: Eq + Hash,
    F: Fn() -> V,
{
    values: HashMap<K, V>,
    factory: F,
}

impl<K, V, F> DefaultDict<K, V, F>
where
    K: Eq + Hash,
    F: Fn() -> V,
{
    fn new(factory: F) -> Self {
        Self {
            values: HashMap::new(),
            factory,
        }
    }

    fn get_mut(&mut self, key: K) -> &mut V {
        self.values.entry(key).or_insert_with(|| (self.factory)())
    }
}

#[derive(Debug, Clone)]
struct Deque<T> {
    values: VecDeque<T>,
}

impl<T: Clone> Deque<T> {
    fn from_slice(values: &[T]) -> Self {
        Self {
            values: values.iter().cloned().collect(),
        }
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

fn main() {
    let mut groups = DefaultDict::new(Vec::<String>::new);
    groups.get_mut("hit".to_string()).push("hot".to_string());
    groups.get_mut("hit".to_string()).push("hut".to_string());
    assert_eq!(groups.get_mut("hit".to_string()).len(), 2);

    let mut seen = DefaultDict::new(HashSet::<String>::new);
    seen.get_mut(1).insert("a".to_string());
    seen.get_mut(1).insert("b".to_string());
    assert!(seen.get_mut(1).contains("a"));

    let mut counts = DefaultDict::new(|| 0_i64);
    *counts.get_mut("steps".to_string()) += 1;
    *counts.get_mut("steps".to_string()) += 2;
    assert_eq!(*counts.get_mut("steps".to_string()), 3);

    let queue = Deque::from_slice(&[1, 2, 3]);
    assert_eq!(queue.len(), 3);
}
