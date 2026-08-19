// src/main.rs
mod __sifr_project_nominals {
    pub use ::std::collections::HashMap;
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
}
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ecollections_x2eCounter;
use ::std::collections::HashMap;
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
fn main() {
    let words: Vec<String> = vec![
        "apple".to_string(), "banana".to_string(), "apple".to_string(), "cherry"
        .to_string(), "banana".to_string(), "apple".to_string()
    ];
    let mut c: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(&words);
    println!("{}", c.get(& "apple".to_string(), 0_i64));
    println!("{}", c.get(& "banana".to_string(), 0_i64));
    println!("{}", c.get(& "cherry".to_string(), 0_i64));
    println!("{}", c.get(& "missing".to_string(), 0_i64));
    println!("{}", c.total());
    println!("{:?}", c.most_common(Some(2_i64)));
    c.increment(&"banana".to_string());
    c.increment(&"banana".to_string());
    println!("{}", c.get(& "banana".to_string(), 0_i64));
    println!("{}", c.total());
    c.increment(&"date".to_string());
    println!("{}", c.get(& "date".to_string(), 0_i64));
    println!("{}", c.total());
    let c2: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = __SifrStdlib_sifr_x2ecollections_x2eCounter::new(
        Some(HashMap::from([("x".to_string(), 10_i64), ("y".to_string(), 20_i64)])),
        None,
    );
    println!("{}", c2.get(& "x".to_string(), 0_i64));
    println!("{}", c2.get(& "y".to_string(), 0_i64));
    println!("{}", c2.total());
}
