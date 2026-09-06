// src/main.rs
mod sifr_generated_generated_support {
    use crate::{
        IndexError, SifrGeneratedStdlibSifrX2ecollectionsX2eCounter,
        SifrGeneratedStdlibSifrX2erandomX2eRandom, SifrGeneratedStdlibSifrX2erandomX2eRandomState,
        ValueError,
    };
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) use ::std::collections::HashMap;
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
    pub(crate) fn reduce<T: Clone + 'static, U: Clone + 'static>(
        func: impl Fn(&U, &T) -> U,
        data: &[T],
        initial: &U,
    ) -> U {
        let mut result: U = initial.clone();
        for val in data.iter().cloned() {
            result = func(&result, &val);
        }
        result
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
    pub(crate) trait SifrGeneratedAdd: Sized {
        #[must_use]
        fn sifr_generated_add(self, rhs: Self) -> Self;
    }
    impl SifrGeneratedAdd for ::sifr_runtime::SifrInt {
        fn sifr_generated_add(self, rhs: Self) -> Self {
            self + rhs
        }
    }
    impl SifrGeneratedAdd for f64 {
        fn sifr_generated_add(self, rhs: Self) -> Self {
            self + rhs
        }
    }
    impl SifrGeneratedAdd for String {
        fn sifr_generated_add(mut self, rhs: Self) -> Self {
            self.push_str(&rhs);
            self
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
    pub(crate) fn take<T: Clone + 'static>(n: SifrInt, data: &[T]) -> Vec<T> {
        let mut result: Vec<T> = Vec::new();
        let mut count: SifrInt = SifrInt::from_i64(0);
        for item in data.iter().cloned() {
            if &count >= &n {
                return result;
            }
            result.push(item);
            count = &count + &SifrInt::from_i64(1);
        }
        result
    }
    pub(crate) fn flatten<T: Clone + 'static>(lists: &[Vec<T>]) -> Vec<T> {
        let mut result: Vec<T> = Vec::new();
        for inner in lists.iter().cloned() {
            for val in inner.iter().cloned() {
                result.push(val);
            }
        }
        result
    }
    pub(crate) fn accumulate<T: Clone + 'static + SifrGeneratedAdd>(
        data: Box<dyn Iterator<Item = T>>,
        initial: Option<T>,
    ) -> Box<dyn Iterator<Item = T>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
                let mut state: Vec<T> = Vec::new();
                if let Some(initial) = initial {
                    state.push(initial);
                    let initial_value: Option<T> = {
                        let sifr_generated_checked_read_collection = &state;
                        let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(initial_value) = initial_value {
                        sifr_generated_yielder.suspend(initial_value.clone()).await;
                    }
                }
                for item in data {
                    if &SifrInt::from(state.len()) == &SifrInt::from_i64(0) {
                        state.push(item);
                    } else {
                        let prev: Option<T> = {
                            let sifr_generated_checked_read_collection = &state;
                            let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                            let sifr_generated_checked_read_normalized =
                                sifr_generated_checked_read_index.normalize_index_or_len(
                                    sifr_generated_checked_read_collection.len(),
                                );
                            sifr_generated_checked_read_collection
                                .get(sifr_generated_checked_read_normalized)
                                .cloned()
                        };
                        if let Some(prev) = prev {
                            let next_val: T = SifrGeneratedAdd::sifr_generated_add(prev, item);
                            let sifr_generated_try_res: Result<(), IndexError> = (|| {
                                {
                                    let sifr_generated_assign_value = next_val.clone();
                                    {
                                        let sifr_generated_index_raw = SifrInt::from_i64(0);
                                        let sifr_generated_index_normalized =
                                            sifr_generated_index_raw
                                                .normalize_index_or_len(state.len());
                                        if let Some(sifr_generated_elem) =
                                            state.get_mut(sifr_generated_index_normalized)
                                        {
                                            *sifr_generated_elem = sifr_generated_assign_value;
                                        } else {
                                            return Err(IndexError::new(
                                                "collection index out of range".to_string(),
                                            ));
                                        }
                                    }
                                }
                                Ok(())
                            })(
                            );
                            if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                                let _e = sifr_generated_try_err.clone();
                                return;
                            }
                        }
                    }
                    let current: Option<T> = {
                        let sifr_generated_checked_read_collection = &state;
                        let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(current) = current {
                        sifr_generated_yielder.suspend(current.clone()).await;
                    }
                }
            },
        ))
    }
    pub(crate) fn compress<T: Clone + 'static>(
        data: Box<dyn Iterator<Item = T>>,
        selectors: Box<dyn Iterator<Item = bool>>,
    ) -> Box<dyn Iterator<Item = T>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
                for (value, selector) in
                    Box::new(data.zip(selectors).map(|sifr_generated_zip_item| {
                        (sifr_generated_zip_item.0, sifr_generated_zip_item.1)
                    }))
                {
                    if selector {
                        sifr_generated_yielder.suspend(value.clone()).await;
                    }
                }
            },
        ))
    }
    pub(crate) fn dropwhile<T: Clone + 'static>(
        pred: impl Fn(&T) -> bool + Send + Sync + 'static,
        data: Box<dyn Iterator<Item = T>>,
    ) -> Box<dyn Iterator<Item = T>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
                let mut dropping: bool = true;
                for val in data {
                    if dropping {
                        if !pred(&val) {
                            dropping = false;
                            sifr_generated_yielder.suspend(val.clone()).await;
                        }
                    } else {
                        sifr_generated_yielder.suspend(val.clone()).await;
                    }
                }
            },
        ))
    }
    pub(crate) fn takewhile<T: Clone + 'static>(
        pred: impl Fn(&T) -> bool + Send + Sync + 'static,
        data: Box<dyn Iterator<Item = T>>,
    ) -> Box<dyn Iterator<Item = T>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
                for val in data {
                    if !pred(&val) {
                        return;
                    }
                    sifr_generated_yielder.suspend(val.clone()).await;
                }
            },
        ))
    }
    pub(crate) fn filterfalse<T: Clone + 'static>(
        pred: impl Fn(&T) -> bool + Send + Sync + 'static,
        data: Box<dyn Iterator<Item = T>>,
    ) -> Box<dyn Iterator<Item = T>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
                for val in data {
                    if !pred(&val) {
                        sifr_generated_yielder.suspend(val.clone()).await;
                    }
                }
            },
        ))
    }
    pub(crate) fn zip_longest<T: Clone + 'static>(
        a: Box<dyn Iterator<Item = T>>,
        b: Box<dyn Iterator<Item = T>>,
        fill: &T,
    ) -> Box<dyn Iterator<Item = Vec<T>>> {
        let fill = fill.clone();
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<Vec<T>>| {
                let mut left: Box<dyn Iterator<Item = T>> = a;
                let mut right: Box<dyn Iterator<Item = T>> = b;
                loop {
                    let left_value: Option<T> = left.next();
                    let right_value: Option<T> = right.next();
                    if left_value.is_none() && right_value.is_none() {
                        return;
                    }
                    let mut pair: Vec<T> = Vec::new();
                    if let Some(left_value) = left_value {
                        pair.push(left_value);
                    } else {
                        pair.push(fill.clone());
                    }
                    if let Some(right_value) = right_value {
                        pair.push(right_value);
                    } else {
                        pair.push(fill.clone());
                    }
                    sifr_generated_yielder.suspend(pair.to_vec()).await;
                }
            },
        ))
    }
    pub(crate) fn random_seed() -> SifrInt {
        ::sifr_stdlib::random::random_seed().into_sifr_int()
    }
    pub(crate) fn random_module_state_words() -> Vec<SifrInt> {
        ::sifr_stdlib::random::random_module_state_words()
            .into_iter()
            .map(::sifr_runtime::interop::SifrIntBridge::into_sifr_int)
            .collect()
    }
    pub(crate) fn random_module_state_index() -> SifrInt {
        ::sifr_stdlib::random::random_module_state_index().into_sifr_int()
    }
    pub(crate) fn random_module_state_gauss_next() -> Option<f64> {
        ::sifr_stdlib::random::random_module_state_gauss_next()
    }
    pub(crate) fn random_module_set_state(
        words: &[SifrInt],
        index: SifrInt,
        gauss_next: Option<f64>,
    ) -> Result<(), ValueError> {
        ::sifr_stdlib::random::random_module_set_state(
            &words
                .iter()
                .cloned()
                .map(::sifr_runtime::interop::SifrIntBridge::from)
                .collect::<Vec<_>>(),
            ::sifr_runtime::interop::SifrIntBridge::from(index),
            gauss_next,
        )
        .map_err(|sifr_generated_bridge_error| ValueError {
            message: sifr_generated_bridge_error.to_string(),
        })
    }
    pub(crate) const fn sifr_generated_const_5f4d545f4e() -> SifrInt {
        SifrInt::from_i64(624)
    }
    pub(crate) const fn sifr_generated_const_5f4d545f4d() -> SifrInt {
        SifrInt::from_i64(397)
    }
    pub(crate) const fn sifr_generated_const_5f4d545f4d41545249585f41() -> SifrInt {
        SifrInt::from_i64(2_567_483_615)
    }
    pub(crate) const fn sifr_generated_const_5f4d545f55505045525f4d41534b() -> SifrInt {
        SifrInt::from_i64(2_147_483_648)
    }
    pub(crate) const fn sifr_generated_const_5f4d545f4c4f5745525f4d41534b() -> SifrInt {
        SifrInt::from_i64(2_147_483_647)
    }
    pub(crate) const fn sifr_generated_const_5f4d545f46() -> SifrInt {
        SifrInt::from_i64(1_812_433_253)
    }
    pub(crate) const fn sifr_generated_const_5f4d545f574f52445f4d41534b() -> SifrInt {
        SifrInt::from_i64(4_294_967_295)
    }
    pub(crate) fn sifr_generated_state_word_at(words: &[SifrInt], index: SifrInt) -> SifrInt {
        let value: Option<SifrInt> = {
            let sifr_generated_checked_read_collection = &words;
            let sifr_generated_checked_read_index = index.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        let Some(value_value_7ce4fd9430e80cea) = value.clone() else {
            return SifrInt::from_i64(0);
        };
        value_value_7ce4fd9430e80cea
    }
    pub(crate) fn sifr_generated_clone_words(words: &[SifrInt]) -> Vec<SifrInt> {
        let mut copied: Vec<SifrInt> = Vec::new();
        for word in words.iter().cloned() {
            copied.push(word);
        }
        copied
    }
    pub(crate) fn sifr_generated_normalize_seed_input(seed_value: Option<SifrInt>) -> SifrInt {
        let Some(seed_value) = seed_value.clone() else {
            return random_seed();
        };
        seed_value.clone()
    }
    pub(crate) fn sifr_generated_seed_words_from_seed(seed_value: SifrInt) -> Vec<SifrInt> {
        let mut words: Vec<SifrInt> =
            vec![&seed_value & &sifr_generated_const_5f4d545f574f52445f4d41534b()];
        let mut i: SifrInt = SifrInt::from_i64(1);
        while &i < &sifr_generated_const_5f4d545f4e() {
            let prev: SifrInt = sifr_generated_state_word_at(&words, &i - &SifrInt::from_i64(1));
            let next_word: SifrInt = &(&(&sifr_generated_const_5f4d545f46()
                * &(&prev ^ &prev.floor_div_known_nonzero(&SifrInt::from_i64(1_073_741_824))))
                + &i)
                & &sifr_generated_const_5f4d545f574f52445f4d41534b();
            words.push(next_word);
            i = &i + &SifrInt::from_i64(1);
        }
        words
    }
    pub(crate) fn sifr_generated_build_state_from_module_storage()
    -> SifrGeneratedStdlibSifrX2erandomX2eRandomState {
        SifrGeneratedStdlibSifrX2erandomX2eRandomState::new(
            SifrInt::from_i64(3),
            random_module_state_words(),
            random_module_state_index(),
            random_module_state_gauss_next(),
        )
    }
    pub(crate) fn sifr_generated_store_state_into_module_storage(
        state: &SifrGeneratedStdlibSifrX2erandomX2eRandomState,
    ) {
        let sifr_generated_set_result: Result<(), ValueError> = random_module_set_state(
            &sifr_generated_clone_words(&state.state_words.clone()),
            state.index.clone(),
            state.gauss_next,
        );
        let _ = sifr_generated_set_result;
    }
    pub(crate) fn sifr_generated_ensure_module_state_initialized() {
        let words: Vec<SifrInt> = random_module_state_words();
        if &SifrInt::from(words.len()) == &sifr_generated_const_5f4d545f4e() {
            return;
        }
        let bootstrap: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            SifrGeneratedStdlibSifrX2erandomX2eRandom::new(Some(SifrInt::from_i64(5489)));
        sifr_generated_store_state_into_module_storage(&bootstrap.getstate());
    }
    pub(crate) fn sifr_generated_module_random() -> SifrGeneratedStdlibSifrX2erandomX2eRandom {
        sifr_generated_ensure_module_state_initialized();
        let mut r: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            SifrGeneratedStdlibSifrX2erandomX2eRandom::new(Some(SifrInt::from_i64(0)));
        let sifr_generated_set_result: Result<(), ValueError> =
            r.setstate(&sifr_generated_build_state_from_module_storage());
        let _ = sifr_generated_set_result;
        r
    }
    pub(crate) fn sifr_generated_sync_module_random(
        generator: &mut SifrGeneratedStdlibSifrX2erandomX2eRandom,
    ) {
        sifr_generated_store_state_into_module_storage(&generator.getstate());
    }
    pub(crate) fn shuffle<T: Clone + 'static>(items: &mut Vec<T>) {
        let mut generator: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            sifr_generated_module_random();
        let n: SifrInt = SifrInt::from(items.len());
        if &n > &SifrInt::from_i64(1) {
            let mut i: SifrInt = &n - &SifrInt::from_i64(1);
            while &i > &SifrInt::from_i64(0) {
                let divisor: SifrInt = &i + &SifrInt::from_i64(1);
                if &divisor == &SifrInt::from_i64(0) {
                    return;
                }
                let j: SifrInt = generator
                    .sifr_generated_next_u32()
                    .floor_mod_known_nonzero(&divisor);
                let left: Option<T> = {
                    let sifr_generated_checked_read_collection = &items;
                    let sifr_generated_checked_read_index = i.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let right: Option<T> = {
                    let sifr_generated_checked_read_collection = &items;
                    let sifr_generated_checked_read_index = j.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(left) = left
                    && let Some(right) = right
                {
                    if &SifrInt::from_i64(0) <= &i && &i < &SifrInt::from(items.len()) {
                        {
                            let sifr_generated_assign_value = right.clone();
                            {
                                let sifr_generated_index_raw = i.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(items.len());
                                if let Some(sifr_generated_elem) =
                                    items.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                }
                            }
                        }
                    }
                    if &SifrInt::from_i64(0) <= &j && &j < &SifrInt::from(items.len()) {
                        {
                            let sifr_generated_assign_value = left.clone();
                            {
                                let sifr_generated_index_raw = j.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(items.len());
                                if let Some(sifr_generated_elem) =
                                    items.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                }
                            }
                        }
                    }
                }
                i = &i - &SifrInt::from_i64(1);
            }
        }
        sifr_generated_sync_module_random(&mut generator);
    }
}
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::*;
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
        pub fn append(&mut self, val: &T) {
            self.data.push_back(val.clone());
            let maxlen_opt: Option<SifrInt> = self.maxlen.clone();
            if let Some(maxlen_opt) = maxlen_opt.clone() {
                let maxlen: SifrInt = maxlen_opt.clone();
                if &SifrInt::from(self.data.len()) > &maxlen {
                    self.data.pop_front();
                }
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
    #[derive(Debug, Clone, PartialEq)]
    pub struct SifrGeneratedStdlibSifrX2erandomX2eRandomState {
        pub version: SifrInt,
        pub state_words: Vec<SifrInt>,
        pub index: SifrInt,
        pub gauss_next: Option<f64>,
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandomState {
        #[must_use]
        pub fn new(
            version: SifrInt,
            state_words: Vec<SifrInt>,
            index: SifrInt,
            gauss_next: Option<f64>,
        ) -> Self {
            let sifr_generated_field_value_bb62c62c9808ea37_76657273696f6e: SifrInt =
                version.clone();
            let sifr_generated_field_value_8e62ac2dd7162e8c_73746174655f776f726473: Vec<SifrInt> =
                state_words;
            let sifr_generated_field_value_83cf8e8f9081468b_696e646578: SifrInt = index.clone();
            let sifr_generated_field_value_edec7000e7b3eeaa_67617573735f6e657874: Option<f64> =
                gauss_next;
            Self {
                version: sifr_generated_field_value_bb62c62c9808ea37_76657273696f6e,
                state_words: sifr_generated_field_value_8e62ac2dd7162e8c_73746174655f776f726473,
                index: sifr_generated_field_value_83cf8e8f9081468b_696e646578,
                gauss_next: sifr_generated_field_value_edec7000e7b3eeaa_67617573735f6e657874,
            }
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct SifrGeneratedStdlibSifrX2erandomX2eRandom {
        pub state_words: Vec<SifrInt>,
        pub index: SifrInt,
        pub gauss_next: Option<f64>,
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        #[must_use]
        pub fn new(seed_value: Option<SifrInt>) -> Self {
            let normalized_seed: SifrInt = sifr_generated_normalize_seed_input(seed_value.clone());
            let sifr_generated_field_value_7e372b502c45daad_5f73746174655f776f726473: Vec<SifrInt> =
                sifr_generated_seed_words_from_seed(normalized_seed.clone());
            let sifr_generated_field_value_497043933c8a2d12_5f696e646578: SifrInt =
                sifr_generated_const_5f4d545f4e().clone();
            let sifr_generated_field_value_88c1b3a412b57c41_5f67617573735f6e657874: Option<f64> =
                None;
            Self {
                state_words: sifr_generated_field_value_7e372b502c45daad_5f73746174655f776f726473,
                index: sifr_generated_field_value_497043933c8a2d12_5f696e646578,
                gauss_next: sifr_generated_field_value_88c1b3a412b57c41_5f67617573735f6e657874,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        pub fn sifr_generated_twist(&mut self) {
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &SifrInt::from_i64(0) <= &i && &i < &SifrInt::from(self.state_words.len()) {
                let y: SifrInt = &(&sifr_generated_state_word_at(&self.state_words, i.clone())
                    & &sifr_generated_const_5f4d545f55505045525f4d41534b())
                    + &(&sifr_generated_state_word_at(
                        &self.state_words,
                        (&i + &SifrInt::from_i64(1))
                            .floor_mod_known_nonzero(&sifr_generated_const_5f4d545f4e()),
                    ) & &sifr_generated_const_5f4d545f4c4f5745525f4d41534b());
                let mut x_a: SifrInt = y.floor_div_known_nonzero(&SifrInt::from_i64(2));
                if &y.floor_mod_known_nonzero(&SifrInt::from_i64(2)) != &SifrInt::from_i64(0) {
                    x_a = &x_a ^ &sifr_generated_const_5f4d545f4d41545249585f41();
                }
                let new_word: SifrInt = &sifr_generated_state_word_at(
                    &self.state_words,
                    (&i + &sifr_generated_const_5f4d545f4d())
                        .floor_mod_known_nonzero(&sifr_generated_const_5f4d545f4e()),
                ) ^ &x_a;
                {
                    let sifr_generated_assign_value =
                        &new_word & &sifr_generated_const_5f4d545f574f52445f4d41534b();
                    {
                        let sifr_generated_index_raw = i.clone();
                        let sifr_generated_index_normalized =
                            sifr_generated_index_raw.normalize_index_or_len(self.state_words.len());
                        if let Some(sifr_generated_elem) =
                            self.state_words.get_mut(sifr_generated_index_normalized)
                        {
                            *sifr_generated_elem = sifr_generated_assign_value;
                        }
                    }
                }
                i = &i + &SifrInt::from_i64(1);
            }
            self.index = SifrInt::from_i64(0);
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        #[must_use]
        pub fn sifr_generated_next_u32(&mut self) -> SifrInt {
            if &self.index.clone() >= &sifr_generated_const_5f4d545f4e() {
                self.sifr_generated_twist();
            }
            let mut y: SifrInt =
                sifr_generated_state_word_at(&self.state_words, self.index.clone());
            self.index = &self.index.clone() + &SifrInt::from_i64(1);
            y = &y ^ &y.floor_div_known_nonzero(&SifrInt::from_i64(2048));
            y = &y ^ &(&(&y * &SifrInt::from_i64(128)) & &SifrInt::from_i64(2_636_928_640));
            y = &y ^ &(&(&y * &SifrInt::from_i64(32768)) & &SifrInt::from_i64(4_022_730_752));
            y = &y ^ &y.floor_div_known_nonzero(&SifrInt::from_i64(262_144));
            &y & &sifr_generated_const_5f4d545f574f52445f4d41534b()
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        #[must_use]
        pub fn getstate(&self) -> SifrGeneratedStdlibSifrX2erandomX2eRandomState {
            SifrGeneratedStdlibSifrX2erandomX2eRandomState::new(
                SifrInt::from_i64(3),
                sifr_generated_clone_words(&self.state_words),
                self.index.clone(),
                self.gauss_next,
            )
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn setstate(
            &mut self,
            state: &SifrGeneratedStdlibSifrX2erandomX2eRandomState,
        ) -> Result<(), ValueError> {
            if &state.version.clone() != &SifrInt::from_i64(3) {
                return Err(ValueError::new("setstate: unsupported version".to_string()));
            }
            if &SifrInt::from(state.state_words.len()) != &sifr_generated_const_5f4d545f4e() {
                return Err(ValueError::new(
                    "setstate: state_words must have length 624".to_string(),
                ));
            }
            if &state.index.clone() < &SifrInt::from_i64(0)
                || &state.index.clone() > &sifr_generated_const_5f4d545f4e()
            {
                return Err(ValueError::new(
                    "setstate: index must be in range [0, 624]".to_string(),
                ));
            }
            let mut normalized: Vec<SifrInt> = Vec::new();
            for word in state.state_words.iter().cloned() {
                if &word < &SifrInt::from_i64(0)
                    || &word > &sifr_generated_const_5f4d545f574f52445f4d41534b()
                {
                    return Err(ValueError::new("setstate: word out of range".to_string()));
                }
                normalized.push(&word & &sifr_generated_const_5f4d545f574f52445f4d41534b());
            }
            self.state_words = normalized;
            self.index = state.index.clone();
            self.gauss_next = state.gauss_next;
            Ok(())
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IndexError {
        pub message: String,
    }
    impl IndexError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for IndexError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for IndexError {}
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::IndexError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecollectionsX2eCounter;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecollectionsX2edeque;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2erandomX2eRandom;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2erandomX2eRandomState;
pub use sifr_generated_project_nominals::ValueError;
fn is_small(x: SifrInt) -> bool {
    &x < &SifrInt::from_i64(5)
}
fn is_even(x: SifrInt) -> bool {
    &x.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0)
}
fn concat(a: &str, b: &str) -> String {
    {
        let mut sifr_generated_concat: String = String::with_capacity(a.len() + b.len());
        sifr_generated_concat.push_str(a);
        sifr_generated_concat.push_str(b);
        sifr_generated_concat
    }
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    println!("=== Generic chain ===");
    let ints: Vec<SifrInt> = chain(&vec![
        vec![SifrInt::from_i64(1), SifrInt::from_i64(2)],
        vec![SifrInt::from_i64(3), SifrInt::from_i64(4)],
    ])
    .collect::<Vec<_>>();
    println!("{ints:?}");
    let strs: Vec<String> = chain(&vec![
        vec!["a".to_string(), "b".to_string()],
        vec!["c".to_string(), "d".to_string()],
    ])
    .collect::<Vec<_>>();
    println!("{strs:?}");
    println!("=== Generic take ===");
    let first3_int: Vec<SifrInt> = take(
        SifrInt::from_i64(3),
        &vec![
            SifrInt::from_i64(10),
            SifrInt::from_i64(20),
            SifrInt::from_i64(30),
            SifrInt::from_i64(40),
            SifrInt::from_i64(50),
        ]
        .into_iter()
        .collect::<Vec<_>>(),
    );
    println!("{first3_int:?}");
    let first2_str: Vec<String> = take(
        SifrInt::from_i64(2),
        &vec!["hello".to_string(), "world".to_string(), "foo".to_string()]
            .into_iter()
            .collect::<Vec<_>>(),
    );
    println!("{first2_str:?}");
    println!("=== Generic flatten ===");
    let nested_int: Vec<Vec<SifrInt>> = vec![
        vec![SifrInt::from_i64(1), SifrInt::from_i64(2)],
        vec![SifrInt::from_i64(3), SifrInt::from_i64(4)],
        vec![SifrInt::from_i64(5)],
    ];
    let flat_int: Vec<SifrInt> = flatten(&nested_int.iter().cloned().collect::<Vec<_>>());
    println!("{flat_int:?}");
    println!("=== Generic accumulate ===");
    let sums: Vec<SifrInt> = accumulate(
        Box::new(
            vec![
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(3),
                SifrInt::from_i64(4),
                SifrInt::from_i64(5),
            ]
            .into_iter(),
        ),
        None,
    )
    .collect::<Vec<_>>();
    println!("{sums:?}");
    let float_sums: Vec<f64> =
        accumulate(Box::new(vec![1.0_f64, 2.5_f64, 3.5_f64].into_iter()), None).collect::<Vec<_>>();
    println!("{float_sums:?}");
    println!("=== Predicate-based dropwhile ===");
    let data: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(3),
        SifrInt::from_i64(7),
        SifrInt::from_i64(2),
        SifrInt::from_i64(8),
    ];
    let dropped: Vec<SifrInt> = dropwhile(
        |sifr_generated_arg0| is_small(sifr_generated_arg0.clone()),
        Box::new(data.clone().into_iter()),
    )
    .collect::<Vec<_>>();
    println!("{dropped:?}");
    println!("=== Predicate-based takewhile ===");
    let taken: Vec<SifrInt> = takewhile(
        |sifr_generated_arg0| is_small(sifr_generated_arg0.clone()),
        Box::new(data.clone().into_iter()),
    )
    .collect::<Vec<_>>();
    println!("{taken:?}");
    println!("=== Predicate-based filterfalse ===");
    let odds: Vec<SifrInt> = filterfalse(
        |sifr_generated_arg0| is_even(sifr_generated_arg0.clone()),
        Box::new(
            vec![
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(3),
                SifrInt::from_i64(4),
                SifrInt::from_i64(5),
                SifrInt::from_i64(6),
            ]
            .into_iter(),
        ),
    )
    .collect::<Vec<_>>();
    println!("{odds:?}");
    println!("=== Generic heapq ===");
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(9),
        SifrInt::from_i64(3),
        SifrInt::from_i64(7),
        SifrInt::from_i64(1),
        SifrInt::from_i64(5),
    ];
    let small: Vec<SifrInt> = nsmallest(SifrInt::from_i64(3), &items);
    println!("{small:?}");
    let big: Vec<SifrInt> = nlargest(SifrInt::from_i64(2), &items);
    println!("{big:?}");
    println!("=== Generic Counter[T] ===");
    let words: Vec<String> = vec![
        "apple".to_string(),
        "banana".to_string(),
        "apple".to_string(),
        "cherry".to_string(),
        "banana".to_string(),
        "apple".to_string(),
    ];
    let c: SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<String> = from_list(&words);
    println!("{}", c.get(&"apple".to_string(), &SifrInt::from_i64(0)));
    println!("{}", c.total());
    let top: Vec<(String, SifrInt)> = c.most_common(&Some(SifrInt::from_i64(2).clone()));
    println!("{top:?}");
    let nums_value_5b9fcbba5284fda0: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(3),
        SifrInt::from_i64(3),
    ];
    let ci: SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<SifrInt> =
        from_list(&nums_value_5b9fcbba5284fda0);
    println!("{}", ci.get(&SifrInt::from_i64(3), &SifrInt::from_i64(0)));
    let c2: SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<String> =
        from_list(&vec!["banana".to_string(), "date".to_string()]);
    let combined: SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<String> = &c + &c2;
    println!(
        "{}",
        combined.get(&"banana".to_string(), &SifrInt::from_i64(0))
    );
    println!("=== Generic deque[T] ===");
    let mut d: SifrGeneratedStdlibSifrX2ecollectionsX2edeque<String> =
        SifrGeneratedStdlibSifrX2ecollectionsX2edeque::new(None, None);
    (&mut d).append(&"first".to_string());
    (&mut d).append(&"second".to_string());
    (&mut d).appendleft(&"zero".to_string());
    let items_d_value_64ad086cd3ff4d9c: Vec<String> = d.to_list();
    println!("{items_d_value_64ad086cd3ff4d9c:?}");
    println!("{}", d.len());
    println!("=== Generic reduce ===");
    let sentence: String = reduce(
        |sifr_generated_arg0, sifr_generated_arg1| {
            concat(sifr_generated_arg0.as_str(), sifr_generated_arg1.as_str())
        },
        &vec!["hello".to_string(), " ".to_string(), "world".to_string()],
        &String::new(),
    );
    println!("{sentence}");
    println!("=== Generic compress ===");
    let data_c_value_f1d709ee29c81197: Vec<String> = vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
        "d".to_string(),
        "e".to_string(),
    ];
    let sel: Vec<bool> = vec![true, false, true, false, true];
    let compressed: Vec<String> = compress(
        Box::new(data_c_value_f1d709ee29c81197.clone().into_iter()),
        Box::new(sel.clone().into_iter()),
    )
    .collect::<Vec<_>>();
    println!("{compressed:?}");
    println!("=== Generic zip_longest ===");
    let zl_str: Vec<Vec<String>> = zip_longest(
        Box::new(vec!["a".to_string(), "b".to_string(), "c".to_string()].into_iter()),
        Box::new(vec!["x".to_string(), "y".to_string()].into_iter()),
        &"-".to_string(),
    )
    .collect::<Vec<_>>();
    for pair in zl_str.iter().cloned() {
        println!("{pair:?}");
    }
    println!("=== Generic shuffle ===");
    let mut shuffled_str: Vec<String> = vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
        "d".to_string(),
        "e".to_string(),
    ];
    shuffle(&mut shuffled_str);
    println!("{}", SifrInt::from(shuffled_str.len()));
}
