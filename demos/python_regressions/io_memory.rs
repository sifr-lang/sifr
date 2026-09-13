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
                (self._buffer.clone())
                    .chars()
                    .skip((tail_start).max(0) as usize),
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
            f,
            "StringIO(_buffer={}, _cursor={}, _closed={})",
            self._buffer, self._cursor, self._closed
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
            _buffer: initial
                .iter()
                .map(|__byte| *__byte as i64)
                .collect::<Vec<i64>>(),
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
    fn _slice_to_bytes(&self, values: &[i64]) -> Result<Vec<u8>, IOError> {
        let __sifr_try_res: Result<Result<Vec<u8>, IOError>, ValueError> = (|| {
            let built: Vec<u8> = ({
                let __vals = values;
                let mut __out = Vec::new();
                for __pair in __vals.iter().enumerate() {
                    if (*__pair.1 < 0) || (*__pair.1 > 255) {
                        return Err(ValueError {
                            message: format!(
                                "byte out of range at index {}: {}",
                                __pair.0, *__pair.1
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
    fn write_bytes(&mut self, data: &[u8]) -> Result<(), IOError> {
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
