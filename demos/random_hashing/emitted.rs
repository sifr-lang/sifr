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

// --- stdlib: sifr.base64 ---
fn b64encode(s: &String) -> String {
    return base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &s.as_bytes(),
    );
}
fn b64decode(s: &String) -> Result<String, ParseError> {
    return {
        let __bytes = base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                &s.as_bytes(),
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        String::from_utf8(__bytes)
            .map_err(|e| ParseError {
                message: e.to_string(),
            })
    };
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

// --- stdlib: sifr.hashlib ---
#[derive(Debug, Clone, PartialEq)]
struct HashObject {
    _algorithm: String,
    _data: Vec<u8>,
    name: String,
    digest_size: i64,
    block_size: i64,
}
impl HashObject {
    fn new(
        algorithm: String,
        data: Vec<u8>,
        name: String,
        digest_size: i64,
        block_size: i64,
    ) -> Self {
        return Self {
            _algorithm: algorithm,
            _data: data,
            name: name,
            digest_size: digest_size,
            block_size: block_size,
        };
    }
    fn update(&mut self, data: &String) {
        self._data = {
            let mut __v = (self._data.clone()).clone();
            __v.extend(
                ({
                    let __s = data;
                    __s.as_bytes().to_vec()
                })
                    .iter()
                    .cloned(),
            );
            __v
        };
    }
    fn update_bytes(&mut self, data: &Vec<u8>) {
        self._data = {
            let mut __v = (self._data.clone()).clone();
            __v.extend((data).iter().cloned());
            __v
        };
    }
    fn hexdigest(&self) -> String {
        return _hash_hex(&self._algorithm.clone(), &self._data.clone());
    }
    fn digest(&self) -> Vec<u8> {
        return _hash_bytes(&self._algorithm.clone(), &self._data.clone());
    }
    fn digest_bytes(&self) -> Vec<u8> {
        return self.digest();
    }
}
fn _build_hash(algorithm: &String, data: &Vec<u8>) -> HashObject {
    let alg: String = algorithm.to_lowercase();
    if alg == "md5".to_string() {
        return HashObject::new(
            alg,
            (data).clone(),
            "md5".to_string(),
            16 as i64,
            64 as i64,
        );
    } else {
        if alg == "sha1".to_string() {
            return HashObject::new(
                alg,
                (data).clone(),
                "sha1".to_string(),
                20 as i64,
                64 as i64,
            );
        } else {
            if alg == "sha224".to_string() {
                return HashObject::new(
                    alg,
                    (data).clone(),
                    "sha224".to_string(),
                    28 as i64,
                    64 as i64,
                );
            } else {
                if alg == "sha256".to_string() {
                    return HashObject::new(
                        alg,
                        (data).clone(),
                        "sha256".to_string(),
                        32 as i64,
                        64 as i64,
                    );
                } else {
                    if alg == "sha384".to_string() {
                        return HashObject::new(
                            alg,
                            (data).clone(),
                            "sha384".to_string(),
                            48 as i64,
                            128 as i64,
                        );
                    } else {
                        if alg == "sha512".to_string() {
                            return HashObject::new(
                                alg,
                                (data).clone(),
                                "sha512".to_string(),
                                64 as i64,
                                128 as i64,
                            );
                        } else {
                            if alg == "blake2b".to_string() {
                                return HashObject::new(
                                    alg,
                                    (data).clone(),
                                    "blake2b".to_string(),
                                    64 as i64,
                                    128 as i64,
                                );
                            } else {
                                if alg == "blake2s".to_string() {
                                    return HashObject::new(
                                        alg,
                                        (data).clone(),
                                        "blake2s".to_string(),
                                        32 as i64,
                                        64 as i64,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    return HashObject::new(
        alg,
        (data).clone(),
        "unknown".to_string(),
        0 as i64,
        0 as i64,
    );
}
fn _hash_bytes(algorithm: &String, data: &Vec<u8>) -> Vec<u8> {
    if algorithm.clone() == "md5".to_string() {
        return md5::compute((data)).0.to_vec();
    } else {
        if algorithm.clone() == "sha1".to_string() {
            return (<sha1::Sha1 as sha1::Digest>::digest)((data)).to_vec();
        } else {
            if algorithm.clone() == "sha224".to_string() {
                return (<sha2::Sha224 as sha2::Digest>::digest)((data)).to_vec();
            } else {
                if algorithm.clone() == "sha256".to_string() {
                    return (<sha2::Sha256 as sha2::Digest>::digest)((data)).to_vec();
                } else {
                    if algorithm.clone() == "sha384".to_string() {
                        return (<sha2::Sha384 as sha2::Digest>::digest)((data)).to_vec();
                    } else {
                        if algorithm.clone() == "sha512".to_string() {
                            return (<sha2::Sha512 as sha2::Digest>::digest)((data))
                                .to_vec();
                        } else {
                            if algorithm.clone() == "blake2b".to_string() {
                                return (<blake2::Blake2b512 as blake2::Digest>::digest)(
                                        (data),
                                    )
                                    .to_vec();
                            } else {
                                if algorithm.clone() == "blake2s".to_string() {
                                    return (<blake2::Blake2s256 as blake2::Digest>::digest)(
                                            (data),
                                        )
                                        .to_vec();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    return vec![];
}
fn _hash_hex(algorithm: &String, data: &Vec<u8>) -> String {
    return _hash_bytes(algorithm, data)
        .iter()
        .map(|__byte| format!("{:02x}", * __byte))
        .collect::<Vec<String>>()
        .join("");
}
fn sha256_obj(data: &String) -> HashObject {
    return _build_hash(
        &"sha256".to_string(),
        &({
            let __s = data;
            __s.as_bytes().to_vec()
        }),
    );
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
    let mut range_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let value: i64 = randint(1 as i64, 5 as i64)?;
    range_ok = (value >= (1 as i64)) && (value <= (5 as i64));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
    }
    assert!(range_ok);
    let payload: String = "random_hashing_seed".to_string();
    let encoded: String = b64encode(&payload);
    let mut decode_ok: bool = false;
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let decoded: String = b64decode(&encoded)?;
    decode_ok = decoded == payload;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
    }
    assert!(decode_ok);
    let mut h: HashObject = sha256_obj(&payload);
    let digest: String = h.hexdigest();
    assert!((digest.len() as i64) == (64 as i64));
    println!("rng_random_hashing_lock_demo: pass");
}
