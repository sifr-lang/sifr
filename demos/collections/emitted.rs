// src/main.rs
mod sifr_generated_generated_support {
    use crate::SifrGeneratedStdlibSifrX2ecollectionsX2eCounter;
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) use ::std::collections::HashMap;
    pub(crate) fn from_list<T: Clone + ::std::hash::Hash + Eq + 'static>(
        items: &[T],
    ) -> SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T> {
        let mut counts: HashMap<T, SifrInt> = HashMap::from([]);
        for item in items.iter().cloned() {
            let val: Option<SifrInt> = counts.get(&item).cloned();
            if let Some(val) = val.clone() {
                {
                    let sifr_generated_assign_value = &val + &SifrInt::from_i64(1);
                    {
                        let sifr_generated_assign_key = item.clone();
                        counts.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                    }
                }
            } else {
                let sifr_generated_assign_value = SifrInt::from_i64(1);
                {
                    let sifr_generated_assign_key = item.clone();
                    counts.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                }
            }
        }
        SifrGeneratedStdlibSifrX2ecollectionsX2eCounter::new(Some(counts), None)
    }
    pub(crate) fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
        assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from(actual.len()) {
            assert_eq!(
                {
                    let sifr_generated_condition_list = &actual;
                    let sifr_generated_condition_index = i.clone();
                    let sifr_generated_condition_normalized = sifr_generated_condition_index
                        .normalize_index_or_len(sifr_generated_condition_list.len());
                    sifr_generated_condition_list
                        .get(sifr_generated_condition_normalized)
                        .copied()
                },
                {
                    let sifr_generated_condition_list = &expected;
                    let sifr_generated_condition_index = i.clone();
                    let sifr_generated_condition_normalized = sifr_generated_condition_index
                        .normalize_index_or_len(sifr_generated_condition_list.len());
                    sifr_generated_condition_list
                        .get(sifr_generated_condition_normalized)
                        .copied()
                }
            );
            i = &i + &SifrInt::from_i64(1);
        }
    }
}
mod sifr_generated_project_nominals {
    use ::sifr_runtime::SifrInt;
    use ::std::collections::HashMap;
    use ::std::collections::VecDeque;
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T: std::hash::Hash + Eq> {
        pub counts: HashMap<T, SifrInt>,
    }
    impl<T: ::std::hash::Hash + Eq + Clone> SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T> {
        #[must_use]
        pub fn new(source: Option<HashMap<T, SifrInt>>, iterable: Option<Vec<T>>) -> Self {
            let mut counts: HashMap<T, SifrInt> = HashMap::from([]);
            if let Some(source) = source {
                for key in source.keys().cloned().collect::<Vec<_>>() {
                    let value: Option<SifrInt> = source.get(&key).cloned();
                    if let Some(value) = value.clone() {
                        {
                            let sifr_generated_assign_value = value.clone();
                            {
                                let sifr_generated_assign_key = key.clone();
                                counts
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    }
                }
            }
            if let Some(iterable) = iterable {
                for item in iterable.iter().cloned() {
                    let value2_value_2127bacf1a4dd308: Option<SifrInt> = counts.get(&item).cloned();
                    if let Some(value2) = value2_value_2127bacf1a4dd308.clone() {
                        {
                            let sifr_generated_assign_value = &value2 + &SifrInt::from_i64(1);
                            {
                                let sifr_generated_assign_key = item.clone();
                                counts
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    } else {
                        let sifr_generated_assign_value = SifrInt::from_i64(1);
                        {
                            let sifr_generated_assign_key = item.clone();
                            counts.insert(sifr_generated_assign_key, sifr_generated_assign_value);
                        }
                    }
                }
            }
            let sifr_generated_field_value_c341febe5aae51e5_636f756e7473: HashMap<T, SifrInt> =
                counts;
            Self {
                counts: sifr_generated_field_value_c341febe5aae51e5_636f756e7473,
            }
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T> {
        #[must_use]
        pub fn get(&self, key: &T, default: &SifrInt) -> SifrInt {
            let val: Option<SifrInt> = self.counts.get(key).cloned();
            let Some(val) = val.clone() else {
                return default.clone();
            };
            val
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone> SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T> {
        #[must_use]
        pub fn most_common(&self, n: &Option<SifrInt>) -> Vec<(T, SifrInt)> {
            let mut result: Vec<(T, SifrInt)> = Vec::new();
            for key in self.counts.keys().cloned().collect::<Vec<_>>() {
                let count: Option<SifrInt> = self.counts.get(&key).cloned();
                if let Some(count) = count.clone() {
                    let entry: (T, SifrInt) = (key.clone(), count.clone());
                    result.push(entry.clone());
                }
            }
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &SifrInt::from_i64(0) <= &i && &i < &SifrInt::from(result.len()) {
                let mut j: SifrInt = &i + &SifrInt::from_i64(1);
                while &SifrInt::from_i64(0) <= &j && &j < &SifrInt::from(result.len()) {
                    let left: Option<(T, SifrInt)> = {
                        let sifr_generated_checked_read_collection = &result;
                        let sifr_generated_checked_read_index = i.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let right: Option<(T, SifrInt)> = {
                        let sifr_generated_checked_read_collection = &result;
                        let sifr_generated_checked_read_index = j.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(left) = left
                        && let Some(right) = right
                        && right.1.clone() > left.1.clone()
                    {
                        {
                            let sifr_generated_assign_value = right.clone();
                            {
                                let sifr_generated_index_raw = i.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(result.len());
                                if let Some(sifr_generated_elem) =
                                    result.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                }
                            }
                        }
                        {
                            let sifr_generated_assign_value = left.clone();
                            {
                                let sifr_generated_index_raw = j.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(result.len());
                                if let Some(sifr_generated_elem) =
                                    result.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
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
            if n <= &SifrInt::from_i64(0) {
                return Vec::new();
            }
            let mut top: Vec<(T, SifrInt)> = Vec::new();
            let mut index: SifrInt = SifrInt::from_i64(0);
            while index < *n {
                if &index >= &SifrInt::from(result.len()) {
                    return top;
                }
                let value: Option<(T, SifrInt)> = {
                    let sifr_generated_checked_read_collection = &result;
                    let sifr_generated_checked_read_index = index.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
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
    impl<T: ::std::hash::Hash + Eq + Clone + PartialOrd>
        SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>
    {
        #[must_use]
        pub fn keys(&self) -> Vec<T> {
            let mut result: Vec<T> = self.counts.keys().cloned().collect::<Vec<_>>();
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &SifrInt::from_i64(0) <= &i && &i < &SifrInt::from(result.len()) {
                let mut j: SifrInt = &i + &SifrInt::from_i64(1);
                while &SifrInt::from_i64(0) <= &j && &j < &SifrInt::from(result.len()) {
                    let left: Option<T> = {
                        let sifr_generated_checked_read_collection = &result;
                        let sifr_generated_checked_read_index = i.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let right: Option<T> = {
                        let sifr_generated_checked_read_collection = &result;
                        let sifr_generated_checked_read_index = j.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(left) = left
                        && let Some(right) = right
                        && right < left
                    {
                        {
                            let sifr_generated_assign_value = right.clone();
                            {
                                let sifr_generated_index_raw = i.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(result.len());
                                if let Some(sifr_generated_elem) =
                                    result.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                }
                            }
                        }
                        {
                            let sifr_generated_assign_value = left.clone();
                            {
                                let sifr_generated_index_raw = j.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(result.len());
                                if let Some(sifr_generated_elem) =
                                    result.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
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
    impl<T: ::std::hash::Hash + Eq> SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T> {
        pub fn clear(&mut self) {
            self.counts = HashMap::new();
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone>
        ::std::ops::Add<&SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>>
        for &SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>
    {
        type Output = SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>;
        fn add(self, other: &SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>) -> Self::Output {
            let mut new_counts: HashMap<T, SifrInt> = HashMap::new();
            for key in Box::new(self.counts.keys().cloned().collect::<Vec<_>>().into_iter()) {
                let a_val: Option<SifrInt> = self.counts.get(&key).cloned();
                if let Some(a_val) = a_val {
                    let b_val_value_f4118a41fdffa885: Option<SifrInt> =
                        other.counts.get(&key).cloned();
                    let b_count: SifrInt = b_val_value_f4118a41fdffa885
                        .clone()
                        .unwrap_or_else(|| SifrInt::from_i64(0));
                    let total: SifrInt = &a_val + &b_count;
                    if &total > &SifrInt::from_i64(0) {
                        {
                            let sifr_generated_assign_value = total.clone();
                            {
                                let sifr_generated_assign_key = key.clone();
                                new_counts
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    }
                }
            }
            for key2 in Box::new(other.counts.keys().cloned().collect::<Vec<_>>().into_iter()) {
                let already: Option<SifrInt> = new_counts.get(&key2).cloned();
                if already.is_none() {
                    let b_val2: Option<SifrInt> = other.counts.get(&key2).cloned();
                    if let Some(b_val2) = b_val2
                        && &b_val2 > &SifrInt::from_i64(0)
                    {
                        {
                            let sifr_generated_assign_value = b_val2.clone();
                            {
                                let sifr_generated_assign_key = key2.clone();
                                new_counts
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    }
                }
            }
            SifrGeneratedStdlibSifrX2ecollectionsX2eCounter::new(Some(new_counts), None)
        }
    }
    impl<T: ::std::hash::Hash + Eq + Clone>
        ::std::ops::Sub<&SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>>
        for &SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>
    {
        type Output = SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>;
        fn sub(self, other: &SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<T>) -> Self::Output {
            let mut new_counts: HashMap<T, SifrInt> = HashMap::new();
            for key in Box::new(self.counts.keys().cloned().collect::<Vec<_>>().into_iter()) {
                let a_val: Option<SifrInt> = self.counts.get(&key).cloned();
                if let Some(a_val) = a_val {
                    let b_val_value_f4118a41fdffa885: Option<SifrInt> =
                        other.counts.get(&key).cloned();
                    let b_count: SifrInt = b_val_value_f4118a41fdffa885
                        .clone()
                        .unwrap_or_else(|| SifrInt::from_i64(0));
                    let diff: SifrInt = &a_val - &b_count;
                    if &diff > &SifrInt::from_i64(0) {
                        {
                            let sifr_generated_assign_value = diff.clone();
                            {
                                let sifr_generated_assign_key = key.clone();
                                new_counts
                                    .insert(sifr_generated_assign_key, sifr_generated_assign_value);
                            }
                        }
                    }
                }
            }
            SifrGeneratedStdlibSifrX2ecollectionsX2eCounter::new(Some(new_counts), None)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        pub data: VecDeque<T>,
        pub maxlen: Option<SifrInt>,
    }
    impl<T: Clone> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        #[must_use]
        pub fn new(items: Option<Vec<T>>, maxlen: Option<SifrInt>) -> Self {
            let mut data: Vec<T> = Vec::new();
            if let Some(items) = items {
                let start: SifrInt = if let Some(maxlen) = maxlen.clone()
                    && &SifrInt::from(items.len()) > &maxlen
                {
                    &SifrInt::from(items.len()) - &maxlen
                } else {
                    SifrInt::from_i64(0)
                };
                let mut i: SifrInt = start.clone();
                while &i < &SifrInt::from(items.len()) {
                    let item_value_2841a0c596d6f426: Option<T> = {
                        let sifr_generated_checked_read_collection = &items;
                        let sifr_generated_checked_read_index = i.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(item) = item_value_2841a0c596d6f426 {
                        data.push(item.clone());
                    }
                    i = &i + &SifrInt::from_i64(1);
                }
            }
            let sifr_generated_field_value_169953f6befb0270_6d61786c656e: Option<SifrInt> =
                maxlen.clone();
            let sifr_generated_field_value_90770dc80a1c57ce_5f64617461: VecDeque<T> =
                VecDeque::from(data);
            Self {
                maxlen: sifr_generated_field_value_169953f6befb0270_6d61786c656e,
                data: sifr_generated_field_value_90770dc80a1c57ce_5f64617461,
            }
        }
    }
    impl<T: Clone> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        pub fn append(&mut self, val: &T) {
            self.data.push_back(val.clone());
            let maxlen_opt: Option<SifrInt> = self.maxlen.clone();
            if let Some(maxlen_opt) = maxlen_opt.clone() {
                let maxlen: SifrInt = maxlen_opt.clone();
                if &SifrInt::from(self.data.len()) > &maxlen {
                    self.data.pop_front();
                }
            }
        }
    }
    impl<T> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        #[must_use]
        pub fn pop(&mut self) -> Option<T> {
            if &SifrInt::from(self.data.len()) == &SifrInt::from_i64(0) {
                return None;
            }
            Some({
                let sifr_generated_nonempty_pop_index = self.data.len() - 1_usize;
                let mut sifr_generated_nonempty_pop_values = self
                    .data
                    .drain(sifr_generated_nonempty_pop_index..=sifr_generated_nonempty_pop_index)
                    .collect::<Vec<_>>();
                sifr_generated_nonempty_pop_values.remove(0_usize)
            })
        }
    }
    impl<T> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        #[must_use]
        pub fn popleft(&mut self) -> Option<T> {
            if &SifrInt::from(self.data.len()) == &SifrInt::from_i64(0) {
                return None;
            }
            Some({
                let sifr_generated_nonempty_pop_index = 0_usize;
                let mut sifr_generated_nonempty_pop_values = self
                    .data
                    .drain(sifr_generated_nonempty_pop_index..=sifr_generated_nonempty_pop_index)
                    .collect::<Vec<_>>();
                sifr_generated_nonempty_pop_values.remove(0_usize)
            })
        }
    }
    impl<T> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        #[must_use]
        pub fn len(&self) -> SifrInt {
            SifrInt::from(self.data.len())
        }
    }
    impl<T> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        pub fn clear(&mut self) {
            self.data.clear();
        }
    }
    impl<T: Clone + PartialEq> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        #[must_use]
        pub fn index(&self, value: &T, start: &SifrInt, stop: &Option<SifrInt>) -> Option<SifrInt> {
            let size: SifrInt = SifrInt::from(self.data.len());
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
                if &end < &SifrInt::from_i64(0) {
                    end = &size + &end;
                }
                if &end < &SifrInt::from_i64(0) {
                    end = SifrInt::from_i64(0);
                }
                if &end > &size {
                    end = size.clone();
                }
            }
            let mut i: SifrInt = begin.clone();
            while &i < &end {
                let current: Option<T> = {
                    let sifr_generated_checked_read_collection = &self.data;
                    let sifr_generated_checked_read_index = i.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(current) = current
                    && current == *value
                {
                    return Some(i);
                }
                i = &i + &SifrInt::from_i64(1);
            }
            None
        }
    }
    impl<T: Clone + PartialEq> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        pub fn remove(&mut self, value: &T) {
            let idx: Option<SifrInt> = self.index(value, &SifrInt::from_i64(0), &None);
            if let Some(idx) = idx.clone() {
                let mut rebuilt: Vec<T> = Vec::new();
                let mut i: SifrInt = SifrInt::from_i64(0);
                while &i < &SifrInt::from(self.data.len()) {
                    let current: Option<T> = {
                        let sifr_generated_checked_read_collection = &self.data;
                        let sifr_generated_checked_read_index = i.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(current) = current
                        && &i != &idx
                    {
                        rebuilt.push(current.clone());
                    }
                    i = &i + &SifrInt::from_i64(1);
                }
                self.data.clear();
                for item in rebuilt.iter().cloned() {
                    self.data.push_back(item.clone());
                }
            }
        }
    }
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
use ::std::collections::HashSet;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecollectionsX2eCounter;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecollectionsX2edeque;
fn collect_set_and_counter_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let left: HashSet<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
    ]
    .into_iter()
    .collect::<std::collections::HashSet<_>>();
    let right: HashSet<SifrInt> = vec![
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
    ]
    .into_iter()
    .collect::<std::collections::HashSet<_>>();
    actual.push(
        &SifrInt::from(
            left.r#union(&right)
                .cloned()
                .collect::<std::collections::HashSet<_>>()
                .len(),
        ) == &SifrInt::from_i64(5),
    );
    actual.push(
        &SifrInt::from(
            left.intersection(&right)
                .cloned()
                .collect::<std::collections::HashSet<_>>()
                .len(),
        ) == &SifrInt::from_i64(1),
    );
    let counts: SifrGeneratedStdlibSifrX2ecollectionsX2eCounter<String> = from_list(&vec![
        "x".to_string(),
        "y".to_string(),
        "x".to_string(),
        "z".to_string(),
        "x".to_string(),
        "y".to_string(),
    ]);
    actual.push(&counts.get(&"x".to_string(), &SifrInt::from_i64(0)) == &SifrInt::from_i64(3));
    actual.push(
        format!(
            "{:?}",
            counts.most_common(&Some(SifrInt::from_i64(2).clone()))
        )
        .as_str()
            == "[(\"x\", 3), (\"y\", 2)]".to_string().as_str(),
    );
    actual
}
fn collect_deque_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let mut d: SifrGeneratedStdlibSifrX2ecollectionsX2edeque<SifrInt> =
        SifrGeneratedStdlibSifrX2ecollectionsX2edeque::new(None, Some(SifrInt::from_i64(2)));
    (&mut d).append(&SifrInt::from_i64(10));
    (&mut d).append(&SifrInt::from_i64(20));
    (&mut d).append(&SifrInt::from_i64(30));
    actual.push(
        &d.len() == &SifrInt::from_i64(2) && (&mut d).popleft() == Some(SifrInt::from_i64(20)),
    );
    let _: Option<SifrInt> = (&mut d).pop();
    actual.push((&mut d).pop().is_none());
    actual
}
fn append_all(target: &mut Vec<bool>, values: &[bool]) {
    for value in values.iter().copied() {
        target.push(value);
    }
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let mut actual: Vec<bool> = Vec::new();
    append_all(&mut actual, &collect_set_and_counter_actual());
    append_all(&mut actual, &collect_deque_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("collections collections parity demo: pass");
}
