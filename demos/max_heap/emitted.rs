// src/main.rs
use ::sifr_runtime::SifrInt;

// --- stdlib: sifr.heapq ---
fn _sift_down_max<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &mut Vec<T>,
    mut pos: SifrInt,
    n: SifrInt,
) {
    let mut done: bool = false;
    while !done {
        let mut largest: SifrInt = pos.clone();
        let left: SifrInt = &(&SifrInt::from_i64(2) * &pos) + &SifrInt::from_i64(1);
        let right: SifrInt = &(&SifrInt::from_i64(2) * &pos) + &SifrInt::from_i64(2);
        if &left < &n {
            let current_val: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = largest.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let left_val: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = left.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
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
        if &right < &n {
            let current_val2: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = largest.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let right_val: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = right.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
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
        if &largest == &pos {
            done = true;
        } else {
            let tmp_pos: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = pos.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let tmp_largest: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = largest.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(tmp_pos) = tmp_pos {
                if let Some(tmp_largest) = tmp_largest {
                    {
                        let __idx_raw = pos.clone();
                        let __idx_norm = __idx_raw.normalize_index_or_len(data.len());
                        if let Some(__elem) = data.get_mut(__idx_norm) {
                            *__elem = tmp_largest.clone();
                        }
                    }
                    {
                        let __idx_raw = largest.clone();
                        let __idx_norm = __idx_raw.normalize_index_or_len(data.len());
                        if let Some(__elem) = data.get_mut(__idx_norm) {
                            *__elem = tmp_pos.clone();
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
    let n: SifrInt = SifrInt::from(data.len());
    let mut i: SifrInt = &n.floor_div_known_nonzero(&SifrInt::from_i64(2))
        - &SifrInt::from_i64(1);
    while &i >= &SifrInt::from_i64(0) {
        _sift_down_max(data, (i).clone(), (n).clone());
        i = &i - &SifrInt::from_i64(1);
    }
}
fn _heappop_max<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
) -> Option<T> {
    "Pop and return the largest item. Heap is modified in-place. O(log n) time.\n    Returns None if the heap is empty."
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
    {
        let Some(__sifr_nonempty_pop_value) = heap.pop() else {
            unreachable!("compiler-verified non-empty pop should return Some");
        };
        __sifr_nonempty_pop_value
    };
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
        _sift_down_max(heap, SifrInt::from_i64(0), (n2).clone());
    }
    top
}
fn _heapreplace_max<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
    item: T,
) -> Option<T> {
    "Pop and return the largest item, then push item onto the heap.\n    Returns None if the heap is empty. O(log n) time."
        .to_string();
    if &SifrInt::from(heap.len()) == &SifrInt::from_i64(0) {
        return None;
    }
    let top: Option<T> = Some(
        heap[::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(0)))].clone(),
    );
    {
        let __idx_raw = SifrInt::from_i64(0);
        let __idx_norm = __idx_raw.normalize_index_or_len(heap.len());
        if let Some(__elem) = heap.get_mut(__idx_norm) {
            *__elem = item.clone();
        }
    }
    let heap_len: SifrInt = SifrInt::from(heap.len());
    _sift_down_max(heap, SifrInt::from_i64(0), (heap_len).clone());
    top
}
// --- end stdlib ---

fn drain(heap: &mut Vec<SifrInt>) -> Vec<SifrInt> {
    let mut result: Vec<SifrInt> = vec![];
    while (&SifrInt::from(heap.len()) > &SifrInt::from_i64(0)) {
        let value: Option<SifrInt> = _heappop_max(heap);
        if let Some(value) = value.clone() {
            result.push(value.clone());
        }
    }
    result
}

fn main() {
    let mut stones: Vec<SifrInt> = vec![SifrInt::from_i64(2), SifrInt::from_i64(7), SifrInt::from_i64(4), SifrInt::from_i64(1), SifrInt::from_i64(8), SifrInt::from_i64(1)];
    _heapify_max(&mut stones);
    println!("{}", format!("{:?}", drain(&mut stones)));
    let mut probe: Vec<SifrInt> = vec![SifrInt::from_i64(4), SifrInt::from_i64(10), SifrInt::from_i64(7)];
    _heapify_max(&mut probe);
    _heapreplace_max(&mut probe, SifrInt::from_i64(6));
    println!("{}", format!("{:?}", drain(&mut probe)));
}
