// src/main.rs
mod __sifr_project_nominals {
    pub use ::std::collections::HashMap;
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
}
pub use __sifr_project_nominals::ValueError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ecollections_x2eCounter;
use ::std::collections::HashMap;
use ::sifr_runtime::SifrInt;
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
fn _compress_impl<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    selectors: &Vec<bool>,
) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(data.len())) {
        if (&i >= &SifrInt::from(selectors.len())) {
            i = SifrInt::from(data.len());
        } else {
            let sel: Option<bool> = {
                let __sifr_index_list = &selectors;
                let __sifr_index_i = i.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
                __sifr_index_list.get(__sifr_index_norm).copied()
            };
            let val: Option<T> = Some(
                data[::sifr_runtime::to_usize_proven(&(i))].clone(),
            );
            if let Some(sel) = sel {
                if let Some(val) = val {
                    if sel {
                        result.push(val.clone());
                    }
                }
            }
            i = &i + &SifrInt::from_i64(1);
        }
    }
    result
}
fn _takewhile_impl<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool,
    data: &Vec<T>,
) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(data.len())) {
        let val: Option<T> = Some(data[::sifr_runtime::to_usize_proven(&(i))].clone());
        if let Some(val) = val {
            if pred(&val) {
                result.push(val.clone());
            } else {
                i = SifrInt::from(data.len());
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    result
}
fn _zip_longest_impl<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    a: &Vec<T>,
    b: &Vec<T>,
    fill: &T,
) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = vec![];
    let len_a: SifrInt = SifrInt::from(a.len());
    let len_b: SifrInt = SifrInt::from(b.len());
    let mut max_len: SifrInt = len_a.clone();
    if &len_b > &max_len {
        max_len = len_b.clone();
    }
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &max_len {
        let mut pair: Vec<T> = vec![];
        if &i < &len_a {
            let va: Option<T> = Some(a[::sifr_runtime::to_usize_proven(&(i))].clone());
            if let Some(va) = va {
                pair.push(va.clone());
            } else {
                pair.push(fill.clone());
            }
        } else {
            pair.push(fill.clone());
        }
        if &i < &len_b {
            let vb: Option<T> = Some(b[::sifr_runtime::to_usize_proven(&(i))].clone());
            if let Some(vb) = vb {
                pair.push(vb.clone());
            } else {
                pair.push(fill.clone());
            }
        } else {
            pair.push(fill.clone());
        }
        result.push(pair.clone());
        i = &i + &SifrInt::from_i64(1);
    }
    result
}
fn _collect_iterable<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: Vec<T>,
) -> Vec<T> {
    let mut collected: Vec<T> = vec![];
    for item in data.iter().cloned() {
        collected.push(item.clone());
    }
    collected
}
fn count(start: SifrInt, step: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    count_from((start).clone(), (step).clone(), SifrInt::from_i64(10000))
}
fn accumulate<
    T: Clone + ::std::fmt::Display + PartialOrd + 'static + ::std::ops::Add<Output = T>,
