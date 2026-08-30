// src/main.rs
use ::sifr_runtime::SifrInt;

// --- stdlib: sifr.heapq ---
fn _sift_down<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
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
                let __sifr_index_list = &data;
                let __sifr_index_i = smallest.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let l_val: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = left.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
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
        if &right < &n {
            let s_val2: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = smallest.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let r_val: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = right.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
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
        if &smallest == &pos {
            done = true;
        } else {
            let tmp_pos: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = pos.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let tmp_sm: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = smallest.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(tmp_pos) = tmp_pos {
                if let Some(tmp_sm) = tmp_sm {
                    {
                        let __idx_raw = pos.clone();
                        let __idx_norm = __idx_raw.normalize_index_or_len(data.len());
                        if let Some(__elem) = data.get_mut(__idx_norm) {
                            *__elem = tmp_sm.clone();
                        }
                    }
                    {
                        let __idx_raw = smallest.clone();
                        let __idx_norm = __idx_raw.normalize_index_or_len(data.len());
                        if let Some(__elem) = data.get_mut(__idx_norm) {
                            *__elem = tmp_pos.clone();
                        }
                    }
                }
            }
            pos = smallest;
        }
    }
}
fn _sift_up<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
    mut pos: SifrInt,
) {
    let mut done: bool = false;
    while !done {
        if &pos <= &SifrInt::from_i64(0) {
            done = true;
        } else {
            let parent: SifrInt = (&pos - &SifrInt::from_i64(1))
                .floor_div_known_nonzero(&SifrInt::from_i64(2));
            let p_val: Option<T> = {
                let __sifr_index_list = &heap;
                let __sifr_index_i = parent.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let c_val: Option<T> = {
                let __sifr_index_list = &heap;
                let __sifr_index_i = pos.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(p_val) = p_val {
                if let Some(c_val) = c_val {
                    if c_val < p_val {
                        {
                            let __idx_raw = parent.clone();
                            let __idx_norm = __idx_raw
                                .normalize_index_or_len(heap.len());
                            if let Some(__elem) = heap.get_mut(__idx_norm) {
                                *__elem = c_val.clone();
                            }
                        }
                        {
                            let __idx_raw = pos.clone();
                            let __idx_norm = __idx_raw
                                .normalize_index_or_len(heap.len());
                            if let Some(__elem) = heap.get_mut(__idx_norm) {
                                *__elem = p_val.clone();
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
fn heappush<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
    item: &T,
) {
    "Push item onto the heap in-place. O(log n) time.".to_string();
    heap.push(item.clone());
    let pos: SifrInt = &SifrInt::from(heap.len()) - &SifrInt::from_i64(1);
    _sift_up(heap, (pos).clone());
}
fn heappop<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
) -> Option<T> {
    "Pop and return the smallest item. Heap is modified in-place. O(log n) time.\n    Returns None if the heap is empty."
        .to_string();
    let n: SifrInt = SifrInt::from(heap.len());
    if &n == &SifrInt::from_i64(0) {
        return None;
    }
    let top: Option<T> = Some(
        heap[::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(0)))].clone(),
    );
    let last: Option<T> = {
        let __sifr_index_list = &heap;
        let __sifr_index_i = &n - &SifrInt::from_i64(1);
        let __sifr_index_norm = __sifr_index_i
            .normalize_index_or_len(__sifr_index_list.len());
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    heap.remove(heap.len() - (1_usize));
    let n2: SifrInt = SifrInt::from(heap.len());
    if &n2 > &SifrInt::from_i64(0) {
        if let Some(last) = last {
            {
                let __idx_raw = SifrInt::from_i64(0);
                let __idx_norm = __idx_raw.normalize_index_or_len(heap.len());
                if let Some(__elem) = heap.get_mut(__idx_norm) {
                    *__elem = last.clone();
                }
            }
        }
        _sift_down(heap, SifrInt::from_i64(0), (n2).clone());
    }
    top
}
// --- end stdlib ---

fn drain_sorted(values: &Vec<SifrInt>) -> Vec<SifrInt> {
    let mut heap: Vec<SifrInt> = vec![];
    let mut order: Vec<SifrInt> = vec![];
    for value in values.iter().cloned() {
        heappush(&mut heap, &value);
    }
    while !heap.is_empty() {
        let item: Option<SifrInt> = heappop(&mut heap);
        if let Some(item) = item.clone() {
            order.push(item.clone());
        }
    }
    order
}

fn main() {
    assert!((format!("{:?}", drain_sorted(&vec![SifrInt::from_i64(5), SifrInt::from_i64(1), SifrInt::from_i64(3)])) == "[1, 3, 5]"));
    let mut heap: Vec<SifrInt> = vec![];
    heappush(&mut heap, &SifrInt::from_i64(7));
    assert!((heappop(&mut heap) == Some(SifrInt::from_i64(7))));
    assert!((heappop(&mut heap) == None));
    println!("heap_option_drain: ok");
}
