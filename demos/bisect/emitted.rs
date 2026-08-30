// src/main.rs
use ::sifr_runtime::SifrInt;

// --- stdlib: sifr.bisect ---
fn bisect_left<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    a: &Vec<T>,
    x: &T,
    lo: SifrInt,
    hi: Option<SifrInt>,
) -> SifrInt {
    let mut left: SifrInt = lo.clone();
    if &left < &SifrInt::from_i64(0) {
        left = SifrInt::from_i64(0);
    }
    let mut right: SifrInt = SifrInt::from(a.len());
    if (hi == None) {
        right = SifrInt::from(a.len());
    } else {
        if let Some(hi) = hi.clone() {
            if (&hi < &SifrInt::from_i64(0)) {
                right = SifrInt::from_i64(0);
            } else {
                if (&hi > &SifrInt::from(a.len())) {
                    right = SifrInt::from(a.len());
                } else {
                    right = hi;
                }
            }
        }
    }
    while (&left < &right) {
        let mid: SifrInt = (&left + &right)
            .floor_div_known_nonzero(&SifrInt::from_i64(2));
        let val: Option<T> = {
            let __sifr_checked_read_collection = &a;
            let __sifr_checked_read_index = mid.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(val) = val {
            if val < *x {
                left = &mid + &SifrInt::from_i64(1);
            } else {
                right = mid;
            }
        } else {
            left = &mid + &SifrInt::from_i64(1);
        }
    }
    left.clone()
}
fn bisect_right<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    a: &Vec<T>,
    x: &T,
    lo: SifrInt,
    hi: Option<SifrInt>,
) -> SifrInt {
    let mut left: SifrInt = lo.clone();
    if &left < &SifrInt::from_i64(0) {
        left = SifrInt::from_i64(0);
    }
    let mut right: SifrInt = SifrInt::from(a.len());
    if (hi == None) {
        right = SifrInt::from(a.len());
    } else {
        if let Some(hi) = hi.clone() {
            if (&hi < &SifrInt::from_i64(0)) {
                right = SifrInt::from_i64(0);
            } else {
                if (&hi > &SifrInt::from(a.len())) {
                    right = SifrInt::from(a.len());
                } else {
                    right = hi;
                }
            }
        }
    }
    while (&left < &right) {
        let mid: SifrInt = (&left + &right)
            .floor_div_known_nonzero(&SifrInt::from_i64(2));
        let val: Option<T> = {
            let __sifr_checked_read_collection = &a;
            let __sifr_checked_read_index = mid.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(val) = val {
            if *x < val {
                right = mid;
            } else {
                left = &mid + &SifrInt::from_i64(1);
            }
        } else {
            left = &mid + &SifrInt::from_i64(1);
        }
    }
    left.clone()
}
fn insort_left<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    a: &mut Vec<T>,
    x: &T,
    lo: SifrInt,
    hi: Option<SifrInt>,
) {
    let pos: SifrInt = bisect_left(a, x, (lo).clone(), (hi).clone());
    a.insert(::sifr_runtime::to_usize_proven(&pos), x.clone());
}
fn insort_right<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    a: &mut Vec<T>,
    x: &T,
    lo: SifrInt,
    hi: Option<SifrInt>,
) {
    let pos: SifrInt = bisect_right(a, x, (lo).clone(), (hi).clone());
    a.insert(::sifr_runtime::to_usize_proven(&pos), x.clone());
}

// --- stdlib: sifr.test ---
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
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
// --- end stdlib ---

fn collect_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let data: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(5)];
    actual.push(&bisect_left(&data, &SifrInt::from_i64(2), SifrInt::from_i64(0), None) == &SifrInt::from_i64(1));
    actual.push(&bisect_right(&data, &SifrInt::from_i64(2), SifrInt::from_i64(0), None) == &SifrInt::from_i64(3));
    actual.push(&bisect_left(&data, &SifrInt::from_i64(4), SifrInt::from_i64(0), None) == &SifrInt::from_i64(4));
    actual.push(&bisect_right(&data, &SifrInt::from_i64(4), SifrInt::from_i64(0), None) == &SifrInt::from_i64(4));
    let mut left_mut: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(3), SifrInt::from_i64(3), SifrInt::from_i64(5)];
    insort_left(&mut left_mut, &SifrInt::from_i64(3), SifrInt::from_i64(0), None);
    actual.push((format!("{:?}", left_mut)).as_str() == ("[1, 3, 3, 3, 5]".to_string()).as_str());
    let mut right_mut: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(3), SifrInt::from_i64(3), SifrInt::from_i64(5)];
    insort_right(&mut right_mut, &SifrInt::from_i64(3), SifrInt::from_i64(0), None);
    actual.push((format!("{:?}", right_mut)).as_str() == ("[1, 3, 3, 3, 5]".to_string()).as_str());
    let mut empty: Vec<SifrInt> = vec![];
    actual.push(&bisect_left(&empty, &SifrInt::from_i64(10), SifrInt::from_i64(0), None) == &SifrInt::from_i64(0));
    insort_right(&mut empty, &SifrInt::from_i64(10), SifrInt::from_i64(0), None);
    actual.push((format!("{:?}", empty)).as_str() == ("[10]".to_string()).as_str());
    actual
}

fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true, true, true];
    let actual: Vec<bool> = collect_actual();
    assert_bool_vector_eq(&actual, &expected);
    println!("bisect_right bisect_right parity demo: pass");
}
