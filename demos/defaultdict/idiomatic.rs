use std::collections::HashMap;

use std::collections::HashSet;

use std::collections::VecDeque;

// --- stdlib: sifr.collections ---
#[derive(Debug, Clone, PartialEq)]
struct deque<T: Clone + std::fmt::Display + PartialOrd> {
    _data: VecDeque<T>,
    maxlen: Option<i64>,
}
impl<T: Clone + std::fmt::Display + PartialOrd> deque<T> {
    fn new(items: Option<Vec<T>>, maxlen: Option<i64>) -> Self {
        let mut data: Vec<T> = vec![];
        if let Some(items) = items {
            let mut start: i64 = 0 as i64;
            if let Some(maxlen) = maxlen {
                if (items.len() as i64) > maxlen {
                    start = (items.len() as i64) - maxlen;
                }
            }
            let mut i: i64 = start;
            while i < (items.len() as i64) {
                let item: Option<T> = Some(items[i as usize].clone());
                if let Some(item) = item {
                    data.push(item.clone());
                }
                i += 1 as i64;
            }
        }
        return Self {
            maxlen: maxlen,
            _data: VecDeque::from(data),
        };
    }
    fn append(&mut self, val: &T) {
        self._data.push_back(val.clone());
        let maxlen_opt: Option<i64> = self.maxlen;
        if let Some(maxlen_opt) = maxlen_opt {
            let maxlen: i64 = maxlen_opt;
            if (self._data.clone().len() as i64) > maxlen {
                self._data.pop_front();
            }
        }
    }
    fn appendleft(&mut self, val: &T) {
        self._data.push_front(val.clone());
        let maxlen_opt: Option<i64> = self.maxlen;
        if let Some(maxlen_opt) = maxlen_opt {
            let maxlen: i64 = maxlen_opt;
            if (self._data.clone().len() as i64) > maxlen {
                self._data.pop_back();
            }
        }
    }
    fn pop(&mut self) -> Option<T> {
        if (self._data.clone().len() as i64) == (0 as i64) {
            return None;
        }
        return self._data.pop_back();
    }
    fn popleft(&mut self) -> Option<T> {
        if (self._data.clone().len() as i64) == (0 as i64) {
            return None;
        }
        return self._data.pop_front();
    }
    fn len(&self) -> i64 {
        return self._data.clone().len() as i64;
    }
    fn to_list(&self) -> Vec<T> {
        let mut result: Vec<T> = vec![];
        for v in self._data.clone().iter().cloned() {
            result.push(v.clone());
        }
        return result;
    }
    fn clear(&mut self) {
        self._data.clear();
    }
    fn extend(&mut self, items: &Vec<T>) {
        for v in items.iter().cloned() {
            self._data.push_back(v.clone());
        }
        let maxlen_opt: Option<i64> = self.maxlen;
        if let Some(maxlen_opt) = maxlen_opt {
            let maxlen: i64 = maxlen_opt;
            while (self._data.clone().len() as i64) > maxlen {
                self._data.pop_front();
            }
        }
    }
    fn extendleft(&mut self, items: &Vec<T>) {
        for v in items.iter().cloned() {
            self._data.push_front(v.clone());
        }
        let maxlen_opt: Option<i64> = self.maxlen;
        if let Some(maxlen_opt) = maxlen_opt {
            let maxlen: i64 = maxlen_opt;
            while (self._data.clone().len() as i64) > maxlen {
                self._data.pop_back();
            }
        }
    }
    fn copy(&self) -> deque<T> {
        return deque::new(Some(self.to_list()), self.maxlen);
    }
    fn reverse(&mut self) {
        let mut items: Vec<T> = self.to_list();
        items.reverse();
        self._data.clear();
        for item in items.iter().cloned() {
            self._data.push_back(item.clone());
        }
    }
    fn rotate(&mut self, n: i64) {
        let length: i64 = self._data.clone().len() as i64;
        if length == (0 as i64) {
            return;
        }
        let mut steps: i64 = n % length;
        if steps < (0 as i64) {
            steps = steps + length;
        }
        let mut count: i64 = 0 as i64;
        while count < steps {
            let value: Option<T> = self._data.pop_back();
            if let Some(value) = value {
                self._data.push_front(value.clone());
            }
            count = count + (1 as i64);
        }
    }
    fn count(&self, value: &T) -> i64 {
        let mut total: i64 = 0 as i64;
        for item in self._data.clone().iter().cloned() {
            if item == *value {
                total = total + (1 as i64);
            }
        }
        return total;
    }
    fn index(&self, value: &T, start: i64, stop: Option<i64>) -> Option<i64> {
        let size: i64 = self._data.clone().len() as i64;
        let mut begin: i64 = start;
        if begin < (0 as i64) {
            begin = size + begin;
            if begin < (0 as i64) {
                begin = 0 as i64;
            }
        }
        let mut end: i64 = size;
        if let Some(stop) = stop {
            end = stop;
            if end < (0 as i64) {
                end = size + end;
            }
            if end < (0 as i64) {
                end = 0 as i64;
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
            i = i + (1 as i64);
        }
        return None;
    }
    fn remove(&mut self, value: &T) {
        let idx: Option<i64> = self.index(value, 0 as i64, None);
        if let Some(idx) = idx {
            let mut rebuilt: Vec<T> = vec![];
            let mut i: i64 = 0 as i64;
            while i < (self._data.clone().len() as i64) {
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
                    if i != idx {
                        rebuilt.push(current.clone());
                    }
                }
                i = i + (1 as i64);
            }
            self._data.clear();
            for item in rebuilt.iter().cloned() {
                self._data.push_back(item.clone());
            }
        }
    }
}

fn main() {
    let mut groups = HashMap::new();
    {
        groups
            .entry("hit".to_string())
            .or_insert(Vec::new())
            .push("hot".to_string());
        ()
    };
    {
        groups
            .entry("hit".to_string())
            .or_insert(Vec::new())
            .push("hut".to_string());
        ()
    };
    assert!(
        (groups
            .entry("hit".to_string())
            .or_insert(Vec::new())
            .clone()
            .len() as i64)
            == (2 as i64)
    );
    let mut seen = HashMap::new();
    {
        seen.entry((1 as i64).clone())
            .or_insert(HashSet::new())
            .insert("a".to_string());
        ()
    };
    {
        seen.entry((1 as i64).clone())
            .or_insert(HashSet::new())
            .insert("b".to_string());
        ()
    };
    assert!(seen
        .entry((1 as i64).clone())
        .or_insert(HashSet::new())
        .clone()
        .contains(&"a".to_string()));
    let mut counts = HashMap::new();
    if let Some(__elem) = counts.get_mut("steps".to_string()) {
        *__elem += 1 as i64;
    }
    if let Some(__elem) = counts.get_mut("steps".to_string()) {
        *__elem += 2 as i64;
    }
    assert!(*counts.entry("steps".to_string()).or_insert(0) == (3 as i64));
    let mut q = deque::new(Some(vec![1 as i64, 2 as i64, 3 as i64]), None);
    assert!((q.len() as i64) == (3 as i64));
}
