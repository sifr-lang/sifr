use std::sync::Mutex;

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

// --- stdlib: sifr.math ---
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
fn random() -> f64 {
    let mut generator: Random = _module_random();
    let value: f64 = generator.random();
    _sync_module_random(&mut generator);
    return value;
}
fn uniform(minimum: f64, maximum: f64) -> f64 {
    let mut generator: Random = _module_random();
    let value: f64 = generator.uniform(minimum, maximum);
    _sync_module_random(&mut generator);
    return value;
}

// --- stdlib: sifr.re ---
fn search(pattern: &String, text: &String) -> Result<Option<String>, RegexError> {
    return regex::Regex::new(&pattern)
        .map(|re| re.find(&text).map(|m| m.as_str().to_string()))
        .map_err(|e| RegexError {
            message: e.to_string(),
            detail: e.to_string(),
        });
}
fn sub(
    pattern: &String,
    replacement: &String,
    text: &String,
) -> Result<String, RegexError> {
    return regex::Regex::new(&pattern)
        .map(|re| re.replace_all(&text, &*replacement).to_string())
        .map_err(|e| RegexError {
            message: e.to_string(),
            detail: e.to_string(),
        });
}
fn findall(pattern: &String, text: &String) -> Result<Vec<String>, RegexError> {
    return regex::Regex::new(&pattern)
        .map(|re| {
            re.find_iter(&text).map(|m| m.as_str().to_string()).collect::<Vec<String>>()
        })
        .map_err(|e| RegexError {
            message: e.to_string(),
            detail: e.to_string(),
        });
}
fn split(pattern: &String, text: &String) -> Result<Vec<String>, RegexError> {
    return regex::Regex::new(&pattern)
        .map(|re| re.split(&text).map(|s| s.to_string()).collect::<Vec<String>>())
        .map_err(|e| RegexError {
            message: e.to_string(),
            detail: e.to_string(),
        });
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

// --- stdlib: sifr.io ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IOBase {
    _closed: bool,
}
impl IOBase {
    fn new() -> Self {
        return Self { _closed: false };
    }
    fn close(&mut self) {
        self._closed = true;
    }
    fn closed(&self) -> bool {
        return self._closed;
    }
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        return Ok(());
    }
    fn seek(&self, offset: i64, whence: i64) -> Result<i64, IOError> {
        let _: i64 = offset;
        let _: i64 = whence;
        return Err(IOError::new(_unsupported_seek_tell_error()));
    }
    fn tell(&self) -> Result<i64, IOError> {
        return Err(IOError::new(_unsupported_seek_tell_error()));
    }
    fn readable(&self) -> bool {
        return false;
    }
    fn writable(&self) -> bool {
        return false;
    }
    fn seekable(&self) -> bool {
        return false;
    }
}
impl std::fmt::Display for IOBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "IOBase(_closed={})", self._closed);
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextIOBase {
    iobase: IOBase,
}
impl TextIOBase {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BinaryIOBase {
    iobase: IOBase,
}
impl BinaryIOBase {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FileHandle {
    _handle: i64,
    _mode: String,
    _closed: bool,
}
impl FileHandle {
    fn new(handle: i64, mode: String) -> Self {
        return Self {
            _handle: handle,
            _mode: mode,
            _closed: false,
        };
    }
    fn close(&mut self) {
        if self._closed {
            return;
        }
        {
            let __hid = self._handle;
            __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner())
                .remove(&__hid);
            ()
        };
        self._closed = true;
    }
    fn closed(&self) -> bool {
        return self._closed;
    }
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        return Ok(());
    }
    fn read(&self) -> Result<String, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        return (|| {
            let __hid = self._handle;
            let mut __handles = __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            match __handles.get_mut(&__hid) {
                Some(SifrFileHandle::TextRead(ref mut __r)) => {
                    let mut __s = String::new();
                    std::io::Read::read_to_string(__r, &mut __s).map_err(__io_err)?;
                    return Ok(__s);
                }
                _ => {
                    return Err(IOError {
                        message: "file not open for reading".to_string(),
                        kind: "Other".to_string(),
                    });
                }
            }
        })();
    }
    fn write(&self, data: &String) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.writable()) {
            return Err(IOError::new("stream is not writable".to_string()));
        }
        return (|| {
            let __hid = self._handle;
            let mut __handles = __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            match __handles.get_mut(&__hid) {
                Some(SifrFileHandle::TextWrite(ref mut __w)) => {
                    let __data = data.as_str();
                    std::io::Write::write_all(__w, __data.as_bytes()).map_err(__io_err)?;
                    return Ok(());
                }
                _ => {
                    return Err(IOError {
                        message: "file not open for writing".to_string(),
                        kind: "Other".to_string(),
                    });
                }
            }
        })();
    }
    fn readline(&self) -> Result<Option<String>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        return (|| {
            let __hid = self._handle;
            let mut __handles = __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            match __handles.get_mut(&__hid) {
                Some(SifrFileHandle::TextRead(ref mut __r)) => {
                    let mut __line = String::new();
                    let __n = std::io::BufRead::read_line(__r, &mut __line)
                        .map_err(__io_err)?;
                    if __n == 0 {
                        return Ok(None);
                    }
                    if __line.ends_with('\n') {
                        __line.pop();
                        if __line.ends_with('\r') {
                            __line.pop();
                        }
                    }
                    return Ok(Some(__line));
                }
                _ => {
                    return Err(IOError {
                        message: "file not open for reading".to_string(),
                        kind: "Other".to_string(),
                    });
                }
            }
        })();
    }
    fn readlines(&self) -> Result<Vec<String>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        return (|| {
            let __hid = self._handle;
            let mut __handles = __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            match __handles.get_mut(&__hid) {
                Some(SifrFileHandle::TextRead(ref mut __r)) => {
                    let mut __lines: Vec<String> = Vec::new();
                    let mut __line = String::new();
                    loop {
                        __line.clear();
                        let __n = std::io::BufRead::read_line(__r, &mut __line)
                            .map_err(__io_err)?;
                        if __n == 0 {
                            break;
                        }
                        let mut __l = __line.clone();
                        if __l.ends_with('\n') {
                            __l.pop();
                            if __l.ends_with('\r') {
                                __l.pop();
                            }
                        }
                        __lines.push(__l);
                    }
                    return Ok(__lines);
                }
                _ => {
                    return Err(IOError {
                        message: "file not open for reading".to_string(),
                        kind: "Other".to_string(),
                    });
                }
            }
        })();
    }
    fn read_bytes(&self) -> Result<Vec<u8>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        return (|| {
            let __hid = self._handle;
            let mut __handles = __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            match __handles.get_mut(&__hid) {
                Some(SifrFileHandle::BinaryRead(ref mut __r)) => {
                    let mut __buf = Vec::new();
                    std::io::Read::read_to_end(__r, &mut __buf).map_err(__io_err)?;
                    return Ok(__buf.to_vec());
                }
                _ => {
                    return Err(IOError {
                        message: "file not open for binary reading".to_string(),
                        kind: "Other".to_string(),
                    });
                }
            }
        })();
    }
    fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.writable()) {
            return Err(IOError::new("stream is not writable".to_string()));
        }
        return (|| {
            let __hid = self._handle;
            let mut __handles = __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            match __handles.get_mut(&__hid) {
                Some(SifrFileHandle::BinaryWrite(ref mut __w)) => {
                    std::io::Write::write_all(__w, &data).map_err(__io_err)?;
                    return Ok(());
                }
                _ => {
                    return Err(IOError {
                        message: "file not open for binary writing".to_string(),
                        kind: "Other".to_string(),
                    });
                }
            }
        })();
    }
    fn seek(&self, offset: i64, whence: i64) -> Result<i64, IOError> {
        let _: i64 = offset;
        let _: i64 = whence;
        return Err(IOError::new(_unsupported_seek_tell_error()));
    }
    fn tell(&self) -> Result<i64, IOError> {
        return Err(IOError::new(_unsupported_seek_tell_error()));
    }
    fn readable(&self) -> bool {
        return _mode_is_readable(&self._mode.clone());
    }
    fn writable(&self) -> bool {
        return _mode_is_writable(&self._mode.clone());
    }
    fn seekable(&self) -> bool {
        return false;
    }
    fn __enter__(&self) -> FileHandle {
        return self.clone();
    }
    fn __exit__(&mut self) {
        self.close();
    }
}
impl std::fmt::Display for FileHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f, "FileHandle(_handle={}, _mode={}, _closed={})", self._handle, self._mode,
            self._closed
        );
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BinaryFileHandle {
    _handle: i64,
    _mode: String,
    _closed: bool,
}
impl BinaryFileHandle {
    fn new(handle: i64, mode: String) -> Self {
        return Self {
            _handle: handle,
            _mode: mode,
            _closed: false,
        };
    }
    fn close(&mut self) {
        if self._closed {
            return;
        }
        {
            let __hid = self._handle;
            __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner())
                .remove(&__hid);
            ()
        };
        self._closed = true;
    }
    fn closed(&self) -> bool {
        return self._closed;
    }
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        return Ok(());
    }
    fn read_bytes(&self, size: Option<i64>) -> Result<Vec<u8>, IOError> {
        let _: Option<i64> = size;
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        return (|| {
            let __hid = self._handle;
            let mut __handles = __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            match __handles.get_mut(&__hid) {
                Some(SifrFileHandle::BinaryRead(ref mut __r)) => {
                    let mut __buf = Vec::new();
                    std::io::Read::read_to_end(__r, &mut __buf).map_err(__io_err)?;
                    return Ok(__buf.to_vec());
                }
                _ => {
                    return Err(IOError {
                        message: "file not open for binary reading".to_string(),
                        kind: "Other".to_string(),
                    });
                }
            }
        })();
    }
    fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.writable()) {
            return Err(IOError::new("stream is not writable".to_string()));
        }
        return (|| {
            let __hid = self._handle;
            let mut __handles = __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            match __handles.get_mut(&__hid) {
                Some(SifrFileHandle::BinaryWrite(ref mut __w)) => {
                    std::io::Write::write_all(__w, &data).map_err(__io_err)?;
                    return Ok(());
                }
                _ => {
                    return Err(IOError {
                        message: "file not open for binary writing".to_string(),
                        kind: "Other".to_string(),
                    });
                }
            }
        })();
    }
    fn seek(&self, offset: i64, whence: i64) -> Result<i64, IOError> {
        let _: i64 = offset;
        let _: i64 = whence;
        return Err(IOError::new(_unsupported_seek_tell_error()));
    }
    fn tell(&self) -> Result<i64, IOError> {
        return Err(IOError::new(_unsupported_seek_tell_error()));
    }
    fn readable(&self) -> bool {
        return _mode_is_readable(&self._mode.clone());
    }
    fn writable(&self) -> bool {
        return _mode_is_writable(&self._mode.clone());
    }
    fn seekable(&self) -> bool {
        return false;
    }
    fn __enter__(&self) -> BinaryFileHandle {
        return self.clone();
    }
    fn __exit__(&mut self) {
        self.close();
    }
}
impl std::fmt::Display for BinaryFileHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f, "BinaryFileHandle(_handle={}, _mode={}, _closed={})", self._handle, self
            ._mode, self._closed
        );
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct StringIO {
    _buffer: String,
    _cursor: i64,
    _closed: bool,
}
impl StringIO {
    fn new(initial: String) -> Self {
        return Self {
            _buffer: format!("{}{}", initial, "".to_string()),
            _cursor: 0 as i64,
            _closed: false,
        };
    }
    fn close(&mut self) {
        self._closed = true;
    }
    fn closed(&self) -> bool {
        return self._closed;
    }
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        return Ok(());
    }
    fn read(&mut self, size: Option<i64>) -> Result<String, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let start: i64 = self._cursor;
        let mut end: i64 = self._buffer.clone().chars().count() as i64;
        if let Some(size) = size {
            let maybe_size: i64 = size;
            if maybe_size >= (0 as i64) {
                let requested: i64 = start + maybe_size;
                if requested < end {
                    end = requested;
                }
            }
        }
        let piece: String = String::from_iter(
            (self._buffer.clone())
                .chars()
                .skip((start).max(0) as usize)
                .take(((end).max(0) - (start).max(0)).max(0) as usize),
        );
        self._cursor = end;
        return Ok(piece);
    }
    fn write(&mut self, data: &String) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let left: String = String::from_iter(
            (self._buffer.clone())
                .chars()
                .skip(0 as usize)
                .take(((self._cursor).max(0) - 0).max(0) as usize),
        );
        let tail_start: i64 = self._cursor + (data.chars().count() as i64);
        let mut right: String = "".to_string();
        if tail_start < (self._buffer.clone().chars().count() as i64) {
            right = String::from_iter(
                (self._buffer.clone()).chars().skip((tail_start).max(0) as usize),
            );
        }
        self._buffer = format!("{}{}{}", left, data, right);
        self._cursor = self._cursor + (data.chars().count() as i64);
        return Ok(());
    }
    fn getvalue(&self) -> String {
        return self._buffer.clone();
    }
    fn seek(&mut self, offset: i64, whence: i64) -> Result<i64, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let mut origin: i64 = 0 as i64;
        if whence == (0 as i64) {
            origin = 0 as i64;
        } else {
            if whence == (1 as i64) {
                origin = self._cursor;
            } else {
                if whence == (2 as i64) {
                    origin = self._buffer.clone().chars().count() as i64;
                } else {
                    return Err(IOError::new(_invalid_whence_error(whence)));
                }
            }
        }
        let mut next_pos: i64 = origin + offset;
        if next_pos < (0 as i64) {
            return Err(IOError::new(_negative_seek_error(next_pos)));
        }
        let end: i64 = self._buffer.clone().chars().count() as i64;
        if next_pos > end {
            next_pos = end;
        }
        self._cursor = next_pos;
        return Ok(self._cursor);
    }
    fn tell(&self) -> Result<i64, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        return Ok(self._cursor);
    }
    fn readable(&self) -> bool {
        return !(self._closed);
    }
    fn writable(&self) -> bool {
        return !(self._closed);
    }
    fn seekable(&self) -> bool {
        return !(self._closed);
    }
}
impl std::fmt::Display for StringIO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f, "StringIO(_buffer={}, _cursor={}, _closed={})", self._buffer, self
            ._cursor, self._closed
        );
    }
}
#[derive(Debug, Clone, PartialEq)]
struct BytesIO {
    _buffer: Vec<i64>,
    _cursor: i64,
    _closed: bool,
}
impl BytesIO {
    fn new(initial: Vec<u8>) -> Self {
        return Self {
            _buffer: initial.iter().map(|__byte| *__byte as i64).collect::<Vec<i64>>(),
            _cursor: 0 as i64,
            _closed: false,
        };
    }
    fn close(&mut self) {
        self._closed = true;
    }
    fn closed(&self) -> bool {
        return self._closed;
    }
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        return Ok(());
    }
    fn _slice_to_bytes(&self, values: &Vec<i64>) -> Result<Vec<u8>, IOError> {
        let __sifr_try_res: Result<Result<Vec<u8>, IOError>, ValueError> = (|| {
            let built: Vec<u8> = ({
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
            })?;
            return Ok(Ok(built));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(IOError::new(e.message));
            }
        }
    }
    fn read_bytes(&mut self, size: Option<i64>) -> Result<Vec<u8>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let start: i64 = self._cursor;
        let mut end: i64 = self._buffer.clone().len() as i64;
        if let Some(size) = size {
            let maybe_size: i64 = size;
            if maybe_size >= (0 as i64) {
                let requested: i64 = start + maybe_size;
                if requested < end {
                    end = requested;
                }
            }
        }
        let chunk: Vec<i64> = Vec::from_iter(
            (self._buffer.clone())
                .iter()
                .skip((start).max(0) as usize)
                .take(((end).max(0) - (start).max(0)).max(0) as usize)
                .cloned(),
        );
        self._cursor = end;
        return self._slice_to_bytes(&chunk);
    }
    fn write_bytes(&mut self, data: &Vec<u8>) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let values: Vec<i64> = data
            .iter()
            .map(|__byte| *__byte as i64)
            .collect::<Vec<i64>>();
        let mut i: i64 = 0 as i64;
        while i < (values.len() as i64) {
            let maybe_value: Option<i64> = Some(values[i as usize]);
            let Some(maybe_value) = maybe_value else {
                return Err(IOError::new("bytes write invariant violation".to_string()));
            };
            let idx: i64 = self._cursor + i;
            if idx < (self._buffer.clone().len() as i64) {
                {
                    let __idx_raw = idx;
                    let __idx_norm = if __idx_raw < 0 {
                        (self._buffer.len() as i64) + __idx_raw
                    } else {
                        __idx_raw
                    };
                    if __idx_norm >= 0 {
                        if let Some(__elem) = self._buffer.get_mut(__idx_norm as usize) {
                            *__elem = maybe_value;
                        }
                    }
                }
            } else {
                self._buffer.push(maybe_value);
            }
            i = i + (1 as i64);
        }
        self._cursor = self._cursor + (values.len() as i64);
        return Ok(());
    }
    fn getvalue(&self) -> Result<Vec<u8>, IOError> {
        return self._slice_to_bytes(&self._buffer.clone());
    }
    fn seek(&mut self, offset: i64, whence: i64) -> Result<i64, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let mut origin: i64 = 0 as i64;
        if whence == (0 as i64) {
            origin = 0 as i64;
        } else {
            if whence == (1 as i64) {
                origin = self._cursor;
            } else {
                if whence == (2 as i64) {
                    origin = self._buffer.clone().len() as i64;
                } else {
                    return Err(IOError::new(_invalid_whence_error(whence)));
                }
            }
        }
        let mut next_pos: i64 = origin + offset;
        if next_pos < (0 as i64) {
            return Err(IOError::new(_negative_seek_error(next_pos)));
        }
        let end: i64 = self._buffer.clone().len() as i64;
        if next_pos > end {
            next_pos = end;
        }
        self._cursor = next_pos;
        return Ok(self._cursor);
    }
    fn tell(&self) -> Result<i64, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        return Ok(self._cursor);
    }
    fn readable(&self) -> bool {
        return !(self._closed);
    }
    fn writable(&self) -> bool {
        return !(self._closed);
    }
    fn seekable(&self) -> bool {
        return !(self._closed);
    }
}
fn _closed_stream_error() -> String {
    return "I/O operation on closed stream".to_string();
}
fn _invalid_whence_error(whence: i64) -> String {
    return format!("{}{}", "invalid whence: ".to_string(), format!("{}", whence));
}
fn _negative_seek_error(offset: i64) -> String {
    return format!(
        "{}{}", "negative seek position: ".to_string(), format!("{}", offset)
    );
}
fn _unsupported_seek_tell_error() -> String {
    return "seek/tell is unsupported for this stream".to_string();
}
fn _mode_is_readable(mode: &String) -> bool {
    return mode.contains(&"r".to_string()) || mode.contains(&"+".to_string());
}
fn _mode_is_writable(mode: &String) -> bool {
    return (mode.contains(&"w".to_string()) || mode.contains(&"a".to_string()))
        || mode.contains(&"+".to_string());
}
fn open(path: &String, mode: &String) -> Result<FileHandle, IOError> {
    let __sifr_try_res: Result<Result<FileHandle, IOError>, IOError> = (|| {
        let handle: i64 = (|| {
            let __path = path.to_string();
            let __mode = mode.to_string();
            let __handle_id = __sifr_next_file_handle_id();
            match __mode.as_str() {
                "r" | "rt" => {
                    let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                    let __reader = std::io::BufReader::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::TextRead(__reader));
                    return Ok(__handle_id);
                }
                "w" | "wt" => {
                    let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                    return Ok(__handle_id);
                }
                "a" | "at" => {
                    let __f = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(__path.as_str())
                        .map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                    return Ok(__handle_id);
                }
                "rb" => {
                    let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                    let __reader = std::io::BufReader::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
                    return Ok(__handle_id);
                }
                "wb" => {
                    let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                    return Ok(__handle_id);
                }
                "ab" => {
                    let __f = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(__path.as_str())
                        .map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                    return Ok(__handle_id);
                }
                _ => {
                    return Err(IOError {
                        message: format!("invalid mode: {}", __mode),
                        kind: "Other".to_string(),
                    });
                }
            }
        })()?;
        return Ok(Ok(FileHandle::new(handle, (mode).clone())));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(IOError::new(e.message));
        }
    }
}
fn open_binary(path: &String, mode: &String) -> Result<BinaryFileHandle, IOError> {
    if !(mode.contains(&"b".to_string())) {
        return Err(IOError::new("open_binary requires binary mode".to_string()));
    }
    let __sifr_try_res: Result<Result<BinaryFileHandle, IOError>, IOError> = (|| {
        let handle: i64 = (|| {
            let __path = path.to_string();
            let __mode = mode.to_string();
            let __handle_id = __sifr_next_file_handle_id();
            match __mode.as_str() {
                "r" | "rt" => {
                    let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                    let __reader = std::io::BufReader::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::TextRead(__reader));
                    return Ok(__handle_id);
                }
                "w" | "wt" => {
                    let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                    return Ok(__handle_id);
                }
                "a" | "at" => {
                    let __f = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(__path.as_str())
                        .map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                    return Ok(__handle_id);
                }
                "rb" => {
                    let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                    let __reader = std::io::BufReader::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
                    return Ok(__handle_id);
                }
                "wb" => {
                    let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                    return Ok(__handle_id);
                }
                "ab" => {
                    let __f = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(__path.as_str())
                        .map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                    return Ok(__handle_id);
                }
                _ => {
                    return Err(IOError {
                        message: format!("invalid mode: {}", __mode),
                        kind: "Other".to_string(),
                    });
                }
            }
        })()?;
        return Ok(Ok(BinaryFileHandle::new(handle, (mode).clone())));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(IOError::new(e.message));
        }
    }
}

