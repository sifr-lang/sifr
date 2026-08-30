// src/main.rs
mod __sifr_project_nominals {
    pub use ::std::collections::HashMap;
    pub use ::std::collections::VecDeque;
    pub use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2ecollections_x2eCounter<T: std::hash::Hash + Eq> {
        pub counts: HashMap<T, SifrInt>,
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn new(source: Option<HashMap<T, SifrInt>>, iterable: Option<Vec<T>>) -> Self {
            let mut counts: HashMap<T, SifrInt> = HashMap::from([]);
            if let Some(source) = source {
                for key in source.keys().cloned().collect::<Vec<_>>() {
                    let value: Option<SifrInt> = source.get(&key).cloned();
                    if let Some(value) = value.clone() {
                        counts.insert(key.clone(), value.clone());
                    }
                }
            }
            if let Some(iterable) = iterable {
                for item in iterable.iter().cloned() {
                    let value2: Option<SifrInt> = counts.get(&item).cloned();
                    if let Some(value2) = value2.clone() {
                        counts.insert(item.clone(), &value2 + &SifrInt::from_i64(1));
                    } else {
                        counts.insert(item.clone(), SifrInt::from_i64(1));
                    }
                }
            }
            let __sifr_field_init_0: HashMap<T, SifrInt> = counts;
            Self {
                counts: __sifr_field_init_0,
            }
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn __iter__(&self) -> Vec<T> {
            Box::new((self.counts.keys().cloned().collect::<Vec<_>>()).into_iter())
                .collect::<Vec<_>>()
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn __getitem__(&self, key: &T) -> SifrInt {
            let val: Option<SifrInt> = self.counts.get(&key).cloned();
            if let Some(val) = val.clone() {
                return val;
            }
            SifrInt::from_i64(0)
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn get(&self, key: &T, default: &SifrInt) -> SifrInt {
            let val: Option<SifrInt> = self.counts.get(&key).cloned();
            if let Some(val) = val.clone() {
                return val;
            }
            default.clone()
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn increment(&mut self, key: &T) {
            let val: Option<SifrInt> = self.counts.get(&key).cloned();
            if let Some(val) = val.clone() {
                self.counts.insert(key.clone(), &val + &SifrInt::from_i64(1));
            } else {
                self.counts.insert(key.clone(), SifrInt::from_i64(1));
            }
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn total(&self) -> SifrInt {
            let mut total: SifrInt = SifrInt::from_i64(0);
            for count in self.counts.values().cloned().collect::<Vec<_>>() {
                total = &total + &count;
            }
            total
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn most_common(&self, n: &Option<SifrInt>) -> Vec<(T, SifrInt)> {
            let mut result: Vec<(T, SifrInt)> = vec![];
            for key in self.counts.keys().cloned().collect::<Vec<_>>() {
                let count: Option<SifrInt> = self.counts.get(&key).cloned();
                if let Some(count) = count.clone() {
                    let entry: (T, SifrInt) = (key, count.clone());
                    result.push(entry.clone());
                }
            }
            let sz: SifrInt = SifrInt::from(result.len());
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &i < &sz {
                let mut j: SifrInt = &i + &SifrInt::from_i64(1);
                while &j < &sz {
                    let left: Option<(T, SifrInt)> = Some(
                        result[::sifr_runtime::to_usize_proven(&(i))].clone(),
                    );
                    let right: Option<(T, SifrInt)> = Some(
                        result[::sifr_runtime::to_usize_proven(&(j))].clone(),
                    );
                    if let Some(left) = left {
                        if let Some(right) = right {
                            if ((right).1.clone() > (left).1.clone()) {
                                {
                                    let __idx_raw = i.clone();
                                    let __idx_norm = __idx_raw
                                        .normalize_index_or_len(result.len());
                                    if let Some(__elem) = result.get_mut(__idx_norm) {
                                        *__elem = right.clone();
                                    }
                                }
                                {
                                    let __idx_raw = j.clone();
                                    let __idx_norm = __idx_raw
                                        .normalize_index_or_len(result.len());
                                    if let Some(__elem) = result.get_mut(__idx_norm) {
                                        *__elem = left.clone();
                                    }
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
            if (&n.clone() <= &SifrInt::from_i64(0)) {
                return vec![];
            }
            let mut top: Vec<(T, SifrInt)> = vec![];
            let mut index: SifrInt = SifrInt::from_i64(0);
            while index < *n {
                if (&index >= &SifrInt::from(result.len())) {
                    return top;
                }
                let value: Option<(T, SifrInt)> = Some(
                    result[::sifr_runtime::to_usize_proven(&(index))].clone(),
                );
                if let Some(value) = value {
                    top.push(value.clone());
                }
                index = &index + &SifrInt::from_i64(1);
            }
            top
        }
    }
    impl<
        T: ::std::hash::Hash + Eq + Clone + PartialOrd,
    > __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn keys(&self) -> Vec<T> {
            let mut result: Vec<T> = self.counts.keys().cloned().collect::<Vec<_>>();
            let sz: SifrInt = SifrInt::from(result.len());
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &i < &sz {
                let mut j: SifrInt = &i + &SifrInt::from_i64(1);
                while &j < &sz {
                    let left: Option<T> = Some(
                        result[::sifr_runtime::to_usize_proven(&(i))].clone(),
                    );
                    let right: Option<T> = Some(
                        result[::sifr_runtime::to_usize_proven(&(j))].clone(),
                    );
                    if let Some(left) = left {
                        if let Some(right) = right {
                            if right < left {
                                {
                                    let __idx_raw = i.clone();
                                    let __idx_norm = __idx_raw
                                        .normalize_index_or_len(result.len());
                                    if let Some(__elem) = result.get_mut(__idx_norm) {
                                        *__elem = right.clone();
                                    }
                                }
                                {
                                    let __idx_raw = j.clone();
                                    let __idx_norm = __idx_raw
                                        .normalize_index_or_len(result.len());
                                    if let Some(__elem) = result.get_mut(__idx_norm) {
                                        *__elem = left.clone();
                                    }
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
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn items(&self) -> Vec<(T, SifrInt)> {
            let mut result: Vec<(T, SifrInt)> = vec![];
            for key in self.counts.keys().cloned().collect::<Vec<_>>() {
                let value: Option<SifrInt> = self.counts.get(&key).cloned();
                if let Some(value) = value.clone() {
                    let entry: (T, SifrInt) = (key, value.clone());
                    result.push(entry.clone());
                }
            }
            result
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn values(&self) -> Vec<SifrInt> {
            self.counts.values().cloned().collect::<Vec<_>>()
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn copy(&self) -> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
            __SifrStdlib_sifr_x2ecollections_x2eCounter::new(Some(self.counts.clone()), None)
        }
    }
    impl<T: ::std::hash::Hash + Eq> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn clear(&mut self) {
            self.counts = HashMap::from([]);
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn update(&mut self, other: &__SifrStdlib_sifr_x2ecollections_x2eCounter<T>) {
            for key in other.counts.keys().cloned().collect::<Vec<_>>() {
                let other_val: Option<SifrInt> = other.counts.get(&key).cloned();
                if let Some(other_val) = other_val.clone() {
                    let existing: Option<SifrInt> = self.counts.get(&key).cloned();
                    if let Some(existing) = existing.clone() {
                        self.counts.insert(key, &existing + &other_val);
                    } else {
                        self.counts.insert(key, other_val);
                    }
                }
            }
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn subtract(&mut self, other: &__SifrStdlib_sifr_x2ecollections_x2eCounter<T>) {
            for key in other.counts.keys().cloned().collect::<Vec<_>>() {
                let other_val: Option<SifrInt> = other.counts.get(&key).cloned();
                if let Some(other_val) = other_val.clone() {
                    let existing: Option<SifrInt> = self.counts.get(&key).cloned();
                    if let Some(existing) = existing.clone() {
                        self.counts.insert(key, &existing - &other_val);
                    } else {
                        self.counts.insert(key, &SifrInt::from_i64(0) - &other_val);
                    }
                }
            }
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn elements(&self) -> Vec<T> {
            let mut result: Vec<T> = vec![];
            let all_keys: Vec<T> = self.counts.keys().cloned().collect::<Vec<_>>();
            let mut ki: SifrInt = SifrInt::from_i64(0);
            while (&ki < &SifrInt::from(all_keys.len())) {
                let key_opt: Option<T> = Some(
                    all_keys[::sifr_runtime::to_usize_proven(&(ki))].clone(),
                );
                if let Some(key_opt) = key_opt {
                    let cnt: Option<SifrInt> = self.counts.get(&key_opt).cloned();
                    if let Some(cnt) = cnt.clone() {
                        let mut i: SifrInt = SifrInt::from_i64(0);
                        while &i < &cnt {
                            let key_copy: Option<T> = Some(
                                all_keys[::sifr_runtime::to_usize_proven(&(ki))].clone(),
                            );
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
    impl<
        T: ::std::hash::Hash + Eq + Clone,
    > ::std::ops::Add<&__SifrStdlib_sifr_x2ecollections_x2eCounter<T>>
    for &__SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        type Output = __SifrStdlib_sifr_x2ecollections_x2eCounter<T>;
        fn add(
            self,
            other: &__SifrStdlib_sifr_x2ecollections_x2eCounter<T>,
        ) -> Self::Output {
            let mut new_counts: HashMap<T, SifrInt> = HashMap::from([]);
            for key in Box::new(
                (self.counts.keys().cloned().collect::<Vec<_>>()).into_iter(),
            ) {
                let a_val: Option<SifrInt> = self.counts.get(&key).cloned();
                if let Some(a_val) = a_val {
                    let b_val: Option<SifrInt> = other.counts.get(&key).cloned();
                    let mut b_count: SifrInt = SifrInt::from_i64(0);
                    if let Some(b_val) = b_val.clone() {
                        b_count = b_val;
                    }
                    let total: SifrInt = &a_val + &b_count;
                    if &total > &SifrInt::from_i64(0) {
                        new_counts.insert(key.clone(), total.clone());
                    }
                }
            }
            for key2 in Box::new(
                (other.counts.keys().cloned().collect::<Vec<_>>()).into_iter(),
            ) {
                let already: Option<SifrInt> = new_counts.get(&key2).cloned();
                if already.is_none() {
                    let b_val2: Option<SifrInt> = other.counts.get(&key2).cloned();
                    if let Some(b_val2) = b_val2.clone() {
                        if &b_val2 > &SifrInt::from_i64(0) {
                            new_counts.insert(key2.clone(), b_val2.clone());
                        }
                    }
                }
            }
            __SifrStdlib_sifr_x2ecollections_x2eCounter::new(Some(new_counts), None)
        }
    }
    impl<
        T: ::std::hash::Hash + Eq + Clone,
    > ::std::ops::Sub<&__SifrStdlib_sifr_x2ecollections_x2eCounter<T>>
    for &__SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        type Output = __SifrStdlib_sifr_x2ecollections_x2eCounter<T>;
        fn sub(
            self,
            other: &__SifrStdlib_sifr_x2ecollections_x2eCounter<T>,
        ) -> Self::Output {
            let mut new_counts: HashMap<T, SifrInt> = HashMap::from([]);
            for key in Box::new(
                (self.counts.keys().cloned().collect::<Vec<_>>()).into_iter(),
            ) {
                let a_val: Option<SifrInt> = self.counts.get(&key).cloned();
                if let Some(a_val) = a_val {
                    let b_val: Option<SifrInt> = other.counts.get(&key).cloned();
                    let mut b_count: SifrInt = SifrInt::from_i64(0);
                    if let Some(b_val) = b_val.clone() {
                        b_count = b_val;
                    }
                    let diff: SifrInt = &a_val - &b_count;
                    if &diff > &SifrInt::from_i64(0) {
                        new_counts.insert(key.clone(), diff.clone());
                    }
                }
            }
            __SifrStdlib_sifr_x2ecollections_x2eCounter::new(Some(new_counts), None)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub _data: VecDeque<T>,
        pub maxlen: Option<SifrInt>,
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn new(items: Option<Vec<T>>, maxlen: Option<SifrInt>) -> Self {
            let mut data: Vec<T> = vec![];
            if let Some(items) = items {
                let mut start: SifrInt = SifrInt::from_i64(0);
                if let Some(maxlen) = maxlen.clone() {
                    if (&SifrInt::from(items.len()) > &maxlen) {
                        start = &SifrInt::from(items.len()) - &maxlen;
                    }
                }
                let mut i: SifrInt = start;
                while (&i < &SifrInt::from(items.len())) {
                    let item: Option<T> = Some(
                        items[::sifr_runtime::to_usize_proven(&(i))].clone(),
                    );
                    if let Some(item) = item {
                        data.push(item.clone());
                    }
                    i = &i + &SifrInt::from_i64(1);
                }
            }
            let __sifr_field_init_0: Option<SifrInt> = maxlen.clone();
            let __sifr_field_init_1: VecDeque<T> = VecDeque::from(data);
            Self {
                maxlen: __sifr_field_init_0,
                _data: __sifr_field_init_1,
            }
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn append(&mut self, val: &T) {
            self._data.push_back(val.clone());
            let maxlen_opt: Option<SifrInt> = self.maxlen.clone();
            if let Some(maxlen_opt) = maxlen_opt.clone() {
                let maxlen: SifrInt = maxlen_opt.clone();
                if (&SifrInt::from(self._data.len()) > &maxlen) {
                    self._data.pop_front();
                }
            }
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn appendleft(&mut self, val: &T) {
            self._data.push_front(val.clone());
            let maxlen_opt: Option<SifrInt> = self.maxlen.clone();
            if let Some(maxlen_opt) = maxlen_opt.clone() {
                let maxlen: SifrInt = maxlen_opt.clone();
                if (&SifrInt::from(self._data.len()) > &maxlen) {
                    self._data.pop_back();
                }
            }
        }
    }
    impl<T> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn pop(&mut self) -> Option<T> {
            if (&SifrInt::from(self._data.len()) == &SifrInt::from_i64(0)) {
                return None;
            }
            Some({
                let __sifr_nonempty_pop_index = self._data.len() - (1_usize);
                let mut __sifr_nonempty_pop_values = self
                    ._data
                    .drain(__sifr_nonempty_pop_index..__sifr_nonempty_pop_index + (1_usize))
                    .collect::<Vec<_>>();
                __sifr_nonempty_pop_values.remove(0_usize)
            })
        }
    }
    impl<T> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn popleft(&mut self) -> Option<T> {
            if (&SifrInt::from(self._data.len()) == &SifrInt::from_i64(0)) {
                return None;
            }
            Some({
                let __sifr_nonempty_pop_index = 0_usize;
                let mut __sifr_nonempty_pop_values = self
                    ._data
                    .drain(__sifr_nonempty_pop_index..__sifr_nonempty_pop_index + (1_usize))
                    .collect::<Vec<_>>();
                __sifr_nonempty_pop_values.remove(0_usize)
            })
        }
    }
    impl<T> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn len(&self) -> SifrInt {
            SifrInt::from(self._data.len())
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn to_list(&self) -> Vec<T> {
            let mut result: Vec<T> = vec![];
            for v in self._data.clone().iter().cloned() {
                result.push(v.clone());
            }
            result
        }
    }
    impl<T> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn clear(&mut self) {
            self._data.clear();
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn extend(&mut self, items: &Vec<T>) {
            for v in items.iter().cloned() {
                self._data.push_back(v.clone());
            }
            let maxlen_opt: Option<SifrInt> = self.maxlen.clone();
            if let Some(maxlen_opt) = maxlen_opt.clone() {
                let maxlen: SifrInt = maxlen_opt.clone();
                while (&SifrInt::from(self._data.len()) > &maxlen) {
                    self._data.pop_front();
                }
            }
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn extendleft(&mut self, items: &Vec<T>) {
            for v in items.iter().cloned() {
                self._data.push_front(v.clone());
            }
            let maxlen_opt: Option<SifrInt> = self.maxlen.clone();
            if let Some(maxlen_opt) = maxlen_opt.clone() {
                let maxlen: SifrInt = maxlen_opt.clone();
                while (&SifrInt::from(self._data.len()) > &maxlen) {
                    self._data.pop_back();
                }
            }
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn copy(&self) -> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
            __SifrStdlib_sifr_x2ecollections_x2edeque::new(
                Some(self.to_list()),
                self.maxlen.clone(),
            )
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn reverse(&mut self) {
            let mut items: Vec<T> = self.to_list();
            items.reverse();
            self._data.clear();
            for item in items.iter().cloned() {
                self._data.push_back(item.clone());
            }
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn rotate(&mut self, n: &SifrInt) {
            let length: SifrInt = SifrInt::from(self._data.len());
            if &length == &SifrInt::from_i64(0) {
                return;
            }
            let mut steps: SifrInt = n.floor_mod_known_nonzero(&length);
            if &steps < &SifrInt::from_i64(0) {
                steps = &steps + &length;
            }
            let mut count: SifrInt = SifrInt::from_i64(0);
            while &count < &steps {
                let value: Option<T> = self._data.pop_back();
                if let Some(value) = value {
                    self._data.push_front(value.clone());
                }
                count = &count + &SifrInt::from_i64(1);
            }
        }
    }
    impl<T: Clone + PartialEq> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn count(&self, value: &T) -> SifrInt {
            let mut total: SifrInt = SifrInt::from_i64(0);
            for item in self._data.clone().iter().cloned() {
                if item == *value {
                    total = &total + &SifrInt::from_i64(1);
                }
            }
            total
        }
    }
    impl<T: Clone + PartialEq> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn index(
            &self,
            value: &T,
            start: &SifrInt,
            stop: &Option<SifrInt>,
        ) -> Option<SifrInt> {
            let size: SifrInt = SifrInt::from(self._data.len());
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
                    end = size;
                }
            }
            let mut i: SifrInt = begin.clone();
            while &i < &end {
                let current: Option<T> = {
                    let __sifr_index_list = &self._data;
                    let __sifr_index_i = i.clone();
                    let __sifr_index_norm = __sifr_index_i
                        .normalize_index_or_len(__sifr_index_list.len());
                    __sifr_index_list.get(__sifr_index_norm).cloned()
                };
                if let Some(current) = current {
                    if current == *value {
                        return Some(i);
                    }
                }
                i = &i + &SifrInt::from_i64(1);
            }
            None
        }
    }
    impl<T: Clone + PartialEq> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn remove(&mut self, value: &T) {
            let idx: Option<SifrInt> = self.index(value, &SifrInt::from_i64(0), &None);
            if let Some(idx) = idx.clone() {
                let mut rebuilt: Vec<T> = vec![];
                let mut i: SifrInt = SifrInt::from_i64(0);
                while (&i < &SifrInt::from(self._data.len())) {
                    let current: Option<T> = Some(
                        self._data.clone()[::sifr_runtime::to_usize_proven(&(i))].clone(),
                    );
                    if let Some(current) = current {
                        if &i != &idx {
                            rebuilt.push(current.clone());
                        }
                    }
                    i = &i + &SifrInt::from_i64(1);
                }
                self._data.clear();
                for item in rebuilt.iter().cloned() {
                    self._data.push_back(item.clone());
                }
            }
        }
    }
}
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ecollections_x2eCounter;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ecollections_x2edeque;
use ::std::collections::HashMap;
use ::std::collections::VecDeque;
use ::sifr_runtime::SifrInt;
fn bisect_right<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    a: &Vec<T>,
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
    } else {
        if let Some(hi) = hi.clone() {
            if &hi < &SifrInt::from_i64(0) {
                right = SifrInt::from_i64(0);
            } else {
                if (&hi > &SifrInt::from(a.len())) {
                    right = SifrInt::from(a.len());
                } else {
                    right = hi;
                }
            }
        }
    }
    while &left < &right {
        let mid: SifrInt = (&left + &right)
            .floor_div_known_nonzero(&SifrInt::from_i64(2));
        let val: Option<T> = {
            let __sifr_index_list = &a;
            let __sifr_index_i = mid.clone();
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
            __sifr_index_list.get(__sifr_index_norm).cloned()
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
fn insort_right<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    a: &mut Vec<T>,
    x: &T,
    lo: SifrInt,
    hi: Option<SifrInt>,
) {
    let pos: SifrInt = bisect_right(a, x, (lo).clone(), (hi).clone());
    a.insert(::sifr_runtime::to_usize_proven(&pos), x.clone());
}
fn from_list<
    T: Clone + ::std::fmt::Display + PartialOrd + ::std::hash::Hash + Eq + 'static,
>(items: &Vec<T>) -> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    let mut counts: HashMap<T, SifrInt> = HashMap::from([]);
    for item in items.iter().cloned() {
        let val: Option<SifrInt> = counts.get(&item).cloned();
        if let Some(val) = val.clone() {
            counts.insert(item.clone(), &val + &SifrInt::from_i64(1));
        } else {
            counts.insert(item.clone(), SifrInt::from_i64(1));
        }
    }
    __SifrStdlib_sifr_x2ecollections_x2eCounter::new(Some(counts), None)
}
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
fn heapify<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(data: &mut Vec<T>) {
    "Convert list to a min-heap in-place. O(n) time.".to_string();
    let n: SifrInt = SifrInt::from(data.len());
    let mut i: SifrInt = &n.floor_div_known_nonzero(&SifrInt::from_i64(2))
        - &SifrInt::from_i64(1);
    while &i >= &SifrInt::from_i64(0) {
        _sift_down(data, (i).clone(), (n).clone());
        i = &i - &SifrInt::from_i64(1);
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
fn heapreplace<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
    item: T,
) -> Option<T> {
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
    _sift_down(heap, SifrInt::from_i64(0), (heap_len).clone());
    top
}
fn heappushpop<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
    item: &T,
) -> Option<T> {
    heappush(heap, item);
    heappop(heap)
}
fn main() {
    let counts: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(
        &vec![
            "delta".to_string(), "alpha".to_string(), "delta".to_string(), "beta"
            .to_string()
        ],
    );
    println!("{:?}", counts.most_common(& None));
    let mut queue: __SifrStdlib_sifr_x2ecollections_x2edeque<SifrInt> = __SifrStdlib_sifr_x2ecollections_x2edeque::new(
        Some(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)]),
        Some(SifrInt::from_i64(4)),
    );
    queue.rotate(&SifrInt::from_i64(1));
    queue.appendleft(&SifrInt::from_i64(0));
    println!("{:?}", queue.to_list());
    let mut ordered: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(3), SifrInt::from_i64(5)
    ];
    insort_right(&mut ordered, &SifrInt::from_i64(4), SifrInt::from_i64(0), None);
    println!(
        "{}", bisect_right(& ordered, & SifrInt::from_i64(4), SifrInt::from_i64(0), None)
    );
    let mut heap: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(3), SifrInt::from_i64(5)
    ];
    heapify(&mut heap);
    println!(
        "{}", (heappushpop(& mut heap, & SifrInt::from_i64(2))).map_or("None".to_string()
        .to_string(), | __v | format!("{}", __v))
    );
    println!(
        "{}", (heapreplace(& mut heap, SifrInt::from_i64(4))).map_or("None".to_string()
        .to_string(), | __v | format!("{}", __v))
    );
}
