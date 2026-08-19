// src/main.rs
mod __sifr_project_nominals {
    pub use ::std::collections::VecDeque;
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
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ecollections_x2edeque;
use ::std::collections::HashMap;
use ::std::collections::HashSet;
use ::std::collections::VecDeque;
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
        (groups.get("hit").map_or(0, | __sifr_bucket | __sifr_bucket.len() as i64) ==
        (2_i64))
    );
    let mut seen: HashMap<i64, HashSet<String>> = HashMap::new();
    {
        seen.entry(1_i64).or_insert(HashSet::new()).insert("a".to_string());
        ()
    };
    {
        seen.entry(1_i64).or_insert(HashSet::new()).insert("b".to_string());
        ()
    };
    assert!(seen.entry(1_i64).or_insert(HashSet::new()).contains(& ("a".to_string())));
    let mut counts: HashMap<String, i64> = HashMap::new();
    {
        let __elem = counts.entry("steps".to_string()).or_insert(0);
        *__elem += 1_i64;
    }
    {
        let __elem = counts.entry("steps".to_string()).or_insert(0);
        *__elem += 2_i64;
    }
    assert!(* counts.entry("steps".to_string()).or_insert(0) == (3_i64));
    let q: __SifrStdlib_sifr_x2ecollections_x2edeque<i64> = __SifrStdlib_sifr_x2ecollections_x2edeque::new(
        Some(vec![1_i64, 2_i64, 3_i64]),
        None,
    );
    assert!((q.len() as i64) == (3_i64));
}