// --- stdlib: sifr.json ---
#[derive(Debug, Clone, PartialEq)]
struct JsonValue {
    kind: String,
    bool_value: Option<bool>,
    int_value: Option<i64>,
    float_value: Option<f64>,
    str_value: Option<String>,
    array_items: Box<Vec<JsonValue>>,
    object_items: Box<Vec<(String, JsonValue)>>,
}
impl JsonValue {
    fn new(
        kind: String,
        bool_value: Option<bool>,
        int_value: Option<i64>,
        float_value: Option<f64>,
        str_value: Option<String>,
    ) -> Self {
        return Self {
            kind: kind,
            bool_value: bool_value,
            int_value: int_value,
            float_value: float_value,
            str_value: str_value,
            array_items: Box::new(vec![]),
            object_items: Box::new(vec![]),
        };
    }
    fn is_null(&self) -> bool {
        return self.kind.clone() == "null".to_string();
    }
    fn is_bool(&self) -> bool {
        return self.kind.clone() == "bool".to_string();
    }
    fn is_int(&self) -> bool {
        return self.kind.clone() == "int".to_string();
    }
    fn is_float(&self) -> bool {
        return self.kind.clone() == "float".to_string();
    }
    fn is_str(&self) -> bool {
        return self.kind.clone() == "str".to_string();
    }
    fn is_array(&self) -> bool {
        return self.kind.clone() == "array".to_string();
    }
    fn is_object(&self) -> bool {
        return self.kind.clone() == "object".to_string();
    }
    fn as_bool(&self) -> Option<bool> {
        return self.bool_value;
    }
    fn as_int(&self) -> Option<i64> {
        return self.int_value;
    }
    fn as_float(&self) -> Option<f64> {
        return self.float_value;
    }
    fn as_str(&self) -> Option<String> {
        return self.str_value.clone();
    }
    fn as_array(&self) -> Option<Vec<JsonValue>> {
        if !(self.is_array()) {
            return None;
        }
        let mut result: Vec<JsonValue> = vec![];
        for item in (self.array_items).as_ref().clone().iter().cloned() {
            result.push(item);
        }
        return Some(result);
    }
    fn as_object(&self) -> Option<Vec<(String, JsonValue)>> {
        if !(self.is_object()) {
            return None;
        }
        let mut result: Vec<(String, JsonValue)> = vec![];
        for (key, value) in (self.object_items).as_ref().clone().iter().cloned() {
            result.push((key, value));
        }
        return Some(result);
    }
    fn at(&self, index: i64) -> Option<JsonValue> {
        if !(self.is_array()) {
            return None;
        }
        if ((index < (0 as i64))
            || (index >= ((self.array_items).as_ref().clone().len() as i64)))
        {
            return None;
        }
        let value: Option<JsonValue> = {
            let __sifr_index_list = &self.array_items;
            let __sifr_index_i = index;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        return value;
    }
    fn get(&self, key: &String) -> Option<JsonValue> {
        if !(self.is_object()) {
            return None;
        }
        for (item_key, item_value) in (self.object_items)
            .as_ref()
            .clone()
            .iter()
            .cloned()
        {
            if item_key == *key {
                return Some(item_value);
            }
        }
        return None;
    }
    fn keys(&self) -> Vec<String> {
        let mut result: Vec<String> = vec![];
        if !(self.is_object()) {
            return result;
        }
        for (item_key, _item_value) in (self.object_items)
            .as_ref()
            .clone()
            .iter()
            .cloned()
        {
            result.push(item_key);
        }
        return result;
    }
    fn values(&self) -> Vec<JsonValue> {
        let mut result: Vec<JsonValue> = vec![];
        if !(self.is_object()) {
            return result;
        }
        for (_item_key, item_value) in (self.object_items)
            .as_ref()
            .clone()
            .iter()
            .cloned()
        {
            result.push(item_value);
        }
        return result;
    }
    fn items(&self) -> Vec<(String, JsonValue)> {
        if !(self.is_object()) {
            return vec![];
        }
        let mut result: Vec<(String, JsonValue)> = vec![];
        for (key, value) in (self.object_items).as_ref().clone().iter().cloned() {
            result.push((key, value));
        }
        return result;
    }
}
impl std::fmt::Display for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f, "{}", { let __json_value = self; fn __sifr_json_value_to_serde(value : &
            JsonValue) -> serde_json::Value { match value.kind.as_str() { "null" => {
            return serde_json::Value::Null; }, "bool" => { if let Some(v) = value
            .bool_value { return serde_json::Value::from(v); } return
            serde_json::Value::Null; }, "int" => { if let Some(v) = value.int_value {
            return serde_json::Value::from(v); } return serde_json::Value::Null; },
            "float" => { if let Some(v) = value.float_value { return
            serde_json::Value::from(v); } return serde_json::Value::Null; }, "str" => {
            if let Some(v) = value.str_value.clone() { return
            serde_json::Value::String(v); } return serde_json::Value::Null; }, "array" =>
            { let mut converted = vec![]; for item in value.array_items.as_ref().iter()
            .cloned() { converted.push(__sifr_json_value_to_serde(& item)); } return
            serde_json::Value::Array(converted); }, "object" => { let mut converted =
            serde_json::Map::new(); for entry in value.object_items.as_ref().iter()
            .cloned() { let entry_key = entry.0; let entry_value = entry.1; converted
            .insert(entry_key, __sifr_json_value_to_serde(& entry_value)); } return
            serde_json::Value::Object(converted); }, _ => { return
            serde_json::Value::Null; }, } } serde_json::to_string(&
            __sifr_json_value_to_serde(& __json_value)).unwrap_or_else(| _err | "null"
            .to_string().to_string()) }
        );
    }
}
fn loads(s: &String) -> Result<JsonValue, JSONDecodeError> {
    return {
        let __json_input = s;
        fn __sifr_json_value_from_serde(
            value: serde_json::Value,
        ) -> Result<JsonValue, JSONDecodeError> {
            match value {
                serde_json::Value::Null => {
                    return Ok(JsonValue {
                        kind: "null".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        array_items: Box::new(vec![]),
                        object_items: Box::new(vec![]),
                    });
                }
                serde_json::Value::Bool(b) => {
                    return Ok(JsonValue {
                        kind: "bool".to_string().to_string(),
                        bool_value: Some(b),
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        array_items: Box::new(vec![]),
                        object_items: Box::new(vec![]),
                    });
                }
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        return Ok(JsonValue {
                            kind: "int".to_string().to_string(),
                            bool_value: None,
                            int_value: Some(i),
                            float_value: None,
                            str_value: None,
                            array_items: Box::new(vec![]),
                            object_items: Box::new(vec![]),
                        });
                    }
                    if n.is_u64() {
                        return Err(JSONDecodeError {
                            message: "json integer out of range for sifr int"
                                .to_string()
                                .to_string(),
                            line: 0,
                            column: 0,
                        });
                    }
                    if let Some(f) = n.as_f64() {
                        return Ok(JsonValue {
                            kind: "float".to_string().to_string(),
                            bool_value: None,
                            int_value: None,
                            float_value: Some(f),
                            str_value: None,
                            array_items: Box::new(vec![]),
                            object_items: Box::new(vec![]),
                        });
                    }
                    return Err(JSONDecodeError {
                        message: "unsupported json number representation"
                            .to_string()
                            .to_string(),
                        line: 0,
                        column: 0,
                    });
                }
                serde_json::Value::String(s) => {
                    return Ok(JsonValue {
                        kind: "str".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: Some(s),
                        array_items: Box::new(vec![]),
                        object_items: Box::new(vec![]),
                    });
                }
                serde_json::Value::Array(items) => {
                    let mut converted = vec![];
                    for item in items {
                        converted.push(__sifr_json_value_from_serde(item)?);
                    }
                    return Ok(JsonValue {
                        kind: "array".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        array_items: Box::new(converted),
                        object_items: Box::new(vec![]),
                    });
                }
                serde_json::Value::Object(entries) => {
                    let mut converted = vec![];
                    for entry in entries {
                        let entry_key = entry.0;
                        let entry_value = entry.1;
                        let converted_value = __sifr_json_value_from_serde(entry_value)?;
                        converted.push((entry_key, converted_value));
                    }
                    return Ok(JsonValue {
                        kind: "object".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        array_items: Box::new(vec![]),
                        object_items: Box::new(converted),
                    });
                }
            }
        }
        serde_json::from_str::<serde_json::Value>(__json_input.as_ref())
            .map_err(|e| JSONDecodeError {
                message: e.to_string(),
                line: e.line() as i64,
                column: e.column() as i64,
            })
            .and_then(|parsed| __sifr_json_value_from_serde(parsed))
    };
}

