// src/main.rs
// --- stdlib: sifr.heapq ---
fn _sift_down_max<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &mut Vec<T>,
    mut pos: i64,
    n: i64,
) {
    let mut done: bool = false;
    while !done {
        let mut largest: i64 = pos;
        let left: i64 = ((2_i64) * pos) + (1_i64);
        let right: i64 = ((2_i64) * pos) + (2_i64);
        if left < n {
            let current_val: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = largest;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let left_val: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = left;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(current_val) = current_val {
                if let Some(left_val) = left_val {
                    if left_val > current_val {
                        largest = left;
                    }
                }
            }
        }
        if right < n {
            let current_val2: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = largest;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let right_val: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = right;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(current_val2) = current_val2 {
                if let Some(right_val) = right_val {
                    if right_val > current_val2 {
                        largest = right;
                    }
                }
            }
        }
        if largest == pos {
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
            let tmp_largest: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = largest;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(tmp_pos) = tmp_pos {
                if let Some(tmp_largest) = tmp_largest {
                    {
                        let __idx_raw = pos;
                        let __idx_norm = if __idx_raw < 0 {
                            (data.len() as i64) + __idx_raw
                        } else {
                            __idx_raw
                        };
                        if __idx_norm >= 0 {
                            if let Some(__elem) = data.get_mut(__idx_norm as usize) {
                                *__elem = tmp_largest.clone();
                            }
                        }
                    }
                    {
                        let __idx_raw = largest;
                        let __idx_norm = if __idx_raw < 0 {
                            (data.len() as i64) + __idx_raw
                        } else {
                            __idx_raw
                        };
                        if __idx_norm >= 0 {
                            if let Some(__elem) = data.get_mut(__idx_norm as usize) {
                                *__elem = tmp_pos.clone();
                            }
                        }
                    }
                }
            }
            pos = largest;
        }
    }
}
fn _heapify_max<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &mut Vec<T>,
) {
    "Convert list to a max-heap in-place. O(n) time.".to_string();
    let n: i64 = data.len() as i64;
    let mut i: i64 = (n / (2_i64)) - (1_i64);
    while i >= (0_i64) {
        _sift_down_max(data, i, n);
        i -= 1_i64;
    }
}
fn _heappop_max<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
) -> Option<T> {
    "Pop and return the largest item. Heap is modified in-place. O(log n) time.\n    Returns None if the heap is empty."
        .to_string();
    let n: i64 = heap.len() as i64;
    if n == (0_i64) {
        return None;
    }
    let top: Option<T> = Some(heap[(0_i64) as usize].clone());
    let last: Option<T> = {
        let __sifr_index_list = &heap;
        let __sifr_index_i = n - (1_i64);
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    {
        let Some(__sifr_nonempty_pop_value) = heap.pop() else {
            unreachable!("compiler-verified non-empty pop should return Some");
        };
        __sifr_nonempty_pop_value
    };
    let n2: i64 = heap.len() as i64;
    if n2 > (0_i64) {
        if let Some(last) = last {
            {
                let __idx_raw = 0_i64;
                let __idx_norm = if __idx_raw < 0 {
                    (heap.len() as i64) + __idx_raw
                } else {
                    __idx_raw
                };
                if __idx_norm >= 0 {
                    if let Some(__elem) = heap.get_mut(__idx_norm as usize) {
                        *__elem = last.clone();
                    }
                }
            }
        }
        _sift_down_max(heap, 0_i64, n2);
    }
    top
}
fn _heapreplace_max<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
    item: T,
) -> Option<T> {
    "Pop and return the largest item, then push item onto the heap.\n    Returns None if the heap is empty. O(log n) time."
        .to_string();
    if (heap.len() as i64) == (0_i64) {
        return None;
    }
    let top: Option<T> = Some(heap[(0_i64) as usize].clone());
    {
        let __idx_raw = 0_i64;
        let __idx_norm = if __idx_raw < 0 {
            (heap.len() as i64) + __idx_raw
        } else {
            __idx_raw
        };
        if __idx_norm >= 0 {
            if let Some(__elem) = heap.get_mut(__idx_norm as usize) {
                *__elem = item.clone();
            }
        }
    }
    let heap_len: i64 = heap.len() as i64;
    _sift_down_max(heap, 0_i64, heap_len);
    top
}
// --- end stdlib ---

fn drain(heap: &mut Vec<i64>) -> Vec<i64> {
    let mut result: Vec<i64> = vec![];
    while ((heap.len() as i64) > (0_i64)) {
        let value: Option<i64> = _heappop_max(heap);
        if let Some(value) = value {
            result.push(value);
        }
    }
    result
}

fn main() {
    let mut stones: Vec<i64> = vec![2_i64, 7_i64, 4_i64, 1_i64, 8_i64, 1_i64];
    _heapify_max(&mut stones);
    println!("{}", format!("{:?}", drain(&mut stones)));
    let mut probe: Vec<i64> = vec![4_i64, 10_i64, 7_i64];
    _heapify_max(&mut probe);
    _heapreplace_max(&mut probe, 6_i64);
    println!("{}", format!("{:?}", drain(&mut probe)));
}
