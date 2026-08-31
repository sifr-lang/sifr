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
use ::std::collections::VecDeque;
use ::sifr_runtime::SifrInt;
fn from_list<
    T: Clone + ::std::fmt::Display + PartialOrd + ::std::hash::Hash + Eq + 'static,
>(items: &Vec<T>) -> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
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
fn reduce<
    T: Clone + ::std::fmt::Display + PartialOrd + 'static,
    U: Clone + ::std::fmt::Display + PartialOrd + 'static,
>(func: impl Fn(&U, &T) -> U, data: &Vec<T>, initial: &U) -> U {
    let mut result: U = (initial).clone();
    for val in data.iter().cloned() {
        result = func(&result, &val);
    }
    result
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
        if (&left < &n) {
            let s_val: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = smallest.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            let l_val: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = left.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(s_val) = s_val {
                if let Some(l_val) = l_val {
                    if (l_val < s_val) {
                        smallest = left;
                    }
                }
            }
        }
        if (&right < &n) {
            let s_val2: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = smallest.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            let r_val: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = right.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(s_val2) = s_val2 {
                if let Some(r_val) = r_val {
                    if (r_val < s_val2) {
                        smallest = right;
                    }
                }
            }
        }
        if (&smallest == &pos) {
            done = true;
        } else {
            let tmp_pos: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = pos.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            let tmp_sm: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = smallest.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(tmp_pos) = tmp_pos {
                if let Some(tmp_sm) = tmp_sm {
                    if (&SifrInt::from_i64(0) <= &pos)
                        && (&pos < &SifrInt::from(data.len()))
                    {
                        {
                            let __assign_value = tmp_sm.clone();
                            {
                                let __index_raw = pos.clone();
                                let __index_normalized = __index_raw
                                    .normalize_index_or_len(data.len());
                                if let Some(__elem) = data.get_mut(__index_normalized) {
                                    *__elem = __assign_value;
                                }
                            }
                        }
                    }
                    if (&SifrInt::from_i64(0) <= &smallest)
                        && (&smallest < &SifrInt::from(data.len()))
                    {
                        {
                            let __assign_value = tmp_pos.clone();
                            {
                                let __index_raw = smallest.clone();
                                let __index_normalized = __index_raw
                                    .normalize_index_or_len(data.len());
                                if let Some(__elem) = data.get_mut(__index_normalized) {
                                    *__elem = __assign_value;
                                }
                            }
                        }
                    }
                }
            }
            pos = smallest;
        }
    }
}
fn heapify<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(data: &mut Vec<T>) {
    "Convert list to a min-heap in-place. O(n) time.".to_string();
    let n: SifrInt = SifrInt::from(data.len());
    let mut i: SifrInt = &n.floor_div_known_nonzero(&SifrInt::from_i64(2))
        - &SifrInt::from_i64(1);
    while (&i >= &SifrInt::from_i64(0)) {
        _sift_down(data, (i).clone(), (n).clone());
        i = &i - &SifrInt::from_i64(1);
    }
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
    let top: Option<T> = {
        let __sifr_checked_read_collection = &heap;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let last: Option<T> = {
        let __sifr_checked_read_collection = &heap;
        let __sifr_checked_read_index = &n - &SifrInt::from_i64(1);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    heap.remove(heap.len() - (1_usize));
    let n2: SifrInt = SifrInt::from(heap.len());
    if (&n2 > &SifrInt::from_i64(0)) {
        if let Some(last) = last {
            {
                let __assign_value = last.clone();
                {
                    let __index_raw = SifrInt::from_i64(0);
                    let __index_normalized = __index_raw
                        .normalize_index_or_len(heap.len());
                    if let Some(__elem) = heap.get_mut(__index_normalized) {
                        *__elem = __assign_value;
                    }
                }
            }
        }
        _sift_down(heap, SifrInt::from_i64(0), (n2).clone());
    }
    top
}
fn nsmallest<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    n: SifrInt,
    data: &Vec<T>,
) -> Vec<T> {
    let mut heap: Vec<T> = data.clone();
    heapify(&mut heap);
    let mut result: Vec<T> = vec![];
    let mut count: SifrInt = SifrInt::from_i64(0);
    while (&count < &n) {
        if (&SifrInt::from(heap.len()) == &SifrInt::from_i64(0)) {
            return result;
        }
        let val: Option<T> = heappop(&mut heap);
        if let Some(val) = val {
            result.push(val.clone());
        }
        count = &count + &SifrInt::from_i64(1);
    }
    result
}
fn nlargest<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    n: SifrInt,
    data: &Vec<T>,
) -> Vec<T> {
    if &n <= &SifrInt::from_i64(0) {
        return vec![];
    }
    if (&n >= &SifrInt::from(data.len())) {
        let mut result: Vec<T> = vec![];
        for val in data.iter().cloned() {
            result.push(val.clone());
        }
        return result;
    }
    let mut heap: Vec<T> = data.clone();
    heapify(&mut heap);
    let mut all_sorted: Vec<T> = vec![];
    while (&SifrInt::from(heap.len()) > &SifrInt::from_i64(0)) {
        let val2: Option<T> = heappop(&mut heap);
        if let Some(val2) = val2 {
            all_sorted.push(val2.clone());
        }
    }
    let mut result2: Vec<T> = vec![];
    let mut i: SifrInt = &SifrInt::from(all_sorted.len()) - &SifrInt::from_i64(1);
    let mut count: SifrInt = SifrInt::from_i64(0);
    while (&count < &n) {
        if (&i < &SifrInt::from_i64(0)) {
            return result2;
        }
        let v: Option<T> = {
            let __sifr_checked_read_collection = &all_sorted;
            let __sifr_checked_read_index = i.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(v) = v {
            result2.push(v.clone());
        }
        i = &i - &SifrInt::from_i64(1);
        count = &count + &SifrInt::from_i64(1);
    }
    result2
}
struct __SifrYielder<T> {
    slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
}
struct __SifrYieldFuture<T> {
    slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    value: Option<T>,
}
impl<T> Unpin for __SifrYieldFuture<T> {}
impl<T> ::std::future::Future for __SifrYieldFuture<T> {
    type Output = ();
    fn poll(
        self: ::std::pin::Pin<&mut Self>,
        _cx: &mut ::std::task::Context<'_>,
    ) -> ::std::task::Poll<()> {
        let state = self.get_mut();
        let Some(value) = state.value.take() else {
            return ::std::task::Poll::Ready(());
        };
        __sifr_store_suspended(&state.slot, value);
        ::std::task::Poll::Pending
    }
}
impl<T> __SifrYielder<T> {
    fn suspend(&self, value: T) -> __SifrYieldFuture<T> {
        __SifrYieldFuture {
            slot: ::std::sync::Arc::clone(&self.slot),
            value: Some(value),
        }
    }
}
fn __sifr_store_suspended<T>(
    slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    value: T,
) {
    match slot.lock() {
        Ok(mut state) => *state = Some(value),
        Err(poisoned) => *poisoned.into_inner() = Some(value),
    }
}
fn __sifr_take_suspended<T>(
    slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
) -> Option<T> {
    match slot.lock() {
        Ok(mut state) => state.take(),
        Err(poisoned) => poisoned.into_inner().take(),
    }
}
struct __SifrGenerator<T> {
    producer: Option<
        ::std::pin::Pin<Box<dyn ::std::future::Future<Output = ()> + 'static>>,
    >,
    yielded: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    complete: bool,
}
impl<T> __SifrGenerator<T> {
    fn new<
        F: FnOnce(__SifrYielder<T>) -> Fut + 'static,
        Fut: ::std::future::Future<Output = ()> + 'static,
    >(factory: F) -> Self {
        let yielded = ::std::sync::Arc::new(::std::sync::Mutex::new(None));
        let producer = factory(__SifrYielder {
            slot: ::std::sync::Arc::clone(&yielded),
        });
        Self {
            producer: Some(Box::pin(producer)),
            yielded,
            complete: false,
        }
    }
}
impl<T> Iterator for __SifrGenerator<T> {
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
            let mut context = ::std::task::Context::from_waker(
                ::std::task::Waker::noop(),
            );
            ::std::future::Future::poll(producer.as_mut(), &mut context).is_ready()
        };
        let yielded = __sifr_take_suspended(&self.yielded);
        if completed {
            self.complete = true;
            self.producer = None;
        }
        yielded
    }
}
fn chain<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    iterables: &Vec<Vec<T>>,
) -> Box<dyn Iterator<Item = T>> {
    let iterables = iterables.clone();
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            for iterable in iterables.iter().cloned() {
                for item in iterable.iter().cloned() {
                    __sifr_yielder.suspend(item.clone()).await;
                }
            }
        }),
    )
}
fn repeat<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    value: T,
    times: SifrInt,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            let holder: Vec<T> = vec![value.clone()];
            let mut i: SifrInt = SifrInt::from_i64(0);
            while (&i < &times) {
                if (&SifrInt::from(holder.len()) > &SifrInt::from_i64(0)) {
                    let current: Option<T> = {
                        let __sifr_checked_read_collection = &holder;
                        let __sifr_checked_read_index = SifrInt::from_i64(0);
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(
                                __sifr_checked_read_collection.len(),
                            );
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(current) = current {
                        __sifr_yielder.suspend(current.clone()).await;
                    }
                }
                i = &i + &SifrInt::from_i64(1);
            }
        }),
    )
}
fn take<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    n: SifrInt,
    data: &Vec<T>,
) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    let mut count: SifrInt = SifrInt::from_i64(0);
    for item in data.iter().cloned() {
        if (&count >= &n) {
            return result;
        }
        result.push(item.clone());
        count = &count + &SifrInt::from_i64(1);
    }
    result
}
fn flatten<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    lists: &Vec<Vec<T>>,
) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    for inner in lists.iter().cloned() {
        for val in inner.iter().cloned() {
            result.push(val.clone());
        }
    }
    result
}
fn pairwise<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = vec![];
    let mut prev_values: Vec<T> = vec![];
    for value in data.iter().cloned() {
        if (&SifrInt::from(prev_values.len()) > &SifrInt::from_i64(0)) {
            let mut pair: Vec<T> = vec![];
            let prev: Option<T> = {
                let __sifr_checked_read_collection = &prev_values;
                let __sifr_checked_read_index = SifrInt::from_i64(0);
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(prev) = prev {
                pair.push(prev.clone());
            }
            pair.push(value.clone());
            result.push(pair.clone());
            let __sifr_try_res: Result<(), IndexError> = (|| {
                {
                    let __assign_value = value.clone();
                    {
                        let __index_raw = SifrInt::from_i64(0);
                        let __index_normalized = __index_raw
                            .normalize_index_or_len(prev_values.len());
                        if let Some(__elem) = prev_values.get_mut(__index_normalized) {
                            *__elem = __assign_value;
                        }
                    }
                }
                Ok(())
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let _e = __sifr_try_err.clone();
                return result;
            }
        } else {
            prev_values.push(value.clone());
        }
    }
    result
}
fn _islice_impl<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    start: SifrInt,
    stop: SifrInt,
    step: SifrInt,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            let mut index: SifrInt = SifrInt::from_i64(0);
            let mut next_yield: SifrInt = start.clone();
            for value in data {
                if &index >= &stop {
                    return;
                }
                if &index == &next_yield {
                    __sifr_yielder.suspend(value.clone()).await;
                    next_yield = &next_yield + &step;
                }
                index = &index + &SifrInt::from_i64(1);
            }
        }),
    )
}
fn islice<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    start_or_stop: SifrInt,
    stop: Option<SifrInt>,
    step: SifrInt,
) -> Result<Box<dyn Iterator<Item = T>>, ValueError> {
    let mut actual_start: SifrInt = SifrInt::from_i64(0);
    let mut actual_stop: SifrInt = start_or_stop.clone();
    if let Some(stop) = stop.clone() {
        actual_start = start_or_stop.clone();
        actual_stop = stop.clone();
    }
    if (&actual_start < &SifrInt::from_i64(0)) || (&actual_stop < &SifrInt::from_i64(0))
    {
        return Err(ValueError::new("islice: indices must be non-negative".to_string()));
    }
    if (&step <= &SifrInt::from_i64(0)) {
        return Err(
            ValueError::new("islice: step must be greater than zero".to_string()),
        );
    }
    Ok(
        _islice_impl(
            Box::new(data),
            (actual_start).clone(),
            (actual_stop).clone(),
            (step).clone(),
        ),
    )
}
fn accumulate<
    T: Clone + ::std::fmt::Display + PartialOrd + 'static + ::std::ops::Add<Output = T>,
