// src/main.rs
mod __sifr_project_nominals {
    pub use ::std::collections::HashMap;
    pub use ::std::collections::VecDeque;
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2ecollections_x2eCounter<T: std::hash::Hash + Eq> {
        pub counts: HashMap<T, i64>,
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn new(source: Option<HashMap<T, i64>>, iterable: Option<Vec<T>>) -> Self {
            let mut counts: HashMap<T, i64> = HashMap::from([]);
            if let Some(source) = source {
                for key in source.keys().cloned().collect::<Vec<_>>() {
                    let value: Option<i64> = source.get(&key).copied();
                    if let Some(value) = value {
                        counts.insert(key.clone(), value);
                    }
                }
            }
            if let Some(iterable) = iterable {
                for item in iterable.iter().cloned() {
                    let value2: Option<i64> = counts.get(&item).copied();
                    if let Some(value2) = value2 {
                        counts.insert(item.clone(), value2 + (1_i64));
                    } else {
                        counts.insert(item.clone(), 1_i64);
                    }
                }
            }
            let __sifr_field_init_0: HashMap<T, i64> = counts;
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
    impl<T: ::std::hash::Hash + Eq> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn __getitem__(&self, key: &T) -> i64 {
            let val: Option<i64> = self.counts.get(&key).copied();
            if let Some(val) = val {
                return val;
            }
            0_i64
        }
    }
    impl<T: ::std::hash::Hash + Eq> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn get(&self, key: &T, default: i64) -> i64 {
            let val: Option<i64> = self.counts.get(&key).copied();
            if let Some(val) = val {
                return val;
            }
            default
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn increment(&mut self, key: &T) {
            let val: Option<i64> = self.counts.get(&key).copied();
            if let Some(val) = val {
                self.counts.insert(key.clone(), val + (1_i64));
            } else {
                self.counts.insert(key.clone(), 1_i64);
            }
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn total(&self) -> i64 {
            let mut total: i64 = 0_i64;
            for count in self.counts.values().cloned().collect::<Vec<_>>() {
                total += count;
            }
            total
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn most_common(&self, n: Option<i64>) -> Vec<(T, i64)> {
            let mut result: Vec<(T, i64)> = vec![];
            for key in self.counts.keys().cloned().collect::<Vec<_>>() {
                let count: Option<i64> = self.counts.get(&key).copied();
                if let Some(count) = count {
                    let entry: (T, i64) = (key.clone(), count);
                    result.push(entry.clone());
                }
            }
            let sz: i64 = result.len() as i64;
            let mut i: i64 = 0_i64;
            while i < sz {
                let mut j: i64 = i + (1_i64);
                while j < sz {
                    let left: Option<(T, i64)> = Some(result[i as usize].clone());
                    let right: Option<(T, i64)> = Some(result[j as usize].clone());
                    if let Some(left) = left {
                        if let Some(right) = right {
                            if ((right).1 > (left).1) {
                                {
                                    let __idx_raw = i;
                                    let __idx_norm = if __idx_raw < 0 {
                                        (result.len() as i64) + __idx_raw
                                    } else {
                                        __idx_raw
                                    };
                                    if __idx_norm >= 0 {
                                        if let Some(__elem) = result.get_mut(__idx_norm as usize) {
                                            *__elem = right.clone();
                                        }
                                    }
                                }
                                {
                                    let __idx_raw = j;
                                    let __idx_norm = if __idx_raw < 0 {
                                        (result.len() as i64) + __idx_raw
                                    } else {
                                        __idx_raw
                                    };
                                    if __idx_norm >= 0 {
                                        if let Some(__elem) = result.get_mut(__idx_norm as usize) {
                                            *__elem = left.clone();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    j += 1_i64;
                }
                i += 1_i64;
            }
            let Some(n) = n else {
                return result;
            };
            if n <= (0_i64) {
                return vec![];
            }
            let mut top: Vec<(T, i64)> = vec![];
            let mut index: i64 = 0_i64;
            while index < n {
                if (index >= (result.len() as i64)) {
                    return top;
                }
                let value: Option<(T, i64)> = Some(result[index as usize].clone());
                if let Some(value) = value {
                    top.push(value.clone());
                }
                index += 1_i64;
            }
            top
        }
    }
    impl<
        T: ::std::hash::Hash + Eq + Clone + PartialOrd,
    > __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn keys(&self) -> Vec<T> {
            let mut result: Vec<T> = self.counts.keys().cloned().collect::<Vec<_>>();
            let sz: i64 = result.len() as i64;
            let mut i: i64 = 0_i64;
            while i < sz {
                let mut j: i64 = i + (1_i64);
                while j < sz {
                    let left: Option<T> = Some(result[i as usize].clone());
                    let right: Option<T> = Some(result[j as usize].clone());
                    if let Some(left) = left {
                        if let Some(right) = right {
                            if right < left {
                                {
                                    let __idx_raw = i;
                                    let __idx_norm = if __idx_raw < 0 {
                                        (result.len() as i64) + __idx_raw
                                    } else {
                                        __idx_raw
                                    };
                                    if __idx_norm >= 0 {
                                        if let Some(__elem) = result.get_mut(__idx_norm as usize) {
                                            *__elem = right.clone();
                                        }
                                    }
                                }
                                {
                                    let __idx_raw = j;
                                    let __idx_norm = if __idx_raw < 0 {
                                        (result.len() as i64) + __idx_raw
                                    } else {
                                        __idx_raw
                                    };
                                    if __idx_norm >= 0 {
                                        if let Some(__elem) = result.get_mut(__idx_norm as usize) {
                                            *__elem = left.clone();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    j += 1_i64;
                }
                i += 1_i64;
            }
            result
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn items(&self) -> Vec<(T, i64)> {
            let mut result: Vec<(T, i64)> = vec![];
            for key in self.counts.keys().cloned().collect::<Vec<_>>() {
                let value: Option<i64> = self.counts.get(&key).copied();
                if let Some(value) = value {
                    let entry: (T, i64) = (key.clone(), value);
                    result.push(entry.clone());
                }
            }
            result
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn values(&self) -> Vec<i64> {
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
                let other_val: Option<i64> = other.counts.get(&key).copied();
                if let Some(other_val) = other_val {
                    let existing: Option<i64> = self.counts.get(&key).copied();
                    if let Some(existing) = existing {
                        self.counts.insert(key, existing + other_val);
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
                let other_val: Option<i64> = other.counts.get(&key).copied();
                if let Some(other_val) = other_val {
                    let existing: Option<i64> = self.counts.get(&key).copied();
                    if let Some(existing) = existing {
                        self.counts.insert(key, existing - other_val);
                    } else {
                        self.counts.insert(key, (0_i64) - other_val);
                    }
                }
            }
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        pub fn elements(&self) -> Vec<T> {
            let mut result: Vec<T> = vec![];
            let all_keys: Vec<T> = self.counts.keys().cloned().collect::<Vec<_>>();
            let mut ki: i64 = 0_i64;
            while (ki < (all_keys.len() as i64)) {
                let key_opt: Option<T> = Some(all_keys[ki as usize].clone());
                if let Some(key_opt) = key_opt {
                    let cnt: Option<i64> = self.counts.get(&key_opt).copied();
                    if let Some(cnt) = cnt {
                        let mut i: i64 = 0_i64;
                        while i < cnt {
                            let key_copy: Option<T> = Some(all_keys[ki as usize].clone());
                            if let Some(key_copy) = key_copy {
                                result.push(key_copy.clone().clone());
                            }
                            i += 1_i64;
                        }
                    }
                }
                ki += 1_i64;
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
            let mut new_counts: HashMap<T, i64> = HashMap::from([]);
            for key in Box::new(
                (self.counts.keys().cloned().collect::<Vec<_>>()).into_iter(),
            ) {
                let a_val: Option<i64> = self.counts.get(&key).copied();
                if let Some(a_val) = a_val {
                    let b_val: Option<i64> = other.counts.get(&key).copied();
                    let mut b_count: i64 = 0_i64;
                    if let Some(b_val) = b_val {
                        b_count = b_val;
                    }
                    let total: i64 = a_val + b_count;
                    if total > (0_i64) {
                        new_counts.insert(key.clone(), total);
                    }
                }
            }
            for key2 in Box::new(
                (other.counts.keys().cloned().collect::<Vec<_>>()).into_iter(),
            ) {
                let already: Option<i64> = new_counts.get(&key2).copied();
                if already.is_none() {
                    let b_val2: Option<i64> = other.counts.get(&key2).copied();
                    if let Some(b_val2) = b_val2 {
                        if b_val2 > (0_i64) {
                            new_counts.insert(key2.clone(), b_val2);
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
            let mut new_counts: HashMap<T, i64> = HashMap::from([]);
            for key in Box::new(
                (self.counts.keys().cloned().collect::<Vec<_>>()).into_iter(),
            ) {
                let a_val: Option<i64> = self.counts.get(&key).copied();
                if let Some(a_val) = a_val {
                    let b_val: Option<i64> = other.counts.get(&key).copied();
                    let mut b_count: i64 = 0_i64;
                    if let Some(b_val) = b_val {
                        b_count = b_val;
                    }
                    let diff: i64 = a_val - b_count;
                    if diff > (0_i64) {
                        new_counts.insert(key.clone(), diff);
                    }
                }
            }
            __SifrStdlib_sifr_x2ecollections_x2eCounter::new(Some(new_counts), None)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub _data: VecDeque<T>,
        pub maxlen: Option<i64>,
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn new(items: Option<Vec<T>>, maxlen: Option<i64>) -> Self {
            let mut data: Vec<T> = vec![];
            if let Some(items) = items {
                let mut start: i64 = 0_i64;
                if let Some(maxlen) = maxlen {
                    if ((items.len() as i64) > maxlen) {
                        start = (items.len() as i64) - maxlen;
                    }
                }
                let mut i: i64 = start;
                while (i < (items.len() as i64)) {
                    let item: Option<T> = Some(items[i as usize].clone());
                    if let Some(item) = item {
                        data.push(item.clone().clone());
                    }
                    i += 1_i64;
                }
            }
            let __sifr_field_init_0: Option<i64> = maxlen;
            let __sifr_field_init_1: VecDeque<T> = VecDeque::from(data);
            Self {
                maxlen: __sifr_field_init_0,
                _data: __sifr_field_init_1,
            }
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn append(&mut self, val: &T) {
            self._data.push_back(val.clone().clone());
            let maxlen_opt: Option<i64> = self.maxlen;
            if let Some(maxlen_opt) = maxlen_opt {
                let maxlen: i64 = maxlen_opt;
                if ((self._data.len() as i64) > maxlen) {
                    self._data.pop_front();
                }
            }
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn appendleft(&mut self, val: &T) {
            self._data.push_front(val.clone().clone());
            let maxlen_opt: Option<i64> = self.maxlen;
            if let Some(maxlen_opt) = maxlen_opt {
                let maxlen: i64 = maxlen_opt;
                if ((self._data.len() as i64) > maxlen) {
                    self._data.pop_back();
                }
            }
        }
    }
    impl<T> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn pop(&mut self) -> Option<T> {
            if ((self._data.len() as i64) == (0_i64)) {
                return None;
            }
            Some({
                let Some(__sifr_nonempty_pop_value) = self._data.pop_back() else {
                    unreachable!("compiler-verified non-empty pop should return Some");
                };
                __sifr_nonempty_pop_value
            })
        }
    }
    impl<T> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn popleft(&mut self) -> Option<T> {
            if ((self._data.len() as i64) == (0_i64)) {
                return None;
            }
            Some({
                let Some(__sifr_nonempty_pop_value) = self._data.pop_front() else {
                    unreachable!("compiler-verified non-empty pop should return Some");
                };
                __sifr_nonempty_pop_value
            })
        }
    }
    impl<T> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn len(&self) -> i64 {
            self._data.len() as i64
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn to_list(&self) -> Vec<T> {
            let mut result: Vec<T> = vec![];
            for v in self._data.clone().iter().cloned() {
                result.push(v.clone().clone());
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
                self._data.push_back(v.clone().clone());
            }
            let maxlen_opt: Option<i64> = self.maxlen;
            if let Some(maxlen_opt) = maxlen_opt {
                let maxlen: i64 = maxlen_opt;
                while ((self._data.len() as i64) > maxlen) {
                    self._data.pop_front();
                }
            }
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn extendleft(&mut self, items: &Vec<T>) {
            for v in items.iter().cloned() {
                self._data.push_front(v.clone().clone());
            }
            let maxlen_opt: Option<i64> = self.maxlen;
            if let Some(maxlen_opt) = maxlen_opt {
                let maxlen: i64 = maxlen_opt;
                while ((self._data.len() as i64) > maxlen) {
                    self._data.pop_back();
                }
            }
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn copy(&self) -> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
            __SifrStdlib_sifr_x2ecollections_x2edeque::new(Some(self.to_list()), self.maxlen)
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn reverse(&mut self) {
            let mut items: Vec<T> = self.to_list();
            items.reverse();
            self._data.clear();
            for item in items.iter().cloned() {
                self._data.push_back(item.clone().clone());
            }
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn rotate(&mut self, n: i64) {
            let length: i64 = self._data.len() as i64;
            if length == (0_i64) {
                return;
            }
            let mut steps: i64 = n % length;
            if steps < (0_i64) {
                steps += length;
            }
            let mut count: i64 = 0_i64;
            while count < steps {
                let value: Option<T> = self._data.pop_back();
                if let Some(value) = value {
                    self._data.push_front(value.clone().clone());
                }
                count += 1_i64;
            }
        }
    }
    impl<T: Clone + PartialEq> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn count(&self, value: &T) -> i64 {
            let mut total: i64 = 0_i64;
            for item in self._data.clone().iter().cloned() {
                if item == *value {
                    total += 1_i64;
                }
            }
            total
        }
    }
    impl<T: Clone + PartialEq> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn index(&self, value: &T, start: i64, stop: Option<i64>) -> Option<i64> {
            let size: i64 = self._data.len() as i64;
            let mut begin: i64 = start;
            if begin < (0_i64) {
                begin = size + begin;
                if begin < (0_i64) {
                    begin = 0_i64;
                }
            }
            let mut end: i64 = size;
            if let Some(stop) = stop {
                end = stop;
                if end < (0_i64) {
                    end = size + end;
                }
                if end < (0_i64) {
                    end = 0_i64;
                }
                if end > size {
                    end = size;
                }
            }
            let mut i: i64 = begin;
            while i < end {
                let current: Option<T> = {
                    let __sifr_index_list = &self._data;
                    let __sifr_index_i = i;
                    let __sifr_index_norm = if __sifr_index_i < 0 {
                        ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                    } else {
                        __sifr_index_i as usize
                    };
                    __sifr_index_list.get(__sifr_index_norm).cloned()
                };
                if let Some(current) = current {
                    if current == *value {
                        return Some(i);
                    }
                }
                i += 1_i64;
            }
            None
        }
    }
    impl<T: Clone + PartialEq> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn remove(&mut self, value: &T) {
            let idx: Option<i64> = self.index(value, 0_i64, None);
            if let Some(idx) = idx {
                let mut rebuilt: Vec<T> = vec![];
                let mut i: i64 = 0_i64;
                while (i < (self._data.len() as i64)) {
                    let current: Option<T> = Some(self._data.clone()[i as usize].clone());
                    if let Some(current) = current {
                        if i != idx {
                            rebuilt.push(current.clone().clone());
                        }
                    }
                    i += 1_i64;
                }
                self._data.clear();
                for item in rebuilt.iter().cloned() {
                    self._data.push_back(item.clone().clone());
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
fn from_list<
    T: Clone + ::std::fmt::Display + PartialOrd + ::std::hash::Hash + Eq + 'static,
>(items: &Vec<T>) -> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    let mut counts: HashMap<T, i64> = HashMap::from([]);
    for item in items.iter().cloned() {
        let val: Option<i64> = counts.get(&item).copied();
        if let Some(val) = val {
            counts.insert(item.clone(), val + (1_i64));
        } else {
            counts.insert(item.clone(), 1_i64);
        }
    }
    __SifrStdlib_sifr_x2ecollections_x2eCounter::new(Some(counts), None)
}
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0_i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i += 1_i64;
    }
}
fn collect_set_and_counter_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let left: HashSet<i64> = (vec![1_i64, 2_i64, 3_i64])
        .into_iter()
        .collect::<std::collections::HashSet<_>>();
    let right: HashSet<i64> = (vec![3_i64, 4_i64, 5_i64])
        .into_iter()
        .collect::<std::collections::HashSet<_>>();
    actual
        .push(
            (left.union(&right).cloned().collect::<std::collections::HashSet<_>>().len()
                as i64) == (5_i64),
        );
    actual
        .push(
            (left
                .intersection(&right)
                .cloned()
                .collect::<std::collections::HashSet<_>>()
                .len() as i64) == (1_i64),
        );
    let counts: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(
        &vec![
            "x".to_string(), "y".to_string(), "x".to_string(), "z".to_string(), "x"
            .to_string(), "y".to_string()
        ],
    );
    actual.push(counts.get(&"x".to_string(), 0_i64) == (3_i64));
    actual
        .push(
            (format!("{:?}", counts.most_common(Some(2_i64)))).as_str()
                == ("[(\"x\", 3), (\"y\", 2)]".to_string()).as_str(),
        );
    actual
}
fn collect_deque_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut d: __SifrStdlib_sifr_x2ecollections_x2edeque<i64> = __SifrStdlib_sifr_x2ecollections_x2edeque::new(
        None,
        Some(2_i64),
    );
    d.append(&(10_i64));
    d.append(&(20_i64));
    d.append(&(30_i64));
    actual
        .push(
            ((d.len() as i64) == (2_i64))
                && (({
                    let Some(__sifr_nonempty_pop_value) = d.popleft() else {
                        unreachable!(
                            "compiler-verified non-empty pop should return Some"
                        );
                    };
                    __sifr_nonempty_pop_value
                }) == (20_i64)),
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
