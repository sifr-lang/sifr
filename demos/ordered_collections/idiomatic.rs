use std::collections::HashMap;

use std::collections::VecDeque;

// --- stdlib: sifr.collections ---
#[derive(Debug, Clone, PartialEq)]
struct Counter<T: Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq> {
    counts: HashMap<T, i64>,
}
impl<T: Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq> Counter<T> {
    fn new(source: Option<HashMap<T, i64>>, iterable: Option<Vec<T>>) -> Self {
        let mut counts: HashMap<T, i64> = HashMap::from([]);
        if let Some(source) = source {
            for key in source.keys().cloned().collect::<Vec<_>>() {
                let value: Option<i64> = source.get(&key).copied();
                if let Some(value) = value {
                    counts.insert(key, value);
                }
            }
        }
        if let Some(iterable) = iterable {
            for item in iterable.iter().cloned() {
                let value2: Option<i64> = counts.get(&item).copied();
                if let Some(value2) = value2 {
                    counts.insert(item, value2 + (1 as i64));
                } else {
                    counts.insert(item, 1 as i64);
                }
            }
        }
        return Self { counts: counts };
    }
    fn get(&self, key: &T, default: i64) -> i64 {
        let val: Option<i64> = self.counts.get(&key).copied();
        if let Some(val) = val {
            return val;
        }
        return default;
    }
    fn increment(&mut self, key: &T) {
        let val: Option<i64> = self.counts.get(&key).copied();
        if let Some(val) = val {
            self.counts.insert(key.clone(), val + (1 as i64));
        } else {
            self.counts.insert(key.clone(), 1 as i64);
        }
    }
    fn total(&self) -> i64 {
        let mut total: i64 = 0 as i64;
        for count in self.counts.clone().values().cloned().collect::<Vec<_>>() {
            total = total + count;
        }
        return total;
    }
    fn most_common(&self, n: Option<i64>) -> Vec<(T, i64)> {
        let mut result: Vec<(T, i64)> = vec![];
        for key in self.counts.clone().keys().cloned().collect::<Vec<_>>() {
            let count: Option<i64> = self.counts.get(&key).copied();
            if let Some(count) = count {
                let entry: (T, i64) = (key, count);
                result.push(entry);
            }
        }
        let sz: i64 = result.len() as i64;
        let mut i: i64 = 0 as i64;
        while i < sz {
            let mut j: i64 = i + (1 as i64);
            while j < sz {
                let left: Option<(T, i64)> = {
                    let __sifr_index_list = &result;
                    let __sifr_index_i = i;
                    let __sifr_index_norm = if __sifr_index_i < 0 {
                        ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                    } else {
                        __sifr_index_i as usize
                    };
                    __sifr_index_list.get(__sifr_index_norm).cloned()
                };
                let right: Option<(T, i64)> = {
                    let __sifr_index_list = &result;
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
                        if (right).1 > (left).1 {
                            {
                                let __idx_raw = i;
                                let __idx_norm = if __idx_raw < 0 {
                                    (result.len() as i64) + __idx_raw
                                } else {
                                    __idx_raw
                                };
                                if __idx_norm >= 0 {
                                    if let Some(__elem) = result.get_mut(__idx_norm as usize) {
                                        *__elem = right;
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
                                        *__elem = left;
                                    }
                                }
                            }
                        }
                    }
                }
                j = j + (1 as i64);
            }
            i = i + (1 as i64);
        }
        let Some(n) = n else {
            return result;
        };
        if n <= (0 as i64) {
            return vec![];
        }
        let mut top: Vec<(T, i64)> = vec![];
        let mut index: i64 = 0 as i64;
        while index < n {
            if index >= (result.len() as i64) {
                return top;
            }
            let value: Option<(T, i64)> = Some(result[index as usize].clone());
            if let Some(value) = value {
                top.push(value);
            }
            index = index + (1 as i64);
        }
        return top;
    }
    fn keys(&self) -> Vec<T> {
        return self.counts.clone().keys().cloned().collect::<Vec<_>>();
    }
    fn items(&self) -> Vec<(T, i64)> {
        let mut result: Vec<(T, i64)> = vec![];
        for key in self.counts.clone().keys().cloned().collect::<Vec<_>>() {
            let value: Option<i64> = self.counts.get(&key).copied();
            if let Some(value) = value {
                let entry: (T, i64) = (key, value);
                result.push(entry);
            }
        }
        return result;
    }
    fn values(&self) -> Vec<i64> {
        return self.counts.clone().values().cloned().collect::<Vec<_>>();
    }
    fn copy(&self) -> Counter<T> {
        return Counter::new(Some(self.counts.clone()), None);
    }
    fn clear(&mut self) {
        self.counts = HashMap::from([]);
    }
    fn update(&mut self, other: &Counter<T>) {
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
    fn subtract(&mut self, other: &Counter<T>) {
        for key in other.counts.keys().cloned().collect::<Vec<_>>() {
            let other_val: Option<i64> = other.counts.get(&key).copied();
            if let Some(other_val) = other_val {
                let existing: Option<i64> = self.counts.get(&key).copied();
                if let Some(existing) = existing {
                    self.counts.insert(key, existing - other_val);
                } else {
                    self.counts.insert(key, (0 as i64) - other_val);
                }
            }
        }
    }
    fn elements(&self) -> Vec<T> {
        let mut result: Vec<T> = vec![];
        let all_keys: Vec<T> = self.counts.clone().keys().cloned().collect::<Vec<_>>();
        let mut ki: i64 = 0 as i64;
        while ki < (all_keys.len() as i64) {
            let key_opt: Option<T> = Some(all_keys[ki as usize].clone());
            if let Some(key_opt) = key_opt {
                let cnt: Option<i64> = self.counts.get(&key_opt).copied();
                if let Some(cnt) = cnt {
                    let mut i: i64 = 0 as i64;
                    while i < cnt {
                        let key_copy: Option<T> = Some(all_keys[ki as usize].clone());
                        if let Some(key_copy) = key_copy {
                            result.push(key_copy.clone());
                        }
                        i = i + (1 as i64);
                    }
                }
            }
            ki = ki + (1 as i64);
        }
        return result;
    }
}
impl<T: Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq> std::ops::Add<&Counter<T>>
    for &Counter<T>
{
    type Output = Counter<T>;
    fn add(self, other: &Counter<T>) -> Self::Output {
        let mut new_counts: HashMap<T, i64> = HashMap::from([]);
        for key in Box::new((self.counts.clone().keys().cloned().collect::<Vec<_>>()).into_iter()) {
            let a_val: Option<i64> = self.counts.get(&key).copied();
            if let Some(a_val) = a_val {
                let b_val: Option<i64> = other.counts.get(&key).copied();
                let mut b_count: i64 = 0 as i64;
                if let Some(b_val) = b_val {
                    b_count = b_val;
                }
                let total: i64 = a_val + b_count;
                if total > (0 as i64) {
                    new_counts.insert(key, total);
                }
            }
        }
        for key2 in Box::new((other.counts.keys().cloned().collect::<Vec<_>>()).into_iter()) {
            let already: Option<i64> = new_counts.get(&key2).copied();
            if already == None {
                let b_val2: Option<i64> = other.counts.get(&key2).copied();
                if let Some(b_val2) = b_val2 {
                    if b_val2 > (0 as i64) {
                        new_counts.insert(key2, b_val2);
                    }
                }
            }
        }
        return Counter::new(Some(new_counts), None);
    }
}
impl<T: Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq> std::ops::Sub<&Counter<T>>
    for &Counter<T>
{
    type Output = Counter<T>;
    fn sub(self, other: &Counter<T>) -> Self::Output {
        let mut new_counts: HashMap<T, i64> = HashMap::from([]);
        for key in Box::new((self.counts.clone().keys().cloned().collect::<Vec<_>>()).into_iter()) {
            let a_val: Option<i64> = self.counts.get(&key).copied();
            if let Some(a_val) = a_val {
                let b_val: Option<i64> = other.counts.get(&key).copied();
                let mut b_count: i64 = 0 as i64;
                if let Some(b_val) = b_val {
                    b_count = b_val;
                }
                let diff: i64 = a_val - b_count;
                if diff > (0 as i64) {
                    new_counts.insert(key, diff);
                }
            }
        }
        return Counter::new(Some(new_counts), None);
    }
}
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
fn from_list<T: Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq + 'static>(
    items: &Vec<T>,
) -> Counter<T> {
    let mut counts: HashMap<T, i64> = HashMap::from([]);
    for item in items.iter().cloned() {
        let val: Option<i64> = counts.get(&item).copied();
        if let Some(val) = val {
            counts.insert(item, val + (1 as i64));
        } else {
            counts.insert(item, 1 as i64);
        }
    }
    return Counter::new(Some(counts), None);
}

