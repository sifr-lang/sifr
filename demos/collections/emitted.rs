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
                        {
                            let __assign_value = value.clone();
                            {
                                let __assign_key = key.clone();
                                counts.insert(__assign_key, __assign_value);
                            }
                        }
                    }
                }
            }
            if let Some(iterable) = iterable {
                for item in iterable.iter().cloned() {
                    let value2: Option<SifrInt> = counts.get(&item).cloned();
                    if let Some(value2) = value2.clone() {
                        {
                            let __assign_value = &value2 + &SifrInt::from_i64(1);
                            {
                                let __assign_key = item.clone();
                                counts.insert(__assign_key, __assign_value);
                            }
                        }
                    } else {
                        {
                            let __assign_value = SifrInt::from_i64(1);
                            {
                                let __assign_key = item.clone();
                                counts.insert(__assign_key, __assign_value);
                            }
                        }
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
            let val: Option<SifrInt> = self.counts.get(key).cloned();
            if let Some(val) = val.clone() {
                return val;
            }
            SifrInt::from_i64(0)
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn get(&self, key: &T, default: &SifrInt) -> SifrInt {
            let val: Option<SifrInt> = self.counts.get(key).cloned();
            if let Some(val) = val.clone() {
                return val;
            }
            default.clone()
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn increment(&mut self, key: &T) {
            let val: Option<SifrInt> = self.counts.get(key).cloned();
            if let Some(val) = val.clone() {
                {
                    let __assign_value = &val + &SifrInt::from_i64(1);
                    {
                        let __assign_key = key.clone();
                        self.counts.insert(__assign_key, __assign_value);
                    }
                }
            } else {
                {
                    let __assign_value = SifrInt::from_i64(1);
                    {
                        let __assign_key = key.clone();
                        self.counts.insert(__assign_key, __assign_value);
                    }
                }
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
            let mut i: SifrInt = SifrInt::from_i64(0);
            while (&SifrInt::from_i64(0) <= &i) && (&i < &SifrInt::from(result.len())) {
                let mut j: SifrInt = &i + &SifrInt::from_i64(1);
                while (&SifrInt::from_i64(0) <= &j) && (&j < &SifrInt::from(result.len())) {
                    let left: Option<(T, SifrInt)> = {
                        let __sifr_checked_read_collection = &result;
                        let __sifr_checked_read_index = i.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(__sifr_checked_read_collection.len());
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    let right: Option<(T, SifrInt)> = {
                        let __sifr_checked_read_collection = &result;
                        let __sifr_checked_read_index = j.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(__sifr_checked_read_collection.len());
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(left) = left {
                        if let Some(right) = right {
                            if ((right).1.clone() > (left).1.clone()) {
                                {
                                    let __assign_value = right.clone();
                                    {
                                        let __index_raw = i.clone();
                                        let __index_normalized = __index_raw
                                            .normalize_index_or_len(result.len());
                                        if let Some(__elem) = result.get_mut(__index_normalized) {
                                            *__elem = __assign_value;
                                        }
                                    }
                                }
                                {
                                    let __assign_value = left.clone();
                                    {
                                        let __index_raw = j.clone();
                                        let __index_normalized = __index_raw
                                            .normalize_index_or_len(result.len());
                                        if let Some(__elem) = result.get_mut(__index_normalized) {
                                            *__elem = __assign_value;
                                        }
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
                let value: Option<(T, SifrInt)> = {
                    let __sifr_checked_read_collection = &result;
                    let __sifr_checked_read_index = index.clone();
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
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
    impl<
        T: ::std::hash::Hash + Eq + Clone + PartialOrd,
    > __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn keys(&self) -> Vec<T> {
            let mut result: Vec<T> = self.counts.keys().cloned().collect::<Vec<_>>();
            let mut i: SifrInt = SifrInt::from_i64(0);
            while (&SifrInt::from_i64(0) <= &i) && (&i < &SifrInt::from(result.len())) {
                let mut j: SifrInt = &i + &SifrInt::from_i64(1);
                while (&SifrInt::from_i64(0) <= &j) && (&j < &SifrInt::from(result.len())) {
                    let left: Option<T> = {
                        let __sifr_checked_read_collection = &result;
                        let __sifr_checked_read_index = i.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(__sifr_checked_read_collection.len());
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    let right: Option<T> = {
                        let __sifr_checked_read_collection = &result;
                        let __sifr_checked_read_index = j.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(__sifr_checked_read_collection.len());
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(left) = left {
                        if let Some(right) = right {
                            if (right < left) {
                                {
                                    let __assign_value = right.clone();
                                    {
                                        let __index_raw = i.clone();
                                        let __index_normalized = __index_raw
                                            .normalize_index_or_len(result.len());
                                        if let Some(__elem) = result.get_mut(__index_normalized) {
                                            *__elem = __assign_value;
                                        }
                                    }
                                }
                                {
                                    let __assign_value = left.clone();
                                    {
                                        let __index_raw = j.clone();
                                        let __index_normalized = __index_raw
                                            .normalize_index_or_len(result.len());
                                        if let Some(__elem) = result.get_mut(__index_normalized) {
                                            *__elem = __assign_value;
                                        }
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
                        {
                            let __assign_value = &existing + &other_val;
                            {
                                let __assign_key = key.clone();
                                self.counts.insert(__assign_key, __assign_value);
                            }
                        }
                    } else {
                        {
                            let __assign_value = other_val.clone();
                            {
                                let __assign_key = key.clone();
                                self.counts.insert(__assign_key, __assign_value);
                            }
                        }
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
                        {
                            let __assign_value = &existing - &other_val;
                            {
                                let __assign_key = key.clone();
                                self.counts.insert(__assign_key, __assign_value);
                            }
                        }
                    } else {
                        {
                            let __assign_value = &SifrInt::from_i64(0) - &other_val;
                            {
                                let __assign_key = key.clone();
                                self.counts.insert(__assign_key, __assign_value);
                            }
                        }
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
                let key_opt: Option<T> = {
                    let __sifr_checked_read_collection = &all_keys;
                    let __sifr_checked_read_index = ki.clone();
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                };
                if let Some(key_opt) = key_opt {
                    let cnt: Option<SifrInt> = self.counts.get(&key_opt).cloned();
                    if let Some(cnt) = cnt.clone() {
                        let mut i: SifrInt = SifrInt::from_i64(0);
                        while (&i < &cnt) {
                            let key_copy: Option<T> = {
                                let __sifr_checked_read_collection = &all_keys;
                                let __sifr_checked_read_index = ki.clone();
                                let __sifr_checked_read_normalized = __sifr_checked_read_index
                                    .normalize_index_or_len(
                                        __sifr_checked_read_collection.len(),
                                    );
                                __sifr_checked_read_collection
                                    .get(__sifr_checked_read_normalized)
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
                        {
                            let __assign_value = total.clone();
                            {
                                let __assign_key = key.clone();
                                new_counts.insert(__assign_key, __assign_value);
                            }
                        }
                    }
                }
            }
            for key2 in Box::new(
                (other.counts.keys().cloned().collect::<Vec<_>>()).into_iter(),
            ) {
                let already: Option<SifrInt> = new_counts.get(&key2).cloned();
                if already.is_none() {
                    let b_val2: Option<SifrInt> = other.counts.get(&key2).cloned();
                    if let Some(b_val2) = b_val2 {
                        if &b_val2 > &SifrInt::from_i64(0) {
                            {
                                let __assign_value = b_val2.clone();
                                {
                                    let __assign_key = key2.clone();
                                    new_counts.insert(__assign_key, __assign_value);
                                }
                            }
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
                        {
                            let __assign_value = diff.clone();
                            {
                                let __assign_key = key.clone();
                                new_counts.insert(__assign_key, __assign_value);
                            }
                        }
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
                    let item: Option<T> = {
                        let __sifr_checked_read_collection = &items;
                        let __sifr_checked_read_index = i.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(__sifr_checked_read_collection.len());
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
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
            while (&count < &steps) {
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
                if (&end < &SifrInt::from_i64(0)) {
                    end = &size + &end;
                }
                if (&end < &SifrInt::from_i64(0)) {
                    end = SifrInt::from_i64(0);
                }
                if (&end > &size) {
                    end = size;
                }
            }
            let mut i: SifrInt = begin.clone();
            while (&i < &end) {
                let current: Option<T> = {
                    let __sifr_checked_read_collection = &self._data;
                    let __sifr_checked_read_index = i.clone();
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
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
                    let current: Option<T> = {
                        let __sifr_checked_read_collection = &self._data;
                        let __sifr_checked_read_index = i.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(__sifr_checked_read_collection.len());
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(current) = current {
                        if (&i != &idx) {
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
use ::std::collections::HashSet;
use ::std::collections::VecDeque;
use ::sifr_runtime::SifrInt;
fn from_list<T: Clone + ::std::hash::Hash + Eq + 'static>(
    items: &Vec<T>,
) -> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    let mut counts: HashMap<T, SifrInt> = HashMap::from([]);
    for item in items.iter().cloned() {
        let val: Option<SifrInt> = counts.get(&item).cloned();
        if let Some(val) = val.clone() {
            {
                let __assign_value = &val + &SifrInt::from_i64(1);
                {
                    let __assign_key = item.clone();
                    counts.insert(__assign_key, __assign_value);
                }
            }
        } else {
            {
                let __assign_value = SifrInt::from_i64(1);
                {
                    let __assign_key = item.clone();
                    counts.insert(__assign_key, __assign_value);
                }
            }
        }
    }
    __SifrStdlib_sifr_x2ecollections_x2eCounter::new(Some(counts), None)
}
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
        assert!(
            ({ let __sifr_condition_list = & actual; let __sifr_condition_index = i
            .clone(); let __sifr_condition_normalized = __sifr_condition_index
            .normalize_index_or_len(__sifr_condition_list.len()); __sifr_condition_list
            .get(__sifr_condition_normalized).copied() }) == ({ let __sifr_condition_list
            = & expected; let __sifr_condition_index = i.clone(); let
            __sifr_condition_normalized = __sifr_condition_index
            .normalize_index_or_len(__sifr_condition_list.len()); __sifr_condition_list
            .get(__sifr_condition_normalized).copied() })
        );
        i = &i + &SifrInt::from_i64(1);
    }
}
fn collect_set_and_counter_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let left: HashSet<SifrInt> = (vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)
    ])
        .into_iter()
        .collect::<std::collections::HashSet<_>>();
    let right: HashSet<SifrInt> = (vec![
        SifrInt::from_i64(3), SifrInt::from_i64(4), SifrInt::from_i64(5)
    ])
        .into_iter()
        .collect::<std::collections::HashSet<_>>();
    actual
        .push(
            &SifrInt::from(
                left
                    .union(&right)
                    .cloned()
                    .collect::<std::collections::HashSet<_>>()
                    .len(),
            ) == &SifrInt::from_i64(5),
        );
    actual
        .push(
            &SifrInt::from(
                left
                    .intersection(&right)
                    .cloned()
                    .collect::<std::collections::HashSet<_>>()
                    .len(),
            ) == &SifrInt::from_i64(1),
        );
    let counts: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(
        &vec![
            "x".to_string(), "y".to_string(), "x".to_string(), "z".to_string(), "x"
            .to_string(), "y".to_string()
        ],
    );
    actual
        .push(
            &counts.get(&"x".to_string(), &SifrInt::from_i64(0)) == &SifrInt::from_i64(3),
        );
    actual
        .push(
            (format!("{:?}", counts.most_common(& Some((SifrInt::from_i64(2)).clone()))))
                .as_str() == ("[(\"x\", 3), (\"y\", 2)]".to_string()).as_str(),
        );
    actual
}
fn collect_deque_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut d: __SifrStdlib_sifr_x2ecollections_x2edeque<SifrInt> = __SifrStdlib_sifr_x2ecollections_x2edeque::new(
        None,
        Some(SifrInt::from_i64(2)),
    );
    d.append(&SifrInt::from_i64(10));
    d.append(&SifrInt::from_i64(20));
    d.append(&SifrInt::from_i64(30));
    actual
        .push(
            (&SifrInt::from(d.len()) == &SifrInt::from_i64(2))
                && (d.popleft() == Some(SifrInt::from_i64(20))),
        );
    let _ = d.pop();
    actual.push(d.pop() == None);
    actual
}
fn append_all(target: &mut Vec<bool>, values: &Vec<bool>) {
    for value in values.iter().copied() {
        target.push(value);
    }
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let mut actual: Vec<bool> = vec![];
    append_all(&mut actual, &collect_set_and_counter_actual());
    append_all(&mut actual, &collect_deque_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("collections collections parity demo: pass");
}
