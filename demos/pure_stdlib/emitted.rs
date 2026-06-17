use std::collections::HashMap;

// --- stdlib: sifr.statistics ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct StatisticsError {
    message: String,
}
impl StatisticsError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}
impl std::fmt::Display for StatisticsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.message);
    }
}
impl std::error::Error for StatisticsError {}
fn _sum(data: &Vec<f64>) -> f64 {
    let mut total: f64 = 0.0 as f64;
    for val in data.iter().copied() {
        total = total + val;
    }
    return total;
}
fn mean(data: &Vec<f64>) -> Result<f64, StatisticsError> {
    let count: i64 = data.len() as i64;
    if count == (0 as i64) {
        return Err(
            StatisticsError::new("mean requires at least one data point".to_string()),
        );
    }
    let total: f64 = _sum(data);
    return Ok(total / (count as f64));
}
fn median(data: &Vec<f64>) -> Result<f64, StatisticsError> {
    let n: i64 = data.len() as i64;
    if n == (0 as i64) {
        return Err(
            StatisticsError::new("median requires at least one data point".to_string()),
        );
    }
    let sorted_data: Vec<f64> = {
        let mut __sifr_sorted_v = (data).iter().copied().collect::<Vec<_>>();
        __sifr_sorted_v.sort_by(f64::total_cmp);
        __sifr_sorted_v
    };
    let mid: i64 = n / (2 as i64);
    if (n % (2 as i64)) == (0 as i64) {
        let a: Option<f64> = {
            let __sifr_index_list = &sorted_data;
            let __sifr_index_i = mid - (1 as i64);
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let b: Option<f64> = {
            let __sifr_index_list = &sorted_data;
            let __sifr_index_i = mid;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(a) = a {
            if let Some(b) = b {
                return Ok((a + b) / (2.0 as f64));
            }
        }
        return Err(StatisticsError::new("median: index error".to_string()));
    } else {
        let val: Option<f64> = {
            let __sifr_index_list = &sorted_data;
            let __sifr_index_i = mid;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(val) = val {
            return Ok(val);
        }
        return Err(StatisticsError::new("median: index error".to_string()));
    }
}
fn variance(data: &Vec<f64>) -> Result<f64, StatisticsError> {
    let n: i64 = data.len() as i64;
    if n < (2 as i64) {
        return Err(
            StatisticsError::new(
                "variance requires at least two data points".to_string(),
            ),
        );
    }
    let avg: f64 = _sum(data) / (n as f64);
    let mut total: f64 = 0.0 as f64;
    for val in data.iter().copied() {
        let diff: f64 = val - avg;
        total = total + (diff * diff);
    }
    return Ok(total / ((n - (1 as i64)) as f64));
}
fn stdev(data: &Vec<f64>) -> Result<f64, StatisticsError> {
    let n: i64 = data.len() as i64;
    if n < (2 as i64) {
        return Err(
            StatisticsError::new("stdev requires at least two data points".to_string()),
        );
    }
    let avg: f64 = _sum(data) / (n as f64);
    let mut total: f64 = 0.0 as f64;
    for val in data.iter().copied() {
        let diff: f64 = val - avg;
        total = total + (diff * diff);
    }
    let v: f64 = total / ((n - (1 as i64)) as f64);
    return Ok((v).sqrt());
}
fn mode(data: &Vec<i64>) -> Result<i64, StatisticsError> {
    if (data.len() as i64) == (0 as i64) {
        return Err(
            StatisticsError::new("mode requires at least one data point".to_string()),
        );
    }
    let mut counts: HashMap<i64, i64> = HashMap::from([]);
    for val in data.iter().copied() {
        let existing: Option<i64> = counts.get(&val).copied();
        if let Some(existing) = existing {
            counts.insert(val, existing + (1 as i64));
        } else {
            counts.insert(val, 1 as i64);
        }
    }
    let mut best: i64 = 0 as i64;
    let mut best_set: bool = false;
    let mut best_count: i64 = 0 as i64;
    for val2 in data.iter().copied() {
        let count2: Option<i64> = counts.get(&val2).copied();
        let mut count2_val: i64 = 0 as i64;
        if let Some(count2) = count2 {
            count2_val = count2;
        }
        if count2_val > best_count {
            best_count = count2_val;
            best = val2;
            best_set = true;
        }
    }
    if best_set {
        return Ok(best);
    }
    return Err(StatisticsError::new("mode: no mode found".to_string()));
}
fn multimode(data: &Vec<i64>) -> Result<Vec<i64>, StatisticsError> {
    if (data.len() as i64) == (0 as i64) {
        return Err(
            StatisticsError::new(
                "multimode requires at least one data point".to_string(),
            ),
        );
    }
    let mut counts: HashMap<i64, i64> = HashMap::from([]);
    for val in data.iter().copied() {
        let existing: Option<i64> = counts.get(&val).copied();
        if let Some(existing) = existing {
            counts.insert(val, existing + (1 as i64));
        } else {
            counts.insert(val, 1 as i64);
        }
    }
    let mut max_count: i64 = 0 as i64;
    for val2 in data.iter().copied() {
        let count2: Option<i64> = counts.get(&val2).copied();
        let mut count2_val: i64 = 0 as i64;
        if let Some(count2) = count2 {
            count2_val = count2;
        }
        if count2_val > max_count {
            max_count = count2_val;
        }
    }
    let mut result: Vec<i64> = vec![];
    let mut seen: HashMap<i64, bool> = HashMap::from([]);
    for val3 in data.iter().copied() {
        let already_opt: Option<bool> = seen.get(&val3).copied();
        let mut already: bool = false;
        if let Some(already_opt) = already_opt {
            already = already_opt;
        }
        if !already {
            let count3: Option<i64> = counts.get(&val3).copied();
            let mut count3_val: i64 = 0 as i64;
            if let Some(count3) = count3 {
                count3_val = count3;
            }
            if count3_val == max_count {
                result.push(val3);
            }
            seen.insert(val3, true);
        }
    }
    return Ok(result);
}
fn quantiles(data: &Vec<f64>, n: i64) -> Result<Vec<f64>, StatisticsError> {
    if (data.len() as i64) < (2 as i64) {
        return Err(
            StatisticsError::new(
                "quantiles requires at least two data points".to_string(),
            ),
        );
    }
    if n < (1 as i64) {
        return Err(StatisticsError::new("quantiles: n must be at least 1".to_string()));
    }
    let sorted_data: Vec<f64> = {
        let mut __sifr_sorted_v = (data).iter().copied().collect::<Vec<_>>();
        __sifr_sorted_v.sort_by(f64::total_cmp);
        __sifr_sorted_v
    };
    let m: i64 = sorted_data.len() as i64;
    let mut result: Vec<f64> = vec![];
    let mut i: i64 = 1 as i64;
    while i < n {
        let idx_f: f64 = ((i as f64) * (m as f64)) / (n as f64);
        let mut idx: i64 = idx_f as i64;
        let frac: f64 = idx_f - (idx as f64);
        if idx >= m {
            idx = m - (1 as i64);
        }
        if idx < (0 as i64) {
            idx = 0 as i64;
        }
        let lo: Option<f64> = {
            let __sifr_index_list = &sorted_data;
            let __sifr_index_i = idx;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let mut lo_val: f64 = 0.0 as f64;
        if let Some(lo) = lo {
            lo_val = lo;
        }
        if frac > (0.0 as f64) {
            let hi_idx: i64 = idx + (1 as i64);
            if hi_idx < m {
                let hi: Option<f64> = {
                    let __sifr_index_list = &sorted_data;
                    let __sifr_index_i = hi_idx;
                    let __sifr_index_norm = if __sifr_index_i < 0 {
                        ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                    } else {
                        __sifr_index_i as usize
                    };
                    __sifr_index_list.get(__sifr_index_norm).copied()
                };
                if let Some(hi) = hi {
                    lo_val = lo_val + (frac * (hi - lo_val));
                }
            }
        }
        result.push(lo_val);
        i = i + (1 as i64);
    }
    return Ok(result);
}
fn covariance(x: &Vec<f64>, y: &Vec<f64>) -> Result<f64, StatisticsError> {
    let n: i64 = x.len() as i64;
    if n < (2 as i64) {
        return Err(
            StatisticsError::new(
                "covariance requires at least two data points".to_string(),
            ),
        );
    }
    if (y.len() as i64) != n {
        return Err(
            StatisticsError::new(
                "covariance: x and y must have the same length".to_string(),
            ),
        );
    }
    let mx: f64 = _sum(x) / (n as f64);
    let my: f64 = _sum(y) / (n as f64);
    let mut total: f64 = 0.0 as f64;
    let mut i: i64 = 0 as i64;
    while i < n {
        let xi: Option<f64> = {
            let __sifr_index_list = &x;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let yi: Option<f64> = {
            let __sifr_index_list = &y;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(xi) = xi {
            if let Some(yi) = yi {
                total = total + ((xi - mx) * (yi - my));
            }
        }
        i = i + (1 as i64);
    }
    return Ok(total / ((n - (1 as i64)) as f64));
}
fn correlation(x: &Vec<f64>, y: &Vec<f64>) -> Result<f64, StatisticsError> {
    let n: i64 = x.len() as i64;
    if n < (2 as i64) {
        return Err(
            StatisticsError::new(
                "correlation requires at least two data points".to_string(),
            ),
        );
    }
    if (y.len() as i64) != n {
        return Err(
            StatisticsError::new(
                "correlation: x and y must have the same length".to_string(),
            ),
        );
    }
    let mx: f64 = _sum(x) / (n as f64);
    let my: f64 = _sum(y) / (n as f64);
    let mut cov_num: f64 = 0.0 as f64;
    let mut sx_num: f64 = 0.0 as f64;
    let mut sy_num: f64 = 0.0 as f64;
    let mut i: i64 = 0 as i64;
    while i < n {
        let xi: Option<f64> = {
            let __sifr_index_list = &x;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let yi: Option<f64> = {
            let __sifr_index_list = &y;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(xi) = xi {
            if let Some(yi) = yi {
                cov_num = cov_num + ((xi - mx) * (yi - my));
                sx_num = sx_num + ((xi - mx) * (xi - mx));
                sy_num = sy_num + ((yi - my) * (yi - my));
            }
        }
        i = i + (1 as i64);
    }
    let sx: f64 = (sx_num / ((n - (1 as i64)) as f64)).sqrt();
    let sy: f64 = (sy_num / ((n - (1 as i64)) as f64)).sqrt();
    if sx == (0.0 as f64) {
        return Err(StatisticsError::new("correlation: x has zero variance".to_string()));
    }
    if sy == (0.0 as f64) {
        return Err(StatisticsError::new("correlation: y has zero variance".to_string()));
    }
    return Ok((cov_num / ((n - (1 as i64)) as f64)) / (sx * sy));
}
fn linear_regression(x: &Vec<f64>, y: &Vec<f64>) -> Result<Vec<f64>, StatisticsError> {
    let n: i64 = x.len() as i64;
    if n < (2 as i64) {
        return Err(
            StatisticsError::new(
                "linear_regression requires at least two data points".to_string(),
            ),
        );
    }
    if (y.len() as i64) != n {
        return Err(
            StatisticsError::new(
                "linear_regression: x and y must have the same length".to_string(),
            ),
        );
    }
    let mx: f64 = _sum(x) / (n as f64);
    let my: f64 = _sum(y) / (n as f64);
    let mut num: f64 = 0.0 as f64;
    let mut den: f64 = 0.0 as f64;
    let mut i: i64 = 0 as i64;
    while i < n {
        let xi: Option<f64> = {
            let __sifr_index_list = &x;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let yi: Option<f64> = {
            let __sifr_index_list = &y;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(xi) = xi {
            if let Some(yi) = yi {
                num = num + ((xi - mx) * (yi - my));
                den = den + ((xi - mx) * (xi - mx));
            }
        }
        i = i + (1 as i64);
    }
    if den == (0.0 as f64) {
        return Err(
            StatisticsError::new("linear_regression: x has zero variance".to_string()),
        );
    }
    let slope: f64 = num / den;
    let intercept: f64 = my - (slope * mx);
    let mut result: Vec<f64> = vec![];
    result.push(slope);
    result.push(intercept);
    return Ok(result);
}

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
fn bytes_from_ints(values: &Vec<i64>) -> Result<Vec<u8>, ValueError> {
    return {
        let __vals = values;
        let mut __out = Vec::new();
        for __pair in __vals.iter().enumerate() {
            if (*__pair.1 < 0) || (*__pair.1 > 255) {
                return Err(ValueError {
                    message: format!(
                        "byte out of range at index {}: {}", __pair.0, * __pair.1
                    ),
                });
            }
            __out.push(*__pair.1 as u8);
        }
        Ok(__out)
    };
}
fn bytes_with_size(size: i64) -> Result<Vec<u8>, ValueError> {
    return {
        let __size = size;
        if __size < 0 {
            return Err(ValueError {
                message: "bytes(size) requires a non-negative size"
                    .to_string()
                    .to_string(),
            });
        }
        Ok((0..__size).map(|_| 0 as u8).collect::<Vec<u8>>())
    };
}
fn encode_utf8_result(s: &String) -> Result<Vec<u8>, ParseError> {
    return Ok({
        let __s = s;
        __s.as_bytes().to_vec()
    });
}
fn count_byte(data: &Vec<u8>, value: i64) -> i64 {
    let mut count: i64 = 0 as i64;
    for b in data.iter().map(|__byte| *__byte as i64) {
        if b == value {
            count = count + (1 as i64);
        }
    }
    return count;
}
fn find_byte(data: &Vec<u8>, value: i64) -> Option<i64> {
    let mut idx: i64 = 0 as i64;
    for b in data.iter().map(|__byte| *__byte as i64) {
        if b == value {
            return Some(idx);
        }
        idx = idx + (1 as i64);
    }
    return None;
}
fn starts_with(data: &Vec<u8>, prefix: &Vec<u8>) -> bool {
    if (prefix.len() as i64) > (data.len() as i64) {
        return false;
    }
    let mut i: i64 = 0 as i64;
    while i < (prefix.len() as i64) {
        let a: Option<i64> = data.get(i as usize).map(|__byte| *__byte as i64);
        let b: Option<i64> = prefix.get(i as usize).map(|__byte| *__byte as i64);
        let Some(a) = a else {
            return false;
        };
        let Some(b) = b else {
            return false;
        };
        if a != b {
            return false;
        }
        i = i + (1 as i64);
    }
    return true;
}
fn ends_with(data: &Vec<u8>, suffix: &Vec<u8>) -> bool {
    if (suffix.len() as i64) > (data.len() as i64) {
        return false;
    }
    let offset: i64 = (data.len() as i64) - (suffix.len() as i64);
    let mut i: i64 = 0 as i64;
    while i < (suffix.len() as i64) {
        let a: Option<i64> = data
            .get((offset + i) as usize)
            .map(|__byte| *__byte as i64);
        let b: Option<i64> = suffix.get(i as usize).map(|__byte| *__byte as i64);
        let Some(a) = a else {
            return false;
        };
        let Some(b) = b else {
            return false;
        };
        if a != b {
            return false;
        }
        i = i + (1 as i64);
    }
    return true;
}

// --- stdlib: sifr.random ---
#[derive(Debug, Clone)]
struct __SifrRandomModuleState {
    words: Vec<i64>,
    index: i64,
    gauss_next: Option<f64>,
}
static __SIFR_RANDOM_MODULE_STATE: std::sync::LazyLock<
    std::sync::Mutex<__SifrRandomModuleState>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(__SifrRandomModuleState {
    words: Vec::new(),
    index: 0,
    gauss_next: None,
}));
const _MT_N: i64 = 624 as i64;
const _MT_M: i64 = 397 as i64;
const _MT_MATRIX_A: i64 = 2567483615 as i64;
const _MT_UPPER_MASK: i64 = 2147483648 as i64;
const _MT_LOWER_MASK: i64 = 2147483647 as i64;
const _MT_F: i64 = 1812433253 as i64;
const _MT_WORD_MASK: i64 = 4294967295 as i64;
#[derive(Debug, Clone, PartialEq)]
struct RandomState {
    version: i64,
    state_words: Vec<i64>,
    index: i64,
    gauss_next: Option<f64>,
}
impl RandomState {
    fn new(
        version: i64,
        state_words: Vec<i64>,
        index: i64,
        gauss_next: Option<f64>,
    ) -> Self {
        return Self {
            version: version,
            state_words: state_words,
            index: index,
            gauss_next: gauss_next,
        };
    }
}
#[derive(Debug, Clone, PartialEq)]
struct Random {
    _state_words: Vec<i64>,
    _index: i64,
    _gauss_next: Option<f64>,
}
impl Random {
    fn new(seed_value: Option<i64>) -> Self {
        let normalized_seed: i64 = _normalize_seed_input(seed_value);
        return Self {
            _state_words: _seed_words_from_seed(normalized_seed),
            _index: _MT_N,
            _gauss_next: None,
        };
    }
    fn seed(&mut self, seed_value: Option<i64>) {
        let normalized_seed: i64 = _normalize_seed_input(seed_value);
        self._state_words = _seed_words_from_seed(normalized_seed);
        self._index = _MT_N;
        self._gauss_next = None;
    }
    fn _twist(&mut self) {
        let mut i: i64 = 0 as i64;
        while i < _MT_N {
            let y: i64 = (_state_word_at(&self._state_words.clone(), i) & _MT_UPPER_MASK)
                + (_state_word_at(&self._state_words.clone(), (i + (1 as i64)) % _MT_N)
                    & _MT_LOWER_MASK);
            let mut x_a: i64 = y >> (1 as i64);
            if (y % (2 as i64)) != (0 as i64) {
                x_a = x_a ^ _MT_MATRIX_A;
            }
            let new_word: i64 = _state_word_at(
                &self._state_words.clone(),
                (i + _MT_M) % _MT_N,
            ) ^ x_a;
            {
                let __idx_raw = i;
                let __idx_norm = if __idx_raw < 0 {
                    (self._state_words.len() as i64) + __idx_raw
                } else {
                    __idx_raw
                };
                if __idx_norm >= 0 {
                    if let Some(__elem) = self._state_words.get_mut(__idx_norm as usize)
                    {
                        *__elem = new_word & _MT_WORD_MASK;
                    }
                }
            }
            i = i + (1 as i64);
        }
        self._index = 0 as i64;
    }
    fn _next_u32(&mut self) -> i64 {
        if self._index >= _MT_N {
            self._twist();
        }
        let mut y: i64 = _state_word_at(&self._state_words.clone(), self._index);
        self._index = self._index + (1 as i64);
        y = y ^ (y >> (11 as i64));
        y = y ^ ((y << (7 as i64)) & (2636928640 as i64));
        y = y ^ ((y << (15 as i64)) & (4022730752 as i64));
        y = y ^ (y >> (18 as i64));
        return y & _MT_WORD_MASK;
    }
    fn random(&mut self) -> f64 {
        return (self._next_u32() as f64) / (4294967296.0 as f64);
    }
    fn uniform(&mut self, minimum: f64, maximum: f64) -> f64 {
        return minimum + ((maximum - minimum) * self.random());
    }
    fn randrange(
        &mut self,
        start: i64,
        stop: Option<i64>,
        step: i64,
    ) -> Result<i64, ValueError> {
        if step == (0 as i64) {
            return Err(ValueError::new("randrange: step must not be zero".to_string()));
        }
        let mut actual_start: i64 = start;
        let mut actual_stop: i64 = start;
        if stop.is_none() {
            actual_start = 0 as i64;
        } else {
            if let Some(stop) = stop {
                actual_stop = stop;
            }
        }
        let width: i64 = actual_stop - actual_start;
        if step > (0 as i64) {
            if width <= (0 as i64) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
        } else {
            if width >= (0 as i64) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
        }
        let mut abs_width: i64 = width;
        if abs_width < (0 as i64) {
            abs_width = (0 as i64) - abs_width;
        }
        let mut abs_step: i64 = step;
        if abs_step < (0 as i64) {
            abs_step = (0 as i64) - abs_step;
        }
        let count: i64 = ((abs_width + abs_step) - (1 as i64)) / abs_step;
        if count <= (0 as i64) {
            return Err(ValueError::new("randrange: empty range".to_string()));
        }
        let pick: i64 = self._next_u32() % count;
        return Ok(actual_start + (pick * step));
    }
    fn randint(&mut self, minimum: i64, maximum: i64) -> Result<i64, ValueError> {
        if minimum > maximum {
            return Err(ValueError::new("randint: min must be <= max".to_string()));
        }
        return self.randrange(minimum, Some(maximum + (1 as i64)), 1 as i64);
    }
    fn getrandbits(&mut self, k: i64) -> Result<i64, ValueError> {
        if k < (0 as i64) {
            return Err(
                ValueError::new("getrandbits: number of bits must be >= 0".to_string()),
            );
        }
        let mut result: i64 = 0 as i64;
        let mut bits_left: i64 = k;
        while bits_left > (0 as i64) {
            let word: i64 = self._next_u32();
            let mut take: i64 = 32 as i64;
            if bits_left < (32 as i64) {
                take = bits_left;
            }
            let mask: i64 = ((1 as i64) << take) - (1 as i64);
            result = (result << take) | (word & mask);
            bits_left = bits_left - take;
        }
        return Ok(result);
    }
    fn randbytes(&mut self, n: i64) -> Result<Vec<u8>, ValueError> {
        if n < (0 as i64) {
            return Err(ValueError::new("randbytes: n must be >= 0".to_string()));
        }
        let mut values: Vec<i64> = vec![];
        let mut i: i64 = 0 as i64;
        while i < n {
            let byte_value: i64 = self._next_u32() & (255 as i64);
            values.push(byte_value);
            i = i + (1 as i64);
        }
        return {
            let __vals = values;
            let mut __out = Vec::new();
            for __pair in __vals.iter().enumerate() {
                if (*__pair.1 < 0) || (*__pair.1 > 255) {
                    return Err(ValueError {
                        message: format!(
                            "byte out of range at index {}: {}", __pair.0, * __pair.1
                        ),
                    });
                }
                __out.push(*__pair.1 as u8);
            }
            Ok(__out)
        };
    }
    fn gauss(&mut self, mu: f64, sigma: f64) -> f64 {
        let cached: Option<f64> = self._gauss_next;
        if let Some(cached) = cached {
            self._gauss_next = None;
            return mu + (sigma * cached);
        }
        let mut u1: f64 = self.random();
        if u1 <= (0.0 as f64) {
            u1 = 0.000000000001 as f64;
        }
        let u2: f64 = self.random();
        let radius: f64 = (-(2.0 as f64) * (u1).ln()).sqrt();
        let theta: f64 = ((2.0 as f64) * std::f64::consts::PI) * u2;
        let z0: f64 = radius * (theta).cos();
        let z1: f64 = radius * (theta).sin();
        let next_cached: Option<f64> = Some(z1);
        self._gauss_next = next_cached;
        return mu + (sigma * z0);
    }
    fn getstate(&self) -> RandomState {
        return RandomState::new(
            3 as i64,
            _clone_words(&self._state_words.clone()),
            self._index,
            self._gauss_next,
        );
    }
    fn setstate(&mut self, state: &RandomState) -> Result<(), ValueError> {
        if state.version != (3 as i64) {
            return Err(ValueError::new("setstate: unsupported version".to_string()));
        }
        if (state.state_words.len() as i64) != _MT_N {
            return Err(
                ValueError::new("setstate: state_words must have length 624".to_string()),
            );
        }
        if ((state.index < (0 as i64)) || (state.index > _MT_N)) {
            return Err(
                ValueError::new("setstate: index must be in range [0, 624]".to_string()),
            );
        }
        let mut normalized: Vec<i64> = vec![];
        for word in state.state_words.iter().copied() {
            if (word < (0 as i64)) || (word > _MT_WORD_MASK) {
                return Err(ValueError::new("setstate: word out of range".to_string()));
            }
            normalized.push(word & _MT_WORD_MASK);
        }
        self._state_words = normalized;
        self._index = state.index;
        self._gauss_next = state.gauss_next;
        return Ok(());
    }
}
fn _state_word_at(words: &Vec<i64>, index: i64) -> i64 {
    let value: Option<i64> = {
        let __sifr_index_list = &words;
        let __sifr_index_i = index;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    if let Some(value) = value {
        return value;
    }
    return 0 as i64;
}
fn _clone_words(words: &Vec<i64>) -> Vec<i64> {
    let mut copied: Vec<i64> = vec![];
    for word in words.iter().copied() {
        copied.push(word);
    }
    return copied;
}
fn _normalize_seed_input(seed_value: Option<i64>) -> i64 {
    if let Some(seed_value) = seed_value {
        return seed_value;
    }
    return (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64() * (1000000.0 as f64)) as i64;
}
fn _seed_words_from_seed(seed_value: i64) -> Vec<i64> {
    let mut words: Vec<i64> = vec![];
    words.push(seed_value & _MT_WORD_MASK);
    let mut i: i64 = 1 as i64;
    while i < _MT_N {
        let prev: i64 = _state_word_at(&words, i - (1 as i64));
        let next_word: i64 = ((_MT_F * (prev ^ (prev >> (30 as i64)))) + i)
            & _MT_WORD_MASK;
        words.push(next_word);
        i = i + (1 as i64);
    }
    return words;
}
fn _build_state_from_module_storage() -> RandomState {
    return RandomState::new(
        3 as i64,
        {
            let __state = __SIFR_RANDOM_MODULE_STATE
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            __state.words.clone()
        },
        {
            let __state = __SIFR_RANDOM_MODULE_STATE
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            __state.index
        },
        {
            let __state = __SIFR_RANDOM_MODULE_STATE
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            __state.gauss_next.clone()
        },
    );
}
fn _store_state_into_module_storage(state: &RandomState) {
    let _set_result: Result<(), ValueError> = {
        let __words = _clone_words(&state.state_words);
        let __index = state.index;
        let __gauss_next = state.gauss_next;
        if (__index < 0) || (__index > 624) {
            Err(ValueError {
                message: "random module state index must be in range [0, 624]"
                    .to_string(),
            })
        } else {
            if __words.len() != 624 {
                Err(ValueError {
                    message: "random module state words must have length 624".to_string(),
                })
            } else {
                {
                    let mut __state = __SIFR_RANDOM_MODULE_STATE
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner());
                    __state.words = __words;
                    __state.index = __index;
                    __state.gauss_next = __gauss_next;
                    Ok(())
                }
            }
        }
    };
    let _: Result<(), ValueError> = _set_result;
}
fn _ensure_module_state_initialized() {
    let words: Vec<i64> = {
        let __state = __SIFR_RANDOM_MODULE_STATE
            .lock()
            .unwrap_or_else(|__err| __err.into_inner());
        __state.words.clone()
    };
    if (words.len() as i64) == _MT_N {
        return;
    }
    let mut bootstrap: Random = Random::new(Some(5489 as i64));
    _store_state_into_module_storage(&bootstrap.getstate());
}
fn _module_random() -> Random {
    _ensure_module_state_initialized();
    let mut r: Random = Random::new(Some(0 as i64));
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let _set_result: Result<(), ValueError> = r
            .setstate(&_build_state_from_module_storage());
        let _: Result<(), ValueError> = _set_result;
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = e.message;
    }
    return r;
}
fn _sync_module_random(generator: &mut Random) {
    _store_state_into_module_storage(&generator.getstate());
}
fn randrange(start: i64, stop: Option<i64>, step: i64) -> Result<i64, ValueError> {
    let mut generator: Random = _module_random();
    let value: Result<i64, ValueError> = generator.randrange(start, stop, step);
    _sync_module_random(&mut generator);
    return value;
}
fn gauss(mu: f64, sigma: f64) -> f64 {
    let mut generator: Random = _module_random();
    let value: f64 = generator.gauss(mu, sigma);
    _sync_module_random(&mut generator);
    return value;
}
fn sample<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
    k: i64,
) -> Result<Vec<T>, ValueError> {
    if k < (0 as i64) {
        return Err(ValueError::new("sample: k must be >= 0".to_string()));
    }
    if k > (items.len() as i64) {
        return Err(ValueError::new("sample larger than population".to_string()));
    }
    let mut pool: Vec<T> = vec![];
    for item in items.iter().cloned() {
        pool.push(item.clone());
    }
    let mut generator: Random = _module_random();
    let mut result: Vec<T> = vec![];
    let mut remaining: i64 = pool.len() as i64;
    let mut i: i64 = 0 as i64;
    while i < k {
        let pick_index: i64 = generator._next_u32() % remaining;
        let picked: Option<T> = {
            let __sifr_index_list = &pool;
            let __sifr_index_i = pick_index;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(picked) = picked {
            result.push(picked.clone());
        }
        let last: Option<T> = {
            let __sifr_index_list = &pool;
            let __sifr_index_i = remaining - (1 as i64);
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(last) = last {
            {
                let __idx_raw = pick_index;
                let __idx_norm = if __idx_raw < 0 {
                    (pool.len() as i64) + __idx_raw
                } else {
                    __idx_raw
                };
                if __idx_norm >= 0 {
                    if let Some(__elem) = pool.get_mut(__idx_norm as usize) {
                        *__elem = last;
                    }
                }
            }
        }
        remaining = remaining - (1 as i64);
        i = i + (1 as i64);
    }
    _sync_module_random(&mut generator);
    return Ok(result);
}
fn shuffle<T: Clone + std::fmt::Display + PartialOrd + 'static>(items: &mut Vec<T>) {
    let mut generator: Random = _module_random();
    let n: i64 = items.len() as i64;
    if n > (1 as i64) {
        let mut i: i64 = n - (1 as i64);
        while i > (0 as i64) {
            let j: i64 = generator._next_u32() % (i + (1 as i64));
            let left: Option<T> = Some(items[i as usize].clone());
            let right: Option<T> = {
                let __sifr_index_list = &items;
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
                    {
                        let __idx_raw = i;
                        let __idx_norm = if __idx_raw < 0 {
                            (items.len() as i64) + __idx_raw
                        } else {
                            __idx_raw
                        };
                        if __idx_norm >= 0 {
                            if let Some(__elem) = items.get_mut(__idx_norm as usize) {
                                *__elem = right;
                            }
                        }
                    }
                    {
                        let __idx_raw = j;
                        let __idx_norm = if __idx_raw < 0 {
                            (items.len() as i64) + __idx_raw
                        } else {
                            __idx_raw
                        };
                        if __idx_norm >= 0 {
                            if let Some(__elem) = items.get_mut(__idx_norm as usize) {
                                *__elem = left;
                            }
                        }
                    }
                }
            }
            i = i - (1 as i64);
        }
    }
    _sync_module_random(&mut generator);
}

// --- stdlib: sifr.functools ---
fn reduce<
    T: Clone + std::fmt::Display + PartialOrd + 'static,
    U: Clone + std::fmt::Display + PartialOrd + 'static,
>(func: impl Fn(&U, &T) -> U, data: &Vec<T>, initial: &U) -> U {
    let mut result: U = (initial).clone();
    for val in data.iter().cloned() {
        result = func(&result, &val);
    }
    return result;
}

// --- stdlib: sifr.itertools ---
fn _compress_impl<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    selectors: &Vec<bool>,
) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    let mut i: i64 = 0 as i64;
    while i < (data.len() as i64) {
        if i >= (selectors.len() as i64) {
            i = data.len() as i64;
        } else {
            let sel: Option<bool> = {
                let __sifr_index_list = &selectors;
                let __sifr_index_i = i;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).copied()
            };
            let val: Option<T> = Some(data[i as usize].clone());
            if let Some(sel) = sel {
                if let Some(val) = val {
                    if sel {
                        result.push(val.clone());
                    }
                }
            }
            i = i + (1 as i64);
        }
    }
    return result;
}
fn _takewhile_impl<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool,
    data: &Vec<T>,
) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    let mut i: i64 = 0 as i64;
    while i < (data.len() as i64) {
        let val: Option<T> = Some(data[i as usize].clone());
        if let Some(val) = val {
            if pred(&val) {
                result.push(val.clone());
            } else {
                i = data.len() as i64;
            }
        }
        i = i + (1 as i64);
    }
    return result;
}
fn _zip_longest_impl<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    a: &Vec<T>,
    b: &Vec<T>,
    fill: &T,
) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = vec![];
    let len_a: i64 = a.len() as i64;
    let len_b: i64 = b.len() as i64;
    let mut max_len: i64 = len_a;
    if len_b > max_len {
        max_len = len_b;
    }
    let mut i: i64 = 0 as i64;
    while i < max_len {
        let mut pair: Vec<T> = vec![];
        if i < len_a {
            let va: Option<T> = {
                let __sifr_index_list = &a;
                let __sifr_index_i = i;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(va) = va {
                pair.push(va.clone());
            } else {
                pair.push(fill.clone());
            }
        } else {
            pair.push(fill.clone());
        }
        if i < len_b {
            let vb: Option<T> = {
                let __sifr_index_list = &b;
                let __sifr_index_i = i;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(vb) = vb {
                pair.push(vb.clone());
            } else {
                pair.push(fill.clone());
            }
        } else {
            pair.push(fill.clone());
        }
        result.push(pair);
        i = i + (1 as i64);
    }
    return result;
}
fn _collect_iterable<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: Vec<T>,
) -> Vec<T> {
    let mut collected: Vec<T> = vec![];
    for item in data.iter().cloned() {
        collected.push(item.clone());
    }
    return collected;
}
fn count(start: i64, step: i64) -> Box<dyn Iterator<Item = i64>> {
    return count_from(start, step, 10000 as i64);
}
fn accumulate<
    T: Clone + std::fmt::Display + PartialOrd + 'static + std::ops::Add<Output = T>,
>(data: &Vec<T>, initial: Option<T>) -> Box<dyn Iterator<Item = T>> {
    let mut result: Vec<T> = vec![];
    if let Some(initial) = initial {
        result.push(initial.clone());
    }
    for item in data.iter().cloned() {
        if (result.len() as i64) == (0 as i64) {
            result.push(item.clone());
        } else {
            let prev: Option<T> = {
                let __sifr_index_list = &result;
                let __sifr_index_i = (result.len() as i64) - (1 as i64);
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(prev) = prev {
                let next_val: T = prev + item;
                result.push(next_val.clone());
            }
        }
    }
    return Box::new((result).iter().cloned());
}
fn compress<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    selectors: &Vec<bool>,
) -> Box<dyn Iterator<Item = T>> {
    let data_owned: Vec<T> = _collect_iterable(
        ((data).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let mut selectors_owned: Vec<bool> = vec![];
    for selector in selectors.iter().copied() {
        selectors_owned.push(selector);
    }
    let result: Vec<T> = _compress_impl(&data_owned, &selectors_owned);
    return Box::new((result).iter().cloned());
}
fn dropwhile<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool,
    data: &Vec<T>,
) -> Box<dyn Iterator<Item = T>> {
    let mut result: Vec<T> = vec![];
    let mut dropping: bool = true;
    for val in data.iter().cloned() {
        if dropping {
            if !(pred(&val)) {
                dropping = false;
                result.push(val.clone());
            }
        } else {
            result.push(val.clone());
        }
    }
    return Box::new((result).iter().cloned());
}
fn takewhile<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool,
    data: &Vec<T>,
) -> Box<dyn Iterator<Item = T>> {
    let data_owned: Vec<T> = _collect_iterable(
        ((data).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let result: Vec<T> = _takewhile_impl(pred, &data_owned);
    return Box::new((result).iter().cloned());
}
fn filterfalse<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool,
    data: &Vec<T>,
) -> Box<dyn Iterator<Item = T>> {
    let mut result: Vec<T> = vec![];
    for val in data.iter().cloned() {
        if !(pred(&val)) {
            result.push(val.clone());
        }
    }
    return Box::new((result).iter().cloned());
}
fn zip_longest<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    a: &Vec<T>,
    b: &Vec<T>,
    fill: &T,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    let a_owned: Vec<T> = _collect_iterable(
        ((a).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let b_owned: Vec<T> = _collect_iterable(
        ((b).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let result: Vec<Vec<T>> = _zip_longest_impl(&a_owned, &b_owned, fill);
    return Box::new((result).iter().cloned());
}
fn count_from(start: i64, step: i64, n: i64) -> Box<dyn Iterator<Item = i64>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: std::vec::IntoIter<i64> = Vec::new().into_iter();
    return Box::new(
        std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<i64> = Vec::new();
                let mut i: i64 = 0 as i64;
                let mut current: i64 = start;
                while i < n {
                    _yields.push(current);
                    current = current + step;
                    i = i + (1 as i64);
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            return __sifr_generator_iter.next();
        }),
    );
}
fn cycle<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    n: i64,
) -> Box<dyn Iterator<Item = T>> {
    let materialized: Vec<T> = _collect_iterable(
        ((data).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let mut result: Vec<T> = vec![];
    if (materialized.len() as i64) > (0 as i64) {
        let mut i: i64 = 0 as i64;
        while i < n {
            let idx: i64 = i % (materialized.len() as i64);
            let val: Option<T> = {
                let __sifr_index_list = &materialized;
                let __sifr_index_i = idx;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(val) = val {
                result.push(val.clone());
            }
            i = i + (1 as i64);
        }
    }
    return Box::new((result).iter().cloned());
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
struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for Error {
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

fn add(a: i64, b: i64) -> i64 {
    return a + b;
}

fn mul(a: i64, b: i64) -> i64 {
    return a * b;
}

fn less_than_three(x: i64) -> bool {
    return x < (3 as i64);
}

fn main() {
    println!("=== math additions ===");
    println!("{}", format!("{}{}", "acosh(1.0) = ".to_string(), format!("{}", (1.0 as f64).acosh())));
    println!("{}", format!("{}{}", "asinh(0.0) = ".to_string(), format!("{}", (0.0 as f64).asinh())));
    println!("{}", format!("{}{}", "atanh(0.0) = ".to_string(), format!("{}", (0.0 as f64).atanh())));
    println!("{}", format!("{}{}", "isqrt(17) = ".to_string(), format!("{}", ((17 as i64) as f64).sqrt() as i64)));
    let p: Vec<f64> = vec![0.0 as f64, 0.0 as f64];
    let q: Vec<f64> = vec![3.0 as f64, 4.0 as f64];
    println!("{}", format!("{}{}", "dist([0,0],[3,4]) = ".to_string(), format!("{}", {
    let __p = &p;
    let __q = &q;
    if __p.len() != __q.len() { f64::NAN } else { if __p.is_empty() { 0.0 } else { {
    let mut __scale: f64 = 0.0;
    let mut __ssq: f64 = 1.0;
    for __i in 0..__p.len() {
        let __d: f64 = (__p[__i] - __q[__i]).abs();
        if __d != 0.0 {
            if __scale < __d {
                let __r: f64 = __scale / __d;
                __ssq = 1.0 + ((__ssq * __r) * __r);
                __scale = __d;
            } else {
                let __r: f64 = __d / __scale;
                __ssq += __r * __r;
            }
        }
    }
    if __scale == 0.0 { 0.0 } else { __scale * __ssq.sqrt() }
} } }
})));
    let data_fsum: Vec<f64> = vec![0.1 as f64, 0.1 as f64, 0.1 as f64, 0.1 as f64, 0.1 as f64, 0.1 as f64, 0.1 as f64, 0.1 as f64, 0.1 as f64, 0.1 as f64];
    println!("{}", format!("{}{}", "fsum(10x0.1) = ".to_string(), format!("{}", {
    let __data = &data_fsum;
    let mut __sum: f64 = 0.0;
    let mut __comp: f64 = 0.0;
    let mut __pos_inf: bool = false;
    let mut __neg_inf: bool = false;
    let mut __has_nan: bool = false;
    for __x in __data.iter() {
        let __v: f64 = *__x;
        if __v.is_nan() {
            __has_nan = true;
            continue;
        }
        if __v.is_infinite() {
            if __v.is_sign_positive() {
                __pos_inf = true;
            } else {
                __neg_inf = true;
            }
            continue;
        }
        let __t: f64 = __sum + __v;
        if __sum.abs() >= __v.abs() {
            __comp += (__sum - __t) + __v;
        } else {
            __comp += (__v - __t) + __sum;
        }
        __sum = __t;
    }
    if __has_nan || (__pos_inf && __neg_inf) { f64::NAN } else { if __pos_inf { f64::INFINITY } else { if __neg_inf { f64::NEG_INFINITY } else { __sum + __comp } } }
})));
    println!("=== statistics (Result[float, StatisticsError]) ===");
    let data: Vec<f64> = vec![1.0 as f64, 2.0 as f64, 3.0 as f64, 4.0 as f64, 5.0 as f64];
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
    let m: f64 = mean(&data)?;
    println!("{}", format!("{}{}", "mean = ".to_string(), format!("{}", m)));
    let med: f64 = median(&data)?;
    println!("{}", format!("{}{}", "median = ".to_string(), format!("{}", med)));
    let v: f64 = variance(&data)?;
    println!("{}", format!("{}{}", "variance = ".to_string(), format!("{}", v)));
    let s: f64 = stdev(&data)?;
    println!("{}", format!("{}{}", "stdev = ".to_string(), format!("{}", s)));
    let idata: Vec<i64> = vec![1 as i64, 2 as i64, 2 as i64, 3 as i64, 3 as i64, 3 as i64];
    let mo: i64 = mode(&idata)?;
    println!("{}", format!("{}{}", "mode = ".to_string(), format!("{}", mo)));
    let mm: Vec<i64> = multimode(&vec![1 as i64, 2 as i64, 2 as i64, 3 as i64, 3 as i64])?;
    println!("{}", format!("{}{}", "multimode len = ".to_string(), format!("{}", mm.len() as i64)));
    let qs: Vec<f64> = quantiles(&data, 4 as i64)?;
    println!("{}", format!("{}{}", "quartiles count = ".to_string(), format!("{}", qs.len() as i64)));
    let x: Vec<f64> = vec![1.0 as f64, 2.0 as f64, 3.0 as f64, 4.0 as f64, 5.0 as f64];
    let y: Vec<f64> = vec![2.0 as f64, 4.0 as f64, 6.0 as f64, 8.0 as f64, 10.0 as f64];
    let cov: f64 = covariance(&x, &y)?;
    println!("{}", format!("{}{}", "covariance = ".to_string(), format!("{}", cov)));
    let r: f64 = correlation(&x, &y)?;
    println!("{}", format!("{}{}", "correlation = ".to_string(), format!("{}", r)));
    let lr: Vec<f64> = linear_regression(&x, &y)?;
    let slope: Option<f64> = {
    let __sifr_index_list = &lr;
    let __sifr_index_i = 0 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
    let intercept: Option<f64> = {
    let __sifr_index_list = &lr;
    let __sifr_index_i = 1 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
    if let Some(slope) = slope {
        println!("{}", format!("{}{}", "slope = ".to_string(), format!("{}", slope)));
    }
    if let Some(intercept) = intercept {
        println!("{}", format!("{}{}", "intercept = ".to_string(), format!("{}", intercept)));
    }
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "error: ".to_string(), e.message));
    }
    let __sifr_try_res: Result<(), StatisticsError> = (|| {
    let empty: Vec<f64> = vec![];
    let bad: f64 = mean(&empty)?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "empty mean error: ".to_string(), e.message));
    }
    println!("=== random additions ===");
    let mut items: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64, 5 as i64];
    shuffle(&mut items);
    println!("{}", format!("{}{}", "shuffle len = ".to_string(), format!("{}", items.len() as i64)));
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let s3: Vec<i64> = sample(&items, 3 as i64)?;
    println!("{}", format!("{}{}", "sample(3) len = ".to_string(), format!("{}", s3.len() as i64)));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "sample error: ".to_string(), e.message));
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let rr: i64 = randrange(0 as i64, Some(100 as i64), 5 as i64)?;
    println!("{}", format!("{}{}", "randrange in range = ".to_string(), format!("{}", rr >= (0 as i64))));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "randrange error: ".to_string(), e.message));
    }
    let g: f64 = gauss(0.0 as f64, 1.0 as f64);
    println!("gauss sample is float = True");
    println!("=== functools.reduce ===");
    let nums: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64, 5 as i64];
    let total: i64 = reduce(|__arg0, __arg1| add((__arg0).clone(), (__arg1).clone()), &nums, &(0 as i64));
    println!("{}", format!("{}{}", "reduce(add) = ".to_string(), format!("{}", total)));
    let product: i64 = reduce(|__arg0, __arg1| mul((__arg0).clone(), (__arg1).clone()), &nums, &(1 as i64));
    println!("{}", format!("{}{}", "reduce(mul) = ".to_string(), format!("{}", product)));
    println!("=== itertools additions ===");
    let idata2: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64, 5 as i64];
    let acc: Vec<i64> = accumulate(&(idata2).iter().copied().collect::<Vec<_>>(), None).collect::<Vec<_>>();
    println!("{}", format!("{}{}", "accumulate = ".to_string(), format!("{:?}", acc)));
    let sel: Vec<bool> = vec![true, false, true, false, true];
    let comp: Vec<i64> = compress(&(idata2).iter().copied().collect::<Vec<_>>(), &(sel).iter().copied().collect::<Vec<_>>()).collect::<Vec<_>>();
    println!("{}", format!("{}{}", "compress = ".to_string(), format!("{:?}", comp)));
    let dw: Vec<i64> = dropwhile(|__arg0| less_than_three((__arg0).clone()), &(idata2).iter().copied().collect::<Vec<_>>()).collect::<Vec<_>>();
    println!("{}", format!("{}{}", "dropwhile(<3) = ".to_string(), format!("{:?}", dw)));
    let tw: Vec<i64> = takewhile(|__arg0| less_than_three((__arg0).clone()), &(idata2).iter().copied().collect::<Vec<_>>()).collect::<Vec<_>>();
    println!("{}", format!("{}{}", "takewhile(<3) = ".to_string(), format!("{:?}", tw)));
    let ff: Vec<i64> = filterfalse(|__arg0| less_than_three((__arg0).clone()), &(idata2).iter().copied().collect::<Vec<_>>()).collect::<Vec<_>>();
    println!("{}", format!("{}{}", "filterfalse(<3) = ".to_string(), format!("{:?}", ff)));
    let a: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64];
    let b: Vec<i64> = vec![4 as i64, 5 as i64];
    let zl: Vec<Vec<i64>> = zip_longest(&(a).iter().copied().collect::<Vec<_>>(), &(b).iter().copied().collect::<Vec<_>>(), &(0 as i64)).collect::<Vec<_>>();
    println!("{}", format!("{}{}", "zip_longest len = ".to_string(), format!("{}", zl.len() as i64)));
    let cf: Vec<i64> = count_from(0 as i64, 2 as i64, 5 as i64).collect::<Vec<_>>();
    println!("{}", format!("{}{}", "count_from(0,2,5) = ".to_string(), format!("{:?}", cf)));
    let mut ctr: Box<dyn Iterator<Item = i64>> = count(0 as i64, 2 as i64);
    let c0: Option<i64> = ctr.next();
    let mut c1: Option<i64> = ctr.next();
    let c2: Option<i64> = ctr.next();
    let mut c3: Option<i64> = ctr.next();
    let c4: Option<i64> = ctr.next();
    println!("{}", format!("{}{}", "count(0,2) first 5 = ".to_string(), format!("{:?}", vec![c0, c1, c2, c3, c4])));
    let cyc: Vec<i64> = cycle(&(vec![1 as i64, 2 as i64, 3 as i64]).into_iter().collect::<Vec<_>>(), 7 as i64).collect::<Vec<_>>();
    println!("{}", format!("{}{}", "cycle([1,2,3], 7) = ".to_string(), format!("{:?}", cyc)));
    println!("=== Counter enhancements ===");
    let mut c1 = from_list(&vec!["a".to_string(), "b".to_string(), "a".to_string(), "c".to_string()]);
    let c2 = from_list(&vec!["b".to_string(), "c".to_string(), "d".to_string()]);
    c1.update(&c2);
    println!("{}", format!("{}{}{}{}", "after update: a=".to_string(), format!("{}", c1.get(&"a".to_string(), 0 as i64)), " b=".to_string(), format!("{}", c1.get(&"b".to_string(), 0 as i64))));
    let mut c3 = from_list(&vec!["x".to_string(), "x".to_string(), "y".to_string()]);
    let c4 = from_list(&vec!["x".to_string()]);
    c3.subtract(&c4);
    println!("{}", format!("{}{}", "after subtract: x=".to_string(), format!("{}", c3.get(&"x".to_string(), 0 as i64))));
    let mut c5 = from_list(&vec!["a".to_string(), "a".to_string(), "b".to_string()]);
    let elems: Vec<String> = c5.elements();
    println!("{}", format!("{}{}", "elements len = ".to_string(), format!("{}", elems.len() as i64)));
    let mut cc = from_list(&vec!["a".to_string(), "b".to_string()]);
    cc.update(&from_list(&vec!["b".to_string(), "c".to_string()]));
    println!("{}", format!("{}{}", "counter_add b = ".to_string(), format!("{}", cc.get(&"b".to_string(), 0 as i64))));
    let mut cd = from_list(&vec!["a".to_string(), "a".to_string(), "b".to_string()]);
    cd.subtract(&from_list(&vec!["a".to_string()]));
    println!("{}", format!("{}{}", "counter_sub a = ".to_string(), format!("{}", cd.get(&"a".to_string(), 0 as i64))));
    println!("=== stdlib_pure_expansion: all features demonstrated ===");
}
