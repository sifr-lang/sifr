use std::sync::Mutex;

// --- stdlib: sifr.timeit ---
fn default_timer() -> f64 {
    return std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
}

// --- stdlib: sifr.difflib ---
fn get_close_matches(
    word: &String,
    possibilities: &Vec<String>,
    n: i64,
    cutoff: f64,
) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    let mut scores: Vec<f64> = vec![];
    for candidate in possibilities.iter().cloned() {
        let score: f64 = _similarity(word, &candidate);
        if score >= cutoff {
            result.push(candidate);
            scores.push(score);
        }
    }
    if (result.len() as i64) <= n {
        return result;
    }
    let mut top: Vec<String> = vec![];
    let mut used: Vec<i64> = vec![];
    let mut count: i64 = 0 as i64;
    while count < n {
        let mut best_idx: i64 = -(1 as i64);
        let mut best_score: f64 = -(1.0 as f64);
        let mut i: i64 = 0 as i64;
        while i < (scores.len() as i64) {
            let mut skip: bool = false;
            for u in used.iter().copied() {
                if u == i {
                    skip = true;
                }
            }
            if !skip {
                let s: Option<f64> = Some(scores[i as usize]);
                if let Some(s) = s {
                    if s > best_score {
                        best_score = s;
                        best_idx = i;
                    }
                }
            }
            i = i + (1 as i64);
        }
        if best_idx >= (0 as i64) {
            used.push(best_idx);
            let val: Option<String> = {
                let __sifr_index_list = &result;
                let __sifr_index_i = best_idx;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(val) = val {
                top.push(val);
            }
        }
        count = count + (1 as i64);
    }
    return top;
}
fn _similarity(a: &String, b: &String) -> f64 {
    let total: i64 = (a.chars().count() as i64) + (b.chars().count() as i64);
    if total == (0 as i64) {
        return 1.0 as f64;
    }
    let mut matches: i64 = 0 as i64;
    let blocks: Vec<(i64, i64, i64)> = _matching_blocks(a, b);
    for block in blocks.iter().copied() {
        let (__sifr_tuple_unpack_0, __sifr_tuple_unpack_1, __sifr_tuple_unpack_2) = block;
        let _ = __sifr_tuple_unpack_0;
        _ = __sifr_tuple_unpack_1;
        let block_size = __sifr_tuple_unpack_2;
        matches = matches + block_size;
    }
    return (((2 as i64) * matches) as f64) / (total as f64);
}
fn _longest_common_substring_range(
    a: &String,
    b: &String,
    a_start: i64,
    a_end: i64,
    b_start: i64,
    b_end: i64,
) -> (i64, i64, i64) {
    let mut best_i: i64 = 0 as i64;
    let mut best_j: i64 = 0 as i64;
    let mut best_len: i64 = 0 as i64;
    let mut i: i64 = a_start;
    while i < a_end {
        let mut j: i64 = b_start;
        while j < b_end {
            let mut k: i64 = 0 as i64;
            while ((i + k) < a_end) && ((j + k) < b_end) {
                let ai: Option<String> = {
                    let __sifr_index_str = &a;
                    let __sifr_index_i = i + k;
                    let __sifr_index_norm = if __sifr_index_i < 0 {
                        ((__sifr_index_str.chars().count() as i64) + __sifr_index_i)
                            as usize
                    } else {
                        __sifr_index_i as usize
                    };
                    __sifr_index_str
                        .chars()
                        .nth(__sifr_index_norm)
                        .map(|c| c.to_string())
                };
                let bj: Option<String> = {
                    let __sifr_index_str = &b;
                    let __sifr_index_i = j + k;
                    let __sifr_index_norm = if __sifr_index_i < 0 {
                        ((__sifr_index_str.chars().count() as i64) + __sifr_index_i)
                            as usize
                    } else {
                        __sifr_index_i as usize
                    };
                    __sifr_index_str
                        .chars()
                        .nth(__sifr_index_norm)
                        .map(|c| c.to_string())
                };
                if ai.is_none() || bj.is_none() {
                    k = k + (1 as i64);
                    continue;
                }
                if ai != bj {
                    break;
                }
                k = k + (1 as i64);
            }
            if k > best_len {
                best_len = k;
                best_i = i;
                best_j = j;
            }
            j = j + (1 as i64);
        }
        i = i + (1 as i64);
    }
    return (best_i, best_j, best_len);
}
fn _sort_blocks(blocks: &Vec<(i64, i64, i64)>) -> Vec<(i64, i64, i64)> {
    let mut sorted_blocks: Vec<(i64, i64, i64)> = vec![];
    for block in blocks.iter().copied() {
        let (bl_a, bl_b, _) = block;
        let mut found_insert_at: bool = false;
        let mut insert_at: i64 = 0 as i64;
        let mut i: i64 = 0 as i64;
        for existing in sorted_blocks.iter().copied() {
            if !found_insert_at {
                let (
                    __sifr_tuple_unpack_0,
                    __sifr_tuple_unpack_1,
                    __sifr_tuple_unpack_2,
                ) = existing;
                let ex_a = __sifr_tuple_unpack_0;
                let ex_b = __sifr_tuple_unpack_1;
                _ = __sifr_tuple_unpack_2;
                if (bl_a < ex_a) || ((bl_a == ex_a) && (bl_b < ex_b)) {
                    insert_at = i;
                    found_insert_at = true;
                }
            }
            i = i + (1 as i64);
        }
        if found_insert_at {
            sorted_blocks.insert(insert_at as usize, block);
        } else {
            sorted_blocks.push(block);
        }
    }
    return sorted_blocks;
}
fn _matching_blocks(a: &String, b: &String) -> Vec<(i64, i64, i64)> {
    let mut pending_a_start: Vec<i64> = vec![0 as i64];
    let mut pending_a_end: Vec<i64> = vec![a.chars().count() as i64];
    let mut pending_b_start: Vec<i64> = vec![0 as i64];
    let mut pending_b_end: Vec<i64> = vec![b.chars().count() as i64];
    let mut unsorted_blocks: Vec<(i64, i64, i64)> = vec![];
    while (pending_a_start.len() as i64) > (0 as i64) {
        let a_start_value: Option<i64> = Some({
            let Some(__sifr_nonempty_pop_value) = pending_a_start.pop() else {
                unreachable!("compiler-verified non-empty pop should return Some");
            };
            __sifr_nonempty_pop_value
        });
        let a_end_value: Option<i64> = pending_a_end.pop();
        let b_start_value: Option<i64> = pending_b_start.pop();
        let b_end_value: Option<i64> = pending_b_end.pop();
        if let Some(a_start_value) = a_start_value {
            if let Some(a_end_value) = a_end_value {
                if let Some(b_start_value) = b_start_value {
                    if let Some(b_end_value) = b_end_value {
                        let (ai, bj, size) = _longest_common_substring_range(
                            a,
                            b,
                            a_start_value,
                            a_end_value,
                            b_start_value,
                            b_end_value,
                        );
                        if size == (0 as i64) {
                            continue;
                        }
                        unsorted_blocks.push((ai, bj, size));
                        let left_a_end: i64 = ai;
                        let left_b_end: i64 = bj;
                        if (a_start_value < left_a_end) && (b_start_value < left_b_end) {
                            pending_a_start.push(a_start_value);
                            pending_a_end.push(left_a_end);
                            pending_b_start.push(b_start_value);
                            pending_b_end.push(left_b_end);
                        }
                        let right_a_start: i64 = ai + size;
                        let right_b_start: i64 = bj + size;
                        if (right_a_start < a_end_value) && (right_b_start < b_end_value)
                        {
                            pending_a_start.push(right_a_start);
                            pending_a_end.push(a_end_value);
                            pending_b_start.push(right_b_start);
                            pending_b_end.push(b_end_value);
                        }
                    }
                }
            }
        }
    }
    let sorted_blocks: Vec<(i64, i64, i64)> = _sort_blocks(&unsorted_blocks);
    let mut merged_blocks: Vec<(i64, i64, i64)> = vec![];
    let mut have_previous: bool = false;
    let mut prev_a: i64 = 0 as i64;
    let mut prev_b: i64 = 0 as i64;
    let mut prev_size: i64 = 0 as i64;
    for block in sorted_blocks.iter().copied() {
        let (bl_a, bl_b, bl_size) = block;
        if !have_previous {
            prev_a = bl_a;
            prev_b = bl_b;
            prev_size = bl_size;
            have_previous = true;
            continue;
        }
        if ((prev_a + prev_size) == bl_a) && ((prev_b + prev_size) == bl_b) {
            prev_size = prev_size + bl_size;
        } else {
            merged_blocks.push((prev_a, prev_b, prev_size));
            prev_a = bl_a;
            prev_b = bl_b;
            prev_size = bl_size;
        }
    }
    if have_previous {
        merged_blocks.push((prev_a, prev_b, prev_size));
    }
    merged_blocks.push((a.chars().count() as i64, b.chars().count() as i64, 0 as i64));
    return merged_blocks;
}
fn unified_diff(a: &Vec<String>, b: &Vec<String>) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    result.push("--- a".to_string());
    result.push("+++ b".to_string());
    let mut max_len: i64 = a.len() as i64;
    if (b.len() as i64) > max_len {
        max_len = b.len() as i64;
    }
    let mut i: i64 = 0 as i64;
    while i < max_len {
        if i < (a.len() as i64) {
            if i < (b.len() as i64) {
                let line_a: Option<String> = Some(a[i as usize].clone());
                let line_b: Option<String> = Some(b[i as usize].clone());
                if let Some(line_a) = line_a {
                    if let Some(line_b) = line_b {
                        if line_a == line_b {
                            result.push(format!("{}{}", " ".to_string(), line_a));
                        } else {
                            result.push(format!("{}{}", "-".to_string(), line_a));
                            result.push(format!("{}{}", "+".to_string(), line_b));
                        }
                    }
                }
            } else {
                let line_a2: Option<String> = Some(a[i as usize].clone());
                if let Some(line_a2) = line_a2 {
                    result.push(format!("{}{}", "-".to_string(), line_a2));
                }
            }
        } else {
            if i < (b.len() as i64) {
                let line_b2: Option<String> = Some(b[i as usize].clone());
                if let Some(line_b2) = line_b2 {
                    result.push(format!("{}{}", "+".to_string(), line_b2));
                }
            }
        }
        i = i + (1 as i64);
    }
    return result;
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

