// src/main.rs
mod sifr_generated_generated_support {
    use crate::{
        FloatOverflowError, FloatPrecisionLossError, IndexError,
        SifrGeneratedStdlibSifrX2ecollectionsX2eCounter, SifrGeneratedStdlibSifrX2erandomX2eRandom,
        SifrGeneratedStdlibSifrX2erandomX2eRandomState,
        SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError, ValueError,
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
    pub(crate) fn count(start: SifrInt, step: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<SifrInt>| {
                let mut current: SifrInt = start.clone();
                loop {
                    sifr_generated_yielder.suspend(current.clone()).await;
                    current = &current + &step;
                }
            },
        ))
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
    pub(crate) fn count_from(
        start: SifrInt,
        step: SifrInt,
        n: SifrInt,
    ) -> Box<dyn Iterator<Item = SifrInt>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<SifrInt>| {
                let mut i: SifrInt = SifrInt::from_i64(0);
                let mut current: SifrInt = start.clone();
                while &i < &n {
                    sifr_generated_yielder.suspend(current.clone()).await;
                    current = &current + &step;
                    i = &i + &SifrInt::from_i64(1);
                }
            },
        ))
    }
    pub(crate) fn cycle<T: Clone + 'static>(
        data: Box<dyn Iterator<Item = T>>,
        n: SifrInt,
    ) -> Box<dyn Iterator<Item = T>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
                let mut saved: Vec<T> = Vec::new();
                let mut emitted: SifrInt = SifrInt::from_i64(0);
                if &n <= &SifrInt::from_i64(0) {
                    return;
                }
                for value in data {
                    saved.push(value.clone());
                    sifr_generated_yielder.suspend(value.clone()).await;
                    emitted = &emitted + &SifrInt::from_i64(1);
                    if &emitted >= &n {
                        return;
                    }
                }
                while &emitted < &n && &SifrInt::from(saved.len()) > &SifrInt::from_i64(0) {
                    for repeated in saved.iter().cloned() {
                        sifr_generated_yielder.suspend(repeated.clone()).await;
                        emitted = &emitted + &SifrInt::from_i64(1);
                        if &emitted >= &n {
                            return;
                        }
                    }
                }
            },
        ))
    }
    #[expect(
        clippy::approx_constant,
        reason = "generated Rust preserves this exact typed Sifr source contract"
    )]
    pub(crate) const PI: f64 = 3.141_592_653_589_793_f64;
    pub(crate) fn sqrt(x: f64) -> f64 {
        ::sifr_stdlib::math::sqrt(x)
    }
    pub(crate) fn log(x: f64) -> f64 {
        ::sifr_stdlib::math::log(x)
    }
    pub(crate) fn sin(x: f64) -> f64 {
        ::sifr_stdlib::math::sin(x)
    }
    pub(crate) fn cos(x: f64) -> f64 {
        ::sifr_stdlib::math::cos(x)
    }
    pub(crate) fn acosh(x: f64) -> f64 {
        ::sifr_stdlib::math::acosh(x)
    }
    pub(crate) fn asinh(x: f64) -> f64 {
        ::sifr_stdlib::math::asinh(x)
    }
    pub(crate) fn atanh(x: f64) -> f64 {
        ::sifr_stdlib::math::atanh(x)
    }
    pub(crate) fn isqrt(n: SifrInt) -> SifrInt {
        ::sifr_stdlib::math::isqrt(::sifr_runtime::interop::SifrIntBridge::from(n)).into_sifr_int()
    }
    pub(crate) fn dist_impl(p: Vec<f64>, q: Vec<f64>) -> f64 {
        ::sifr_stdlib::math::dist(p, q)
    }
    pub(crate) fn fsum_impl(data: Vec<f64>) -> f64 {
        ::sifr_stdlib::math::fsum(data)
    }
    pub(crate) fn sifr_generated_copy_float_list(data: &[f64]) -> Vec<f64> {
        let mut out: Vec<f64> = Vec::new();
        for value in data.iter().copied() {
            out.push(value);
        }
        out
    }
    pub(crate) fn dist(p: &[f64], q: &[f64]) -> f64 {
        dist_impl(
            sifr_generated_copy_float_list(p),
            sifr_generated_copy_float_list(q),
        )
    }
    pub(crate) fn fsum(data: &[f64]) -> f64 {
        fsum_impl(sifr_generated_copy_float_list(data))
    }
    pub(crate) fn random_word_to_unit_float(value: SifrInt) -> f64 {
        ::sifr_stdlib::random::random_word_to_unit_float(
            ::sifr_runtime::interop::SifrIntBridge::from(value),
        )
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
    pub(crate) fn randrange(
        start: SifrInt,
        stop: Option<SifrInt>,
        step_argument_af0b4e191da20cef: SifrInt,
    ) -> Result<SifrInt, ValueError> {
        let mut generator: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            sifr_generated_module_random();
        let value: Result<SifrInt, ValueError> =
            generator.randrange(&start, &stop, &step_argument_af0b4e191da20cef);
        sifr_generated_sync_module_random(&mut generator);
        value
    }
    pub(crate) fn gauss(mu: f64, sigma: f64) -> f64 {
        let mut generator: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            sifr_generated_module_random();
        let value: f64 = generator.gauss(mu, sigma);
        sifr_generated_sync_module_random(&mut generator);
        value
    }
    pub(crate) fn sample<T: Clone + 'static>(
        items: &[T],
        k: SifrInt,
    ) -> Result<Vec<T>, ValueError> {
        if &k < &SifrInt::from_i64(0) {
            return Err(ValueError::new("sample: k must be >= 0".to_string()));
        }
        if &k > &SifrInt::from(items.len()) {
            return Err(ValueError::new("sample larger than population".to_string()));
        }
        let mut pool: Vec<T> = Vec::new();
        for item in items.iter().cloned() {
            pool.push(item);
        }
        let mut generator: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            sifr_generated_module_random();
        let mut result: Vec<T> = Vec::new();
        let mut remaining: SifrInt = SifrInt::from(pool.len());
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &k {
            if &remaining == &SifrInt::from_i64(0) {
                return Err(ValueError::new("sample larger than population".to_string()));
            }
            let pick_index: SifrInt = generator
                .sifr_generated_next_u32()
                .floor_mod_known_nonzero(&remaining);
            let picked: Option<T> = {
                let sifr_generated_checked_read_collection = &pool;
                let sifr_generated_checked_read_index = pick_index.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(picked) = picked {
                result.push(picked);
            }
            let last: Option<T> = {
                let sifr_generated_checked_read_collection = &pool;
                let sifr_generated_checked_read_index = &remaining - &SifrInt::from_i64(1);
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(last) = last
                && &SifrInt::from_i64(0) <= &pick_index
                && &pick_index < &SifrInt::from(pool.len())
            {
                {
                    let sifr_generated_assign_value = last.clone();
                    {
                        let sifr_generated_index_raw = pick_index.clone();
                        let sifr_generated_index_normalized =
                            sifr_generated_index_raw.normalize_index_or_len(pool.len());
                        if let Some(sifr_generated_elem) =
                            pool.get_mut(sifr_generated_index_normalized)
                        {
                            *sifr_generated_elem = sifr_generated_assign_value;
                        }
                    }
                }
            }
            remaining = &remaining - &SifrInt::from_i64(1);
            i = &i + &SifrInt::from_i64(1);
        }
        sifr_generated_sync_module_random(&mut generator);
        Ok(result)
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub(crate) enum SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(FloatOverflowError),
        SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
            FloatPrecisionLossError,
        ),
    }
    impl From<FloatOverflowError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn from(value: FloatOverflowError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                value,
            )
        }
    }
    impl From<FloatPrecisionLossError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn from(value: FloatPrecisionLossError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    pub(crate) fn sifr_generated_sum(data: &[f64]) -> f64 {
        let mut total: f64 = 0.0_f64;
        for val in data.iter().copied() {
            total += val;
        }
        total
    }
    pub(crate) fn sifr_generated_float_int(
        value: SifrInt,
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let sifr_generated_try_res: Result<
            Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError>,
            SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0,
        > = (|| {
            let converted: f64 = value
                .clone()
                .checked_to_f64()
                .map_err(|sifr_generated_float_error| match sifr_generated_float_error {
                    ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                            FloatOverflowError::new(
                                "exact integer is outside the finite float range"
                                    .to_string(),
                            ),
                        )
                    }
                    ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                        SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                            FloatPrecisionLossError::new(
                                "exact integer cannot be represented without float precision loss"
                                    .to_string(),
                            ),
                        )
                    }
                })?;
            Ok(Ok(converted))
        })();
        sifr_generated_try_res
            .unwrap_or_else(|sifr_generated_try_err| match sifr_generated_try_err {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass18X3aFloatOverflowError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let error = sifr_generated_try_variant_error.clone();
                    Err(
                        SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                            error.message.clone(),
                        ),
                    )
                }
                SifrGeneratedUnion8X3asequence5X3aunion1X3a231X3a5X3aclass18X3aFloatOverflowError1X3a036X3a5X3aclass23X3aFloatPrecisionLossError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3aFloatPrecisionLossError1X3a0(
                    sifr_generated_try_variant_error,
                ) => {
                    let error = sifr_generated_try_variant_error.clone();
                    Err(
                        SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                            error.message.clone(),
                        ),
                    )
                }
            })
    }
    pub(crate) fn sifr_generated_divide_by_int(
        numerator: f64,
        denominator: SifrInt,
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let sifr_generated_try_res: Result<
            Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError>,
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let divisor: f64 = sifr_generated_float_int(denominator.clone())?;
            Ok(Ok(numerator / divisor))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let error = sifr_generated_try_err.clone();
            Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                error.message.clone(),
            ))
        })
    }
    pub(crate) fn mean(
        data: &[f64],
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let count: SifrInt = SifrInt::from(data.len());
        if &count == &SifrInt::from_i64(0) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "mean requires at least one data point".to_string(),
            ));
        }
        let total: f64 = sifr_generated_sum(data);
        sifr_generated_divide_by_int(total, count.clone())
    }
    pub(crate) fn median(
        data: &[f64],
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let n: SifrInt = SifrInt::from(data.len());
        if &n == &SifrInt::from_i64(0) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "median requires at least one data point".to_string(),
            ));
        }
        let sorted_data: Vec<f64> = {
            let mut sifr_generated_sorted_values = data.iter().copied().collect::<Vec<_>>();
            sifr_generated_sorted_values.sort_by(
                |sifr_generated_sorted_left, sifr_generated_sorted_right| {
                    sifr_generated_sorted_left
                        .partial_cmp(sifr_generated_sorted_right)
                        .unwrap_or(::std::cmp::Ordering::Equal)
                },
            );
            sifr_generated_sorted_values
        };
        let mid: SifrInt = n.floor_div_known_nonzero(&SifrInt::from_i64(2));
        if &n.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0) {
            let a: Option<f64> = {
                let sifr_generated_checked_read_collection = &sorted_data;
                let sifr_generated_checked_read_index = &mid - &SifrInt::from_i64(1);
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            let b: Option<f64> = {
                let sifr_generated_checked_read_collection = &sorted_data;
                let sifr_generated_checked_read_index = mid.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(a) = a
                && let Some(b) = b
            {
                return Ok((a + b) / 2.0_f64);
            }
            Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "median: index error".to_string(),
            ))
        } else {
            let val: Option<f64> = {
                let sifr_generated_checked_read_collection = &sorted_data;
                let sifr_generated_checked_read_index = mid.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            let Some(val) = val else {
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    "median: index error".to_string(),
                ));
            };
            Ok(val)
        }
    }
    pub(crate) fn variance(
        data: &[f64],
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let n: SifrInt = SifrInt::from(data.len());
        if &n < &SifrInt::from_i64(2) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "variance requires at least two data points".to_string(),
            ));
        }
        let sifr_generated_try_res: Result<
            (f64,),
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let avg: f64 = sifr_generated_divide_by_int(sifr_generated_sum(data), n.clone())?;
            Ok((avg,))
        })();
        let (avg,) = match sifr_generated_try_res {
            Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
            Err(sifr_generated_try_err) => {
                let error = sifr_generated_try_err.clone();
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    error.message.clone(),
                ));
            }
        };
        let mut total: f64 = 0.0_f64;
        for val in data.iter().copied() {
            let diff: f64 = val - avg;
            total += diff * diff;
        }
        sifr_generated_divide_by_int(total, &n - &SifrInt::from_i64(1))
    }
    pub(crate) fn stdev(
        data: &[f64],
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let n: SifrInt = SifrInt::from(data.len());
        if &n < &SifrInt::from_i64(2) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "stdev requires at least two data points".to_string(),
            ));
        }
        let sifr_generated_try_res: Result<
            (f64,),
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let avg: f64 = sifr_generated_divide_by_int(sifr_generated_sum(data), n.clone())?;
            Ok((avg,))
        })();
        let (avg,) = match sifr_generated_try_res {
            Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
            Err(sifr_generated_try_err) => {
                let error = sifr_generated_try_err.clone();
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    error.message.clone(),
                ));
            }
        };
        let mut total: f64 = 0.0_f64;
        for val in data.iter().copied() {
            let diff: f64 = val - avg;
            total += diff * diff;
        }
        let sifr_generated_try_res: Result<
            (f64,),
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let v: f64 = sifr_generated_divide_by_int(total, &n - &SifrInt::from_i64(1))?;
            Ok((v,))
        })();
        let (v,) = match sifr_generated_try_res {
            Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
            Err(sifr_generated_try_err) => {
                let error = sifr_generated_try_err.clone();
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    error.message.clone(),
                ));
            }
        };
        Ok(sqrt(v))
    }
    pub(crate) fn mode(
        data: &[SifrInt],
    ) -> Result<SifrInt, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        if &SifrInt::from(data.len()) == &SifrInt::from_i64(0) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "mode requires at least one data point".to_string(),
            ));
        }
        let mut counts: HashMap<SifrInt, SifrInt> = HashMap::from([]);
        for val in data.iter().cloned() {
            let existing: Option<SifrInt> = counts.get(&val).cloned();
            if let Some(existing) = existing.clone() {
                {
                    let sifr_generated_assign_value = &existing + &SifrInt::from_i64(1);
                    {
                        let sifr_generated_assign_key = val.clone();
                        counts.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                    }
                }
            } else {
                let sifr_generated_assign_value = SifrInt::from_i64(1);
                {
                    let sifr_generated_assign_key = val.clone();
                    counts.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                }
            }
        }
        let mut best: SifrInt = SifrInt::from_i64(0);
        let mut best_set: bool = false;
        let mut best_count: SifrInt = SifrInt::from_i64(0);
        for val2 in data.iter().cloned() {
            let count2_value_c3423dbe5aaebcf2: Option<SifrInt> = counts.get(&val2).cloned();
            let count2_val: SifrInt = count2_value_c3423dbe5aaebcf2
                .clone()
                .unwrap_or_else(|| SifrInt::from_i64(0));
            if &count2_val > &best_count {
                best_count = count2_val;
                best = val2;
                best_set = true;
            }
        }
        if best_set {
            return Ok(best.clone());
        }
        Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
            "mode: no mode found".to_string(),
        ))
    }
    pub(crate) fn multimode(
        data: &[SifrInt],
    ) -> Result<Vec<SifrInt>, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        if &SifrInt::from(data.len()) == &SifrInt::from_i64(0) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "multimode requires at least one data point".to_string(),
            ));
        }
        let mut counts: HashMap<SifrInt, SifrInt> = HashMap::from([]);
        for val in data.iter().cloned() {
            let existing: Option<SifrInt> = counts.get(&val).cloned();
            if let Some(existing) = existing.clone() {
                {
                    let sifr_generated_assign_value = &existing + &SifrInt::from_i64(1);
                    {
                        let sifr_generated_assign_key = val.clone();
                        counts.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                    }
                }
            } else {
                let sifr_generated_assign_value = SifrInt::from_i64(1);
                {
                    let sifr_generated_assign_key = val.clone();
                    counts.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                }
            }
        }
        let mut max_count: SifrInt = SifrInt::from_i64(0);
        for val2 in data.iter().cloned() {
            let count2_value_c3423dbe5aaebcf2: Option<SifrInt> = counts.get(&val2).cloned();
            let count2_val: SifrInt = count2_value_c3423dbe5aaebcf2
                .clone()
                .unwrap_or_else(|| SifrInt::from_i64(0));
            if &count2_val > &max_count {
                max_count = count2_val;
            }
        }
        let mut result: Vec<SifrInt> = Vec::new();
        let mut seen: HashMap<SifrInt, bool> = HashMap::from([]);
        for val3 in data.iter().cloned() {
            let already_opt: Option<bool> = seen.get(&val3).cloned();
            let already: bool = already_opt.is_some_and(|already_opt| already_opt);
            if !already {
                let count3_value_c3423ebe5aaebea5: Option<SifrInt> = counts.get(&val3).cloned();
                let count3_val_value_7442ae8ecb6bc585: SifrInt = count3_value_c3423ebe5aaebea5
                    .clone()
                    .unwrap_or_else(|| SifrInt::from_i64(0));
                if &count3_val_value_7442ae8ecb6bc585 == &max_count {
                    result.push(val3.clone());
                }
                {
                    let sifr_generated_assign_value = true;
                    {
                        let sifr_generated_assign_key = val3.clone();
                        seen.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                    }
                }
            }
        }
        Ok(result)
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(crate) fn quantiles(
        data: &[f64],
        n: SifrInt,
    ) -> Result<Vec<f64>, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        if &SifrInt::from(data.len()) < &SifrInt::from_i64(2) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "quantiles requires at least two data points".to_string(),
            ));
        }
        if &n < &SifrInt::from_i64(1) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "quantiles: n must be at least 1".to_string(),
            ));
        }
        let sorted_data: Vec<f64> = {
            let mut sifr_generated_sorted_values = data.iter().copied().collect::<Vec<_>>();
            sifr_generated_sorted_values.sort_by(
                |sifr_generated_sorted_left, sifr_generated_sorted_right| {
                    sifr_generated_sorted_left
                        .partial_cmp(sifr_generated_sorted_right)
                        .unwrap_or(::std::cmp::Ordering::Equal)
                },
            );
            sifr_generated_sorted_values
        };
        let m: SifrInt = SifrInt::from(sorted_data.len());
        let mut result: Vec<f64> = Vec::new();
        let mut i: SifrInt = SifrInt::from_i64(1);
        while &i < &n {
            let sifr_generated_try_res: Result<
                (f64, f64, f64),
                SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
            > = (|| {
                let i_float: f64 = sifr_generated_float_int(i.clone())?;
                let m_float_value_b0fecb9ab83ca525: f64 = sifr_generated_float_int(m.clone())?;
                let n_float_value_15c49f18b6cbd018: f64 = sifr_generated_float_int(n.clone())?;
                Ok((
                    i_float,
                    m_float_value_b0fecb9ab83ca525,
                    n_float_value_15c49f18b6cbd018,
                ))
            })();
            let (i_float, m_float_value_b0fecb9ab83ca525, n_float_value_15c49f18b6cbd018) =
                match sifr_generated_try_res {
                    Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
                    Err(sifr_generated_try_err) => {
                        let error = sifr_generated_try_err.clone();
                        return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                            error.message.clone(),
                        ));
                    }
                };
            let idx_f: f64 =
                i_float * m_float_value_b0fecb9ab83ca525 / n_float_value_15c49f18b6cbd018;
            let mut idx: SifrInt = SifrInt::from_i64(0);
            let sifr_generated_try_res: Result<(), ValueError> = (|| {
                let converted_idx: SifrInt =
                    SifrInt::from_f64_trunc(idx_f).ok_or_else(|| ValueError {
                        message: "cannot convert non-finite float to int".to_string(),
                    })?;
                idx = converted_idx;
                Ok(())
            })();
            if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                let error = sifr_generated_try_err.clone();
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    error.message.clone(),
                ));
            }
            let sifr_generated_try_res: Result<
                (f64,),
                SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
            > = (|| {
                let idx_float: f64 = sifr_generated_float_int(idx.clone())?;
                Ok((idx_float,))
            })();
            let (idx_float,) = match sifr_generated_try_res {
                Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
                Err(sifr_generated_try_err) => {
                    let error = sifr_generated_try_err.clone();
                    return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                        error.message.clone(),
                    ));
                }
            };
            let frac: f64 = idx_f - idx_float;
            if &idx >= &m {
                idx = &m - &SifrInt::from_i64(1);
            }
            if &idx < &SifrInt::from_i64(0) {
                idx = SifrInt::from_i64(0);
            }
            let lo: Option<f64> = {
                let sifr_generated_checked_read_collection = &sorted_data;
                let sifr_generated_checked_read_index = idx.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            let mut lo_val: f64 = lo.unwrap_or(0.0_f64);
            if frac > 0.0_f64 {
                let hi_idx: SifrInt = &idx + &SifrInt::from_i64(1);
                if &hi_idx < &m {
                    let hi: Option<f64> = {
                        let sifr_generated_checked_read_collection = &sorted_data;
                        let sifr_generated_checked_read_index = hi_idx.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(hi) = hi {
                        lo_val += frac * (hi - lo_val);
                    }
                }
            }
            result.push(lo_val);
            i = &i + &SifrInt::from_i64(1);
        }
        Ok(result)
    }
    pub(crate) fn covariance(
        x: &[f64],
        y: &[f64],
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let n: SifrInt = SifrInt::from(x.len());
        if &n < &SifrInt::from_i64(2) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "covariance requires at least two data points".to_string(),
            ));
        }
        if &SifrInt::from(y.len()) != &n {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "covariance: x and y must have the same length".to_string(),
            ));
        }
        let sifr_generated_try_res: Result<
            (f64, f64),
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let mx: f64 = sifr_generated_divide_by_int(sifr_generated_sum(x), n.clone())?;
            let my: f64 = sifr_generated_divide_by_int(sifr_generated_sum(y), n.clone())?;
            Ok((mx, my))
        })();
        let (mx, my) = match sifr_generated_try_res {
            Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
            Err(sifr_generated_try_err) => {
                let error = sifr_generated_try_err.clone();
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    error.message.clone(),
                ));
            }
        };
        let mut total: f64 = 0.0_f64;
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &n {
            let xi: Option<f64> = {
                let sifr_generated_checked_read_collection = &x;
                let sifr_generated_checked_read_index = i.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            let yi: Option<f64> = {
                let sifr_generated_checked_read_collection = &y;
                let sifr_generated_checked_read_index = i.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(xi) = xi
                && let Some(yi) = yi
            {
                total += (xi - mx) * (yi - my);
            }
            i = &i + &SifrInt::from_i64(1);
        }
        sifr_generated_divide_by_int(total, &n - &SifrInt::from_i64(1))
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(crate) fn correlation(
        x: &[f64],
        y: &[f64],
    ) -> Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let n: SifrInt = SifrInt::from(x.len());
        if &n < &SifrInt::from_i64(2) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "correlation requires at least two data points".to_string(),
            ));
        }
        if &SifrInt::from(y.len()) != &n {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "correlation: x and y must have the same length".to_string(),
            ));
        }
        let sifr_generated_try_res: Result<
            (f64, f64),
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let mx: f64 = sifr_generated_divide_by_int(sifr_generated_sum(x), n.clone())?;
            let my: f64 = sifr_generated_divide_by_int(sifr_generated_sum(y), n.clone())?;
            Ok((mx, my))
        })();
        let (mx, my) = match sifr_generated_try_res {
            Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
            Err(sifr_generated_try_err) => {
                let error = sifr_generated_try_err.clone();
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    error.message.clone(),
                ));
            }
        };
        let mut cov_num: f64 = 0.0_f64;
        let mut sx_num: f64 = 0.0_f64;
        let mut sy_num_value_0e49c538a785c2b2: f64 = 0.0_f64;
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &n {
            let xi: Option<f64> = {
                let sifr_generated_checked_read_collection = &x;
                let sifr_generated_checked_read_index = i.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            let yi: Option<f64> = {
                let sifr_generated_checked_read_collection = &y;
                let sifr_generated_checked_read_index = i.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(xi) = xi
                && let Some(yi) = yi
            {
                cov_num += (xi - mx) * (yi - my);
                sx_num += (xi - mx) * (xi - mx);
                sy_num_value_0e49c538a785c2b2 += (yi - my) * (yi - my);
            }
            i = &i + &SifrInt::from_i64(1);
        }
        let sifr_generated_try_res: Result<
            (f64, f64),
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let sx_variance: f64 =
                sifr_generated_divide_by_int(sx_num, &n - &SifrInt::from_i64(1))?;
            let sy_variance_value_29a72f81ad7b8e6d: f64 = sifr_generated_divide_by_int(
                sy_num_value_0e49c538a785c2b2,
                &n - &SifrInt::from_i64(1),
            )?;
            let sx: f64 = sqrt(sx_variance);
            let sy: f64 = sqrt(sy_variance_value_29a72f81ad7b8e6d);
            Ok((sx, sy))
        })();
        let (sx, sy) = match sifr_generated_try_res {
            Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
            Err(sifr_generated_try_err) => {
                let error = sifr_generated_try_err.clone();
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    error.message.clone(),
                ));
            }
        };
        if sx == 0.0_f64 {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "correlation: x has zero variance".to_string(),
            ));
        }
        if sy == 0.0_f64 {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "correlation: y has zero variance".to_string(),
            ));
        }
        let sifr_generated_try_res: Result<
            Result<f64, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError>,
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let covariance_value: f64 =
                sifr_generated_divide_by_int(cov_num, &n - &SifrInt::from_i64(1))?;
            Ok(Ok(covariance_value / (sx * sy)))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let error = sifr_generated_try_err.clone();
            Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                error.message.clone(),
            ))
        })
    }
    pub(crate) fn linear_regression(
        x: &[f64],
        y: &[f64],
    ) -> Result<Vec<f64>, SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> {
        let n: SifrInt = SifrInt::from(x.len());
        if &n < &SifrInt::from_i64(2) {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "linear_regression requires at least two data points".to_string(),
            ));
        }
        if &SifrInt::from(y.len()) != &n {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "linear_regression: x and y must have the same length".to_string(),
            ));
        }
        let sifr_generated_try_res: Result<
            (f64, f64),
            SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError,
        > = (|| {
            let mx: f64 = sifr_generated_divide_by_int(sifr_generated_sum(x), n.clone())?;
            let my: f64 = sifr_generated_divide_by_int(sifr_generated_sum(y), n.clone())?;
            Ok((mx, my))
        })();
        let (mx, my) = match sifr_generated_try_res {
            Ok(sifr_generated_try_bindings) => sifr_generated_try_bindings,
            Err(sifr_generated_try_err) => {
                let error = sifr_generated_try_err.clone();
                return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                    error.message.clone(),
                ));
            }
        };
        let mut num: f64 = 0.0_f64;
        let mut den: f64 = 0.0_f64;
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &n {
            let xi: Option<f64> = {
                let sifr_generated_checked_read_collection = &x;
                let sifr_generated_checked_read_index = i.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            let yi: Option<f64> = {
                let sifr_generated_checked_read_collection = &y;
                let sifr_generated_checked_read_index = i.clone();
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(xi) = xi
                && let Some(yi) = yi
            {
                num += (xi - mx) * (yi - my);
                den += (xi - mx) * (xi - mx);
            }
            i = &i + &SifrInt::from_i64(1);
        }
        if den == 0.0_f64 {
            return Err(SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError::new(
                "linear_regression: x has zero variance".to_string(),
            ));
        }
        let slope: f64 = num / den;
        let intercept: f64 = my - slope * mx;
        let result: Vec<f64> = vec![slope, intercept];
        Ok(result)
    }
}
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::*;
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
        pub fn update(&mut self, other: &SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>) {
            for key in other.counts.keys().cloned().collect::<Vec<_>>() {
                let other_val: Option<SifrInt> = other.counts.get(&key).cloned();
                if let Some(other_val) = other_val.clone() {
                    let existing: Option<SifrInt> = self.counts.get(&key).cloned();
                    if let Some(existing) = existing.clone() {
                        {
                            let sifr_generated_assign_value = &existing + &other_val;
                            {
                                let sifr_generated_assign_key = key.clone();
                                self.counts
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    } else {
                        let sifr_generated_assign_value = other_val.clone();
                        {
                            let sifr_generated_assign_key = key.clone();
                            self.counts
                                .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                        }
                    }
                }
            }
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T> {
        pub fn subtract(&mut self, other: &SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>) {
            for key in other.counts.keys().cloned().collect::<Vec<_>>() {
                let other_val: Option<SifrInt> = other.counts.get(&key).cloned();
                if let Some(other_val) = other_val.clone() {
                    let existing: Option<SifrInt> = self.counts.get(&key).cloned();
                    if let Some(existing) = existing.clone() {
                        {
                            let sifr_generated_assign_value = &existing - &other_val;
                            {
                                let sifr_generated_assign_key = key.clone();
                                self.counts
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    } else {
                        let sifr_generated_assign_value = &SifrInt::from_i64(0) - &other_val;
                        {
                            let sifr_generated_assign_key = key.clone();
                            self.counts
                                .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                        }
                    }
                }
            }
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T> {
        #[must_use]
        pub fn elements(&self) -> Vec<T> {
            let mut result: Vec<T> = Vec::new();
            let all_keys: Vec<T> = self.counts.keys().cloned().collect::<Vec<_>>();
            let mut ki: SifrInt = SifrInt::from_i64(0);
            while &ki < &SifrInt::from(all_keys.len()) {
                let key_opt: Option<T> = {
                    let sifr_generated_checked_read_collection = &all_keys;
                    let sifr_generated_checked_read_index = ki.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(key_opt) = key_opt {
                    let cnt: Option<SifrInt> = self.counts.get(&key_opt).cloned();
                    if let Some(cnt) = cnt.clone() {
                        let mut i: SifrInt = SifrInt::from_i64(0);
                        while &i < &cnt {
                            let key_copy: Option<T> = {
                                let sifr_generated_checked_read_collection = &all_keys;
                                let sifr_generated_checked_read_index = ki.clone();
                                let sifr_generated_checked_read_normalized =
                                    sifr_generated_checked_read_index.normalize_index_or_len(
                                        sifr_generated_checked_read_collection.len(),
                                    );
                                sifr_generated_checked_read_collection
                                    .get(sifr_generated_checked_read_normalized)
                                    .cloned()
                            };
                            if let Some(key_copy) = key_copy {
                                result.push(key_copy.clone());
                            }
                            i = &i + &SifrInt::from_i64(1);
                        }
                    }
                }
                ki = &ki + &SifrInt::from_i64(1);
            }
            result
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
        pub fn random(&mut self) -> f64 {
            random_word_to_unit_float(self.sifr_generated_next_u32())
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn randrange(
            &mut self,
            start: &SifrInt,
            stop: &Option<SifrInt>,
            step_argument_af0b4e191da20cef: &SifrInt,
        ) -> Result<SifrInt, ValueError> {
            if step_argument_af0b4e191da20cef == &SifrInt::from_i64(0) {
                return Err(ValueError::new(
                    "randrange: step must not be zero".to_string(),
                ));
            }
            let mut actual_start: SifrInt = start.clone();
            let mut actual_stop_value_351bdef5a4961be0: SifrInt = start.clone();
            if stop.is_none() {
                actual_start = SifrInt::from_i64(0);
            } else if let Some(stop) = stop.as_ref() {
                actual_stop_value_351bdef5a4961be0 = stop.clone();
            }
            let width: SifrInt = &actual_stop_value_351bdef5a4961be0 - &actual_start;
            if step_argument_af0b4e191da20cef > &SifrInt::from_i64(0) {
                if &width <= &SifrInt::from_i64(0) {
                    return Err(ValueError::new("randrange: empty range".to_string()));
                }
            } else if &width >= &SifrInt::from_i64(0) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
            let mut abs_width: SifrInt = width.clone();
            if &abs_width < &SifrInt::from_i64(0) {
                abs_width = &SifrInt::from_i64(0) - &abs_width;
            }
            let mut abs_step: SifrInt = step_argument_af0b4e191da20cef.clone();
            if &abs_step < &SifrInt::from_i64(0) {
                abs_step = &SifrInt::from_i64(0) - &abs_step;
            }
            if &abs_step == &SifrInt::from_i64(0) {
                return Err(ValueError::new(
                    "randrange: step must not be zero".to_string(),
                ));
            }
            let count: SifrInt = (&(&abs_width + &abs_step) - &SifrInt::from_i64(1))
                .floor_div_known_nonzero(&abs_step);
            if &count <= &SifrInt::from_i64(0) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
            if &count == &SifrInt::from_i64(0) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
            let pick: SifrInt = self
                .sifr_generated_next_u32()
                .floor_mod_known_nonzero(&count);
            Ok(&actual_start + &(&pick * step_argument_af0b4e191da20cef))
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        #[must_use]
        pub fn gauss(&mut self, mu: f64, sigma: f64) -> f64 {
            let cached: Option<f64> = self.gauss_next;
            if let Some(cached) = cached {
                self.gauss_next = None;
                return mu + sigma * cached;
            }
            let mut u1: f64 = self.random();
            if u1 <= 0.0_f64 {
                u1 = 0.000_000_000_001_f64;
            }
            let u2: f64 = self.random();
            let radius: f64 = sqrt(-2.0_f64 * log(u1));
            let theta: f64 = 2.0_f64 * PI * u2;
            let z0: f64 = radius * cos(theta);
            let z1: f64 = radius * sin(theta);
            let next_cached: Option<f64> = Some(z1);
            self.gauss_next = next_cached;
            mu + sigma * z0
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
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {
        pub message: String,
    }
    impl SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Debug for SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("StatisticsError")
                .field("message", &self.message)
                .finish()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }
    impl ::std::error::Error for SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError {}
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
    pub struct FloatOverflowError {
        pub message: String,
    }
    impl FloatOverflowError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for FloatOverflowError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for FloatOverflowError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct FloatPrecisionLossError {
        pub message: String,
    }
    impl FloatPrecisionLossError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for FloatPrecisionLossError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for FloatPrecisionLossError {}
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
pub use sifr_generated_project_nominals::FloatOverflowError;
pub use sifr_generated_project_nominals::FloatPrecisionLossError;
pub use sifr_generated_project_nominals::IndexError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecollectionsX2eCounter;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2erandomX2eRandom;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2erandomX2eRandomState;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError;
pub use sifr_generated_project_nominals::ValueError;
fn add(a: SifrInt, b: SifrInt) -> SifrInt {
    &a + &b
}
fn mul(a: SifrInt, b: SifrInt) -> SifrInt {
    &a * &b
}
fn less_than_three(x: SifrInt) -> bool {
    &x < &SifrInt::from_i64(3)
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
#[expect(
    clippy::many_single_char_names,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn main() {
    println!("=== math additions ===");
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(13usize);
        sifr_generated_concat.push_str("acosh(1.0) = ");
        sifr_generated_concat.push_str(acosh(1.0_f64).to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(13usize);
        sifr_generated_concat.push_str("asinh(0.0) = ");
        sifr_generated_concat.push_str(asinh(0.0_f64).to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(13usize);
        sifr_generated_concat.push_str("atanh(0.0) = ");
        sifr_generated_concat.push_str(atanh(0.0_f64).to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(12usize);
        sifr_generated_concat.push_str("isqrt(17) = ");
        sifr_generated_concat.push_str(isqrt(SifrInt::from_i64(17)).to_string().as_str());
        sifr_generated_concat
    });
    let p: Vec<f64> = vec![0.0_f64, 0.0_f64];
    let q: Vec<f64> = vec![3.0_f64, 4.0_f64];
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(20usize);
        sifr_generated_concat.push_str("dist([0,0],[3,4]) = ");
        sifr_generated_concat.push_str(dist(&p, &q).to_string().as_str());
        sifr_generated_concat
    });
    let data_fsum: Vec<f64> = vec![
        0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64,
    ];
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(15usize);
        sifr_generated_concat.push_str("fsum(10x0.1) = ");
        sifr_generated_concat.push_str(fsum(&data_fsum).to_string().as_str());
        sifr_generated_concat
    });
    println!("=== statistics (Result[float, StatisticsError]) ===");
    let data: Vec<f64> = vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64, 5.0_f64];
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let m: f64 = mean(&data)?;
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(7usize);
                sifr_generated_concat.push_str("mean = ");
                sifr_generated_concat.push_str(m.to_string().as_str());
                sifr_generated_concat
            });
            let med: f64 = median(&data)?;
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(9usize);
                sifr_generated_concat.push_str("median = ");
                sifr_generated_concat.push_str(med.to_string().as_str());
                sifr_generated_concat
            });
            let v: f64 = variance(&data)?;
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(11usize);
                sifr_generated_concat.push_str("variance = ");
                sifr_generated_concat.push_str(v.to_string().as_str());
                sifr_generated_concat
            });
            let s: f64 = stdev(&data)?;
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(8usize);
                sifr_generated_concat.push_str("stdev = ");
                sifr_generated_concat.push_str(s.to_string().as_str());
                sifr_generated_concat
            });
            let idata_value_4763ae3b1301126c: Vec<SifrInt> = vec![
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(2),
                SifrInt::from_i64(3),
                SifrInt::from_i64(3),
                SifrInt::from_i64(3),
            ];
            let mo: SifrInt = mode(&idata_value_4763ae3b1301126c)?;
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(7usize);
                sifr_generated_concat.push_str("mode = ");
                sifr_generated_concat.push_str(mo.to_string().as_str());
                sifr_generated_concat
            });
            let mm: Vec<SifrInt> = multimode(&vec![
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(2),
                SifrInt::from_i64(3),
                SifrInt::from_i64(3),
            ])?;
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(16usize);
                sifr_generated_concat.push_str("multimode len = ");
                sifr_generated_concat.push_str(SifrInt::from(mm.len()).to_string().as_str());
                sifr_generated_concat
            });
            let qs: Vec<f64> = quantiles(&data, SifrInt::from_i64(4))?;
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(18usize);
                sifr_generated_concat.push_str("quartiles count = ");
                sifr_generated_concat.push_str(SifrInt::from(qs.len()).to_string().as_str());
                sifr_generated_concat
            });
            let x: Vec<f64> = vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64, 5.0_f64];
            let y: Vec<f64> = vec![2.0_f64, 4.0_f64, 6.0_f64, 8.0_f64, 10.0_f64];
            let cov: f64 = covariance(&x, &y)?;
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(13usize);
                sifr_generated_concat.push_str("covariance = ");
                sifr_generated_concat.push_str(cov.to_string().as_str());
                sifr_generated_concat
            });
            let r: f64 = correlation(&x, &y)?;
            println!("{}", {
                let mut sifr_generated_concat: String = String::with_capacity(14usize);
                sifr_generated_concat.push_str("correlation = ");
                sifr_generated_concat.push_str(r.to_string().as_str());
                sifr_generated_concat
            });
            let lr: Vec<f64> = linear_regression(&x, &y)?;
            let slope: Option<f64> = {
                let sifr_generated_checked_read_collection = &lr;
                let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            let intercept: Option<f64> = {
                let sifr_generated_checked_read_collection = &lr;
                let sifr_generated_checked_read_index = SifrInt::from_i64(1);
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(slope) = slope {
                println!("{}", {
                    let mut sifr_generated_concat: String = String::with_capacity(8usize);
                    sifr_generated_concat.push_str("slope = ");
                    sifr_generated_concat.push_str(slope.to_string().as_str());
                    sifr_generated_concat
                });
            }
            if let Some(intercept) = intercept {
                println!("{}", {
                    let mut sifr_generated_concat: String = String::with_capacity(12usize);
                    sifr_generated_concat.push_str("intercept = ");
                    sifr_generated_concat.push_str(intercept.to_string().as_str());
                    sifr_generated_concat
                });
            }
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(7usize);
            sifr_generated_concat.push_str("error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2estatisticsX2eStatisticsError> =
        (|| {
            let empty: Vec<f64> = Vec::new();
            let _bad: f64 = mean(&empty)?;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(18usize);
            sifr_generated_concat.push_str("empty mean error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    println!("=== random additions ===");
    let mut items: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
    ];
    shuffle(&mut items);
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(14usize);
        sifr_generated_concat.push_str("shuffle len = ");
        sifr_generated_concat.push_str(SifrInt::from(items.len()).to_string().as_str());
        sifr_generated_concat
    });
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let s3: Vec<SifrInt> = sample(&items, SifrInt::from_i64(3))?;
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(16usize);
            sifr_generated_concat.push_str("sample(3) len = ");
            sifr_generated_concat.push_str(SifrInt::from(s3.len()).to_string().as_str());
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(14usize);
            sifr_generated_concat.push_str("sample error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let rr: SifrInt = randrange(
            SifrInt::from_i64(0),
            Some(SifrInt::from_i64(100)),
            SifrInt::from_i64(5),
        )?;
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(21usize);
            sifr_generated_concat.push_str("randrange in range = ");
            sifr_generated_concat.push_str((&rr >= &SifrInt::from_i64(0)).to_string().as_str());
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(17usize);
            sifr_generated_concat.push_str("randrange error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let _g: f64 = gauss(0.0_f64, 1.0_f64);
    println!("gauss sample is float = True");
    println!("=== functools.reduce ===");
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
    ];
    let total: SifrInt = reduce(
        |sifr_generated_arg0, sifr_generated_arg1| {
            add(sifr_generated_arg0.clone(), sifr_generated_arg1.clone())
        },
        &nums,
        &SifrInt::from_i64(0),
    );
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(14usize);
        sifr_generated_concat.push_str("reduce(add) = ");
        sifr_generated_concat.push_str(total.to_string().as_str());
        sifr_generated_concat
    });
    let product: SifrInt = reduce(
        |sifr_generated_arg0, sifr_generated_arg1| {
            mul(sifr_generated_arg0.clone(), sifr_generated_arg1.clone())
        },
        &nums,
        &SifrInt::from_i64(1),
    );
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(14usize);
        sifr_generated_concat.push_str("reduce(mul) = ");
        sifr_generated_concat.push_str(product.to_string().as_str());
        sifr_generated_concat
    });
    println!("=== itertools additions ===");
    let idata2: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
    ];
    let acc: Vec<SifrInt> =
        accumulate(Box::new(idata2.clone().into_iter()), None).collect::<Vec<_>>();
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(13usize);
        sifr_generated_concat.push_str("accumulate = ");
        sifr_generated_concat.push_str(format!("{acc:?}").as_str());
        sifr_generated_concat
    });
    let sel: Vec<bool> = vec![true, false, true, false, true];
    let comp: Vec<SifrInt> = compress(
        Box::new(idata2.clone().into_iter()),
        Box::new(sel.clone().into_iter()),
    )
    .collect::<Vec<_>>();
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(11usize);
        sifr_generated_concat.push_str("compress = ");
        sifr_generated_concat.push_str(format!("{comp:?}").as_str());
        sifr_generated_concat
    });
    let dw: Vec<SifrInt> = dropwhile(
        |sifr_generated_arg0| less_than_three(sifr_generated_arg0.clone()),
        Box::new(idata2.clone().into_iter()),
    )
    .collect::<Vec<_>>();
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(16usize);
        sifr_generated_concat.push_str("dropwhile(<3) = ");
        sifr_generated_concat.push_str(format!("{dw:?}").as_str());
        sifr_generated_concat
    });
    let tw: Vec<SifrInt> = takewhile(
        |sifr_generated_arg0| less_than_three(sifr_generated_arg0.clone()),
        Box::new(idata2.clone().into_iter()),
    )
    .collect::<Vec<_>>();
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(16usize);
        sifr_generated_concat.push_str("takewhile(<3) = ");
        sifr_generated_concat.push_str(format!("{tw:?}").as_str());
        sifr_generated_concat
    });
    let ff: Vec<SifrInt> = filterfalse(
        |sifr_generated_arg0| less_than_three(sifr_generated_arg0.clone()),
        Box::new(idata2.clone().into_iter()),
    )
    .collect::<Vec<_>>();
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(18usize);
        sifr_generated_concat.push_str("filterfalse(<3) = ");
        sifr_generated_concat.push_str(format!("{ff:?}").as_str());
        sifr_generated_concat
    });
    let a: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
    ];
    let b: Vec<SifrInt> = vec![SifrInt::from_i64(4), SifrInt::from_i64(5)];
    let zl: Vec<Vec<SifrInt>> = zip_longest(
        Box::new(a.clone().into_iter()),
        Box::new(b.clone().into_iter()),
        &SifrInt::from_i64(0),
    )
    .collect::<Vec<_>>();
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(18usize);
        sifr_generated_concat.push_str("zip_longest len = ");
        sifr_generated_concat.push_str(SifrInt::from(zl.len()).to_string().as_str());
        sifr_generated_concat
    });
    let cf: Vec<SifrInt> = count_from(
        SifrInt::from_i64(0),
        SifrInt::from_i64(2),
        SifrInt::from_i64(5),
    )
    .collect::<Vec<_>>();
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(20usize);
        sifr_generated_concat.push_str("count_from(0,2,5) = ");
        sifr_generated_concat.push_str(format!("{cf:?}").as_str());
        sifr_generated_concat
    });
    let mut ctr: Box<dyn Iterator<Item = SifrInt>> =
        count(SifrInt::from_i64(0), SifrInt::from_i64(2));
    let count0: Option<SifrInt> = ctr.next();
    let count1_value_c3423cbe5aaebb3f: Option<SifrInt> = ctr.next();
    let count2_value_c3423dbe5aaebcf2: Option<SifrInt> = ctr.next();
    let count3_value_c3423ebe5aaebea5: Option<SifrInt> = ctr.next();
    let count4_value_c34237be5aaeb2c0: Option<SifrInt> = ctr.next();
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(21usize);
        sifr_generated_concat.push_str("count(0,2) first 5 = ");
        sifr_generated_concat.push_str(
            format!(
                "{:?}",
                vec![
                    count0,
                    count1_value_c3423cbe5aaebb3f,
                    count2_value_c3423dbe5aaebcf2,
                    count3_value_c3423ebe5aaebea5,
                    count4_value_c34237be5aaeb2c0
                ]
            )
            .as_str(),
        );
        sifr_generated_concat
    });
    let cyc: Vec<SifrInt> = cycle(
        Box::new(
            vec![
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(3),
            ]
            .into_iter(),
        ),
        SifrInt::from_i64(7),
    )
    .collect::<Vec<_>>();
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(20usize);
        sifr_generated_concat.push_str("cycle([1,2,3], 7) = ");
        sifr_generated_concat.push_str(format!("{cyc:?}").as_str());
        sifr_generated_concat
    });
    println!("=== Counter enhancements ===");
    let mut c1: SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<String> = from_list(&vec![
        "a".to_string(),
        "b".to_string(),
        "a".to_string(),
        "c".to_string(),
    ]);
    let c2: SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<String> =
        from_list(&vec!["b".to_string(), "c".to_string(), "d".to_string()]);
    (&mut c1).update(&c2);
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(16usize + 3usize);
        sifr_generated_concat.push_str("after update: a=");
        sifr_generated_concat.push_str(
            c1.get(&"a".to_string(), &SifrInt::from_i64(0))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat.push_str(" b=");
        sifr_generated_concat.push_str(
            c1.get(&"b".to_string(), &SifrInt::from_i64(0))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    let mut c3: SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<String> =
        from_list(&vec!["x".to_string(), "x".to_string(), "y".to_string()]);
    let c4: SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<String> =
        from_list(&vec!["x".to_string()]);
    (&mut c3).subtract(&c4);
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(18usize);
        sifr_generated_concat.push_str("after subtract: x=");
        sifr_generated_concat.push_str(
            c3.get(&"x".to_string(), &SifrInt::from_i64(0))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    let c5: SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<String> =
        from_list(&vec!["a".to_string(), "a".to_string(), "b".to_string()]);
    let elems: Vec<String> = c5.elements();
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(15usize);
        sifr_generated_concat.push_str("elements len = ");
        sifr_generated_concat.push_str(SifrInt::from(elems.len()).to_string().as_str());
        sifr_generated_concat
    });
    let mut cc: SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<String> =
        from_list(&vec!["a".to_string(), "b".to_string()]);
    (&mut cc).update(&from_list(&vec!["b".to_string(), "c".to_string()]));
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(16usize);
        sifr_generated_concat.push_str("counter_add b = ");
        sifr_generated_concat.push_str(
            cc.get(&"b".to_string(), &SifrInt::from_i64(0))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    let mut cd: SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<String> =
        from_list(&vec!["a".to_string(), "a".to_string(), "b".to_string()]);
    (&mut cd).subtract(&from_list(&vec!["a".to_string()]));
    println!("{}", {
        let mut sifr_generated_concat: String = String::with_capacity(16usize);
        sifr_generated_concat.push_str("counter_sub a = ");
        sifr_generated_concat.push_str(
            cd.get(&"a".to_string(), &SifrInt::from_i64(0))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    println!("=== stdlib_pure_expansion: all features demonstrated ===");
}