// --- stdlib: sifr.bisect ---
fn bisect_right<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    a: &Vec<T>,
    x: &T,
    lo: i64,
    hi: Option<i64>,
) -> i64 {
    let mut left: i64 = lo;
    if left < (0 as i64) {
        left = 0 as i64;
    }
    let mut right: i64 = a.len() as i64;
    if hi.is_none() {
        right = a.len() as i64;
    } else {
        if let Some(hi) = hi {
            if hi < (0 as i64) {
                right = 0 as i64;
            } else {
                if hi > (a.len() as i64) {
                    right = a.len() as i64;
                } else {
                    right = hi;
                }
            }
        }
    }
    while left < right {
        let mid: i64 = (left + right) / (2 as i64);
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
                left = mid + (1 as i64);
            }
        } else {
            left = mid + (1 as i64);
        }
    }
    return left;
}
fn bisect<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    a: &Vec<T>,
    x: &T,
    lo: i64,
    hi: Option<i64>,
) -> i64 {
    return bisect_right(a, x, lo, hi);
}
fn insort_right<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    a: &mut Vec<T>,
    x: &T,
    lo: i64,
    hi: Option<i64>,
) {
    let pos: i64 = bisect_right(a, x, lo, hi);
    a.insert(pos as usize, x.clone());
}
fn insort<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    a: &mut Vec<T>,
    x: &T,
    lo: i64,
    hi: Option<i64>,
) {
    insort_right(a, x, lo, hi);
}

