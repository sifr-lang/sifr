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
fn perm(n: i64, k: i64) -> i64 {
    if k < (0 as i64) {
        return 0 as i64;
    }
    if k > n {
        return 0 as i64;
    }
    let mut result: i64 = 1 as i64;
    let mut i: i64 = 0 as i64;
    while i < k {
        result = result * (n - i);
        i = i + (1 as i64);
    }
    return result;
}
fn log_base(x: f64, base: f64) -> f64 {
    return (x).ln() / (base).ln();
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
fn prod(data: &Vec<i64>) -> i64 {
    let mut result: i64 = 1 as i64;
    for val in data.iter().copied() {
        result = result * val;
    }
    return result;
}
fn frexp_mantissa(x: f64) -> f64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x == 0.0 {
            vec![__x, 0.0]
        } else {
            if !__x.is_finite() {
                vec![__x, 0.0]
            } else {
                {
                    let __bits: u64 = __x.to_bits();
                    let __sign_mask: u64 = (1 as u64) << 63;
                    let __frac_mask: u64 = ((1 as u64) << 52) - (1 as u64);
                    let __sign: u64 = __bits & __sign_mask;
                    let __exp: i32 = ((__bits >> 52) & (2047 as u64)) as i32;
                    let __frac: u64 = __bits & __frac_mask;
                    if __exp == 0 {
                        {
                            let __scaled: f64 = __x * (2.0 as f64).powi(54);
                            let __sbits: u64 = __scaled.to_bits();
                            let __sexp: i32 = ((__sbits >> 52) & (2047 as u64)) as i32;
                            let __sfrac: u64 = __sbits & __frac_mask;
                            let __mant: f64 = f64::from_bits(
                                (__sign | ((1022 as u64) << 52)) | __sfrac,
                            );
                            let __e: i32 = (__sexp - 1022) - 54;
                            vec![__mant, __e as f64]
                        }
                    } else {
                        {
                            let __mant: f64 = f64::from_bits(
                                (__sign | ((1022 as u64) << 52)) | __frac,
                            );
                            let __e: i32 = __exp - 1022;
                            vec![__mant, __e as f64]
                        }
                    }
                }
            }
        }
    };
    let m: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(m) = m else {
        return f64::NAN;
    };
    return m;
}
fn frexp_exponent(x: f64) -> i64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x == 0.0 {
            vec![__x, 0.0]
        } else {
            if !__x.is_finite() {
                vec![__x, 0.0]
            } else {
                {
                    let __bits: u64 = __x.to_bits();
                    let __sign_mask: u64 = (1 as u64) << 63;
                    let __frac_mask: u64 = ((1 as u64) << 52) - (1 as u64);
                    let __sign: u64 = __bits & __sign_mask;
                    let __exp: i32 = ((__bits >> 52) & (2047 as u64)) as i32;
                    let __frac: u64 = __bits & __frac_mask;
                    if __exp == 0 {
                        {
                            let __scaled: f64 = __x * (2.0 as f64).powi(54);
                            let __sbits: u64 = __scaled.to_bits();
                            let __sexp: i32 = ((__sbits >> 52) & (2047 as u64)) as i32;
                            let __sfrac: u64 = __sbits & __frac_mask;
                            let __mant: f64 = f64::from_bits(
                                (__sign | ((1022 as u64) << 52)) | __sfrac,
                            );
                            let __e: i32 = (__sexp - 1022) - 54;
                            vec![__mant, __e as f64]
                        }
                    } else {
                        {
                            let __mant: f64 = f64::from_bits(
                                (__sign | ((1022 as u64) << 52)) | __frac,
                            );
                            let __e: i32 = __exp - 1022;
                            vec![__mant, __e as f64]
                        }
                    }
                }
            }
        }
    };
    let exp_val: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(exp_val) = exp_val else {
        return 0 as i64;
    };
    return (exp_val).trunc() as i64;
}
fn modf_fractional(x: f64) -> f64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x.is_nan() {
            vec![f64::NAN, f64::NAN]
        } else {
            if __x.is_infinite() {
                vec![(0.0 as f64).copysign(__x), __x]
            } else {
                {
                    let __int = __x.trunc();
                    let mut __frac = __x - __int;
                    if __frac == 0.0 {
                        __frac = (0.0 as f64).copysign(__x);
                    }
                    vec![__frac, __int]
                }
            }
        }
    };
    let f: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(f) = f else {
        return f64::NAN;
    };
    return f;
}
fn modf_integral(x: f64) -> f64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x.is_nan() {
            vec![f64::NAN, f64::NAN]
        } else {
            if __x.is_infinite() {
                vec![(0.0 as f64).copysign(__x), __x]
            } else {
                {
                    let __int = __x.trunc();
                    let mut __frac = __x - __int;
                    if __frac == 0.0 {
                        __frac = (0.0 as f64).copysign(__x);
                    }
                    vec![__frac, __int]
                }
            }
        }
    };
    let i: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(i) = i else {
        return f64::NAN;
    };
    return i;
}
fn pow(x: f64, y: f64) -> f64 {
    return (x).powf(y);
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
fn randint(minimum: i64, maximum: i64) -> Result<i64, ValueError> {
    let mut generator: Random = _module_random();
    let value: Result<i64, ValueError> = generator.randint(minimum, maximum);
    _sync_module_random(&mut generator);
    return value;
}

// --- stdlib: sifr.itertools ---
fn batched<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    n: i64,
) -> Result<Vec<Vec<T>>, ValueError> {
    if n <= (0 as i64) {
        return Err(ValueError::new("batched: n must be > 0".to_string()));
    }
    let mut result: Vec<Vec<T>> = vec![];
    let mut current_batch: Vec<T> = vec![];
    for value in data.iter().cloned() {
        current_batch.push(value.clone());
        if (current_batch.len() as i64) == n {
            result.push(current_batch);
            current_batch = vec![];
        }
    }
    if (current_batch.len() as i64) > (0 as i64) {
        result.push(current_batch);
    }
    return Ok(result);
}

