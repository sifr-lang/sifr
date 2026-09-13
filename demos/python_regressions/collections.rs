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
fn from_list<T: Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq + 'static>(
    items: &[T],
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

// --- stdlib: sifr.itertools ---
fn chain<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    iterables: &[Vec<T>],
) -> Box<dyn Iterator<Item = T>> {
    let iterables = iterables.to_vec();
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: std::vec::IntoIter<T> = Vec::new().into_iter();
    return Box::new(std::iter::from_fn(move || {
        if !__sifr_generator_initialized {
            let mut _yields: Vec<T> = Vec::new();
            for iterable in iterables.iter().cloned() {
                for item in iterable.iter().cloned() {
                    _yields.push(item);
                }
            }
            __sifr_generator_iter = _yields.into_iter();
            __sifr_generator_initialized = true;
        }
        return __sifr_generator_iter.next();
    }));
}
fn repeat<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    value: T,
    times: i64,
) -> Box<dyn Iterator<Item = T>> {
    let holder: Vec<T> = vec![value];
    let mut result: Vec<T> = vec![];
    let mut i: i64 = 0 as i64;
    while i < times {
        if (holder.len() as i64) > (0 as i64) {
            result.push(
                ({
                    let Some(__sifr_index_value) = ({
                        let __sifr_index_list = &holder;
                        let __sifr_index_i = 0 as i64;
                        let __sifr_index_norm = if __sifr_index_i < 0 {
                            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                        } else {
                            __sifr_index_i as usize
                        };
                        __sifr_index_list.get(__sifr_index_norm).cloned()
                    }) else {
                        unreachable!("compiler-verified index should be in range");
                    };
                    __sifr_index_value
                })
                .clone(),
            );
        }
        i = i + (1 as i64);
    }
    return Box::new(result.into_iter());
}
fn take<T: Clone + std::fmt::Display + PartialOrd + 'static>(n: i64, data: &[T]) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    let mut count: i64 = 0 as i64;
    for item in data.iter().cloned() {
        if count >= n {
            return result;
        }
        result.push(item.clone());
        count = count + (1 as i64);
    }
    return result;
}
fn flatten<T: Clone + std::fmt::Display + PartialOrd + 'static>(lists: &[Vec<T>]) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    for inner in lists.iter().cloned() {
        for val in inner.iter().cloned() {
            result.push(val.clone());
        }
    }
    return result;
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

// --- stdlib: sifr.bisect ---
fn bisect_left<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    a: &[T],
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
            if val < *x {
                left = mid + (1 as i64);
            } else {
                right = mid;
            }
        } else {
            left = mid + (1 as i64);
        }
    }
    return left;
}
fn bisect_right<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    a: &[T],
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
