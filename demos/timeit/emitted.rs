// src/main.rs
pub mod sifr_generated_generated_support {
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
        assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < actual.len() {
            assert_eq!(
                {
                    let sifr_generated_condition_list = &actual;
                    let sifr_generated_condition_index = i.clone();
                    let sifr_generated_condition_normalized = sifr_generated_condition_index
                        .normalize_index_or_len(sifr_generated_condition_list.len());
                    sifr_generated_condition_list
                        .get(sifr_generated_condition_normalized)
                        .copied()
                },
                {
                    let sifr_generated_condition_list = &expected;
                    let sifr_generated_condition_index = i.clone();
                    let sifr_generated_condition_normalized = sifr_generated_condition_index
                        .normalize_index_or_len(sifr_generated_condition_list.len());
                    sifr_generated_condition_list
                        .get(sifr_generated_condition_normalized)
                        .copied()
                }
            );
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
    }
    pub(super) fn perf_counter() -> f64 {
        ::sifr_stdlib::time::perf_counter()
    }
    pub(super) fn sleep(seconds: f64) {
        ::sifr_stdlib::time::sleep(seconds);
    }
    pub(super) fn default_timer() -> f64 {
        perf_counter()
    }
    pub(super) fn sifr_generated_elapsed_non_negative(start: f64, end: f64) -> f64 {
        let elapsed: f64 = end - start;
        if elapsed < 0.0_f64 {
            return 0.0_f64;
        }
        elapsed
    }
    #[expect(
        clippy::needless_pass_by_value,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn timeit(stmt: impl Fn(), number: SifrInt) -> f64 {
        let start: f64 = perf_counter();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < number {
            stmt();
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
        let end: f64 = perf_counter();
        sifr_generated_elapsed_non_negative(start, end)
    }
    #[expect(
        clippy::needless_pass_by_value,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn repeat(stmt: impl Fn(), count: SifrInt, number: SifrInt) -> Vec<f64> {
        let mut results: Vec<f64> = Vec::new();
        let mut r: SifrInt = SifrInt::from_i64(0);
        while r < count {
            let start: f64 = perf_counter();
            let mut i: SifrInt = SifrInt::from_i64(0);
            while i < number {
                stmt();
                i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
            }
            let end: f64 = perf_counter();
            let elapsed: f64 = sifr_generated_elapsed_non_negative(start, end);
            results.push(elapsed);
            r = ::std::ops::Add::add(&r, &SifrInt::from_i64(1));
        }
        results
    }
}
use crate::sifr_generated_generated_support::{
    assert_bool_vector_eq, default_timer, repeat, sleep, timeit,
};
use ::sifr_runtime::SifrInt;
fn workload() {
    let mut total: SifrInt = SifrInt::from_i64(0);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while i < SifrInt::from_i64(100) {
        total = ::std::ops::Add::add(&total, &i);
        i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
    }
}
fn all_non_negative(values: &[f64]) -> bool {
    let mut i: SifrInt = SifrInt::from_i64(0);
    while i < values.len() {
        let current: Option<f64> = {
            let sifr_generated_checked_read_collection = &values;
            let sifr_generated_checked_read_index = &i;
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .copied()
        };
        let Some(current_value_2a2e8a5afcc8d89a) = current else {
            return false;
        };
        if current_value_2a2e8a5afcc8d89a < 0.0_f64 {
            return false;
        }
        i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
    }
    true
}
fn collect_timer_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let t1: f64 = default_timer();
    sleep(0.01_f64);
    let t2: f64 = default_timer();
    actual.push(t2 >= t1);
    actual
}
fn collect_repeat_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let elapsed: f64 = timeit(workload, SifrInt::from_i64(10));
    actual.push(elapsed >= 0.0_f64);
    let repeated: Vec<f64> = repeat(workload, SifrInt::from_i64(3), SifrInt::from_i64(10));
    actual.push(repeated.len() == SifrInt::from_i64(3));
    actual.push(all_non_negative(&repeated));
    actual
}
fn collect_edge_actual() -> Vec<bool> {
    vec![
        repeat(workload, SifrInt::from_i64(0), SifrInt::from_i64(5)).len() == SifrInt::from_i64(0),
        timeit(workload, SifrInt::from_i64(0)) >= 0.0_f64,
        repeat(workload, SifrInt::from_i64(2), SifrInt::from_i64(0)).len() == SifrInt::from_i64(2),
    ]
}
fn append_all(target: &mut Vec<bool>, values: &[bool]) {
    for value in values.iter().copied() {
        target.push(value);
    }
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true, true];
    let mut actual: Vec<bool> = Vec::new();
    append_all(&mut actual, &collect_timer_actual());
    append_all(&mut actual, &collect_repeat_actual());
    append_all(&mut actual, &collect_edge_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("timeit timeit parity demo: pass");
}
