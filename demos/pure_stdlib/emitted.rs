// src/main.rs
use ::std::collections::HashMap;

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

// --- stdlib: sifr.functools ---
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

// --- stdlib: sifr.itertools ---
fn _compress_impl<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    selectors: &Vec<bool>,
) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    let mut i: i64 = 0_i64;
    while (i < (data.len() as i64)) {
        if (i >= (selectors.len() as i64)) {
            i = data.len() as i64;
        } else {
            let sel: Option<bool> = {
                let __sifr_index_list = &selectors;
                let __sifr_index_i = i;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).copied()
            };
            let val: Option<T> = Some(data[i as usize].clone());
            if let Some(sel) = sel {
                if let Some(val) = val {
                    if sel {
                        result.push(val.clone().clone());
                    }
                }
            }
            i += 1_i64;
        }
    }
    result
}
fn _takewhile_impl<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool,
    data: &Vec<T>,
) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    let mut i: i64 = 0_i64;
    while (i < (data.len() as i64)) {
        let val: Option<T> = Some(data[i as usize].clone());
        if let Some(val) = val {
            if pred(&val) {
                result.push(val.clone().clone());
            } else {
                i = data.len() as i64;
            }
        }
        i += 1_i64;
    }
    result
}
fn _zip_longest_impl<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    a: &Vec<T>,
    b: &Vec<T>,
    fill: &T,
) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = vec![];
    let len_a: i64 = a.len() as i64;
    let len_b: i64 = b.len() as i64;
    let mut max_len: i64 = len_a;
    if len_b > max_len {
        max_len = len_b;
    }
    let mut i: i64 = 0_i64;
    while i < max_len {
        let mut pair: Vec<T> = vec![];
        if i < len_a {
            let va: Option<T> = Some(a[i as usize].clone());
            if let Some(va) = va {
                pair.push(va.clone().clone());
            } else {
                pair.push(fill.clone().clone());
            }
        } else {
            pair.push(fill.clone().clone());
        }
        if i < len_b {
            let vb: Option<T> = Some(b[i as usize].clone());
            if let Some(vb) = vb {
                pair.push(vb.clone().clone());
            } else {
                pair.push(fill.clone().clone());
            }
        } else {
            pair.push(fill.clone().clone());
        }
        result.push(pair.clone());
        i += 1_i64;
    }
    result
}
fn _collect_iterable<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: Vec<T>,
) -> Vec<T> {
    let mut collected: Vec<T> = vec![];
    for item in data.iter().cloned() {
        collected.push(item.clone().clone());
    }
    collected
}
fn count(start: i64, step: i64) -> Box<dyn Iterator<Item = i64>> {
    count_from(start, step, 10000_i64)
}
fn accumulate<
    T: Clone + ::std::fmt::Display + PartialOrd + 'static + ::std::ops::Add<Output = T>,
