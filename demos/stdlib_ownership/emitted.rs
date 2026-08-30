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
}
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ecollections_x2eCounter;
use ::std::collections::HashMap;
use ::sifr_runtime::SifrInt;
fn bisect_left<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
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
            if val < *x {
                left = &mid + &SifrInt::from_i64(1);
            } else {
                right = mid;
            }
        } else {
            left = &mid + &SifrInt::from_i64(1);
        }
    }
    left.clone()
}
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
fn insort_left<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    a: &mut Vec<T>,
    x: &T,
    lo: SifrInt,
    hi: Option<SifrInt>,
) {
    let pos: SifrInt = bisect_left(a, x, (lo).clone(), (hi).clone());
    a.insert(::sifr_runtime::to_usize_proven(&pos), x.clone());
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
    {
        let Some(__sifr_nonempty_pop_value) = heap.pop() else {
            unreachable!("compiler-verified non-empty pop should return Some");
        };
        __sifr_nonempty_pop_value
    };
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
fn nsmallest<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    n: SifrInt,
    data: &Vec<T>,
) -> Vec<T> {
    let mut heap: Vec<T> = data.clone();
    heapify(&mut heap);
    let mut result: Vec<T> = vec![];
    let mut count: SifrInt = SifrInt::from_i64(0);
    while &count < &n {
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
    while &count < &n {
        if &i < &SifrInt::from_i64(0) {
            return result2;
        }
        let v: Option<T> = {
            let __sifr_index_list = &all_sorted;
            let __sifr_index_i = i.clone();
            let __sifr_index_norm = __sifr_index_i
                .normalize_index_or_len(__sifr_index_list.len());
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(v) = v {
            result2.push(v.clone());
        }
        i = &i - &SifrInt::from_i64(1);
        count = &count + &SifrInt::from_i64(1);
    }
    result2
}
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
                        _yields.push(item.clone());
                    }
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            __sifr_generator_iter.next()
        }),
    )
}
fn demo_heapq() {
    println!("=== Section 1: heapq with mut params ===");
    let mut data: Vec<SifrInt> = vec![
        SifrInt::from_i64(5), SifrInt::from_i64(3), SifrInt::from_i64(8),
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(7),
        SifrInt::from_i64(4)
    ];
    heapify(&mut data);
    println!("heapified (min at root):");
    let min_val: Option<SifrInt> = {
        let __sifr_index_list = &data;
        let __sifr_index_i = SifrInt::from_i64(0);
        let __sifr_index_norm = __sifr_index_i
            .normalize_index_or_len(__sifr_index_list.len());
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    if let Some(min_val) = min_val.clone() {
        println!("{}", min_val);
    }
    heappush(&mut data, &SifrInt::from_i64(0));
    println!("after push(0), new min:");
    let new_min: Option<SifrInt> = {
        let __sifr_index_list = &data;
        let __sifr_index_i = SifrInt::from_i64(0);
        let __sifr_index_norm = __sifr_index_i
            .normalize_index_or_len(__sifr_index_list.len());
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    if let Some(new_min) = new_min.clone() {
        println!("{}", new_min);
    }
    let popped: Option<SifrInt> = heappop(&mut data);
    if let Some(popped) = popped.clone() {
        println!("popped:");
        println!("{}", popped);
    }
    println!("remaining size:");
    println!("{}", SifrInt::from(data.len()));
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(9), SifrInt::from_i64(3), SifrInt::from_i64(7),
        SifrInt::from_i64(1), SifrInt::from_i64(5), SifrInt::from_i64(6),
        SifrInt::from_i64(2), SifrInt::from_i64(8), SifrInt::from_i64(4)
    ];
    let small3: Vec<SifrInt> = nsmallest(SifrInt::from_i64(3), &items);
    let large3: Vec<SifrInt> = nlargest(SifrInt::from_i64(3), &items);
    println!("3 smallest:");
    println!("{:?}", small3);
    println!("3 largest:");
    println!("{:?}", large3);
    println!("items still valid, length:");
    println!("{}", SifrInt::from(items.len()));
}
fn demo_bisect() {
    println!("=== Section 2: bisect_right insort_right with mut params ===");
    let mut sorted_ints: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(3), SifrInt::from_i64(5),
        SifrInt::from_i64(7), SifrInt::from_i64(9)
    ];
    let pos_left: SifrInt = bisect_left(
        &sorted_ints,
        &SifrInt::from_i64(6),
        SifrInt::from_i64(0),
        None,
    );
    let pos_right: SifrInt = bisect_right(
        &sorted_ints,
        &SifrInt::from_i64(5),
        SifrInt::from_i64(0),
        None,
    );
    println!("insert 6 at position (left):");
    println!("{}", pos_left);
    println!("insert after 5 at position (right):");
    println!("{}", pos_right);
    insort_left(&mut sorted_ints, &SifrInt::from_i64(6), SifrInt::from_i64(0), None);
    println!("after insort_left(6):");
    println!("{:?}", sorted_ints);
    let mut data: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(2),
        SifrInt::from_i64(3)
    ];
    insort_right(&mut data, &SifrInt::from_i64(2), SifrInt::from_i64(0), None);
    println!("after insort_right(2) with duplicates:");
    println!("{:?}", data);
    insort_left(&mut sorted_ints, &SifrInt::from_i64(0), SifrInt::from_i64(0), None);
    insort_right(&mut sorted_ints, &SifrInt::from_i64(10), SifrInt::from_i64(0), None);
    println!("after more inserts:");
    println!("{:?}", sorted_ints);
}
fn demo_itertools() {
    println!("=== Section 3: itertools chain ===");
    let a: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)
    ];
    let b: Vec<SifrInt> = vec![
        SifrInt::from_i64(4), SifrInt::from_i64(5), SifrInt::from_i64(6)
    ];
    let result: Vec<SifrInt> = chain(&vec![(a).clone(), (b).clone()])
        .collect::<Vec<_>>();
    println!("chain (borrow both):");
    println!("{:?}", result);
    println!("a still usable:");
    println!("{}", SifrInt::from(a.len()));
    println!("b still usable:");
    println!("{}", SifrInt::from(b.len()));
    let x: Vec<SifrInt> = vec![
        SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30)
    ];
    let y: Vec<SifrInt> = vec![
        SifrInt::from_i64(40), SifrInt::from_i64(50), SifrInt::from_i64(60)
    ];
    let combined: Vec<SifrInt> = chain(&vec![(x).clone(), (y).clone()])
        .collect::<Vec<_>>();
    println!("chain result:");
    println!("{:?}", combined);
}
fn demo_counter() {
    println!("=== Section 4: Counter with native dict[str, int] ===");
    let words: Vec<String> = vec![
        "apple".to_string(), "banana".to_string(), "apple".to_string(), "cherry"
        .to_string(), "banana".to_string(), "apple".to_string()
    ];
    let mut c: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(&words);
    println!("apple count:");
    println!("{}", c.get(& "apple".to_string(), & SifrInt::from_i64(0)));
    println!("banana count:");
    println!("{}", c.get(& "banana".to_string(), & SifrInt::from_i64(0)));
    println!("missing key returns 0:");
    println!("{}", c.get(& "missing".to_string(), & SifrInt::from_i64(0)));
    println!("total elements:");
    println!("{}", c.total());
    c.increment(&"cherry".to_string());
    c.increment(&"cherry".to_string());
    println!("cherry after 2 increments:");
    println!("{}", c.get(& "cherry".to_string(), & SifrInt::from_i64(0)));
    let top: Vec<(String, SifrInt)> = c
        .most_common(&Some((SifrInt::from_i64(1)).clone()));
    println!("top 1 most common:");
    println!("{:?}", top);
    let keys: Vec<String> = c.keys();
    println!("unique keys count:");
    println!("{}", SifrInt::from(keys.len()));
}
fn main() {
    demo_heapq();
    demo_bisect();
    demo_itertools();
    demo_counter();
    println!("=== borrow_stdlib demo complete ===");
}
