// src/main.rs
mod sifr_generated_generated_support {
    use crate::SifrGeneratedStdlibSifrX2ecollectionsX2eCounter;
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) use ::std::collections::HashMap;
    pub(crate) fn bisect_right<T: Clone + 'static + PartialOrd>(
        a: &[T],
        x: &T,
        lo: SifrInt,
        hi: Option<SifrInt>,
    ) -> SifrInt {
        let mut left: SifrInt = lo.clone();
        if &left < &SifrInt::from_i64(0) {
            left = SifrInt::from_i64(0);
        }
        let mut right: SifrInt = SifrInt::from(a.len());
        if hi.is_none() {
            right = SifrInt::from(a.len());
        } else if let Some(hi) = hi.clone() {
            if &hi < &SifrInt::from_i64(0) {
                right = SifrInt::from_i64(0);
            } else if &hi > &SifrInt::from(a.len()) {
                right = SifrInt::from(a.len());
            } else {
                right = hi;
            }
        }
        while &left < &right {
            let mid: SifrInt = (&left + &right).floor_div_known_nonzero(&SifrInt::from_i64(2));
            let val: Option<T> = {
                let sifr_generated_checked_read_collection = &a;
                let sifr_generated_checked_read_index = mid.clone();
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
                    left = &mid + &SifrInt::from_i64(1);
                }
            } else {
                left = &mid + &SifrInt::from_i64(1);
            }
        }
        left.clone()
    }
    pub(crate) fn insort_right<T: Clone + 'static + PartialOrd>(
        a: &mut Vec<T>,
        x: &T,
        lo: SifrInt,
        hi: Option<SifrInt>,
    ) {
        let pos: SifrInt = bisect_right(a, x, lo.clone(), hi.clone());
        a.insert(pos.clamp_slice_bound(a.len()), x.clone());
    }
    pub(crate) fn from_list<T: Clone + ::std::hash::Hash + Eq + 'static>(
        items: &[T],
    ) -> SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T> {
        let mut counts: HashMap<T, SifrInt> = HashMap::from([]);
        for item in items.iter().cloned() {
            let val: Option<SifrInt> = counts.get(&item).cloned();
            if let Some(val) = val.clone() {
                {
                    let sifr_generated_assign_value = &val + &SifrInt::from_i64(1);
                    {
                        let sifr_generated_assign_key = item.clone();
                        counts.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                    }
                }
            } else {
                let sifr_generated_assign_value = SifrInt::from_i64(1);
                {
                    let sifr_generated_assign_key = item.clone();
                    counts.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                }
            }
        }
        SifrGeneratedStdlibSifrX2ecollectionsX2eCounter::new(Some(counts), None)
    }
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
    pub(crate) fn heapreplace<T: Clone + 'static + PartialOrd>(
        heap: &mut Vec<T>,
        item: T,
    ) -> Option<T> {
        if &SifrInt::from(heap.len()) == &SifrInt::from_i64(0) {
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
            let sifr_generated_assign_value = item.clone();
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
        sifr_generated_sift_down(heap, SifrInt::from_i64(0), heap_len.clone());
        top
    }
    pub(crate) fn heappushpop<T: Clone + 'static + PartialOrd>(
        heap: &mut Vec<T>,
        item: &T,
    ) -> Option<T> {
        heappush(heap, item);
        heappop(heap)
    }
}
mod sifr_generated_project_nominals {
    use ::sifr_runtime::SifrInt;
    use ::std::collections::HashMap;
    use ::std::collections::VecDeque;
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T: std::hash::Hash + Eq> {
        pub counts: HashMap<T, SifrInt>,
    }
    impl<T: ::std::hash::Hash + Eq + Clone> SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T> {
        #[must_use]
        pub fn new(source: Option<HashMap<T, SifrInt>>, iterable: Option<Vec<T>>) -> Self {
            let mut counts: HashMap<T, SifrInt> = HashMap::from([]);
            if let Some(source) = source {
                for key in source.keys().cloned().collect::<Vec<_>>() {
                    let value: Option<SifrInt> = source.get(&key).cloned();
                    if let Some(value) = value.clone() {
                        {
                            let sifr_generated_assign_value = value.clone();
                            {
                                let sifr_generated_assign_key = key.clone();
                                counts
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    }
                }
            }
            if let Some(iterable) = iterable {
                for item in iterable.iter().cloned() {
                    let value2_value_2127bacf1a4dd308: Option<SifrInt> = counts.get(&item).cloned();
                    if let Some(value2) = value2_value_2127bacf1a4dd308.clone() {
                        {
                            let sifr_generated_assign_value = &value2 + &SifrInt::from_i64(1);
                            {
                                let sifr_generated_assign_key = item.clone();
                                counts
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    } else {
                        let sifr_generated_assign_value = SifrInt::from_i64(1);
                        {
                            let sifr_generated_assign_key = item.clone();
                            counts.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                        }
                    }
                }
            }
            let sifr_generated_field_value_c341febe5aae51e5_636f756e7473: HashMap<T, SifrInt> =
                counts;
            Self {
                counts: sifr_generated_field_value_c341febe5aae51e5_636f756e7473,
            }
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T> {
        #[must_use]
        pub fn get(&self, key: &T, default: &SifrInt) -> SifrInt {
            let val: Option<SifrInt> = self.counts.get(key).cloned();
            let Some(val) = val.clone() else {
                return default.clone();
            };
            val
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T> {
        #[must_use]
        pub fn most_common(&self, n: &Option<SifrInt>) -> Vec<(T, SifrInt)> {
            let mut result: Vec<(T, SifrInt)> = Vec::new();
            for key in self.counts.keys().cloned().collect::<Vec<_>>() {
                let count: Option<SifrInt> = self.counts.get(&key).cloned();
                if let Some(count) = count.clone() {
                    let entry: (T, SifrInt) = (key.clone(), count.clone());
                    result.push(entry.clone());
                }
            }
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &SifrInt::from_i64(0) <= &i && &i < &SifrInt::from(result.len()) {
                let mut j: SifrInt = &i + &SifrInt::from_i64(1);
                while &SifrInt::from_i64(0) <= &j && &j < &SifrInt::from(result.len()) {
                    let left: Option<(T, SifrInt)> = {
                        let sifr_generated_checked_read_collection = &result;
                        let sifr_generated_checked_read_index = i.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let right: Option<(T, SifrInt)> = {
                        let sifr_generated_checked_read_collection = &result;
                        let sifr_generated_checked_read_index = j.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(left) = left
                        && let Some(right) = right
                        && right.1.clone() > left.1.clone()
                    {
                        {
                            let sifr_generated_assign_value = right.clone();
                            {
                                let sifr_generated_index_raw = i.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(result.len());
                                if let Some(sifr_generated_elem) =
                                    result.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                }
                            }
                        }
                        {
                            let sifr_generated_assign_value = left.clone();
                            {
                                let sifr_generated_index_raw = j.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(result.len());
                                if let Some(sifr_generated_elem) =
                                    result.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                }
                            }
                        }
                    }
                    j = &j + &SifrInt::from_i64(1);
                }
                i = &i + &SifrInt::from_i64(1);
            }
            let Some(n) = n.as_ref() else {
                return result;
            };
            if n <= &SifrInt::from_i64(0) {
                return Vec::new();
            }
            let mut top: Vec<(T, SifrInt)> = Vec::new();
            let mut index: SifrInt = SifrInt::from_i64(0);
            while index < *n {
                if &index >= &SifrInt::from(result.len()) {
                    return top;
                }
                let value: Option<(T, SifrInt)> = {
                    let sifr_generated_checked_read_collection = &result;
                    let sifr_generated_checked_read_index = index.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(value) = value {
                    top.push(value.clone());
                }
                index = &index + &SifrInt::from_i64(1);
            }
            top
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone + PartialOrd>
        SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>
    {
        #[must_use]
        pub fn keys(&self) -> Vec<T> {
            let mut result: Vec<T> = self.counts.keys().cloned().collect::<Vec<_>>();
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &SifrInt::from_i64(0) <= &i && &i < &SifrInt::from(result.len()) {
                let mut j: SifrInt = &i + &SifrInt::from_i64(1);
                while &SifrInt::from_i64(0) <= &j && &j < &SifrInt::from(result.len()) {
                    let left: Option<T> = {
                        let sifr_generated_checked_read_collection = &result;
                        let sifr_generated_checked_read_index = i.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let right: Option<T> = {
                        let sifr_generated_checked_read_collection = &result;
                        let sifr_generated_checked_read_index = j.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(left) = left
                        && let Some(right) = right
                        && right < left
                    {
                        {
                            let sifr_generated_assign_value = right.clone();
                            {
                                let sifr_generated_index_raw = i.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(result.len());
                                if let Some(sifr_generated_elem) =
                                    result.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                }
                            }
                        }
                        {
                            let sifr_generated_assign_value = left.clone();
                            {
                                let sifr_generated_index_raw = j.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(result.len());
                                if let Some(sifr_generated_elem) =
                                    result.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                }
                            }
                        }
                    }
                    j = &j + &SifrInt::from_i64(1);
                }
                i = &i + &SifrInt::from_i64(1);
            }
            result
        }
    }
    impl<T: ::std::hash::Hash + Eq> SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T> {
        pub fn clear(&mut self) {
            self.counts = HashMap::new();
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone>
        ::std::ops::Add<&SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>>
        for &SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>
    {
        type Output = SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>;
        fn add(self, other: &SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>) -> Self::Output {
            let mut new_counts: HashMap<T, SifrInt> = HashMap::new();
            for key in Box::new(self.counts.keys().cloned().collect::<Vec<_>>().into_iter()) {
                let a_val: Option<SifrInt> = self.counts.get(&key).cloned();
                if let Some(a_val) = a_val {
                    let b_val_value_f4118a41fdffa885: Option<SifrInt> =
                        other.counts.get(&key).cloned();
                    let b_count: SifrInt = b_val_value_f4118a41fdffa885
                        .clone()
                        .unwrap_or_else(|| SifrInt::from_i64(0));
                    let total: SifrInt = &a_val + &b_count;
                    if &total > &SifrInt::from_i64(0) {
                        {
                            let sifr_generated_assign_value = total.clone();
                            {
                                let sifr_generated_assign_key = key.clone();
                                new_counts
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    }
                }
            }
            for key2 in Box::new(other.counts.keys().cloned().collect::<Vec<_>>().into_iter()) {
                let already: Option<SifrInt> = new_counts.get(&key2).cloned();
                if already.is_none() {
                    let b_val2: Option<SifrInt> = other.counts.get(&key2).cloned();
                    if let Some(b_val2) = b_val2
                        && &b_val2 > &SifrInt::from_i64(0)
                    {
                        {
                            let sifr_generated_assign_value = b_val2.clone();
                            {
                                let sifr_generated_assign_key = key2.clone();
                                new_counts
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    }
                }
            }
            SifrGeneratedStdlibSifrX2ecollectionsX2eCounter::new(Some(new_counts), None)
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone>
        ::std::ops::Sub<&SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>>
        for &SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>
    {
        type Output = SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>;
        fn sub(self, other: &SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>) -> Self::Output {
            let mut new_counts: HashMap<T, SifrInt> = HashMap::new();
            for key in Box::new(self.counts.keys().cloned().collect::<Vec<_>>().into_iter()) {
                let a_val: Option<SifrInt> = self.counts.get(&key).cloned();
                if let Some(a_val) = a_val {
                    let b_val_value_f4118a41fdffa885: Option<SifrInt> =
                        other.counts.get(&key).cloned();
                    let b_count: SifrInt = b_val_value_f4118a41fdffa885
                        .clone()
                        .unwrap_or_else(|| SifrInt::from_i64(0));
                    let diff: SifrInt = &a_val - &b_count;
                    if &diff > &SifrInt::from_i64(0) {
                        {
                            let sifr_generated_assign_value = diff.clone();
                            {
                                let sifr_generated_assign_key = key.clone();
                                new_counts
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    }
                }
            }
            SifrGeneratedStdlibSifrX2ecollectionsX2eCounter::new(Some(new_counts), None)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        pub data: VecDeque<T>,
        pub maxlen: Option<SifrInt>,
    }
    impl<T: Clone> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        #[must_use]
        pub fn new(items: Option<Vec<T>>, maxlen: Option<SifrInt>) -> Self {
            let mut data: Vec<T> = Vec::new();
            if let Some(items) = items {
                let start: SifrInt = if let Some(maxlen) = maxlen.clone()
                    && &SifrInt::from(items.len()) > &maxlen
                {
                    &SifrInt::from(items.len()) - &maxlen
                } else {
                    SifrInt::from_i64(0)
                };
                let mut i: SifrInt = start.clone();
                while &i < &SifrInt::from(items.len()) {
                    let item_value_2841a0c596d6f426: Option<T> = {
                        let sifr_generated_checked_read_collection = &items;
                        let sifr_generated_checked_read_index = i.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(item) = item_value_2841a0c596d6f426 {
                        data.push(item.clone());
                    }
                    i = &i + &SifrInt::from_i64(1);
                }
            }
            let sifr_generated_field_value_169953f6befb0270_6d61786c656e: Option<SifrInt> =
                maxlen.clone();
            let sifr_generated_field_value_90770dc80a1c57ce_5f64617461: VecDeque<T> =
                VecDeque::from(data);
            Self {
                maxlen: sifr_generated_field_value_169953f6befb0270_6d61786c656e,
                data: sifr_generated_field_value_90770dc80a1c57ce_5f64617461,
            }
        }
    }
    impl<T: Clone> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        pub fn appendleft(&mut self, val: &T) {
            self.data.push_front(val.clone());
            let maxlen_opt: Option<SifrInt> = self.maxlen.clone();
            if let Some(maxlen_opt) = maxlen_opt.clone() {
                let maxlen: SifrInt = maxlen_opt.clone();
                if &SifrInt::from(self.data.len()) > &maxlen {
                    self.data.pop_back();
                }
            }
        }
    }
    impl<T> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        #[must_use]
        pub fn len(&self) -> SifrInt {
            SifrInt::from(self.data.len())
        }
    }
    impl<T: Clone> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        #[must_use]
        pub fn to_list(&self) -> Vec<T> {
            let mut result: Vec<T> = Vec::new();
            for v in self.data.iter().cloned() {
                result.push(v.clone());
            }
            result
        }
    }
    impl<T> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        pub fn clear(&mut self) {
            self.data.clear();
        }
    }
    impl<T: Clone> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        pub fn rotate(&mut self, n: &SifrInt) {
            let length: SifrInt = SifrInt::from(self.data.len());
            if &length == &SifrInt::from_i64(0) {
                return;
            }
            let mut steps: SifrInt = n.floor_mod_known_nonzero(&length);
            if &steps < &SifrInt::from_i64(0) {
                steps = &steps + &length;
            }
            let mut count: SifrInt = SifrInt::from_i64(0);
            while &count < &steps {
                let value: Option<T> = self.data.pop_back();
                if let Some(value) = value {
                    self.data.push_front(value.clone());
                }
                count = &count + &SifrInt::from_i64(1);
            }
        }
    }
    impl<T: Clone + PartialEq> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        #[must_use]
        pub fn index(&self, value: &T, start: &SifrInt, stop: &Option<SifrInt>) -> Option<SifrInt> {
            let size: SifrInt = SifrInt::from(self.data.len());
            let mut begin: SifrInt = start.clone();
            if &begin < &SifrInt::from_i64(0) {
                begin = &size + &begin;
                if &begin < &SifrInt::from_i64(0) {
                    begin = SifrInt::from_i64(0);
                }
            }
            let mut end: SifrInt = size.clone();
            if let Some(stop) = stop.as_ref() {
                end = stop.clone();
                if &end < &SifrInt::from_i64(0) {
                    end = &size + &end;
                }
                if &end < &SifrInt::from_i64(0) {
                    end = SifrInt::from_i64(0);
                }
                if &end > &size {
                    end = size.clone();
                }
            }
            let mut i: SifrInt = begin.clone();
            while &i < &end {
                let current: Option<T> = {
                    let sifr_generated_checked_read_collection = &self.data;
                    let sifr_generated_checked_read_index = i.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(current) = current
                    && current == *value
                {
                    return Some(i);
                }
                i = &i + &SifrInt::from_i64(1);
            }
            None
        }
    }
    impl<T: Clone + PartialEq> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        pub fn remove(&mut self, value: &T) {
            let idx: Option<SifrInt> = self.index(value, &SifrInt::from_i64(0), &None);
            if let Some(idx) = idx.clone() {
                let mut rebuilt: Vec<T> = Vec::new();
                let mut i: SifrInt = SifrInt::from_i64(0);
                while &i < &SifrInt::from(self.data.len()) {
                    let current: Option<T> = {
                        let sifr_generated_checked_read_collection = &self.data;
                        let sifr_generated_checked_read_index = i.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(current) = current
                        && &i != &idx
                    {
                        rebuilt.push(current.clone());
                    }
                    i = &i + &SifrInt::from_i64(1);
                }
                self.data.clear();
                for item in rebuilt.iter().cloned() {
                    self.data.push_back(item.clone());
                }
            }
        }
    }
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecollectionsX2eCounter;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecollectionsX2edeque;
fn main() {
    let counts: SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<String> = from_list(&vec![
        "delta".to_string(),
        "alpha".to_string(),
        "delta".to_string(),
        "beta".to_string(),
    ]);
    println!("{:?}", counts.most_common(&None));
    let mut queue: SifrGeneratedStdlibSifrX2ecollectionsX2edeque<SifrInt> =
        SifrGeneratedStdlibSifrX2ecollectionsX2edeque::new(
            Some(vec![
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(3),
            ]),
            Some(SifrInt::from_i64(4)),
        );
    (&mut queue).rotate(&SifrInt::from_i64(1));
    (&mut queue).appendleft(&SifrInt::from_i64(0));
    println!("{:?}", queue.to_list());
    let mut ordered: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(3),
        SifrInt::from_i64(5),
    ];
    insort_right(
        &mut ordered,
        &SifrInt::from_i64(4),
        SifrInt::from_i64(0),
        None,
    );
    println!(
        "{}",
        bisect_right(&ordered, &SifrInt::from_i64(4), SifrInt::from_i64(0), None)
    );
    let mut heap: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(3),
        SifrInt::from_i64(5),
    ];
    heapify(&mut heap);
    println!(
        "{}",
        heappushpop(&mut heap, &SifrInt::from_i64(2)).map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    println!(
        "{}",
        heapreplace(&mut heap, SifrInt::from_i64(4)).map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
}
