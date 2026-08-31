// src/main.rs
use ::sifr_runtime::SifrInt;

// --- stdlib: sifr.heapq ---
fn _sift_down<T: Clone + 'static + PartialOrd>(
    data: &mut Vec<T>,
    mut pos: SifrInt,
    n: SifrInt,
) {
    let mut done: bool = false;
    while !done {
        let mut smallest: SifrInt = pos.clone();
        let left: SifrInt = &(&SifrInt::from_i64(2) * &pos) + &SifrInt::from_i64(1);
        let right: SifrInt = &(&SifrInt::from_i64(2) * &pos) + &SifrInt::from_i64(2);
        if (&left < &n) {
            let s_val: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = smallest.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            let l_val: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = left.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(s_val) = s_val {
                if let Some(l_val) = l_val {
                    if (l_val < s_val) {
                        smallest = left;
                    }
                }
            }
        }
        if (&right < &n) {
            let s_val2: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = smallest.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            let r_val: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = right.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(s_val2) = s_val2 {
                if let Some(r_val) = r_val {
                    if (r_val < s_val2) {
                        smallest = right;
                    }
                }
            }
        }
        if (&smallest == &pos) {
            done = true;
        } else {
            let tmp_pos: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = pos.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            let tmp_sm: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = smallest.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(tmp_pos) = tmp_pos {
                if let Some(tmp_sm) = tmp_sm {
                    if (&SifrInt::from_i64(0) <= &pos)
                        && (&pos < &SifrInt::from(data.len()))
                    {
                        {
                            let __assign_value = tmp_sm.clone();
                            {
                                let __index_raw = pos.clone();
                                let __index_normalized = __index_raw
                                    .normalize_index_or_len(data.len());
                                if let Some(__elem) = data.get_mut(__index_normalized) {
                                    *__elem = __assign_value;
                                }
                            }
                        }
                    }
                    if (&SifrInt::from_i64(0) <= &smallest)
                        && (&smallest < &SifrInt::from(data.len()))
                    {
                        {
                            let __assign_value = tmp_pos.clone();
                            {
                                let __index_raw = smallest.clone();
                                let __index_normalized = __index_raw
                                    .normalize_index_or_len(data.len());
                                if let Some(__elem) = data.get_mut(__index_normalized) {
                                    *__elem = __assign_value;
                                }
                            }
                        }
                    }
                }
            }
            pos = smallest;
        }
    }
}
fn _sift_up<T: Clone + 'static + PartialOrd>(heap: &mut Vec<T>, mut pos: SifrInt) {
    let mut done: bool = false;
    while !done {
        if (&pos <= &SifrInt::from_i64(0)) {
            done = true;
        } else {
            let parent: SifrInt = (&pos - &SifrInt::from_i64(1))
                .floor_div_known_nonzero(&SifrInt::from_i64(2));
            let p_val: Option<T> = {
                let __sifr_checked_read_collection = &heap;
                let __sifr_checked_read_index = parent.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            let c_val: Option<T> = {
                let __sifr_checked_read_collection = &heap;
                let __sifr_checked_read_index = pos.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(p_val) = p_val {
                if let Some(c_val) = c_val {
                    if (c_val < p_val) {
                        if (&SifrInt::from_i64(0) <= &parent)
                            && (&parent < &SifrInt::from(heap.len()))
                        {
                            {
                                let __assign_value = c_val.clone();
                                {
                                    let __index_raw = parent.clone();
                                    let __index_normalized = __index_raw
                                        .normalize_index_or_len(heap.len());
                                    if let Some(__elem) = heap.get_mut(__index_normalized) {
                                        *__elem = __assign_value;
                                    }
                                }
                            }
                        }
                        if (&SifrInt::from_i64(0) <= &pos)
                            && (&pos < &SifrInt::from(heap.len()))
                        {
                            {
                                let __assign_value = p_val.clone();
                                {
                                    let __index_raw = pos.clone();
                                    let __index_normalized = __index_raw
                                        .normalize_index_or_len(heap.len());
                                    if let Some(__elem) = heap.get_mut(__index_normalized) {
                                        *__elem = __assign_value;
                                    }
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
fn heapify<T: Clone + 'static>(data: &mut Vec<T>) {
    "Convert list to a min-heap in-place. O(n) time.".to_string();
    let n: SifrInt = SifrInt::from(data.len());
    let mut i: SifrInt = &n.floor_div_known_nonzero(&SifrInt::from_i64(2))
        - &SifrInt::from_i64(1);
    while (&i >= &SifrInt::from_i64(0)) {
        _sift_down(data, (i).clone(), (n).clone());
        i = &i - &SifrInt::from_i64(1);
    }
}
fn heappush<T: Clone + 'static>(heap: &mut Vec<T>, item: &T) {
    "Push item onto the heap in-place. O(log n) time.".to_string();
    heap.push(item.clone());
    let pos: SifrInt = &SifrInt::from(heap.len()) - &SifrInt::from_i64(1);
    _sift_up(heap, (pos).clone());
}
fn heappop<T: Clone + 'static>(heap: &mut Vec<T>) -> Option<T> {
    "Pop and return the smallest item. Heap is modified in-place. O(log n) time.\n    Returns None if the heap is empty."
        .to_string();
    let n: SifrInt = SifrInt::from(heap.len());
    if &n == &SifrInt::from_i64(0) {
        return None;
    }
    let top: Option<T> = {
        let __sifr_checked_read_collection = &heap;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let last: Option<T> = {
        let __sifr_checked_read_collection = &heap;
        let __sifr_checked_read_index = &n - &SifrInt::from_i64(1);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    heap.remove(heap.len() - (1_usize));
    let n2: SifrInt = SifrInt::from(heap.len());
    if (&n2 > &SifrInt::from_i64(0)) {
        if let Some(last) = last {
            {
                let __assign_value = last.clone();
                {
                    let __index_raw = SifrInt::from_i64(0);
                    let __index_normalized = __index_raw
                        .normalize_index_or_len(heap.len());
                    if let Some(__elem) = heap.get_mut(__index_normalized) {
                        *__elem = __assign_value;
                    }
                }
            }
        }
        _sift_down(heap, SifrInt::from_i64(0), (n2).clone());
    }
    top
}
fn nsmallest<T: Clone + 'static>(n: SifrInt, data: &Vec<T>) -> Vec<T> {
    let mut heap: Vec<T> = data.clone();
    heapify(&mut heap);
    let mut result: Vec<T> = vec![];
    let mut count: SifrInt = SifrInt::from_i64(0);
    while (&count < &n) {
        if (&SifrInt::from(heap.len()) == &SifrInt::from_i64(0)) {
            return result;
        }
        let val: Option<T> = heappop(&mut heap);
        if let Some(val) = val {
            result.push(val.clone());
        }
        count = &count + &SifrInt::from_i64(1);
    }
    result
}
fn nlargest<T: Clone + 'static>(n: SifrInt, data: &Vec<T>) -> Vec<T> {
    if &n <= &SifrInt::from_i64(0) {
        return vec![];
    }
    if (&n >= &SifrInt::from(data.len())) {
        let mut result: Vec<T> = vec![];
        for val in data.iter().cloned() {
            result.push(val.clone());
        }
        return result;
    }
    let mut heap: Vec<T> = data.clone();
    heapify(&mut heap);
    let mut all_sorted: Vec<T> = vec![];
    while (&SifrInt::from(heap.len()) > &SifrInt::from_i64(0)) {
        let val2: Option<T> = heappop(&mut heap);
        if let Some(val2) = val2 {
            all_sorted.push(val2.clone());
        }
    }
    let mut result2: Vec<T> = vec![];
    let mut i: SifrInt = &SifrInt::from(all_sorted.len()) - &SifrInt::from_i64(1);
    let mut count: SifrInt = SifrInt::from_i64(0);
    while (&count < &n) {
        if (&i < &SifrInt::from_i64(0)) {
            return result2;
        }
        let v: Option<T> = {
            let __sifr_checked_read_collection = &all_sorted;
            let __sifr_checked_read_index = i.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(v) = v {
            result2.push(v.clone());
        }
        i = &i - &SifrInt::from_i64(1);
        count = &count + &SifrInt::from_i64(1);
    }
    result2
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
    let mut heap: Vec<SifrInt> = vec![];
    heappush(&mut heap, &SifrInt::from_i64(5));
    heappush(&mut heap, &SifrInt::from_i64(1));
    heappush(&mut heap, &SifrInt::from_i64(3));
    let first: Option<SifrInt> = heappop(&mut heap);
    let second: Option<SifrInt> = heappop(&mut heap);
    actual.push(first.is_some() && (first == Some((SifrInt::from_i64(1)).clone())));
    actual.push(second.is_some() && (second == Some((SifrInt::from_i64(3)).clone())));
    let mut data: Vec<SifrInt> = vec![SifrInt::from_i64(4), SifrInt::from_i64(2), SifrInt::from_i64(7), SifrInt::from_i64(1), SifrInt::from_i64(5)];
    heapify(&mut data);
    let top: Option<SifrInt> = heappop(&mut data);
    actual.push(top.is_some() && (top == Some((SifrInt::from_i64(1)).clone())));
    let items: Vec<SifrInt> = vec![SifrInt::from_i64(9), SifrInt::from_i64(3), SifrInt::from_i64(7), SifrInt::from_i64(1), SifrInt::from_i64(5)];
    actual.push((format!("{:?}", nsmallest(SifrInt::from_i64(3), &items))).as_str() == ("[1, 3, 5]".to_string()).as_str());
    actual.push((format!("{:?}", nlargest(SifrInt::from_i64(2), &items))).as_str() == ("[9, 7]".to_string()).as_str());
    let mut empty_heap: Vec<SifrInt> = vec![];
    actual.push(heappop(&mut empty_heap) == None);
    actual.push((format!("{:?}", items)).as_str() == ("[9, 3, 7, 1, 5]".to_string()).as_str());
    actual
}

fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true, true];
    let actual: Vec<bool> = collect_actual();
    assert_bool_vector_eq(&actual, &expected);
    println!("heapq heapq parity demo: pass");
}
