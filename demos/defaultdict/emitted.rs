// src/main.rs
mod __sifr_project_nominals {
    pub use ::std::collections::VecDeque;
    pub use ::sifr_runtime::SifrInt;
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
        pub fn extend(&mut self, items: &[T]) {
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
        pub fn extendleft(&mut self, items: &[T]) {
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
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ecollections_x2edeque;
use ::std::collections::HashMap;
use ::std::collections::HashSet;
use ::std::collections::VecDeque;
use ::sifr_runtime::SifrInt;
fn main() {
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    {
        groups.entry("hit".to_string()).or_insert(Vec::new()).push("hot".to_string());
        ()
    };
    {
        groups.entry("hit".to_string()).or_insert(Vec::new()).push("hut".to_string());
        ()
    };
    assert!(
        (& groups.get("hit").map_or_else(|| SifrInt::from_i64(0), | __sifr_bucket |
        SifrInt::from(__sifr_bucket.len())) == & SifrInt::from_i64(2))
    );
    let mut seen: HashMap<SifrInt, HashSet<String>> = HashMap::new();
    {
        seen.entry(SifrInt::from_i64(1))
            .or_insert(HashSet::new())
            .insert("a".to_string());
        ()
    };
    {
        seen.entry(SifrInt::from_i64(1))
            .or_insert(HashSet::new())
            .insert("b".to_string());
        ()
    };
    assert!(
        seen.get(& SifrInt::from_i64(1)).is_some_and(| __sifr_defaultdict_bucket |
        __sifr_defaultdict_bucket.contains(& ("a".to_string())))
    );
    let mut counts: HashMap<String, SifrInt> = HashMap::new();
    {
        let __elem = counts.entry("steps".to_string()).or_insert(SifrInt::from_i64(0));
        *__elem += SifrInt::from_i64(1);
    }
    {
        let __elem = counts.entry("steps".to_string()).or_insert(SifrInt::from_i64(0));
        *__elem += SifrInt::from_i64(2);
    }
    assert!(
        &* counts.entry("steps".to_string()).or_insert(SifrInt::from_i64(0)) == &
        SifrInt::from_i64(3)
    );
    let q: __SifrStdlib_sifr_x2ecollections_x2edeque<SifrInt> = __SifrStdlib_sifr_x2ecollections_x2edeque::new(
        Some(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)]),
        None,
    );
    assert!(& SifrInt::from(q.len()) == & SifrInt::from_i64(3));
}