// --- stdlib: sifr.string ---
fn capwords(s: &String) -> String {
    let normalized: String = s
        .replace(&"\t".to_string(), &" ".to_string())
        .replace(&"\n".to_string(), &" ".to_string())
        .replace(&"\r".to_string(), &" ".to_string())
        .replace(&"\u{b}".to_string(), &" ".to_string())
        .replace(&"\u{c}".to_string(), &" ".to_string());
    let words: Vec<String> = normalized
        .split(&" ".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: String = "".to_string();
    let mut first: bool = true;
    for word in words.iter().cloned() {
        if (word.chars().count() as i64) > (0 as i64) {
            if !first {
                result = format!("{}{}", result, " ".to_string());
            }
            first = false;
            let cap: String = {
                let _s = word.clone();
                let mut _c = _s.chars();
                _c.next()
                    .map(|f| f.to_uppercase().to_string() + &_c.as_str().to_lowercase())
                    .unwrap_or_default()
            };
            result = format!("{}{}", result, cap);
        }
    }
    return result;
}

// --- stdlib: sifr.time ---
fn time() -> f64 {
    return std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
}

// --- stdlib: sifr.fnmatch ---
fn fnmatch(name: &String, pattern: &String) -> bool {
    return _match(name, 0 as i64, pattern, 0 as i64);
}
fn _match(name: &String, mut ni: i64, pattern: &String, mut pi: i64) -> bool {
    while pi < (pattern.chars().count() as i64) {
        let pc: Option<String> = Some({
            let Some(__indexed_char) = pattern.chars().nth(pi as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(pc) = pc {
            if pc == "*".to_string() {
                pi = pi + (1 as i64);
                if pi == (pattern.len() as i64) {
                    return true;
                }
                let mut j: i64 = ni;
                while j <= (name.chars().count() as i64) {
                    if _match(name, j, pattern, pi) {
                        return true;
                    }
                    j = j + (1 as i64);
                }
                return false;
            } else {
                if pc == "?".to_string() {
                    if ni >= (name.len() as i64) {
                        return false;
                    }
                    ni = ni + (1 as i64);
                    pi = pi + (1 as i64);
                } else {
                    if ni >= (name.len() as i64) {
                        return false;
                    }
                    let nc: Option<String> = Some({
                        let Some(__indexed_char) = name.chars().nth(ni as usize) else {
                            unreachable!(
                                "compiler-verified string index should be in range"
                            );
                        };
                        __indexed_char.to_string()
                    });
                    if let Some(nc) = nc {
                        if nc != pc {
                            return false;
                        }
                    } else {
                        return false;
                    }
                    ni = ni + (1 as i64);
                    pi = pi + (1 as i64);
                }
            }
        } else {
            return false;
        }
    }
    return ni == (name.chars().count() as i64);
}
fn fnmatch_filter(names: &Vec<String>, pattern: &String) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for name in names.iter().cloned() {
        if fnmatch(&name, pattern) {
            result.push(name);
        }
    }
    return result;
}

// --- stdlib: sifr.platform ---
fn system() -> String {
    return if cfg!(target_os = "windows") {
        "Windows".to_string().to_string()
    } else {
        if cfg!(target_os = "macos") {
            "Darwin".to_string().to_string()
        } else {
            if cfg!(target_os = "linux") {
                "Linux".to_string().to_string()
            } else {
                std::env::consts::OS.to_string()
            }
        }
    };
}
fn machine() -> String {
    return std::env::consts::ARCH.to_string();
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

enum SifrFileHandle {
    TextRead(std::io::BufReader<std::fs::File>),
    TextWrite(std::io::BufWriter<std::fs::File>),
    BinaryRead(std::io::BufReader<std::fs::File>),
    BinaryWrite(std::io::BufWriter<std::fs::File>),
}

static __SIFR_FILE_HANDLES: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<i64, SifrFileHandle>>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

static __SIFR_NEXT_FILE_HANDLE_ID: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(1);

fn __sifr_next_file_handle_id() -> i64 {
    return __SIFR_NEXT_FILE_HANDLE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

fn main() {
    println!("{}", ((2.0 as f64) as f64).powf((10.0 as f64) as f64));
    println!("{}", (-(42.5 as f64)).abs());
    let encoded: String = b64encode(&"Hello, Sifr!".to_string());
    println!("{}", encoded);
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let decoded: String = b64decode(&encoded)?;
    println!("{}", decoded);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("base64 error: {}", err.message);
    }
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let r: i64 = randint(1 as i64, 100 as i64)?;
    println!("{}", r >= (1 as i64));
    println!("{}", r <= (100 as i64));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message);
    }
    let f: f64 = random();
    println!("{}", f >= (0.0 as f64));
    println!("{}", f < (1.0 as f64));
    let u: f64 = uniform(10.0 as f64, 20.0 as f64);
    println!("{}", u >= (10.0 as f64));
    println!("{}", u <= (20.0 as f64));
    let s: String = system();
    println!("{}", (s.chars().count() as i64) > (0 as i64));
    let m: String = machine();
    println!("{}", (m.chars().count() as i64) > (0 as i64));
    let t: f64 = time();
    println!("{}", t > (0.0 as f64));
    let names: Vec<String> = vec!["foo.py".to_string(), "bar.txt".to_string(), "baz.py".to_string(), "qux.rs".to_string()];
    let matched: Vec<String> = fnmatch_filter(&names, &"*.py".to_string());
    println!("{}", matched.len() as i64);
    let __sifr_try_res: Result<(), RegexError> = (|| {
    let found: Option<String> = search(&"[0-9]+".to_string(), &"hello 42 world".to_string())?;
    if let Some(found) = found {
        println!("{}", found);
    }
    let all_nums: Vec<String> = findall(&"[0-9]+".to_string(), &"abc123def456ghi789".to_string())?;
    println!("{}", all_nums.len() as i64);
    let parts: Vec<String> = split(&",".to_string(), &"a,b,c,d".to_string())?;
    println!("{}", parts.len() as i64);
    let replaced: String = sub(&"[0-9]+".to_string(), &"NUM".to_string(), &"item1 and item2".to_string())?;
    println!("{}", replaced);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("regex error: {}", err.message);
    }
    let __sifr_try_res: Result<(), JSONDecodeError> = (|| {
    let data: JsonValue = loads(&"{\"key\":\"value\"}".to_string())?;
    println!("{}", data);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("json error: {}", err.message);
    }
    let out: String = serde_json::to_string(&"hello".to_string()).unwrap_or_default();
    println!("{}", out);
    println!("{}", capwords(&"hello world from sifr".to_string()));
}
