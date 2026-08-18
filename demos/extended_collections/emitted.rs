// src/main.rs
use ::std::collections::HashMap;

// --- stdlib: _sifr.collections ---
fn _new_set_impl() -> Vec<i64> {
    ::sifr_stdlib::collections::new_set()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn _set_from_list_impl(items: Vec<i64>) -> Vec<i64> {
    ::sifr_stdlib::collections::set_from_list(
            items
                .into_iter()
                .map(::sifr_runtime::interop::SifrIntBridge::from)
                .collect::<Vec<_>>(),
        )
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn _set_add_impl(s: Vec<i64>, item: i64) -> Vec<i64> {
    ::sifr_stdlib::collections::set_add(
            s
                .into_iter()
                .map(::sifr_runtime::interop::SifrIntBridge::from)
                .collect::<Vec<_>>(),
            ::sifr_runtime::interop::SifrIntBridge::from(item),
        )
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn _set_contains_impl(s: &Vec<i64>, item: i64) -> bool {
    ::sifr_stdlib::collections::set_contains(
        &s
            .iter()
            .copied()
            .map(::sifr_runtime::interop::SifrIntBridge::from)
            .collect::<Vec<_>>(),
        ::sifr_runtime::interop::SifrIntBridge::from(item),
    )
}
fn _set_remove_impl(s: Vec<i64>, item: i64) -> Vec<i64> {
    ::sifr_stdlib::collections::set_remove(
            s
                .into_iter()
                .map(::sifr_runtime::interop::SifrIntBridge::from)
                .collect::<Vec<_>>(),
            ::sifr_runtime::interop::SifrIntBridge::from(item),
        )
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn _set_len_impl(s: &Vec<i64>) -> i64 {
    ::sifr_stdlib::collections::set_len(
            &s
                .iter()
                .copied()
                .map(::sifr_runtime::interop::SifrIntBridge::from)
                .collect::<Vec<_>>(),
        )
        .to_i64_saturating()
}
fn _set_union_impl(a: Vec<i64>, b: Vec<i64>) -> Vec<i64> {
    ::sifr_stdlib::collections::set_union(
            a
                .into_iter()
                .map(::sifr_runtime::interop::SifrIntBridge::from)
                .collect::<Vec<_>>(),
            b
                .into_iter()
                .map(::sifr_runtime::interop::SifrIntBridge::from)
                .collect::<Vec<_>>(),
        )
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn _set_intersection_impl(a: Vec<i64>, b: Vec<i64>) -> Vec<i64> {
    ::sifr_stdlib::collections::set_intersection(
            a
                .into_iter()
                .map(::sifr_runtime::interop::SifrIntBridge::from)
                .collect::<Vec<_>>(),
            b
                .into_iter()
                .map(::sifr_runtime::interop::SifrIntBridge::from)
                .collect::<Vec<_>>(),
        )
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}

// --- stdlib: sifr.collections ---
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2ecollections_x2eCounter<T: std::hash::Hash + Eq> {
    counts: HashMap<T, i64>,
}
impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    fn new(source: Option<HashMap<T, i64>>, iterable: Option<Vec<T>>) -> Self {
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
    fn __iter__(&self) -> Vec<T> {
        Box::new((self.counts.keys().cloned().collect::<Vec<_>>()).into_iter())
            .collect::<Vec<_>>()
    }
}
impl<T: ::std::hash::Hash + Eq> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    fn __getitem__(&self, key: &T) -> i64 {
        let val: Option<i64> = self.counts.get(&key).copied();
        if let Some(val) = val {
            return val;
        }
        0_i64
    }
}
impl<T: ::std::hash::Hash + Eq> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    fn get(&self, key: &T, default: i64) -> i64 {
        let val: Option<i64> = self.counts.get(&key).copied();
        if let Some(val) = val {
            return val;
        }
        default
    }
}
impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    fn increment(&mut self, key: &T) {
        let val: Option<i64> = self.counts.get(&key).copied();
        if let Some(val) = val {
            self.counts.insert(key.clone(), val + (1_i64));
        } else {
            self.counts.insert(key.clone(), 1_i64);
        }
    }
}
impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    fn total(&self) -> i64 {
        let mut total: i64 = 0_i64;
        for count in self.counts.values().cloned().collect::<Vec<_>>() {
            total += count;
        }
        total
    }
}
impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    fn most_common(&self, n: Option<i64>) -> Vec<(T, i64)> {
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
    fn keys(&self) -> Vec<T> {
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
    fn items(&self) -> Vec<(T, i64)> {
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
    fn values(&self) -> Vec<i64> {
        self.counts.values().cloned().collect::<Vec<_>>()
    }
}
impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    fn copy(&self) -> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
        __SifrStdlib_sifr_x2ecollections_x2eCounter::new(Some(self.counts.clone()), None)
    }
}
impl<T: ::std::hash::Hash + Eq> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    fn clear(&mut self) {
        self.counts = HashMap::from([]);
    }
}
impl<T: ::std::hash::Hash + Eq + Clone> __SifrStdlib_sifr_x2ecollections_x2eCounter<T> {
    fn update(&mut self, other: &__SifrStdlib_sifr_x2ecollections_x2eCounter<T>) {
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
    fn subtract(&mut self, other: &__SifrStdlib_sifr_x2ecollections_x2eCounter<T>) {
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
    fn elements(&self) -> Vec<T> {
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
fn _copy_int_list(items: &Vec<i64>) -> Vec<i64> {
    let mut copied: Vec<i64> = vec![];
    for item in items.iter().copied() {
        copied.push(item);
    }
    copied
}
fn set_from_list(items: &Vec<i64>) -> Vec<i64> {
    _set_from_list_impl(_copy_int_list(items))
}
fn set_add(s: &Vec<i64>, item: i64) -> Vec<i64> {
    _set_add_impl(_copy_int_list(s), item)
}
fn set_contains(s: &Vec<i64>, item: i64) -> bool {
    _set_contains_impl(s, item)
}
fn set_len(s: &Vec<i64>) -> i64 {
    _set_len_impl(s)
}
fn set_union(a: &Vec<i64>, b: &Vec<i64>) -> Vec<i64> {
    _set_union_impl(_copy_int_list(a), _copy_int_list(b))
}
fn set_intersection(a: &Vec<i64>, b: &Vec<i64>) -> Vec<i64> {
    _set_intersection_impl(_copy_int_list(a), _copy_int_list(b))
}
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
// --- end stdlib ---

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IOError {
    message: String,
    kind: String,
}

impl IOError {
    fn new(message: String) -> Self {
        Self { message, kind: "Other".to_string() }
    }
}

impl ::std::fmt::Display for IOError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for IOError {
}

fn __io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
    let msg = e.to_string();
    let kind = {
    let __sifr_io_kind = (&e as &dyn ::std::any::Any).downcast_ref::<std::io::Error>().map(::std::io::Error::kind);
    match __sifr_io_kind {
    Some(::std::io::ErrorKind::NotFound) => {
        "FileNotFound".to_string()
    },
    Some(::std::io::ErrorKind::PermissionDenied) => {
        "PermissionDenied".to_string()
    },
    Some(::std::io::ErrorKind::AlreadyExists) => {
        "FileExists".to_string()
    },
    Some(::std::io::ErrorKind::IsADirectory) => {
        "IsADirectory".to_string()
    },
    Some(::std::io::ErrorKind::NotADirectory) => {
        "NotADirectory".to_string()
    },
    Some(::std::io::ErrorKind::DirectoryNotEmpty) => {
        "DirectoryNotEmpty".to_string()
    },
    _ => {
        "Other".to_string()
    },
}
};
    IOError { message: msg, kind }
}

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

impl ::std::error::Error for ParseError {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl ::std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for ValueError {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct JSONDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl JSONDecodeError {
    fn new(message: String) -> Self {
        Self { message, line: 0, column: 0 }
    }
}

impl ::std::fmt::Display for JSONDecodeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for JSONDecodeError {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct JsonIntegerRangeError {
    message: String,
    path: String,
    profile: String,
}

impl JsonIntegerRangeError {
    fn new(message: String) -> Self {
        Self { message, path: String::new(), profile: String::new() }
    }
}

impl ::std::fmt::Display for JsonIntegerRangeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for JsonIntegerRangeError {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct JsonLimitError {
    message: String,
    limit: i64,
}

impl JsonLimitError {
    fn new(message: String) -> Self {
        Self { message, limit: 0 }
    }
}

impl ::std::fmt::Display for JsonLimitError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for JsonLimitError {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TOMLDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl TOMLDecodeError {
    fn new(message: String) -> Self {
        Self { message, line: 0, column: 0 }
    }
}

impl ::std::fmt::Display for TOMLDecodeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for TOMLDecodeError {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RegexError {
    message: String,
    detail: String,
}

impl RegexError {
    fn new(message: String) -> Self {
        Self { message, detail: String::new() }
    }
}

impl ::std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for RegexError {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TimeoutError {
    message: String,
}

impl TimeoutError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl ::std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for TimeoutError {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ScopeFailure {
    message: String,
}

impl ScopeFailure {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl ::std::fmt::Display for ScopeFailure {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for ScopeFailure {
}

fn main() {
    println!("=== Set Operations ===");
    println!("Set from [1,2,2,3,3]: length = {}", set_len(&set_from_list(&vec![1_i64, 2_i64, 2_i64, 3_i64, 3_i64])));
    println!("After adding 4: length = {}", set_len(&set_add(&set_from_list(&vec![1_i64, 2_i64, 3_i64]), 4_i64)));
    println!("Contains 2: {}", set_contains(&set_from_list(&vec![1_i64, 2_i64, 3_i64]), 2_i64));
    println!("Contains 5: {}", set_contains(&set_from_list(&vec![1_i64, 2_i64, 3_i64]), 5_i64));
    println!("Union [1,2,3] | [3,4,5]: length = {}", set_len(&set_union(&set_from_list(&vec![1_i64, 2_i64, 3_i64]), &set_from_list(&vec![3_i64, 4_i64, 5_i64]))));
    println!("Intersection [1,2,3] & [3,4,5]: length = {}", set_len(&set_intersection(&set_from_list(&vec![1_i64, 2_i64, 3_i64]), &set_from_list(&vec![3_i64, 4_i64, 5_i64]))));
    println!("=== Counter ===");
    let fruits: Vec<String> = vec!["apple".to_string(), "banana".to_string(), "apple".to_string(), "cherry".to_string(), "banana".to_string(), "apple".to_string()];
    let fruit_counter: __SifrStdlib_sifr_x2ecollections_x2eCounter<String> = from_list(&fruits);
    println!("apple count: {}", fruit_counter.get(&"apple".to_string(), 0_i64));
    println!("banana count: {}", fruit_counter.get(&"banana".to_string(), 0_i64));
    println!("cherry count: {}", fruit_counter.get(&"cherry".to_string(), 0_i64));
    println!("=== Bytes ===");
    println!("\'hello\' encoded: {} bytes", vec![(104_i64) as u8, (101_i64) as u8, (108_i64) as u8, (108_i64) as u8, (111_i64) as u8].len() as i64);
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let roundtrip: String = ::sifr_runtime::encoding::decode_text(&vec![(83_i64) as u8, (105_i64) as u8, (102_i64) as u8, (114_i64) as u8], &"utf-8".to_string(), &"strict".to_string()).map_err(|__message| ParseError { message: __message })?;
    println!("Roundtrip: {}", roundtrip);
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("decode error: {}", err.message);
    }
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let hex_hello: String = {
    let __bytes_receiver = &vec![(104_i64) as u8, (101_i64) as u8, (108_i64) as u8, (108_i64) as u8, (111_i64) as u8];
    let mut __hex = String::with_capacity(__bytes_receiver.len().saturating_mul(2));
    for __byte in __bytes_receiver.iter() {
        __hex.push_str(&format!("{:02x}", *__byte));
    }
    __hex
};
    println!("\'hello\' as hex: {}", hex_hello);
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("hex encode error: {}", err.message);
    }
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let hex_bytes: Vec<u8> = ({
    let s: String = "536966".to_string().to_string();
    let mut cleaned = String::new();
    for ch in s.chars() {
        if ch.is_ascii_whitespace() {
            continue;
        }
        if !ch.is_ascii_hexdigit() {
            return Err(ParseError { message: format!("invalid hex character: {}", ch) });
        }
        cleaned.push(ch);
    }
    if (cleaned.len() % 2) != 0 {
        return Err(ParseError { message: "fromhex() arg must contain an even number of hexadecimal digits".to_string().to_string() });
    }
    let mut result = Vec::new();
    for pair in cleaned.as_bytes().chunks(2) {
        let pair_str = ::std::str::from_utf8(pair).map_err(|e| ParseError { message: e.to_string() })?;
        result.push(u8::from_str_radix(pair_str, 16).map_err(|e| ParseError { message: e.to_string() })?);
    }
    Ok::<Vec<u8>, ParseError>(result)
})?;
    let decoded: String = ::sifr_runtime::encoding::decode_text(&hex_bytes, &"utf-8".to_string(), &"strict".to_string()).map_err(|__message| ParseError { message: __message })?;
    println!("Hex \'536966\' decoded: {}", decoded);
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("hex error: {}", err.message);
    }
    println!("=== Demo complete ===");
}