>(data: Box<dyn Iterator<Item = T>>, initial: Option<T>) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            let mut state: Vec<T> = vec![];
            if let Some(initial) = initial {
                state.push(initial.clone());
                let initial_value: Option<T> = {
                    let __sifr_checked_read_collection = &state;
                    let __sifr_checked_read_index = SifrInt::from_i64(0);
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                };
                if let Some(initial_value) = initial_value {
                    __sifr_yielder.suspend(initial_value.clone()).await;
                }
            }
            for item in data {
                if (&SifrInt::from(state.len()) == &SifrInt::from_i64(0)) {
                    state.push(item.clone());
                } else {
                    let prev: Option<T> = {
                        let __sifr_checked_read_collection = &state;
                        let __sifr_checked_read_index = SifrInt::from_i64(0);
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(
                                __sifr_checked_read_collection.len(),
                            );
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(prev) = prev {
                        let next_val: T = prev + item;
                        let __sifr_try_res: Result<(), IndexError> = (|| {
                            {
                                let __assign_value = next_val.clone();
                                {
                                    let __index_raw = SifrInt::from_i64(0);
                                    let __index_normalized = __index_raw
                                        .normalize_index_or_len(state.len());
                                    if let Some(__elem) = state.get_mut(__index_normalized) {
                                        *__elem = __assign_value;
                                    } else {
                                        return Err(
                                            IndexError::new("collection index out of range".to_string()),
                                        );
                                    }
                                }
                            }
                            Ok(())
                        })();
                        if let Err(__sifr_try_err) = __sifr_try_res {
                            let _e = __sifr_try_err.clone();
                            return;
                        }
                    }
                }
                let current: Option<T> = {
                    let __sifr_checked_read_collection = &state;
                    let __sifr_checked_read_index = SifrInt::from_i64(0);
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                };
                if let Some(current) = current {
                    __sifr_yielder.suspend(current.clone()).await;
                }
            }
        }),
    )
}
fn compress<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    selectors: Box<dyn Iterator<Item = bool>>,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            for (value, selector) in Box::new(
                data.zip(selectors).map(|__zip_item| (__zip_item.0, __zip_item.1)),
            ) {
                if selector {
                    __sifr_yielder.suspend(value.clone()).await;
                }
            }
        }),
    )
}
fn dropwhile<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool + Send + Sync + 'static,
    data: Box<dyn Iterator<Item = T>>,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            let mut dropping: bool = true;
            for val in data {
                if dropping {
                    if !pred(&val) {
                        dropping = false;
                        __sifr_yielder.suspend(val.clone()).await;
                    }
                } else {
                    __sifr_yielder.suspend(val.clone()).await;
                }
            }
        }),
    )
}
fn takewhile<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool + Send + Sync + 'static,
    data: Box<dyn Iterator<Item = T>>,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            for val in data {
                if !pred(&val) {
                    return;
                }
                __sifr_yielder.suspend(val.clone()).await;
            }
        }),
    )
}
fn filterfalse<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool + Send + Sync + 'static,
    data: Box<dyn Iterator<Item = T>>,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            for val in data {
                if !pred(&val) {
                    __sifr_yielder.suspend(val.clone()).await;
                }
            }
        }),
    )
}
fn zip_longest<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    a: Box<dyn Iterator<Item = T>>,
    b: Box<dyn Iterator<Item = T>>,
    fill: &T,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    let fill = fill.clone();
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<Vec<T>>| {
            let mut left: Box<dyn Iterator<Item = T>> = a;
            let mut right: Box<dyn Iterator<Item = T>> = b;
            loop {
                let left_value: Option<T> = left.next();
                let right_value: Option<T> = right.next();
                if (left_value == None) && (right_value == None) {
                    return;
                }
                let mut pair: Vec<T> = vec![];
                if let Some(left_value) = left_value {
                    pair.push(left_value.clone());
                } else {
                    pair.push(fill.clone());
                }
                if let Some(right_value) = right_value {
                    pair.push(right_value.clone());
                } else {
                    pair.push(fill.clone());
                }
                __sifr_yielder.suspend(pair.clone()).await;
            }
        }),
    )
}
fn cycle<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    n: SifrInt,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            let mut saved: Vec<T> = vec![];
            let mut emitted: SifrInt = SifrInt::from_i64(0);
            for value in data {
                if (&emitted >= &n) {
                    return;
                }
                saved.push(value.clone());
                let current: Option<T> = {
                    let __sifr_index_list = &saved;
                    let __sifr_index_i = SifrInt::from(saved.len())
                        - SifrInt::from_i64(1);
                    let __sifr_index_norm = __sifr_index_i
                        .normalize_index_or_len(__sifr_index_list.len());
                    __sifr_index_list.get(__sifr_index_norm).cloned()
                };
                if let Some(current) = current {
                    __sifr_yielder.suspend(current.clone()).await;
                    emitted = &emitted + &SifrInt::from_i64(1);
                }
            }
            let size: SifrInt = SifrInt::from(saved.len());
            while (&emitted < &n) && (&size > &SifrInt::from_i64(0)) {
                let index: SifrInt = emitted.floor_mod_known_nonzero(&size);
                let repeated: Option<T> = {
                    let __sifr_checked_read_collection = &saved;
                    let __sifr_checked_read_index = index.clone();
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                };
                if let Some(repeated) = repeated {
                    __sifr_yielder.suspend(repeated.clone()).await;
                }
                emitted = &emitted + &SifrInt::from_i64(1);
            }
        }),
    )
}
fn random_int(min: SifrInt, max: SifrInt) -> SifrInt {
    ::sifr_stdlib::random::random_int(
            ::sifr_runtime::interop::SifrIntBridge::from(min),
            ::sifr_runtime::interop::SifrIntBridge::from(max),
        )
        .into_sifr_int()
}
fn random_float() -> f64 {
    ::sifr_stdlib::random::random_float()
}
fn random_word_to_unit_float(value: SifrInt) -> f64 {
    ::sifr_stdlib::random::random_word_to_unit_float(
        ::sifr_runtime::interop::SifrIntBridge::from(value),
    )
}
fn random_seed() -> SifrInt {
    ::sifr_stdlib::random::random_seed().into_sifr_int()
}
fn random_uniform(min: f64, max: f64) -> f64 {
    ::sifr_stdlib::random::random_uniform(min, max)
}
fn random_randrange(
    start: SifrInt,
    stop: SifrInt,
    step: SifrInt,
) -> Result<SifrInt, ValueError> {
    ::sifr_stdlib::random::random_randrange(
            ::sifr_runtime::interop::SifrIntBridge::from(start),
            ::sifr_runtime::interop::SifrIntBridge::from(stop),
            ::sifr_runtime::interop::SifrIntBridge::from(step),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn random_gauss(mu: f64, sigma: f64) -> f64 {
    ::sifr_stdlib::random::random_gauss(mu, sigma)
}
fn random_module_state_words() -> Vec<SifrInt> {
    ::sifr_stdlib::random::random_module_state_words()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
fn random_module_state_index() -> SifrInt {
    ::sifr_stdlib::random::random_module_state_index().into_sifr_int()
}
fn random_module_state_gauss_next() -> Option<f64> {
    ::sifr_stdlib::random::random_module_state_gauss_next()
}
fn random_module_set_state(
    words: &Vec<SifrInt>,
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
            gauss_next.map(|__sifr_bridge_item_0| __sifr_bridge_item_0),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_encode(s: &String) -> String {
    ::sifr_stdlib::base64::base64_encode(s)
}
fn base64_encode_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::base64::base64_encode_bytes(data)
}
fn base64_decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::base64_decode_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_encode_opts(
    s: &String,
    altchars: &String,
    wrapcol: SifrInt,
) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_encode_opts(
            s,
            altchars,
            ::sifr_runtime::interop::SifrIntBridge::from(wrapcol),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_decode_opts(
    s: &String,
    altchars: &String,
    validate: bool,
    ignorechars: &String,
) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode_opts(s, altchars, validate, ignorechars)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn urlsafe_b64encode(s: &String) -> String {
    ::sifr_stdlib::base64::urlsafe_b64encode(s)
}
fn urlsafe_b64encode_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::base64::urlsafe_b64encode_bytes(data)
}
fn urlsafe_b64decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn urlsafe_b64decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn b32encode(s: &String) -> String {
    ::sifr_stdlib::base64::b32encode(s)
}
fn b32decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::b32decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn b32hexencode(s: &String) -> String {
    ::sifr_stdlib::base64::b32hexencode(s)
}
fn b32hexdecode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::b32hexdecode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn sha256_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha256_bytes(data)
}
fn md5_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::md5_bytes(data)
}
fn sha1_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha1_bytes(data)
}
fn sha224_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha224_bytes(data)
}
fn sha384_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha384_bytes(data)
}
fn sha512_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha512_bytes(data)
}
fn blake2b_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2b_bytes(data)
}
fn blake2s_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2s_bytes(data)
}
const PI: f64 = 3.141592653589793_f64;
const E: f64 = 2.718281828459045_f64;
const TAU: f64 = 6.283185307179586_f64;
const INF: f64 = f64::INFINITY;
const NAN: f64 = f64::NAN;
fn sqrt(x: f64) -> f64 {
    ::sifr_stdlib::math::sqrt(x)
}
fn floor(x: f64) -> SifrInt {
    ::sifr_stdlib::math::floor(x).into_sifr_int()
}
fn ceil(x: f64) -> SifrInt {
    ::sifr_stdlib::math::ceil(x).into_sifr_int()
}
fn log(x: f64) -> f64 {
    ::sifr_stdlib::math::log(x)
}
fn cbrt(x: f64) -> f64 {
    ::sifr_stdlib::math::cbrt(x)
}
fn sin(x: f64) -> f64 {
    ::sifr_stdlib::math::sin(x)
}
fn cos(x: f64) -> f64 {
    ::sifr_stdlib::math::cos(x)
}
fn tan(x: f64) -> f64 {
    ::sifr_stdlib::math::tan(x)
}
fn pow_val(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::pow_val(x, y)
}
fn min_val(a: f64, b: f64) -> f64 {
    ::sifr_stdlib::math::min_val(a, b)
}
fn max_val(a: f64, b: f64) -> f64 {
    ::sifr_stdlib::math::max_val(a, b)
}
fn round_val(x: f64) -> SifrInt {
    ::sifr_stdlib::math::round_val(x).into_sifr_int()
}
fn asin(x: f64) -> f64 {
    ::sifr_stdlib::math::asin(x)
}
fn acos(x: f64) -> f64 {
    ::sifr_stdlib::math::acos(x)
}
fn atan(x: f64) -> f64 {
    ::sifr_stdlib::math::atan(x)
}
fn atan2(y: f64, x: f64) -> f64 {
    ::sifr_stdlib::math::atan2(y, x)
}
fn sinh(x: f64) -> f64 {
    ::sifr_stdlib::math::sinh(x)
}
fn cosh(x: f64) -> f64 {
    ::sifr_stdlib::math::cosh(x)
}
fn tanh(x: f64) -> f64 {
    ::sifr_stdlib::math::tanh(x)
}
fn log10(x: f64) -> f64 {
    ::sifr_stdlib::math::log10(x)
}
fn log2(x: f64) -> f64 {
    ::sifr_stdlib::math::log2(x)
}
fn exp2(x: f64) -> f64 {
    ::sifr_stdlib::math::exp2(x)
}
fn degrees(x: f64) -> f64 {
    ::sifr_stdlib::math::degrees(x)
}
fn radians(x: f64) -> f64 {
    ::sifr_stdlib::math::radians(x)
}
fn isnan(x: f64) -> bool {
    ::sifr_stdlib::math::isnan(x)
}
fn isinf(x: f64) -> bool {
    ::sifr_stdlib::math::isinf(x)
}
fn trunc(x: f64) -> SifrInt {
    ::sifr_stdlib::math::trunc(x).into_sifr_int()
}
fn copysign(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::copysign(x, y)
}
fn signbit(x: f64) -> bool {
    ::sifr_stdlib::math::signbit(x)
}
fn fmod(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::fmod(x, y)
}
fn remainder(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::remainder(x, y)
}
fn hypot(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::hypot(x, y)
}
fn fma(x: f64, y: f64, z: f64) -> f64 {
    ::sifr_stdlib::math::fma(x, y, z)
}
fn fmax(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::fmax(x, y)
}
fn fmin(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::fmin(x, y)
}
fn exp(x: f64) -> f64 {
    ::sifr_stdlib::math::exp(x)
}
fn expm1(x: f64) -> f64 {
    ::sifr_stdlib::math::expm1(x)
}
fn log1p(x: f64) -> f64 {
    ::sifr_stdlib::math::log1p(x)
}
fn fabs(x: f64) -> f64 {
    ::sifr_stdlib::math::fabs(x)
}
fn isfinite(x: f64) -> bool {
    ::sifr_stdlib::math::isfinite(x)
}
fn isnormal(x: f64) -> bool {
    ::sifr_stdlib::math::isnormal(x)
}
fn issubnormal(x: f64) -> bool {
    ::sifr_stdlib::math::issubnormal(x)
}
fn acosh(x: f64) -> f64 {
    ::sifr_stdlib::math::acosh(x)
}
fn asinh(x: f64) -> f64 {
    ::sifr_stdlib::math::asinh(x)
}
fn atanh(x: f64) -> f64 {
    ::sifr_stdlib::math::atanh(x)
}
fn isqrt(n: SifrInt) -> SifrInt {
    ::sifr_stdlib::math::isqrt(::sifr_runtime::interop::SifrIntBridge::from(n))
        .into_sifr_int()
}
fn dist_impl(p: Vec<f64>, q: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::dist(p, q)
}
fn fsum_impl(data: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::fsum(data)
}
fn sumprod_impl(p: Vec<f64>, q: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::sumprod(p, q)
}
fn erf(x: f64) -> f64 {
    ::sifr_stdlib::math::erf(x)
}
fn erfc(x: f64) -> f64 {
    ::sifr_stdlib::math::erfc(x)
}
fn gamma(x: f64) -> f64 {
    ::sifr_stdlib::math::gamma(x)
}
fn lgamma(x: f64) -> f64 {
    ::sifr_stdlib::math::lgamma(x)
}
fn frexp(x: f64) -> Vec<f64> {
    ::sifr_stdlib::math::frexp(x)
}
fn ldexp(m: f64, e: SifrInt) -> f64 {
    ::sifr_stdlib::math::ldexp(m, ::sifr_runtime::interop::SifrIntBridge::from(e))
}
fn modf(x: f64) -> Vec<f64> {
    ::sifr_stdlib::math::modf(x)
}
fn nextafter(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::nextafter(x, y)
}
fn ulp(x: f64) -> f64 {
    ::sifr_stdlib::math::ulp(x)
}
fn factorial(n: SifrInt) -> SifrInt {
    if &n < &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let mut result: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(2);
    while &i <= &n {
        result = &result * &i;
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
}
fn gcd(a: SifrInt, b: SifrInt) -> SifrInt {
    let mut x: SifrInt = a.clone();
    let mut y: SifrInt = b.clone();
    if &x < &SifrInt::from_i64(0) {
        x = &SifrInt::from_i64(0) - &x;
    }
    if &y < &SifrInt::from_i64(0) {
        y = &SifrInt::from_i64(0) - &y;
    }
    while (&y != &SifrInt::from_i64(0)) {
        let temp: SifrInt = y.clone();
        y = x.floor_mod_known_nonzero(&y);
        x = temp;
    }
    x.clone()
}
fn lcm(a: SifrInt, b: SifrInt) -> SifrInt {
    if &a == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if &b == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let g: SifrInt = gcd((a).clone(), (b).clone());
    if &g == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let mut x: SifrInt = a.clone();
    if &x < &SifrInt::from_i64(0) {
        x = &SifrInt::from_i64(0) - &x;
    }
    let mut y: SifrInt = b.clone();
    if &y < &SifrInt::from_i64(0) {
        y = &SifrInt::from_i64(0) - &y;
    }
    &x.floor_div_known_nonzero(&g) * &y
}
fn comb(n: SifrInt, k: SifrInt) -> SifrInt {
    if &k < &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if &k > &n {
        return SifrInt::from_i64(0);
    }
    if &k == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(1);
    }
    if &k == &n {
        return SifrInt::from_i64(1);
    }
    let mut r: SifrInt = k.clone();
    if &r > &(&n - &k) {
        r = &n - &k;
    }
    let mut result: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &r) {
        result = &result * &(&n - &i);
        let divisor: SifrInt = &i + &SifrInt::from_i64(1);
        if (&divisor == &SifrInt::from_i64(0)) {
            return SifrInt::from_i64(0);
        }
        result = result.floor_div_known_nonzero(&divisor);
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
}
fn perm(n: SifrInt, k: SifrInt) -> SifrInt {
    if &k < &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if &k > &n {
        return SifrInt::from_i64(0);
    }
    let mut result: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &k {
        result = &result * &(&n - &i);
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
}
fn log_base(x: f64, base: f64) -> f64 {
    log(x) / log(base)
}
fn isclose(a: f64, b: f64, rel_tol: f64, abs_tol: f64) -> bool {
    if rel_tol < (0.0_f64) {
        return false;
    }
    if abs_tol < (0.0_f64) {
        return false;
    }
    if a == b {
        return true;
    }
    if isnan(a) || isnan(b) {
        return false;
    }
    if isinf(a) || isinf(b) {
        return false;
    }
    let mut diff: f64 = a - b;
    if diff < (0.0_f64) {
        diff = (0.0_f64) - diff;
    }
    let mut a_abs: f64 = a;
    if a_abs < (0.0_f64) {
        a_abs = (0.0_f64) - a_abs;
    }
    let mut b_abs: f64 = b;
    if b_abs < (0.0_f64) {
        b_abs = (0.0_f64) - b_abs;
    }
    let mut larger_abs: f64 = a_abs;
    if b_abs > larger_abs {
        larger_abs = b_abs;
    }
    let mut rel_bound: f64 = rel_tol * larger_abs;
    if abs_tol > rel_bound {
        rel_bound = abs_tol;
    }
    diff <= rel_bound
}
fn prod(data: &Vec<SifrInt>) -> SifrInt {
    let mut result: SifrInt = SifrInt::from_i64(1);
    for val in data.iter().cloned() {
        result = &result * &val;
    }
    result.clone()
}
fn _copy_float_list(data: &Vec<f64>) -> Vec<f64> {
    let mut out: Vec<f64> = vec![];
    for value in data.iter().copied() {
        out.push(value);
    }
    out
}
fn dist(p: &Vec<f64>, q: &Vec<f64>) -> f64 {
    dist_impl(_copy_float_list(p), _copy_float_list(q))
}
fn fsum(data: &Vec<f64>) -> f64 {
    fsum_impl(_copy_float_list(data))
}
fn sumprod(p: &Vec<f64>, q: &Vec<f64>) -> f64 {
    sumprod_impl(_copy_float_list(p), _copy_float_list(q))
}
fn frexp_mantissa(x: f64) -> f64 {
    let parts: Vec<f64> = frexp(x);
    let m: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(m) = m else {
        return NAN;
    };
    m
}
fn frexp_exponent(x: f64) -> SifrInt {
    let parts: Vec<f64> = frexp(x);
    let exp_val: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(1);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(exp_val) = exp_val else {
        return SifrInt::from_i64(0);
    };
    trunc(exp_val)
}
fn modf_fractional(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let f: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(f) = f else {
        return NAN;
    };
    f
}
fn modf_integral(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let i: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(1);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(i) = i else {
        return NAN;
    };
    i
}
fn pow(x: f64, y: f64) -> f64 {
    pow_val(x, y)
}
fn __const__MT_N() -> SifrInt {
    SifrInt::from_i64(624)
}
fn __const__MT_M() -> SifrInt {
    SifrInt::from_i64(397)
}
fn __const__MT_MATRIX_A() -> SifrInt {
    SifrInt::from_i64(2567483615)
}
fn __const__MT_UPPER_MASK() -> SifrInt {
    SifrInt::from_i64(2147483648)
}
fn __const__MT_LOWER_MASK() -> SifrInt {
    SifrInt::from_i64(2147483647)
}
fn __const__MT_F() -> SifrInt {
    SifrInt::from_i64(1812433253)
}
fn __const__MT_WORD_MASK() -> SifrInt {
    SifrInt::from_i64(4294967295)
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2erandom_x2eRandomState {
    version: SifrInt,
    state_words: Vec<SifrInt>,
    index: SifrInt,
    gauss_next: Option<f64>,
}
impl __SifrStdlib_sifr_x2erandom_x2eRandomState {
    fn new(
        version: SifrInt,
        state_words: Vec<SifrInt>,
        index: SifrInt,
        gauss_next: Option<f64>,
    ) -> Self {
        let __sifr_field_init_0: SifrInt = version.clone();
        let __sifr_field_init_1: Vec<SifrInt> = state_words;
        let __sifr_field_init_2: SifrInt = index.clone();
        let __sifr_field_init_3: Option<f64> = gauss_next;
        Self {
            version: __sifr_field_init_0,
            state_words: __sifr_field_init_1,
            index: __sifr_field_init_2,
            gauss_next: __sifr_field_init_3,
        }
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandomState {}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2erandom_x2eRandom {
    _state_words: Vec<SifrInt>,
    _index: SifrInt,
    _gauss_next: Option<f64>,
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn new(seed_value: Option<SifrInt>) -> Self {
        let normalized_seed: SifrInt = _normalize_seed_input((seed_value).clone());
        let __sifr_field_init_0: Vec<SifrInt> = _seed_words_from_seed(
            (normalized_seed).clone(),
        );
        let __sifr_field_init_1: SifrInt = __const__MT_N().clone();
        let __sifr_field_init_2: Option<f64> = None;
        Self {
            _state_words: __sifr_field_init_0,
            _index: __sifr_field_init_1,
            _gauss_next: __sifr_field_init_2,
        }
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn seed(&mut self, seed_value: &Option<SifrInt>) {
        let normalized_seed: SifrInt = _normalize_seed_input(
            (seed_value.clone()).clone(),
        );
        self._state_words = _seed_words_from_seed((normalized_seed).clone());
        self._index = __const__MT_N().clone();
        self._gauss_next = None;
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn _twist(&mut self) {
        let mut i: SifrInt = SifrInt::from_i64(0);
        while (&SifrInt::from_i64(0) <= &i)
            && (&i < &SifrInt::from(self._state_words.len()))
        {
            let y: SifrInt = &(&_state_word_at(&self._state_words, (i).clone())
                & &__const__MT_UPPER_MASK())
                + &(&_state_word_at(
                    &self._state_words,
                    (&i + &SifrInt::from_i64(1))
                        .floor_mod_known_nonzero(&__const__MT_N()),
                ) & &__const__MT_LOWER_MASK());
            let mut x_a: SifrInt = y.floor_div_known_nonzero(&SifrInt::from_i64(2));
            if (&y.floor_mod_known_nonzero(&SifrInt::from_i64(2))
                != &SifrInt::from_i64(0))
            {
                x_a = &x_a ^ &__const__MT_MATRIX_A();
            }
            let new_word: SifrInt = &_state_word_at(
                &self._state_words,
                (&i + &__const__MT_M()).floor_mod_known_nonzero(&__const__MT_N()),
            ) ^ &x_a;
            {
                let __assign_value = &new_word & &__const__MT_WORD_MASK();
                {
                    let __index_raw = i.clone();
                    let __index_normalized = __index_raw
                        .normalize_index_or_len(self._state_words.len());
                    if let Some(__elem) = self._state_words.get_mut(__index_normalized) {
                        *__elem = __assign_value;
                    }
                }
            }
            i = &i + &SifrInt::from_i64(1);
        }
        self._index = SifrInt::from_i64(0);
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn _next_u32(&mut self) -> SifrInt {
        if (&self._index.clone() >= &__const__MT_N()) {
            self._twist();
        }
        let mut y: SifrInt = _state_word_at(&self._state_words, self._index.clone());
        self._index = &self._index.clone() + &SifrInt::from_i64(1);
        y = &y ^ &y.floor_div_known_nonzero(&SifrInt::from_i64(2048));
        y = &y ^ &(&(&y * &SifrInt::from_i64(128)) & &SifrInt::from_i64(2636928640));
        y = &y ^ &(&(&y * &SifrInt::from_i64(32768)) & &SifrInt::from_i64(4022730752));
        y = &y ^ &y.floor_div_known_nonzero(&SifrInt::from_i64(262144));
        &y & &__const__MT_WORD_MASK()
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn random(&mut self) -> f64 {
        random_word_to_unit_float(self._next_u32())
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn uniform(&mut self, minimum: f64, maximum: f64) -> f64 {
        minimum + ((maximum - minimum) * self.random())
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn randrange(
        &mut self,
        start: &SifrInt,
        stop: &Option<SifrInt>,
        step: &SifrInt,
    ) -> Result<SifrInt, ValueError> {
        if (&step.clone() == &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randrange: step must not be zero".to_string()));
        }
        let mut actual_start: SifrInt = start.clone();
        let mut actual_stop: SifrInt = start.clone();
        if (stop.clone() == None) {
            actual_start = SifrInt::from_i64(0);
        } else {
            if let Some(stop) = stop.as_ref() {
                actual_stop = stop.clone();
            }
        }
        let width: SifrInt = &actual_stop - &actual_start;
        if (&step.clone() > &SifrInt::from_i64(0)) {
            if (&width <= &SifrInt::from_i64(0)) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
        } else {
            if (&width >= &SifrInt::from_i64(0)) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
        }
        let mut abs_width: SifrInt = width.clone();
        if &abs_width < &SifrInt::from_i64(0) {
            abs_width = &SifrInt::from_i64(0) - &abs_width;
        }
        let mut abs_step: SifrInt = step.clone();
        if &abs_step < &SifrInt::from_i64(0) {
            abs_step = &SifrInt::from_i64(0) - &abs_step;
        }
        if (&abs_step == &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randrange: step must not be zero".to_string()));
        }
        let count: SifrInt = (&(&abs_width + &abs_step) - &SifrInt::from_i64(1))
            .floor_div_known_nonzero(&abs_step);
        if (&count <= &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randrange: empty range".to_string()));
        }
        if (&count == &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randrange: empty range".to_string()));
        }
        let pick: SifrInt = self._next_u32().floor_mod_known_nonzero(&count);
        Ok(&actual_start + &(&pick * step))
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn randint(
        &mut self,
        minimum: &SifrInt,
        maximum: &SifrInt,
    ) -> Result<SifrInt, ValueError> {
        if *minimum > *maximum {
            return Err(ValueError::new("randint: min must be <= max".to_string()));
        }
        self.randrange(
            minimum,
            &Some((maximum + &SifrInt::from_i64(1)).clone()),
            &SifrInt::from_i64(1),
        )
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn getrandbits(&mut self, k: &SifrInt) -> Result<SifrInt, ValueError> {
        if (&k.clone() < &SifrInt::from_i64(0)) {
            return Err(
                ValueError::new("getrandbits: number of bits must be >= 0".to_string()),
            );
        }
        let mut result: SifrInt = SifrInt::from_i64(0);
        let mut bits_left: SifrInt = k.clone();
        while (&bits_left > &SifrInt::from_i64(0)) {
            let word: SifrInt = self._next_u32();
            let mut take: SifrInt = SifrInt::from_i64(32);
            if (&bits_left < &SifrInt::from_i64(32)) {
                take = bits_left.clone();
            }
            let mut mask: SifrInt = SifrInt::from_i64(0);
            let mut shifted_result: SifrInt = result;
            let mut shift_index: SifrInt = SifrInt::from_i64(0);
            while (&shift_index < &take) {
                mask = &(&mask * &SifrInt::from_i64(2)) + &SifrInt::from_i64(1);
                shifted_result = &shifted_result * &SifrInt::from_i64(2);
                shift_index = &shift_index + &SifrInt::from_i64(1);
            }
            result = &shifted_result | &(&word & &mask);
            bits_left = &bits_left - &take;
        }
        Ok(result.clone())
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn randbytes(&mut self, n: &SifrInt) -> Result<Vec<u8>, ValueError> {
        if (&n.clone() < &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randbytes: n must be >= 0".to_string()));
        }
        let mut values: Vec<SifrInt> = vec![];
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < *n {
            let byte_value: SifrInt = &self._next_u32() & &SifrInt::from_i64(255);
            values.push(byte_value.clone());
            i = &i + &SifrInt::from_i64(1);
        }
        {
            let __vals = values;
            let mut __out = Vec::new();
            for __pair in __vals.iter().enumerate() {
                __out
                    .push(
                        __pair
                            .1
                            .try_to_u8()
                            .map_err(|_error| ValueError {
                                message: format!(
                                    "byte out of range at index {}: {}", __pair.0, * __pair.1
                                ),
                            })?,
                    );
            }
            Ok::<Vec<u8>, ValueError>(__out)
        }
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn gauss(&mut self, mu: f64, sigma: f64) -> f64 {
        let cached: Option<f64> = self._gauss_next;
        if let Some(cached) = cached {
            self._gauss_next = None;
            return mu + (sigma * cached);
        }
        let mut u1: f64 = self.random();
        if u1 <= (0.0_f64) {
            u1 = 0.000000000001_f64;
        }
        let u2: f64 = self.random();
        let radius: f64 = sqrt(-(2.0_f64) * log(u1));
        let theta: f64 = ((2.0_f64) * PI) * u2;
        let z0: f64 = radius * cos(theta);
        let z1: f64 = radius * sin(theta);
        let next_cached: Option<f64> = Some(z1);
        self._gauss_next = next_cached;
        mu + (sigma * z0)
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn getstate(&self) -> __SifrStdlib_sifr_x2erandom_x2eRandomState {
        __SifrStdlib_sifr_x2erandom_x2eRandomState::new(
            SifrInt::from_i64(3),
            _clone_words(&self._state_words),
            self._index.clone(),
            self._gauss_next,
        )
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn setstate(
        &mut self,
        state: &__SifrStdlib_sifr_x2erandom_x2eRandomState,
    ) -> Result<(), ValueError> {
        if (&state.version.clone() != &SifrInt::from_i64(3)) {
            return Err(ValueError::new("setstate: unsupported version".to_string()));
        }
        if (&SifrInt::from(state.state_words.len()) != &__const__MT_N()) {
            return Err(
                ValueError::new("setstate: state_words must have length 624".to_string()),
            );
        }
        if (&state.index.clone() < &SifrInt::from_i64(0))
            || (&state.index.clone() > &__const__MT_N())
        {
            return Err(
                ValueError::new("setstate: index must be in range [0, 624]".to_string()),
            );
        }
        let mut normalized: Vec<SifrInt> = vec![];
        for word in state.state_words.clone().iter().cloned() {
            if (&word < &SifrInt::from_i64(0)) || (&word > &__const__MT_WORD_MASK()) {
                return Err(ValueError::new("setstate: word out of range".to_string()));
            }
            normalized.push(&word & &__const__MT_WORD_MASK());
        }
        self._state_words = normalized;
        self._index = state.index.clone();
        self._gauss_next = state.gauss_next;
        Ok(())
    }
}
fn _state_word_at(words: &Vec<SifrInt>, index: SifrInt) -> SifrInt {
    let value: Option<SifrInt> = {
        let __sifr_checked_read_collection = &words;
        let __sifr_checked_read_index = index.clone();
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    if let Some(value) = value.clone() {
        return value;
    }
    SifrInt::from_i64(0)
}
fn _clone_words(words: &Vec<SifrInt>) -> Vec<SifrInt> {
    let mut copied: Vec<SifrInt> = vec![];
    for word in words.iter().cloned() {
        copied.push(word.clone());
    }
    copied
}
fn _normalize_seed_input(seed_value: Option<SifrInt>) -> SifrInt {
    if let Some(seed_value) = seed_value.clone() {
        return seed_value.clone();
    }
    random_seed()
}
fn _seed_words_from_seed(seed_value: SifrInt) -> Vec<SifrInt> {
    let mut words: Vec<SifrInt> = vec![];
    words.push(&seed_value & &__const__MT_WORD_MASK());
    let mut i: SifrInt = SifrInt::from_i64(1);
    while (&i < &__const__MT_N()) {
        let prev: SifrInt = _state_word_at(&words, &i - &SifrInt::from_i64(1));
        let next_word: SifrInt = &(&(&__const__MT_F()
            * &(&prev ^ &prev.floor_div_known_nonzero(&SifrInt::from_i64(1073741824))))
            + &i) & &__const__MT_WORD_MASK();
        words.push(next_word.clone());
        i = &i + &SifrInt::from_i64(1);
    }
    words
}
fn _build_state_from_module_storage() -> __SifrStdlib_sifr_x2erandom_x2eRandomState {
    __SifrStdlib_sifr_x2erandom_x2eRandomState::new(
        SifrInt::from_i64(3),
        random_module_state_words(),
        random_module_state_index(),
        random_module_state_gauss_next(),
    )
}
fn _store_state_into_module_storage(state: &__SifrStdlib_sifr_x2erandom_x2eRandomState) {
    let _set_result: Result<(), ValueError> = random_module_set_state(
        &_clone_words(&state.state_words.clone()),
        state.index.clone(),
        state.gauss_next,
    );
    let _ = _set_result;
}
fn _ensure_module_state_initialized() {
    let words: Vec<SifrInt> = random_module_state_words();
    if &SifrInt::from(words.len()) == &__const__MT_N() {
        return;
    }
    let bootstrap: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(SifrInt::from_i64(5489)),
    );
    _store_state_into_module_storage(&bootstrap.getstate());
}
fn _module_random() -> __SifrStdlib_sifr_x2erandom_x2eRandom {
    _ensure_module_state_initialized();
    let mut r: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(SifrInt::from_i64(0)),
    );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let _set_result: Result<(), ValueError> = r
            .setstate(&_build_state_from_module_storage());
        let _ = _set_result;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = e.message.clone();
    }
    r
}
fn _sync_module_random(generator: &mut __SifrStdlib_sifr_x2erandom_x2eRandom) {
    _store_state_into_module_storage(&generator.getstate());
}
fn shuffle<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(items: &mut Vec<T>) {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let n: SifrInt = SifrInt::from(items.len());
    if (&n > &SifrInt::from_i64(1)) {
        let mut i: SifrInt = &n - &SifrInt::from_i64(1);
        while (&i > &SifrInt::from_i64(0)) {
            let divisor: SifrInt = &i + &SifrInt::from_i64(1);
            if (&divisor == &SifrInt::from_i64(0)) {
                return;
            }
            let j: SifrInt = generator._next_u32().floor_mod_known_nonzero(&divisor);
            let left: Option<T> = {
                let __sifr_checked_read_collection = &items;
                let __sifr_checked_read_index = i.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            let right: Option<T> = {
                let __sifr_checked_read_collection = &items;
                let __sifr_checked_read_index = j.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(left) = left {
                if let Some(right) = right {
                    if (&SifrInt::from_i64(0) <= &i)
                        && (&i < &SifrInt::from(items.len()))
                    {
                        {
                            let __assign_value = right.clone();
                            {
                                let __index_raw = i.clone();
                                let __index_normalized = __index_raw
                                    .normalize_index_or_len(items.len());
                                if let Some(__elem) = items.get_mut(__index_normalized) {
                                    *__elem = __assign_value;
                                }
                            }
                        }
                    }
                    if (&SifrInt::from_i64(0) <= &j)
                        && (&j < &SifrInt::from(items.len()))
                    {
                        {
                            let __assign_value = left.clone();
                            {
                                let __index_raw = j.clone();
                                let __index_normalized = __index_raw
                                    .normalize_index_or_len(items.len());
                                if let Some(__elem) = items.get_mut(__index_normalized) {
                                    *__elem = __assign_value;
                                }
                            }
                        }
                    }
                }
            }
            i = &i - &SifrInt::from_i64(1);
        }
    }
    _sync_module_random(&mut generator);
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ParseError {
    message: String,
}
impl ParseError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for ParseError {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ValueError {
    message: String,
}
impl ValueError {
    fn new(message: String) -> Self {
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
struct IndexError {
    message: String,
}
impl IndexError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for IndexError {}
fn is_small(x: SifrInt) -> bool {
    &x < &SifrInt::from_i64(5)
}
fn is_even(x: SifrInt) -> bool {
    (&x.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0))
}
fn concat(a: &String, b: &String) -> String {
    {
        let mut __sifr_concat: String = String::with_capacity(a.len() + b.len());
        __sifr_concat.push_str((a).as_str());
        __sifr_concat.push_str((b).as_str());
        __sifr_concat
    }
}
fn main() {
    println!("=== Generic chain ===");
    let ints: Vec<SifrInt> = chain(
            &vec![
                vec![SifrInt::from_i64(1), SifrInt::from_i64(2)],
                vec![SifrInt::from_i64(3), SifrInt::from_i64(4)]
            ],
        )
        .collect::<Vec<_>>();
    println!("{:?}", ints);
    let strs: Vec<String> = chain(
            &vec![
                vec!["a".to_string(), "b".to_string()], vec!["c".to_string(), "d"
                .to_string()]
            ],
        )
        .collect::<Vec<_>>();
    println!("{:?}", strs);
    println!("=== Generic take ===");
    let first3_int: Vec<SifrInt> = take(
        SifrInt::from_i64(3),
        &(vec![
            SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30),
            SifrInt::from_i64(40), SifrInt::from_i64(50)
        ])
            .into_iter()
            .collect::<Vec<_>>(),
    );
    println!("{:?}", first3_int);
    let first2_str: Vec<String> = take(
        SifrInt::from_i64(2),
        &(vec!["hello".to_string(), "world".to_string(), "foo".to_string()])
            .into_iter()
            .collect::<Vec<_>>(),
    );
    println!("{:?}", first2_str);
    println!("=== Generic flatten ===");
    let nested_int: Vec<Vec<SifrInt>> = vec![
        vec![SifrInt::from_i64(1), SifrInt::from_i64(2)], vec![SifrInt::from_i64(3),
        SifrInt::from_i64(4)], vec![SifrInt::from_i64(5)]
    ];
    let flat_int: Vec<SifrInt> = flatten(
        &(nested_int).iter().cloned().collect::<Vec<_>>(),
    );
    println!("{:?}", flat_int);
    println!("=== Generic accumulate ===");
    let sums: Vec<SifrInt> = accumulate(
            Box::new(
                vec![
                    SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3),
                    SifrInt::from_i64(4), SifrInt::from_i64(5)
                ]
                    .into_iter(),
            ),
            None,
        )
        .collect::<Vec<_>>();
    println!("{:?}", sums);
    let float_sums: Vec<f64> = accumulate(
            Box::new(vec![1.0_f64, 2.5_f64, 3.5_f64].into_iter()),
            None,
        )
        .collect::<Vec<_>>();
    println!("{:?}", float_sums);
    println!("=== Predicate-based dropwhile ===");
    let data: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(3), SifrInt::from_i64(7),
        SifrInt::from_i64(2), SifrInt::from_i64(8)
    ];
    let dropped: Vec<SifrInt> = dropwhile(
            |__arg0| is_small((__arg0).clone()),
            Box::new(data.clone().into_iter()),
        )
        .collect::<Vec<_>>();
    println!("{:?}", dropped);
    println!("=== Predicate-based takewhile ===");
    let taken: Vec<SifrInt> = takewhile(
            |__arg0| is_small((__arg0).clone()),
            Box::new(data.clone().into_iter()),
        )
        .collect::<Vec<_>>();
    println!("{:?}", taken);
    println!("=== Predicate-based filterfalse ===");
    let odds: Vec<SifrInt> = filterfalse(
            |__arg0| is_even((__arg0).clone()),
            Box::new(
                vec![
                    SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3),
                    SifrInt::from_i64(4), SifrInt::from_i64(5), SifrInt::from_i64(6)
                ]
                    .into_iter(),
            ),
        )
        .collect::<Vec<_>>();
    println!("{:?}", odds);
    println!("=== Generic heapq ===");
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(9), SifrInt::from_i64(3), SifrInt::from_i64(7),
        SifrInt::from_i64(1), SifrInt::from_i64(5)
    ];
    let small: Vec<SifrInt> = nsmallest(SifrInt::from_i64(3), &items);
    println!("{:?}", small);
    let big: Vec<SifrInt> = nlargest(SifrInt::from_i64(2), &items);
    println!("{:?}", big);
    println!("=== Generic Counter[T] ===");
    let words: Vec<String> = vec![
        "apple".to_string(), "banana".to_string(), "apple".to_string(), "cherry"
        .to_string(), "banana".to_string(), "apple".to_string()
    ];
    let c: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(&words);
    println!("{}", c.get(& "apple".to_string(), & SifrInt::from_i64(0)));
    println!("{}", c.total());
    let top: Vec<(String, SifrInt)> = c
        .most_common(&Some((SifrInt::from_i64(2)).clone()));
    println!("{:?}", top);
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(2),
        SifrInt::from_i64(3), SifrInt::from_i64(3), SifrInt::from_i64(3)
    ];
    let ci: __SifrStdlib_sifr_x2ecollections_x2eCounter<SifrInt> = from_list(&nums);
    println!("{}", ci.get(& SifrInt::from_i64(3), & SifrInt::from_i64(0)));
    let c2: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(
        &vec!["banana".to_string(), "date".to_string()],
    );
    let combined: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = &c + &c2;
    println!("{}", combined.get(& "banana".to_string(), & SifrInt::from_i64(0)));
    println!("=== Generic deque[T] ===");
    let mut d: __SifrStdlib_sifr_x2ecollections_x2edeque<String> = __SifrStdlib_sifr_x2ecollections_x2edeque::new(
        None,
        None,
    );
    d.append(&"first".to_string());
    d.append(&"second".to_string());
    d.appendleft(&"zero".to_string());
    let items_d: Vec<String> = d.to_list();
    println!("{:?}", items_d);
    println!("{}", SifrInt::from(d.len()));
    println!("=== Generic reduce ===");
    let sentence: String = reduce(
        concat,
        &vec!["hello".to_string(), " ".to_string(), "world".to_string()],
        &"".to_string(),
    );
    println!("{}", sentence);
    println!("=== Generic compress ===");
    let data_c: Vec<String> = vec![
        "a".to_string(), "b".to_string(), "c".to_string(), "d".to_string(), "e"
        .to_string()
    ];
    let sel: Vec<bool> = vec![true, false, true, false, true];
    let compressed: Vec<String> = compress(
            Box::new(data_c.clone().into_iter()),
            Box::new(sel.clone().into_iter()),
        )
        .collect::<Vec<_>>();
    println!("{:?}", compressed);
    println!("=== Generic zip_longest ===");
    let zl_str: Vec<Vec<String>> = zip_longest(
            Box::new(
                vec!["a".to_string(), "b".to_string(), "c".to_string()].into_iter(),
            ),
            Box::new(vec!["x".to_string(), "y".to_string()].into_iter()),
            &"-".to_string(),
        )
        .collect::<Vec<_>>();
    for pair in zl_str.iter().cloned() {
        println!("{:?}", pair);
    }
    println!("=== Generic shuffle ===");
    let mut shuffled_str: Vec<String> = vec![
        "a".to_string(), "b".to_string(), "c".to_string(), "d".to_string(), "e"
        .to_string()
    ];
    shuffle(&mut shuffled_str);
    println!("{}", SifrInt::from(shuffled_str.len()));
}
