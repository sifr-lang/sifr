// src/main.rs
use ::std::collections::HashMap;

// --- stdlib: sifr.bisect ---
fn bisect_left<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
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
            if val < *x {
                left = mid + (1_i64);
            } else {
                right = mid;
            }
        } else {
            left = mid + (1_i64);
        }
    }
    left
}
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
fn insort_left<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    a: &mut Vec<T>,
    x: &T,
    lo: i64,
    hi: Option<i64>,
) {
    let pos: i64 = bisect_left(a, x, lo, hi);
    a.insert(pos as usize, x.clone());
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

// --- stdlib: sifr.collections ---
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2ecollections_x2eCounter<T: std::hash::Hash + Eq> {
    counts: HashMap<T, i64>,
}
impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    fn new(source: Option<HashMap<T, i64>>, iterable: Option<Vec<T>>) -> Self {
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
    fn __iter__(&self) -> Vec<T> {
        Box::new((self.counts.keys().cloned().collect::<Vec<_>>()).into_iter())
            .collect::<Vec<_>>()
    }
}
impl<T: ::std::hash::Hash + Eq> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    fn __getitem__(&self, key: &T) -> i64 {
        let val: Option<i64> = self.counts.get(&key).copied();
        if let Some(val) = val {
            return val;
        }
        0_i64
    }
}
impl<T: ::std::hash::Hash + Eq> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    fn get(&self, key: &T, default: i64) -> i64 {
        let val: Option<i64> = self.counts.get(&key).copied();
        if let Some(val) = val {
            return val;
        }
        default
    }
}
impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    fn increment(&mut self, key: &T) {
        let val: Option<i64> = self.counts.get(&key).copied();
        if let Some(val) = val {
            self.counts.insert(key.clone(), val + (1_i64));
        } else {
            self.counts.insert(key.clone(), 1_i64);
        }
    }
}
impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    fn total(&self) -> i64 {
        let mut total: i64 = 0_i64;
        for count in self.counts.values().cloned().collect::<Vec<_>>() {
            total += count;
        }
        total
    }
}
impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    fn most_common(&self, n: Option<i64>) -> Vec<(T, i64)> {
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
    fn keys(&self) -> Vec<T> {
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
    fn items(&self) -> Vec<(T, i64)> {
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
    fn values(&self) -> Vec<i64> {
        self.counts.values().cloned().collect::<Vec<_>>()
    }
}
impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    fn copy(&self) -> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        __SifrStdlib_sifr_x2ecollections_x2eCounter::new(Some(self.counts.clone()), None)
    }
}
impl<T: ::std::hash::Hash + Eq> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    fn clear(&mut self) {
        self.counts = HashMap::from([]);
    }
}
impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    fn update(&mut self, other: &__SifrStdlib_sifr_x2ecollections_x2eCounter<T>) {
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
    fn subtract(&mut self, other: &__SifrStdlib_sifr_x2ecollections_x2eCounter<T>) {
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
    fn elements(&self) -> Vec<T> {
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

// --- stdlib: sifr.heapq ---
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
fn nsmallest<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    n: i64,
    data: &Vec<T>,
) -> Vec<T> {
    let mut heap: Vec<T> = data.clone();
    heapify(&mut heap);
    let mut result: Vec<T> = vec![];
    let mut count: i64 = 0_i64;
    while count < n {
        if ((heap.len() as i64) == (0_i64)) {
            return result;
        }
        let val: Option<T> = heappop(&mut heap);
        if let Some(val) = val {
            result.push(val.clone().clone());
        }
        count += 1_i64;
    }
    result
}
fn nlargest<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    n: i64,
    data: &Vec<T>,
) -> Vec<T> {
    if n <= (0_i64) {
        return vec![];
    }
    if (n >= (data.len() as i64)) {
        let mut result: Vec<T> = vec![];
        for val in data.iter().cloned() {
            result.push(val.clone().clone());
        }
        return result;
    }
    let mut heap: Vec<T> = data.clone();
    heapify(&mut heap);
    let mut all_sorted: Vec<T> = vec![];
    while ((heap.len() as i64) > (0_i64)) {
        let val2: Option<T> = heappop(&mut heap);
        if let Some(val2) = val2 {
            all_sorted.push(val2.clone().clone());
        }
    }
    let mut result2: Vec<T> = vec![];
    let mut i: i64 = (all_sorted.len() as i64) - (1_i64);
    let mut count: i64 = 0_i64;
    while count < n {
        if i < (0_i64) {
            return result2;
        }
        let v: Option<T> = {
            let __sifr_index_list = &all_sorted;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(v) = v {
            result2.push(v.clone().clone());
        }
        i -= 1_i64;
        count += 1_i64;
    }
    result2
}

// --- stdlib: sifr.itertools ---
fn chain<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    iterables: &Vec<Vec<T>>,
) -> Box<dyn Iterator<Item = T>> {
    let iterables = iterables.clone();
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<T> = Vec::new().into_iter();
    Box::new(
        ::std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<T> = Vec::new();
                for iterable in iterables.iter().cloned() {
                    for item in iterable.iter().cloned() {
                        _yields.push(item);
                    }
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            __sifr_generator_iter.next()
        }),
    )
}
// --- end stdlib ---

