use std::collections::VecDeque;

fn drain(mut values: Vec<i64>) -> i64 {
    let mut total = 0;
    while let Some(item) = values.pop() {
        total += item;
    }
    total
}

fn drain_front(values: Vec<i64>) -> i64 {
    let mut total = 0;
    let mut values: VecDeque<i64> = values.into();
    while let Some(item) = values.pop_front() {
        total += item;
    }
    total
}

fn main() {
    assert_eq!(drain(vec![1, 2, 3, 4]), 10);
    assert_eq!(drain(vec![]), 0);
    assert_eq!(drain_front(vec![1, 2, 3, 4]), 10);
    assert_eq!(drain_front(vec![]), 0);
}
