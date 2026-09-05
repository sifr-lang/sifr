// src/main.rs
pub mod sifr_generated_generated_support {
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn bisect_left<T: Clone + 'static + PartialOrd>(
        a: &[T],
        x: &T,
        lo: &SifrInt,
        hi: Option<&SifrInt>,
    ) -> SifrInt {
        let hi: Option<SifrInt> = hi.cloned();
        let mut left: SifrInt = (*lo).clone();
        if left < SifrInt::from_i64(0) {
            left = SifrInt::from_i64(0);
        }
        let mut right: SifrInt = SifrInt::from(a.len());
        if hi.is_none() {
            right = SifrInt::from(a.len());
        } else if let Some(hi) = hi {
            if hi < SifrInt::from_i64(0) {
                right = SifrInt::from_i64(0);
            } else if hi > a.len() {
                right = SifrInt::from(a.len());
            } else {
                right = hi;
            }
        }
        while left < right {
            let mid: SifrInt =
                ::std::ops::Add::add(&left, &right).floor_div_known_nonzero(&SifrInt::from_i64(2));
            let val: Option<T> = {
                let sifr_generated_checked_read_collection = &a;
                let sifr_generated_checked_read_index = &mid;
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(val) = val {
                if val < *x {
                    left = ::std::ops::Add::add(&mid, &SifrInt::from_i64(1));
                } else {
                    right = mid;
                }
            } else {
                left = ::std::ops::Add::add(&mid, &SifrInt::from_i64(1));
            }
        }
        left
    }
    pub(super) fn bisect_right<T: Clone + 'static + PartialOrd>(
        a: &[T],
        x: &T,
        lo: &SifrInt,
        hi: Option<&SifrInt>,
    ) -> SifrInt {
        let hi: Option<SifrInt> = hi.cloned();
        let mut left: SifrInt = (*lo).clone();
        if left < SifrInt::from_i64(0) {
            left = SifrInt::from_i64(0);
        }
        let mut right: SifrInt = SifrInt::from(a.len());
        if hi.is_none() {
            right = SifrInt::from(a.len());
        } else if let Some(hi) = hi {
            if hi < SifrInt::from_i64(0) {
                right = SifrInt::from_i64(0);
            } else if hi > a.len() {
                right = SifrInt::from(a.len());
            } else {
                right = hi;
            }
        }
        while left < right {
            let mid: SifrInt =
                ::std::ops::Add::add(&left, &right).floor_div_known_nonzero(&SifrInt::from_i64(2));
            let val: Option<T> = {
                let sifr_generated_checked_read_collection = &a;
                let sifr_generated_checked_read_index = &mid;
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(val) = val {
                if *x < val {
                    right = mid;
                } else {
                    left = ::std::ops::Add::add(&mid, &SifrInt::from_i64(1));
                }
            } else {
                left = ::std::ops::Add::add(&mid, &SifrInt::from_i64(1));
            }
        }
        left
    }
    pub(super) fn insort_left<T: Clone + 'static + PartialOrd>(
        a: &mut Vec<T>,
        x: &T,
        lo: &SifrInt,
        hi: Option<&SifrInt>,
    ) {
        let hi: Option<SifrInt> = hi.cloned();
        let pos: SifrInt = bisect_left(a, x, lo, hi.as_ref());
        a.insert(pos.clamp_slice_bound(a.len()), x.clone());
    }
    pub(super) fn insort_right<T: Clone + 'static + PartialOrd>(
        a: &mut Vec<T>,
        x: &T,
        lo: &SifrInt,
        hi: Option<&SifrInt>,
    ) {
        let hi: Option<SifrInt> = hi.cloned();
        let pos: SifrInt = bisect_right(a, x, lo, hi.as_ref());
        a.insert(pos.clamp_slice_bound(a.len()), x.clone());
    }
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
}
use crate::sifr_generated_generated_support::{
    assert_bool_vector_eq, bisect_left, bisect_right, insort_left, insort_right,
};
use ::sifr_runtime::SifrInt;
fn collect_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let data: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(5),
    ];
    actual.push(
        bisect_left(&data, &SifrInt::from_i64(2), &SifrInt::from_i64(0), None)
            == SifrInt::from_i64(1),
    );
    actual.push(
        bisect_right(&data, &SifrInt::from_i64(2), &SifrInt::from_i64(0), None)
            == SifrInt::from_i64(3),
    );
    actual.push(
        bisect_left(&data, &SifrInt::from_i64(4), &SifrInt::from_i64(0), None)
            == SifrInt::from_i64(4),
    );
    actual.push(
        bisect_right(&data, &SifrInt::from_i64(4), &SifrInt::from_i64(0), None)
            == SifrInt::from_i64(4),
    );
    let mut left_mut: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(3),
        SifrInt::from_i64(3),
        SifrInt::from_i64(5),
    ];
    insort_left(
        &mut left_mut,
        &SifrInt::from_i64(3),
        &SifrInt::from_i64(0),
        None,
    );
    actual.push(format!("{left_mut:?}").as_str() == "[1, 3, 3, 3, 5]");
    let mut right_mut: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(3),
        SifrInt::from_i64(3),
        SifrInt::from_i64(5),
    ];
    insort_right(
        &mut right_mut,
        &SifrInt::from_i64(3),
        &SifrInt::from_i64(0),
        None,
    );
    actual.push(format!("{right_mut:?}").as_str() == "[1, 3, 3, 3, 5]");
    let mut empty: Vec<SifrInt> = Vec::new();
    actual.push(
        bisect_left(&empty, &SifrInt::from_i64(10), &SifrInt::from_i64(0), None)
            == SifrInt::from_i64(0),
    );
    insort_right(
        &mut empty,
        &SifrInt::from_i64(10),
        &SifrInt::from_i64(0),
        None,
    );
    actual.push(format!("{empty:?}").as_str() == "[10]");
    actual
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true, true, true];
    let actual: Vec<bool> = collect_actual();
    assert_bool_vector_eq(&actual, &expected);
    println!("bisect_right bisect_right parity demo: pass");
}