>(data: &Vec<T>, initial: Option<T>) -> Box<dyn Iterator<Item = T>> {
    let mut result: Vec<T> = vec![];
    if let Some(initial) = initial {
        result.push(initial.clone());
    }
    for item in data.iter().cloned() {
        if (&SifrInt::from(result.len()) == &SifrInt::from_i64(0)) {
            result.push(item.clone());
        } else {
            let prev: Option<T> = {
                let __sifr_index_list = &result;
                let __sifr_index_i = SifrInt::from(result.len()) - SifrInt::from_i64(1);
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(prev) = prev {
                let next_val: T = prev + item;
                result.push(next_val.clone());
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
                result.push(val.clone());
            }
        } else {
            result.push(val.clone());
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
            result.push(val.clone());
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
fn count_from(
    start: SifrInt,
    step: SifrInt,
    n: SifrInt,
) -> Box<dyn Iterator<Item = SifrInt>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<SifrInt> = Vec::new()
        .into_iter();
    Box::new(
        ::std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<SifrInt> = Vec::new();
                let mut i: SifrInt = SifrInt::from_i64(0);
                let mut current: SifrInt = start.clone();
                while &i < &n {
                    _yields.push(current.clone());
                    current = &current + &step;
                    i = &i + &SifrInt::from_i64(1);
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
    n: SifrInt,
) -> Box<dyn Iterator<Item = T>> {
    let materialized: Vec<T> = _collect_iterable(
        ((data).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let mut result: Vec<T> = vec![];
    let size: SifrInt = SifrInt::from(materialized.len());
    if &size > &SifrInt::from_i64(0) {
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &n {
            let idx: SifrInt = i.floor_mod_known_nonzero(&size);
            let val: Option<T> = {
                let __sifr_index_list = &materialized;
                let __sifr_index_i = idx.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(val) = val {
                result.push(val.clone());
            }
            i = &i + &SifrInt::from_i64(1);
        }
    }
    Box::new(result.into_iter())
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
    while &y != &SifrInt::from_i64(0) {
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
    while &i < &r {
        result = &result * &(&n - &i);
        let divisor: SifrInt = &i + &SifrInt::from_i64(1);
        if &divisor == &SifrInt::from_i64(0) {
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
        let __sifr_index_list = &parts;
        let __sifr_index_i = SifrInt::from_i64(0);
        let __sifr_index_norm = __sifr_index_i
            .normalize_index_or_len(__sifr_index_list.len());
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(m) = m else {
        return NAN;
    };
    m
}
fn frexp_exponent(x: f64) -> SifrInt {
    let parts: Vec<f64> = frexp(x);
    let exp_val: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = SifrInt::from_i64(1);
        let __sifr_index_norm = __sifr_index_i
            .normalize_index_or_len(__sifr_index_list.len());
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(exp_val) = exp_val else {
        return SifrInt::from_i64(0);
    };
    trunc(exp_val)
}
fn modf_fractional(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let f: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = SifrInt::from_i64(0);
        let __sifr_index_norm = __sifr_index_i
            .normalize_index_or_len(__sifr_index_list.len());
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
        let __sifr_index_i = SifrInt::from_i64(1);
        let __sifr_index_norm = __sifr_index_i
            .normalize_index_or_len(__sifr_index_list.len());
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
        while &i < &__const__MT_N() {
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
                let __idx_raw = i.clone();
                let __idx_norm = __idx_raw
                    .normalize_index_or_len(self._state_words.len());
                if let Some(__elem) = self._state_words.get_mut(__idx_norm) {
                    *__elem = &new_word & &__const__MT_WORD_MASK();
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
            if &width <= &SifrInt::from_i64(0) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
        } else {
            if &width >= &SifrInt::from_i64(0) {
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
        if &abs_step == &SifrInt::from_i64(0) {
            return Err(ValueError::new("randrange: step must not be zero".to_string()));
        }
        let count: SifrInt = (&(&abs_width + &abs_step) - &SifrInt::from_i64(1))
            .floor_div_known_nonzero(&abs_step);
        if &count <= &SifrInt::from_i64(0) {
            return Err(ValueError::new("randrange: empty range".to_string()));
        }
        if &count == &SifrInt::from_i64(0) {
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
        while &bits_left > &SifrInt::from_i64(0) {
            let word: SifrInt = self._next_u32();
            let mut take: SifrInt = SifrInt::from_i64(32);
            if &bits_left < &SifrInt::from_i64(32) {
                take = bits_left.clone();
            }
            let mut mask: SifrInt = SifrInt::from_i64(0);
            let mut shifted_result: SifrInt = result;
            let mut shift_index: SifrInt = SifrInt::from_i64(0);
            while &shift_index < &take {
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
                            .map_err(|_error| Err(ValueError {
                                message: format!(
                                    "byte out of range at index {}: {}", __pair.0, * __pair.1
                                ),
                            }))?,
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
        let __sifr_index_list = &words;
        let __sifr_index_i = index.clone();
        let __sifr_index_norm = __sifr_index_i
            .normalize_index_or_len(__sifr_index_list.len());
        __sifr_index_list.get(__sifr_index_norm).cloned()
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
    while &i < &__const__MT_N() {
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
fn randrange(
    start: SifrInt,
    stop: Option<SifrInt>,
    step: SifrInt,
) -> Result<SifrInt, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<SifrInt, ValueError> = generator.randrange(&start, &stop, &step);
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
    k: SifrInt,
) -> Result<Vec<T>, ValueError> {
    if &k < &SifrInt::from_i64(0) {
        return Err(ValueError::new("sample: k must be >= 0".to_string()));
    }
    if (&k > &SifrInt::from(items.len())) {
        return Err(ValueError::new("sample larger than population".to_string()));
    }
    let mut pool: Vec<T> = vec![];
    for item in items.iter().cloned() {
        pool.push(item.clone());
    }
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let mut result: Vec<T> = vec![];
    let mut remaining: SifrInt = SifrInt::from(pool.len());
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &k {
        if &remaining == &SifrInt::from_i64(0) {
            return Err(ValueError::new("sample larger than population".to_string()));
        }
        let pick_index: SifrInt = generator
            ._next_u32()
            .floor_mod_known_nonzero(&remaining);
        let picked: Option<T> = {
            let __sifr_index_list = &pool;
            let __sifr_index_i = pick_index.clone();
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(picked) = picked {
            result.push(picked.clone());
        }
        let last: Option<T> = {
            let __sifr_index_list = &pool;
            let __sifr_index_i = &remaining - &SifrInt::from_i64(1);
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(last) = last {
            {
                let __idx_raw = pick_index.clone();
                let __idx_norm = __idx_raw.normalize_index_or_len(pool.len());
                if let Some(__elem) = pool.get_mut(__idx_norm) {
                    *__elem = last.clone();
                }
            }
        }
        remaining = &remaining - &SifrInt::from_i64(1);
        i = &i + &SifrInt::from_i64(1);
    }
    _sync_module_random(&mut generator);
    Ok(result)
}
fn shuffle<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(items: &mut Vec<T>) {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let n: SifrInt = SifrInt::from(items.len());
    if &n > &SifrInt::from_i64(1) {
        let mut i: SifrInt = &n - &SifrInt::from_i64(1);
        while &i > &SifrInt::from_i64(0) {
            let divisor: SifrInt = &i + &SifrInt::from_i64(1);
            if &divisor == &SifrInt::from_i64(0) {
                return;
            }
            let j: SifrInt = generator._next_u32().floor_mod_known_nonzero(&divisor);
            let left: Option<T> = {
                let __sifr_index_list = &items;
                let __sifr_index_i = i.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let right: Option<T> = {
                let __sifr_index_list = &items;
                let __sifr_index_i = j.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(left) = left {
                if let Some(right) = right {
                    {
                        let __idx_raw = i.clone();
                        let __idx_norm = __idx_raw.normalize_index_or_len(items.len());
                        if let Some(__elem) = items.get_mut(__idx_norm) {
                            *__elem = right.clone();
                        }
                    }
                    {
                        let __idx_raw = j.clone();
                        let __idx_norm = __idx_raw.normalize_index_or_len(items.len());
                        if let Some(__elem) = items.get_mut(__idx_norm) {
                            *__elem = left.clone();
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
enum __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    __SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(FloatOverflowError),
    __SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
        FloatPrecisionLossError,
    ),
}
impl From<FloatOverflowError>
for __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    fn from(value: FloatOverflowError) -> Self {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
            value,
        )
    }
}
impl ::std::fmt::Display
for __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
        }
    }
}
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
fn _float_int(
    value: SifrInt,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let __sifr_try_res: Result<
        Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError>,
        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
    > = (|| {
        let converted: f64 = value
            .clone()
            .checked_to_f64()
            .map_err(|__sifr_float_error| match __sifr_float_error {
                ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        FloatOverflowError::new(
                            "exact integer is outside the finite float range".to_string(),
                        ),
                    )
                }
                ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        FloatPrecisionLossError::new(
                            "exact integer cannot be represented without float precision loss"
                                .to_string(),
                        ),
                    )
                }
            })?;
        Ok(Ok(converted))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            match __sifr_try_err {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let error = __sifr_try_variant_error.clone();
                    return Err(
                        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                            error.message.clone(),
                        ),
                    );
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let error = __sifr_try_variant_error.clone();
                    return Err(
                        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                            error.message.clone(),
                        ),
                    );
                }
            }
        }
    }
}
fn _divide_by_int(
    numerator: f64,
    denominator: SifrInt,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let __sifr_try_res: Result<
        Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError>,
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let divisor: f64 = _float_int((denominator).clone())?;
        Ok(Ok(numerator / divisor))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    }
}
fn mean(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let count: SifrInt = SifrInt::from(data.len());
    if &count == &SifrInt::from_i64(0) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "mean requires at least one data point".to_string(),
            ),
        );
    }
    let total: f64 = _sum(data);
    _divide_by_int(total, (count).clone())
}
fn median(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: SifrInt = SifrInt::from(data.len());
    if &n == &SifrInt::from_i64(0) {
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
    let mid: SifrInt = n.floor_div_known_nonzero(&SifrInt::from_i64(2));
    if (&n.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0)) {
        let a: Option<f64> = {
            let __sifr_index_list = &sorted_data;
            let __sifr_index_i = &mid - &SifrInt::from_i64(1);
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let b: Option<f64> = {
            let __sifr_index_list = &sorted_data;
            let __sifr_index_i = mid.clone();
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
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
            let __sifr_index_i = mid.clone();
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
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
    let n: SifrInt = SifrInt::from(data.len());
    if &n < &SifrInt::from_i64(2) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "variance requires at least two data points".to_string(),
            ),
        );
    }
    let __sifr_try_res: Result<
        (f64,),
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let avg: f64 = _divide_by_int(_sum(data), (n).clone())?;
        Ok((avg,))
    })();
    let (avg,) = match __sifr_try_res {
        Ok(__sifr_try_bindings) => __sifr_try_bindings,
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    };
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        let diff: f64 = val - avg;
        total += diff * diff;
    }
    _divide_by_int(total, &n - &SifrInt::from_i64(1))
}
fn stdev(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: SifrInt = SifrInt::from(data.len());
    if &n < &SifrInt::from_i64(2) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "stdev requires at least two data points".to_string(),
            ),
        );
    }
    let __sifr_try_res: Result<
        (f64,),
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let avg: f64 = _divide_by_int(_sum(data), (n).clone())?;
        Ok((avg,))
    })();
    let (avg,) = match __sifr_try_res {
        Ok(__sifr_try_bindings) => __sifr_try_bindings,
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    };
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        let diff: f64 = val - avg;
        total += diff * diff;
    }
    let __sifr_try_res: Result<
        (f64,),
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let v: f64 = _divide_by_int(total, &n - &SifrInt::from_i64(1))?;
        Ok((v,))
    })();
    let (v,) = match __sifr_try_res {
        Ok(__sifr_try_bindings) => __sifr_try_bindings,
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    };
    Ok(sqrt(v))
}
fn mode(
    data: &Vec<SifrInt>,
) -> Result<SifrInt, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    if (&SifrInt::from(data.len()) == &SifrInt::from_i64(0)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "mode requires at least one data point".to_string(),
            ),
        );
    }
    let mut counts: HashMap<SifrInt, SifrInt> = HashMap::from([]);
    for val in data.iter().cloned() {
        let existing: Option<SifrInt> = counts.get(&val).cloned();
        if let Some(existing) = existing.clone() {
            counts.insert(val.clone(), &existing + &SifrInt::from_i64(1));
        } else {
            counts.insert(val.clone(), SifrInt::from_i64(1));
        }
    }
    let mut best: SifrInt = SifrInt::from_i64(0);
    let mut best_set: bool = false;
    let mut best_count: SifrInt = SifrInt::from_i64(0);
    for val2 in data.iter().cloned() {
        let count2: Option<SifrInt> = counts.get(&val2).cloned();
        let mut count2_val: SifrInt = SifrInt::from_i64(0);
        if let Some(count2) = count2.clone() {
            count2_val = count2;
        }
        if &count2_val > &best_count {
            best_count = count2_val;
            best = val2;
            best_set = true;
        }
    }
    if best_set {
        return Ok(best.clone());
    }
    Err(
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
            "mode: no mode found".to_string(),
        ),
    )
}
fn multimode(
    data: &Vec<SifrInt>,
) -> Result<Vec<SifrInt>, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    if (&SifrInt::from(data.len()) == &SifrInt::from_i64(0)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "multimode requires at least one data point".to_string(),
            ),
        );
    }
    let mut counts: HashMap<SifrInt, SifrInt> = HashMap::from([]);
    for val in data.iter().cloned() {
        let existing: Option<SifrInt> = counts.get(&val).cloned();
        if let Some(existing) = existing.clone() {
            counts.insert(val.clone(), &existing + &SifrInt::from_i64(1));
        } else {
            counts.insert(val.clone(), SifrInt::from_i64(1));
        }
    }
    let mut max_count: SifrInt = SifrInt::from_i64(0);
    for val2 in data.iter().cloned() {
        let count2: Option<SifrInt> = counts.get(&val2).cloned();
        let mut count2_val: SifrInt = SifrInt::from_i64(0);
        if let Some(count2) = count2.clone() {
            count2_val = count2;
        }
        if &count2_val > &max_count {
            max_count = count2_val;
        }
    }
    let mut result: Vec<SifrInt> = vec![];
    let mut seen: HashMap<SifrInt, bool> = HashMap::from([]);
    for val3 in data.iter().cloned() {
        let already_opt: Option<bool> = seen.get(&val3).copied();
        let mut already: bool = false;
        if let Some(already_opt) = already_opt {
            already = already_opt;
        }
        if !already {
            let count3: Option<SifrInt> = counts.get(&val3).cloned();
            let mut count3_val: SifrInt = SifrInt::from_i64(0);
            if let Some(count3) = count3.clone() {
                count3_val = count3;
            }
            if &count3_val == &max_count {
                result.push(val3.clone());
            }
            seen.insert(val3.clone(), true);
        }
    }
    Ok(result)
}
fn quantiles(
    data: &Vec<f64>,
    n: SifrInt,
) -> Result<Vec<f64>, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    if (&SifrInt::from(data.len()) < &SifrInt::from_i64(2)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "quantiles requires at least two data points".to_string(),
            ),
        );
    }
    if &n < &SifrInt::from_i64(1) {
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
    let m: SifrInt = SifrInt::from(sorted_data.len());
    let mut result: Vec<f64> = vec![];
    let mut i: SifrInt = SifrInt::from_i64(1);
    while &i < &n {
        let __sifr_try_res: Result<
            (f64, f64, f64),
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
        > = (|| {
            let i_float: f64 = _float_int((i).clone())?;
            let m_float: f64 = _float_int((m).clone())?;
            let n_float: f64 = _float_int((n).clone())?;
            Ok((i_float, m_float, n_float))
        })();
        let (i_float, m_float, n_float) = match __sifr_try_res {
            Ok(__sifr_try_bindings) => __sifr_try_bindings,
            Err(__sifr_try_err) => {
                let error = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                        error.message.clone(),
                    ),
                );
            }
        };
        let idx_f: f64 = (i_float * m_float) / n_float;
        let mut idx: SifrInt = SifrInt::from_i64(0);
        let __sifr_try_res: Result<(), ValueError> = (|| {
            let converted_idx: SifrInt = SifrInt::from_f64_trunc(idx_f)
                .ok_or_else(|| ValueError {
                    message: "cannot convert non-finite float to int".to_string(),
                })?;
            idx = converted_idx;
            Ok(())
        })();
        if let Err(__sifr_try_err) = __sifr_try_res {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
        let __sifr_try_res: Result<
            (f64,),
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
        > = (|| {
            let idx_float: f64 = _float_int((idx).clone())?;
            Ok((idx_float,))
        })();
        let (idx_float,) = match __sifr_try_res {
            Ok(__sifr_try_bindings) => __sifr_try_bindings,
            Err(__sifr_try_err) => {
                let error = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                        error.message.clone(),
                    ),
                );
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
            let __sifr_index_list = &sorted_data;
            let __sifr_index_i = idx.clone();
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let mut lo_val: f64 = 0.0_f64;
        if let Some(lo) = lo {
            lo_val = lo;
        }
        if frac > (0.0_f64) {
            let hi_idx: SifrInt = &idx + &SifrInt::from_i64(1);
            if &hi_idx < &m {
                let hi: Option<f64> = Some(
                    sorted_data[::sifr_runtime::to_usize_proven(&(hi_idx))],
                );
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
fn covariance(
    x: &Vec<f64>,
    y: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: SifrInt = SifrInt::from(x.len());
    if &n < &SifrInt::from_i64(2) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "covariance requires at least two data points".to_string(),
            ),
        );
    }
    if (&SifrInt::from(y.len()) != &n) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "covariance: x and y must have the same length".to_string(),
            ),
        );
    }
    let __sifr_try_res: Result<
        (f64, f64),
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let mx: f64 = _divide_by_int(_sum(x), (n).clone())?;
        let my: f64 = _divide_by_int(_sum(y), (n).clone())?;
        Ok((mx, my))
    })();
    let (mx, my) = match __sifr_try_res {
        Ok(__sifr_try_bindings) => __sifr_try_bindings,
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    };
    let mut total: f64 = 0.0_f64;
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &n {
        let xi: Option<f64> = Some(x[::sifr_runtime::to_usize_proven(&(i))]);
        let yi: Option<f64> = {
            let __sifr_index_list = &y;
            let __sifr_index_i = i.clone();
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(xi) = xi {
            if let Some(yi) = yi {
                total += (xi - mx) * (yi - my);
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    _divide_by_int(total, &n - &SifrInt::from_i64(1))
}
fn correlation(
    x: &Vec<f64>,
    y: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: SifrInt = SifrInt::from(x.len());
    if &n < &SifrInt::from_i64(2) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "correlation requires at least two data points".to_string(),
            ),
        );
    }
    if (&SifrInt::from(y.len()) != &n) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "correlation: x and y must have the same length".to_string(),
            ),
        );
    }
    let __sifr_try_res: Result<
        (f64, f64),
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let mx: f64 = _divide_by_int(_sum(x), (n).clone())?;
        let my: f64 = _divide_by_int(_sum(y), (n).clone())?;
        Ok((mx, my))
    })();
    let (mx, my) = match __sifr_try_res {
        Ok(__sifr_try_bindings) => __sifr_try_bindings,
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    };
    let mut cov_num: f64 = 0.0_f64;
    let mut sx_num: f64 = 0.0_f64;
    let mut sy_num: f64 = 0.0_f64;
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &n {
        let xi: Option<f64> = Some(x[::sifr_runtime::to_usize_proven(&(i))]);
        let yi: Option<f64> = {
            let __sifr_index_list = &y;
            let __sifr_index_i = i.clone();
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(xi) = xi {
            if let Some(yi) = yi {
                cov_num += (xi - mx) * (yi - my);
                sx_num += (xi - mx) * (xi - mx);
                sy_num += (yi - my) * (yi - my);
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    let __sifr_try_res: Result<
        (f64, f64),
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let sx_variance: f64 = _divide_by_int(sx_num, &n - &SifrInt::from_i64(1))?;
        let sy_variance: f64 = _divide_by_int(sy_num, &n - &SifrInt::from_i64(1))?;
        let sx: f64 = sqrt(sx_variance);
        let sy: f64 = sqrt(sy_variance);
        Ok((sx, sy))
    })();
    let (sx, sy) = match __sifr_try_res {
        Ok(__sifr_try_bindings) => __sifr_try_bindings,
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    };
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
    let __sifr_try_res: Result<
        Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError>,
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let covariance_value: f64 = _divide_by_int(cov_num, &n - &SifrInt::from_i64(1))?;
        Ok(Ok(covariance_value / (sx * sy)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    }
}
fn linear_regression(
    x: &Vec<f64>,
    y: &Vec<f64>,
) -> Result<Vec<f64>, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let n: SifrInt = SifrInt::from(x.len());
    if &n < &SifrInt::from_i64(2) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "linear_regression requires at least two data points".to_string(),
            ),
        );
    }
    if (&SifrInt::from(y.len()) != &n) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "linear_regression: x and y must have the same length".to_string(),
            ),
        );
    }
    let __sifr_try_res: Result<
        (f64, f64),
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let mx: f64 = _divide_by_int(_sum(x), (n).clone())?;
        let my: f64 = _divide_by_int(_sum(y), (n).clone())?;
        Ok((mx, my))
    })();
    let (mx, my) = match __sifr_try_res {
        Ok(__sifr_try_bindings) => __sifr_try_bindings,
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    };
    let mut num: f64 = 0.0_f64;
    let mut den: f64 = 0.0_f64;
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &n {
        let xi: Option<f64> = Some(x[::sifr_runtime::to_usize_proven(&(i))]);
        let yi: Option<f64> = {
            let __sifr_index_list = &y;
            let __sifr_index_i = i.clone();
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(xi) = xi {
            if let Some(yi) = yi {
                num += (xi - mx) * (yi - my);
                den += (xi - mx) * (xi - mx);
            }
        }
        i = &i + &SifrInt::from_i64(1);
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
impl ::std::error::Error for Error {}
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
struct FloatOverflowError {
    message: String,
}
impl FloatOverflowError {
    fn new(message: String) -> Self {
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
struct FloatPrecisionLossError {
    message: String,
}
impl FloatPrecisionLossError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for FloatPrecisionLossError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for FloatPrecisionLossError {}
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
impl From<FloatOverflowError> for Error {
    fn from(err: FloatOverflowError) -> Self {
        Self::new(err.message)
    }
}
impl From<FloatPrecisionLossError> for Error {
    fn from(err: FloatPrecisionLossError) -> Self {
        Self::new(err.message)
    }
}
fn add(a: SifrInt, b: SifrInt) -> SifrInt {
    &a + &b
}
fn mul(a: SifrInt, b: SifrInt) -> SifrInt {
    &a * &b
}
fn less_than_three(x: SifrInt) -> bool {
    &x < &SifrInt::from_i64(3)
}
fn main() {
    println!("=== math additions ===");
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(13usize + 0usize);
        __sifr_concat.push_str("acosh(1.0) = "); __sifr_concat.push_str((format!("{}",
        acosh(1.0_f64))).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(13usize + 0usize);
        __sifr_concat.push_str("asinh(0.0) = "); __sifr_concat.push_str((format!("{}",
        asinh(0.0_f64))).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(13usize + 0usize);
        __sifr_concat.push_str("atanh(0.0) = "); __sifr_concat.push_str((format!("{}",
        atanh(0.0_f64))).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(12usize + 0usize);
        __sifr_concat.push_str("isqrt(17) = "); __sifr_concat.push_str((format!("{}",
        isqrt(SifrInt::from_i64(17)))).as_str()); __sifr_concat }
    );
    let p: Vec<f64> = vec![0.0_f64, 0.0_f64];
    let q: Vec<f64> = vec![3.0_f64, 4.0_f64];
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(20usize + 0usize);
        __sifr_concat.push_str("dist([0,0],[3,4]) = "); __sifr_concat
        .push_str((format!("{}", dist(& p, & q))).as_str()); __sifr_concat }
    );
    let data_fsum: Vec<f64> = vec![
        0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64, 0.1_f64,
        0.1_f64
    ];
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(15usize + 0usize);
        __sifr_concat.push_str("fsum(10x0.1) = "); __sifr_concat.push_str((format!("{}",
        fsum(& data_fsum))).as_str()); __sifr_concat }
    );
    println!("=== statistics (Result[float, StatisticsError]) ===");
    let data: Vec<f64> = vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64, 5.0_f64];
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (||
    {
        let m: f64 = mean(&data)?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(7usize +
            0usize); __sifr_concat.push_str("mean = "); __sifr_concat
            .push_str((format!("{}", m)).as_str()); __sifr_concat }
        );
        let med: f64 = median(&data)?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(9usize +
            0usize); __sifr_concat.push_str("median = "); __sifr_concat
            .push_str((format!("{}", med)).as_str()); __sifr_concat }
        );
        let v: f64 = variance(&data)?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(11usize +
            0usize); __sifr_concat.push_str("variance = "); __sifr_concat
            .push_str((format!("{}", v)).as_str()); __sifr_concat }
        );
        let s: f64 = stdev(&data)?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(8usize +
            0usize); __sifr_concat.push_str("stdev = "); __sifr_concat
            .push_str((format!("{}", s)).as_str()); __sifr_concat }
        );
        let idata: Vec<SifrInt> = vec![
            SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(2),
            SifrInt::from_i64(3), SifrInt::from_i64(3), SifrInt::from_i64(3)
        ];
        let mo: SifrInt = mode(&idata)?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(7usize +
            0usize); __sifr_concat.push_str("mode = "); __sifr_concat
            .push_str((format!("{}", mo)).as_str()); __sifr_concat }
        );
        let mm: Vec<SifrInt> = multimode(
            &vec![
                SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(2),
                SifrInt::from_i64(3), SifrInt::from_i64(3)
            ],
        )?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(16usize +
            0usize); __sifr_concat.push_str("multimode len = "); __sifr_concat
            .push_str((format!("{}", SifrInt::from(mm.len()))).as_str()); __sifr_concat }
        );
        let qs: Vec<f64> = quantiles(&data, SifrInt::from_i64(4))?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(18usize +
            0usize); __sifr_concat.push_str("quartiles count = "); __sifr_concat
            .push_str((format!("{}", SifrInt::from(qs.len()))).as_str()); __sifr_concat }
        );
        let x: Vec<f64> = vec![1.0_f64, 2.0_f64, 3.0_f64, 4.0_f64, 5.0_f64];
        let y: Vec<f64> = vec![2.0_f64, 4.0_f64, 6.0_f64, 8.0_f64, 10.0_f64];
        let cov: f64 = covariance(&x, &y)?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(13usize +
            0usize); __sifr_concat.push_str("covariance = "); __sifr_concat
            .push_str((format!("{}", cov)).as_str()); __sifr_concat }
        );
        let r: f64 = correlation(&x, &y)?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(14usize +
            0usize); __sifr_concat.push_str("correlation = "); __sifr_concat
            .push_str((format!("{}", r)).as_str()); __sifr_concat }
        );
        let lr: Vec<f64> = linear_regression(&x, &y)?;
        let slope: Option<f64> = {
            let __sifr_index_list = &lr;
            let __sifr_index_i = SifrInt::from_i64(0);
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let intercept: Option<f64> = {
            let __sifr_index_list = &lr;
            let __sifr_index_i = SifrInt::from_i64(1);
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(slope) = slope {
            println!(
                "{}", { let mut __sifr_concat : String = String::with_capacity(8usize +
                0usize); __sifr_concat.push_str("slope = "); __sifr_concat
                .push_str((format!("{}", slope)).as_str()); __sifr_concat }
            );
        }
        if let Some(intercept) = intercept {
            println!(
                "{}", { let mut __sifr_concat : String = String::with_capacity(12usize +
                0usize); __sifr_concat.push_str("intercept = "); __sifr_concat
                .push_str((format!("{}", intercept)).as_str()); __sifr_concat }
            );
        }
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(7usize +
            0usize); __sifr_concat.push_str("error: "); __sifr_concat.push_str((e.message
            .clone()).as_str()); __sifr_concat }
        );
    }
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (||
    {
        let empty: Vec<f64> = vec![];
        let bad: f64 = mean(&empty)?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(18usize +
            0usize); __sifr_concat.push_str("empty mean error: "); __sifr_concat
            .push_str((e.message.clone()).as_str()); __sifr_concat }
        );
    }
    println!("=== random additions ===");
    let mut items: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3),
        SifrInt::from_i64(4), SifrInt::from_i64(5)
    ];
    shuffle(&mut items);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(14usize + 0usize);
        __sifr_concat.push_str("shuffle len = "); __sifr_concat.push_str((format!("{}",
        SifrInt::from(items.len()))).as_str()); __sifr_concat }
    );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let s3: Vec<SifrInt> = sample(&items, SifrInt::from_i64(3))?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(16usize +
            0usize); __sifr_concat.push_str("sample(3) len = "); __sifr_concat
            .push_str((format!("{}", SifrInt::from(s3.len()))).as_str()); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(14usize +
            0usize); __sifr_concat.push_str("sample error: "); __sifr_concat.push_str((e
            .message.clone()).as_str()); __sifr_concat }
        );
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let rr: SifrInt = randrange(
            SifrInt::from_i64(0),
            Some(SifrInt::from_i64(100)),
            SifrInt::from_i64(5),
        )?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(21usize +
            0usize); __sifr_concat.push_str("randrange in range = "); __sifr_concat
            .push_str((format!("{}", & rr >= & SifrInt::from_i64(0))).as_str());
            __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(17usize +
            0usize); __sifr_concat.push_str("randrange error: "); __sifr_concat
            .push_str((e.message.clone()).as_str()); __sifr_concat }
        );
    }
    let g: f64 = gauss(0.0_f64, 1.0_f64);
    println!("gauss sample is float = True");
    println!("=== functools.reduce ===");
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3),
        SifrInt::from_i64(4), SifrInt::from_i64(5)
    ];
    let total: SifrInt = reduce(
        |__arg0, __arg1| add((__arg0).clone(), (__arg1).clone()),
        &nums,
        &SifrInt::from_i64(0),
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(14usize + 0usize);
        __sifr_concat.push_str("reduce(add) = "); __sifr_concat.push_str((format!("{}",
        total)).as_str()); __sifr_concat }
    );
    let product: SifrInt = reduce(
        |__arg0, __arg1| mul((__arg0).clone(), (__arg1).clone()),
        &nums,
        &SifrInt::from_i64(1),
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(14usize + 0usize);
        __sifr_concat.push_str("reduce(mul) = "); __sifr_concat.push_str((format!("{}",
        product)).as_str()); __sifr_concat }
    );
    println!("=== itertools additions ===");
    let idata2: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3),
        SifrInt::from_i64(4), SifrInt::from_i64(5)
    ];
    let acc: Vec<SifrInt> = accumulate(
            &(idata2).iter().cloned().collect::<Vec<_>>(),
            None,
        )
        .collect::<Vec<_>>();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(13usize + 0usize);
        __sifr_concat.push_str("accumulate = "); __sifr_concat.push_str((format!("{:?}",
        acc)).as_str()); __sifr_concat }
    );
    let sel: Vec<bool> = vec![true, false, true, false, true];
    let comp: Vec<SifrInt> = compress(
            &(idata2).iter().cloned().collect::<Vec<_>>(),
            &(sel).iter().copied().collect::<Vec<_>>(),
        )
        .collect::<Vec<_>>();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(11usize + 0usize);
        __sifr_concat.push_str("compress = "); __sifr_concat.push_str((format!("{:?}",
        comp)).as_str()); __sifr_concat }
    );
    let dw: Vec<SifrInt> = dropwhile(
            |__arg0| less_than_three((__arg0).clone()),
            &(idata2).iter().cloned().collect::<Vec<_>>(),
        )
        .collect::<Vec<_>>();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("dropwhile(<3) = "); __sifr_concat
        .push_str((format!("{:?}", dw)).as_str()); __sifr_concat }
    );
    let tw: Vec<SifrInt> = takewhile(
            |__arg0| less_than_three((__arg0).clone()),
            &(idata2).iter().cloned().collect::<Vec<_>>(),
        )
        .collect::<Vec<_>>();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("takewhile(<3) = "); __sifr_concat
        .push_str((format!("{:?}", tw)).as_str()); __sifr_concat }
    );
    let ff: Vec<SifrInt> = filterfalse(
            |__arg0| less_than_three((__arg0).clone()),
            &(idata2).iter().cloned().collect::<Vec<_>>(),
        )
        .collect::<Vec<_>>();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(18usize + 0usize);
        __sifr_concat.push_str("filterfalse(<3) = "); __sifr_concat
        .push_str((format!("{:?}", ff)).as_str()); __sifr_concat }
    );
    let a: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)
    ];
    let b: Vec<SifrInt> = vec![SifrInt::from_i64(4), SifrInt::from_i64(5)];
    let zl: Vec<Vec<SifrInt>> = zip_longest(
            &(a).iter().cloned().collect::<Vec<_>>(),
            &(b).iter().cloned().collect::<Vec<_>>(),
            &SifrInt::from_i64(0),
        )
        .collect::<Vec<_>>();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(18usize + 0usize);
        __sifr_concat.push_str("zip_longest len = "); __sifr_concat
        .push_str((format!("{}", SifrInt::from(zl.len()))).as_str()); __sifr_concat }
    );
    let cf: Vec<SifrInt> = count_from(
            SifrInt::from_i64(0),
            SifrInt::from_i64(2),
            SifrInt::from_i64(5),
        )
        .collect::<Vec<_>>();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(20usize + 0usize);
        __sifr_concat.push_str("count_from(0,2,5) = "); __sifr_concat
        .push_str((format!("{:?}", cf)).as_str()); __sifr_concat }
    );
    let mut ctr: Box<dyn Iterator<Item = SifrInt>> = count(
        SifrInt::from_i64(0),
        SifrInt::from_i64(2),
    );
    let count0: Option<SifrInt> = ctr.next();
    let count1: Option<SifrInt> = ctr.next();
    let count2: Option<SifrInt> = ctr.next();
    let count3: Option<SifrInt> = ctr.next();
    let count4: Option<SifrInt> = ctr.next();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(21usize + 0usize);
        __sifr_concat.push_str("count(0,2) first 5 = "); __sifr_concat
        .push_str((format!("{:?}", vec![count0, count1, count2, count3, count4]))
        .as_str()); __sifr_concat }
    );
    let cyc: Vec<SifrInt> = cycle(
            &(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)])
                .into_iter()
                .collect::<Vec<_>>(),
            SifrInt::from_i64(7),
        )
        .collect::<Vec<_>>();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(20usize + 0usize);
        __sifr_concat.push_str("cycle([1,2,3], 7) = "); __sifr_concat
        .push_str((format!("{:?}", cyc)).as_str()); __sifr_concat }
    );
    println!("=== Counter enhancements ===");
    let mut c1: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(
        &vec!["a".to_string(), "b".to_string(), "a".to_string(), "c".to_string()],
    );
    let c2: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(
        &vec!["b".to_string(), "c".to_string(), "d".to_string()],
    );
    c1.update(&c2);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(((16usize +
        0usize) + 3usize) + 0usize); __sifr_concat.push_str("after update: a=");
        __sifr_concat.push_str((format!("{}", c1.get(& "a".to_string(), &
        SifrInt::from_i64(0)))).as_str()); __sifr_concat.push_str(" b="); __sifr_concat
        .push_str((format!("{}", c1.get(& "b".to_string(), & SifrInt::from_i64(0))))
        .as_str()); __sifr_concat }
    );
    let mut c3: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(
        &vec!["x".to_string(), "x".to_string(), "y".to_string()],
    );
    let c4: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(
        &vec!["x".to_string()],
    );
    c3.subtract(&c4);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(18usize + 0usize);
        __sifr_concat.push_str("after subtract: x="); __sifr_concat
        .push_str((format!("{}", c3.get(& "x".to_string(), & SifrInt::from_i64(0))))
        .as_str()); __sifr_concat }
    );
    let c5: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(
        &vec!["a".to_string(), "a".to_string(), "b".to_string()],
    );
    let elems: Vec<String> = c5.elements();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(15usize + 0usize);
        __sifr_concat.push_str("elements len = "); __sifr_concat.push_str((format!("{}",
        SifrInt::from(elems.len()))).as_str()); __sifr_concat }
    );
    let mut cc: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(
        &vec!["a".to_string(), "b".to_string()],
    );
    cc.update(&from_list(&vec!["b".to_string(), "c".to_string()]));
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("counter_add b = "); __sifr_concat.push_str((format!("{}",
        cc.get(& "b".to_string(), & SifrInt::from_i64(0)))).as_str()); __sifr_concat }
    );
    let mut cd: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(
        &vec!["a".to_string(), "a".to_string(), "b".to_string()],
    );
    cd.subtract(&from_list(&vec!["a".to_string()]));
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("counter_sub a = "); __sifr_concat.push_str((format!("{}",
        cd.get(& "a".to_string(), & SifrInt::from_i64(0)))).as_str()); __sifr_concat }
    );
    println!("=== stdlib_pure_expansion: all features demonstrated ===");
}
