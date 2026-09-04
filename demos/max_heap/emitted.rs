// src/main.rs
pub mod sifr_generated_generated_support {
    pub(super) use ::sifr_runtime::SifrInt;
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn sifr_generated_sift_down_max<T: Clone + 'static + PartialOrd>(
        data: &mut Vec<T>,
        mut pos: SifrInt,
        n: SifrInt,
    ) {
        let mut done: bool = false;
        while !done {
            let mut largest: SifrInt = pos.clone();
            let left: SifrInt = ::std::ops::Add::add(
                &::std::ops::Mul::mul(&SifrInt::from_i64(2), &pos),
                &SifrInt::from_i64(1),
            );
            let right: SifrInt = ::std::ops::Add::add(
                &::std::ops::Mul::mul(&SifrInt::from_i64(2), &pos),
                &SifrInt::from_i64(2),
            );
            if left < n {
                let current_val: Option<T> = {
                    let sifr_generated_checked_read_collection = &data;
                    let sifr_generated_checked_read_index = &largest;
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let left_val: Option<T> = {
                    let sifr_generated_checked_read_collection = &data;
                    let sifr_generated_checked_read_index = &left;
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(current_val) = current_val
                    && let Some(left_val) = left_val
                    && left_val > current_val
                {
                    largest = left;
                }
            }
            if right < n {
                let current_val2_value_4fa82455325b79cc: Option<T> = {
                    let sifr_generated_checked_read_collection = &data;
                    let sifr_generated_checked_read_index = &largest;
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let right_val: Option<T> = {
                    let sifr_generated_checked_read_collection = &data;
                    let sifr_generated_checked_read_index = &right;
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(current_val2) = current_val2_value_4fa82455325b79cc
                    && let Some(right_val) = right_val
                    && right_val > current_val2
                {
                    largest = right;
                }
            }
            if largest == pos {
                done = true;
            } else {
                let tmp_pos: Option<T> = {
                    let sifr_generated_checked_read_collection = &data;
                    let sifr_generated_checked_read_index = &pos;
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let tmp_largest: Option<T> = {
                    let sifr_generated_checked_read_collection = &data;
                    let sifr_generated_checked_read_index = &largest;
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(tmp_pos) = tmp_pos
                    && let Some(tmp_largest) = tmp_largest
                {
                    if SifrInt::from_i64(0) <= pos && pos < data.len() {
                        {
                            let sifr_generated_assign_value = tmp_largest;
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
                    if SifrInt::from_i64(0) <= largest && largest < data.len() {
                        {
                            let sifr_generated_assign_value = tmp_pos;
                            {
                                let sifr_generated_index_raw = largest.clone();
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
                pos = largest;
            }
        }
    }
    pub(super) fn sifr_generated_heapify_max<T: Clone + 'static + PartialOrd>(data: &mut Vec<T>) {
        "Convert list to a max-heap in-place. O(n) time.".to_string();
        let n: SifrInt = SifrInt::from(data.len());
        let mut i: SifrInt = ::std::ops::Sub::sub(
            &n.floor_div_known_nonzero(&SifrInt::from_i64(2)),
            &SifrInt::from_i64(1),
        );
        while i >= SifrInt::from_i64(0) {
            sifr_generated_sift_down_max(data, i.clone(), n.clone());
            i = ::std::ops::Sub::sub(&i, &SifrInt::from_i64(1));
        }
    }
    pub(super) fn sifr_generated_heappop_max<T: Clone + 'static + PartialOrd>(
        heap: &mut Vec<T>,
    ) -> Option<T> {
        "Pop and return the largest item. Heap is modified in-place. O(log n) time.\n    Returns None if the heap is empty."
            .to_string();
        let n: SifrInt = SifrInt::from(heap.len());
        if n == SifrInt::from_i64(0) {
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
            let sifr_generated_checked_read_index = ::std::ops::Sub::sub(&n, &SifrInt::from_i64(1));
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        heap.remove(heap.len().saturating_sub(1_usize));
        let n2: SifrInt = SifrInt::from(heap.len());
        if n2 > SifrInt::from_i64(0) {
            if let Some(last) = last {
                {
                    let sifr_generated_assign_value = last;
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
            sifr_generated_sift_down_max(heap, SifrInt::from_i64(0), n2);
        }
        top
    }
    pub(super) fn sifr_generated_heapreplace_max<T: Clone + 'static + PartialOrd>(
        heap: &mut Vec<T>,
        item: T,
    ) -> Option<T> {
        "Pop and return the largest item, then push item onto the heap.\n    Returns None if the heap is empty. O(log n) time."
            .to_string();
        if heap.len() == SifrInt::from_i64(0) {
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
        {
            let sifr_generated_assign_value = item;
            {
                let sifr_generated_index_raw = SifrInt::from_i64(0);
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(heap.len());
                if let Some(sifr_generated_elem) = heap.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem = sifr_generated_assign_value;
                }
            }
        }
        let heap_len: SifrInt = SifrInt::from(heap.len());
        sifr_generated_sift_down_max(heap, SifrInt::from_i64(0), heap_len);
        top
    }
}
use crate::sifr_generated_generated_support::{
    sifr_generated_heapify_max, sifr_generated_heappop_max, sifr_generated_heapreplace_max,
};
use ::sifr_runtime::SifrInt;
fn drain(heap: &mut Vec<SifrInt>) -> Vec<SifrInt> {
    let mut result: Vec<SifrInt> = Vec::new();
    while heap.len() > SifrInt::from_i64(0) {
        let value: Option<SifrInt> = sifr_generated_heappop_max(heap);
        if let Some(value) = value {
            result.push(value);
        }
    }
    result
}
fn main() {
    let mut stones: Vec<SifrInt> = vec![
        SifrInt::from_i64(2),
        SifrInt::from_i64(7),
        SifrInt::from_i64(4),
        SifrInt::from_i64(1),
        SifrInt::from_i64(8),
        SifrInt::from_i64(1),
    ];
    sifr_generated_heapify_max(&mut stones);
    println!("{:?}", drain(&mut stones));
    let mut probe: Vec<SifrInt> = vec![
        SifrInt::from_i64(4),
        SifrInt::from_i64(10),
        SifrInt::from_i64(7),
    ];
    sifr_generated_heapify_max(&mut probe);
    sifr_generated_heapreplace_max(&mut probe, SifrInt::from_i64(6));
    println!("{:?}", drain(&mut probe));
}
