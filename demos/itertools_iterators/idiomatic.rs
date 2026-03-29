use std::iter;

fn repeat_n<T: Clone>(value: T, times: usize) -> impl Iterator<Item = T> {
    iter::repeat(value).take(times)
}

fn count_from(start: i64, step: i64) -> impl Iterator<Item = i64> {
    iter::successors(Some(start), move |value| Some(value + step))
}

fn main() {
    let chained: Vec<i64> = [1_i64, 2].into_iter().chain([3]).collect();
    println!("{chained:?}");

    let repeated: Vec<i64> = repeat_n(7_i64, 3).collect();
    println!("{repeated:?}");

    let sliced: Vec<i64> = [10_i64, 20, 30, 40, 50]
        .into_iter()
        .skip(1)
        .take(4)
        .step_by(2)
        .collect();
    println!("{sliced:?}");

    let mut counter = count_from(5, 2);
    for _ in 0..4 {
        if let Some(value) = counter.next() {
            println!("{value}");
        }
    }
}
