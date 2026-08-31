// src/main.rs
use ::sifr_runtime::SifrInt;

// --- stdlib: sifr.test ---
fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
        assert!(
            ({ let __sifr_condition_list = & actual; let __sifr_condition_index = i
            .clone(); let __sifr_condition_normalized = __sifr_condition_index
            .normalize_index_or_len(__sifr_condition_list.len()); __sifr_condition_list
            .get(__sifr_condition_normalized).copied() }) == ({ let __sifr_condition_list
            = & expected; let __sifr_condition_index = i.clone(); let
            __sifr_condition_normalized = __sifr_condition_index
            .normalize_index_or_len(__sifr_condition_list.len()); __sifr_condition_list
            .get(__sifr_condition_normalized).copied() })
        );
        i = &i + &SifrInt::from_i64(1);
    }
}

// --- stdlib: _sifr.time ---
fn time_now() -> f64 {
    ::sifr_stdlib::time::time_now()
}
fn time_format(epoch: f64, fmt: &str) -> String {
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
fn strptime(s: &str, fmt: &str) -> Result<String, ValueError> {
    ::sifr_stdlib::time::strptime(s, fmt)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _strptime_intrinsic(s: &str, fmt: &str) -> Result<String, ValueError> {
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
fn time_strptime(s: &str, fmt: &str) -> Result<Vec<SifrInt>, ValueError> {
    ::sifr_stdlib::time::time_strptime(s, fmt)
        .map(|__sifr_bridge_ok| {
            __sifr_bridge_ok
                .into_iter()
                .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
                .collect()
        })
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn time_gmtime() -> Vec<SifrInt> {
    ::sifr_stdlib::time::time_gmtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
fn time_localtime() -> Vec<SifrInt> {
    ::sifr_stdlib::time::time_localtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
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
fn timeit(stmt: impl Fn(), number: SifrInt) -> f64 {
    let start: f64 = perf_counter();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &number) {
        stmt();
        i = &i + &SifrInt::from_i64(1);
    }
    let end: f64 = perf_counter();
    _elapsed_non_negative(start, end)
}
fn repeat(stmt: impl Fn(), count: SifrInt, number: SifrInt) -> Vec<f64> {
    let mut results: Vec<f64> = vec![];
    let mut r: SifrInt = SifrInt::from_i64(0);
    while (&r < &count) {
        let start: f64 = perf_counter();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while (&i < &number) {
            stmt();
            i = &i + &SifrInt::from_i64(1);
        }
        let end: f64 = perf_counter();
        let elapsed: f64 = _elapsed_non_negative(start, end);
        results.push(elapsed);
        r = &r + &SifrInt::from_i64(1);
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
    let mut total: SifrInt = SifrInt::from_i64(0);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from_i64(100) {
        total = &total + &i;
        i = &i + &SifrInt::from_i64(1);
    }
}

fn all_non_negative(values: &[f64]) -> bool {
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(values.len())) {
        let current: Option<f64> = {
    let __sifr_checked_read_collection = &values;
    let __sifr_checked_read_index = i.clone();
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
};
        let Some(current) = current else {
            return false;
        };
        if (current < (0.0_f64)) {
            return false;
        }
        i = &i + &SifrInt::from_i64(1);
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
    let elapsed: f64 = timeit(workload, SifrInt::from_i64(10));
    actual.push(elapsed >= (0.0_f64));
    let repeated: Vec<f64> = repeat(workload, SifrInt::from_i64(3), SifrInt::from_i64(10));
    actual.push(&SifrInt::from(repeated.len()) == &SifrInt::from_i64(3));
    actual.push(all_non_negative(&repeated));
    actual
}

fn collect_edge_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push(&SifrInt::from(repeat(workload, SifrInt::from_i64(0), SifrInt::from_i64(5)).len()) == &SifrInt::from_i64(0));
    actual.push(timeit(workload, SifrInt::from_i64(0)) >= (0.0_f64));
    actual.push(&SifrInt::from(repeat(workload, SifrInt::from_i64(2), SifrInt::from_i64(0)).len()) == &SifrInt::from_i64(2));
    actual
}

fn append_all(target: &mut Vec<bool>, values: &[bool]) {
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