// --- stdlib: sifr.secrets ---
fn randbelow(n: i64) -> Result<i64, ValueError> {
    if n <= (0 as i64) {
        return Err(ValueError::new("randbelow: n must be > 0".to_string()));
    }
    return Ok({
        let __start = 0 as i64;
        let __end = n - (1 as i64);
        __start + rand::RngExt::random_range(&mut rand::rng(), 0..(__end - __start) + 1)
    });
}

// --- stdlib: sifr.uuid ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct UUID {
    _hex: String,
}
impl UUID {
    fn new(hex_str: String) -> Self {
        return Self {
            _hex: format!("{}{}", hex_str, "".to_string()),
        };
    }
    fn hex(&self) -> String {
        let mut result: String = "".to_string();
        let mut i: i64 = 0 as i64;
        while i < (self._hex.clone().chars().count() as i64) {
            let ch: Option<String> = {
                let __sifr_index_str = &self._hex;
                let __sifr_index_i = i;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
            };
            if let Some(ch) = ch {
                if ch != "-".to_string() {
                    result = format!("{}{}", result, ch);
                }
            }
            i = i + (1 as i64);
        }
        return result;
    }
    fn urn(&self) -> String {
        return format!("{}{}", "urn:uuid:".to_string(), self._hex.clone());
    }
    fn to_str(&self) -> String {
        return format!("{}{}", self._hex.clone(), "".to_string());
    }
    fn version(&self) -> i64 {
        let marker: Option<String> = {
            let __sifr_index_str = &self._hex;
            let __sifr_index_i = 14 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let Some(marker) = marker else {
            return -(1 as i64);
        };
        return _hex_digit_value(&marker);
    }
}
impl std::fmt::Display for UUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "UUID(_hex={})", self._hex);
    }
}
fn _to_lower_hex_char(ch: &String) -> String {
    if ch.clone() == "A".to_string() {
        return "a".to_string();
    }
    if ch.clone() == "B".to_string() {
        return "b".to_string();
    }
    if ch.clone() == "C".to_string() {
        return "c".to_string();
    }
    if ch.clone() == "D".to_string() {
        return "d".to_string();
    }
    if ch.clone() == "E".to_string() {
        return "e".to_string();
    }
    if ch.clone() == "F".to_string() {
        return "f".to_string();
    }
    return format!("{}{}", ch, "".to_string());
}
fn _is_hex_char(ch: &String) -> bool {
    if ch.clone() == "0".to_string() {
        return true;
    }
    if ch.clone() == "1".to_string() {
        return true;
    }
    if ch.clone() == "2".to_string() {
        return true;
    }
    if ch.clone() == "3".to_string() {
        return true;
    }
    if ch.clone() == "4".to_string() {
        return true;
    }
    if ch.clone() == "5".to_string() {
        return true;
    }
    if ch.clone() == "6".to_string() {
        return true;
    }
    if ch.clone() == "7".to_string() {
        return true;
    }
    if ch.clone() == "8".to_string() {
        return true;
    }
    if ch.clone() == "9".to_string() {
        return true;
    }
    if ch.clone() == "a".to_string() {
        return true;
    }
    if ch.clone() == "b".to_string() {
        return true;
    }
    if ch.clone() == "c".to_string() {
        return true;
    }
    if ch.clone() == "d".to_string() {
        return true;
    }
    if ch.clone() == "e".to_string() {
        return true;
    }
    if ch.clone() == "f".to_string() {
        return true;
    }
    if ch.clone() == "A".to_string() {
        return true;
    }
    if ch.clone() == "B".to_string() {
        return true;
    }
    if ch.clone() == "C".to_string() {
        return true;
    }
    if ch.clone() == "D".to_string() {
        return true;
    }
    if ch.clone() == "E".to_string() {
        return true;
    }
    if ch.clone() == "F".to_string() {
        return true;
    }
    return false;
}
fn _hex_digit_value(ch: &String) -> i64 {
    if ch.clone() == "0".to_string() {
        return 0 as i64;
    }
    if ch.clone() == "1".to_string() {
        return 1 as i64;
    }
    if ch.clone() == "2".to_string() {
        return 2 as i64;
    }
    if ch.clone() == "3".to_string() {
        return 3 as i64;
    }
    if ch.clone() == "4".to_string() {
        return 4 as i64;
    }
    if ch.clone() == "5".to_string() {
        return 5 as i64;
    }
    if ch.clone() == "6".to_string() {
        return 6 as i64;
    }
    if ch.clone() == "7".to_string() {
        return 7 as i64;
    }
    if ch.clone() == "8".to_string() {
        return 8 as i64;
    }
    if ch.clone() == "9".to_string() {
        return 9 as i64;
    }
    if ((ch.clone() == "a".to_string()) || (ch.clone() == "A".to_string())) {
        return 10 as i64;
    }
    if ((ch.clone() == "b".to_string()) || (ch.clone() == "B".to_string())) {
        return 11 as i64;
    }
    if ((ch.clone() == "c".to_string()) || (ch.clone() == "C".to_string())) {
        return 12 as i64;
    }
    if ((ch.clone() == "d".to_string()) || (ch.clone() == "D".to_string())) {
        return 13 as i64;
    }
    if ((ch.clone() == "e".to_string()) || (ch.clone() == "E".to_string())) {
        return 14 as i64;
    }
    if ((ch.clone() == "f".to_string()) || (ch.clone() == "F".to_string())) {
        return 15 as i64;
    }
    return -(1 as i64);
}
fn _substring(value: &String, start: i64, end: i64) -> String {
    let mut result: String = "".to_string();
    let mut i: i64 = start;
    while i < end {
        let ch: Option<String> = {
            let __sifr_index_str = &value;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if let Some(ch) = ch {
            result = format!("{}{}", result, ch);
        }
        i = i + (1 as i64);
    }
    return result;
}
fn _starts_with(value: &String, prefix: &String) -> bool {
    if (value.len() as i64) < (prefix.len() as i64) {
        return false;
    }
    let mut i: i64 = 0 as i64;
    while i < (prefix.chars().count() as i64) {
        let left: Option<String> = {
            let __sifr_index_str = &value;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let right: Option<String> = Some({
            let Some(__indexed_char) = prefix.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if left != right {
            return false;
        }
        i = i + (1 as i64);
    }
    return true;
}
fn _canonical_uuid_text(input_text: &String) -> Result<String, ValueError> {
    let mut normalized_input: String = format!("{}{}", input_text, "".to_string());
    if _starts_with(&normalized_input, &"urn:uuid:".to_string()) {
        normalized_input = _substring(
            &normalized_input,
            9 as i64,
            normalized_input.chars().count() as i64,
        );
    }
    if (normalized_input.chars().count() as i64) >= (2 as i64) {
        let first: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = 0 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let last: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = (normalized_input.chars().count() as i64) - (1 as i64);
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if ((first == Some("{".to_string())) && (last == Some("}".to_string()))) {
            normalized_input = _substring(
                &normalized_input,
                1 as i64,
                (normalized_input.chars().count() as i64) - (1 as i64),
            );
        }
    }
    let input_len: i64 = normalized_input.chars().count() as i64;
    let mut hex_only: String = "".to_string();
    let mut i: i64 = 0 as i64;
    while i < input_len {
        let ch_opt: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            if ch == "-".to_string() {} else {
                if !(_is_hex_char(&ch)) {
                    return Err(ValueError::new("invalid UUID hex string".to_string()));
                }
                hex_only = format!("{}{}", hex_only, _to_lower_hex_char(& ch));
            }
        }
        i = i + (1 as i64);
    }
    if (hex_only.chars().count() as i64) != (32 as i64) {
        return Err(
            ValueError::new("UUID hex string must be 32 hex characters".to_string()),
        );
    }
    if input_len == (36 as i64) {
        let h1: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = 8 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let h2: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = 13 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let h3: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = 18 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let h4: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = 23 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if ((((h1 != Some("-".to_string())) || (h2 != Some("-".to_string())))
            || (h3 != Some("-".to_string()))) || (h4 != Some("-".to_string())))
        {
            return Err(ValueError::new("invalid UUID hex string".to_string()));
        }
    } else {
        if input_len != (32 as i64) {
            return Err(ValueError::new("invalid UUID hex string".to_string()));
        }
    }
    let mut canonical: String = "".to_string();
    let mut j: i64 = 0 as i64;
    while j < (hex_only.chars().count() as i64) {
        if (((j == (8 as i64)) || (j == (12 as i64))) || (j == (16 as i64)))
            || (j == (20 as i64))
        {
            canonical = format!("{}{}", canonical, "-".to_string());
        }
        let part: Option<String> = Some({
            let Some(__indexed_char) = hex_only.chars().nth(j as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(part) = part {
            canonical = format!("{}{}", canonical, part);
        }
        j = j + (1 as i64);
    }
    return Ok(canonical);
}
fn uuid_from_hex(hex_str: &String) -> Result<UUID, ValueError> {
    let __sifr_try_res: Result<Result<UUID, ValueError>, ValueError> = (|| {
        let canonical: String = _canonical_uuid_text(hex_str)?;
        return Ok(Ok(UUID::new(canonical)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(ValueError::new(e.message));
        }
    }
}

// --- stdlib: sifr.textwrap ---
fn _replace_whitespace_chars(text: &String, replace_tabs: bool) -> String {
    let normalized: String = text
        .replace(&"\n".to_string(), &" ".to_string())
        .replace(&"\r".to_string(), &" ".to_string())
        .replace(&"\u{b}".to_string(), &" ".to_string())
        .replace(&"\u{c}".to_string(), &" ".to_string());
    if replace_tabs {
        return normalized.replace(&"\t".to_string(), &" ".to_string());
    }
    return normalized;
}
fn _expand_tabs_impl(text: &String, tabsize: i64) -> String {
    let mut effective_tabsize: i64 = tabsize;
    if effective_tabsize <= (0 as i64) {
        effective_tabsize = 1 as i64;
    }
    let mut result: String = "".to_string();
    let mut column: i64 = 0 as i64;
    let mut i: i64 = 0 as i64;
    while i < (text.chars().count() as i64) {
        let ch_opt: Option<String> = Some({
            let Some(__indexed_char) = text.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            if ch == "\t".to_string() {
                let mut spaces: i64 = effective_tabsize - (column % effective_tabsize);
                if spaces <= (0 as i64) {
                    spaces = effective_tabsize;
                }
                let mut j: i64 = 0 as i64;
                while j < spaces {
                    result = format!("{}{}", result, " ".to_string());
                    j = j + (1 as i64);
                }
                column = column + spaces;
            } else {
                if (ch == "\n".to_string()) || (ch == "\r".to_string()) {
                    result = format!("{}{}", result, ch);
                    column = 0 as i64;
                } else {
                    result = format!("{}{}", result, ch);
                    column = column + (1 as i64);
                }
            }
        }
        i = i + (1 as i64);
    }
    return result;
}
fn _prepare_text(
    text: &String,
    expand_tabs: bool,
    tabsize: i64,
    replace_whitespace: bool,
) -> String {
    let mut prepared: String = format!("{}{}", text, "".to_string());
    if expand_tabs {
        prepared = _expand_tabs_impl(&prepared, tabsize);
    }
    if replace_whitespace {
        prepared = _replace_whitespace_chars(&prepared, true);
    }
    return prepared;
}
fn _normalize_whitespace(text: &String) -> String {
    return _prepare_text(text, true, 8 as i64, true);
}
fn _split_word_units(word: &String, break_on_hyphens: bool) -> Vec<String> {
    if !break_on_hyphens {
        return vec![format!("{}{}", word, "".to_string())];
    }
    let parts: Vec<String> = word
        .split(&"-".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if (parts.len() as i64) <= (1 as i64) {
        return vec![format!("{}{}", word, "".to_string())];
    }
    let mut units: Vec<String> = vec![];
    let mut index: i64 = 0 as i64;
    for part in parts.iter().cloned() {
        let is_last: bool = index == ((parts.len() as i64) - (1 as i64));
        if is_last {
            if (part.chars().count() as i64) > (0 as i64) {
                units.push(part);
            }
        } else {
            if (part.chars().count() as i64) == (0 as i64) {
                units.push("-".to_string());
            } else {
                units.push(format!("{}{}", part, "-".to_string()));
            }
        }
        index = index + (1 as i64);
    }
    if (units.len() as i64) == (0 as i64) {
        units.push(format!("{}{}", word, "".to_string()));
    }
    return units;
}
fn _trim_line(line: &String) -> String {
    let mut start: i64 = 0 as i64;
    while ((start < (line.chars().count() as i64))
        && (({
            let __sifr_index_str = &line;
            let __sifr_index_i = start;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        }) == Some(" ".to_string())))
    {
        start = start + (1 as i64);
    }
    let mut end: i64 = line.chars().count() as i64;
    while ((end > start)
        && (({
            let __sifr_index_str = &line;
            let __sifr_index_i = end - (1 as i64);
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        }) == Some(" ".to_string())))
    {
        end = end - (1 as i64);
    }
    return String::from_iter(
        (line)
            .chars()
            .skip((start).max(0) as usize)
            .take(((end).max(0) - (start).max(0)).max(0) as usize),
    );
}
fn _finalize_line(line: &String, drop_whitespace: bool) -> String {
    if drop_whitespace {
        return _trim_line(line);
    }
    return format!("{}{}", line, "".to_string());
}
fn _wrap_impl(text: &String, width: i64) -> Vec<String> {
    let normalized: String = _normalize_whitespace(text);
    return _wrap_with_indents(
        &normalized,
        width,
        &"".to_string(),
        &"".to_string(),
        true,
        true,
    );
}
fn _effective_content_width(total_width: i64, indent: &String) -> i64 {
    let available: i64 = total_width - (indent.chars().count() as i64);
    if available <= (0 as i64) {
        return 1 as i64;
    }
    return available;
}
fn _push_current_line(
    result: &mut Vec<String>,
    line: &String,
    indent: &String,
    drop_whitespace: bool,
) {
    let candidate: String = _finalize_line(
        &format!("{}{}", indent, line),
        drop_whitespace,
    );
    if drop_whitespace {
        if (candidate.chars().count() as i64) > (0 as i64) {
            result.push(candidate);
        }
    } else {
        result.push(candidate);
    }
}
fn _wrap_with_indents(
    text: &String,
    total_width: i64,
    initial_indent: &String,
    subsequent_indent: &String,
    break_on_hyphens: bool,
    drop_whitespace: bool,
) -> Vec<String> {
    let words: Vec<String> = text
        .split(&" ".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: Vec<String> = vec![];
    let mut current: String = "".to_string();
    let mut first_line: bool = true;
    let mut current_limit: i64 = _effective_content_width(total_width, initial_indent);
    for raw_word in words.iter().cloned() {
        let units: Vec<String> = _split_word_units(&raw_word, break_on_hyphens);
        for word in units.iter().cloned() {
            if (word.chars().count() as i64) == (0 as i64) {
                if drop_whitespace {
                    continue;
                }
                if (current.chars().count() as i64) > (0 as i64) {
                    if ((current.chars().count() as i64) + (1 as i64)) <= current_limit {
                        current = format!("{}{}", current, " ".to_string());
                    }
                }
                continue;
            }
            if (current.chars().count() as i64) == (0 as i64) {
                current = word;
            } else {
                if (((current.chars().count() as i64) + (1 as i64))
                    + (word.chars().count() as i64)) <= current_limit
                {
                    current = format!("{}{}{}", current, " ".to_string(), word);
                } else {
                    if first_line {
                        _push_current_line(
                            &mut result,
                            &current,
                            initial_indent,
                            drop_whitespace,
                        );
                        first_line = false;
                        current_limit = _effective_content_width(
                            total_width,
                            subsequent_indent,
                        );
                    } else {
                        _push_current_line(
                            &mut result,
                            &current,
                            subsequent_indent,
                            drop_whitespace,
                        );
                    }
                    current = word;
                }
            }
        }
    }
    if (current.chars().count() as i64) > (0 as i64) {
        if first_line {
            _push_current_line(&mut result, &current, initial_indent, drop_whitespace);
        } else {
            _push_current_line(
                &mut result,
                &current,
                subsequent_indent,
                drop_whitespace,
            );
        }
    }
    return result;
}
fn wrap(text: &String, width: i64) -> Result<Vec<String>, ValueError> {
    if width <= (0 as i64) {
        return Err(ValueError::new("wrap: width must be > 0".to_string()));
    }
    return Ok(_wrap_impl(text, width));
}

// --- stdlib: sifr.ipaddress ---
fn is_valid_ipv4(addr: &String) -> bool {
    let parts: Vec<String> = addr
        .split(&".".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if (parts.len() as i64) != (4 as i64) {
        return false;
    }
    for part in parts.iter().cloned() {
        if (part.len() as i64) == (0 as i64) {
            return false;
        }
        if (part.len() as i64) > (3 as i64) {
            return false;
        }
        if (part.chars().count() as i64) > (1 as i64) {
            let first_digit: Option<String> = Some({
                let Some(__indexed_char) = part.chars().nth((0 as i64) as usize) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char.to_string()
            });
            if ((first_digit != None) && (first_digit == Some("0".to_string()))) {
                return false;
            }
        }
        let val: i64 = _parse_int(&part);
        if val < (0 as i64) {
            return false;
        }
        if val > (255 as i64) {
            return false;
        }
    }
    return true;
}
fn _parse_int(s: &String) -> i64 {
    let mut result: i64 = 0 as i64;
    let mut i: i64 = 0 as i64;
    while i < (s.chars().count() as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = s.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch) = ch {
            if ch == "0".to_string() {
                result = result * (10 as i64);
            } else {
                if ch == "1".to_string() {
                    result = (result * (10 as i64)) + (1 as i64);
                } else {
                    if ch == "2".to_string() {
                        result = (result * (10 as i64)) + (2 as i64);
                    } else {
                        if ch == "3".to_string() {
                            result = (result * (10 as i64)) + (3 as i64);
                        } else {
                            if ch == "4".to_string() {
                                result = (result * (10 as i64)) + (4 as i64);
                            } else {
                                if ch == "5".to_string() {
                                    result = (result * (10 as i64)) + (5 as i64);
                                } else {
                                    if ch == "6".to_string() {
                                        result = (result * (10 as i64)) + (6 as i64);
                                    } else {
                                        if ch == "7".to_string() {
                                            result = (result * (10 as i64)) + (7 as i64);
                                        } else {
                                            if ch == "8".to_string() {
                                                result = (result * (10 as i64)) + (8 as i64);
                                            } else {
                                                if ch == "9".to_string() {
                                                    result = (result * (10 as i64)) + (9 as i64);
                                                } else {
                                                    return -(1 as i64);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        i = i + (1 as i64);
    }
    return result;
}
fn _ip_to_int_raw(addr: &String) -> i64 {
    let parts: Vec<String> = addr
        .split(&".".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: i64 = 0 as i64;
    for part in parts.iter().cloned() {
        let val: i64 = _parse_int(&part);
        result = (result * (256 as i64)) + val;
    }
    return result;
}
fn ip_to_int(addr: &String) -> Result<i64, ValueError> {
    if !(is_valid_ipv4(addr)) {
        return Err(ValueError::new("invalid IPv4 address".to_string()));
    }
    return Ok(_ip_to_int_raw(addr));
}

// --- stdlib: sifr.datetime ---
#[derive(Debug, Clone)]
struct timezone {
    _offset: i64,
}
impl timezone {
    fn new(offset: i64) -> Self {
        return Self { _offset: offset };
    }
    fn offset(&self) -> i64 {
        return self._offset;
    }
    fn iso_suffix(&self) -> String {
        let mut sign: String = "+".to_string();
        if self._offset < (0 as i64) {
            sign = "-".to_string();
        }
        let mut abs_offset: i64 = self._offset;
        if abs_offset < (0 as i64) {
            abs_offset = -abs_offset;
        }
        let h: i64 = abs_offset / (3600 as i64);
        let m: i64 = (abs_offset % (3600 as i64)) / (60 as i64);
        let mut hs: String = format!("{}", h);
        if (hs.len() as i64) < (2 as i64) {
            hs = format!("{}{}", "0".to_string(), hs);
        }
        let mut ms: String = format!("{}", m);
        if (ms.len() as i64) < (2 as i64) {
            ms = format!("{}{}", "0".to_string(), ms);
        }
        return format!("{}{}{}{}", sign, hs, ":".to_string(), ms);
    }
}
impl PartialEq for timezone {
    fn eq(&self, other: &timezone) -> bool {
        return self._offset == other._offset;
    }
}
impl std::fmt::Display for timezone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self._offset == (0 as i64) {
            return write!(f, "{}", "UTC".to_string());
        }
        return write!(f, "{}", format!("{}{}", "UTC".to_string(), self.iso_suffix()));
    }
}
#[derive(Debug, Clone)]
struct datetime {
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
    _tz_offset: Option<i64>,
}
impl datetime {
    fn new(
        year: i64,
        month: i64,
        day: i64,
        hour: i64,
        minute: i64,
        second: i64,
        tz_offset: Option<i64>,
    ) -> Self {
        return Self {
            year: year,
            month: month,
            day: day,
            hour: hour,
            minute: minute,
            second: second,
            _tz_offset: tz_offset,
        };
    }
    fn isoformat(&self) -> String {
        let y: String = format!("{}", self.year);
        let mut mo: String = format!("{}", self.month);
        if (mo.len() as i64) < (2 as i64) {
            mo = format!("{}{}", "0".to_string(), mo);
        }
        let mut d: String = format!("{}", self.day);
        if (d.len() as i64) < (2 as i64) {
            d = format!("{}{}", "0".to_string(), d);
        }
        let mut h: String = format!("{}", self.hour);
        if (h.len() as i64) < (2 as i64) {
            h = format!("{}{}", "0".to_string(), h);
        }
        let mut mi: String = format!("{}", self.minute);
        if (mi.len() as i64) < (2 as i64) {
            mi = format!("{}{}", "0".to_string(), mi);
        }
        let mut s: String = format!("{}", self.second);
        if (s.len() as i64) < (2 as i64) {
            s = format!("{}{}", "0".to_string(), s);
        }
        let base: String = format!(
            "{}{}{}{}{}{}{}{}{}{}{}", y, "-".to_string(), mo, "-".to_string(), d, "T"
            .to_string(), h, ":".to_string(), mi, ":".to_string(), s
        );
        let tz_offset_opt: Option<i64> = self._tz_offset;
        if let Some(tz_offset_opt) = tz_offset_opt {
            let offset: i64 = tz_offset_opt;
            let mut sign: String = "+".to_string();
            let mut abs_offset: i64 = offset;
            if abs_offset < (0 as i64) {
                sign = "-".to_string();
                abs_offset = -abs_offset;
            }
            let h_off: i64 = abs_offset / (3600 as i64);
            let m_off: i64 = (abs_offset % (3600 as i64)) / (60 as i64);
            let mut hs_off: String = format!("{}", h_off);
            if (hs_off.len() as i64) < (2 as i64) {
                hs_off = format!("{}{}", "0".to_string(), hs_off);
            }
            let mut ms_off: String = format!("{}", m_off);
            if (ms_off.len() as i64) < (2 as i64) {
                ms_off = format!("{}{}", "0".to_string(), ms_off);
            }
            return format!("{}{}{}{}{}", base, sign, hs_off, ":".to_string(), ms_off);
        }
        return base;
    }
    fn timestamp(&self) -> i64 {
        let mut days: i64 = 0 as i64;
        if self.year >= (1970 as i64) {
            let mut y: i64 = 1970 as i64;
            while y < self.year {
                days = days + _days_in_year(y);
                y = y + (1 as i64);
            }
        } else {
            let mut y: i64 = 1969 as i64;
            while y >= self.year {
                days = days - _days_in_year(y);
                y = y - (1 as i64);
            }
        }
        let mut m: i64 = 1 as i64;
        while m < self.month {
            days = days + _days_in_month(self.year, m);
            m = m + (1 as i64);
        }
        days = (days + self.day) - (1 as i64);
        let naive_timestamp: i64 = (((days * (86400 as i64))
            + (self.hour * (3600 as i64))) + (self.minute * (60 as i64))) + self.second;
        let tz_offset_opt: Option<i64> = self._tz_offset;
        if let Some(tz_offset_opt) = tz_offset_opt {
            let offset: i64 = tz_offset_opt;
            return naive_timestamp - offset;
        }
        return naive_timestamp;
    }
    fn astimezone(&self, tz: &Option<timezone>) -> Result<datetime, ValueError> {
        let mut target: timezone = timezone::new(0 as i64);
        if let Some(tz) = tz.as_ref() {
            let __sifr_try_res: Result<(), ValueError> = (|| {
                let tz_text: String = format!("{}", tz);
                let target_offset: i64 = _timezone_offset_from_text(&tz_text)?;
                target = timezone::new(target_offset);
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                return Err(ValueError::new(e.message));
            }
        }
        return from_timestamp(self.timestamp() as f64, &Some(target));
    }
}
impl PartialEq for datetime {
    fn eq(&self, other: &datetime) -> bool {
        let same_tz: bool = self._tz_offset == other._tz_offset;
        return (((((((self.year == other.year) && (self.month == other.month))
            && (self.day == other.day)) && (self.hour == other.hour))
            && (self.minute == other.minute)) && (self.second == other.second))
            && (same_tz));
    }
}
impl std::fmt::Display for datetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.isoformat());
    }
}
fn _is_leap_year(year: i64) -> bool {
    return (((year % (4 as i64)) == (0 as i64)) && ((year % (100 as i64)) != (0 as i64)))
        || ((year % (400 as i64)) == (0 as i64));
}
fn _days_in_year(year: i64) -> i64 {
    if _is_leap_year(year) {
        return 366 as i64;
    }
    return 365 as i64;
}
fn _days_in_month(year: i64, month: i64) -> i64 {
    let month_days: Vec<i64> = vec![
        31 as i64, 28 as i64, 31 as i64, 30 as i64, 31 as i64, 30 as i64, 31 as i64, 31
        as i64, 30 as i64, 31 as i64, 30 as i64, 31 as i64
    ];
    let idx: i64 = month - (1 as i64);
    let d: Option<i64> = {
        let __sifr_index_list = &month_days;
        let __sifr_index_i = idx;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    if ((month == (2 as i64)) && (_is_leap_year(year))) {
        return 29 as i64;
    }
    if let Some(d) = d {
        return d;
    }
    return 0 as i64;
}
fn _parse_datetime_iso(
    value: &String,
) -> Result<(i64, i64, i64, i64, i64, i64), ValueError> {
    if (value.chars().count() as i64) < (19 as i64) {
        return Err(ValueError::new("invalid datetime string".to_string()));
    }
    if (((((({
        let Some(__indexed_char) = value.chars().nth((4 as i64) as usize) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char.to_string()
    }) != "-".to_string())
        || (({
            let Some(__indexed_char) = value.chars().nth((7 as i64) as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        }) != "-".to_string()))
        || (({
            let Some(__indexed_char) = value.chars().nth((10 as i64) as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        }) != "T".to_string()))
        || (({
            let Some(__indexed_char) = value.chars().nth((13 as i64) as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        }) != ":".to_string()))
        || (({
            let Some(__indexed_char) = value.chars().nth((16 as i64) as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        }) != ":".to_string()))
    {
        return Err(ValueError::new("invalid datetime string".to_string()));
    }
    let __sifr_try_res: Result<
        Result<(i64, i64, i64, i64, i64, i64), ValueError>,
        ParseError,
    > = (|| {
        let year: i64 = (_substring(value, 0 as i64, 4 as i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let month: i64 = (_substring(value, 5 as i64, 7 as i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let day: i64 = (_substring(value, 8 as i64, 10 as i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let hour: i64 = (_substring(value, 11 as i64, 13 as i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let minute: i64 = (_substring(value, 14 as i64, 16 as i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let second: i64 = (_substring(value, 17 as i64, 19 as i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        return Ok(Ok((year, month, day, hour, minute, second)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let _e = __sifr_try_err.clone();
            return Err(ValueError::new("invalid datetime string".to_string()));
        }
    }
}
fn _timezone_offset_from_text(text: &String) -> Result<i64, ValueError> {
    if text.clone() == "UTC".to_string() {
        return Ok(0 as i64);
    }
    if (text.chars().count() as i64) != (9 as i64) {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    if _substring(text, 0 as i64, 3 as i64) != "UTC".to_string() {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    let sign_value: String = _substring(text, 3 as i64, 4 as i64);
    if (sign_value != "+".to_string()) && (sign_value != "-".to_string()) {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    if ({
        let __sifr_index_str = &text;
        let __sifr_index_i = 6 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
    }) != Some(":".to_string())
    {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    let __sifr_try_res: Result<Result<i64, ValueError>, ParseError> = (|| {
        let hours: i64 = (_substring(text, 4 as i64, 6 as i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let minutes: i64 = (_substring(text, 7 as i64, 9 as i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let mut offset: i64 = (hours * (3600 as i64)) + (minutes * (60 as i64));
        if sign_value == "-".to_string() {
            offset = -offset;
        }
        return Ok(Ok(offset));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let _e = __sifr_try_err.clone();
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
    }
}
fn _from_timestamp_with_tz(
    ts: f64,
    tz: &Option<timezone>,
) -> Result<datetime, ValueError> {
    let __sifr_try_res: Result<Result<datetime, ValueError>, ValueError> = (|| {
        let whole_seconds: i64 = ts as i64;
        let mut adjusted_seconds: i64 = whole_seconds;
        let mut tz_offset_value: i64 = 0 as i64;
        let mut tz_has_offset: bool = false;
        if let Some(tz) = tz.as_ref() {
            let tz_text: String = format!("{}", tz);
            let tz_offset: i64 = _timezone_offset_from_text(&tz_text)?;
            adjusted_seconds = whole_seconds + tz_offset;
            tz_offset_value = tz_offset;
            tz_has_offset = true;
        }
        let rendered: String = ({
            let __ts = (adjusted_seconds as f64) as i64;
            chrono::DateTime::from_timestamp(__ts, 0)
                .map(|dt| dt.format(&"%Y-%m-%dT%H:%M:%S".to_string()).to_string())
                .ok_or_else(|| ValueError {
                    message: "invalid timestamp".to_string(),
                })
        })?;
        let parts: (i64, i64, i64, i64, i64, i64) = _parse_datetime_iso(&rendered)?;
        let year_part: Option<i64> = Some((parts).0);
        let month_part: Option<i64> = Some((parts).1);
        let day_part: Option<i64> = Some((parts).2);
        let hour_part: Option<i64> = Some((parts).3);
        let minute_part: Option<i64> = Some((parts).4);
        let second_part: Option<i64> = Some((parts).5);
        let mut year: i64 = 0 as i64;
        let mut month: i64 = 1 as i64;
        let mut day: i64 = 1 as i64;
        let mut hour: i64 = 0 as i64;
        let mut minute: i64 = 0 as i64;
        let mut second: i64 = 0 as i64;
        if let Some(year_part) = year_part {
            year = year_part;
        }
        if let Some(month_part) = month_part {
            month = month_part;
        }
        if let Some(day_part) = day_part {
            day = day_part;
        }
        if let Some(hour_part) = hour_part {
            hour = hour_part;
        }
        if let Some(minute_part) = minute_part {
            minute = minute_part;
        }
        if let Some(second_part) = second_part {
            second = second_part;
        }
        if tz_has_offset {
            return Ok(
                Ok(
                    datetime::new(
                        year,
                        month,
                        day,
                        hour,
                        minute,
                        second,
                        Some(tz_offset_value),
                    ),
                ),
            );
        }
        return Ok(Ok(datetime::new(year, month, day, hour, minute, second, None)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(ValueError::new(e.message));
        }
    }
}
fn from_timestamp(ts: f64, tz: &Option<timezone>) -> Result<datetime, ValueError> {
    return _from_timestamp_with_tz(ts, tz);
}

// --- stdlib: sifr.graphlib ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CycleError {
    message: String,
}
impl CycleError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}
impl std::fmt::Display for CycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.message);
    }
}
impl std::error::Error for CycleError {}
fn topological_sort(
    num_nodes: i64,
    from_nodes: &Vec<i64>,
    to_nodes: &Vec<i64>,
) -> Result<Vec<i64>, CycleError> {
    let mut result: Vec<i64> = vec![];
    let mut visited: Vec<i64> = vec![];
    let mut i: i64 = 0 as i64;
    while i < num_nodes {
        visited.push(0 as i64);
        i = i + (1 as i64);
    }
    let mut processed: i64 = 0 as i64;
    while processed < num_nodes {
        let mut found_any: bool = false;
        let mut node: i64 = 0 as i64;
        while node < num_nodes {
            let v: Option<i64> = {
                let __sifr_index_list = &visited;
                let __sifr_index_i = node;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).copied()
            };
            if let Some(v) = v {
                if v == (0 as i64) {
                    let mut has_dep: bool = false;
                    let mut j: i64 = 0 as i64;
                    while j < (to_nodes.len() as i64) {
                        let to_val: Option<i64> = Some(to_nodes[j as usize]);
                        let from_val: Option<i64> = {
                            let __sifr_index_list = &from_nodes;
                            let __sifr_index_i = j;
                            let __sifr_index_norm = if __sifr_index_i < 0 {
                                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                            } else {
                                __sifr_index_i as usize
                            };
                            __sifr_index_list.get(__sifr_index_norm).copied()
                        };
                        if let Some(to_val) = to_val {
                            if let Some(from_val) = from_val {
                                if to_val == node {
                                    let dep_v: Option<i64> = {
                                        let __sifr_index_list = &visited;
                                        let __sifr_index_i = from_val;
                                        let __sifr_index_norm = if __sifr_index_i < 0 {
                                            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                                        } else {
                                            __sifr_index_i as usize
                                        };
                                        __sifr_index_list.get(__sifr_index_norm).copied()
                                    };
                                    if let Some(dep_v) = dep_v {
                                        if dep_v == (0 as i64) {
                                            has_dep = true;
                                        }
                                    }
                                }
                            }
                        }
                        j = j + (1 as i64);
                    }
                    if !has_dep {
                        result.push(node);
                        {
                            let __idx_raw = node;
                            let __idx_norm = if __idx_raw < 0 {
                                (visited.len() as i64) + __idx_raw
                            } else {
                                __idx_raw
                            };
                            if __idx_norm >= 0 {
                                if let Some(__elem) = visited.get_mut(__idx_norm as usize) {
                                    *__elem = 1 as i64;
                                }
                            }
                        }
                        processed = processed + (1 as i64);
                        found_any = true;
                    }
                }
            }
            node = node + (1 as i64);
        }
        if !found_any {
            return Err(CycleError::new("cycle detected in graph".to_string()));
        }
    }
    return Ok(result);
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

fn main() {
    println!("=== 1. random.randint: Validates a <= b ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let r: i64 = randint(1 as i64, 10 as i64)?;
    println!("randint(1, 10) = ok");
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message);
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let r2: i64 = randint(5 as i64, 3 as i64)?;
    println!("should not reach here");
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("randint(5, 3) -> ValueError: {}", e.message);
    }
    println!("=== 2. secrets.randbelow: Validates n > 0 ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let s: i64 = randbelow(100 as i64)?;
    println!("randbelow(100) = ok");
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message);
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let s2: i64 = randbelow(0 as i64)?;
    println!("should not reach here");
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("randbelow(0) -> ValueError: {}", e.message);
    }
    println!("=== 3. textwrap.wrap: Validates width > 0 ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let lines: Vec<String> = wrap(&"hello world".to_string(), 5 as i64)?;
    println!("{}", format!("{}{}{}", "wrap(hello world, 5) = ok (".to_string(), format!("{}", lines.len() as i64), " lines)".to_string()));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "error: ".to_string(), e.message));
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let lines2: Vec<String> = wrap(&"hello".to_string(), 0 as i64)?;
    println!("should not reach here");
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "wrap(hello, 0) -> ValueError: ".to_string(), e.message));
    }
    println!("=== 4. itertools.batched: Validates n > 0 ===");
    let data: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64, 5 as i64];
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let b: Vec<Vec<i64>> = batched(&(data).iter().copied().collect::<Vec<_>>(), 2 as i64)?;
    println!("{}", format!("{}{}{}", "batched([1,2,3,4,5], 2) = ok (".to_string(), format!("{}", b.len() as i64), " batches)".to_string()));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "error: ".to_string(), e.message));
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let b2: Vec<Vec<i64>> = batched(&(data).iter().copied().collect::<Vec<_>>(), 0 as i64)?;
    println!("should not reach here");
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "batched(data, 0) -> ValueError: ".to_string(), e.message));
    }
    println!("=== 5. graphlib.topological_sort: Cycle Detection ===");
    let __sifr_try_res: Result<(), CycleError> = (|| {
    let order: Vec<i64> = topological_sort(3 as i64, &vec![0 as i64, 0 as i64], &vec![1 as i64, 2 as i64])?;
    println!("{}", format!("{}{}", "acyclic graph: ".to_string(), format!("{:?}", order)));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "error: ".to_string(), e.message));
    }
    let __sifr_try_res: Result<(), CycleError> = (|| {
    let order2: Vec<i64> = topological_sort(2 as i64, &vec![0 as i64, 1 as i64], &vec![1 as i64, 0 as i64])?;
    println!("should not reach here");
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("cyclic graph -> CycleError: {}", e.message);
    }
    println!("=== 6. uuid.uuid_from_hex: Validates hex format ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let u: UUID = uuid_from_hex(&"550e8400e29b41d4a716446655440000".to_string())?;
    println!("valid UUID hex: ok");
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message);
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let u2: UUID = uuid_from_hex(&"xyz-invalid!".to_string())?;
    println!("should not reach here");
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("invalid chars -> ValueError: {}", e.message);
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let u3: UUID = uuid_from_hex(&"abcd1234".to_string())?;
    println!("should not reach here");
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("wrong length -> ValueError: {}", e.message);
    }
    println!("=== 7. ipaddress.ip_to_int: Validates IPv4 format ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let ip: i64 = ip_to_int(&"192.168.1.1".to_string())?;
    println!("ip_to_int(192.168.1.1) = ok");
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "error: ".to_string(), e.message));
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let ip2: i64 = ip_to_int(&"bad".to_string())?;
    println!("should not reach here");
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "ip_to_int(bad) -> ValueError: ".to_string(), e.message));
    }
    println!("=== 8. datetime.from_timestamp: Validates timestamp ===");
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let dt: datetime = from_timestamp(0.0 as f64, &None)?;
    println!("from_timestamp(0.0) = ok");
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message);
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let dt2: datetime = from_timestamp(-(99999999999999.0 as f64), &None)?;
    println!("should not reach here");
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("from_timestamp(invalid) -> ValueError: {}", e.message);
    }
    println!("=== 9. SubscriptAssign: Bounds-checked (safe no-op) ===");
    let mut nums: Vec<i64> = vec![10 as i64, 20 as i64, 30 as i64];
    println!("{}", format!("{}{}", "before: ".to_string(), format!("{:?}", nums)));
    {
        let __idx_raw = 99 as i64;
        let __idx_norm = if __idx_raw < 0 { (nums.len() as i64) + __idx_raw } else { __idx_raw };
        if __idx_norm >= 0 {
            if let Some(__elem) = nums.get_mut(__idx_norm as usize) {
                *__elem = 999 as i64;
            }
        }
    }
    println!("{}", format!("{}{}", "after out-of-bounds assign: ".to_string(), format!("{:?}", nums)));
    {
        let __idx_raw = 1 as i64;
        let __idx_norm = if __idx_raw < 0 { (nums.len() as i64) + __idx_raw } else { __idx_raw };
        if __idx_norm >= 0 {
            if let Some(__elem) = nums.get_mut(__idx_norm as usize) {
                *__elem = 99 as i64;
            }
        }
    }
    println!("{}", format!("{}{}", "after valid assign: ".to_string(), format!("{:?}", nums)));
    println!("demo complete!");
}