>(data: &Vec<T>, initial: Option<T>) -> Box<dyn Iterator<Item = T>> {
    let mut result: Vec<T> = vec![];
    if let Some(initial) = initial {
        result.push(initial.clone().clone());
    }
    for item in data.iter().cloned() {
        if ((result.len() as i64) == (0_i64)) {
            result.push(item.clone().clone());
        } else {
            let prev: Option<T> = {
                let __sifr_index_list = &result;
                let __sifr_index_i = (result.len() as i64) - (1_i64);
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(prev) = prev {
                let next_val: T = prev + item;
                result.push(next_val.clone().clone());
            }
        }
    }
    Box::new(result.into_iter())
}
fn compress<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    selectors: &Vec<bool>,
) -> Box<dyn Iterator<Item = T>> {
    let data_owned: Vec<T> = _collect_iterable(
        ((data).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let mut selectors_owned: Vec<bool> = vec![];
    for selector in selectors.iter().copied() {
        selectors_owned.push(selector);
    }
    let result: Vec<T> = _compress_impl(&data_owned, &selectors_owned);
    Box::new(result.into_iter())
}
fn dropwhile<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool,
    data: &Vec<T>,
) -> Box<dyn Iterator<Item = T>> {
    let mut result: Vec<T> = vec![];
    let mut dropping: bool = true;
    for val in data.iter().cloned() {
        if dropping {
            if !(pred(&val)) {
                dropping = false;
                result.push(val.clone().clone());
            }
        } else {
            result.push(val.clone().clone());
        }
    }
    Box::new(result.into_iter())
}
fn takewhile<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool,
    data: &Vec<T>,
) -> Box<dyn Iterator<Item = T>> {
    let data_owned: Vec<T> = _collect_iterable(
        ((data).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let result: Vec<T> = _takewhile_impl(pred, &data_owned);
    Box::new(result.into_iter())
}
fn filterfalse<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool,
    data: &Vec<T>,
) -> Box<dyn Iterator<Item = T>> {
    let mut result: Vec<T> = vec![];
    for val in data.iter().cloned() {
        if !(pred(&val)) {
            result.push(val.clone().clone());
        }
    }
    Box::new(result.into_iter())
}
fn zip_longest<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    a: &Vec<T>,
    b: &Vec<T>,
    fill: &T,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    let a_owned: Vec<T> = _collect_iterable(
        ((a).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let b_owned: Vec<T> = _collect_iterable(
        ((b).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let result: Vec<Vec<T>> = _zip_longest_impl(&a_owned, &b_owned, fill);
    Box::new(result.into_iter())
}
fn count_from(start: i64, step: i64, n: i64) -> Box<dyn Iterator<Item = i64>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<i64> = Vec::new().into_iter();
    Box::new(
        ::std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<i64> = Vec::new();
                let mut i: i64 = 0_i64;
                let mut current: i64 = start;
                while i < n {
                    _yields.push(current);
                    current += step;
                    i += 1_i64;
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            __sifr_generator_iter.next()
        }),
    )
}
fn cycle<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    n: i64,
) -> Box<dyn Iterator<Item = T>> {
    let materialized: Vec<T> = _collect_iterable(
        ((data).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let mut result: Vec<T> = vec![];
    if ((materialized.len() as i64) > (0_i64)) {
        let mut i: i64 = 0_i64;
        while i < n {
            let idx: i64 = i % (materialized.len() as i64);
            let val: Option<T> = {
                let __sifr_index_list = &materialized;
                let __sifr_index_i = idx;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(val) = val {
                result.push(val.clone().clone());
            }
            i += 1_i64;
        }
    }
    Box::new(result.into_iter())
}

// --- stdlib: _sifr.math ---
const PI: f64 = 3.141592653589793_f64;
const E: f64 = 2.718281828459045_f64;
const TAU: f64 = 6.283185307179586_f64;
const INF: f64 = f64::INFINITY;
const NAN: f64 = f64::NAN;
fn sqrt(x: f64) -> f64 {
    ::sifr_stdlib::math::sqrt(x)
}
fn floor(x: f64) -> i64 {
    ::sifr_stdlib::math::floor(x).to_i64_saturating()
}
fn ceil(x: f64) -> i64 {
    ::sifr_stdlib::math::ceil(x).to_i64_saturating()
}
fn abs_val(x: f64) -> f64 {
    ::sifr_stdlib::math::abs_val(x)
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
fn round_val(x: f64) -> i64 {
    ::sifr_stdlib::math::round_val(x).to_i64_saturating()
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
fn trunc(x: f64) -> i64 {
    ::sifr_stdlib::math::trunc(x).to_i64_saturating()
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
fn isqrt(n: i64) -> i64 {
    ::sifr_stdlib::math::isqrt(::sifr_runtime::interop::SifrIntBridge::from(n))
        .to_i64_saturating()
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
fn ldexp(m: f64, e: i64) -> f64 {
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

// --- stdlib: sifr.math ---
fn factorial(n: i64) -> i64 {
    if n < (0_i64) {
        return 0_i64;
    }
    let mut result: i64 = 1_i64;
    let mut i: i64 = 2_i64;
    while i <= n {
        result *= i;
        i += 1_i64;
    }
    result
}
fn gcd(a: i64, b: i64) -> i64 {
    let mut x: i64 = a;
    let mut y: i64 = b;
    if x < (0_i64) {
        x = (0_i64) - x;
    }
    if y < (0_i64) {
        y = (0_i64) - y;
    }
    while y != (0_i64) {
        let temp: i64 = y;
        y = x % y;
        x = temp;
    }
    x
}
fn lcm(a: i64, b: i64) -> i64 {
    if a == (0_i64) {
        return 0_i64;
    }
    if b == (0_i64) {
        return 0_i64;
    }
    let g: i64 = gcd(a, b);
    let mut x: i64 = a;
    if x < (0_i64) {
        x = (0_i64) - x;
    }
    let mut y: i64 = b;
    if y < (0_i64) {
        y = (0_i64) - y;
    }
    (x / g) * y
}
fn comb(n: i64, k: i64) -> i64 {
    if k < (0_i64) {
        return 0_i64;
    }
    if k > n {
        return 0_i64;
    }
    if k == (0_i64) {
        return 1_i64;
    }
    if k == n {
        return 1_i64;
    }
    let mut r: i64 = k;
    if r > (n - k) {
        r = n - k;
    }
    let mut result: i64 = 1_i64;
    let mut i: i64 = 0_i64;
    while i < r {
        result *= n - i;
        result /= i + (1_i64);
        i += 1_i64;
    }
    result
}
fn perm(n: i64, k: i64) -> i64 {
    if k < (0_i64) {
        return 0_i64;
    }
    if k > n {
        return 0_i64;
    }
    let mut result: i64 = 1_i64;
    let mut i: i64 = 0_i64;
    while i < k {
        result *= n - i;
        i += 1_i64;
    }
    result
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
fn prod(data: &Vec<i64>) -> i64 {
    let mut result: i64 = 1_i64;
    for val in data.iter().copied() {
        result *= val;
    }
    result
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
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(m) = m else {
        return NAN;
    };
    m
}
fn frexp_exponent(x: f64) -> i64 {
    let parts: Vec<f64> = frexp(x);
    let exp_val: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(exp_val) = exp_val else {
        return 0_i64;
    };
    trunc(exp_val)
}
fn modf_fractional(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let f: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(f) = f else {
        return NAN;
    };
    f
}
fn modf_integral(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let i: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(i) = i else {
        return NAN;
    };
    i
}
fn pow(x: f64, y: f64) -> f64 {
    pow_val(x, y)
}

// --- stdlib: _sifr.crypto ---
fn random_int(min: i64, max: i64) -> i64 {
    ::sifr_stdlib::random::random_int(
            ::sifr_runtime::interop::SifrIntBridge::from(min),
            ::sifr_runtime::interop::SifrIntBridge::from(max),
        )
        .to_i64_saturating()
}
fn random_float() -> f64 {
    ::sifr_stdlib::random::random_float()
}
fn random_uniform(min: f64, max: f64) -> f64 {
    ::sifr_stdlib::random::random_uniform(min, max)
}
fn random_randrange(start: i64, stop: i64, step: i64) -> Result<i64, ValueError> {
    ::sifr_stdlib::random::random_randrange(
            ::sifr_runtime::interop::SifrIntBridge::from(start),
            ::sifr_runtime::interop::SifrIntBridge::from(stop),
            ::sifr_runtime::interop::SifrIntBridge::from(step),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn random_gauss(mu: f64, sigma: f64) -> f64 {
    ::sifr_stdlib::random::random_gauss(mu, sigma)
}
fn random_module_state_words() -> Vec<i64> {
    ::sifr_stdlib::random::random_module_state_words()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn random_module_state_index() -> i64 {
    ::sifr_stdlib::random::random_module_state_index().to_i64_saturating()
}
fn random_module_state_gauss_next() -> Option<f64> {
    ::sifr_stdlib::random::random_module_state_gauss_next()
}
fn random_module_set_state(
    words: &Vec<i64>,
    index: i64,
    gauss_next: Option<f64>,
) -> Result<(), ValueError> {
    ::sifr_stdlib::random::random_module_set_state(
            &words
                .iter()
                .copied()
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
    wrapcol: i64,
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

// --- stdlib: _sifr.time ---
fn time_now() -> f64 {
    ::sifr_stdlib::time::time_now()
}
fn time_format(epoch: f64, fmt: &String) -> String {
    ::sifr_stdlib::time::time_format(epoch, fmt)
}
fn perf_counter() -> f64 {
    ::sifr_stdlib::time::perf_counter()
}
fn sleep(seconds: f64) {
    ::sifr_stdlib::time::sleep(seconds);
}
fn monotonic() -> f64 {
    ::sifr_stdlib::time::monotonic()
}
fn strptime(s: &String, fmt: &String) -> Result<String, ValueError> {
    ::sifr_stdlib::time::strptime(s, fmt)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _strptime_intrinsic(s: &String, fmt: &String) -> Result<String, ValueError> {
    ::sifr_stdlib::time::strptime(s, fmt)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn gmtime(epoch: f64) -> String {
    ::sifr_stdlib::time::gmtime(epoch)
}
fn _gmtime_intrinsic(epoch: f64) -> String {
    ::sifr_stdlib::time::gmtime(epoch)
}
fn localtime(epoch: f64) -> String {
    ::sifr_stdlib::time::localtime(epoch)
}
fn _localtime_intrinsic(epoch: f64) -> String {
    ::sifr_stdlib::time::localtime(epoch)
}
fn time_strptime(s: &String, fmt: &String) -> Result<Vec<i64>, ValueError> {
    ::sifr_stdlib::time::time_strptime(s, fmt)
        .map(|__sifr_bridge_ok| {
            __sifr_bridge_ok
                .into_iter()
                .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
                .collect()
        })
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn time_gmtime() -> Vec<i64> {
    ::sifr_stdlib::time::time_gmtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn time_localtime() -> Vec<i64> {
    ::sifr_stdlib::time::time_localtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}

// --- stdlib: sifr.random ---
const _MT_N: i64 = 624_i64;
const _MT_M: i64 = 397_i64;
const _MT_MATRIX_A: i64 = 2567483615_i64;
const _MT_UPPER_MASK: i64 = 2147483648_i64;
const _MT_LOWER_MASK: i64 = 2147483647_i64;
const _MT_F: i64 = 1812433253_i64;
const _MT_WORD_MASK: i64 = 4294967295_i64;
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2erandom_x2eRandomState {
    version: i64,
    state_words: Vec<i64>,
    index: i64,
    gauss_next: Option<f64>,
}
impl __SifrStdlib_sifr_x2erandom_x2eRandomState {
    fn new(
        version: i64,
        state_words: Vec<i64>,
        index: i64,
        gauss_next: Option<f64>,
    ) -> Self {
        let __sifr_field_init_0: i64 = version;
        let __sifr_field_init_1: Vec<i64> = state_words;
        let __sifr_field_init_2: i64 = index;
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
    _state_words: Vec<i64>,
    _index: i64,
    _gauss_next: Option<f64>,
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn new(seed_value: Option<i64>) -> Self {
        let normalized_seed: i64 = _normalize_seed_input(seed_value);
        let __sifr_field_init_0: Vec<i64> = _seed_words_from_seed(normalized_seed);
        let __sifr_field_init_1: i64 = _MT_N;
        let __sifr_field_init_2: Option<f64> = None;
        Self {
            _state_words: __sifr_field_init_0,
            _index: __sifr_field_init_1,
            _gauss_next: __sifr_field_init_2,
        }
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn seed(&mut self, seed_value: Option<i64>) {
        let normalized_seed: i64 = _normalize_seed_input(seed_value);
        self._state_words = _seed_words_from_seed(normalized_seed);
        self._index = _MT_N;
        self._gauss_next = None;
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn _twist(&mut self) {
        let mut i: i64 = 0_i64;
        while i < _MT_N {
            let y: i64 = (_state_word_at(&self._state_words, i) & _MT_UPPER_MASK)
                + (_state_word_at(&self._state_words, (i + (1_i64)) % _MT_N)
                    & _MT_LOWER_MASK);
            let mut x_a: i64 = y >> (1_i64);
            if (y % (2_i64)) != (0_i64) {
                x_a = x_a ^ _MT_MATRIX_A;
            }
            let new_word: i64 = _state_word_at(&self._state_words, (i + _MT_M) % _MT_N)
                ^ x_a;
            {
                let __idx_raw = i;
                let __idx_norm = if __idx_raw < 0 {
                    (self._state_words.len() as i64) + __idx_raw
                } else {
                    __idx_raw
                };
                if __idx_norm >= 0 {
                    if let Some(__elem) = self._state_words.get_mut(__idx_norm as usize)
                    {
                        *__elem = new_word & _MT_WORD_MASK;
                    }
                }
            }
            i += 1_i64;
        }
        self._index = 0_i64;
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn _next_u32(&mut self) -> i64 {
        if (self._index >= _MT_N) {
            self._twist();
        }
        let mut y: i64 = _state_word_at(&self._state_words, self._index);
        self._index += 1_i64;
        y = y ^ (y >> (11_i64));
        y = y ^ ((y << (7_i64)) & (2636928640_i64));
        y = y ^ ((y << (15_i64)) & (4022730752_i64));
        y = y ^ (y >> (18_i64));
        y & _MT_WORD_MASK
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn random(&mut self) -> f64 {
        (self._next_u32() as f64) / (4294967296.0_f64)
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
        start: i64,
        stop: Option<i64>,
        step: i64,
    ) -> Result<i64, ValueError> {
        if step == (0_i64) {
            return Err(ValueError::new("randrange: step must not be zero".to_string()));
        }
        let mut actual_start: i64 = start;
        let mut actual_stop: i64 = start;
        if stop.is_none() {
            actual_start = 0_i64;
        } else {
            if let Some(stop) = stop {
                actual_stop = stop;
            }
        }
        let width: i64 = actual_stop - actual_start;
        if step > (0_i64) {
            if width <= (0_i64) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
        } else {
            if width >= (0_i64) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
        }
        let mut abs_width: i64 = width;
        if abs_width < (0_i64) {
            abs_width = (0_i64) - abs_width;
        }
        let mut abs_step: i64 = step;
        if abs_step < (0_i64) {
            abs_step = (0_i64) - abs_step;
        }
        let count: i64 = ((abs_width + abs_step) - (1_i64)) / abs_step;
        if count <= (0_i64) {
            return Err(ValueError::new("randrange: empty range".to_string()));
        }
        let pick: i64 = self._next_u32() % count;
        Ok(actual_start + (pick * step))
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn randint(&mut self, minimum: i64, maximum: i64) -> Result<i64, ValueError> {
        if minimum > maximum {
            return Err(ValueError::new("randint: min must be <= max".to_string()));
        }
        self.randrange(minimum, Some(maximum + (1_i64)), 1_i64)
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn getrandbits(&mut self, k: i64) -> Result<i64, ValueError> {
        if k < (0_i64) {
            return Err(
                ValueError::new("getrandbits: number of bits must be >= 0".to_string()),
            );
        }
        let mut result: i64 = 0_i64;
        let mut bits_left: i64 = k;
        while bits_left > (0_i64) {
            let word: i64 = self._next_u32();
            let mut take: i64 = 32_i64;
            if bits_left < (32_i64) {
                take = bits_left;
            }
            let mask: i64 = ((1_i64) << take) - (1_i64);
            result = (result << take) | (word & mask);
            bits_left -= take;
        }
        Ok(result)
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn randbytes(&mut self, n: i64) -> Result<Vec<u8>, ValueError> {
        if n < (0_i64) {
            return Err(ValueError::new("randbytes: n must be >= 0".to_string()));
        }
        let mut values: Vec<i64> = vec![];
        let mut i: i64 = 0_i64;
        while i < n {
            let byte_value: i64 = self._next_u32() & (255_i64);
            values.push(byte_value);
            i += 1_i64;
        }
        {
            let __vals = values;
            let mut __out = Vec::new();
            for __pair in __vals.iter().enumerate() {
                if (*__pair.1 < 0) || (*__pair.1 > 255) {
                    return Err(ValueError {
                        message: format!(
                            "byte out of range at index {}: {}", __pair.0, * __pair.1
                        ),
                    });
                }
                __out.push(*__pair.1 as u8);
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
            3_i64,
            _clone_words(&self._state_words),
            self._index,
            self._gauss_next,
        )
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn setstate(
        &mut self,
        state: &__SifrStdlib_sifr_x2erandom_x2eRandomState,
    ) -> Result<(), ValueError> {
        if (state.version != (3_i64)) {
            return Err(ValueError::new("setstate: unsupported version".to_string()));
        }
        if ((state.state_words.len() as i64) != _MT_N) {
            return Err(
                ValueError::new("setstate: state_words must have length 624".to_string()),
            );
        }
        if (state.index < (0_i64)) || (state.index > _MT_N) {
            return Err(
                ValueError::new("setstate: index must be in range [0, 624]".to_string()),
            );
        }
        let mut normalized: Vec<i64> = vec![];
        for word in state.state_words.clone().iter().copied() {
            if (word < (0_i64)) || (word > _MT_WORD_MASK) {
                return Err(ValueError::new("setstate: word out of range".to_string()));
            }
            normalized.push(word & _MT_WORD_MASK);
        }
        self._state_words = normalized;
        self._index = state.index;
        self._gauss_next = state.gauss_next;
        Ok(())
    }
}
fn _state_word_at(words: &Vec<i64>, index: i64) -> i64 {
    let value: Option<i64> = {
        let __sifr_index_list = &words;
        let __sifr_index_i = index;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    if let Some(value) = value {
        return value;
    }
    0_i64
}
fn _clone_words(words: &Vec<i64>) -> Vec<i64> {
    let mut copied: Vec<i64> = vec![];
    for word in words.iter().copied() {
        copied.push(word);
    }
    copied
}
fn _normalize_seed_input(seed_value: Option<i64>) -> i64 {
    if let Some(seed_value) = seed_value {
        return seed_value;
    }
    (time_now() * (1000000.0_f64)) as i64
}
fn _seed_words_from_seed(seed_value: i64) -> Vec<i64> {
    let mut words: Vec<i64> = vec![];
    words.push(seed_value & _MT_WORD_MASK);
    let mut i: i64 = 1_i64;
    while i < _MT_N {
        let prev: i64 = _state_word_at(&words, i - (1_i64));
        let next_word: i64 = ((_MT_F * (prev ^ (prev >> (30_i64)))) + i) & _MT_WORD_MASK;
        words.push(next_word);
        i += 1_i64;
    }
    words
}
fn _build_state_from_module_storage() -> __SifrStdlib_sifr_x2erandom_x2eRandomState {
    __SifrStdlib_sifr_x2erandom_x2eRandomState::new(
        3_i64,
        random_module_state_words(),
        random_module_state_index(),
        random_module_state_gauss_next(),
    )
}
fn _store_state_into_module_storage(state: &__SifrStdlib_sifr_x2erandom_x2eRandomState) {
    let _set_result: Result<(), ValueError> = random_module_set_state(
        &_clone_words(&state.state_words.clone()),
        state.index,
        state.gauss_next,
    );
    let _ = _set_result;
}
fn _ensure_module_state_initialized() {
    let words: Vec<i64> = random_module_state_words();
    if (words.len() as i64) == _MT_N {
        return;
    }
    let bootstrap: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(5489_i64),
    );
    _store_state_into_module_storage(&bootstrap.getstate());
}
fn _module_random() -> __SifrStdlib_sifr_x2erandom_x2eRandom {
    _ensure_module_state_initialized();
    let mut r: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(0_i64),
    );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let _set_result: Result<(), ValueError> = r
            .setstate(&_build_state_from_module_storage());
        let _ = _set_result;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = e.message;
    }
    r
}
fn _sync_module_random(generator: &mut __SifrStdlib_sifr_x2erandom_x2eRandom) {
    _store_state_into_module_storage(&generator.getstate());
}
fn randrange(start: i64, stop: Option<i64>, step: i64) -> Result<i64, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<i64, ValueError> = generator.randrange(start, stop, step);
    _sync_module_random(&mut generator);
    value
}
fn gauss(mu: f64, sigma: f64) -> f64 {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: f64 = generator.gauss(mu, sigma);
    _sync_module_random(&mut generator);
    value
}
fn sample<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
    k: i64,
) -> Result<Vec<T>, ValueError> {
    if k < (0_i64) {
        return Err(ValueError::new("sample: k must be >= 0".to_string()));
    }
    if (k > (items.len() as i64)) {
        return Err(ValueError::new("sample larger than population".to_string()));
    }
    let mut pool: Vec<T> = vec![];
    for item in items.iter().cloned() {
        pool.push(item.clone().clone());
    }
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let mut result: Vec<T> = vec![];
    let mut remaining: i64 = pool.len() as i64;
    let mut i: i64 = 0_i64;
    while i < k {
        let pick_index: i64 = generator._next_u32() % remaining;
        let picked: Option<T> = {
            let __sifr_index_list = &pool;
            let __sifr_index_i = pick_index;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(picked) = picked {
            result.push(picked.clone().clone());
        }
        let last: Option<T> = {
            let __sifr_index_list = &pool;
            let __sifr_index_i = remaining - (1_i64);
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(last) = last {
            {
                let __idx_raw = pick_index;
                let __idx_norm = if __idx_raw < 0 {
                    (pool.len() as i64) + __idx_raw
                } else {
                    __idx_raw
                };
                if __idx_norm >= 0 {
                    if let Some(__elem) = pool.get_mut(__idx_norm as usize) {
                        *__elem = last.clone();
                    }
                }
            }
        }
        remaining -= 1_i64;
        i += 1_i64;
    }
    _sync_module_random(&mut generator);
    Ok(result)
}
fn shuffle<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(items: &mut Vec<T>) {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let n: i64 = items.len() as i64;
    if n > (1_i64) {
        let mut i: i64 = n - (1_i64);
        while i > (0_i64) {
            let j: i64 = generator._next_u32() % (i + (1_i64));
            let left: Option<T> = Some(items[i as usize].clone());
            let right: Option<T> = {
                let __sifr_index_list = &items;
                let __sifr_index_i = j;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(left) = left {
                if let Some(right) = right {
                    {
                        let __idx_raw = i;
                        let __idx_norm = if __idx_raw < 0 {
                            (items.len() as i64) + __idx_raw
                        } else {
                            __idx_raw
                        };
                        if __idx_norm >= 0 {
                            if let Some(__elem) = items.get_mut(__idx_norm as usize) {
                                *__elem = right.clone();
                            }
                        }
                    }
                    {
                        let __idx_raw = j;
                        let __idx_norm = if __idx_raw < 0 {
                            (items.len() as i64) + __idx_raw
                        } else {
                            __idx_raw
                        };
                        if __idx_norm >= 0 {
                            if let Some(__elem) = items.get_mut(__idx_norm as usize) {
                                *__elem = left.clone();
                            }
                        }
                    }
                }
            }
            i -= 1_i64;
        }
    }
    _sync_module_random(&mut generator);
}

// --- stdlib: sifr.statistics ---
#[derive(Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    message: String,
}
impl __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {}
impl ::std::fmt::Debug for __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("StatisticsError").field("message", &self.message).finish()
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl ::std::error::Error for __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {}
fn _sum(data: &Vec<f64>) -> f64 {
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        total += val;
    }
    total
}
fn mean(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let count: i64 = data.len() as i64;
    if count == (0_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "mean requires at least one data point".to_string(),
            ),
        );
    }
    let total: f64 = _sum(data);
    Ok(total / (count as f64))
}
fn median(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: i64 = data.len() as i64;
    if n == (0_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "median requires at least one data point".to_string(),
            ),
        );
    }
    let sorted_data: Vec<f64> = {
        let mut __sifr_sorted_v = (data).iter().copied().collect::<Vec<_>>();
        __sifr_sorted_v.sort_by(f64::total_cmp);
        __sifr_sorted_v
    };
    let mid: i64 = n / (2_i64);
    if (n % (2_i64)) == (0_i64) {
        let a: Option<f64> = {
            let __sifr_index_list = &sorted_data;
            let __sifr_index_i = mid - (1_i64);
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let b: Option<f64> = {
            let __sifr_index_list = &sorted_data;
            let __sifr_index_i = mid;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(a) = a {
            if let Some(b) = b {
                return Ok((a + b) / (2.0_f64));
            }
        }
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "median: index error".to_string(),
            ),
        );
    } else {
        let val: Option<f64> = {
            let __sifr_index_list = &sorted_data;
            let __sifr_index_i = mid;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(val) = val {
            return Ok(val);
        }
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "median: index error".to_string(),
            ),
        );
    }
}
fn variance(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: i64 = data.len() as i64;
    if n < (2_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "variance requires at least two data points".to_string(),
            ),
        );
    }
    let avg: f64 = _sum(data) / (n as f64);
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        let diff: f64 = val - avg;
        total += diff * diff;
    }
    Ok(total / ((n - (1_i64)) as f64))
}
fn stdev(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: i64 = data.len() as i64;
    if n < (2_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "stdev requires at least two data points".to_string(),
            ),
        );
    }
    let avg: f64 = _sum(data) / (n as f64);
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        let diff: f64 = val - avg;
        total += diff * diff;
    }
    let v: f64 = total / ((n - (1_i64)) as f64);
    Ok(sqrt(v))
}
fn mode(
    data: &Vec<i64>,
) -> Result<i64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    if ((data.len() as i64) == (0_i64)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "mode requires at least one data point".to_string(),
            ),
        );
    }
    let mut counts: HashMap<i64, i64> = HashMap::from([]);
    for val in data.iter().copied() {
        let existing: Option<i64> = counts.get(&val).copied();
        if let Some(existing) = existing {
            counts.insert(val, existing + (1_i64));
        } else {
            counts.insert(val, 1_i64);
        }
    }
    let mut best: i64 = 0_i64;
    let mut best_set: bool = false;
    let mut best_count: i64 = 0_i64;
    for val2 in data.iter().copied() {
        let count2: Option<i64> = counts.get(&val2).copied();
        let mut count2_val: i64 = 0_i64;
        if let Some(count2) = count2 {
            count2_val = count2;
        }
        if count2_val > best_count {
            best_count = count2_val;
            best = val2;
            best_set = true;
        }
    }
    if best_set {
        return Ok(best);
    }
    Err(
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
            "mode: no mode found".to_string(),
        ),
    )
}
fn multimode(
    data: &Vec<i64>,
) -> Result<Vec<i64>, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    if ((data.len() as i64) == (0_i64)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "multimode requires at least one data point".to_string(),
            ),
        );
    }
    let mut counts: HashMap<i64, i64> = HashMap::from([]);
    for val in data.iter().copied() {
        let existing: Option<i64> = counts.get(&val).copied();
        if let Some(existing) = existing {
            counts.insert(val, existing + (1_i64));
        } else {
            counts.insert(val, 1_i64);
        }
    }
    let mut max_count: i64 = 0_i64;
    for val2 in data.iter().copied() {
        let count2: Option<i64> = counts.get(&val2).copied();
        let mut count2_val: i64 = 0_i64;
        if let Some(count2) = count2 {
            count2_val = count2;
        }
        if count2_val > max_count {
            max_count = count2_val;
        }
    }
    let mut result: Vec<i64> = vec![];
    let mut seen: HashMap<i64, bool> = HashMap::from([]);
    for val3 in data.iter().copied() {
        let already_opt: Option<bool> = seen.get(&val3).copied();
        let mut already: bool = false;
        if let Some(already_opt) = already_opt {
            already = already_opt;
        }
        if !already {
            let count3: Option<i64> = counts.get(&val3).copied();
            let mut count3_val: i64 = 0_i64;
            if let Some(count3) = count3 {
                count3_val = count3;
            }
            if count3_val == max_count {
                result.push(val3);
            }
            seen.insert(val3, true);
        }
    }
    Ok(result)
}
fn quantiles(
    data: &Vec<f64>,
    n: i64,
) -> Result<Vec<f64>, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    if ((data.len() as i64) < (2_i64)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "quantiles requires at least two data points".to_string(),
            ),
        );
    }
    if n < (1_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "quantiles: n must be at least 1".to_string(),
            ),
        );
    }
    let sorted_data: Vec<f64> = {
        let mut __sifr_sorted_v = (data).iter().copied().collect::<Vec<_>>();
        __sifr_sorted_v.sort_by(f64::total_cmp);
        __sifr_sorted_v
    };
    let m: i64 = sorted_data.len() as i64;
    let mut result: Vec<f64> = vec![];
    let mut i: i64 = 1_i64;
    while i < n {
        let idx_f: f64 = ((i as f64) * (m as f64)) / (n as f64);
        let mut idx: i64 = idx_f as i64;
        let frac: f64 = idx_f - (idx as f64);
        if idx >= m {
            idx = m - (1_i64);
        }
        if idx < (0_i64) {
            idx = 0_i64;
        }
        let lo: Option<f64> = {
            let __sifr_index_list = &sorted_data;
            let __sifr_index_i = idx;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let mut lo_val: f64 = 0.0_f64;
        if let Some(lo) = lo {
            lo_val = lo;
        }
        if frac > (0.0_f64) {
            let hi_idx: i64 = idx + (1_i64);
            if hi_idx < m {
                let hi: Option<f64> = Some(sorted_data[hi_idx as usize]);
                if let Some(hi) = hi {
                    lo_val += frac * (hi - lo_val);
                }
            }
        }
        result.push(lo_val);
        i += 1_i64;
    }
    Ok(result)
}
fn covariance(
    x: &Vec<f64>,
    y: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: i64 = x.len() as i64;
    if n < (2_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "covariance requires at least two data points".to_string(),
            ),
        );
    }
    if ((y.len() as i64) != n) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "covariance: x and y must have the same length".to_string(),
            ),
        );
    }
    let mx: f64 = _sum(x) / (n as f64);
    let my: f64 = _sum(y) / (n as f64);
    let mut total: f64 = 0.0_f64;
    let mut i: i64 = 0_i64;
    while i < n {
        let xi: Option<f64> = Some(x[i as usize]);
        let yi: Option<f64> = {
            let __sifr_index_list = &y;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(xi) = xi {
            if let Some(yi) = yi {
                total += (xi - mx) * (yi - my);
            }
        }
        i += 1_i64;
    }
    Ok(total / ((n - (1_i64)) as f64))
}
fn correlation(
    x: &Vec<f64>,
    y: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: i64 = x.len() as i64;
    if n < (2_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "correlation requires at least two data points".to_string(),
            ),
        );
    }
    if ((y.len() as i64) != n) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "correlation: x and y must have the same length".to_string(),
            ),
        );
    }
    let mx: f64 = _sum(x) / (n as f64);
    let my: f64 = _sum(y) / (n as f64);
    let mut cov_num: f64 = 0.0_f64;
    let mut sx_num: f64 = 0.0_f64;
    let mut sy_num: f64 = 0.0_f64;
    let mut i: i64 = 0_i64;
    while i < n {
        let xi: Option<f64> = Some(x[i as usize]);
        let yi: Option<f64> = {
            let __sifr_index_list = &y;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(xi) = xi {
            if let Some(yi) = yi {
                cov_num += (xi - mx) * (yi - my);
                sx_num += (xi - mx) * (xi - mx);
                sy_num += (yi - my) * (yi - my);
            }
        }
        i += 1_i64;
    }
    let sx: f64 = sqrt(sx_num / ((n - (1_i64)) as f64));
    let sy: f64 = sqrt(sy_num / ((n - (1_i64)) as f64));
    if sx == (0.0_f64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "correlation: x has zero variance".to_string(),
            ),
        );
    }
    if sy == (0.0_f64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "correlation: y has zero variance".to_string(),
            ),
        );
    }
    Ok((cov_num / ((n - (1_i64)) as f64)) / (sx * sy))
}
fn linear_regression(
    x: &Vec<f64>,
    y: &Vec<f64>,
) -> Result<Vec<f64>, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: i64 = x.len() as i64;
    if n < (2_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "linear_regression requires at least two data points".to_string(),
            ),
        );
    }
    if ((y.len() as i64) != n) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "linear_regression: x and y must have the same length".to_string(),
            ),
        );
    }
    let mx: f64 = _sum(x) / (n as f64);
    let my: f64 = _sum(y) / (n as f64);
    let mut num: f64 = 0.0_f64;
    let mut den: f64 = 0.0_f64;
    let mut i: i64 = 0_i64;
    while i < n {
        let xi: Option<f64> = Some(x[i as usize]);
        let yi: Option<f64> = {
            let __sifr_index_list = &y;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(xi) = xi {
            if let Some(yi) = yi {
                num += (xi - mx) * (yi - my);
                den += (xi - mx) * (xi - mx);
            }
        }
        i += 1_i64;
    }
    if den == (0.0_f64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "linear_regression: x has zero variance".to_string(),
            ),
        );
    }
    let slope: f64 = num / den;
    let intercept: f64 = my - (slope * mx);
    let mut result: Vec<f64> = vec![];
    result.push(slope);
    result.push(intercept);
    Ok(result)
}
// --- end stdlib ---

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for Error {
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

impl ::std::error::Error for ParseError {
}

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

impl ::std::error::Error for ValueError {
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Self::new(err.message)
    }
}

