// --- stdlib: sifr.test ---
fn assert_eq<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    actual: &T,
    expected: &T,
) {
    assert!(* actual == * expected);
}
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0 as i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i = i + (1 as i64);
    }
}

// --- stdlib: sifr.bisect ---
fn bisect_left<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    a: &Vec<T>,
    x: &T,
    lo: i64,
    hi: Option<i64>,
) -> i64 {
    let mut left: i64 = lo;
    if left < (0 as i64) {
        left = 0 as i64;
    }
    let mut right: i64 = a.len() as i64;
    if hi.is_none() {
        right = a.len() as i64;
    } else {
        if let Some(hi) = hi {
            if hi < (0 as i64) {
                right = 0 as i64;
            } else {
                if hi > (a.len() as i64) {
                    right = a.len() as i64;
                } else {
                    right = hi;
                }
            }
        }
    }
    while left < right {
        let mid: i64 = (left + right) / (2 as i64);
        let val: Option<T> = {
            let __sifr_index_list = &a;
            let __sifr_index_i = mid;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(val) = val {
            if val < *x {
                left = mid + (1 as i64);
            } else {
                right = mid;
            }
        } else {
            left = mid + (1 as i64);
        }
    }
    return left;
}
fn bisect_right<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    a: &Vec<T>,
    x: &T,
    lo: i64,
    hi: Option<i64>,
) -> i64 {
    let mut left: i64 = lo;
    if left < (0 as i64) {
        left = 0 as i64;
    }
    let mut right: i64 = a.len() as i64;
    if hi.is_none() {
        right = a.len() as i64;
    } else {
        if let Some(hi) = hi {
            if hi < (0 as i64) {
                right = 0 as i64;
            } else {
                if hi > (a.len() as i64) {
                    right = a.len() as i64;
                } else {
                    right = hi;
                }
            }
        }
    }
    while left < right {
        let mid: i64 = (left + right) / (2 as i64);
        let val: Option<T> = {
            let __sifr_index_list = &a;
            let __sifr_index_i = mid;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(val) = val {
            if *x < val {
                right = mid;
            } else {
                left = mid + (1 as i64);
            }
        } else {
            left = mid + (1 as i64);
        }
    }
    return left;
}
fn insort_left<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    a: &mut Vec<T>,
    x: &T,
    lo: i64,
    hi: Option<i64>,
) {
    let pos: i64 = bisect_left(a, x, lo, hi);
    a.insert(pos as usize, x.clone());
}
fn insort_right<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    a: &mut Vec<T>,
    x: &T,
    lo: i64,
    hi: Option<i64>,
) {
    let pos: i64 = bisect_right(a, x, lo, hi);
    a.insert(pos as usize, x.clone());
}

fn collect_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let data: Vec<i64> = vec![1 as i64, 2 as i64, 2 as i64, 3 as i64, 5 as i64];
    actual.push(bisect_left(&data, &(2 as i64), 0 as i64, None) == (1 as i64));
    actual.push(bisect_right(&data, &(2 as i64), 0 as i64, None) == (3 as i64));
    actual.push(bisect_left(&data, &(4 as i64), 0 as i64, None) == (4 as i64));
    actual.push(bisect_right(&data, &(4 as i64), 0 as i64, None) == (4 as i64));
    let mut left_mut: Vec<i64> = vec![1 as i64, 3 as i64, 3 as i64, 5 as i64];
    insort_left(&mut left_mut, &(3 as i64), 0 as i64, None);
    actual.push((format!("{:?}", left_mut)).as_str() == ("[1, 3, 3, 3, 5]".to_string()).as_str());
    let mut right_mut: Vec<i64> = vec![1 as i64, 3 as i64, 3 as i64, 5 as i64];
    insort_right(&mut right_mut, &(3 as i64), 0 as i64, None);
    actual.push((format!("{:?}", right_mut)).as_str() == ("[1, 3, 3, 3, 5]".to_string()).as_str());
    let mut empty: Vec<i64> = vec![];
    actual.push(bisect_left(&empty, &(10 as i64), 0 as i64, None) == (0 as i64));
    insort_right(&mut empty, &(10 as i64), 0 as i64, None);
    actual.push((format!("{:?}", empty)).as_str() == ("[10]".to_string()).as_str());
    return actual;
}

fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true, true, true];
    let actual: Vec<bool> = collect_actual();
    assert_bool_vector_eq(&actual, &expected);
    println!("bisect bisect parity demo: pass");
}
