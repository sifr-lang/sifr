// src/main.rs
mod sifr_generated_generated_support {
    use crate::SifrGeneratedStdlibSifrX2ecollectionsX2eCounter;
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) use ::std::collections::HashMap;
    pub(crate) fn bisect_left<T: Clone + 'static + PartialOrd>(
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
    pub(crate) fn insort_left<T: Clone + 'static + PartialOrd>(
        a: &mut Vec<T>,
        x: &T,
        lo: SifrInt,
        hi: Option<SifrInt>,
    ) {
        let pos: SifrInt = bisect_left(a, x, lo.clone(), hi.clone());
        a.insert(pos.clamp_slice_bound(a.len()), x.clone());
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
    pub(crate) struct SifrGeneratedYielder<T> {
        pub(crate) slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    }
    pub(crate) struct SifrGeneratedYieldFuture<T> {
        pub(crate) slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        pub(crate) value: Option<T>,
    }
    impl<T> Unpin for SifrGeneratedYieldFuture<T> {}
    impl<T> ::std::future::Future for SifrGeneratedYieldFuture<T> {
        type Output = ();
        fn poll(
            self: ::std::pin::Pin<&mut Self>,
            _: &mut ::std::task::Context<'_>,
        ) -> ::std::task::Poll<()> {
            let state = self.get_mut();
            let Some(value) = state.value.take() else {
                return ::std::task::Poll::Ready(());
            };
            sifr_generated_store_suspended(&state.slot, value);
            ::std::task::Poll::Pending
        }
    }
    impl<T> SifrGeneratedYielder<T> {
        pub(crate) fn suspend(&self, value: T) -> SifrGeneratedYieldFuture<T> {
            SifrGeneratedYieldFuture {
                slot: ::std::sync::Arc::clone(&self.slot),
                value: Some(value),
            }
        }
    }
    pub(crate) fn sifr_generated_store_suspended<T>(
        slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        value: T,
    ) {
        match slot.lock() {
            Ok(mut state) => *state = Some(value),
            Err(poisoned) => *poisoned.into_inner() = Some(value),
        }
    }
    pub(crate) fn sifr_generated_take_suspended<T>(
        slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    ) -> Option<T> {
        match slot.lock() {
            Ok(mut state) => state.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        }
    }
    pub(crate) struct SifrGeneratedGenerator<T> {
        pub(crate) producer:
            Option<::std::pin::Pin<Box<dyn ::std::future::Future<Output = ()> + 'static>>>,
        pub(crate) yielded: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        pub(crate) complete: bool,
    }
    impl<T> SifrGeneratedGenerator<T> {
        pub(crate) fn new<
            F: FnOnce(SifrGeneratedYielder<T>) -> Fut + 'static,
            Fut: ::std::future::Future<Output = ()> + 'static,
        >(
            factory: F,
        ) -> Self {
            let yielded = ::std::sync::Arc::new(::std::sync::Mutex::new(None));
            let producer = factory(SifrGeneratedYielder {
                slot: ::std::sync::Arc::clone(&yielded),
            });
            Self {
                producer: Some(Box::pin(producer)),
                yielded,
                complete: false,
            }
        }
    }
    impl<T> Iterator for SifrGeneratedGenerator<T> {
        type Item = T;
        fn next(&mut self) -> Option<T> {
            if self.complete {
                return None;
            }
            let completed = {
                let Some(producer) = self.producer.as_mut() else {
                    self.complete = true;
                    return None;
                };
                let mut context = ::std::task::Context::from_waker(::std::task::Waker::noop());
                ::std::future::Future::poll(producer.as_mut(), &mut context).is_ready()
            };
            let yielded = sifr_generated_take_suspended(&self.yielded);
            if completed {
                self.complete = true;
                self.producer = None;
            }
            yielded
        }
    }
    pub(crate) fn chain<T: Clone + 'static>(iterables: &[Vec<T>]) -> Box<dyn Iterator<Item = T>> {
        let iterables = iterables.to_vec();
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
                for iterable in iterables.iter().cloned() {
                    for item in iterable.iter().cloned() {
                        sifr_generated_yielder.suspend(item.clone()).await;
                    }
                }
            },
        ))
    }
}
mod sifr_generated_project_nominals {
    use ::sifr_runtime::SifrInt;
    use ::std::collections::HashMap;
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
        pub fn increment(&mut self, key: &T) {
            let val: Option<SifrInt> = self.counts.get(key).cloned();
            if let Some(val) = val.clone() {
                {
                    let sifr_generated_assign_value = &val + &SifrInt::from_i64(1);
                    {
                        let sifr_generated_assign_key = key.clone();
                        self.counts
                            .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                    }
                }
            } else {
                let sifr_generated_assign_value = SifrInt::from_i64(1);
                {
                    let sifr_generated_assign_key = key.clone();
                    self.counts
                        .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                }
            }
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T> {
        #[must_use]
        pub fn total(&self) -> SifrInt {
            let mut total: SifrInt = SifrInt::from_i64(0);
            for count in self.counts.values().cloned().collect::<Vec<_>>() {
                total = &total + &count;
            }
            total
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
    impl<T: ::std::hash::Hash + Eq + Clone> SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T> {
        #[must_use]
        pub fn values(&self) -> Vec<SifrInt> {
            self.counts.values().cloned().collect::<Vec<_>>()
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
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecollectionsX2eCounter;
fn demo_heapq() {
    println!("=== Section 1: heapq with mut params ===");
    let mut data: Vec<SifrInt> = vec![
        SifrInt::from_i64(5),
        SifrInt::from_i64(3),
        SifrInt::from_i64(8),
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(7),
        SifrInt::from_i64(4),
    ];
    heapify(&mut data);
    println!("heapified (min at root):");
    let min_val: Option<SifrInt> = {
        let sifr_generated_checked_read_collection = &data;
        let sifr_generated_checked_read_index = SifrInt::from_i64(0);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    };
    if let Some(min_val) = min_val.clone() {
        println!("{min_val}");
    }
    heappush(&mut data, &SifrInt::from_i64(0));
    println!("after push(0), new min:");
    let new_min: Option<SifrInt> = {
        let sifr_generated_checked_read_collection = &data;
        let sifr_generated_checked_read_index = SifrInt::from_i64(0);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    };
    if let Some(new_min) = new_min.clone() {
        println!("{new_min}");
    }
    let popped: Option<SifrInt> = heappop(&mut data);
    if let Some(popped) = popped.clone() {
        println!("popped:");
        println!("{popped}");
    }
    println!("remaining size:");
    println!("{}", SifrInt::from(data.len()));
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(9),
        SifrInt::from_i64(3),
        SifrInt::from_i64(7),
        SifrInt::from_i64(1),
        SifrInt::from_i64(5),
        SifrInt::from_i64(6),
        SifrInt::from_i64(2),
        SifrInt::from_i64(8),
        SifrInt::from_i64(4),
    ];
    let small3: Vec<SifrInt> = nsmallest(SifrInt::from_i64(3), &items);
    let large3: Vec<SifrInt> = nlargest(SifrInt::from_i64(3), &items);
    println!("3 smallest:");
    println!("{small3:?}");
    println!("3 largest:");
    println!("{large3:?}");
    println!("items still valid, length:");
    println!("{}", SifrInt::from(items.len()));
}
fn demo_bisect() {
    println!("=== Section 2: bisect_right insort_right with mut params ===");
    let mut sorted_ints: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(3),
        SifrInt::from_i64(5),
        SifrInt::from_i64(7),
        SifrInt::from_i64(9),
    ];
    let pos_left: SifrInt = bisect_left(
        &sorted_ints,
        &SifrInt::from_i64(6),
        SifrInt::from_i64(0),
        None,
    );
    let pos_right: SifrInt = bisect_right(
        &sorted_ints,
        &SifrInt::from_i64(5),
        SifrInt::from_i64(0),
        None,
    );
    println!("insert 6 at position (left):");
    println!("{pos_left}");
    println!("insert after 5 at position (right):");
    println!("{pos_right}");
    insort_left(
        &mut sorted_ints,
        &SifrInt::from_i64(6),
        SifrInt::from_i64(0),
        None,
    );
    println!("after insort_left(6):");
    println!("{sorted_ints:?}");
    let mut data: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
    ];
    insort_right(&mut data, &SifrInt::from_i64(2), SifrInt::from_i64(0), None);
    println!("after insort_right(2) with duplicates:");
    println!("{data:?}");
    insort_left(
        &mut sorted_ints,
        &SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        None,
    );
    insort_right(
        &mut sorted_ints,
        &SifrInt::from_i64(10),
        SifrInt::from_i64(0),
        None,
    );
    println!("after more inserts:");
    println!("{sorted_ints:?}");
}
fn demo_itertools() {
    println!("=== Section 3: itertools chain ===");
    let a: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
    ];
    let b: Vec<SifrInt> = vec![
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
        SifrInt::from_i64(6),
    ];
    let result: Vec<SifrInt> = chain(&vec![a.to_vec(), b.to_vec()]).collect::<Vec<_>>();
    println!("chain (borrow both):");
    println!("{result:?}");
    println!("a still usable:");
    println!("{}", SifrInt::from(a.len()));
    println!("b still usable:");
    println!("{}", SifrInt::from(b.len()));
    let x: Vec<SifrInt> = vec![
        SifrInt::from_i64(10),
        SifrInt::from_i64(20),
        SifrInt::from_i64(30),
    ];
    let y: Vec<SifrInt> = vec![
        SifrInt::from_i64(40),
        SifrInt::from_i64(50),
        SifrInt::from_i64(60),
    ];
    let combined: Vec<SifrInt> = chain(&vec![x, y]).collect::<Vec<_>>();
    println!("chain result:");
    println!("{combined:?}");
}
fn demo_counter() {
    println!("=== Section 4: Counter with native dict[str, int] ===");
    let words: Vec<String> = vec![
        "apple".to_string(),
        "banana".to_string(),
        "apple".to_string(),
        "cherry".to_string(),
        "banana".to_string(),
        "apple".to_string(),
    ];
    let mut c: SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<String> = from_list(&words);
    println!("apple count:");
    println!("{}", c.get(&"apple".to_string(), &SifrInt::from_i64(0)));
    println!("banana count:");
    println!("{}", c.get(&"banana".to_string(), &SifrInt::from_i64(0)));
    println!("missing key returns 0:");
    println!("{}", c.get(&"missing".to_string(), &SifrInt::from_i64(0)));
    println!("total elements:");
    println!("{}", c.total());
    (&mut c).increment(&"cherry".to_string());
    (&mut c).increment(&"cherry".to_string());
    println!("cherry after 2 increments:");
    println!("{}", c.get(&"cherry".to_string(), &SifrInt::from_i64(0)));
    let top: Vec<(String, SifrInt)> = c.most_common(&Some(SifrInt::from_i64(1).clone()));
    println!("top 1 most common:");
    println!("{top:?}");
    let keys: Vec<String> = c.keys();
    println!("unique keys count:");
    println!("{}", SifrInt::from(keys.len()));
}
fn main() {
    demo_heapq();
    demo_bisect();
    demo_itertools();
    demo_counter();
    println!("=== borrow_stdlib demo complete ===");
}