// --- stdlib: sifr.heapq ---
fn _sift_down<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: &mut Vec<T>,
    mut pos: i64,
    n: i64,
) {
    let mut done: bool = false;
    while !done {
        let mut smallest: i64 = pos;
        let left: i64 = ((2 as i64) * pos) + (1 as i64);
        let right: i64 = ((2 as i64) * pos) + (2 as i64);
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
                                *__elem = tmp_sm;
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
                                *__elem = tmp_pos;
                            }
                        }
                    }
                }
            }
            pos = smallest;
        }
    }
}
fn _sift_up<T: Clone + std::fmt::Display + PartialOrd + 'static>(heap: &mut Vec<T>, mut pos: i64) {
    let mut done: bool = false;
    while !done {
        if pos <= (0 as i64) {
            done = true;
        } else {
            let parent: i64 = (pos - (1 as i64)) / (2 as i64);
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
                                    *__elem = c_val;
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
                                    *__elem = p_val;
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
fn heapify<T: Clone + std::fmt::Display + PartialOrd + 'static>(data: &mut Vec<T>) {
    "Convert list to a min-heap in-place. O(n) time.".to_string();
    let n: i64 = data.len() as i64;
    let mut i: i64 = (n / (2 as i64)) - (1 as i64);
    while i >= (0 as i64) {
        _sift_down(data, i, n);
        i = i - (1 as i64);
    }
}
fn heappush<T: Clone + std::fmt::Display + PartialOrd + 'static>(heap: &mut Vec<T>, item: &T) {
    "Push item onto the heap in-place. O(log n) time.".to_string();
    heap.push(item.clone());
    let pos: i64 = (heap.len() as i64) - (1 as i64);
    _sift_up(heap, pos);
}
fn heappop<T: Clone + std::fmt::Display + PartialOrd + 'static>(heap: &mut Vec<T>) -> Option<T> {
    "Pop and return the smallest item. Heap is modified in-place. O(log n) time.\n    Returns None if the heap is empty."
        .to_string();
    let n: i64 = heap.len() as i64;
    if n == (0 as i64) {
        return None;
    }
    let top: Option<T> = {
        let __sifr_index_list = &heap;
        let __sifr_index_i = 0 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    let last: Option<T> = {
        let __sifr_index_list = &heap;
        let __sifr_index_i = n - (1 as i64);
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    heap.pop();
    let n2: i64 = heap.len() as i64;
    if n2 > (0 as i64) {
        if let Some(last) = last {
            {
                let __idx_raw = 0 as i64;
                let __idx_norm = if __idx_raw < 0 {
                    (heap.len() as i64) + __idx_raw
                } else {
                    __idx_raw
                };
                if __idx_norm >= 0 {
                    if let Some(__elem) = heap.get_mut(__idx_norm as usize) {
                        *__elem = last;
                    }
                }
            }
        }
        _sift_down(heap, 0 as i64, n2);
    }
    return top;
}
fn heapreplace<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
    item: T,
) -> Option<T> {
    if (heap.len() as i64) == (0 as i64) {
        return None;
    }
    let top: Option<T> = Some(heap[(0 as i64) as usize].clone());
    {
        let __idx_raw = 0 as i64;
        let __idx_norm = if __idx_raw < 0 {
            (heap.len() as i64) + __idx_raw
        } else {
            __idx_raw
        };
        if __idx_norm >= 0 {
            if let Some(__elem) = heap.get_mut(__idx_norm as usize) {
                *__elem = item;
            }
        }
    }
    _sift_down(heap, 0 as i64, heap.len() as i64);
    return top;
}
fn heappushpop<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
    item: &T,
) -> Option<T> {
    heappush(heap, item);
    return heappop(heap);
}

fn main() {
    let mut counts = from_list(&vec![
        "delta".to_string(),
        "alpha".to_string(),
        "delta".to_string(),
        "beta".to_string(),
    ]);
    println!("{:?}", counts.most_common(None));
    let mut queue = deque::new(Some(vec![1 as i64, 2 as i64, 3 as i64]), Some(4 as i64));
    queue.rotate(1 as i64);
    queue.appendleft(&(0 as i64));
    println!("{:?}", queue.to_list());
    let mut ordered: Vec<i64> = vec![1 as i64, 3 as i64, 5 as i64];
    insort(&mut ordered, &(4 as i64), 0 as i64, None);
    println!("{}", bisect(&ordered, &(4 as i64), 0 as i64, None));
    let mut heap: Vec<i64> = vec![1 as i64, 3 as i64, 5 as i64];
    heapify(&mut heap);
    println!(
        "{}",
        (heappushpop(&mut heap, &(2 as i64)))
            .map_or("None".to_string().to_string(), |__v| format!("{}", __v))
    );
    println!(
        "{}",
        (heapreplace(&mut heap, 4 as i64))
            .map_or("None".to_string().to_string(), |__v| format!("{}", __v))
    );
}