// --- stdlib: sifr.tomllib ---
#[derive(Debug, Clone, PartialEq)]
struct TomlValue {
    kind: String,
    bool_value: Option<bool>,
    int_value: Option<i64>,
    float_value: Option<f64>,
    str_value: Option<String>,
    datetime_value: Option<String>,
    array_items: Box<Vec<TomlValue>>,
    table_items: Box<Vec<(String, TomlValue)>>,
}
impl TomlValue {
    fn new(
        kind: String,
        bool_value: Option<bool>,
        int_value: Option<i64>,
        float_value: Option<f64>,
        str_value: Option<String>,
        datetime_value: Option<String>,
    ) -> Self {
        return Self {
            kind: kind,
            bool_value: bool_value,
            int_value: int_value,
            float_value: float_value,
            str_value: str_value,
            datetime_value: datetime_value,
            array_items: Box::new(vec![]),
            table_items: Box::new(vec![]),
        };
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
    fn is_datetime(&self) -> bool {
        return self.kind.clone() == "datetime".to_string();
    }
    fn is_array(&self) -> bool {
        return self.kind.clone() == "array".to_string();
    }
    fn is_table(&self) -> bool {
        return self.kind.clone() == "table".to_string();
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
    fn as_datetime(&self) -> Option<String> {
        return self.datetime_value.clone();
    }
    fn as_array(&self) -> Option<Vec<TomlValue>> {
        if !(self.is_array()) {
            return None;
        }
        let mut result: Vec<TomlValue> = vec![];
        for item in (self.array_items).as_ref().clone().iter().cloned() {
            result.push(item);
        }
        return Some(result);
    }
    fn as_table(&self) -> Option<Vec<(String, TomlValue)>> {
        if !(self.is_table()) {
            return None;
        }
        let mut result: Vec<(String, TomlValue)> = vec![];
        for (key, value) in (self.table_items).as_ref().clone().iter().cloned() {
            result.push((key, value));
        }
        return Some(result);
    }
    fn at(&self, index: i64) -> Option<TomlValue> {
        if !(self.is_array()) {
            return None;
        }
        if ((index < (0 as i64))
            || (index >= ((self.array_items).as_ref().clone().len() as i64)))
        {
            return None;
        }
        let value: Option<TomlValue> = {
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
    fn get(&self, key: &String) -> Option<TomlValue> {
        if !(self.is_table()) {
            return None;
        }
        for (item_key, item_value) in (self.table_items).as_ref().clone().iter().cloned()
        {
            if item_key == *key {
                return Some(item_value);
            }
        }
        return None;
    }
    fn keys(&self) -> Vec<String> {
        let mut result: Vec<String> = vec![];
        if !(self.is_table()) {
            return result;
        }
        for (item_key, _item_value) in (self.table_items)
            .as_ref()
            .clone()
            .iter()
            .cloned()
        {
            result.push(item_key);
        }
        return result;
    }
    fn values(&self) -> Vec<TomlValue> {
        let mut result: Vec<TomlValue> = vec![];
        if !(self.is_table()) {
            return result;
        }
        for (_item_key, item_value) in (self.table_items)
            .as_ref()
            .clone()
            .iter()
            .cloned()
        {
            result.push(item_value);
        }
        return result;
    }
    fn items(&self) -> Vec<(String, TomlValue)> {
        if !(self.is_table()) {
            return vec![];
        }
        let mut result: Vec<(String, TomlValue)> = vec![];
        for (key, value) in (self.table_items).as_ref().clone().iter().cloned() {
            result.push((key, value));
        }
        return result;
    }
}
fn loads(text: &String) -> Result<TomlValue, TOMLDecodeError> {
    return {
        let __toml_input = &text;
        fn __sifr_toml_value_from_parsed(
            value: toml::Value,
        ) -> Result<TomlValue, TOMLDecodeError> {
            match value {
                toml::Value::Boolean(v) => {
                    return Ok(TomlValue {
                        kind: "bool".to_string().to_string(),
                        bool_value: Some(v),
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        datetime_value: None,
                        array_items: Box::new(vec![]),
                        table_items: Box::new(vec![]),
                    });
                }
                toml::Value::Integer(v) => {
                    return Ok(TomlValue {
                        kind: "int".to_string().to_string(),
                        bool_value: None,
                        int_value: Some(v),
                        float_value: None,
                        str_value: None,
                        datetime_value: None,
                        array_items: Box::new(vec![]),
                        table_items: Box::new(vec![]),
                    });
                }
                toml::Value::Float(v) => {
                    return Ok(TomlValue {
                        kind: "float".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: Some(v),
                        str_value: None,
                        datetime_value: None,
                        array_items: Box::new(vec![]),
                        table_items: Box::new(vec![]),
                    });
                }
                toml::Value::String(v) => {
                    return Ok(TomlValue {
                        kind: "str".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: Some(v),
                        datetime_value: None,
                        array_items: Box::new(vec![]),
                        table_items: Box::new(vec![]),
                    });
                }
                toml::Value::Datetime(v) => {
                    return Ok(TomlValue {
                        kind: "datetime".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        datetime_value: Some(v.to_string()),
                        array_items: Box::new(vec![]),
                        table_items: Box::new(vec![]),
                    });
                }
                toml::Value::Array(items) => {
                    let mut converted = vec![];
                    for item in items {
                        converted.push(__sifr_toml_value_from_parsed(item)?);
                    }
                    return Ok(TomlValue {
                        kind: "array".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        datetime_value: None,
                        array_items: Box::new(converted),
                        table_items: Box::new(vec![]),
                    });
                }
                toml::Value::Table(items) => {
                    let mut converted = vec![];
                    for entry in items {
                        let entry_key = entry.0;
                        let entry_value = entry.1;
                        let converted_value = __sifr_toml_value_from_parsed(
                            entry_value,
                        )?;
                        converted.push((entry_key, converted_value));
                    }
                    return Ok(TomlValue {
                        kind: "table".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        datetime_value: None,
                        array_items: Box::new(vec![]),
                        table_items: Box::new(converted),
                    });
                }
            }
        }
        __toml_input
            .parse::<toml::Value>()
            .map_err(|e| TOMLDecodeError {
                message: e.to_string(),
                line: 0,
                column: 0,
            })
            .and_then(|parsed| __sifr_toml_value_from_parsed(parsed))
    };
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
fn now(tz: &Option<timezone>) -> datetime {
    let current_epoch: f64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    let __sifr_try_res: Result<datetime, ValueError> = (|| {
        let current: datetime = _from_timestamp_with_tz(current_epoch, tz)?;
        return Ok(current);
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let _e = __sifr_try_err.clone();
            let parts: Vec<i64> = {
                let __dt = chrono::Local::now();
                vec![
                    chrono::Datelike::year(& __dt) as i64, chrono::Datelike::month(&
                    __dt) as i64, chrono::Datelike::day(& __dt) as i64,
                    chrono::Timelike::hour(& __dt) as i64, chrono::Timelike::minute(&
                    __dt) as i64, chrono::Timelike::second(& __dt) as i64
                ]
            };
            let mut yr: i64 = 0 as i64;
            let mut mo: i64 = 1 as i64;
            let mut dy: i64 = 1 as i64;
            let mut hr: i64 = 0 as i64;
            let mut mn: i64 = 0 as i64;
            let mut sc: i64 = 0 as i64;
            for (i, v) in Box::new(
                (parts)
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|__pair| ((__pair.0 as i64) + 0, __pair.1)),
            ) {
                if i == (0 as i64) {
                    yr = v;
                }
                if i == (1 as i64) {
                    mo = v;
                }
                if i == (2 as i64) {
                    dy = v;
                }
                if i == (3 as i64) {
                    hr = v;
                }
                if i == (4 as i64) {
                    mn = v;
                }
                if i == (5 as i64) {
                    sc = v;
                }
            }
            if let Some(tz) = tz.as_ref() {
                let __sifr_try_res: Result<datetime, ValueError> = (|| {
                    let parsed_offset: i64 = _timezone_offset_from_text(
                        &format!("{}", tz),
                    )?;
                    return Ok(
                        datetime::new(yr, mo, dy, hr, mn, sc, Some(parsed_offset)),
                    );
                    unreachable!("sifr try/except return capture fell through");
                })();
                match __sifr_try_res {
                    Ok(__sifr_ret_val) => {
                        return __sifr_ret_val;
                    }
                    Err(__sifr_try_err) => {
                        let _e = __sifr_try_err.clone();
                        return datetime::new(yr, mo, dy, hr, mn, sc, None);
                    }
                }
            }
            return datetime::new(yr, mo, dy, hr, mn, sc, None);
        }
    }
}
fn from_timestamp(ts: f64, tz: &Option<timezone>) -> Result<datetime, ValueError> {
    return _from_timestamp_with_tz(ts, tz);
}

// --- stdlib: sifr.logging ---
fn log_info(msg: &String) {
    println!("{}", format!("{}{}", "[INFO] ".to_string(), msg));
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

// --- stdlib: sifr.test ---
fn assert_eq<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    actual: &T,
    expected: &T,
) {
    assert!(* actual == * expected);
}
fn assert_true(value: bool) {
    assert!(value);
}

// --- stdlib: sifr.pathlib ---
fn join_path(base: &String, child: &String) -> String {
    if (base.len() as i64) == (0 as i64) {
        return format!("{}{}", child, "".to_string());
    }
    let last: Option<String> = {
        let __sifr_index_str = &base;
        let __sifr_index_i = (base.chars().count() as i64) - (1 as i64);
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
    };
    if let Some(last) = last {
        if last == "/".to_string() {
            return format!("{}{}", base, child);
        }
    }
    return format!("{}{}{}", base, "/".to_string(), child);
}
fn basename(path: &String) -> String {
    let mut i: i64 = (path.chars().count() as i64) - (1 as i64);
    while i >= (0 as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = path.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch) = ch {
            if ch == "/".to_string() {
                return String::from_iter(
                    (path).chars().skip((i + (1 as i64)).max(0) as usize),
                );
            }
        }
        i = i - (1 as i64);
    }
    return format!("{}{}", path, "".to_string());
}
fn extension(path: &String) -> String {
    let mut i: i64 = (path.chars().count() as i64) - (1 as i64);
    while i >= (0 as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = path.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch) = ch {
            if ch == ".".to_string() {
                return String::from_iter((path).chars().skip((i).max(0) as usize));
            }
            if ch == "/".to_string() {
                return "".to_string();
            }
        }
        i = i - (1 as i64);
    }
    return "".to_string();
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
fn _in_ipv4_range(value: i64, start: i64, end: i64) -> bool {
    if value < start {
        return false;
    }
    if value > end {
        return false;
    }
    return true;
}
fn _is_private_ipv4_value(value: i64) -> bool {
    let mut private_hit: bool = false;
    if _in_ipv4_range(value, 0 as i64, 16777215 as i64) {
        private_hit = true;
    } else {
        if _in_ipv4_range(value, 167772160 as i64, 184549375 as i64) {
            private_hit = true;
        } else {
            if _in_ipv4_range(value, 2130706432 as i64, 2147483647 as i64) {
                private_hit = true;
            } else {
                if _in_ipv4_range(value, 2851995648 as i64, 2852061183 as i64) {
                    private_hit = true;
                } else {
                    if _in_ipv4_range(value, 2886729728 as i64, 2887778303 as i64) {
                        private_hit = true;
                    } else {
                        if _in_ipv4_range(value, 3221225472 as i64, 3221225727 as i64) {
                            private_hit = true;
                        } else {
                            if _in_ipv4_range(
                                value,
                                3221225642 as i64,
                                3221225643 as i64,
                            ) {
                                private_hit = true;
                            } else {
                                if _in_ipv4_range(
                                    value,
                                    3221225984 as i64,
                                    3221226239 as i64,
                                ) {
                                    private_hit = true;
                                } else {
                                    if _in_ipv4_range(
                                        value,
                                        3232235520 as i64,
                                        3232301055 as i64,
                                    ) {
                                        private_hit = true;
                                    } else {
                                        if _in_ipv4_range(
                                            value,
                                            3323068416 as i64,
                                            3323199487 as i64,
                                        ) {
                                            private_hit = true;
                                        } else {
                                            if _in_ipv4_range(
                                                value,
                                                3325256704 as i64,
                                                3325256959 as i64,
                                            ) {
                                                private_hit = true;
                                            } else {
                                                if _in_ipv4_range(
                                                    value,
                                                    3405803776 as i64,
                                                    3405804031 as i64,
                                                ) {
                                                    private_hit = true;
                                                } else {
                                                    if _in_ipv4_range(
                                                        value,
                                                        4026531840 as i64,
                                                        4294967295 as i64,
                                                    ) {
                                                        private_hit = true;
                                                    } else {
                                                        if value == (4294967295 as i64) {
                                                            private_hit = true;
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
            }
        }
    }
    if private_hit {
        if value == (3221225481 as i64) {
            return false;
        }
        if value == (3221225482 as i64) {
            return false;
        }
    }
    return private_hit;
}
fn is_private(addr: &String) -> bool {
    if !(is_valid_ipv4(addr)) {
        return false;
    }
    let val: i64 = _ip_to_int_raw(addr);
    return _is_private_ipv4_value(val);
}
fn is_loopback(addr: &String) -> bool {
    if !(is_valid_ipv4(addr)) {
        return false;
    }
    let parts: Vec<String> = addr
        .split(&".".to_string())
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if (parts.len() as i64) == (4 as i64) {
        let first: Option<String> = Some(parts[(0 as i64) as usize].clone());
        if let Some(first) = first {
            if first == "127".to_string() {
                return true;
            }
        }
    }
    return false;
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

static __SIFR_GLOBAL_LOG_LEVEL: std::sync::LazyLock<std::sync::Mutex<i64>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(20));

fn main() {
    assert!(std::f64::consts::TAU > (6.0 as f64));
    assert!((f64::NAN).is_nan());
    let __sifr_try_res: Result<(), IOError> = (|| {
    let cwd: String = std::env::current_dir().map(|p| p.to_string_lossy().to_string()).map_err(__io_err)?;
    assert!((cwd.chars().count() as i64) > (0 as i64));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("getcwd error: {}", err.message);
        assert!(format!("{}", format!("getcwd error: {}", err.message)) == "stdlib_parity demo: all checks passed!".to_string());
    }
    let __sifr_try_res: Result<(), RegexError> = (|| {
    let matches: Vec<String> = regex::Regex::new(&"[0-9]+".to_string()).map(|re| re.find_iter(&"abc123def456".to_string()).map(|m| m.as_str().to_string()).collect::<Vec<String>>()).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })?;
    assert_eq!(matches.len() as i64, 2 as i64);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("regex error: {}", err.message);
        assert!(format!("{}", format!("regex error: {}", err.message)) == "Total stdlib modules: 37".to_string());
    }
    let from_nodes: Vec<i64> = vec![0 as i64, 0 as i64, 1 as i64];
    let to_nodes: Vec<i64> = vec![1 as i64, 2 as i64, 2 as i64];
    let __sifr_try_res: Result<(), CycleError> = (|| {
    let order: Vec<i64> = topological_sort(3 as i64, &from_nodes, &to_nodes)?;
    assert_eq!(order.len() as i64, 3 as i64);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message);
    }
    let id: String = {
    let seg1 = rand::random::<u32>();
    let seg2 = rand::random::<u16>();
    let seg3 = (rand::random::<u16>() & 4095) | 16384;
    let seg4 = (rand::random::<u16>() & 16383) | 32768;
    let seg5_hi = rand::random::<u32>();
    let seg5_lo = rand::random::<u16>();
    let seg5 = ((seg5_hi as u64) << 16) | (seg5_lo as u64);
    format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}", seg1, seg2, seg3, seg4, seg5)
};
    assert!((id.chars().count() as i64) > (0 as i64));
    let sys: String = if cfg!(target_os = "windows") { "Windows".to_string().to_string() } else { if cfg!(target_os = "macos") { "Darwin".to_string().to_string() } else { if cfg!(target_os = "linux") { "Linux".to_string().to_string() } else { std::env::consts::OS.to_string() } } };
    assert!((sys.chars().count() as i64) > (0 as i64));
    let arch: String = std::env::consts::ARCH.to_string();
    assert!((arch.chars().count() as i64) > (0 as i64));
    let p: String = join_path(&"/usr".to_string(), &"local".to_string());
    assert_eq!(p, "/usr/local");
    assert_eq!(basename(&"/home/user/file.txt".to_string()), "file.txt");
    assert_eq!(extension(&"file.tar.gz".to_string()), ".gz");
    let words: Vec<String> = vec!["apple".to_string(), "ape".to_string(), "application".to_string()];
    let close: Vec<String> = get_close_matches(&"app".to_string(), &words, 2 as i64, 0.3 as f64);
    assert!((close.len() as i64) > (0 as i64));
    assert!(is_valid_ipv4(&"192.168.1.1".to_string()));
    assert!(!(is_valid_ipv4(&"999.1.1.1".to_string())));
    assert!(is_private(&"10.0.0.1".to_string()));
    assert!(is_loopback(&"127.0.0.1".to_string()));
    let start: f64 = default_timer();
    let end: f64 = default_timer();
    assert!(end >= start);
    let __sifr_try_res: Result<(), TOMLDecodeError> = (|| {
    let mut toml_result: TomlValue = loads(&"key = \"value\"".to_string())?;
    assert!((toml_result.keys().len() as i64) > (0 as i64));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("toml error: {}", err.message);
    }
    let dt_now: datetime = now(&None);
    assert!((format!("{}", dt_now).chars().count() as i64) > (0 as i64));
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let mut dt_epoch: datetime = from_timestamp(0.0 as f64, &None)?;
    assert!((dt_epoch.isoformat().chars().count() as i64) > (0 as i64));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message);
    }
    println!("stdlib_parity demo: all checks passed!");
    println!("Total stdlib modules: 37");
}
