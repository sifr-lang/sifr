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

// --- stdlib: sifr.heapq ---
fn _sift_down<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: &mut Vec<T>,
    mut pos: i64,
    n: i64,
) {
    let mut done: bool = false;
    while !done {
        let mut smallest: i64 = pos;
        let left: i64 = ((2 as i64) * pos) + (1 as i64);
        let right: i64 = ((2 as i64) * pos) + (2 as i64);
        if left < n {
            let s_val: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = smallest;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let l_val: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = left;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(s_val) = s_val {
                if let Some(l_val) = l_val {
                    if l_val < s_val {
                        smallest = left;
                    }
                }
            }
        }
        if right < n {
            let s_val2: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = smallest;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let r_val: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = right;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(s_val2) = s_val2 {
                if let Some(r_val) = r_val {
                    if r_val < s_val2 {
                        smallest = right;
                    }
                }
            }
        }
        if smallest == pos {
            done = true;
        } else {
            let tmp_pos: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = pos;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let tmp_sm: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = smallest;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(tmp_pos) = tmp_pos {
                if let Some(tmp_sm) = tmp_sm {
                    {
                        let __idx_raw = pos;
                        let __idx_norm = if __idx_raw < 0 {
                            (data.len() as i64) + __idx_raw
                        } else {
                            __idx_raw
                        };
                        if __idx_norm >= 0 {
                            if let Some(__elem) = data.get_mut(__idx_norm as usize) {
                                *__elem = tmp_sm;
                            }
                        }
                    }
                    {
                        let __idx_raw = smallest;
                        let __idx_norm = if __idx_raw < 0 {
                            (data.len() as i64) + __idx_raw
                        } else {
                            __idx_raw
                        };
                        if __idx_norm >= 0 {
                            if let Some(__elem) = data.get_mut(__idx_norm as usize) {
                                *__elem = tmp_pos;
                            }
                        }
                    }
                }
            }
            pos = smallest;
        }
    }
}
fn _sift_up<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
    mut pos: i64,
) {
    let mut done: bool = false;
    while !done {
        if pos <= (0 as i64) {
            done = true;
        } else {
            let parent: i64 = (pos - (1 as i64)) / (2 as i64);
            let p_val: Option<T> = {
                let __sifr_index_list = &heap;
                let __sifr_index_i = parent;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let c_val: Option<T> = {
                let __sifr_index_list = &heap;
                let __sifr_index_i = pos;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(p_val) = p_val {
                if let Some(c_val) = c_val {
                    if c_val < p_val {
                        {
                            let __idx_raw = parent;
                            let __idx_norm = if __idx_raw < 0 {
                                (heap.len() as i64) + __idx_raw
                            } else {
                                __idx_raw
                            };
                            if __idx_norm >= 0 {
                                if let Some(__elem) = heap.get_mut(__idx_norm as usize) {
                                    *__elem = c_val;
                                }
                            }
                        }
                        {
                            let __idx_raw = pos;
                            let __idx_norm = if __idx_raw < 0 {
                                (heap.len() as i64) + __idx_raw
                            } else {
                                __idx_raw
                            };
                            if __idx_norm >= 0 {
                                if let Some(__elem) = heap.get_mut(__idx_norm as usize) {
                                    *__elem = p_val;
                                }
                            }
                        }
                        pos = parent;
                    } else {
                        done = true;
                    }
                } else {
                    done = true;
                }
            } else {
                done = true;
            }
        }
    }
}
fn heapify<T: Clone + std::fmt::Display + PartialOrd + 'static>(data: &mut Vec<T>) {
    "Convert list to a min-heap in-place. O(n) time.".to_string();
    let n: i64 = data.len() as i64;
    let mut i: i64 = (n / (2 as i64)) - (1 as i64);
    while i >= (0 as i64) {
        _sift_down(data, i, n);
        i = i - (1 as i64);
    }
}
fn heappush<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
    item: &T,
) {
    "Push item onto the heap in-place. O(log n) time.".to_string();
    heap.push(item.clone());
    let pos: i64 = (heap.len() as i64) - (1 as i64);
    _sift_up(heap, pos);
}
fn heappop<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
) -> Option<T> {
    "Pop and return the smallest item. Heap is modified in-place. O(log n) time.\n    Returns None if the heap is empty."
        .to_string();
    let n: i64 = heap.len() as i64;
    if n == (0 as i64) {
        return None;
    }
    let top: Option<T> = {
        let __sifr_index_list = &heap;
        let __sifr_index_i = 0 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    let last: Option<T> = {
        let __sifr_index_list = &heap;
        let __sifr_index_i = n - (1 as i64);
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    heap.pop();
    let n2: i64 = heap.len() as i64;
    if n2 > (0 as i64) {
        if let Some(last) = last {
            {
                let __idx_raw = 0 as i64;
                let __idx_norm = if __idx_raw < 0 {
                    (heap.len() as i64) + __idx_raw
                } else {
                    __idx_raw
                };
                if __idx_norm >= 0 {
                    if let Some(__elem) = heap.get_mut(__idx_norm as usize) {
                        *__elem = last;
                    }
                }
            }
        }
        _sift_down(heap, 0 as i64, n2);
    }
    return top;
}
fn heapify_copy<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    for val in data.iter().cloned() {
        result.push(val.clone());
    }
    heapify(&mut result);
    return result;
}
fn nsmallest<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    n: i64,
    data: &Vec<T>,
) -> Vec<T> {
    let mut heap: Vec<T> = heapify_copy(data);
    let mut result: Vec<T> = vec![];
    let mut count: i64 = 0 as i64;
    while count < n {
        if (heap.len() as i64) == (0 as i64) {
            return result;
        }
        let val: Option<T> = heappop(&mut heap);
        if let Some(val) = val {
            result.push(val.clone());
        }
        count = count + (1 as i64);
    }
    return result;
}
fn nlargest<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    n: i64,
    data: &Vec<T>,
) -> Vec<T> {
    if n <= (0 as i64) {
        return vec![];
    }
    if n >= (data.len() as i64) {
        let mut result: Vec<T> = vec![];
        for val in data.iter().cloned() {
            result.push(val.clone());
        }
        return result;
    }
    let mut heap: Vec<T> = heapify_copy(data);
    let mut all_sorted: Vec<T> = vec![];
    while (heap.len() as i64) > (0 as i64) {
        let val2: Option<T> = heappop(&mut heap);
        if let Some(val2) = val2 {
            all_sorted.push(val2.clone());
        }
    }
    let mut result2: Vec<T> = vec![];
    let mut i: i64 = (all_sorted.len() as i64) - (1 as i64);
    let mut count: i64 = 0 as i64;
    while count < n {
        if i < (0 as i64) {
            return result2;
        }
        let v: Option<T> = {
            let __sifr_index_list = &all_sorted;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(v) = v {
            result2.push(v.clone());
        }
        i = i - (1 as i64);
        count = count + (1 as i64);
    }
    return result2;
}

fn collect_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut heap: Vec<i64> = vec![];
    heappush(&mut heap, &(5 as i64));
    heappush(&mut heap, &(1 as i64));
    heappush(&mut heap, &(3 as i64));
    let first: Option<i64> = heappop(&mut heap);
    let second: Option<i64> = heappop(&mut heap);
    actual.push(first.is_some() && (first == Some(1 as i64)));
    actual.push(second.is_some() && (second == Some(3 as i64)));
    let mut data: Vec<i64> = vec![4 as i64, 2 as i64, 7 as i64, 1 as i64, 5 as i64];
    heapify(&mut data);
    let top: Option<i64> = heappop(&mut data);
    actual.push(top.is_some() && (top == Some(1 as i64)));
    let items: Vec<i64> = vec![9 as i64, 3 as i64, 7 as i64, 1 as i64, 5 as i64];
    actual.push((format!("{:?}", nsmallest(3 as i64, &items))).as_str() == ("[1, 3, 5]".to_string()).as_str());
    actual.push((format!("{:?}", nlargest(2 as i64, &items))).as_str() == ("[9, 7]".to_string()).as_str());
    let mut empty_heap: Vec<i64> = vec![];
    actual.push(heappop(&mut empty_heap) == None);
    actual.push((format!("{:?}", items)).as_str() == ("[9, 3, 7, 1, 5]".to_string()).as_str());
    return actual;
}

fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true, true];
    let actual: Vec<bool> = collect_actual();
    assert_bool_vector_eq(&actual, &expected);
    println!("heapq heapq parity demo: pass");
}
