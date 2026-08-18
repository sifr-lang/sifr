// src/main.rs
// --- stdlib: sifr.test ---
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0_i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i += 1_i64;
    }
}

// --- stdlib: _sifr.time ---
fn time_now() -> f64 {
    ::sifr_stdlib::time::time_now()
}
fn time_format(epoch: f64, fmt: &String) -> String {
    ::sifr_stdlib::time::time_format(epoch, fmt)
}
fn perf_counter() -> f64 {
    ::sifr_stdlib::time::perf_counter()
}
fn sleep(seconds: f64) {
    ::sifr_stdlib::time::sleep(seconds);
}
fn monotonic() -> f64 {
    ::sifr_stdlib::time::monotonic()
}
fn strptime(s: &String, fmt: &String) -> Result<String, ValueError> {
    ::sifr_stdlib::time::strptime(s, fmt)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _strptime_intrinsic(s: &String, fmt: &String) -> Result<String, ValueError> {
    ::sifr_stdlib::time::strptime(s, fmt)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn gmtime(epoch: f64) -> String {
    ::sifr_stdlib::time::gmtime(epoch)
}
fn _gmtime_intrinsic(epoch: f64) -> String {
    ::sifr_stdlib::time::gmtime(epoch)
}
fn localtime(epoch: f64) -> String {
    ::sifr_stdlib::time::localtime(epoch)
}
fn _localtime_intrinsic(epoch: f64) -> String {
    ::sifr_stdlib::time::localtime(epoch)
}
fn time_strptime(s: &String, fmt: &String) -> Result<Vec<i64>, ValueError> {
    ::sifr_stdlib::time::time_strptime(s, fmt)
        .map(|__sifr_bridge_ok| {
            __sifr_bridge_ok
                .into_iter()
                .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
                .collect()
        })
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn time_gmtime() -> Vec<i64> {
    ::sifr_stdlib::time::time_gmtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn time_localtime() -> Vec<i64> {
    ::sifr_stdlib::time::time_localtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}

// --- stdlib: sifr.timeit ---
fn default_timer() -> f64 {
    perf_counter()
}
fn _elapsed_non_negative(start: f64, end: f64) -> f64 {
    let elapsed: f64 = end - start;
    if elapsed < (0.0_f64) {
        return 0.0_f64;
    }
    elapsed
}
fn timeit(stmt: impl Fn(), number: i64) -> f64 {
    let start: f64 = perf_counter();
    let mut i: i64 = 0_i64;
    while i < number {
        stmt();
        i += 1_i64;
    }
    let end: f64 = perf_counter();
    _elapsed_non_negative(start, end)
}
fn repeat(stmt: impl Fn(), count: i64, number: i64) -> Vec<f64> {
    let mut results: Vec<f64> = vec![];
    let mut r: i64 = 0_i64;
    while r < count {
        let start: f64 = perf_counter();
        let mut i: i64 = 0_i64;
        while i < number {
            stmt();
            i += 1_i64;
        }
        let end: f64 = perf_counter();
        let elapsed: f64 = _elapsed_non_negative(start, end);
        results.push(elapsed);
        r += 1_i64;
    }
    results
}
// --- end stdlib ---

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl ::std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for ValueError {
}

fn workload() {
    let mut total: i64 = 0_i64;
    let mut i: i64 = 0_i64;
    while i < (100_i64) {
        total += i;
        i += 1_i64;
    }
}

fn all_non_negative(values: &Vec<f64>) -> bool {
    let mut i: i64 = 0_i64;
    while (i < (values.len() as i64)) {
        let current: Option<f64> = Some(values[i as usize]);
        let Some(current) = current else {
            return false;
        };
        if current < (0.0_f64) {
            return false;
        }
        i += 1_i64;
    }
    true
}

fn collect_timer_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let t1: f64 = default_timer();
    sleep(0.01_f64);
    let t2: f64 = default_timer();
    actual.push(t2 >= t1);
    actual
}

fn collect_repeat_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let elapsed: f64 = timeit(workload, 10_i64);
    actual.push(elapsed >= (0.0_f64));
    let repeated: Vec<f64> = repeat(workload, 3_i64, 10_i64);
    actual.push((repeated.len() as i64) == (3_i64));
    actual.push(all_non_negative(&repeated));
    actual
}

fn collect_edge_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push((repeat(workload, 0_i64, 5_i64).len() as i64) == (0_i64));
    actual.push(timeit(workload, 0_i64) >= (0.0_f64));
    actual.push((repeat(workload, 2_i64, 0_i64).len() as i64) == (2_i64));
    actual
}

fn append_all(target: &mut Vec<bool>, values: &Vec<bool>) {
    for value in values.iter().copied() {
        target.push(value);
    }
}

fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true, true];
    let mut actual: Vec<bool> = vec![];
    append_all(&mut actual, &collect_timer_actual());
    append_all(&mut actual, &collect_repeat_actual());
    append_all(&mut actual, &collect_edge_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("timeit timeit parity demo: pass");
}
