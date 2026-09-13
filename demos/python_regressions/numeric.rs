// --- stdlib: sifr.datetime ---
#[derive(Debug, Clone)]
struct timedelta {
    _days: i64,
    _seconds: i64,
}
impl timedelta {
    fn new(days: i64, seconds: i64) -> Self {
        return Self {
            _days: days,
            _seconds: seconds,
        };
    }
    fn total_seconds(&self) -> i64 {
        return (self._days * (86400 as i64)) + self._seconds;
    }
    fn days(&self) -> i64 {
        return self._days;
    }
    fn seconds(&self) -> i64 {
        return self._seconds;
    }
}
impl std::ops::Add<&timedelta> for &timedelta {
    type Output = timedelta;
    fn add(self, other: &timedelta) -> Self::Output {
        let total: i64 = self.total_seconds() + other.total_seconds();
        let d: i64 = total / (86400 as i64);
        let s: i64 = total % (86400 as i64);
        return timedelta::new(d, s);
    }
}
impl std::ops::Sub<&timedelta> for &timedelta {
    type Output = timedelta;
    fn sub(self, other: &timedelta) -> Self::Output {
        let total: i64 = self.total_seconds() - other.total_seconds();
        let d: i64 = total / (86400 as i64);
        let s: i64 = total % (86400 as i64);
        return timedelta::new(d, s);
    }
}
impl PartialEq for timedelta {
    fn eq(&self, other: &timedelta) -> bool {
        return self.total_seconds() == other.total_seconds();
    }
}
impl std::fmt::Display for timedelta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "timedelta(_days={}, _seconds={})",
            self._days, self._seconds
        );
    }
}

// --- stdlib: sifr.math ---
fn factorial(n: i64) -> i64 {
    if n < (0 as i64) {
        return 0 as i64;
    }
    let mut result: i64 = 1 as i64;
    let mut i: i64 = 2 as i64;
    while i <= n {
        result = result * i;
        i = i + (1 as i64);
    }
    return result;
}
fn gcd(a: i64, b: i64) -> i64 {
    let mut x: i64 = a;
    let mut y: i64 = b;
    if x < (0 as i64) {
        x = (0 as i64) - x;
    }
    if y < (0 as i64) {
        y = (0 as i64) - y;
    }
    while y != (0 as i64) {
        let temp: i64 = y;
        y = x % y;
        x = temp;
    }
    return x;
}
fn lcm(a: i64, b: i64) -> i64 {
    if a == (0 as i64) {
        return 0 as i64;
    }
    if b == (0 as i64) {
        return 0 as i64;
    }
    let g: i64 = gcd(a, b);
    let mut x: i64 = a;
    if x < (0 as i64) {
        x = (0 as i64) - x;
    }
    let mut y: i64 = b;
    if y < (0 as i64) {
        y = (0 as i64) - y;
    }
    return (x / g) * y;
}
fn comb(n: i64, k: i64) -> i64 {
    if k < (0 as i64) {
        return 0 as i64;
    }
    if k > n {
        return 0 as i64;
    }
    if k == (0 as i64) {
        return 1 as i64;
    }
    if k == n {
        return 1 as i64;
    }
    let mut r: i64 = k;
    if r > (n - k) {
        r = n - k;
    }
    let mut result: i64 = 1 as i64;
    let mut i: i64 = 0 as i64;
    while i < r {
        result = result * (n - i);
        result = result / (i + (1 as i64));
        i = i + (1 as i64);
    }
    return result;
}
fn isclose(a: f64, b: f64, rel_tol: f64, abs_tol: f64) -> bool {
    if rel_tol < (0.0 as f64) {
        return false;
    }
    if abs_tol < (0.0 as f64) {
        return false;
    }
    if a == b {
        return true;
    }
    if (((a).is_nan()) || ((b).is_nan())) {
        return false;
    }
    if (((a).is_infinite()) || ((b).is_infinite())) {
        return false;
    }
    let mut diff: f64 = a - b;
    if diff < (0.0 as f64) {
        diff = (0.0 as f64) - diff;
    }
    let mut a_abs: f64 = a;
    if a_abs < (0.0 as f64) {
        a_abs = (0.0 as f64) - a_abs;
    }
    let mut b_abs: f64 = b;
    if b_abs < (0.0 as f64) {
        b_abs = (0.0 as f64) - b_abs;
    }
    let mut rel_bound: f64 = rel_tol * (a_abs).max(b_abs);
    if abs_tol > rel_bound {
        rel_bound = abs_tol;
    }
    return diff <= rel_bound;
}

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
fn _sum(data: &[f64]) -> f64 {
    let mut total: f64 = 0.0 as f64;
    for val in data.iter().copied() {
        total = total + val;
    }
    return total;
}
fn mean(data: &[f64]) -> Result<f64, StatisticsError> {
    let count: i64 = data.len() as i64;
    if count == (0 as i64) {
        return Err(StatisticsError::new(
            "mean requires at least one data point".to_string(),
        ));
    }
    let total: f64 = _sum(data);
    return Ok(total / (count as f64));
}
fn median(data: &[f64]) -> Result<f64, StatisticsError> {
    let n: i64 = data.len() as i64;
    if n == (0 as i64) {
        return Err(StatisticsError::new(
            "median requires at least one data point".to_string(),
        ));
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
fn stdev(data: &[f64]) -> Result<f64, StatisticsError> {
    let n: i64 = data.len() as i64;
    if n < (2 as i64) {
        return Err(StatisticsError::new(
            "stdev requires at least two data points".to_string(),
        ));
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
