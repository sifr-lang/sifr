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
use ::std::collections::VecDeque;
fn bisect_right<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    a: &Vec<T>,
    x: &T,
    lo: i64,
    hi: Option<i64>,
) -> i64 {
    let mut left: i64 = lo;
    if left < (0_i64) {
        left = 0_i64;
    }
    let mut right: i64 = a.len() as i64;
    if hi.is_none() {
        right = a.len() as i64;
    } else {
        if let Some(hi) = hi {
            if hi < (0_i64) {
                right = 0_i64;
            } else {
                if (hi > (a.len() as i64)) {
                    right = a.len() as i64;
                } else {
                    right = hi;
                }
            }
        }
    }
    while left < right {
        let mid: i64 = (left + right) / (2_i64);
        let val: Option<T> = {
            let __sifr_index_list = &a;
            let __sifr_index_i = mid;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(val) = val {
            if *x < val {
                right = mid;
            } else {
                left = mid + (1_i64);
            }
        } else {
            left = mid + (1_i64);
        }
    }
    left
}
fn insort_right<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    a: &mut Vec<T>,
    x: &T,
    lo: i64,
    hi: Option<i64>,
) {
    let pos: i64 = bisect_right(a, x, lo, hi);
    a.insert(pos as usize, x.clone());
}
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
fn _sift_down<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &mut Vec<T>,
    mut pos: i64,
    n: i64,
) {
    let mut done: bool = false;
    while !done {
        let mut smallest: i64 = pos;
        let left: i64 = ((2_i64) * pos) + (1_i64);
        let right: i64 = ((2_i64) * pos) + (2_i64);
        if left < n {
            let s_val: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = smallest;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let l_val: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = left;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
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
        if right < n {
            let s_val2: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = smallest;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let r_val: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = right;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
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
        if smallest == pos {
            done = true;
        } else {
            let tmp_pos: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = pos;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let tmp_sm: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = smallest;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(tmp_pos) = tmp_pos {
                if let Some(tmp_sm) = tmp_sm {
                    {
                        let __idx_raw = pos;
                        let __idx_norm = if __idx_raw < 0 {
                            (data.len() as i64) + __idx_raw
                        } else {
                            __idx_raw
                        };
                        if __idx_norm >= 0 {
                            if let Some(__elem) = data.get_mut(__idx_norm as usize) {
                                *__elem = tmp_sm.clone();
                            }
                        }
                    }
                    {
                        let __idx_raw = smallest;
                        let __idx_norm = if __idx_raw < 0 {
                            (data.len() as i64) + __idx_raw
                        } else {
                            __idx_raw
                        };
                        if __idx_norm >= 0 {
                            if let Some(__elem) = data.get_mut(__idx_norm as usize) {
                                *__elem = tmp_pos.clone();
                            }
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
    mut pos: i64,
) {
    let mut done: bool = false;
    while !done {
        if pos <= (0_i64) {
            done = true;
        } else {
            let parent: i64 = (pos - (1_i64)) / (2_i64);
            let p_val: Option<T> = {
                let __sifr_index_list = &heap;
                let __sifr_index_i = parent;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let c_val: Option<T> = {
                let __sifr_index_list = &heap;
                let __sifr_index_i = pos;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(p_val) = p_val {
                if let Some(c_val) = c_val {
                    if c_val < p_val {
                        {
                            let __idx_raw = parent;
                            let __idx_norm = if __idx_raw < 0 {
                                (heap.len() as i64) + __idx_raw
                            } else {
                                __idx_raw
                            };
                            if __idx_norm >= 0 {
                                if let Some(__elem) = heap.get_mut(__idx_norm as usize) {
                                    *__elem = c_val.clone();
                                }
                            }
                        }
                        {
                            let __idx_raw = pos;
                            let __idx_norm = if __idx_raw < 0 {
                                (heap.len() as i64) + __idx_raw
                            } else {
                                __idx_raw
                            };
                            if __idx_norm >= 0 {
                                if let Some(__elem) = heap.get_mut(__idx_norm as usize) {
                                    *__elem = p_val.clone();
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
fn heapify<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(data: &mut Vec<T>) {
    "Convert list to a min-heap in-place. O(n) time.".to_string();
    let n: i64 = data.len() as i64;
    let mut i: i64 = (n / (2_i64)) - (1_i64);
    while i >= (0_i64) {
        _sift_down(data, i, n);
        i -= 1_i64;
    }
}
fn heappush<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
    item: &T,
) {
    "Push item onto the heap in-place. O(log n) time.".to_string();
    heap.push(item.clone().clone());
    let pos: i64 = (heap.len() as i64) - (1_i64);
    _sift_up(heap, pos);
}
fn heappop<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
) -> Option<T> {
    "Pop and return the smallest item. Heap is modified in-place. O(log n) time.\n    Returns None if the heap is empty."
        .to_string();
    let n: i64 = heap.len() as i64;
    if n == (0_i64) {
        return None;
    }
    let top: Option<T> = Some(heap[(0_i64) as usize].clone());
    let last: Option<T> = {
        let __sifr_index_list = &heap;
        let __sifr_index_i = n - (1_i64);
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    {
        let Some(__sifr_nonempty_pop_value) = heap.pop() else {
            unreachable!("compiler-verified non-empty pop should return Some");
        };
        __sifr_nonempty_pop_value
    };
    let n2: i64 = heap.len() as i64;
    if n2 > (0_i64) {
        if let Some(last) = last {
            {
                let __idx_raw = 0_i64;
                let __idx_norm = if __idx_raw < 0 {
                    (heap.len() as i64) + __idx_raw
                } else {
                    __idx_raw
                };
                if __idx_norm >= 0 {
                    if let Some(__elem) = heap.get_mut(__idx_norm as usize) {
                        *__elem = last.clone();
                    }
                }
            }
        }
        _sift_down(heap, 0_i64, n2);
    }
    top
}
fn heapreplace<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
    item: T,
) -> Option<T> {
    if (heap.len() as i64) == (0_i64) {
        return None;
    }
    let top: Option<T> = Some(heap[(0_i64) as usize].clone());
    {
        let __idx_raw = 0_i64;
        let __idx_norm = if __idx_raw < 0 {
            (heap.len() as i64) + __idx_raw
        } else {
            __idx_raw
        };
        if __idx_norm >= 0 {
            if let Some(__elem) = heap.get_mut(__idx_norm as usize) {
                *__elem = item.clone();
            }
        }
    }
    let heap_len: i64 = heap.len() as i64;
    _sift_down(heap, 0_i64, heap_len);
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
    println!("{:?}", counts.most_common(None));
    let mut queue: __SifrStdlib_sifr_x2ecollections_x2edeque<i64> = __SifrStdlib_sifr_x2ecollections_x2edeque::new(
        Some(vec![1_i64, 2_i64, 3_i64]),
        Some(4_i64),
    );
    queue.rotate(1_i64);
    queue.appendleft(&(0_i64));
    println!("{:?}", queue.to_list());
    let mut ordered: Vec<i64> = vec![1_i64, 3_i64, 5_i64];
    insort_right(&mut ordered, &(4_i64), 0_i64, None);
    println!("{}", bisect_right(& ordered, & (4_i64), 0_i64, None));
    let mut heap: Vec<i64> = vec![1_i64, 3_i64, 5_i64];
    heapify(&mut heap);
    println!(
        "{}", (heappushpop(& mut heap, & (2_i64))).map_or("None".to_string().to_string(),
        | __v | format!("{}", __v))
    );
    println!(
        "{}", (heapreplace(& mut heap, 4_i64)).map_or("None".to_string().to_string(), |
        __v | format!("{}", __v))
    );
}
