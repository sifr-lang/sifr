// src/main.rs
mod sifr_generated_generated_support {
    pub(crate) use ::sifr_runtime::SifrInt;
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(crate) fn sifr_generated_sift_down<T: Clone + 'static + PartialOrd>(
        data: &mut Vec<T>,
        mut pos: SifrInt,
        n: SifrInt,
    ) {
        let mut done: bool = false;
        while !done {
            let mut smallest: SifrInt = pos.clone();
            let left: SifrInt = &(&SifrInt::from_i64(2) * &pos) + &SifrInt::from_i64(1);
            let right: SifrInt = &(&SifrInt::from_i64(2) * &pos) + &SifrInt::from_i64(2);
            if &left < &n {
                let s_val: Option<T> = {
                    let sifr_generated_checked_read_collection = &data;
                    let sifr_generated_checked_read_index = smallest.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let l_val_value_c583c4339eb822b3: Option<T> = {
                    let sifr_generated_checked_read_collection = &data;
                    let sifr_generated_checked_read_index = left.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(s_val) = s_val
                    && let Some(l_val) = l_val_value_c583c4339eb822b3
                    && l_val < s_val
                {
                    smallest = left;
                }
            }
            if &right < &n {
                let s_val2_value_8b32ab056d206424: Option<T> = {
                    let sifr_generated_checked_read_collection = &data;
                    let sifr_generated_checked_read_index = smallest.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let r_val_value_839f97b21b19be35: Option<T> = {
                    let sifr_generated_checked_read_collection = &data;
                    let sifr_generated_checked_read_index = right.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(s_val2) = s_val2_value_8b32ab056d206424
                    && let Some(r_val) = r_val_value_839f97b21b19be35
                    && r_val < s_val2
                {
                    smallest = right;
                }
            }
            if &smallest == &pos {
                done = true;
            } else {
                let tmp_pos: Option<T> = {
                    let sifr_generated_checked_read_collection = &data;
                    let sifr_generated_checked_read_index = pos.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let tmp_sm_value_cf4d6d82a6cdd887: Option<T> = {
                    let sifr_generated_checked_read_collection = &data;
                    let sifr_generated_checked_read_index = smallest.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(tmp_pos) = tmp_pos
                    && let Some(tmp_sm) = tmp_sm_value_cf4d6d82a6cdd887
                {
                    if &SifrInt::from_i64(0) <= &pos && &pos < &SifrInt::from(data.len()) {
                        {
                            let sifr_generated_assign_value = tmp_sm.clone();
                            {
                                let sifr_generated_index_raw = pos.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(data.len());
                                if let Some(sifr_generated_elem) =
                                    data.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                }
                            }
                        }
                    }
                    if &SifrInt::from_i64(0) <= &smallest && &smallest < &SifrInt::from(data.len())
                    {
                        {
                            let sifr_generated_assign_value = tmp_pos.clone();
                            {
                                let sifr_generated_index_raw = smallest.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(data.len());
                                if let Some(sifr_generated_elem) =
                                    data.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                }
                            }
                        }
                    }
                }
                pos = smallest;
            }
        }
    }
    pub(crate) fn sifr_generated_sift_up<T: Clone + 'static + PartialOrd>(
        heap: &mut Vec<T>,
        mut pos: SifrInt,
    ) {
        let mut done: bool = false;
        while !done {
            if &pos <= &SifrInt::from_i64(0) {
                done = true;
            } else {
                let parent: SifrInt =
                    (&pos - &SifrInt::from_i64(1)).floor_div_known_nonzero(&SifrInt::from_i64(2));
                let p_val: Option<T> = {
                    let sifr_generated_checked_read_collection = &heap;
                    let sifr_generated_checked_read_index = parent.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let c_val_value_6b01c611cd56bc8e: Option<T> = {
                    let sifr_generated_checked_read_collection = &heap;
                    let sifr_generated_checked_read_index = pos.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(p_val) = p_val {
                    if let Some(c_val) = c_val_value_6b01c611cd56bc8e {
                        if c_val < p_val {
                            if &SifrInt::from_i64(0) <= &parent
                                && &parent < &SifrInt::from(heap.len())
                            {
                                {
                                    let sifr_generated_assign_value = c_val.clone();
                                    {
                                        let sifr_generated_index_raw = parent.clone();
                                        let sifr_generated_index_normalized =
                                            sifr_generated_index_raw
                                                .normalize_index_or_len(heap.len());
                                        if let Some(sifr_generated_elem) =
                                            heap.get_mut(sifr_generated_index_normalized)
                                        {
                                            *sifr_generated_elem = sifr_generated_assign_value;
                                        }
                                    }
                                }
                            }
                            if &SifrInt::from_i64(0) <= &pos && &pos < &SifrInt::from(heap.len()) {
                                {
                                    let sifr_generated_assign_value = p_val.clone();
                                    {
                                        let sifr_generated_index_raw = pos.clone();
                                        let sifr_generated_index_normalized =
                                            sifr_generated_index_raw
                                                .normalize_index_or_len(heap.len());
                                        if let Some(sifr_generated_elem) =
                                            heap.get_mut(sifr_generated_index_normalized)
                                        {
                                            *sifr_generated_elem = sifr_generated_assign_value;
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
    pub(crate) fn heapify<T: Clone + 'static + PartialOrd>(data: &mut Vec<T>) {
        "Convert list to a min-heap in-place. O(n) time.".to_string();
        let n: SifrInt = SifrInt::from(data.len());
        let mut i: SifrInt =
            &n.floor_div_known_nonzero(&SifrInt::from_i64(2)) - &SifrInt::from_i64(1);
        while &i >= &SifrInt::from_i64(0) {
            sifr_generated_sift_down(data, i.clone(), n.clone());
            i = &i - &SifrInt::from_i64(1);
        }
    }
    pub(crate) fn heappush<T: Clone + 'static + PartialOrd>(heap: &mut Vec<T>, item: &T) {
        "Push item onto the heap in-place. O(log n) time.".to_string();
        heap.push(item.clone());
        let pos: SifrInt = &SifrInt::from(heap.len()) - &SifrInt::from_i64(1);
        sifr_generated_sift_up(heap, pos.clone());
    }
    pub(crate) fn heappop<T: Clone + 'static + PartialOrd>(heap: &mut Vec<T>) -> Option<T> {
        "Pop and return the smallest item. Heap is modified in-place. O(log n) time.\n    Returns None if the heap is empty."
            .to_string();
        let n: SifrInt = SifrInt::from(heap.len());
        if &n == &SifrInt::from_i64(0) {
            return None;
        }
        let top: Option<T> = {
            let sifr_generated_checked_read_collection = &heap;
            let sifr_generated_checked_read_index = SifrInt::from_i64(0);
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        let last: Option<T> = {
            let sifr_generated_checked_read_collection = &heap;
            let sifr_generated_checked_read_index = &n - &SifrInt::from_i64(1);
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        heap.remove(heap.len() - 1_usize);
        let n2: SifrInt = SifrInt::from(heap.len());
        if &n2 > &SifrInt::from_i64(0) {
            if let Some(last) = last {
                {
                    let sifr_generated_assign_value = last.clone();
                    {
                        let sifr_generated_index_raw = SifrInt::from_i64(0);
                        let sifr_generated_index_normalized =
                            sifr_generated_index_raw.normalize_index_or_len(heap.len());
                        if let Some(sifr_generated_elem) =
                            heap.get_mut(sifr_generated_index_normalized)
                        {
                            *sifr_generated_elem = sifr_generated_assign_value;
                        }
                    }
                }
            }
            sifr_generated_sift_down(heap, SifrInt::from_i64(0), n2.clone());
        }
        top
    }
    pub(crate) fn nsmallest<T: Clone + 'static + PartialOrd>(n: SifrInt, data: &[T]) -> Vec<T> {
        let mut heap: Vec<T> = data.to_vec();
        heapify(&mut heap);
        let mut result: Vec<T> = Vec::new();
        let mut count: SifrInt = SifrInt::from_i64(0);
        while &count < &n {
            if &SifrInt::from(heap.len()) == &SifrInt::from_i64(0) {
                return result;
            }
            let val: Option<T> = heappop(&mut heap);
            if let Some(val) = val {
                result.push(val);
            }
            count = &count + &SifrInt::from_i64(1);
        }
        result
    }
    pub(crate) fn nlargest<T: Clone + 'static + PartialOrd>(n: SifrInt, data: &[T]) -> Vec<T> {
        if &n <= &SifrInt::from_i64(0) {
            return Vec::new();
        }
        if &n >= &SifrInt::from(data.len()) {
            let mut result: Vec<T> = Vec::new();
            for val in data.iter().cloned() {
                result.push(val);
            }
            return result;
        }
        let mut heap: Vec<T> = data.to_vec();
        heapify(&mut heap);
        let mut all_sorted: Vec<T> = Vec::new();
        while &SifrInt::from(heap.len()) > &SifrInt::from_i64(0) {
            let val2: Option<T> = heappop(&mut heap);
            if let Some(val2) = val2 {
                all_sorted.push(val2);
            }
        }
        let mut result2: Vec<T> = Vec::new();
        let mut i: SifrInt = &SifrInt::from(all_sorted.len()) - &SifrInt::from_i64(1);
        let mut count: SifrInt = SifrInt::from_i64(0);
        while &count < &n {
            if &i < &SifrInt::from_i64(0) {
                return result2;
            }
            let v: Option<T> = {
                let sifr_generated_checked_read_collection = &all_sorted;
                let sifr_generated_checked_read_index = i.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(v) = v {
                result2.push(v);
            }
            i = &i - &SifrInt::from_i64(1);
            count = &count + &SifrInt::from_i64(1);
        }
        result2
    }
    pub(crate) fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
        assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(actual.len()) {
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
            i = &i + &SifrInt::from_i64(1);
        }
    }
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
fn collect_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let mut heap: Vec<SifrInt> = Vec::new();
    heappush(&mut heap, &SifrInt::from_i64(5));
    heappush(&mut heap, &SifrInt::from_i64(1));
    heappush(&mut heap, &SifrInt::from_i64(3));
    let first: Option<SifrInt> = heappop(&mut heap);
    let second: Option<SifrInt> = heappop(&mut heap);
    actual.push(first.is_some() && first == Some(SifrInt::from_i64(1)));
    actual.push(second.is_some() && second == Some(SifrInt::from_i64(3)));
    let mut data: Vec<SifrInt> = vec![
        SifrInt::from_i64(4),
        SifrInt::from_i64(2),
        SifrInt::from_i64(7),
        SifrInt::from_i64(1),
        SifrInt::from_i64(5),
    ];
    heapify(&mut data);
    let top: Option<SifrInt> = heappop(&mut data);
    actual.push(top.is_some() && top == Some(SifrInt::from_i64(1)));
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(9),
        SifrInt::from_i64(3),
        SifrInt::from_i64(7),
        SifrInt::from_i64(1),
        SifrInt::from_i64(5),
    ];
    actual.push(
        format!("{:?}", nsmallest(SifrInt::from_i64(3), &items)).as_str()
            == "[1, 3, 5]".to_string().as_str(),
    );
    actual.push(
        format!("{:?}", nlargest(SifrInt::from_i64(2), &items)).as_str()
            == "[9, 7]".to_string().as_str(),
    );
    let mut empty_heap: Vec<SifrInt> = Vec::new();
    actual.push(heappop(&mut empty_heap).is_none());
    actual.push(format!("{items:?}").as_str() == "[9, 3, 7, 1, 5]".to_string().as_str());
    actual
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true, true];
    let actual: Vec<bool> = collect_actual();
    assert_bool_vector_eq(&actual, &expected);
    println!("heapq heapq parity demo: pass");
}
