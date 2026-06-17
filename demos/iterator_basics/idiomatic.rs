use std::iter;

fn odds(limit: i64) -> impl Iterator<Item = i64> {
    (0..limit).filter(|value| value % 2 == 1)
}

fn repeat_n<T: Clone>(value: T, times: usize) -> impl Iterator<Item = T> {
    iter::repeat(value).take(times)
}

fn count_from(start: i64, step: i64) -> impl Iterator<Item = i64> {
    iter::successors(Some(start), move |value| Some(value + step))
}

fn combinations_of_two(values: &[i64]) -> Vec<Vec<i64>> {
    let mut result = Vec::new();
    for i in 0..values.len() {
        for j in i + 1..values.len() {
            result.push(vec![values[i], values[j]]);
        }
    }
    result
}

fn product_repeat_two(values: &[i64]) -> Vec<Vec<i64>> {
    let mut result = Vec::new();
    for &left in values {
        for &right in values {
            result.push(vec![left, right]);
        }
    }
    result
}

fn main() {
    let nums = vec![1_i64, 2, 3, 4];

    let mut it = nums.iter().copied();
    assert_eq!(it.next(), Some(1));
    assert_eq!(it.next(), Some(2));

    let doubled: Vec<i64> = nums.iter().map(|value| value * 2).collect();
    assert_eq!(format!("{doubled:?}"), "[2, 4, 6, 8]");

    let mut odd_it = odds(7);
    assert_eq!(odd_it.next(), Some(1));
    assert_eq!(odd_it.next(), Some(3));
    assert_eq!(odd_it.next(), Some(5));
    assert_eq!(odd_it.next(), None);

    let zipped: Vec<(i64, &str)> = [1_i64, 2].into_iter().zip(["a", "b"]).collect();
    assert_eq!(format!("{zipped:?}"), "[(1, \"a\"), (2, \"b\")]");

    let enumerated: Vec<(usize, &str)> = ["x", "y"]
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i + 4, v))
        .collect();
    assert_eq!(format!("{enumerated:?}"), "[(4, \"x\"), (5, \"y\")]");

    let reversed: Vec<i64> = [9_i64, 8, 7].into_iter().rev().collect();
    assert_eq!(format!("{reversed:?}"), "[7, 8, 9]");

    let chained: Vec<i64> = [1_i64, 2].into_iter().chain([3]).collect();
    assert_eq!(format!("{chained:?}"), "[1, 2, 3]");

    let repeated: Vec<i64> = repeat_n(5_i64, 3).collect();
    assert_eq!(format!("{repeated:?}"), "[5, 5, 5]");

    let sliced: Vec<i64> = [10_i64, 20, 30, 40, 50]
        .into_iter()
        .skip(1)
        .take(4)
        .step_by(2)
        .collect();
    assert_eq!(format!("{sliced:?}"), "[20, 40]");

    let mut counter = count_from(2, 3);
    assert_eq!(counter.next(), Some(2));
    assert_eq!(counter.next(), Some(5));
    assert_eq!(counter.next(), Some(8));

    let combos = combinations_of_two(&[1, 2, 3]);
    assert_eq!(format!("{combos:?}"), "[[1, 2], [1, 3], [2, 3]]");

    let products = product_repeat_two(&[1, 2]);
    assert_eq!(format!("{products:?}"), "[[1, 1], [1, 2], [2, 1], [2, 2]]");

    println!("iter_iterator_basics_closure_demo: ok");
}
