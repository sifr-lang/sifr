use std::collections::HashMap;

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
impl<
    T: Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq,
> std::ops::Add<&Counter<T>> for &Counter<T> {
    type Output = Counter<T>;
    fn add(self, other: &Counter<T>) -> Self::Output {
        let mut new_counts: HashMap<T, i64> = HashMap::from([]);
        for key in Box::new(
            (self.counts.clone().keys().cloned().collect::<Vec<_>>()).into_iter(),
        ) {
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
        for key2 in Box::new(
            (other.counts.keys().cloned().collect::<Vec<_>>()).into_iter(),
        ) {
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
impl<
    T: Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq,
> std::ops::Sub<&Counter<T>> for &Counter<T> {
    type Output = Counter<T>;
    fn sub(self, other: &Counter<T>) -> Self::Output {
        let mut new_counts: HashMap<T, i64> = HashMap::from([]);
        for key in Box::new(
            (self.counts.clone().keys().cloned().collect::<Vec<_>>()).into_iter(),
        ) {
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

// --- stdlib: sifr.bytes ---
fn decode_utf8(data: &Vec<u8>) -> Result<String, ParseError> {
    return String::from_utf8(data.iter().copied().collect::<Vec<u8>>())
        .map_err(|e| ParseError {
            message: e.to_string(),
        });
}
fn bytes_from_hex(s: &String) -> Result<Vec<u8>, ParseError> {
    return {
        let s: String = s.to_string();
        let mut cleaned = String::new();
        for ch in s.chars() {
            if ch.is_ascii_whitespace() {
                continue;
            }
            if !ch.is_ascii_hexdigit() {
                return Err(ParseError {
                    message: format!("invalid hex character: {}", ch),
                });
            }
            cleaned.push(ch);
        }
        if (cleaned.len() % 2) != 0 {
            return Err(ParseError {
                message: "fromhex() arg must contain an even number of hexadecimal digits"
                    .to_string()
                    .to_string(),
            });
        }
        let mut result = Vec::new();
        for pair in cleaned.as_bytes().chunks(2) {
            let pair_str = std::str::from_utf8(pair)
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            result
                .push(
                    u8::from_str_radix(pair_str, 16)
                        .map_err(|e| ParseError {
                            message: e.to_string(),
                        })?,
                );
        }
        Ok(result)
    };
}

#[derive(Debug, Clone)]
struct IOError {
    message: String,
    kind: String,
}

impl IOError {
    fn new(message: String) -> Self {
        return Self { message: message, kind: "Other".to_string() };
    }
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for IOError {
}

fn __io_err(e: std::io::Error) -> IOError {
    let msg = e.to_string();
    let kind = if e.kind() == std::io::ErrorKind::NotFound { "FileNotFound".to_string() } else { if e.kind() == std::io::ErrorKind::PermissionDenied { "PermissionDenied".to_string() } else { if e.kind() == std::io::ErrorKind::AlreadyExists { "FileExists".to_string() } else { "Other".to_string() } } };
    return IOError { message: msg, kind: kind };
}

#[derive(Debug, Clone)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ParseError {
}

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ValueError {
}

#[derive(Debug, Clone)]
struct JSONDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl JSONDecodeError {
    fn new(message: String) -> Self {
        return Self { message: message, line: 0, column: 0 };
    }
}

impl std::fmt::Display for JSONDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for JSONDecodeError {
}

#[derive(Debug, Clone)]
struct TOMLDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl TOMLDecodeError {
    fn new(message: String) -> Self {
        return Self { message: message, line: 0, column: 0 };
    }
}

impl std::fmt::Display for TOMLDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for TOMLDecodeError {
}

#[derive(Debug, Clone)]
struct RegexError {
    message: String,
    detail: String,
}

impl RegexError {
    fn new(message: String) -> Self {
        return Self { message: message, detail: String::new() };
    }
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for RegexError {
}

fn main() {
    println!("=== Set Operations ===");
    println!("Set from [1,2,2,3,3]: length = {}", ({
    let __items = vec![1 as i64, 2 as i64, 2 as i64, 3 as i64, 3 as i64];
    let mut s = __items.clone();
    s.sort();
    s.dedup();
    s
}).len() as i64);
    println!("After adding 4: length = {}", ({
    let __items = {
    let __items = vec![1 as i64, 2 as i64, 3 as i64];
    let mut s = __items.clone();
    s.sort();
    s.dedup();
    s
};
    let mut s = __items.clone();
    let v = 4 as i64;
    if !s.contains(&v) {
        s.push(v);
    }
    s
}).len() as i64);
    println!("Contains 2: {}", ({
    let __items = vec![1 as i64, 2 as i64, 3 as i64];
    let mut s = __items.clone();
    s.sort();
    s.dedup();
    s
}).contains(&(2 as i64)));
    println!("Contains 5: {}", ({
    let __items = vec![1 as i64, 2 as i64, 3 as i64];
    let mut s = __items.clone();
    s.sort();
    s.dedup();
    s
}).contains(&(5 as i64)));
    println!("Union [1,2,3] | [3,4,5]: length = {}", ({
    let __left = ({
    let __items = vec![1 as i64, 2 as i64, 3 as i64];
    let mut s = __items.clone();
    s.sort();
    s.dedup();
    s
}).clone();
    let __right = ({
    let __items = vec![3 as i64, 4 as i64, 5 as i64];
    let mut s = __items.clone();
    s.sort();
    s.dedup();
    s
}).clone();
    let mut s = __left.clone();
    for v in __right.iter() {
        if !s.contains(v) {
            s.push(*v);
        }
    }
    s.sort();
    s
}).len() as i64);
    println!("Intersection [1,2,3] & [3,4,5]: length = {}", ({
    let __left = ({
    let __items = vec![1 as i64, 2 as i64, 3 as i64];
    let mut s = __items.clone();
    s.sort();
    s.dedup();
    s
}).clone();
    let __right = ({
    let __items = vec![3 as i64, 4 as i64, 5 as i64];
    let mut s = __items.clone();
    s.sort();
    s.dedup();
    s
}).clone();
    let __a = __left.clone();
    let __b = __right.clone();
    __a.iter().filter(|x| __b.contains(x)).cloned().collect::<Vec<i64>>()
}).len() as i64);
    println!("=== Counter ===");
    let fruits: Vec<String> = vec!["apple".to_string(), "banana".to_string(), "apple".to_string(), "cherry".to_string(), "banana".to_string(), "apple".to_string()];
    let mut fruit_counter = from_list(&fruits);
    println!("apple count: {}", fruit_counter.get(&"apple".to_string(), 0 as i64));
    println!("banana count: {}", fruit_counter.get(&"banana".to_string(), 0 as i64));
    println!("cherry count: {}", fruit_counter.get(&"cherry".to_string(), 0 as i64));
    println!("=== Bytes ===");
    println!("\'hello\' encoded: {} bytes", ({
    let __s = "hello".to_string();
    __s.as_bytes().to_vec()
}).len() as i64);
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let roundtrip: String = String::from_utf8(({
    let __s = "Sifr".to_string();
    __s.as_bytes().to_vec()
}).iter().copied().collect::<Vec<u8>>()).map_err(|e| ParseError { message: e.to_string() })?;
    println!("Roundtrip: {}", roundtrip);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("decode error: {}", err.message);
    }
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let hex_hello: String = Ok(({
    let __s = "hello".to_string();
    __s.as_bytes().to_vec()
}).iter().map(|__byte| format!("{:02x}", *__byte)).collect::<Vec<String>>().join(""))?;
    println!("\'hello\' as hex: {}", hex_hello);
    return Ok(());
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
        let pair_str = std::str::from_utf8(pair).map_err(|e| ParseError { message: e.to_string() })?;
        result.push(u8::from_str_radix(pair_str, 16).map_err(|e| ParseError { message: e.to_string() })?);
    }
    Ok(result)
})?;
    let decoded: String = String::from_utf8(hex_bytes.iter().copied().collect::<Vec<u8>>()).map_err(|e| ParseError { message: e.to_string() })?;
    println!("Hex \'536966\' decoded: {}", decoded);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("hex error: {}", err.message);
    }
    println!("=== Demo complete ===");
}
