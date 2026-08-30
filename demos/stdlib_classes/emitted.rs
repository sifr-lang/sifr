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
                let Some(__sifr_checked_value_0) = ({
                    let __sifr_checked_read_collection = &result;
                    let __sifr_checked_read_index = i.clone();
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                }) else {
                    break;
                };
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
                let Some(__sifr_checked_value_4) = ({
                    let __sifr_checked_read_collection = &result;
                    let __sifr_checked_read_index = i.clone();
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                }) else {
                    break;
                };
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
                let Some(__sifr_checked_value_7) = ({
                    let __sifr_checked_read_collection = &all_keys;
                    let __sifr_checked_read_index = ki.clone();
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                }) else {
                    break;
                };
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
}
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
fn main() {
    let words: Vec<String> = vec![
        "apple".to_string(), "banana".to_string(), "apple".to_string(), "cherry"
        .to_string(), "banana".to_string(), "apple".to_string()
    ];
    let mut c: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(&words);
    println!("{}", c.get(& "apple".to_string(), & SifrInt::from_i64(0)));
    println!("{}", c.get(& "banana".to_string(), & SifrInt::from_i64(0)));
    println!("{}", c.get(& "cherry".to_string(), & SifrInt::from_i64(0)));
    println!("{}", c.get(& "missing".to_string(), & SifrInt::from_i64(0)));
    println!("{}", c.total());
    println!("{:?}", c.most_common(& Some((SifrInt::from_i64(2)).clone())));
    c.increment(&"banana".to_string());
    c.increment(&"banana".to_string());
    println!("{}", c.get(& "banana".to_string(), & SifrInt::from_i64(0)));
    println!("{}", c.total());
    c.increment(&"date".to_string());
    println!("{}", c.get(& "date".to_string(), & SifrInt::from_i64(0)));
    println!("{}", c.total());
    let c2: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = __SifrStdlib_sifr_x2ecollections_x2eCounter::new(
        Some(
            HashMap::from([
                ("x".to_string(), SifrInt::from_i64(10)),
                ("y".to_string(), SifrInt::from_i64(20)),
            ]),
        ),
        None,
    );
    println!("{}", c2.get(& "x".to_string(), & SifrInt::from_i64(0)));
    println!("{}", c2.get(& "y".to_string(), & SifrInt::from_i64(0)));
    println!("{}", c2.total());
}