fn demo_heapq() {
    println!("=== Section 1: heapq with mut params ===");
    let mut data: Vec<i64> = vec![5_i64, 3_i64, 8_i64, 1_i64, 2_i64, 7_i64, 4_i64];
    heapify(&mut data);
    println!("heapified (min at root):");
    let min_val: Option<i64> = {
    let __sifr_index_list = &data;
    let __sifr_index_i = 0_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
    if let Some(min_val) = min_val {
        println!("{}", min_val);
    }
    heappush(&mut data, &(0_i64));
    println!("after push(0), new min:");
    let new_min: Option<i64> = {
    let __sifr_index_list = &data;
    let __sifr_index_i = 0_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
    if let Some(new_min) = new_min {
        println!("{}", new_min);
    }
    let popped: Option<i64> = heappop(&mut data);
    if let Some(popped) = popped {
        println!("popped:");
        println!("{}", popped);
    }
    println!("remaining size:");
    println!("{}", data.len() as i64);
    let items: Vec<i64> = vec![9_i64, 3_i64, 7_i64, 1_i64, 5_i64, 6_i64, 2_i64, 8_i64, 4_i64];
    let small3: Vec<i64> = nsmallest(3_i64, &items);
    let large3: Vec<i64> = nlargest(3_i64, &items);
    println!("3 smallest:");
    println!("{:?}", small3);
    println!("3 largest:");
    println!("{:?}", large3);
    println!("items still valid, length:");
    println!("{}", items.len() as i64);
}

fn demo_bisect() {
    println!("=== Section 2: bisect_right insort_right with mut params ===");
    let mut sorted_ints: Vec<i64> = vec![1_i64, 3_i64, 5_i64, 7_i64, 9_i64];
    let pos_left: i64 = bisect_left(&sorted_ints, &(6_i64), 0_i64, None);
    let pos_right: i64 = bisect_right(&sorted_ints, &(5_i64), 0_i64, None);
    println!("insert 6 at position (left):");
    println!("{}", pos_left);
    println!("insert after 5 at position (right):");
    println!("{}", pos_right);
    insort_left(&mut sorted_ints, &(6_i64), 0_i64, None);
    println!("after insort_left(6):");
    println!("{:?}", sorted_ints);
    let mut data: Vec<i64> = vec![1_i64, 2_i64, 2_i64, 3_i64];
    insort_right(&mut data, &(2_i64), 0_i64, None);
    println!("after insort_right(2) with duplicates:");
    println!("{:?}", data);
    insort_left(&mut sorted_ints, &(0_i64), 0_i64, None);
    insort_right(&mut sorted_ints, &(10_i64), 0_i64, None);
    println!("after more inserts:");
    println!("{:?}", sorted_ints);
}

fn demo_itertools() {
    println!("=== Section 3: itertools chain ===");
    let a: Vec<i64> = vec![1_i64, 2_i64, 3_i64];
    let b: Vec<i64> = vec![4_i64, 5_i64, 6_i64];
    let result: Vec<i64> = chain(&vec![(a).clone(), (b).clone()]).collect::<Vec<_>>();
    println!("chain (borrow both):");
    println!("{:?}", result);
    println!("a still usable:");
    println!("{}", a.len() as i64);
    println!("b still usable:");
    println!("{}", b.len() as i64);
    let x: Vec<i64> = vec![10_i64, 20_i64, 30_i64];
    let y: Vec<i64> = vec![40_i64, 50_i64, 60_i64];
    let combined: Vec<i64> = chain(&vec![(x).clone(), (y).clone()]).collect::<Vec<_>>();
    println!("chain result:");
    println!("{:?}", combined);
}

fn demo_counter() {
    println!("=== Section 4: Counter with native dict[str, int] ===");
    let words: Vec<String> = vec!["apple".to_string(), "banana".to_string(), "apple".to_string(), "cherry".to_string(), "banana".to_string(), "apple".to_string()];
    let mut c: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(&words);
    println!("apple count:");
    println!("{}", c.get(&"apple".to_string(), 0_i64));
    println!("banana count:");
    println!("{}", c.get(&"banana".to_string(), 0_i64));
    println!("missing key returns 0:");
    println!("{}", c.get(&"missing".to_string(), 0_i64));
    println!("total elements:");
    println!("{}", c.total());
    c.increment(&"cherry".to_string());
    c.increment(&"cherry".to_string());
    println!("cherry after 2 increments:");
    println!("{}", c.get(&"cherry".to_string(), 0_i64));
    let top: Vec<(String, i64)> = c.most_common(Some(1_i64));
    println!("top 1 most common:");
    println!("{:?}", top);
    let keys: Vec<String> = c.keys();
    println!("unique keys count:");
    println!("{}", keys.len() as i64);
}

fn main() {
    demo_heapq();
    demo_bisect();
    demo_itertools();
    demo_counter();
    println!("=== borrow_stdlib demo complete ===");
}
