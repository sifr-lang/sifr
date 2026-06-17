use std::sync::LazyLock;
use std::time::{Duration, Instant};

static TIMER_ZERO: LazyLock<Instant> = LazyLock::new(Instant::now);

fn default_timer() -> f64 {
    TIMER_ZERO.elapsed().as_secs_f64()
}

fn sleep(seconds: f64) {
    if seconds.is_finite() && seconds > 0.0 {
        std::thread::sleep(Duration::from_secs_f64(seconds));
    }
}

fn timeit<F>(mut workload: F, iterations: usize) -> f64
where
    F: FnMut(),
{
    let started = Instant::now();
    for _ in 0..iterations {
        workload();
    }
    started.elapsed().as_secs_f64()
}

fn repeat<F>(workload: F, count: usize, iterations: usize) -> Vec<f64>
where
    F: FnMut() + Copy,
{
    (0..count).map(|_| timeit(workload, iterations)).collect()
}

fn workload() {
    let _total: i64 = (0..100).sum();
}

fn all_non_negative(values: &[f64]) -> bool {
    values.iter().all(|value| *value >= 0.0)
}

fn collect_timer_actual() -> Vec<bool> {
    let before = default_timer();
    sleep(0.01);
    vec![default_timer() >= before]
}

fn collect_repeat_actual() -> Vec<bool> {
    let repeated = repeat(workload, 3, 10);
    vec![
        timeit(workload, 10) >= 0.0,
        repeated.len() == 3,
        all_non_negative(&repeated),
    ]
}

fn collect_edge_actual() -> Vec<bool> {
    vec![
        repeat(workload, 0, 5).is_empty(),
        timeit(workload, 0) >= 0.0,
        repeat(workload, 2, 0).len() == 2,
    ]
}

fn main() {
    let mut actual = Vec::new();
    actual.extend(collect_timer_actual());
    actual.extend(collect_repeat_actual());
    actual.extend(collect_edge_actual());

    let expected = vec![true, true, true, true, true, true, true];
    assert_eq!(actual, expected);
    println!("timeit timeit parity demo: pass");
}