impl From<ValueError> for Error {
    fn from(err: ValueError) -> Self {
        Self::new(err.message)
    }
}

fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn mul(a: i64, b: i64) -> i64 {
    a * b
}

fn less_than_three(x: i64) -> bool {
    x < (3_i64)
}

fn main() {
    println!("=== math additions ===");
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(13usize + 0usize);
    __sifr_concat.push_str("acosh(1.0) = ");
    __sifr_concat.push_str((format!("{}", acosh(1.0_f64))).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(13usize + 0usize);
    __sifr_concat.push_str("asinh(0.0) = ");
    __sifr_concat.push_str((format!("{}", asinh(0.0_f64))).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(13usize + 0usize);
    __sifr_concat.push_str("atanh(0.0) = ");
    __sifr_concat.push_str((format!("{}", atanh(0.0_f64))).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(12usize + 0usize);
    __sifr_concat.push_str("isqrt(17) = ");
    __sifr_concat.push_str((format!("{}", isqrt(17_i64))).as_str());
    __sifr_concat
});
    let p: Vec<f64> = vec![0.0_f64, 0.0_f64];
    let q: Vec<f64> = vec![3.0_f64, 4.0_f64];
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(20usize + 0usize);
    __sifr_concat.push_str("dist([0,0],[3,4]) = ");
    __sifr_concat.push_str((format!("{}", dist(&p, &q))).as_str());
    __sifr_concat
});
    let data_fsum: Vec<f64> = vec![0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64];
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(15usize + 0usize);
    __sifr_concat.push_str("fsum(10x0.1) = ");
    __sifr_concat.push_str((format!("{}", fsum(&data_fsum))).as_str());
    __sifr_concat
});
    println!("=== statistics (Result[float, StatisticsError]) ===");
    let data: Vec<f64> = vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64, 5.0_f64];
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let m: f64 = mean(&data)?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(7usize + 0usize);
    __sifr_concat.push_str("mean = ");
    __sifr_concat.push_str((format!("{}", m)).as_str());
    __sifr_concat
});
    let med: f64 = median(&data)?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(9usize + 0usize);
    __sifr_concat.push_str("median = ");
    __sifr_concat.push_str((format!("{}", med)).as_str());
    __sifr_concat
});
    let v: f64 = variance(&data)?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(11usize + 0usize);
    __sifr_concat.push_str("variance = ");
    __sifr_concat.push_str((format!("{}", v)).as_str());
    __sifr_concat
});
    let s: f64 = stdev(&data)?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(8usize + 0usize);
    __sifr_concat.push_str("stdev = ");
    __sifr_concat.push_str((format!("{}", s)).as_str());
    __sifr_concat
});
    let idata: Vec<i64> = vec![1_i64, 2_i64, 2_i64, 3_i64, 3_i64, 3_i64];
    let mo: i64 = mode(&idata)?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(7usize + 0usize);
    __sifr_concat.push_str("mode = ");
    __sifr_concat.push_str((format!("{}", mo)).as_str());
    __sifr_concat
});
    let mm: Vec<i64> = multimode(&vec![1_i64, 2_i64, 2_i64, 3_i64, 3_i64])?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
    __sifr_concat.push_str("multimode len = ");
    __sifr_concat.push_str((format!("{}", mm.len() as i64)).as_str());
    __sifr_concat
});
    let qs: Vec<f64> = quantiles(&data, 4_i64)?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(18usize + 0usize);
    __sifr_concat.push_str("quartiles count = ");
    __sifr_concat.push_str((format!("{}", qs.len() as i64)).as_str());
    __sifr_concat
});
    let x: Vec<f64> = vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64, 5.0_f64];
    let y: Vec<f64> = vec![2.0_f64, 4.0_f64, 6.0_f64, 8.0_f64, 10.0_f64];
    let cov: f64 = covariance(&x, &y)?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(13usize + 0usize);
    __sifr_concat.push_str("covariance = ");
    __sifr_concat.push_str((format!("{}", cov)).as_str());
    __sifr_concat
});
    let r: f64 = correlation(&x, &y)?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(14usize + 0usize);
    __sifr_concat.push_str("correlation = ");
    __sifr_concat.push_str((format!("{}", r)).as_str());
    __sifr_concat
});
    let lr: Vec<f64> = linear_regression(&x, &y)?;
    let slope: Option<f64> = {
    let __sifr_index_list = &lr;
    let __sifr_index_i = 0_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
    let intercept: Option<f64> = {
    let __sifr_index_list = &lr;
    let __sifr_index_i = 1_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
    if let Some(slope) = slope {
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(8usize + 0usize);
    __sifr_concat.push_str("slope = ");
    __sifr_concat.push_str((format!("{}", slope)).as_str());
    __sifr_concat
});
    }
    if let Some(intercept) = intercept {
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(12usize + 0usize);
    __sifr_concat.push_str("intercept = ");
    __sifr_concat.push_str((format!("{}", intercept)).as_str());
    __sifr_concat
});
    }
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(7usize + 0usize);
    __sifr_concat.push_str("error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (|| {
    let empty: Vec<f64> = vec![];
    let bad: f64 = mean(&empty)?;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(18usize + 0usize);
    __sifr_concat.push_str("empty mean error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
    println!("=== random additions ===");
    let mut items: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    shuffle(&mut items);
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(14usize + 0usize);
    __sifr_concat.push_str("shuffle len = ");
    __sifr_concat.push_str((format!("{}", items.len() as i64)).as_str());
    __sifr_concat
});
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let s3: Vec<i64> = sample(&items, 3_i64)?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
    __sifr_concat.push_str("sample(3) len = ");
    __sifr_concat.push_str((format!("{}", s3.len() as i64)).as_str());
    __sifr_concat
});
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(14usize + 0usize);
    __sifr_concat.push_str("sample error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let rr: i64 = randrange(0_i64, Some(100_i64), 5_i64)?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(21usize + 0usize);
    __sifr_concat.push_str("randrange in range = ");
    __sifr_concat.push_str((format!("{}", rr >= (0_i64))).as_str());
    __sifr_concat
});
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(17usize + 0usize);
    __sifr_concat.push_str("randrange error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
    let g: f64 = gauss(0.0_f64, 1.0_f64);
    println!("gauss sample is float = True");
    println!("=== functools.reduce ===");
    let nums: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    let total: i64 = reduce(|__arg0, __arg1| add((__arg0).clone(), (__arg1).clone()), &nums, &(0_i64));
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(14usize + 0usize);
    __sifr_concat.push_str("reduce(add) = ");
    __sifr_concat.push_str((format!("{}", total)).as_str());
    __sifr_concat
});
    let product: i64 = reduce(|__arg0, __arg1| mul((__arg0).clone(), (__arg1).clone()), &nums, &(1_i64));
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(14usize + 0usize);
    __sifr_concat.push_str("reduce(mul) = ");
    __sifr_concat.push_str((format!("{}", product)).as_str());
    __sifr_concat
});
    println!("=== itertools additions ===");
    let idata2: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    let acc: Vec<i64> = accumulate(&(idata2).iter().copied().collect::<Vec<_>>(), None).collect::<Vec<_>>();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(13usize + 0usize);
    __sifr_concat.push_str("accumulate = ");
    __sifr_concat.push_str((format!("{:?}", acc)).as_str());
    __sifr_concat
});
    let sel: Vec<bool> = vec![true, false, true, false, true];
    let comp: Vec<i64> = compress(&(idata2).iter().copied().collect::<Vec<_>>(), &(sel).iter().copied().collect::<Vec<_>>()).collect::<Vec<_>>();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(11usize + 0usize);
    __sifr_concat.push_str("compress = ");
    __sifr_concat.push_str((format!("{:?}", comp)).as_str());
    __sifr_concat
});
    let dw: Vec<i64> = dropwhile(|__arg0| less_than_three((__arg0).clone()), &(idata2).iter().copied().collect::<Vec<_>>()).collect::<Vec<_>>();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
    __sifr_concat.push_str("dropwhile(<3) = ");
    __sifr_concat.push_str((format!("{:?}", dw)).as_str());
    __sifr_concat
});
    let tw: Vec<i64> = takewhile(|__arg0| less_than_three((__arg0).clone()), &(idata2).iter().copied().collect::<Vec<_>>()).collect::<Vec<_>>();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
    __sifr_concat.push_str("takewhile(<3) = ");
    __sifr_concat.push_str((format!("{:?}", tw)).as_str());
    __sifr_concat
});
    let ff: Vec<i64> = filterfalse(|__arg0| less_than_three((__arg0).clone()), &(idata2).iter().copied().collect::<Vec<_>>()).collect::<Vec<_>>();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(18usize + 0usize);
    __sifr_concat.push_str("filterfalse(<3) = ");
    __sifr_concat.push_str((format!("{:?}", ff)).as_str());
    __sifr_concat
});
    let a: Vec<i64> = vec![1_i64, 2_i64, 3_i64];
    let b: Vec<i64> = vec![4_i64, 5_i64];
    let zl: Vec<Vec<i64>> = zip_longest(&(a).iter().copied().collect::<Vec<_>>(), &(b).iter().copied().collect::<Vec<_>>(), &(0_i64)).collect::<Vec<_>>();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(18usize + 0usize);
    __sifr_concat.push_str("zip_longest len = ");
    __sifr_concat.push_str((format!("{}", zl.len() as i64)).as_str());
    __sifr_concat
});
    let cf: Vec<i64> = count_from(0_i64, 2_i64, 5_i64).collect::<Vec<_>>();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(20usize + 0usize);
    __sifr_concat.push_str("count_from(0,2,5) = ");
    __sifr_concat.push_str((format!("{:?}", cf)).as_str());
    __sifr_concat
});
    let mut ctr: Box<dyn Iterator<Item = i64>> = count(0_i64, 2_i64);
    let count0: Option<i64> = ctr.next();
    let count1: Option<i64> = ctr.next();
    let count2: Option<i64> = ctr.next();
    let count3: Option<i64> = ctr.next();
    let count4: Option<i64> = ctr.next();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(21usize + 0usize);
    __sifr_concat.push_str("count(0,2) first 5 = ");
    __sifr_concat.push_str((format!("{:?}", vec![count0, count1, count2, count3, count4])).as_str());
    __sifr_concat
});
    let cyc: Vec<i64> = cycle(&(vec![1_i64, 2_i64, 3_i64]).into_iter().collect::<Vec<_>>(), 7_i64).collect::<Vec<_>>();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(20usize + 0usize);
    __sifr_concat.push_str("cycle([1,2,3], 7) = ");
    __sifr_concat.push_str((format!("{:?}", cyc)).as_str());
    __sifr_concat
});
    println!("=== Counter enhancements ===");
    let mut c1: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(&vec!["a".to_string(), "b".to_string(), "a".to_string(), "c".to_string()]);
    let c2: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(&vec!["b".to_string(), "c".to_string(), "d".to_string()]);
    c1.update(&c2);
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(((16usize + 0usize) + 3usize) + 0usize);
    __sifr_concat.push_str("after update: a=");
    __sifr_concat.push_str((format!("{}", c1.get(&"a".to_string(), 0_i64))).as_str());
    __sifr_concat.push_str(" b=");
    __sifr_concat.push_str((format!("{}", c1.get(&"b".to_string(), 0_i64))).as_str());
    __sifr_concat
});
    let mut c3: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(&vec!["x".to_string(), "x".to_string(), "y".to_string()]);
    let c4: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(&vec!["x".to_string()]);
    c3.subtract(&c4);
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(18usize + 0usize);
    __sifr_concat.push_str("after subtract: x=");
    __sifr_concat.push_str((format!("{}", c3.get(&"x".to_string(), 0_i64))).as_str());
    __sifr_concat
});
    let c5: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(&vec!["a".to_string(), "a".to_string(), "b".to_string()]);
    let elems: Vec<String> = c5.elements();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(15usize + 0usize);
    __sifr_concat.push_str("elements len = ");
    __sifr_concat.push_str((format!("{}", elems.len() as i64)).as_str());
    __sifr_concat
});
    let mut cc: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(&vec!["a".to_string(), "b".to_string()]);
    cc.update(&from_list(&vec!["b".to_string(), "c".to_string()]));
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
    __sifr_concat.push_str("counter_add b = ");
    __sifr_concat.push_str((format!("{}", cc.get(&"b".to_string(), 0_i64))).as_str());
    __sifr_concat
});
    let mut cd: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(&vec!["a".to_string(), "a".to_string(), "b".to_string()]);
    cd.subtract(&from_list(&vec!["a".to_string()]));
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
    __sifr_concat.push_str("counter_sub a = ");
    __sifr_concat.push_str((format!("{}", cd.get(&"a".to_string(), 0_i64))).as_str());
    __sifr_concat
});
    println!("=== stdlib_pure_expansion: all features demonstrated ===");
}
