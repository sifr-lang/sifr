// src/main.rs
use ::std::collections::HashMap;

use ::std::collections::VecDeque;

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
struct __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
    _data: VecDeque<T>,
    maxlen: Option<i64>,
}
impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
    fn new(items: Option<Vec<T>>, maxlen: Option<i64>) -> Self {
        let mut data: Vec<T> = vec![];
        if let Some(items) = items {
            let mut start: i64 = 0_i64;
            if let Some(maxlen) = maxlen {
                if ((items.len() as i64) > maxlen) {
                    start = (items.len() as i64) - maxlen;
                }
            }
            let mut i: i64 = start;
            while (i < (items.len() as i64)) {
                let item: Option<T> = Some(items[i as usize].clone());
                if let Some(item) = item {
                    data.push(item.clone().clone());
                }
                i += 1_i64;
            }
        }
        let __sifr_field_init_0: Option<i64> = maxlen;
        let __sifr_field_init_1: VecDeque<T> = VecDeque::from(data);
        Self {
            maxlen: __sifr_field_init_0,
            _data: __sifr_field_init_1,
        }
    }
}
impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
    fn append(&mut self, val: &T) {
        self._data.push_back(val.clone().clone());
        let maxlen_opt: Option<i64> = self.maxlen;
        if let Some(maxlen_opt) = maxlen_opt {
            let maxlen: i64 = maxlen_opt;
            if ((self._data.len() as i64) > maxlen) {
                self._data.pop_front();
            }
        }
    }
}
impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
    fn appendleft(&mut self, val: &T) {
        self._data.push_front(val.clone().clone());
        let maxlen_opt: Option<i64> = self.maxlen;
        if let Some(maxlen_opt) = maxlen_opt {
            let maxlen: i64 = maxlen_opt;
            if ((self._data.len() as i64) > maxlen) {
                self._data.pop_back();
            }
        }
    }
}
impl<T> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
    fn pop(&mut self) -> Option<T> {
        if ((self._data.len() as i64) == (0_i64)) {
            return None;
        }
        Some({
            let Some(__sifr_nonempty_pop_value) = self._data.pop_back() else {
                unreachable!("compiler-verified non-empty pop should return Some");
            };
            __sifr_nonempty_pop_value
        })
    }
}
impl<T> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
    fn popleft(&mut self) -> Option<T> {
        if ((self._data.len() as i64) == (0_i64)) {
            return None;
        }
        Some({
            let Some(__sifr_nonempty_pop_value) = self._data.pop_front() else {
                unreachable!("compiler-verified non-empty pop should return Some");
            };
            __sifr_nonempty_pop_value
        })
    }
}
impl<T> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
    fn len(&self) -> i64 {
        self._data.len() as i64
    }
}
impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
    fn to_list(&self) -> Vec<T> {
        let mut result: Vec<T> = vec![];
        for v in self._data.clone().iter().cloned() {
            result.push(v.clone().clone());
        }
        result
    }
}
impl<T> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
    fn clear(&mut self) {
        self._data.clear();
    }
}
impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
    fn extend(&mut self, items: &Vec<T>) {
        for v in items.iter().cloned() {
            self._data.push_back(v.clone().clone());
        }
        let maxlen_opt: Option<i64> = self.maxlen;
        if let Some(maxlen_opt) = maxlen_opt {
            let maxlen: i64 = maxlen_opt;
            while ((self._data.len() as i64) > maxlen) {
                self._data.pop_front();
            }
        }
    }
}
impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
    fn extendleft(&mut self, items: &Vec<T>) {
        for v in items.iter().cloned() {
            self._data.push_front(v.clone().clone());
        }
        let maxlen_opt: Option<i64> = self.maxlen;
        if let Some(maxlen_opt) = maxlen_opt {
            let maxlen: i64 = maxlen_opt;
            while ((self._data.len() as i64) > maxlen) {
                self._data.pop_back();
            }
        }
    }
}
impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
    fn copy(&self) -> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        __SifrStdlib_sifr_x2ecollections_x2edeque::new(Some(self.to_list()), self.maxlen)
    }
}
impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
    fn reverse(&mut self) {
        let mut items: Vec<T> = self.to_list();
        items.reverse();
        self._data.clear();
        for item in items.iter().cloned() {
            self._data.push_back(item.clone().clone());
        }
    }
}
impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
    fn rotate(&mut self, n: i64) {
        let length: i64 = self._data.len() as i64;
        if length == (0_i64) {
            return;
        }
        let mut steps: i64 = n % length;
        if steps < (0_i64) {
            steps += length;
        }
        let mut count: i64 = 0_i64;
        while count < steps {
            let value: Option<T> = self._data.pop_back();
            if let Some(value) = value {
                self._data.push_front(value.clone().clone());
            }
            count += 1_i64;
        }
    }
}
impl<T: Clone + PartialEq> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
    fn count(&self, value: &T) -> i64 {
        let mut total: i64 = 0_i64;
        for item in self._data.clone().iter().cloned() {
            if item == *value {
                total += 1_i64;
            }
        }
        total
    }
}
impl<T: Clone + PartialEq> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
    fn index(&self, value: &T, start: i64, stop: Option<i64>) -> Option<i64> {
        let size: i64 = self._data.len() as i64;
        let mut begin: i64 = start;
        if begin < (0_i64) {
            begin = size + begin;
            if begin < (0_i64) {
                begin = 0_i64;
            }
        }
        let mut end: i64 = size;
        if let Some(stop) = stop {
            end = stop;
            if end < (0_i64) {
                end = size + end;
            }
            if end < (0_i64) {
                end = 0_i64;
            }
            if end > size {
                end = size;
            }
        }
        let mut i: i64 = begin;
        while i < end {
            let current: Option<T> = {
                let __sifr_index_list = &self._data;
                let __sifr_index_i = i;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(current) = current {
                if current == *value {
                    return Some(i);
                }
            }
            i += 1_i64;
        }
        None
    }
}
impl<T: Clone + PartialEq> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
    fn remove(&mut self, value: &T) {
        let idx: Option<i64> = self.index(value, 0_i64, None);
        if let Some(idx) = idx {
            let mut rebuilt: Vec<T> = vec![];
            let mut i: i64 = 0_i64;
            while (i < (self._data.len() as i64)) {
                let current: Option<T> = Some(self._data.clone()[i as usize].clone());
                if let Some(current) = current {
                    if i != idx {
                        rebuilt.push(current.clone().clone());
                    }
                }
                i += 1_i64;
            }
            self._data.clear();
            for item in rebuilt.iter().cloned() {
                self._data.push_back(item.clone().clone());
            }
        }
    }
}

// --- stdlib: _sifr.encoding ---
fn _encoding_is_supported_impl(label: &String) -> bool {
    ::sifr_stdlib::encoding::encoding_is_supported(label)
}
fn _encoding_canonical_label_impl(label: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::encoding::encoding_canonical_label(label)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_decode_text_impl(
    data: &Vec<u8>,
    encoding: &String,
    errors: &String,
) -> Result<String, ParseError> {
    ::sifr_stdlib::encoding::encoding_decode_text(data, encoding, errors)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_decode_recoveries_impl(
    data: &Vec<u8>,
    encoding: &String,
    errors: &String,
) -> Result<Vec<String>, ParseError> {
    ::sifr_stdlib::encoding::encoding_decode_recoveries(data, encoding, errors)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_decode_incremental_text_impl(
    data: &Vec<u8>,
    pending: &Vec<u8>,
    encoding: &String,
    errors: &String,
    r#final: bool,
) -> Result<String, ParseError> {
    ::sifr_stdlib::encoding::encoding_decode_incremental_text(
            data,
            pending,
            encoding,
            errors,
            r#final,
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_decode_incremental_recoveries_impl(
    data: &Vec<u8>,
    pending: &Vec<u8>,
    encoding: &String,
    errors: &String,
    r#final: bool,
) -> Result<Vec<String>, ParseError> {
    ::sifr_stdlib::encoding::encoding_decode_incremental_recoveries(
            data,
            pending,
            encoding,
            errors,
            r#final,
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_decode_incremental_pending_impl(
    data: &Vec<u8>,
    pending: &Vec<u8>,
    encoding: &String,
    r#final: bool,
) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::encoding::encoding_decode_incremental_pending(
            data,
            pending,
            encoding,
            r#final,
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_encode_bytes_impl(
    text: &String,
    encoding: &String,
    errors: &String,
) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::encoding::encoding_encode_bytes(text, encoding, errors)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_encode_recoveries_impl(
    text: &String,
    encoding: &String,
    errors: &String,
) -> Result<Vec<String>, ParseError> {
    ::sifr_stdlib::encoding::encoding_encode_recoveries(text, encoding, errors)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}

// --- stdlib: _sifr.fs ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrIoNativeFileHandle {
    _id: String,
}
impl __SifrIoNativeFileHandle {
    fn new(id: String) -> Self {
        let __sifr_field_init_0: String = id;
        Self { _id: __sifr_field_init_0 }
    }
}
impl __SifrIoNativeFileHandle {}
impl ::std::fmt::Display for __SifrIoNativeFileHandle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "NativeFileHandle(_id={})", self._id)
    }
}
fn read_text(path: &String) -> Result<String, IOError> {
    ::sifr_stdlib::fs::read_text(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn write_text(path: &String, content: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::write_text(path, content)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn exists(path: &String) -> bool {
    ::sifr_stdlib::fs::exists(path)
}
fn read_lines(path: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::read_lines(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn append_text(path: &String, content: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::append_text(path, content)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _open_file(path: &String, mode: &String) -> Result<String, IOError> {
    ::sifr_stdlib::fs::open_file(path, mode)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_read(handle: &String) -> Result<String, IOError> {
    ::sifr_stdlib::fs::file_read(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_write(handle: &String, data: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::file_write(handle, data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_readline(handle: &String) -> Result<Option<String>, IOError> {
    ::sifr_stdlib::fs::file_readline(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_readlines(handle: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::file_readlines(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_close(handle: &String) {
    ::sifr_stdlib::fs::file_close(handle);
}
fn _file_read_bytes(handle: &String) -> Result<Vec<u8>, IOError> {
    ::sifr_stdlib::fs::file_read_bytes(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_write_bytes(handle: &String, data: &Vec<u8>) -> Result<(), IOError> {
    ::sifr_stdlib::fs::file_write_bytes(handle, data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn open_file(path: &String, mode: &String) -> Result<__SifrIoNativeFileHandle, IOError> {
    let __sifr_try_res: Result<Result<__SifrIoNativeFileHandle, IOError>, IOError> = (|| {
        let handle_id: String = _open_file(path, mode)?;
        return Ok(Ok(__SifrIoNativeFileHandle::new(handle_id)));
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
fn file_read(handle: &__SifrIoNativeFileHandle) -> Result<String, IOError> {
    _file_read(&handle._id.clone())
}
fn file_write(handle: &__SifrIoNativeFileHandle, data: &String) -> Result<(), IOError> {
    _file_write(&handle._id.clone(), data)
}
fn file_readline(handle: &__SifrIoNativeFileHandle) -> Result<Option<String>, IOError> {
    _file_readline(&handle._id.clone())
}
fn file_readlines(handle: &__SifrIoNativeFileHandle) -> Result<Vec<String>, IOError> {
    _file_readlines(&handle._id.clone())
}
fn file_close(handle: &__SifrIoNativeFileHandle) {
    _file_close(&handle._id.clone());
}
fn file_read_bytes(handle: &__SifrIoNativeFileHandle) -> Result<Vec<u8>, IOError> {
    _file_read_bytes(&handle._id.clone())
}
fn file_write_bytes(
    handle: &__SifrIoNativeFileHandle,
    data: &Vec<u8>,
) -> Result<(), IOError> {
    _file_write_bytes(&handle._id.clone(), data)
}
fn getcwd() -> Result<String, IOError> {
    ::sifr_stdlib::fs::getcwd()
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn listdir(path: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::listdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn mkdir(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::mkdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rmdir(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rmdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn remove_file(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::remove_file(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rename(src: &String, dst: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rename(src, dst)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn chdir(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::chdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn stat_size(path: &String) -> Result<i64, IOError> {
    ::sifr_stdlib::fs::stat_size(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn disk_usage(path: &String) -> Vec<i64> {
    ::sifr_stdlib::fs::disk_usage(path)
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn is_file(path: &String) -> bool {
    ::sifr_stdlib::fs::is_file(path)
}
fn is_dir(path: &String) -> bool {
    ::sifr_stdlib::fs::is_dir(path)
}
fn copy_file(src: &String, dst: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::copy_file(src, dst)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn walk_dir(path: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::walk_dir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rmdir_all(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rmdir_all(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn gettempdir() -> String {
    ::sifr_stdlib::fs::gettempdir()
}
fn makedirs(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::makedirs(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn touch(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::touch(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn resolve_path(path: &String) -> Result<String, IOError> {
    ::sifr_stdlib::fs::resolve_path(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn iterdir(path: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::iterdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn glob_pattern(dir: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::glob_pattern(dir, pattern)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rglob_pattern(dir: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::rglob_pattern(dir, pattern)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}

// --- stdlib: sifr.encoding ---
fn __const_ENCODING_UTF8() -> String {
    "utf-8".to_string().to_string()
}
fn __const_ENCODING_UTF8_SIG() -> String {
    "utf-8-sig".to_string().to_string()
}
fn __const_ENCODING_ASCII() -> String {
    "ascii".to_string().to_string()
}
fn __const_ENCODING_LATIN1() -> String {
    "latin-1".to_string().to_string()
}
fn __const_ENCODING_UTF16_LE() -> String {
    "utf-16-le".to_string().to_string()
}
fn __const_ENCODING_UTF16_BE() -> String {
    "utf-16-be".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1250() -> String {
    "windows-1250".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1251() -> String {
    "windows-1251".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1252() -> String {
    "windows-1252".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1253() -> String {
    "windows-1253".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1254() -> String {
    "windows-1254".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1255() -> String {
    "windows-1255".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1256() -> String {
    "windows-1256".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1257() -> String {
    "windows-1257".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1258() -> String {
    "windows-1258".to_string().to_string()
}
fn __const_DECODE_ERRORS_STRICT() -> String {
    "strict".to_string().to_string()
}
fn __const_DECODE_ERRORS_REPLACE() -> String {
    "replace".to_string().to_string()
}
fn __const_DECODE_ERRORS_IGNORE() -> String {
    "ignore".to_string().to_string()
}
fn __const_DECODE_ERRORS_BACKSLASH_REPLACE() -> String {
    "backslashreplace".to_string().to_string()
}
fn __const_ENCODE_ERRORS_STRICT() -> String {
    "strict".to_string().to_string()
}
fn __const_ENCODE_ERRORS_REPLACE() -> String {
    "replace".to_string().to_string()
}
fn __const_ENCODE_ERRORS_IGNORE() -> String {
    "ignore".to_string().to_string()
}
fn __const_ENCODE_ERRORS_BACKSLASH_REPLACE() -> String {
    "backslashreplace".to_string().to_string()
}
fn __const_ENCODE_ERRORS_XMLCHARREF_REPLACE() -> String {
    "xmlcharrefreplace".to_string().to_string()
}
fn __const_ENCODE_ERRORS_NAME_REPLACE() -> String {
    "namereplace".to_string().to_string()
}
#[derive(Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eDecodeError {
    message: String,
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeError {}
impl ::std::fmt::Debug for __SifrStdlib_sifr_x2eencoding_x2eDecodeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("DecodeError").field("message", &self.message).finish()
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eDecodeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl ::std::error::Error for __SifrStdlib_sifr_x2eencoding_x2eDecodeError {}
#[derive(Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eEncodeError {
    message: String,
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeError {}
impl ::std::fmt::Debug for __SifrStdlib_sifr_x2eencoding_x2eEncodeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("EncodeError").field("message", &self.message).finish()
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eEncodeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl ::std::error::Error for __SifrStdlib_sifr_x2eencoding_x2eEncodeError {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    label: String,
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    fn new(label: String) -> Self {
        let __sifr_field_init_0: String = {
            let mut __sifr_concat: String = String::with_capacity(label.len() + 0usize);
            __sifr_concat.push_str((label).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        Self { label: __sifr_field_init_0 }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    fn canonical_label(
        &self,
    ) -> Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
        encoding_canonical_label(&self.label)
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    fn is_supported(&self) -> bool {
        encoding_is_supported(&self.label)
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Encoding(label={})", self.label)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    name: String,
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    fn new(name: String) -> Self {
        let __sifr_field_init_0: String = {
            let mut __sifr_concat: String = String::with_capacity(name.len() + 0usize);
            __sifr_concat.push_str((name).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        Self { name: __sifr_field_init_0 }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "DecodeErrorHandler(name={})", self.name)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    name: String,
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    fn new(name: String) -> Self {
        let __sifr_field_init_0: String = {
            let mut __sifr_concat: String = String::with_capacity(name.len() + 0usize);
            __sifr_concat.push_str((name).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        Self { name: __sifr_field_init_0 }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "EncodeErrorHandler(name={})", self.name)
    }
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
    text: String,
    recoveries: Vec<String>,
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
    fn new(text: String, recoveries: Vec<String>) -> Self {
        let __sifr_field_init_0: String = {
            let mut __sifr_concat: String = String::with_capacity(text.len() + 0usize);
            __sifr_concat.push_str((text).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        let __sifr_field_init_1: Vec<String> = recoveries;
        Self {
            text: __sifr_field_init_0,
            recoveries: __sifr_field_init_1,
        }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
    fn get_text(&self) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((self.text.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
    fn get_recoveries(&self) -> Vec<String> {
        self.recoveries.clone()
    }
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
    data: Vec<u8>,
    recoveries: Vec<String>,
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
    fn new(data: Vec<u8>, recoveries: Vec<String>) -> Self {
        let __sifr_field_init_0: Vec<u8> = data;
        let __sifr_field_init_1: Vec<String> = recoveries;
        Self {
            data: __sifr_field_init_0,
            recoveries: __sifr_field_init_1,
        }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
    fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
    fn get_recoveries(&self) -> Vec<String> {
        self.recoveries.clone()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eDecoder {
    _encoding: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
    _errors: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler,
    _exhausted: bool,
    _pending: Vec<u8>,
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecoder {
    fn new(
        enc: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
        errors: Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
    ) -> Self {
        let __sifr_field_init_0: __SifrStdlib_sifr_x2eencoding_x2eEncoding = enc;
        let __sifr_field_init_1: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler = _decode_handler_or_strict(
            &errors,
        );
        let __sifr_field_init_2: bool = false;
        let __sifr_field_init_3: Vec<u8> = vec![];
        Self {
            _encoding: __sifr_field_init_0,
            _errors: __sifr_field_init_1,
            _exhausted: __sifr_field_init_2,
            _pending: __sifr_field_init_3,
        }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecoder {
    fn decode(
        &mut self,
        data: &Vec<u8>,
        r#final: bool,
    ) -> Result<
        __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
        __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
    > {
        if self._exhausted {
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(
                    "decoder is exhausted".to_string(),
                ),
            );
        }
        let __sifr_try_res: Result<
            Result<
                __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
            >,
            __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
        > = (|| {
            let outcome: __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome = encoding_decode_incremental_outcome(
                data,
                &self._pending,
                &self._encoding.clone().label,
                &self._errors.clone().name,
                r#final,
            )?;
            let next_pending: Vec<u8> = encoding_decode_incremental_pending(
                data,
                &self._pending,
                &self._encoding.clone().label,
                r#final,
            )?;
            self._pending = next_pending;
            if r#final {
                self._pending = vec![];
                self._exhausted = true;
            }
            return Ok(Ok(outcome));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(__SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message));
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eEncoder {
    _encoding: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
    _errors: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler,
    _exhausted: bool,
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncoder {
    fn new(
        enc: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
        errors: Option<__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler>,
    ) -> Self {
        let __sifr_field_init_0: __SifrStdlib_sifr_x2eencoding_x2eEncoding = enc;
        let __sifr_field_init_1: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler = _encode_handler_or_strict(
            &errors,
        );
        let __sifr_field_init_2: bool = false;
        Self {
            _encoding: __sifr_field_init_0,
            _errors: __sifr_field_init_1,
            _exhausted: __sifr_field_init_2,
        }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncoder {
    fn encode(
        &mut self,
        text: &String,
        r#final: bool,
    ) -> Result<
        __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome,
        __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
    > {
        if self._exhausted {
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(
                    "encoder is exhausted".to_string(),
                ),
            );
        }
        let __sifr_try_res: Result<
            Result<
                __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome,
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
            >,
            __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
        > = (|| {
            let outcome: __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome = encode_outcome(
                text,
                &self._encoding,
                &Some((self._errors.clone()).clone()),
            )?;
            if r#final {
                self._exhausted = true;
            }
            return Ok(Ok(outcome));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(__SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message));
            }
        }
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eEncoder {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(
            f, "Encoder(_encoding={}, _errors={}, _exhausted={})", self._encoding, self
            ._errors, self._exhausted
        )
    }
}
fn encoding_is_supported(label: &String) -> bool {
    _encoding_is_supported_impl(label)
}
fn encoding_canonical_label(
    label: &String,
) -> Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
    let __sifr_try_res: Result<
        Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
        ParseError,
    > = (|| {
        let value: String = _encoding_canonical_label_impl(label)?;
        return Ok(Ok(value));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(__SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message));
        }
    }
}
fn encoding_decode_text(
    data: &Vec<u8>,
    encoding: &String,
    errors: &String,
) -> Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
    let __sifr_try_res: Result<
        Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
        ParseError,
    > = (|| {
        let text: String = _encoding_decode_text_impl(data, encoding, errors)?;
        return Ok(Ok(text));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(__SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message));
        }
    }
}
fn encoding_decode_recoveries(
    data: &Vec<u8>,
    encoding: &String,
    errors: &String,
) -> Result<Vec<String>, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
    let __sifr_try_res: Result<
        Result<Vec<String>, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
        ParseError,
    > = (|| {
        let recoveries: Vec<String> = _encoding_decode_recoveries_impl(
            data,
            encoding,
            errors,
        )?;
        return Ok(Ok(recoveries));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(__SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message));
        }
    }
}
fn encoding_decode_outcome(
    data: &Vec<u8>,
    encoding: &String,
    errors: &String,
) -> Result<
    __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
    __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
> {
    let __sifr_try_res: Result<
        Result<
            __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
            __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
        >,
        ParseError,
    > = (|| {
        let text: String = _encoding_decode_text_impl(data, encoding, errors)?;
        let recoveries: Vec<String> = _encoding_decode_recoveries_impl(
            data,
            encoding,
            errors,
        )?;
        return Ok(
            Ok(__SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome::new(text, recoveries)),
        );
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(__SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message));
        }
    }
}
fn encoding_decode_incremental_outcome(
    data: &Vec<u8>,
    pending: &Vec<u8>,
    encoding: &String,
    errors: &String,
    r#final: bool,
) -> Result<
    __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
    __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
> {
    let __sifr_try_res: Result<
        Result<
            __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
            __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
        >,
        ParseError,
    > = (|| {
        let text: String = _encoding_decode_incremental_text_impl(
            data,
            pending,
            encoding,
            errors,
            r#final,
        )?;
        let recoveries: Vec<String> = _encoding_decode_incremental_recoveries_impl(
            data,
            pending,
            encoding,
            errors,
            r#final,
        )?;
        return Ok(
            Ok(__SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome::new(text, recoveries)),
        );
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(__SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message));
        }
    }
}
fn encoding_decode_incremental_pending(
    data: &Vec<u8>,
    pending: &Vec<u8>,
    encoding: &String,
    r#final: bool,
) -> Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
    let __sifr_try_res: Result<
        Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
        ParseError,
    > = (|| {
        let next_pending: Vec<u8> = _encoding_decode_incremental_pending_impl(
            data,
            pending,
            encoding,
            r#final,
        )?;
        return Ok(Ok(next_pending));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(__SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message));
        }
    }
}
fn encoding_encode_bytes(
    text: &String,
    encoding: &String,
    errors: &String,
) -> Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError> {
    let __sifr_try_res: Result<
        Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError>,
        ParseError,
    > = (|| {
        let data: Vec<u8> = _encoding_encode_bytes_impl(text, encoding, errors)?;
        return Ok(Ok(data));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(__SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message));
        }
    }
}
fn encoding_encode_recoveries(
    text: &String,
    encoding: &String,
    errors: &String,
) -> Result<Vec<String>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError> {
    let __sifr_try_res: Result<
        Result<Vec<String>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError>,
        ParseError,
    > = (|| {
        let recoveries: Vec<String> = _encoding_encode_recoveries_impl(
            text,
            encoding,
            errors,
        )?;
        return Ok(Ok(recoveries));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(__SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message));
        }
    }
}
fn encoding_encode_outcome(
    text: &String,
    encoding: &String,
    errors: &String,
) -> Result<
    __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome,
    __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
> {
    let __sifr_try_res: Result<
        Result<
            __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome,
            __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
        >,
        ParseError,
    > = (|| {
        let data: Vec<u8> = _encoding_encode_bytes_impl(text, encoding, errors)?;
        let recoveries: Vec<String> = _encoding_encode_recoveries_impl(
            text,
            encoding,
            errors,
        )?;
        return Ok(
            Ok(__SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome::new(data, recoveries)),
        );
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(__SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message));
        }
    }
}
fn encoding(label: &String) -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new((label).clone())
}
fn utf8() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF8())
}
fn utf8_sig() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF8_SIG())
}
fn ascii() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_ASCII())
}
fn latin1() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_LATIN1())
}
fn utf16_le() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF16_LE())
}
fn utf16_be() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF16_BE())
}
fn windows1252() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_WINDOWS_1252())
}
fn strict_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
        __const_DECODE_ERRORS_STRICT(),
    )
}
fn replace_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
        __const_DECODE_ERRORS_REPLACE(),
    )
}
fn ignore_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
        __const_DECODE_ERRORS_IGNORE(),
    )
}
fn backslash_replace_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
        __const_DECODE_ERRORS_BACKSLASH_REPLACE(),
    )
}
fn strict_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        __const_ENCODE_ERRORS_STRICT(),
    )
}
fn replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        __const_ENCODE_ERRORS_REPLACE(),
    )
}
fn ignore_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        __const_ENCODE_ERRORS_IGNORE(),
    )
}
fn backslash_replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        __const_ENCODE_ERRORS_BACKSLASH_REPLACE(),
    )
}
fn xmlcharref_replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        __const_ENCODE_ERRORS_XMLCHARREF_REPLACE(),
    )
}
fn name_replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        __const_ENCODE_ERRORS_NAME_REPLACE(),
    )
}
fn _decode_handler_name(
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
) -> String {
    if let Some(errors) = errors.as_ref() {
        return {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((errors.name.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    __const_DECODE_ERRORS_STRICT()
}
fn _encode_handler_name(
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler>,
) -> String {
    if let Some(errors) = errors.as_ref() {
        return {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((errors.name.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    __const_ENCODE_ERRORS_STRICT()
}
fn _decode_handler_or_strict(
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
) -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    if let Some(errors) = errors.as_ref() {
        return __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
            format!("{}{}", errors.name.clone(), ""),
        );
    }
    strict_decode_handler()
}
fn _encode_handler_or_strict(
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler>,
) -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    if let Some(errors) = errors.as_ref() {
        return __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
            format!("{}{}", errors.name.clone(), ""),
        );
    }
    strict_encode_handler()
}
fn decode_outcome(
    data: &Vec<u8>,
    enc: &__SifrStdlib_sifr_x2eencoding_x2eEncoding,
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
) -> Result<
    __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
    __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
> {
    let handler_name: String = _decode_handler_name(errors);
    let __sifr_try_res: Result<
        Result<
            __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
            __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
        >,
        __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
    > = (|| {
        return Ok(encoding_decode_outcome(data, &enc.label.clone(), &handler_name));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(__SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message));
        }
    }
}
fn decode(
    data: &Vec<u8>,
    enc: &__SifrStdlib_sifr_x2eencoding_x2eEncoding,
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
) -> Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
    let __sifr_try_res: Result<
        Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
        __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
    > = (|| {
        let outcome: __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome = decode_outcome(
            data,
            enc,
            errors,
        )?;
        return Ok(Ok(outcome.get_text()));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(__SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message));
        }
    }
}
fn encode_outcome(
    text: &String,
    enc: &__SifrStdlib_sifr_x2eencoding_x2eEncoding,
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler>,
) -> Result<
    __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome,
    __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
> {
    let handler_name: String = _encode_handler_name(errors);
    let __sifr_try_res: Result<
        Result<
            __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome,
            __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
        >,
        __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
    > = (|| {
        return Ok(encoding_encode_outcome(text, &enc.label.clone(), &handler_name));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(__SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message));
        }
    }
}
fn encode(
    text: &String,
    enc: &__SifrStdlib_sifr_x2eencoding_x2eEncoding,
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler>,
) -> Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError> {
    let __sifr_try_res: Result<
        Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError>,
        __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
    > = (|| {
        let outcome: __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome = encode_outcome(
            text,
            enc,
            errors,
        )?;
        return Ok(Ok(outcome.get_data()));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(__SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message));
        }
    }
}

// --- stdlib: sifr.io ---
#[derive(Debug, Clone)]
enum __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
    __SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(IOError),
    __SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a0(
        __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
    ),
}
impl From<IOError>
for __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
    fn from(value: IOError) -> Self {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
            value,
        )
    }
}
impl ::std::fmt::Display
for __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
        }
    }
}
#[derive(Debug, Clone)]
enum __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
    __SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(IOError),
    __SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a0(
        __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
    ),
}
impl From<IOError>
for __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
    fn from(value: IOError) -> Self {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
            value,
        )
    }
}
impl ::std::fmt::Display
for __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eio_x2eIOBase {
    _closed: bool,
}
impl __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn new() -> Self {
        let __sifr_field_init_0: bool = false;
        Self {
            _closed: __sifr_field_init_0,
        }
    }
}
impl __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn close(&mut self) {
        self._closed = true;
    }
}
impl __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn closed(&self) -> bool {
        self._closed
    }
}
impl __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(())
    }
}
impl __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn seek(&self, offset: i64, whence: i64) -> Result<i64, IOError> {
        let _ = offset;
        let _ = whence;
        Err(IOError::new(_unsupported_seek_tell_error()))
    }
}
impl __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn tell(&self) -> Result<i64, IOError> {
        Err(IOError::new(_unsupported_seek_tell_error()))
    }
}
impl __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn readable(&self) -> bool {
        false
    }
}
impl __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn writable(&self) -> bool {
        false
    }
}
impl __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn seekable(&self) -> bool {
        false
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "IOBase(_closed={})", self._closed)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eio_x2eTextIOBase {
    iobase: __SifrStdlib_sifr_x2eio_x2eIOBase,
}
impl ::std::ops::Deref for __SifrStdlib_sifr_x2eio_x2eTextIOBase {
    type Target = __SifrStdlib_sifr_x2eio_x2eIOBase;
    fn deref(&self) -> &Self::Target {
        &self.iobase
    }
}
impl ::std::ops::DerefMut for __SifrStdlib_sifr_x2eio_x2eTextIOBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.iobase
    }
}
impl ::std::convert::From<__SifrStdlib_sifr_x2eio_x2eTextIOBase>
for __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn from(value: __SifrStdlib_sifr_x2eio_x2eTextIOBase) -> Self {
        value.iobase
    }
}
impl __SifrStdlib_sifr_x2eio_x2eTextIOBase {}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eio_x2eTextIOBase {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "TextIOBase(iobase={})", self.iobase)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eio_x2eBinaryIOBase {
    iobase: __SifrStdlib_sifr_x2eio_x2eIOBase,
}
impl ::std::ops::Deref for __SifrStdlib_sifr_x2eio_x2eBinaryIOBase {
    type Target = __SifrStdlib_sifr_x2eio_x2eIOBase;
    fn deref(&self) -> &Self::Target {
        &self.iobase
    }
}
impl ::std::ops::DerefMut for __SifrStdlib_sifr_x2eio_x2eBinaryIOBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.iobase
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBinaryIOBase {}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eio_x2eBinaryIOBase {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "BinaryIOBase(iobase={})", self.iobase)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrIoFileHandle {
    _handle: __SifrIoNativeFileHandle,
    _mode: String,
    _closed: bool,
}
impl __SifrIoFileHandle {
    fn new(handle: __SifrIoNativeFileHandle, mode: String) -> Self {
        let __sifr_field_init_0: __SifrIoNativeFileHandle = handle;
        let __sifr_field_init_1: String = mode;
        let __sifr_field_init_2: bool = false;
        Self {
            _handle: __sifr_field_init_0,
            _mode: __sifr_field_init_1,
            _closed: __sifr_field_init_2,
        }
    }
}
impl __SifrIoFileHandle {
    fn close(&mut self) {
        if self._closed {
            return;
        }
        file_close(&self._handle);
        self._closed = true;
    }
}
impl __SifrIoFileHandle {
    fn closed(&self) -> bool {
        self._closed
    }
}
impl __SifrIoFileHandle {
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(())
    }
}
impl __SifrIoFileHandle {
    fn read(&self) -> Result<String, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        file_read(&self._handle)
    }
}
impl __SifrIoFileHandle {
    fn write(&self, data: &String) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.writable()) {
            return Err(IOError::new("stream is not writable".to_string()));
        }
        file_write(&self._handle, data)
    }
}
impl __SifrIoFileHandle {
    fn readline(&self) -> Result<Option<String>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        file_readline(&self._handle)
    }
}
impl __SifrIoFileHandle {
    fn readlines(&self) -> Result<Vec<String>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        file_readlines(&self._handle)
    }
}
impl __SifrIoFileHandle {
    fn read_bytes(&self) -> Result<Vec<u8>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        file_read_bytes(&self._handle)
    }
}
impl __SifrIoFileHandle {
    fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.writable()) {
            return Err(IOError::new("stream is not writable".to_string()));
        }
        file_write_bytes(&self._handle, data)
    }
}
impl __SifrIoFileHandle {
    fn seek(&self, offset: i64, whence: i64) -> Result<i64, IOError> {
        let _ = offset;
        let _ = whence;
        Err(IOError::new(_unsupported_seek_tell_error()))
    }
}
impl __SifrIoFileHandle {
    fn tell(&self) -> Result<i64, IOError> {
        Err(IOError::new(_unsupported_seek_tell_error()))
    }
}
impl __SifrIoFileHandle {
    fn readable(&self) -> bool {
        _mode_is_readable(&self._mode)
    }
}
impl __SifrIoFileHandle {
    fn writable(&self) -> bool {
        _mode_is_writable(&self._mode)
    }
}
impl __SifrIoFileHandle {
    fn seekable(&self) -> bool {
        false
    }
}
impl __SifrIoFileHandle {
    fn __enter__(&self) -> __SifrIoFileHandle {
        self.clone()
    }
}
impl __SifrIoFileHandle {
    fn __exit__(&mut self) {
        self.close();
    }
}
impl ::std::fmt::Display for __SifrIoFileHandle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(
            f, "FileHandle(_handle={:?}, _mode={}, _closed={})", self._handle, self
            ._mode, self._closed
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrIoBinaryFileHandle {
    _handle: __SifrIoNativeFileHandle,
    _mode: String,
    _closed: bool,
}
impl __SifrIoBinaryFileHandle {
    fn new(handle: __SifrIoNativeFileHandle, mode: String) -> Self {
        let __sifr_field_init_0: __SifrIoNativeFileHandle = handle;
        let __sifr_field_init_1: String = mode;
        let __sifr_field_init_2: bool = false;
        Self {
            _handle: __sifr_field_init_0,
            _mode: __sifr_field_init_1,
            _closed: __sifr_field_init_2,
        }
    }
}
impl __SifrIoBinaryFileHandle {
    fn close(&mut self) {
        if self._closed {
            return;
        }
        file_close(&self._handle);
        self._closed = true;
    }
}
impl __SifrIoBinaryFileHandle {
    fn closed(&self) -> bool {
        self._closed
    }
}
impl __SifrIoBinaryFileHandle {
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(())
    }
}
impl __SifrIoBinaryFileHandle {
    fn read_bytes(&self, size: Option<i64>) -> Result<Vec<u8>, IOError> {
        let _ = size;
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        file_read_bytes(&self._handle)
    }
}
impl __SifrIoBinaryFileHandle {
    fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.writable()) {
            return Err(IOError::new("stream is not writable".to_string()));
        }
        file_write_bytes(&self._handle, data)
    }
}
impl __SifrIoBinaryFileHandle {
    fn seek(&self, offset: i64, whence: i64) -> Result<i64, IOError> {
        let _ = offset;
        let _ = whence;
        Err(IOError::new(_unsupported_seek_tell_error()))
    }
}
impl __SifrIoBinaryFileHandle {
    fn tell(&self) -> Result<i64, IOError> {
        Err(IOError::new(_unsupported_seek_tell_error()))
    }
}
impl __SifrIoBinaryFileHandle {
    fn readable(&self) -> bool {
        _mode_is_readable(&self._mode)
    }
}
impl __SifrIoBinaryFileHandle {
    fn writable(&self) -> bool {
        _mode_is_writable(&self._mode)
    }
}
impl __SifrIoBinaryFileHandle {
    fn seekable(&self) -> bool {
        false
    }
}
impl __SifrIoBinaryFileHandle {
    fn __enter__(&self) -> __SifrIoBinaryFileHandle {
        self.clone()
    }
}
impl __SifrIoBinaryFileHandle {
    fn __exit__(&mut self) {
        self.close();
    }
}
impl ::std::fmt::Display for __SifrIoBinaryFileHandle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(
            f, "BinaryFileHandle(_handle={:?}, _mode={}, _closed={})", self._handle, self
            ._mode, self._closed
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrIoTextFileHandle {
    _binary: __SifrIoBinaryFileHandle,
    _encoding: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
    _decode_errors: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler,
    _encode_errors: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler,
}
impl __SifrIoTextFileHandle {
    fn new(
        binary: __SifrIoBinaryFileHandle,
        enc: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
        decode_errors: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler,
        encode_errors: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler,
    ) -> Self {
        let __sifr_field_init_0: __SifrIoBinaryFileHandle = binary;
        let __sifr_field_init_1: __SifrStdlib_sifr_x2eencoding_x2eEncoding = enc;
        let __sifr_field_init_2: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler = decode_errors;
        let __sifr_field_init_3: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler = encode_errors;
        Self {
            _binary: __sifr_field_init_0,
            _encoding: __sifr_field_init_1,
            _decode_errors: __sifr_field_init_2,
            _encode_errors: __sifr_field_init_3,
        }
    }
}
impl __SifrIoTextFileHandle {
    fn close(&mut self) {
        self._binary.close();
    }
}
impl __SifrIoTextFileHandle {
    fn closed(&self) -> bool {
        self._binary.closed()
    }
}
impl __SifrIoTextFileHandle {
    fn flush(&self) -> Result<(), IOError> {
        self._binary.flush()
    }
}
impl __SifrIoTextFileHandle {
    fn read(&self) -> Result<String, IOError> {
        let __sifr_try_res: Result<
            Result<String, IOError>,
            __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0,
        > = (|| {
            let data: Vec<u8> = (self._binary.read_bytes(None))
                .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                    __e,
                ))?;
            let text: String = (decode(
                &data,
                &self._encoding,
                &Some((self._decode_errors.clone()).clone()),
            ))
                .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a0(
                    __e,
                ))?;
            return Ok(Ok(text));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                match __sifr_try_err {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                        __sifr_try_variant_error,
                    ) => {
                        let e = __sifr_try_variant_error.clone();
                        return Err(IOError::new(e.message));
                    }
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a0(
                        __sifr_try_variant_error,
                    ) => {
                        let e = __sifr_try_variant_error.clone();
                        return Err(
                            IOError::new({
                                let mut __sifr_concat: String = String::with_capacity(
                                    20usize + 0usize,
                                );
                                __sifr_concat.push_str("text decode failed: ");
                                __sifr_concat.push_str((e.message).as_str());
                                __sifr_concat
                            }),
                        );
                    }
                }
            }
        }
    }
}
impl __SifrIoTextFileHandle {
    fn write(&self, text: &String) -> Result<(), IOError> {
        let __sifr_try_res: Result<
            Result<(), IOError>,
            __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0,
        > = (|| {
            let data: Vec<u8> = (encode(
                text,
                &self._encoding,
                &Some((self._encode_errors.clone()).clone()),
            ))
                .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a0(
                    __e,
                ))?;
            let result: () = (self._binary.write_bytes(&data))
                .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                    __e,
                ))?;
            return Ok(Ok(()));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                match __sifr_try_err {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                        __sifr_try_variant_error,
                    ) => {
                        let e = __sifr_try_variant_error.clone();
                        return Err(IOError::new(e.message));
                    }
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a0(
                        __sifr_try_variant_error,
                    ) => {
                        let e = __sifr_try_variant_error.clone();
                        return Err(
                            IOError::new({
                                let mut __sifr_concat: String = String::with_capacity(
                                    20usize + 0usize,
                                );
                                __sifr_concat.push_str("text encode failed: ");
                                __sifr_concat.push_str((e.message).as_str());
                                __sifr_concat
                            }),
                        );
                    }
                }
            }
        }
    }
}
impl __SifrIoTextFileHandle {
    fn readline(&self) -> Result<Option<String>, IOError> {
        Err(
            IOError::new(
                "TextFileHandle.readline is deferred; use read().split(\"\\n\")"
                    .to_string(),
            ),
        )
    }
}
impl __SifrIoTextFileHandle {
    fn readlines(&self) -> Result<Vec<String>, IOError> {
        Err(
            IOError::new(
                "TextFileHandle.readlines is deferred; use read().split(\"\\n\")"
                    .to_string(),
            ),
        )
    }
}
impl __SifrIoTextFileHandle {
    fn readable(&self) -> bool {
        self._binary.readable()
    }
}
impl __SifrIoTextFileHandle {
    fn writable(&self) -> bool {
        self._binary.writable()
    }
}
impl __SifrIoTextFileHandle {
    fn seekable(&self) -> bool {
        self._binary.seekable()
    }
}
impl __SifrIoTextFileHandle {
    fn __enter__(&self) -> __SifrIoTextFileHandle {
        self.clone()
    }
}
impl __SifrIoTextFileHandle {
    fn __exit__(&mut self) {
        self.close();
    }
}
impl ::std::fmt::Display for __SifrIoTextFileHandle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(
            f,
            "TextFileHandle(_binary={}, _encoding={:?}, _decode_errors={:?}, _encode_errors={:?})",
            self._binary, self._encoding, self._decode_errors, self._encode_errors
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eio_x2eTextReader {
    _closed: bool,
}
impl __SifrStdlib_sifr_x2eio_x2eTextReader {
    fn new() -> Self {
        let __sifr_field_init_0: bool = false;
        Self {
            _closed: __sifr_field_init_0,
        }
    }
}
impl __SifrStdlib_sifr_x2eio_x2eTextReader {
    fn read(&self) -> Result<String, IOError> {
        Err(
            IOError::new(
                "TextReader direct construction is unsupported; use open_text"
                    .to_string(),
            ),
        )
    }
}
impl __SifrStdlib_sifr_x2eio_x2eTextReader {
    fn readline(&self) -> Result<Option<String>, IOError> {
        Err(
            IOError::new(
                "TextReader.readline is deferred; use read().split(\"\\n\")".to_string(),
            ),
        )
    }
}
impl __SifrStdlib_sifr_x2eio_x2eTextReader {
    fn readlines(&self) -> Result<Vec<String>, IOError> {
        Err(
            IOError::new(
                "TextReader.readlines is deferred; use read().split(\"\\n\")".to_string(),
            ),
        )
    }
}
impl __SifrStdlib_sifr_x2eio_x2eTextReader {
    fn close(&mut self) {
        self._closed = true;
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eio_x2eTextReader {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "TextReader(_closed={})", self._closed)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eio_x2eTextWriter {
    _closed: bool,
}
impl __SifrStdlib_sifr_x2eio_x2eTextWriter {
    fn new() -> Self {
        let __sifr_field_init_0: bool = false;
        Self {
            _closed: __sifr_field_init_0,
        }
    }
}
impl __SifrStdlib_sifr_x2eio_x2eTextWriter {
    fn write(&self, text: &String) -> Result<(), IOError> {
        let _ = (text).clone();
        Err(
            IOError::new(
                "TextWriter direct construction is unsupported; use open_text"
                    .to_string(),
            ),
        )
    }
}
impl __SifrStdlib_sifr_x2eio_x2eTextWriter {
    fn close(&mut self) {
        self._closed = true;
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eio_x2eTextWriter {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "TextWriter(_closed={})", self._closed)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eio_x2eStringIO {
    _buffer: String,
    _cursor: i64,
    _closed: bool,
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn new(initial: String) -> Self {
        let __sifr_field_init_0: String = {
            let mut __sifr_concat: String = String::with_capacity(
                initial.len() + 0usize,
            );
            __sifr_concat.push_str((initial).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        let __sifr_field_init_1: i64 = 0_i64;
        let __sifr_field_init_2: bool = false;
        Self {
            _buffer: __sifr_field_init_0,
            _cursor: __sifr_field_init_1,
            _closed: __sifr_field_init_2,
        }
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn close(&mut self) {
        self._closed = true;
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn closed(&self) -> bool {
        self._closed
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(())
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn read(&mut self, size: Option<i64>) -> Result<String, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let start: i64 = self._cursor;
        let mut end: i64 = self._buffer.chars().count() as i64;
        if let Some(size) = size {
            let maybe_size: i64 = size;
            if maybe_size >= (0_i64) {
                let requested: i64 = start + maybe_size;
                if requested < end {
                    end = requested;
                }
            }
        }
        let piece: String = {
            let _slice_src = &self._buffer.clone();
            let _slice_len_i64 = _slice_src.chars().count() as i64;
            let _slice_start_i64 = if start < 0 {
                (_slice_len_i64 + start).max(0)
            } else {
                start.min(_slice_len_i64)
            };
            let _slice_stop_i64 = if end < 0 {
                (_slice_len_i64 + end).max(0)
            } else {
                end.min(_slice_len_i64)
            };
            String::from_iter(
                _slice_src
                    .chars()
                    .skip(_slice_start_i64 as usize)
                    .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize),
            )
        };
        self._cursor = end;
        Ok(piece)
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn write(&mut self, data: &String) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let left: String = {
            let _slice_src = &self._buffer.clone();
            let _slice_len_i64 = _slice_src.chars().count() as i64;
            let _slice_start_i64 = 0;
            let _slice_stop_i64 = if self._cursor < 0 {
                (_slice_len_i64 + self._cursor).max(0)
            } else {
                self._cursor.min(_slice_len_i64)
            };
            String::from_iter(
                _slice_src
                    .chars()
                    .skip(_slice_start_i64 as usize)
                    .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize),
            )
        };
        let tail_start: i64 = self._cursor + (data.chars().count() as i64);
        let mut right: String = "".to_string();
        if (tail_start < (self._buffer.chars().count() as i64)) {
            right = {
                let _slice_src = &self._buffer.clone();
                let _slice_len_i64 = _slice_src.chars().count() as i64;
                let _slice_start_i64 = if tail_start < 0 {
                    (_slice_len_i64 + tail_start).max(0)
                } else {
                    tail_start.min(_slice_len_i64)
                };
                let _slice_stop_i64 = _slice_len_i64;
                String::from_iter(
                    _slice_src
                        .chars()
                        .skip(_slice_start_i64 as usize)
                        .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize),
                )
            };
        }
        self._buffer = {
            let mut __sifr_concat: String = String::with_capacity(
                (left.len() + data.len()) + right.len(),
            );
            __sifr_concat.push_str((left).as_str());
            __sifr_concat.push_str((data).as_str());
            __sifr_concat.push_str((right).as_str());
            __sifr_concat
        };
        self._cursor += data.chars().count() as i64;
        Ok(())
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn getvalue(&self) -> String {
        self._buffer.clone()
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn seek(&mut self, offset: i64, whence: i64) -> Result<i64, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let mut origin: i64 = 0_i64;
        if whence == (0_i64) {
            origin = 0_i64;
        } else {
            if whence == (1_i64) {
                origin = self._cursor;
            } else {
                if whence == (2_i64) {
                    origin = self._buffer.chars().count() as i64;
                } else {
                    return Err(IOError::new(_invalid_whence_error(whence)));
                }
            }
        }
        let mut next_pos: i64 = origin + offset;
        if next_pos < (0_i64) {
            return Err(IOError::new(_negative_seek_error(next_pos)));
        }
        let end: i64 = self._buffer.chars().count() as i64;
        if next_pos > end {
            next_pos = end;
        }
        self._cursor = next_pos;
        Ok(self._cursor)
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn tell(&self) -> Result<i64, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(self._cursor)
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn readable(&self) -> bool {
        !(self._closed)
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn writable(&self) -> bool {
        !(self._closed)
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn seekable(&self) -> bool {
        !(self._closed)
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(
            f, "StringIO(_buffer={}, _cursor={}, _closed={})", self._buffer, self
            ._cursor, self._closed
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eio_x2eBytesIO {
    _buffer: Vec<u8>,
    _cursor: i64,
    _closed: bool,
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn new(initial: Vec<u8>) -> Self {
        let __sifr_field_init_0: Vec<u8> = initial;
        let __sifr_field_init_1: i64 = 0_i64;
        let __sifr_field_init_2: bool = false;
        Self {
            _buffer: __sifr_field_init_0,
            _cursor: __sifr_field_init_1,
            _closed: __sifr_field_init_2,
        }
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn close(&mut self) {
        self._closed = true;
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn closed(&self) -> bool {
        self._closed
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(())
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn read_bytes(&mut self, size: Option<i64>) -> Result<Vec<u8>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let start: i64 = self._cursor;
        let mut end: i64 = self._buffer.len() as i64;
        if let Some(size) = size {
            let maybe_size: i64 = size;
            if maybe_size >= (0_i64) {
                let requested: i64 = start + maybe_size;
                if requested < end {
                    end = requested;
                }
            }
        }
        let chunk: Vec<u8> = {
            let _slice_src = &self._buffer.clone();
            let _slice_len_i64 = _slice_src.len() as i64;
            let _slice_start_i64 = if start < 0 {
                (_slice_len_i64 + start).max(0)
            } else {
                start.min(_slice_len_i64)
            };
            let _slice_stop_i64 = if end < 0 {
                (_slice_len_i64 + end).max(0)
            } else {
                end.min(_slice_len_i64)
            };
            Vec::from_iter(
                _slice_src
                    .iter()
                    .skip(_slice_start_i64 as usize)
                    .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                    .cloned(),
            )
        };
        self._cursor = end;
        Ok(chunk)
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn write_bytes(&mut self, data: &Vec<u8>) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let left: Vec<u8> = {
            let _slice_src = &self._buffer.clone();
            let _slice_len_i64 = _slice_src.len() as i64;
            let _slice_start_i64 = 0;
            let _slice_stop_i64 = if self._cursor < 0 {
                (_slice_len_i64 + self._cursor).max(0)
            } else {
                self._cursor.min(_slice_len_i64)
            };
            Vec::from_iter(
                _slice_src
                    .iter()
                    .skip(_slice_start_i64 as usize)
                    .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                    .cloned(),
            )
        };
        let tail_start: i64 = self._cursor + (data.len() as i64);
        let mut right: Vec<u8> = vec![];
        if (tail_start < (self._buffer.len() as i64)) {
            right = {
                let _slice_src = &self._buffer.clone();
                let _slice_len_i64 = _slice_src.len() as i64;
                let _slice_start_i64 = if tail_start < 0 {
                    (_slice_len_i64 + tail_start).max(0)
                } else {
                    tail_start.min(_slice_len_i64)
                };
                let _slice_stop_i64 = _slice_len_i64;
                Vec::from_iter(
                    _slice_src
                        .iter()
                        .skip(_slice_start_i64 as usize)
                        .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                        .cloned(),
                )
            };
        }
        self._buffer = {
            let mut __v = ({
                let mut __v = (left).clone();
                __v.extend((data).iter().cloned());
                __v
            })
                .clone();
            __v.extend((right).iter().cloned());
            __v
        };
        self._cursor += data.len() as i64;
        Ok(())
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn getvalue(&self) -> Result<Vec<u8>, IOError> {
        Ok(self._buffer.clone())
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn seek(&mut self, offset: i64, whence: i64) -> Result<i64, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let mut origin: i64 = 0_i64;
        if whence == (0_i64) {
            origin = 0_i64;
        } else {
            if whence == (1_i64) {
                origin = self._cursor;
            } else {
                if whence == (2_i64) {
                    origin = self._buffer.len() as i64;
                } else {
                    return Err(IOError::new(_invalid_whence_error(whence)));
                }
            }
        }
        let mut next_pos: i64 = origin + offset;
        if next_pos < (0_i64) {
            return Err(IOError::new(_negative_seek_error(next_pos)));
        }
        let end: i64 = self._buffer.len() as i64;
        if next_pos > end {
            next_pos = end;
        }
        self._cursor = next_pos;
        Ok(self._cursor)
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn tell(&self) -> Result<i64, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(self._cursor)
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn readable(&self) -> bool {
        !(self._closed)
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn writable(&self) -> bool {
        !(self._closed)
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn seekable(&self) -> bool {
        !(self._closed)
    }
}
fn _closed_stream_error() -> String {
    "I/O operation on closed stream".to_string()
}
fn _invalid_whence_error(whence: i64) -> String {
    {
        let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("invalid whence: ");
        __sifr_concat.push_str((format!("{}", whence)).as_str());
        __sifr_concat
    }
}
fn _negative_seek_error(offset: i64) -> String {
    {
        let mut __sifr_concat: String = String::with_capacity(24usize + 0usize);
        __sifr_concat.push_str("negative seek position: ");
        __sifr_concat.push_str((format!("{}", offset)).as_str());
        __sifr_concat
    }
}
fn _unsupported_seek_tell_error() -> String {
    "seek/tell is unsupported for this stream".to_string()
}
fn _mode_is_readable(mode: &String) -> bool {
    mode.contains(&"r".to_string()) || mode.contains(&"+".to_string())
}
fn _mode_is_writable(mode: &String) -> bool {
    (mode.contains(&"w".to_string()) || mode.contains(&"a".to_string()))
        || mode.contains(&"+".to_string())
}
fn _text_binary_mode(mode: &String) -> Result<String, IOError> {
    if mode.contains(&"b".to_string()) {
        return Err(
            IOError::new("open_text requires a text mode without \'b\'".to_string()),
        );
    }
    if ((mode).as_str() == "r") || ((mode).as_str() == "rt") {
        return Ok("rb".to_string());
    }
    if ((mode).as_str() == "w") || ((mode).as_str() == "wt") {
        return Ok("wb".to_string());
    }
    if ((mode).as_str() == "a") || ((mode).as_str() == "at") {
        return Ok("ab".to_string());
    }
    Err(
        IOError::new({
            let mut __sifr_concat: String = String::with_capacity(19usize + mode.len());
            __sifr_concat.push_str("invalid text mode: ");
            __sifr_concat.push_str((mode).as_str());
            __sifr_concat
        }),
    )
}
fn _text_encoding_or_default(
    enc: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncoding>,
) -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    if let Some(enc) = enc.as_ref() {
        return __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(
            format!("{}{}", enc.label.clone(), ""),
        );
    }
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new("utf-8".to_string())
}
fn _decode_errors_or_default(
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
) -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    if let Some(errors) = errors.as_ref() {
        return __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
            format!("{}{}", errors.name.clone(), ""),
        );
    }
    strict_decode_handler()
}
fn _encode_errors_from_decode_errors(
    errors: &__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler,
) -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        format!("{}{}", errors.name.clone(), ""),
    )
}
fn open(path: &String, mode: &String) -> Result<__SifrIoFileHandle, IOError> {
    let __sifr_try_res: Result<Result<__SifrIoFileHandle, IOError>, IOError> = (|| {
        let handle: __SifrIoNativeFileHandle = open_file(path, mode)?;
        return Ok(Ok(__SifrIoFileHandle::new(handle, (mode).clone())));
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
fn open_binary(
    path: &String,
    mode: &String,
) -> Result<__SifrIoBinaryFileHandle, IOError> {
    if !(mode.contains(&"b".to_string())) {
        return Err(IOError::new("open_binary requires binary mode".to_string()));
    }
    let __sifr_try_res: Result<Result<__SifrIoBinaryFileHandle, IOError>, IOError> = (|| {
        let handle: __SifrIoNativeFileHandle = open_file(path, mode)?;
        return Ok(Ok(__SifrIoBinaryFileHandle::new(handle, (mode).clone())));
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
fn open_text(
    path: &String,
    mode: &String,
    encoding: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncoding>,
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
) -> Result<__SifrIoTextFileHandle, IOError> {
    let __sifr_try_res: Result<Result<__SifrIoTextFileHandle, IOError>, IOError> = (|| {
        let binary_mode: String = _text_binary_mode(mode)?;
        let text_encoding: __SifrStdlib_sifr_x2eencoding_x2eEncoding = _text_encoding_or_default(
            encoding,
        );
        let decode_errors: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler = _decode_errors_or_default(
            errors,
        );
        let encode_errors: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler = _encode_errors_from_decode_errors(
            &decode_errors,
        );
        let binary: __SifrIoBinaryFileHandle = open_binary(path, &binary_mode)?;
        return Ok(
            Ok(
                __SifrIoTextFileHandle::new(
                    binary,
                    text_encoding,
                    decode_errors,
                    encode_errors,
                ),
            ),
        );
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

// --- stdlib: sifr.csv ---
const QUOTE_ALL: i64 = 1_i64;
const QUOTE_NONNUMERIC: i64 = 2_i64;
const QUOTE_NONE: i64 = 3_i64;
const QUOTE_STRINGS: i64 = 4_i64;
const QUOTE_NOTNULL: i64 = 5_i64;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2ecsv_x2eDialect {
    delimiter: String,
    quotechar: String,
    escapechar: String,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: String,
    quoting: i64,
}
impl __SifrStdlib_sifr_x2ecsv_x2eDialect {
    fn new(
        delimiter: String,
        quotechar: String,
        escapechar: String,
        doublequote: bool,
        skipinitialspace: bool,
        lineterminator: String,
        quoting: i64,
    ) -> Self {
        let mut resolved_quoting: i64 = quoting;
        _validate_char(&"delimiter".to_string(), &delimiter);
        if quotechar != "" {
            _validate_char(&"quotechar".to_string(), &quotechar);
        }
        if escapechar != "" {
            _validate_char(&"escapechar".to_string(), &escapechar);
        }
        if (quotechar == "") && (resolved_quoting != QUOTE_NONE) {
            resolved_quoting = QUOTE_NONE;
        }
        let __sifr_field_init_0: String = delimiter;
        let __sifr_field_init_1: String = quotechar;
        let __sifr_field_init_2: String = escapechar;
        let __sifr_field_init_3: bool = doublequote;
        let __sifr_field_init_4: bool = skipinitialspace;
        let __sifr_field_init_5: String = lineterminator;
        let __sifr_field_init_6: i64 = resolved_quoting;
        Self {
            delimiter: __sifr_field_init_0,
            quotechar: __sifr_field_init_1,
            escapechar: __sifr_field_init_2,
            doublequote: __sifr_field_init_3,
            skipinitialspace: __sifr_field_init_4,
            lineterminator: __sifr_field_init_5,
            quoting: __sifr_field_init_6,
        }
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2eDialect {}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2ecsv_x2eDialect {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(
            f,
            "Dialect(delimiter={}, quotechar={}, escapechar={}, doublequote={}, skipinitialspace={}, lineterminator={}, quoting={})",
            self.delimiter, self.quotechar, self.escapechar, self.doublequote, self
            .skipinitialspace, self.lineterminator, self.quoting
        )
    }
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2ecsv_x2ereader {
    _rows: Vec<Vec<String>>,
    _pos: i64,
    dialect: __SifrStdlib_sifr_x2ecsv_x2eDialect,
}
impl __SifrStdlib_sifr_x2ecsv_x2ereader {
    fn new(
        text: String,
        dialect: Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
        delimiter: String,
        quotechar: String,
        escapechar: String,
        doublequote: bool,
        skipinitialspace: bool,
        quoting: i64,
    ) -> Self {
        let resolved_dialect: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
            &dialect,
            &delimiter,
            &quotechar,
            &escapechar,
            doublequote,
            skipinitialspace,
            &"\n".to_string(),
            quoting,
        );
        let rows: Vec<Vec<String>> = parse_csv(
            &text,
            &None,
            &format!("{}{}", resolved_dialect.delimiter, ""),
            &format!("{}{}", resolved_dialect.quotechar, ""),
            &format!("{}{}", resolved_dialect.escapechar, ""),
            resolved_dialect.doublequote,
            resolved_dialect.skipinitialspace,
            resolved_dialect.quoting,
        );
        let __sifr_field_init_0: __SifrStdlib_sifr_x2ecsv_x2eDialect = resolved_dialect;
        let __sifr_field_init_1: Vec<Vec<String>> = rows;
        let __sifr_field_init_2: i64 = 0_i64;
        Self {
            dialect: __sifr_field_init_0,
            _rows: __sifr_field_init_1,
            _pos: __sifr_field_init_2,
        }
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2ereader {
    fn __next__(&mut self) -> Option<Vec<String>> {
        if (self._pos >= (self._rows.len() as i64)) {
            return None;
        }
        let row: Option<Vec<String>> = {
            let __sifr_index_list = &self._rows;
            let __sifr_index_i = self._pos;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        self._pos += 1_i64;
        let Some(row) = row else {
            return None;
        };
        let mut result: Vec<String> = vec![];
        for field in row.iter().cloned() {
            result.push(format!("{}{}", field, ""));
        }
        Some(result)
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2ereader {
    fn rows(&self) -> Vec<Vec<String>> {
        let mut result: Vec<Vec<String>> = vec![];
        for row in self._rows.clone().iter().cloned() {
            let mut copied: Vec<String> = vec![];
            for field in row.iter().cloned() {
                copied.push(format!("{}{}", field, ""));
            }
            result.push(copied.clone());
        }
        result
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2ereader {
    fn line_num(&self) -> i64 {
        self._pos
    }
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2ecsv_x2ewriter {
    _rows: Vec<Vec<String>>,
    dialect: __SifrStdlib_sifr_x2ecsv_x2eDialect,
}
impl __SifrStdlib_sifr_x2ecsv_x2ewriter {
    fn new(
        dialect: Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
        delimiter: String,
        quotechar: String,
        escapechar: String,
        doublequote: bool,
        skipinitialspace: bool,
        lineterminator: String,
        quoting: i64,
    ) -> Self {
        let resolved_dialect: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
            &dialect,
            &delimiter,
            &quotechar,
            &escapechar,
            doublequote,
            skipinitialspace,
            &lineterminator,
            quoting,
        );
        let __sifr_field_init_0: __SifrStdlib_sifr_x2ecsv_x2eDialect = resolved_dialect;
        let __sifr_field_init_1: Vec<Vec<String>> = vec![];
        Self {
            dialect: __sifr_field_init_0,
            _rows: __sifr_field_init_1,
        }
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2ewriter {
    fn writerow(&mut self, row: &Vec<String>) {
        let mut copied: Vec<String> = vec![];
        for value in row.iter().cloned() {
            copied.push(value.clone());
        }
        self._rows.push(copied.clone());
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2ewriter {
    fn writerows(&mut self, rows: &Vec<Vec<String>>) {
        for row in rows.iter().cloned() {
            let mut copied: Vec<String> = vec![];
            for value in row.iter().cloned() {
                copied.push(format!("{}{}", value, ""));
            }
            self._rows.push(copied.clone());
        }
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2ewriter {
    fn getvalue(&self) -> String {
        format_csv(
            &self._rows,
            &Some((self.dialect.clone()).clone()),
            &",".to_string(),
            &"\"".to_string(),
            &"".to_string(),
            true,
            false,
            &"\n".to_string(),
            0_i64,
        )
    }
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2ecsv_x2eDictReader {
    _fieldnames: Vec<String>,
    _rows: Vec<Vec<String>>,
    _pos: i64,
    restkey: String,
    restval: String,
    dialect: __SifrStdlib_sifr_x2ecsv_x2eDialect,
}
impl __SifrStdlib_sifr_x2ecsv_x2eDictReader {
    fn new(
        text: String,
        fieldnames: Option<Vec<String>>,
        restkey: String,
        restval: String,
        dialect: Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
        delimiter: String,
        quotechar: String,
        escapechar: String,
        doublequote: bool,
        skipinitialspace: bool,
        quoting: i64,
    ) -> Self {
        let resolved_dialect: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
            &dialect,
            &delimiter,
            &quotechar,
            &escapechar,
            doublequote,
            skipinitialspace,
            &"\n".to_string(),
            quoting,
        );
        let all_rows: Vec<Vec<String>> = parse_csv(
            &text,
            &None,
            &format!("{}{}", resolved_dialect.delimiter, ""),
            &format!("{}{}", resolved_dialect.quotechar, ""),
            &format!("{}{}", resolved_dialect.escapechar, ""),
            resolved_dialect.doublequote,
            resolved_dialect.skipinitialspace,
            resolved_dialect.quoting,
        );
        let mut fieldnames_data: Vec<String> = vec![];
        let mut rows_data: Vec<Vec<String>> = vec![];
        if let Some(fieldnames) = fieldnames {
            for field in fieldnames.iter().cloned() {
                fieldnames_data.push(format!("{}{}", field, ""));
            }
            for row in all_rows.iter().cloned() {
                let mut copied_row: Vec<String> = vec![];
                for value in row.iter().cloned() {
                    copied_row.push(format!("{}{}", value, ""));
                }
                rows_data.push(copied_row.clone());
            }
        } else {
            for (index, row) in Box::new(
                (all_rows)
                    .iter()
                    .cloned()
                    .enumerate()
                    .map(|__pair| ((__pair.0 as i64) + 0, __pair.1)),
            ) {
                if index == (0_i64) {
                    for field in row.iter().cloned() {
                        fieldnames_data.push(format!("{}{}", field, ""));
                    }
                } else {
                    let mut copied_row2: Vec<String> = vec![];
                    for value in row.iter().cloned() {
                        copied_row2.push(format!("{}{}", value, ""));
                    }
                    rows_data.push(copied_row2.clone());
                }
            }
        }
        let __sifr_field_init_0: __SifrStdlib_sifr_x2ecsv_x2eDialect = resolved_dialect;
        let __sifr_field_init_1: String = restkey;
        let __sifr_field_init_2: String = restval;
        let __sifr_field_init_3: i64 = 0_i64;
        let __sifr_field_init_4: Vec<String> = fieldnames_data;
        let __sifr_field_init_5: Vec<Vec<String>> = rows_data;
        Self {
            dialect: __sifr_field_init_0,
            restkey: __sifr_field_init_1,
            restval: __sifr_field_init_2,
            _pos: __sifr_field_init_3,
            _fieldnames: __sifr_field_init_4,
            _rows: __sifr_field_init_5,
        }
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2eDictReader {
    fn fieldnames(&self) -> Vec<String> {
        let mut copied: Vec<String> = vec![];
        for field in self._fieldnames.clone().iter().cloned() {
            copied.push(format!("{}{}", field, ""));
        }
        copied
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2eDictReader {
    fn __next__(&mut self) -> Option<HashMap<String, String>> {
        while (self._pos < (self._rows.len() as i64)) {
            let row: Option<Vec<String>> = {
                let __sifr_index_list = &self._rows;
                let __sifr_index_i = self._pos;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            self._pos += 1_i64;
            let Some(row) = row else {
                return None;
            };
            if ((row.len() as i64) == (0_i64)) {
                continue;
            }
            return Some(
                _dict_reader_row(&self._fieldnames, &row, &self.restkey, &self.restval),
            );
        }
        None
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2eDictReader {
    fn rows(&self) -> Vec<HashMap<String, String>> {
        let mut result: Vec<HashMap<String, String>> = vec![];
        for row in self._rows.clone().iter().cloned() {
            if ((row.len() as i64) == (0_i64)) {
                continue;
            }
            result
                .push(
                    _dict_reader_row(
                        &self._fieldnames,
                        &row,
                        &self.restkey,
                        &self.restval,
                    ),
                );
        }
        result
    }
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2ecsv_x2eDictWriter {
    fieldnames: Vec<String>,
    restval: String,
    extrasaction: String,
    _writer: __SifrStdlib_sifr_x2ecsv_x2ewriter,
}
impl __SifrStdlib_sifr_x2ecsv_x2eDictWriter {
    fn new(
        fieldnames: Vec<String>,
        restval: String,
        extrasaction: String,
        dialect: Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
        delimiter: String,
        quotechar: String,
        escapechar: String,
        doublequote: bool,
        skipinitialspace: bool,
        lineterminator: String,
        quoting: i64,
    ) -> Self {
        let mut fieldnames_data: Vec<String> = vec![];
        for field in fieldnames.iter().cloned() {
            fieldnames_data.push(format!("{}{}", field, ""));
        }
        let mut action: String = extrasaction.to_lowercase();
        if (action != "raise") && (action != "ignore") {
            action = "raise".to_string();
        }
        let writer_value: __SifrStdlib_sifr_x2ecsv_x2ewriter = __SifrStdlib_sifr_x2ecsv_x2ewriter::new(
            dialect,
            delimiter,
            quotechar,
            escapechar,
            doublequote,
            skipinitialspace,
            lineterminator,
            quoting,
        );
        let __sifr_field_init_0: Vec<String> = fieldnames_data;
        let __sifr_field_init_1: String = restval;
        let __sifr_field_init_2: String = action;
        let __sifr_field_init_3: __SifrStdlib_sifr_x2ecsv_x2ewriter = writer_value;
        Self {
            fieldnames: __sifr_field_init_0,
            restval: __sifr_field_init_1,
            extrasaction: __sifr_field_init_2,
            _writer: __sifr_field_init_3,
        }
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2eDictWriter {
    fn writeheader(&mut self) {
        let mut current_writer: __SifrStdlib_sifr_x2ecsv_x2ewriter = self
            ._writer
            .clone();
        current_writer.writerow(&self.fieldnames.clone());
        self._writer = current_writer;
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2eDictWriter {
    fn writerow(&mut self, row: &HashMap<String, String>) {
        let mut ordered: Vec<String> = vec![];
        for fieldname in self.fieldnames.clone().iter().cloned() {
            if row.contains_key(&fieldname) {
                ordered.push(_dict_value_at(row, &fieldname));
            } else {
                ordered.push(self.restval.clone());
            }
        }
        let mut current_writer: __SifrStdlib_sifr_x2ecsv_x2ewriter = self
            ._writer
            .clone();
        current_writer.writerow(&ordered);
        self._writer = current_writer;
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2eDictWriter {
    fn writerows(&mut self, rows: &Vec<HashMap<String, String>>) {
        let mut current_writer: __SifrStdlib_sifr_x2ecsv_x2ewriter = self
            ._writer
            .clone();
        for row in rows.iter().cloned() {
            let mut ordered: Vec<String> = vec![];
            for fieldname in self.fieldnames.clone().iter().cloned() {
                if row.contains_key(&fieldname) {
                    ordered.push(_dict_value_at(&row, &fieldname));
                } else {
                    ordered.push(self.restval.clone());
                }
            }
            current_writer.writerow(&ordered);
        }
        self._writer = current_writer;
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2eDictWriter {
    fn getvalue(&self) -> String {
        self._writer.getvalue()
    }
}
fn _copy_dialect(
    dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect,
) -> __SifrStdlib_sifr_x2ecsv_x2eDialect {
    __SifrStdlib_sifr_x2ecsv_x2eDialect::new(
        format!("{}{}", dialect.delimiter.clone(), ""),
        format!("{}{}", dialect.quotechar.clone(), ""),
        format!("{}{}", dialect.escapechar.clone(), ""),
        dialect.doublequote,
        dialect.skipinitialspace,
        format!("{}{}", dialect.lineterminator.clone(), ""),
        dialect.quoting,
    )
}
fn _validate_char(name: &String, value: &String) {
    let _ = (name).clone();
    let _ = (value).clone();
}
fn _resolve_dialect(
    dialect: &Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: &String,
    quoting: i64,
) -> __SifrStdlib_sifr_x2ecsv_x2eDialect {
    if let Some(dialect) = dialect.as_ref() {
        return _copy_dialect(dialect);
    }
    __SifrStdlib_sifr_x2ecsv_x2eDialect::new(
        (delimiter).clone(),
        (quotechar).clone(),
        (escapechar).clone(),
        doublequote,
        skipinitialspace,
        (lineterminator).clone(),
        quoting,
    )
}
fn _quotechar_value(dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect) -> String {
    let quotechar: String = {
        let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
        __sifr_concat.push_str((dialect.quotechar.clone()).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if quotechar == "" {
        return "\"".to_string();
    }
    quotechar
}
fn _append_field(row: &mut Vec<String>, field: String) {
    row.push(format!("{}{}", field, ""));
}
fn _append_row(rows: &mut Vec<Vec<String>>, row: Vec<String>) {
    rows.push(row.clone());
}
fn _char_at(text: &String, index: i64) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    if (index < (0_i64)) || (index >= (__sifr_chars_text.len() as i64)) {
        return "".to_string();
    }
    let ch: Option<String> = Some({
        let Some(__indexed_char) = __sifr_chars_text
            .get(index as usize)
            .map(|c| c.to_string()) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char
    });
    let Some(ch) = ch else {
        return "".to_string();
    };
    ch
}
fn _list_value_at(values: &Vec<String>, index: i64) -> String {
    if (index < (0_i64)) || (index >= (values.len() as i64)) {
        return "".to_string();
    }
    for (current_index, value) in Box::new(
        (values)
            .iter()
            .cloned()
            .enumerate()
            .map(|__pair| ((__pair.0 as i64) + 0, __pair.1)),
    ) {
        if current_index == index {
            return {
                let mut __sifr_concat: String = String::with_capacity(
                    value.len() + 0usize,
                );
                __sifr_concat.push_str((value).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
        }
    }
    "".to_string()
}
fn _dict_value_at(values: &HashMap<String, String>, key: &String) -> String {
    for item_key in values.keys().cloned() {
        if item_key != *key {
            continue;
        }
        let value: Option<String> = values.get(&item_key).cloned();
        let Some(value) = value else {
            return "".to_string();
        };
        return {
            let mut __sifr_concat: String = String::with_capacity(value.len() + 0usize);
            __sifr_concat.push_str((value).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    "".to_string()
}
fn _first_char(text: &String) -> String {
    _char_at(text, 0_i64)
}
fn _last_char(text: &String) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    _char_at(text, (text.chars().count() as i64) - (1_i64))
}
fn parse_csv(
    text: &String,
    dialect: &Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    quoting: i64,
) -> Vec<Vec<String>> {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let resolved: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
        dialect,
        delimiter,
        quotechar,
        escapechar,
        doublequote,
        skipinitialspace,
        &"\n".to_string(),
        quoting,
    );
    let mut rows: Vec<Vec<String>> = vec![];
    let mut row: Vec<String> = vec![];
    let mut field: String = "".to_string();
    let mut in_quotes: bool = false;
    let mut field_started: bool = false;
    let mut i: i64 = 0_i64;
    while (i < (__sifr_chars_text.len() as i64)) {
        let ch_value: String = _char_at(text, i);
        if in_quotes {
            if (resolved.escapechar != "") && (ch_value == resolved.escapechar) {
                if ((i + (1_i64)) < (__sifr_chars_text.len() as i64)) {
                    let escaped_value: String = _char_at(text, i + (1_i64));
                    field.push_str((escaped_value).as_str());
                    i += 2_i64;
                    continue;
                }
                field.push_str((ch_value).as_str());
                i += 1_i64;
                continue;
            }
            if (resolved.quotechar != "") && (ch_value == resolved.quotechar) {
                let quotechar: String = _quotechar_value(&resolved);
                if (resolved.doublequote
                    && ((i + (1_i64)) < (__sifr_chars_text.len() as i64)))
                    && (_char_at(text, i + (1_i64)) == quotechar.clone())
                {
                    field.push_str((quotechar).as_str());
                    i += 2_i64;
                    continue;
                }
                in_quotes = false;
                i += 1_i64;
                continue;
            }
            field.push_str((ch_value).as_str());
            i += 1_i64;
            continue;
        }
        if (!field_started && resolved.skipinitialspace) && (ch_value == " ") {
            i += 1_i64;
            continue;
        }
        if (resolved.escapechar != "") && (ch_value == resolved.escapechar) {
            if ((i + (1_i64)) < (__sifr_chars_text.len() as i64)) {
                let escaped_plain_value: String = _char_at(text, i + (1_i64));
                field.push_str((escaped_plain_value).as_str());
                field_started = true;
                i += 2_i64;
                continue;
            }
            field.push_str((ch_value).as_str());
            field_started = true;
            i += 1_i64;
            continue;
        }
        if (resolved.quoting != QUOTE_NONE) && (resolved.quotechar != "") {
            let quotechar2: String = _quotechar_value(&resolved);
            if ch_value == quotechar2 {
                in_quotes = true;
                field_started = true;
                i += 1_i64;
                continue;
            }
        }
        if (ch_value == resolved.delimiter) {
            _append_field(&mut row, field);
            field = "".to_string();
            field_started = false;
            i += 1_i64;
            continue;
        }
        if (ch_value == "\n") || (ch_value == "\r") {
            if ((ch_value == "\r") && ((i + (1_i64)) < (__sifr_chars_text.len() as i64)))
                && (_char_at(text, i + (1_i64)) == "\n")
            {
                i += 1_i64;
            }
            if ((row.len() as i64) == (0_i64)) && (field == "") {
                _append_row(&mut rows, vec![]);
            } else {
                _append_field(&mut row, field);
                _append_row(&mut rows, row);
            }
            row = vec![];
            field = "".to_string();
            field_started = false;
            i += 1_i64;
            continue;
        }
        field.push_str((ch_value).as_str());
        field_started = true;
        i += 1_i64;
    }
    if in_quotes {
        in_quotes = false;
    }
    if ((row.len() as i64) > (0_i64)) || (field != "") {
        _append_field(&mut row, field);
        _append_row(&mut rows, row);
    }
    rows
}
fn _needs_quote(field: &String, dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect) -> bool {
    let __sifr_chars_field: Vec<char> = field.chars().collect::<Vec<char>>();
    if (dialect.quoting == QUOTE_ALL) {
        return true;
    }
    if (dialect.quoting == QUOTE_NONNUMERIC) {
        return true;
    }
    if (dialect.quoting == QUOTE_STRINGS) {
        return true;
    }
    if (dialect.quoting == QUOTE_NOTNULL) {
        return true;
    }
    if (dialect.quoting == QUOTE_NONE) {
        return false;
    }
    if (field).contains((dialect.delimiter.clone()).as_str()) {
        return true;
    }
    if field.contains(&"\n".to_string()) || field.contains(&"\r".to_string()) {
        return true;
    }
    if (dialect.quotechar.clone() != "") {
        let quotechar: String = _quotechar_value(dialect);
        if field.contains(&quotechar) {
            return true;
        }
    }
    if ((__sifr_chars_field.len() as i64) > (0_i64)) {
        let first: String = _first_char(field);
        let last: String = _last_char(field);
        if first == " " {
            return true;
        }
        if last == " " {
            return true;
        }
    }
    false
}
fn _quote_field(
    field: &String,
    dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect,
) -> String {
    let quotechar: String = _quotechar_value(dialect);
    let mut escaped: String = {
        let mut __sifr_concat: String = String::with_capacity(field.len() + 0usize);
        __sifr_concat.push_str((field).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if escaped.contains(&quotechar) {
        if dialect.doublequote {
            escaped = escaped
                .replace(&quotechar, &format!("{}{}", quotechar, quotechar));
        } else {
            if (dialect.escapechar.clone() != "") {
                let escapechar_value: String = {
                    let mut __sifr_concat: String = String::with_capacity(
                        0usize + 0usize,
                    );
                    __sifr_concat.push_str((dialect.escapechar.clone()).as_str());
                    __sifr_concat.push_str("");
                    __sifr_concat
                };
                escaped = escaped
                    .replace(&quotechar, &format!("{}{}", escapechar_value, quotechar));
            } else {
                escaped = escaped
                    .replace(&quotechar, &format!("{}{}", quotechar, quotechar));
            }
        }
    }
    {
        let mut __sifr_concat: String = String::with_capacity(
            (quotechar.len() + escaped.len()) + quotechar.len(),
        );
        __sifr_concat.push_str((quotechar).as_str());
        __sifr_concat.push_str((escaped).as_str());
        __sifr_concat.push_str((quotechar).as_str());
        __sifr_concat
    }
}
fn _escape_unquoted_field(
    field: &String,
    dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect,
) -> String {
    let mut result: String = {
        let mut __sifr_concat: String = String::with_capacity(field.len() + 0usize);
        __sifr_concat.push_str((field).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if (result).contains((dialect.delimiter.clone()).as_str()) {
        if (dialect.escapechar.clone() != "") {
            result = result
                .replace(
                    &dialect.delimiter.clone(),
                    &format!(
                        "{}{}", dialect.escapechar.clone(), dialect.delimiter.clone()
                    ),
                );
        }
    }
    if result.contains(&"\n".to_string()) {
        if (dialect.escapechar.clone() != "") {
            result = result
                .replace('\n', &format!("{}{}", dialect.escapechar.clone(), "\n"));
        }
    }
    if result.contains(&"\r".to_string()) {
        if (dialect.escapechar.clone() != "") {
            result = result
                .replace('\r', &format!("{}{}", dialect.escapechar.clone(), "\r"));
        }
    }
    if (dialect.quotechar.clone() != "") {
        let quotechar2: String = _quotechar_value(dialect);
        if result.contains(&quotechar2) {
            if (dialect.escapechar.clone() != "") {
                result = result
                    .replace(
                        &quotechar2,
                        &format!("{}{}", dialect.escapechar.clone(), quotechar2),
                    );
            } else {
                result = result
                    .replace(&quotechar2, &format!("{}{}", quotechar2, quotechar2));
            }
        }
    }
    result
}
fn format_row(
    fields: &Vec<String>,
    dialect: &Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    quoting: i64,
) -> String {
    let resolved: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
        dialect,
        delimiter,
        quotechar,
        escapechar,
        doublequote,
        skipinitialspace,
        &"\n".to_string(),
        quoting,
    );
    let mut parts: Vec<String> = vec![];
    for field in fields.iter().cloned() {
        if _needs_quote(&field, &resolved) {
            parts.push(_quote_field(&field, &resolved));
        } else {
            parts.push(_escape_unquoted_field(&field, &resolved));
        }
    }
    parts.join(&resolved.delimiter)
}
fn format_csv(
    rows: &Vec<Vec<String>>,
    dialect: &Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: &String,
    quoting: i64,
) -> String {
    let resolved: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
        dialect,
        delimiter,
        quotechar,
        escapechar,
        doublequote,
        skipinitialspace,
        lineterminator,
        quoting,
    );
    let mut rendered: Vec<String> = vec![];
    let resolved_delimiter: String = {
        let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
        __sifr_concat.push_str((resolved.delimiter).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    let resolved_quotechar: String = {
        let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
        __sifr_concat.push_str((resolved.quotechar).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    let resolved_escapechar: String = {
        let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
        __sifr_concat.push_str((resolved.escapechar).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    let resolved_lineterminator: String = {
        let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
        __sifr_concat.push_str((resolved.lineterminator).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    for row in rows.iter().cloned() {
        rendered
            .push(
                format_row(
                    &row,
                    &None,
                    &resolved_delimiter,
                    &resolved_quotechar,
                    &resolved_escapechar,
                    resolved.doublequote,
                    resolved.skipinitialspace,
                    resolved.quoting,
                ),
            );
    }
    rendered.join(&resolved_lineterminator)
}
fn _dict_reader_row(
    fieldnames: &Vec<String>,
    row: &Vec<String>,
    restkey: &String,
    restval: &String,
) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::from([]);
    for (i, key) in Box::new(
        (fieldnames)
            .iter()
            .cloned()
            .enumerate()
            .map(|__pair| ((__pair.0 as i64) + 0, __pair.1)),
    ) {
        if (i < (row.len() as i64)) {
            {
                let __assign_key = key.clone();
                let __assign_value = _list_value_at(row, i);
                result.insert(__assign_key, __assign_value);
            }
        } else {
            result
                .insert(
                    key.clone(),
                    {
                        let mut __sifr_concat: String = String::with_capacity(
                            restval.len() + 0usize,
                        );
                        __sifr_concat.push_str((restval).as_str());
                        __sifr_concat.push_str("");
                        __sifr_concat
                    },
                );
        }
    }
    if ((restkey).as_str() != "") && ((row.len() as i64) > (fieldnames.len() as i64)) {
        let mut extras: Vec<String> = vec![];
        let mut j: i64 = fieldnames.len() as i64;
        while (j < (row.len() as i64)) {
            extras.push(_list_value_at(row, j));
            j += 1_i64;
        }
        {
            let __assign_key = restkey.clone();
            let __assign_value = format!("{:?}", extras);
            result.insert(__assign_key, __assign_value);
        }
    }
    result
}

// --- stdlib: _sifr.datetime ---
fn datetime_now() -> String {
    ::sifr_stdlib::time::datetime_now()
}
fn datetime_now_struct() -> Vec<i64> {
    ::sifr_stdlib::time::datetime_now_struct()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn datetime_format(dt: &String, fmt: &String) -> String {
    ::sifr_stdlib::time::datetime_format(dt, fmt)
}
fn datetime_from_timestamp(ts: f64) -> Result<String, ValueError> {
    ::sifr_stdlib::time::datetime_from_timestamp(ts)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}

// --- stdlib: _sifr.time ---
fn time_now() -> f64 {
    ::sifr_stdlib::time::time_now()
}
fn time_format(epoch: f64, fmt: &String) -> String {
    ::sifr_stdlib::time::time_format(epoch, fmt)
}
fn perf_counter() -> f64 {
    ::sifr_stdlib::time::perf_counter()
}
fn sleep(seconds: f64) {
    ::sifr_stdlib::time::sleep(seconds);
}
fn monotonic() -> f64 {
    ::sifr_stdlib::time::monotonic()
}
fn strptime(s: &String, fmt: &String) -> Result<String, ValueError> {
    ::sifr_stdlib::time::strptime(s, fmt)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _strptime_intrinsic(s: &String, fmt: &String) -> Result<String, ValueError> {
    ::sifr_stdlib::time::strptime(s, fmt)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn gmtime(epoch: f64) -> String {
    ::sifr_stdlib::time::gmtime(epoch)
}
fn _gmtime_intrinsic(epoch: f64) -> String {
    ::sifr_stdlib::time::gmtime(epoch)
}
fn localtime(epoch: f64) -> String {
    ::sifr_stdlib::time::localtime(epoch)
}
fn _localtime_intrinsic(epoch: f64) -> String {
    ::sifr_stdlib::time::localtime(epoch)
}
fn time_strptime(s: &String, fmt: &String) -> Result<Vec<i64>, ValueError> {
    ::sifr_stdlib::time::time_strptime(s, fmt)
        .map(|__sifr_bridge_ok| {
            __sifr_bridge_ok
                .into_iter()
                .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
                .collect()
        })
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn time_gmtime() -> Vec<i64> {
    ::sifr_stdlib::time::time_gmtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn time_localtime() -> Vec<i64> {
    ::sifr_stdlib::time::time_localtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}

// --- stdlib: sifr.datetime ---
#[derive(Debug, Clone)]
struct __SifrStdlib_sifr_x2edatetime_x2etimezone {
    _offset: i64,
}
impl __SifrStdlib_sifr_x2edatetime_x2etimezone {
    fn new(offset: i64) -> Self {
        let __sifr_field_init_0: i64 = offset;
        Self {
            _offset: __sifr_field_init_0,
        }
    }
}
impl __SifrStdlib_sifr_x2edatetime_x2etimezone {
    fn offset(&self) -> i64 {
        self._offset
    }
}
impl __SifrStdlib_sifr_x2edatetime_x2etimezone {
    fn iso_suffix(&self) -> String {
        let mut sign: String = "+".to_string();
        if (self._offset < (0_i64)) {
            sign = "-".to_string();
        }
        let mut abs_offset: i64 = self._offset;
        if abs_offset < (0_i64) {
            abs_offset = -abs_offset;
        }
        let h: i64 = abs_offset / (3600_i64);
        let m: i64 = (abs_offset % (3600_i64)) / (60_i64);
        let mut hs: String = format!("{}", h);
        if ((hs.chars().count() as i64) < (2_i64)) {
            hs = {
                let mut __sifr_concat: String = String::with_capacity(1usize + hs.len());
                __sifr_concat.push('0');
                __sifr_concat.push_str((hs).as_str());
                __sifr_concat
            };
        }
        let mut ms: String = format!("{}", m);
        if ((ms.chars().count() as i64) < (2_i64)) {
            ms = {
                let mut __sifr_concat: String = String::with_capacity(1usize + ms.len());
                __sifr_concat.push('0');
                __sifr_concat.push_str((ms).as_str());
                __sifr_concat
            };
        }
        {
            let mut __sifr_concat: String = String::with_capacity(
                ((sign.len() + hs.len()) + 1usize) + ms.len(),
            );
            __sifr_concat.push_str((sign).as_str());
            __sifr_concat.push_str((hs).as_str());
            __sifr_concat.push(':');
            __sifr_concat.push_str((ms).as_str());
            __sifr_concat
        }
    }
}
impl PartialEq for __SifrStdlib_sifr_x2edatetime_x2etimezone {
    fn eq(&self, other: &__SifrStdlib_sifr_x2edatetime_x2etimezone) -> bool {
        self._offset == other._offset
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2edatetime_x2etimezone {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        if (self._offset == (0_i64)) {
            return write!(f, "{}", "UTC".to_string());
        }
        write!(
            f, "{}", { let mut __sifr_concat : String = String::with_capacity(3usize +
            0usize); __sifr_concat.push_str("UTC"); __sifr_concat.push_str((self
            .iso_suffix()).as_str()); __sifr_concat }
        )
    }
}
#[derive(Debug, Clone)]
struct __SifrStdlib_sifr_x2edatetime_x2edatetime {
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
    microsecond: i64,
    _tz_offset: Option<i64>,
}
impl __SifrStdlib_sifr_x2edatetime_x2edatetime {
    fn new(
        year: i64,
        month: i64,
        day: i64,
        hour: i64,
        minute: i64,
        second: i64,
        microsecond: i64,
        tz_offset: Option<i64>,
    ) -> Self {
        let __sifr_field_init_0: i64 = year;
        let __sifr_field_init_1: i64 = month;
        let __sifr_field_init_2: i64 = day;
        let __sifr_field_init_3: i64 = hour;
        let __sifr_field_init_4: i64 = minute;
        let __sifr_field_init_5: i64 = second;
        let __sifr_field_init_6: i64 = microsecond;
        let __sifr_field_init_7: Option<i64> = tz_offset;
        Self {
            year: __sifr_field_init_0,
            month: __sifr_field_init_1,
            day: __sifr_field_init_2,
            hour: __sifr_field_init_3,
            minute: __sifr_field_init_4,
            second: __sifr_field_init_5,
            microsecond: __sifr_field_init_6,
            _tz_offset: __sifr_field_init_7,
        }
    }
}
impl __SifrStdlib_sifr_x2edatetime_x2edatetime {
    fn isoformat(&self) -> String {
        let y: String = format!("{}", self.year);
        let mut mo: String = format!("{}", self.month);
        if ((mo.chars().count() as i64) < (2_i64)) {
            mo = {
                let mut __sifr_concat: String = String::with_capacity(1usize + mo.len());
                __sifr_concat.push('0');
                __sifr_concat.push_str((mo).as_str());
                __sifr_concat
            };
        }
        let mut d: String = format!("{}", self.day);
        if ((d.chars().count() as i64) < (2_i64)) {
            d = {
                let mut __sifr_concat: String = String::with_capacity(1usize + d.len());
                __sifr_concat.push('0');
                __sifr_concat.push_str((d).as_str());
                __sifr_concat
            };
        }
        let mut h: String = format!("{}", self.hour);
        if ((h.chars().count() as i64) < (2_i64)) {
            h = {
                let mut __sifr_concat: String = String::with_capacity(1usize + h.len());
                __sifr_concat.push('0');
                __sifr_concat.push_str((h).as_str());
                __sifr_concat
            };
        }
        let mut mi: String = format!("{}", self.minute);
        if ((mi.chars().count() as i64) < (2_i64)) {
            mi = {
                let mut __sifr_concat: String = String::with_capacity(1usize + mi.len());
                __sifr_concat.push('0');
                __sifr_concat.push_str((mi).as_str());
                __sifr_concat
            };
        }
        let mut s: String = format!("{}", self.second);
        if ((s.chars().count() as i64) < (2_i64)) {
            s = {
                let mut __sifr_concat: String = String::with_capacity(1usize + s.len());
                __sifr_concat.push('0');
                __sifr_concat.push_str((s).as_str());
                __sifr_concat
            };
        }
        let mut base: String = {
            let mut __sifr_concat: String = String::with_capacity(
                (((((((((y.len() + 1usize) + mo.len()) + 1usize) + d.len()) + 1usize)
                    + h.len()) + 1usize) + mi.len()) + 1usize) + s.len(),
            );
            __sifr_concat.push_str((y).as_str());
            __sifr_concat.push('-');
            __sifr_concat.push_str((mo).as_str());
            __sifr_concat.push('-');
            __sifr_concat.push_str((d).as_str());
            __sifr_concat.push('T');
            __sifr_concat.push_str((h).as_str());
            __sifr_concat.push(':');
            __sifr_concat.push_str((mi).as_str());
            __sifr_concat.push(':');
            __sifr_concat.push_str((s).as_str());
            __sifr_concat
        };
        if (self.microsecond != (0_i64)) {
            base.push('.');
            base.push_str((_six_digits(self.microsecond)).as_str());
        }
        let tz_offset_opt: Option<i64> = self._tz_offset;
        if let Some(tz_offset_opt) = tz_offset_opt {
            let offset: i64 = tz_offset_opt;
            let mut sign: String = "+".to_string();
            let mut abs_offset: i64 = offset;
            if abs_offset < (0_i64) {
                sign = "-".to_string();
                abs_offset = -abs_offset;
            }
            let h_off: i64 = abs_offset / (3600_i64);
            let m_off: i64 = (abs_offset % (3600_i64)) / (60_i64);
            let mut hs_off: String = format!("{}", h_off);
            if ((hs_off.chars().count() as i64) < (2_i64)) {
                hs_off = {
                    let mut __sifr_concat: String = String::with_capacity(
                        1usize + hs_off.len(),
                    );
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((hs_off).as_str());
                    __sifr_concat
                };
            }
            let mut ms_off: String = format!("{}", m_off);
            if ((ms_off.chars().count() as i64) < (2_i64)) {
                ms_off = {
                    let mut __sifr_concat: String = String::with_capacity(
                        1usize + ms_off.len(),
                    );
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((ms_off).as_str());
                    __sifr_concat
                };
            }
            return {
                let mut __sifr_concat: String = String::with_capacity(
                    (((base.len() + sign.len()) + hs_off.len()) + 1usize) + ms_off.len(),
                );
                __sifr_concat.push_str((base).as_str());
                __sifr_concat.push_str((sign).as_str());
                __sifr_concat.push_str((hs_off).as_str());
                __sifr_concat.push(':');
                __sifr_concat.push_str((ms_off).as_str());
                __sifr_concat
            };
        }
        base
    }
}
impl __SifrStdlib_sifr_x2edatetime_x2edatetime {
    fn timestamp(&self) -> i64 {
        let mut days: i64 = 0_i64;
        if (self.year >= (1970_i64)) {
            let mut y: i64 = 1970_i64;
            while (y < self.year) {
                days += _days_in_year(y);
                y += 1_i64;
            }
        } else {
            let mut y: i64 = 1969_i64;
            while (y >= self.year) {
                days -= _days_in_year(y);
                y -= 1_i64;
            }
        }
        let mut m: i64 = 1_i64;
        while (m < self.month) {
            days += _days_in_month(self.year, m);
            m += 1_i64;
        }
        days = (days + self.day) - (1_i64);
        let naive_timestamp: i64 = (((days * (86400_i64)) + (self.hour * (3600_i64)))
            + (self.minute * (60_i64))) + self.second;
        let tz_offset_opt: Option<i64> = self._tz_offset;
        if let Some(tz_offset_opt) = tz_offset_opt {
            let offset: i64 = tz_offset_opt;
            return naive_timestamp - offset;
        }
        naive_timestamp
    }
}
impl __SifrStdlib_sifr_x2edatetime_x2edatetime {
    fn timestamp_microseconds(&self) -> i64 {
        (self.timestamp() * (1000000_i64)) + self.microsecond
    }
}
impl __SifrStdlib_sifr_x2edatetime_x2edatetime {
    fn astimezone(
        &self,
        tz: &Option<__SifrStdlib_sifr_x2edatetime_x2etimezone>,
    ) -> Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError> {
        let mut target: __SifrStdlib_sifr_x2edatetime_x2etimezone = __SifrStdlib_sifr_x2edatetime_x2etimezone::new(
            0_i64,
        );
        if let Some(tz) = tz.as_ref() {
            let __sifr_try_res: Result<(), ValueError> = (|| {
                let tz_text: String = format!("{}", tz);
                let target_offset: i64 = _timezone_offset_from_text(&tz_text)?;
                target = __SifrStdlib_sifr_x2edatetime_x2etimezone::new(target_offset);
                Ok(())
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                return Err(ValueError::new(e.message));
            }
        }
        _from_timestamp_microseconds_with_tz(
            self.timestamp_microseconds(),
            &Some((target).clone()),
        )
    }
}
impl PartialEq for __SifrStdlib_sifr_x2edatetime_x2edatetime {
    fn eq(&self, other: &__SifrStdlib_sifr_x2edatetime_x2edatetime) -> bool {
        let same_tz: bool = self._tz_offset == other._tz_offset;
        (((((((((self.year == other.year)) && ((self.month == other.month)))
            && ((self.day == other.day))) && ((self.hour == other.hour)))
            && ((self.minute == other.minute))) && ((self.second == other.second)))
            && ((self.microsecond == other.microsecond))) && (same_tz))
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2edatetime_x2edatetime {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.isoformat())
    }
}
#[derive(Debug, Clone)]
struct __SifrStdlib_sifr_x2edatetime_x2edate {
    year: i64,
    month: i64,
    day: i64,
}
impl __SifrStdlib_sifr_x2edatetime_x2edate {
    fn new(year: i64, month: i64, day: i64) -> Self {
        Self { year, month, day }
    }
}
impl __SifrStdlib_sifr_x2edatetime_x2edate {
    fn isoformat(&self) -> String {
        let y: String = format!("{}", self.year);
        let mut mo: String = format!("{}", self.month);
        if ((mo.chars().count() as i64) < (2_i64)) {
            mo = {
                let mut __sifr_concat: String = String::with_capacity(1usize + mo.len());
                __sifr_concat.push('0');
                __sifr_concat.push_str((mo).as_str());
                __sifr_concat
            };
        }
        let mut d: String = format!("{}", self.day);
        if ((d.chars().count() as i64) < (2_i64)) {
            d = {
                let mut __sifr_concat: String = String::with_capacity(1usize + d.len());
                __sifr_concat.push('0');
                __sifr_concat.push_str((d).as_str());
                __sifr_concat
            };
        }
        {
            let mut __sifr_concat: String = String::with_capacity(
                (((y.len() + 1usize) + mo.len()) + 1usize) + d.len(),
            );
            __sifr_concat.push_str((y).as_str());
            __sifr_concat.push('-');
            __sifr_concat.push_str((mo).as_str());
            __sifr_concat.push('-');
            __sifr_concat.push_str((d).as_str());
            __sifr_concat
        }
    }
}
impl PartialEq for __SifrStdlib_sifr_x2edatetime_x2edate {
    fn eq(&self, other: &__SifrStdlib_sifr_x2edatetime_x2edate) -> bool {
        ((((self.year == other.year)) && ((self.month == other.month)))
            && ((self.day == other.day)))
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2edatetime_x2edate {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.isoformat())
    }
}
fn _is_leap_year(year: i64) -> bool {
    (((year % (4_i64)) == (0_i64)) && ((year % (100_i64)) != (0_i64)))
        || ((year % (400_i64)) == (0_i64))
}
fn _days_in_year(year: i64) -> i64 {
    if _is_leap_year(year) {
        return 366_i64;
    }
    365_i64
}
fn _days_in_month(year: i64, month: i64) -> i64 {
    let month_days: Vec<i64> = vec![
        31_i64, 28_i64, 31_i64, 30_i64, 31_i64, 30_i64, 31_i64, 31_i64, 30_i64, 31_i64,
        30_i64, 31_i64
    ];
    let idx: i64 = month - (1_i64);
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
    if (month == (2_i64)) && _is_leap_year(year) {
        return 29_i64;
    }
    if let Some(d) = d {
        return d;
    }
    0_i64
}
fn _substring(value: &String, start: i64, end: i64) -> String {
    let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    let mut result: String = "".to_string();
    let mut i: i64 = start;
    while i < end {
        let ch: Option<String> = __sifr_chars_value
            .get(i as usize)
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            result.push_str((ch).as_str());
        }
        i += 1_i64;
    }
    result
}
fn _six_digits(value: i64) -> String {
    let mut rendered: String = format!("{}", value);
    let mut __sifr_chars_rendered: Vec<char> = rendered.chars().collect::<Vec<char>>();
    while ((__sifr_chars_rendered.len() as i64) < (6_i64)) {
        rendered = {
            let mut __sifr_concat: String = String::with_capacity(
                1usize + rendered.len(),
            );
            __sifr_concat.push('0');
            __sifr_concat.push_str((rendered).as_str());
            __sifr_concat
        };
        __sifr_chars_rendered = rendered.chars().collect::<Vec<char>>();
    }
    rendered
}
fn _parse_datetime_iso(
    value: &String,
) -> Result<(i64, i64, i64, i64, i64, i64), ValueError> {
    let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    if ((__sifr_chars_value.len() as i64) < (19_i64)) {
        return Err(ValueError::new("invalid datetime string".to_string()));
    }
    if ((((({
        let Some(__indexed_char) = __sifr_chars_value
            .get((4_i64) as usize)
            .map(|c| c.to_string()) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char
    }) != "-")
        || (({
            let Some(__indexed_char) = __sifr_chars_value
                .get((7_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != "-"))
        || (({
            let Some(__indexed_char) = __sifr_chars_value
                .get((10_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != "T"))
        || (({
            let Some(__indexed_char) = __sifr_chars_value
                .get((13_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != ":"))
        || (({
            let Some(__indexed_char) = __sifr_chars_value
                .get((16_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != ":")
    {
        return Err(ValueError::new("invalid datetime string".to_string()));
    }
    let __sifr_try_res: Result<
        Result<(i64, i64, i64, i64, i64, i64), ValueError>,
        ParseError,
    > = (|| {
        let year: i64 = (_substring(value, 0_i64, 4_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let month: i64 = (_substring(value, 5_i64, 7_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let day: i64 = (_substring(value, 8_i64, 10_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let hour: i64 = (_substring(value, 11_i64, 13_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let minute: i64 = (_substring(value, 14_i64, 16_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let second: i64 = (_substring(value, 17_i64, 19_i64))
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
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    if (text).as_str() == "UTC" {
        return Ok(0_i64);
    }
    if ((__sifr_chars_text.len() as i64) != (9_i64)) {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    if (_substring(text, 0_i64, 3_i64) != "UTC") {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    let sign_value: String = _substring(text, 3_i64, 4_i64);
    if (sign_value != "+") && (sign_value != "-") {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    if (__sifr_chars_text.get((6_i64) as usize).map(|c| c.to_string())
        != Some(":".to_string()))
    {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    let __sifr_try_res: Result<Result<i64, ValueError>, ParseError> = (|| {
        let hours: i64 = (_substring(text, 4_i64, 6_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let minutes: i64 = (_substring(text, 7_i64, 9_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let mut offset: i64 = (hours * (3600_i64)) + (minutes * (60_i64));
        if sign_value == "-" {
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
    tz: &Option<__SifrStdlib_sifr_x2edatetime_x2etimezone>,
) -> Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError> {
    let __sifr_try_res: Result<
        Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError>,
        ValueError,
    > = (|| {
        let whole_seconds: i64 = ts as i64;
        let fractional: f64 = ts - (whole_seconds as f64);
        let mut microsecond: i64 = (fractional * (1000000.0_f64)) as i64;
        if microsecond < (0_i64) {
            microsecond = -microsecond;
        }
        let mut adjusted_seconds: i64 = whole_seconds;
        let mut tz_offset_value: i64 = 0_i64;
        let mut tz_has_offset: bool = false;
        if let Some(tz) = tz.as_ref() {
            let tz_text: String = format!("{}", tz);
            let tz_offset: i64 = _timezone_offset_from_text(&tz_text)?;
            adjusted_seconds = whole_seconds + tz_offset;
            tz_offset_value = tz_offset;
            tz_has_offset = true;
        }
        let rendered: String = datetime_from_timestamp(adjusted_seconds as f64)?;
        let parts: (i64, i64, i64, i64, i64, i64) = _parse_datetime_iso(&rendered)?;
        let year_part: Option<i64> = Some((parts).0);
        let month_part: Option<i64> = Some((parts).1);
        let day_part: Option<i64> = Some((parts).2);
        let hour_part: Option<i64> = Some((parts).3);
        let minute_part: Option<i64> = Some((parts).4);
        let second_part: Option<i64> = Some((parts).5);
        let mut year: i64 = 0_i64;
        let mut month: i64 = 1_i64;
        let mut day: i64 = 1_i64;
        let mut hour: i64 = 0_i64;
        let mut minute: i64 = 0_i64;
        let mut second: i64 = 0_i64;
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
                    __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                        year,
                        month,
                        day,
                        hour,
                        minute,
                        second,
                        microsecond,
                        Some(tz_offset_value),
                    ),
                ),
            );
        }
        return Ok(
            Ok(
                __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                    year,
                    month,
                    day,
                    hour,
                    minute,
                    second,
                    microsecond,
                    None,
                ),
            ),
        );
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
fn _from_timestamp_microseconds_with_tz(
    value: i64,
    tz: &Option<__SifrStdlib_sifr_x2edatetime_x2etimezone>,
) -> Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError> {
    let whole_seconds: i64 = value / (1000000_i64);
    let microsecond: i64 = value % (1000000_i64);
    let __sifr_try_res: Result<
        Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError>,
        ValueError,
    > = (|| {
        let result: __SifrStdlib_sifr_x2edatetime_x2edatetime = _from_timestamp_with_tz(
            whole_seconds as f64,
            tz,
        )?;
        return Ok(
            Ok(
                __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                    result.year,
                    result.month,
                    result.day,
                    result.hour,
                    result.minute,
                    result.second,
                    microsecond,
                    result._tz_offset,
                ),
            ),
        );
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

// --- stdlib: _sifr.logging ---
fn set_global_level(level: i64) {
    ::sifr_stdlib::logging::set_global_level(
        ::sifr_runtime::interop::SifrIntBridge::from(level),
    );
}
fn get_global_level() -> i64 {
    ::sifr_stdlib::logging::get_global_level().to_i64_saturating()
}

// --- stdlib: sifr.logging ---
const DEBUG: i64 = 10_i64;
const INFO: i64 = 20_i64;
const WARNING: i64 = 30_i64;
const ERROR: i64 = 40_i64;
const CRITICAL: i64 = 50_i64;
const NOTSET: i64 = 0_i64;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2elogging_x2eFormatter {
    _fmt: String,
}
impl __SifrStdlib_sifr_x2elogging_x2eFormatter {
    fn new(fmt: String) -> Self {
        let __sifr_field_init_0: String = fmt;
        Self { _fmt: __sifr_field_init_0 }
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eFormatter {
    fn template(&self) -> String {
        self._fmt.clone()
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eFormatter {
    fn format(&self, level: &String, name: &String, msg: &String) -> String {
        let mut result: String = self._fmt.clone();
        result = result.replace("%(levelname)s", &level);
        result = result.replace("%(name)s", &name);
        result = result.replace("%(message)s", &msg);
        result
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2elogging_x2eFormatter {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Formatter(_fmt={})", self._fmt)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
    _level: i64,
    _formatter: __SifrStdlib_sifr_x2elogging_x2eFormatter,
}
impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
    fn new(level: i64) -> Self {
        let __sifr_field_init_0: i64 = level;
        let __sifr_field_init_1: __SifrStdlib_sifr_x2elogging_x2eFormatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
            "%(levelname)s:%(name)s:%(message)s".to_string(),
        );
        Self {
            _level: __sifr_field_init_0,
            _formatter: __sifr_field_init_1,
        }
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
    fn level(&self) -> i64 {
        self._level
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
    fn set_formatter(&mut self, fmt: &__SifrStdlib_sifr_x2elogging_x2eFormatter) {
        self._formatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
            format!("{}{}", fmt._fmt.clone(), ""),
        );
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
    fn format_template(&self) -> String {
        self._formatter.template()
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
    fn _allows(&self, level_num: i64) -> bool {
        if (self._level == NOTSET) {
            return true;
        }
        (level_num >= self._level)
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
    fn emit(&self, level: &String, name: &String, msg: &String) {
        let level_num: i64 = _level_name_to_num(level);
        if !(self._allows(level_num)) {
            return;
        }
        let line: String = self._formatter.format(level, name, msg);
        println!("{}", line);
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(
            f, "StreamHandler(_level={}, _formatter={})", self._level, self._formatter
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2elogging_x2eFileHandler {
    _path: String,
    _level: i64,
    _formatter: __SifrStdlib_sifr_x2elogging_x2eFormatter,
}
impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
    fn new(path: String, level: i64) -> Self {
        let __sifr_field_init_0: String = {
            let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
            __sifr_concat.push_str((path).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        let __sifr_field_init_1: i64 = level;
        let __sifr_field_init_2: __SifrStdlib_sifr_x2elogging_x2eFormatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
            "%(levelname)s:%(name)s:%(message)s".to_string(),
        );
        Self {
            _path: __sifr_field_init_0,
            _level: __sifr_field_init_1,
            _formatter: __sifr_field_init_2,
        }
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
    fn path(&self) -> String {
        self._path.clone()
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
    fn level(&self) -> i64 {
        self._level
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
    fn set_formatter(&mut self, fmt: &__SifrStdlib_sifr_x2elogging_x2eFormatter) {
        self._formatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
            format!("{}{}", fmt._fmt.clone(), ""),
        );
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
    fn format_template(&self) -> String {
        self._formatter.template()
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
    fn _allows(&self, level_num: i64) -> bool {
        if (self._level == NOTSET) {
            return true;
        }
        (level_num >= self._level)
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
    fn emit(&self, level: &String, name: &String, msg: &String) {
        let level_num: i64 = _level_name_to_num(level);
        if !(self._allows(level_num)) {
            return;
        }
        let line: String = {
            let mut __sifr_concat: String = String::with_capacity(0usize + 1usize);
            __sifr_concat.push_str((self._formatter.format(level, name, msg)).as_str());
            __sifr_concat.push('\n');
            __sifr_concat
        };
        let __sifr_try_res: Result<(), IOError> = (|| {
            let mut fh: __SifrIoTextFileHandle = open_text(
                &self._path,
                &"a".to_string(),
                &Some((utf8()).clone()),
                &None,
            )?;
            let __sifr_try_res: Result<(), IOError> = (|| {
                let _ = fh.write(&line)?;
                Ok(())
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e2 = __sifr_try_err.clone();
                let _ = e2.message;
            }
            fh.close();
            Ok(())
        })();
        if let Err(__sifr_try_err) = __sifr_try_res {
            let e = __sifr_try_err.clone();
            let _ = e.message;
        }
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2elogging_x2eFileHandler {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(
            f, "FileHandler(_path={}, _level={}, _formatter={})", self._path, self
            ._level, self._formatter
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2elogging_x2eNullHandler {
    _level: i64,
    _formatter: __SifrStdlib_sifr_x2elogging_x2eFormatter,
}
impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
    fn new(level: i64) -> Self {
        let __sifr_field_init_0: i64 = level;
        let __sifr_field_init_1: __SifrStdlib_sifr_x2elogging_x2eFormatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
            "%(levelname)s:%(name)s:%(message)s".to_string(),
        );
        Self {
            _level: __sifr_field_init_0,
            _formatter: __sifr_field_init_1,
        }
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
    fn level(&self) -> i64 {
        self._level
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
    fn set_formatter(&mut self, fmt: &__SifrStdlib_sifr_x2elogging_x2eFormatter) {
        self._formatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
            format!("{}{}", fmt._fmt.clone(), ""),
        );
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
    fn format_template(&self) -> String {
        self._formatter.template()
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
    fn emit(&self, level: &String, name: &String, msg: &String) {
        let _ = (level).clone();
        let _ = (name).clone();
        let _ = (msg).clone();
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2elogging_x2eNullHandler {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "NullHandler(_level={}, _formatter={})", self._level, self._formatter)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2elogging_x2eLogger {
    _name: String,
    _level: i64,
    _log_path: String,
    _handler_kind: String,
    _handler_path: String,
    _handler_level: i64,
    _handler_fmt: String,
}
impl __SifrStdlib_sifr_x2elogging_x2eLogger {
    fn new(name: String, level: i64) -> Self {
        let __sifr_field_init_0: String = name;
        let __sifr_field_init_1: i64 = level;
        let __sifr_field_init_2: String = "".to_string();
        let __sifr_field_init_3: String = "".to_string();
        let __sifr_field_init_4: String = "".to_string();
        let __sifr_field_init_5: i64 = NOTSET;
        let __sifr_field_init_6: String = "%(levelname)s:%(name)s:%(message)s"
            .to_string();
        Self {
            _name: __sifr_field_init_0,
            _level: __sifr_field_init_1,
            _log_path: __sifr_field_init_2,
            _handler_kind: __sifr_field_init_3,
            _handler_path: __sifr_field_init_4,
            _handler_level: __sifr_field_init_5,
            _handler_fmt: __sifr_field_init_6,
        }
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eLogger {
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eLogger {
    fn set_file(&mut self, path: &String) {
        self._log_path = {
            let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
            __sifr_concat.push_str((path).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eLogger {
    fn add_handler(&mut self, handler: &__SifrStdlib_sifr_x2elogging_x2eFileHandler) {
        self._handler_kind = "file".to_string();
        self._handler_path = handler.path();
        self._handler_level = handler.level();
        self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eLogger {
    fn set_stream_handler(
        &mut self,
        handler: &__SifrStdlib_sifr_x2elogging_x2eStreamHandler,
    ) {
        self._handler_kind = "stream".to_string();
        self._handler_path = "".to_string();
        self._handler_level = handler.level();
        self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eLogger {
    fn set_null_handler(
        &mut self,
        handler: &__SifrStdlib_sifr_x2elogging_x2eNullHandler,
    ) {
        self._handler_kind = "null".to_string();
        self._handler_path = "".to_string();
        self._handler_level = handler.level();
        self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eLogger {
    fn clear_handler(&mut self) {
        self._handler_kind = "".to_string();
        self._handler_path = "".to_string();
        self._handler_level = NOTSET;
        self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eLogger {
    fn _handler_allows(&self, level_num: i64) -> bool {
        if (self._handler_level == NOTSET) {
            return true;
        }
        (level_num >= self._handler_level)
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eLogger {
    fn _handler_line(&self, level: &String, msg: &String) -> String {
        let formatter: __SifrStdlib_sifr_x2elogging_x2eFormatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
            self._handler_fmt.clone(),
        );
        formatter.format(level, &self._name.clone(), msg)
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eLogger {
    fn _emit(&self, level: &String, level_num: i64, msg: &String) {
        if (self._level > level_num) {
            return;
        }
        if (self._handler_kind.clone() == "null") {
            return;
        }
        if (self._handler_kind.clone() == "stream") {
            if self._handler_allows(level_num) {
                println!("{}", self._handler_line(level, msg));
            }
            return;
        }
        if (self._handler_kind.clone() == "file") {
            if self._handler_allows(level_num) && (self._handler_path.clone() != "") {
                let line: String = {
                    let mut __sifr_concat: String = String::with_capacity(
                        0usize + 1usize,
                    );
                    __sifr_concat.push_str((self._handler_line(level, msg)).as_str());
                    __sifr_concat.push('\n');
                    __sifr_concat
                };
                let __sifr_try_res: Result<(), IOError> = (|| {
                    let mut fh: __SifrIoTextFileHandle = open_text(
                        &self._handler_path,
                        &"a".to_string(),
                        &Some((utf8()).clone()),
                        &None,
                    )?;
                    let __sifr_try_res: Result<(), IOError> = (|| {
                        let _ = fh.write(&line)?;
                        Ok(())
                    })();
                    if let Err(__sifr_try_err) = __sifr_try_res {
                        let e2 = __sifr_try_err.clone();
                        let _ = e2.message;
                    }
                    fh.close();
                    Ok(())
                })();
                if let Err(__sifr_try_err) = __sifr_try_res {
                    let e = __sifr_try_err.clone();
                    let _ = e.message;
                }
            }
            return;
        }
        let line: String = {
            let mut __sifr_concat: String = String::with_capacity(
                ((((1usize + level.len()) + 2usize) + 0usize) + 2usize) + msg.len(),
            );
            __sifr_concat.push('[');
            __sifr_concat.push_str((level).as_str());
            __sifr_concat.push_str("] ");
            __sifr_concat.push_str((self._name.clone()).as_str());
            __sifr_concat.push_str(": ");
            __sifr_concat.push_str((msg).as_str());
            __sifr_concat
        };
        println!("{}", line);
        if (self._log_path.clone() != "") {
            let __sifr_try_res: Result<(), IOError> = (|| {
                let mut fh: __SifrIoTextFileHandle = open_text(
                    &self._log_path,
                    &"a".to_string(),
                    &Some((utf8()).clone()),
                    &None,
                )?;
                let __sifr_try_res: Result<(), IOError> = (|| {
                    let _ = fh
                        .write(
                            &({
                                let mut __sifr_concat: String = String::with_capacity(
                                    line.len() + 1usize,
                                );
                                __sifr_concat.push_str((line).as_str());
                                __sifr_concat.push('\n');
                                __sifr_concat
                            }),
                        )?;
                    Ok(())
                })();
                if let Err(__sifr_try_err) = __sifr_try_res {
                    let e2 = __sifr_try_err.clone();
                    let _ = e2.message;
                }
                fh.close();
                Ok(())
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                let _ = e.message;
            }
        }
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eLogger {
    fn debug(&self, msg: &String) {
        self._emit(&"DEBUG".to_string(), DEBUG, msg);
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eLogger {
    fn info(&self, msg: &String) {
        self._emit(&"INFO".to_string(), INFO, msg);
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eLogger {
    fn warning(&self, msg: &String) {
        self._emit(&"WARNING".to_string(), WARNING, msg);
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eLogger {
    fn error(&self, msg: &String) {
        self._emit(&"ERROR".to_string(), ERROR, msg);
    }
}
impl __SifrStdlib_sifr_x2elogging_x2eLogger {
    fn critical(&self, msg: &String) {
        self._emit(&"CRITICAL".to_string(), CRITICAL, msg);
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2elogging_x2eLogger {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(
            f,
            "Logger(_name={}, _level={}, _log_path={}, _handler_kind={}, _handler_path={}, _handler_level={}, _handler_fmt={})",
            self._name, self._level, self._log_path, self._handler_kind, self
            ._handler_path, self._handler_level, self._handler_fmt
        )
    }
}
fn _level_name_to_num(level: &String) -> i64 {
    if (level).as_str() == "DEBUG" {
        return DEBUG;
    }
    if (level).as_str() == "INFO" {
        return INFO;
    }
    if (level).as_str() == "WARNING" {
        return WARNING;
    }
    if (level).as_str() == "ERROR" {
        return ERROR;
    }
    if (level).as_str() == "CRITICAL" {
        return CRITICAL;
    }
    NOTSET
}
fn getLogger(name: &String) -> __SifrStdlib_sifr_x2elogging_x2eLogger {
    let level: i64 = get_global_level();
    __SifrStdlib_sifr_x2elogging_x2eLogger::new((name).clone(), level)
}

// --- stdlib: sifr.pathlib ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2epathlib_x2ePath {
    _path: String,
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn new(path: String) -> Self {
        let __sifr_field_init_0: String = path;
        Self { _path: __sifr_field_init_0 }
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn name(&self) -> String {
        basename(&self._path)
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn parent(&self) -> __SifrStdlib_sifr_x2epathlib_x2ePath {
        __SifrStdlib_sifr_x2epathlib_x2ePath::new(dirname(&self._path))
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn suffix(&self) -> String {
        extension(&self._path)
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn stem(&self) -> String {
        stem(&self._path)
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn exists(&self) -> bool {
        exists(&self._path)
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn is_file(&self) -> bool {
        is_file(&self._path)
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn is_dir(&self) -> bool {
        is_dir(&self._path)
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn is_absolute(&self) -> bool {
        is_absolute(&self._path)
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn read_text(&self) -> Result<String, IOError> {
        read_text(&self._path)
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn write_text(&self, content: &String) -> Result<(), IOError> {
        write_text(&self._path, content)
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn mkdir(&self) -> Result<(), IOError> {
        mkdir(&self._path)
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn joinpath(&self, child: &String) -> __SifrStdlib_sifr_x2epathlib_x2ePath {
        __SifrStdlib_sifr_x2epathlib_x2ePath::new(join_path(&self._path, child))
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn to_str(&self) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((self._path.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn touch(&self) -> Result<(), IOError> {
        touch(&self._path)
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn unlink(&self) -> Result<(), IOError> {
        remove_file(&self._path)
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn rmdir(&self) -> Result<(), IOError> {
        rmdir(&self._path)
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn resolve(&self) -> Result<String, IOError> {
        resolve_path(&self._path)
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn iterdir(&self) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        _iterdir_to_iter(&self._path)
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn with_name(&self, name: &String) -> __SifrStdlib_sifr_x2epathlib_x2ePath {
        let parent: String = dirname(&self._path);
        if parent == "" {
            return __SifrStdlib_sifr_x2epathlib_x2ePath::new(format!("{}{}", name, ""));
        }
        __SifrStdlib_sifr_x2epathlib_x2ePath::new(
            format!("{}{}", format!("{}{}", parent, "/"), name),
        )
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn with_suffix(&self, suffix: &String) -> __SifrStdlib_sifr_x2epathlib_x2ePath {
        let s: String = stem(&self._path);
        let parent: String = dirname(&self._path);
        if parent == "" {
            return __SifrStdlib_sifr_x2epathlib_x2ePath::new(format!("{}{}", s, suffix));
        }
        __SifrStdlib_sifr_x2epathlib_x2ePath::new(
            format!("{}{}", format!("{}{}", format!("{}{}", parent, "/"), s), suffix),
        )
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn glob(
        &self,
        pattern: &String,
    ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        _glob_to_iter(&self._path, pattern)
    }
}
impl __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn rglob(
        &self,
        pattern: &String,
    ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        _rglob_to_iter(&self._path, pattern)
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2epathlib_x2ePath {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Path(_path={})", self._path)
    }
}
fn join_path(base: &String, child: &String) -> String {
    let __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
    if ((__sifr_chars_base.len() as i64) == (0_i64)) {
        return {
            let mut __sifr_concat: String = String::with_capacity(child.len() + 0usize);
            __sifr_concat.push_str((child).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    let last: Option<String> = __sifr_chars_base
        .get(((base.chars().count() as i64) - (1_i64)) as usize)
        .map(|c| c.to_string());
    if let Some(last) = last {
        if last == "/" {
            return {
                let mut __sifr_concat: String = String::with_capacity(
                    base.len() + child.len(),
                );
                __sifr_concat.push_str((base).as_str());
                __sifr_concat.push_str((child).as_str());
                __sifr_concat
            };
        }
    }
    {
        let mut __sifr_concat: String = String::with_capacity(
            (base.len() + 1usize) + child.len(),
        );
        __sifr_concat.push_str((base).as_str());
        __sifr_concat.push('/');
        __sifr_concat.push_str((child).as_str());
        __sifr_concat
    }
}
fn basename(path: &String) -> String {
    let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
    let mut i: i64 = (__sifr_chars_path.len() as i64) - (1_i64);
    while i >= (0_i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_path
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch) = ch {
            if ch == "/" {
                return {
                    let _slice_src = &__sifr_chars_path;
                    let _slice_len_i64 = _slice_src.len() as i64;
                    let _slice_start_i64 = if (i + (1_i64)) < 0 {
                        (_slice_len_i64 + (i + (1_i64))).max(0)
                    } else {
                        (i + (1_i64)).min(_slice_len_i64)
                    };
                    let _slice_stop_i64 = _slice_len_i64;
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start_i64 as usize)
                            .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                            .copied(),
                    )
                };
            }
        }
        i -= 1_i64;
    }
    {
        let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
        __sifr_concat.push_str((path).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn dirname(path: &String) -> String {
    let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
    let mut i: i64 = (__sifr_chars_path.len() as i64) - (1_i64);
    while i >= (0_i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_path
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch) = ch {
            if ch == "/" {
                return {
                    let _slice_src = &__sifr_chars_path;
                    let _slice_len_i64 = _slice_src.len() as i64;
                    let _slice_start_i64 = 0;
                    let _slice_stop_i64 = if i < 0 {
                        (_slice_len_i64 + i).max(0)
                    } else {
                        i.min(_slice_len_i64)
                    };
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start_i64 as usize)
                            .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                            .copied(),
                    )
                };
            }
        }
        i -= 1_i64;
    }
    "".to_string()
}
fn extension(path: &String) -> String {
    let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
    let mut i: i64 = (__sifr_chars_path.len() as i64) - (1_i64);
    while i >= (0_i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_path
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch) = ch {
            if ch == "." {
                return {
                    let _slice_src = &__sifr_chars_path;
                    let _slice_len_i64 = _slice_src.len() as i64;
                    let _slice_start_i64 = if i < 0 {
                        (_slice_len_i64 + i).max(0)
                    } else {
                        i.min(_slice_len_i64)
                    };
                    let _slice_stop_i64 = _slice_len_i64;
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start_i64 as usize)
                            .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                            .copied(),
                    )
                };
            }
            if ch == "/" {
                return "".to_string();
            }
        }
        i -= 1_i64;
    }
    "".to_string()
}
fn stem(path: &String) -> String {
    let base: String = basename(path);
    let __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
    let mut i: i64 = (__sifr_chars_base.len() as i64) - (1_i64);
    while i > (0_i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_base
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch) = ch {
            if ch == "." {
                return {
                    let _slice_src = &__sifr_chars_base;
                    let _slice_len_i64 = _slice_src.len() as i64;
                    let _slice_start_i64 = 0;
                    let _slice_stop_i64 = if i < 0 {
                        (_slice_len_i64 + i).max(0)
                    } else {
                        i.min(_slice_len_i64)
                    };
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start_i64 as usize)
                            .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                            .copied(),
                    )
                };
            }
        }
        i -= 1_i64;
    }
    {
        let mut __sifr_concat: String = String::with_capacity(base.len() + 0usize);
        __sifr_concat.push_str((base).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn is_absolute(path: &String) -> bool {
    let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
    if ((__sifr_chars_path.len() as i64) == (0_i64)) {
        return false;
    }
    if ((__sifr_chars_path.len() as i64) >= (3_i64)) {
        let colon: Option<String> = __sifr_chars_path
            .get((1_i64) as usize)
            .map(|c| c.to_string());
        let sep: Option<String> = __sifr_chars_path
            .get((2_i64) as usize)
            .map(|c| c.to_string());
        if let Some(colon) = colon {
            if let Some(sep) = sep {
                if (colon == ":") && ((sep == "/") || (sep == "\\")) {
                    return true;
                }
            }
        }
    }
    let first: Option<String> = Some({
        let Some(__indexed_char) = __sifr_chars_path
            .get((0_i64) as usize)
            .map(|c| c.to_string()) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char
    });
    if let Some(first) = first {
        if (first == "/") || (first == "\\") {
            return true;
        }
    }
    false
}
fn _iter_list_str(entries: Vec<String>) -> Box<dyn Iterator<Item = String>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<String> = Vec::new().into_iter();
    Box::new(
        ::std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<String> = Vec::new();
                let mut i: i64 = 0_i64;
                while (i < (entries.len() as i64)) {
                    _yields.push(entries[i as usize].clone());
                    i += 1_i64;
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            __sifr_generator_iter.next()
        }),
    )
}
fn _iterdir_list(path: &String) -> Result<Vec<String>, IOError> {
    iterdir(path)
}
fn _glob_list(path: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    glob_pattern(path, pattern)
}
fn _rglob_list(path: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    rglob_pattern(path, pattern)
}
fn _iterdir_to_iter(path: &String) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
    let __sifr_try_res: Result<
        Result<Box<dyn Iterator<Item = String>>, IOError>,
        IOError,
    > = (|| {
        let entries: Vec<String> = _iterdir_list(path)?;
        return Ok(Ok(_iter_list_str(entries)));
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
fn _glob_to_iter(
    path: &String,
    pattern: &String,
) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
    let __sifr_try_res: Result<
        Result<Box<dyn Iterator<Item = String>>, IOError>,
        IOError,
    > = (|| {
        let entries: Vec<String> = _glob_list(path, pattern)?;
        return Ok(Ok(_iter_list_str(entries)));
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
fn _rglob_to_iter(
    path: &String,
    pattern: &String,
) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
    let __sifr_try_res: Result<
        Result<Box<dyn Iterator<Item = String>>, IOError>,
        IOError,
    > = (|| {
        let entries: Vec<String> = _rglob_list(path, pattern)?;
        return Ok(Ok(_iter_list_str(entries)));
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

// --- stdlib: _sifr.regex ---
type __SifrStdlib___sifr_x2eregex_x2eCompiledPattern = ::sifr_runtime::interop::Handle<
    ::sifr_stdlib::regex::CompiledPattern,
>;
trait __SifrOpaque__SifrStdlib___sifr_x2eregex_x2eCompiledPatternMethods {
    fn search(&self, text: &String) -> Result<Option<String>, RegexError>;
    fn is_match(&self, text: &String) -> Result<bool, RegexError>;
    fn sub(&self, replacement: &String, text: &String) -> Result<String, RegexError>;
    fn findall(&self, text: &String) -> Result<Vec<String>, RegexError>;
    fn split(&self, text: &String) -> Result<Vec<String>, RegexError>;
    fn pattern(&self) -> Result<String, RegexError>;
    fn flags(&self) -> Result<i64, RegexError>;
}
impl __SifrOpaque__SifrStdlib___sifr_x2eregex_x2eCompiledPatternMethods
for __SifrStdlib___sifr_x2eregex_x2eCompiledPattern {
    fn search(&self, text: &String) -> Result<Option<String>, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_search(self, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    fn is_match(&self, text: &String) -> Result<bool, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_is_match(self, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    fn sub(&self, replacement: &String, text: &String) -> Result<String, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_replace(self, replacement, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    fn findall(&self, text: &String) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_findall(self, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    fn split(&self, text: &String) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_split(self, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    fn pattern(&self) -> Result<String, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_source(self)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    fn flags(&self) -> Result<i64, RegexError> {
        ::sifr_stdlib::regex::compiled_pattern_flags(self)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
}
fn compile_pattern(
    pattern: &String,
) -> Result<__SifrStdlib___sifr_x2eregex_x2eCompiledPattern, RegexError> {
    ::sifr_stdlib::regex::compile_pattern(pattern)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn compile_pattern_flags(
    pattern: &String,
    flags: i64,
) -> Result<__SifrStdlib___sifr_x2eregex_x2eCompiledPattern, RegexError> {
    ::sifr_stdlib::regex::compile_pattern_flags(
            pattern,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_match(pattern: &String, text: &String) -> Result<bool, RegexError> {
    ::sifr_stdlib::regex::re_match(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_find(pattern: &String, text: &String) -> Result<Option<String>, RegexError> {
    ::sifr_stdlib::regex::re_find(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_replace(
    pattern: &String,
    replacement: &String,
    text: &String,
) -> Result<String, RegexError> {
    ::sifr_stdlib::regex::re_replace(pattern, replacement, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_findall(pattern: &String, text: &String) -> Result<Vec<String>, RegexError> {
    ::sifr_stdlib::regex::re_findall(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_split(pattern: &String, text: &String) -> Result<Vec<String>, RegexError> {
    ::sifr_stdlib::regex::re_split(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_find_start(pattern: &String, text: &String) -> Result<i64, RegexError> {
    ::sifr_stdlib::regex::re_find_start(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_find_end(pattern: &String, text: &String) -> Result<i64, RegexError> {
    ::sifr_stdlib::regex::re_find_end(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_match_flags(
    pattern: &String,
    text: &String,
    flags: i64,
) -> Result<bool, RegexError> {
    ::sifr_stdlib::regex::re_match_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_find_flags(
    pattern: &String,
    text: &String,
    flags: i64,
) -> Result<Option<String>, RegexError> {
    ::sifr_stdlib::regex::re_find_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_replace_flags(
    pattern: &String,
    replacement: &String,
    text: &String,
    flags: i64,
) -> Result<String, RegexError> {
    ::sifr_stdlib::regex::re_replace_flags(
            pattern,
            replacement,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_findall_flags(
    pattern: &String,
    text: &String,
    flags: i64,
) -> Result<Vec<String>, RegexError> {
    ::sifr_stdlib::regex::re_findall_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_split_flags(
    pattern: &String,
    text: &String,
    flags: i64,
) -> Result<Vec<String>, RegexError> {
    ::sifr_stdlib::regex::re_split_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}

// --- stdlib: sifr.re ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2ere_x2eMatch {
    _matched: String,
    _start: i64,
    _end: i64,
}
impl __SifrStdlib_sifr_x2ere_x2eMatch {
    fn new(matched: String, start: i64, end: i64) -> Self {
        let __sifr_field_init_0: String = matched;
        let __sifr_field_init_1: i64 = start;
        let __sifr_field_init_2: i64 = end;
        Self {
            _matched: __sifr_field_init_0,
            _start: __sifr_field_init_1,
            _end: __sifr_field_init_2,
        }
    }
}
impl __SifrStdlib_sifr_x2ere_x2eMatch {
    fn group(&self) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((self._matched.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
}
impl __SifrStdlib_sifr_x2ere_x2eMatch {
    fn start(&self) -> i64 {
        self._start
    }
}
impl __SifrStdlib_sifr_x2ere_x2eMatch {
    fn end(&self) -> i64 {
        self._end
    }
}
impl __SifrStdlib_sifr_x2ere_x2eMatch {
    fn span(&self) -> Vec<i64> {
        let result: Vec<i64> = vec![self._start, self._end];
        result
    }
}
impl __SifrStdlib_sifr_x2ere_x2eMatch {
    fn to_str(&self) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((self._matched.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2ere_x2eMatch {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(
            f, "Match(_matched={}, _start={}, _end={})", self._matched, self._start, self
            ._end
        )
    }
}
struct __SifrStdlib_sifr_x2ere_x2ePattern {
    _compiled: __SifrStdlib___sifr_x2eregex_x2eCompiledPattern,
    _pattern: String,
    _flags: i64,
}
impl __SifrStdlib_sifr_x2ere_x2ePattern {
    fn new(
        compiled: __SifrStdlib___sifr_x2eregex_x2eCompiledPattern,
        pattern: String,
        flags: i64,
    ) -> Self {
        let __sifr_field_init_0: __SifrStdlib___sifr_x2eregex_x2eCompiledPattern = compiled;
        let __sifr_field_init_1: String = pattern;
        let __sifr_field_init_2: i64 = flags;
        Self {
            _compiled: __sifr_field_init_0,
            _pattern: __sifr_field_init_1,
            _flags: __sifr_field_init_2,
        }
    }
}
impl __SifrStdlib_sifr_x2ere_x2ePattern {
    fn search(&self, text: &String) -> Result<Option<String>, RegexError> {
        self._compiled.search(text)
    }
}
impl __SifrStdlib_sifr_x2ere_x2ePattern {
    fn is_match(&self, text: &String) -> Result<bool, RegexError> {
        self._compiled.is_match(text)
    }
}
impl __SifrStdlib_sifr_x2ere_x2ePattern {
    fn sub(&self, replacement: &String, text: &String) -> Result<String, RegexError> {
        self._compiled.sub(replacement, text)
    }
}
impl __SifrStdlib_sifr_x2ere_x2ePattern {
    fn findall(&self, text: &String) -> Result<Vec<String>, RegexError> {
        self._compiled.findall(text)
    }
}
impl __SifrStdlib_sifr_x2ere_x2ePattern {
    fn finditer(
        &self,
        text: &String,
    ) -> Result<Box<dyn Iterator<Item = __SifrStdlib_sifr_x2ere_x2eMatch>>, RegexError> {
        let __sifr_try_res: Result<
            Result<
                Box<dyn Iterator<Item = __SifrStdlib_sifr_x2ere_x2eMatch>>,
                RegexError,
            >,
            RegexError,
        > = (|| {
            let found_items: Vec<String> = self._compiled.findall(text)?;
            let matches: Vec<__SifrStdlib_sifr_x2ere_x2eMatch> = _finditer_from_items(
                &found_items,
                text,
            );
            return Ok(Ok(_iter_matches(matches)));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(RegexError::new(e.message));
            }
        }
    }
}
impl __SifrStdlib_sifr_x2ere_x2ePattern {
    fn split(&self, text: &String) -> Result<Vec<String>, RegexError> {
        self._compiled.split(text)
    }
}
impl __SifrStdlib_sifr_x2ere_x2ePattern {
    fn pattern(&self) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((self._pattern.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
}
impl __SifrStdlib_sifr_x2ere_x2ePattern {
    fn flags(&self) -> i64 {
        self._flags
    }
}
fn _iter_matches(
    matches: Vec<__SifrStdlib_sifr_x2ere_x2eMatch>,
) -> Box<dyn Iterator<Item = __SifrStdlib_sifr_x2ere_x2eMatch>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<
        __SifrStdlib_sifr_x2ere_x2eMatch,
    > = Vec::new().into_iter();
    Box::new(
        ::std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<__SifrStdlib_sifr_x2ere_x2eMatch> = Vec::new();
                let mut i: i64 = 0_i64;
                while (i < (matches.len() as i64)) {
                    _yields.push(matches[i as usize].clone());
                    i += 1_i64;
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            __sifr_generator_iter.next()
        }),
    )
}
fn _find_index_from(text: &String, needle: &String, start: i64) -> i64 {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let __sifr_chars_needle: Vec<char> = needle.chars().collect::<Vec<char>>();
    if start < (0_i64) {
        return -(1_i64);
    }
    if ((__sifr_chars_needle.len() as i64) == (0_i64)) {
        if (start <= (__sifr_chars_text.len() as i64)) {
            return start;
        }
        return -(1_i64);
    }
    let max_start: i64 = (__sifr_chars_text.len() as i64)
        - (__sifr_chars_needle.len() as i64);
    let mut i: i64 = start;
    while i <= max_start {
        if (({
            let _slice_src = &__sifr_chars_text;
            let _slice_len_i64 = _slice_src.len() as i64;
            let _slice_start_i64 = if i < 0 {
                (_slice_len_i64 + i).max(0)
            } else {
                i.min(_slice_len_i64)
            };
            let _slice_stop_i64 = if (i + (__sifr_chars_needle.len() as i64)) < 0 {
                (_slice_len_i64 + (i + (__sifr_chars_needle.len() as i64))).max(0)
            } else {
                (i + (__sifr_chars_needle.len() as i64)).min(_slice_len_i64)
            };
            String::from_iter(
                _slice_src
                    .iter()
                    .skip(_slice_start_i64 as usize)
                    .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                    .copied(),
            )
        }) == needle.clone())
        {
            return i;
        }
        i += 1_i64;
    }
    -(1_i64)
}
fn _finditer_from_items(
    found_items: &Vec<String>,
    text: &String,
) -> Vec<__SifrStdlib_sifr_x2ere_x2eMatch> {
    let mut matches: Vec<__SifrStdlib_sifr_x2ere_x2eMatch> = vec![];
    let mut cursor: i64 = 0_i64;
    for found in found_items.iter().cloned() {
        let __sifr_chars_found: Vec<char> = found.chars().collect::<Vec<char>>();
        let mut start: i64 = _find_index_from(text, &found, cursor);
        if start < (0_i64) {
            start = cursor;
        }
        let found_len: i64 = __sifr_chars_found.len() as i64;
        let end: i64 = start + found_len;
        matches.push(__SifrStdlib_sifr_x2ere_x2eMatch::new(found, start, end));
        if found_len == (0_i64) {
            cursor = end + (1_i64);
        } else {
            cursor = end;
        }
    }
    matches
}
fn compile(pattern: &String) -> Result<__SifrStdlib_sifr_x2ere_x2ePattern, RegexError> {
    let __sifr_try_res: Result<
        Result<__SifrStdlib_sifr_x2ere_x2ePattern, RegexError>,
        RegexError,
    > = (|| {
        let compiled: __SifrStdlib___sifr_x2eregex_x2eCompiledPattern = compile_pattern(
            pattern,
        )?;
        return Ok(
            Ok(
                __SifrStdlib_sifr_x2ere_x2ePattern::new(
                    compiled,
                    (pattern).clone(),
                    0_i64,
                ),
            ),
        );
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(RegexError::new(error.message));
        }
    }
}
fn fullmatch(pattern: &String, text: &String) -> Result<bool, RegexError> {
    let anchored: String = {
        let mut __sifr_concat: String = String::with_capacity(
            (1usize + pattern.len()) + 1usize,
        );
        __sifr_concat.push('^');
        __sifr_concat.push_str((pattern).as_str());
        __sifr_concat.push('$');
        __sifr_concat
    };
    re_match(&anchored, text)
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
struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for Error {
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

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Self::new(err.message)
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Self::new(err.message)
    }
}

impl From<ValueError> for Error {
    fn from(err: ValueError) -> Self {
        Self::new(err.message)
    }
}

impl From<RegexError> for Error {
    fn from(err: RegexError) -> Self {
        Self::new(err.message)
    }
}

fn main() {
    let mut d: __SifrStdlib_sifr_x2ecollections_x2edeque<i64> = __SifrStdlib_sifr_x2ecollections_x2edeque::new(None, Some(3_i64));
    d.append(&(1_i64));
    d.append(&(2_i64));
    d.append(&(3_i64));
    d.append(&(4_i64));
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(23usize + 0usize);
    __sifr_concat.push_str("deque len (maxlen=3) = ");
    __sifr_concat.push_str((format!("{}", d.len() as i64)).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
    __sifr_concat.push_str("deque popleft = ");
    __sifr_concat.push_str(((d.popleft()).map_or("None".to_string().to_string(), |__v| format!("{}", __v))).as_str());
    __sifr_concat
});
    let dt: __SifrStdlib_sifr_x2edatetime_x2edatetime = __SifrStdlib_sifr_x2edatetime_x2edatetime::new(2024_i64, 6_i64, 15_i64, 9_i64, 30_i64, 0_i64, 0_i64, None);
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(21usize + 0usize);
    __sifr_concat.push_str("datetime isoformat = ");
    __sifr_concat.push_str((dt.isoformat()).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
    __sifr_concat.push_str("datetime year = ");
    __sifr_concat.push_str((format!("{}", dt.year)).as_str());
    __sifr_concat
});
    let today: __SifrStdlib_sifr_x2edatetime_x2edate = __SifrStdlib_sifr_x2edatetime_x2edate::new(2024_i64, 6_i64, 15_i64);
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(17usize + 0usize);
    __sifr_concat.push_str("date isoformat = ");
    __sifr_concat.push_str((today.isoformat()).as_str());
    __sifr_concat
});
    let p: __SifrStdlib_sifr_x2epathlib_x2ePath = __SifrStdlib_sifr_x2epathlib_x2ePath::new("/tmp/demo_file.txt".to_string());
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _ = p.touch()?;
    println!("path touch ok = true");
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(14usize + 0usize);
    __sifr_concat.push_str("path exists = ");
    __sifr_concat.push_str((format!("{}", p.exists())).as_str());
    __sifr_concat
});
    let _2: () = p.unlink()?;
    println!("path unlink ok = true");
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(12usize + 0usize);
    __sifr_concat.push_str("path error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
    let p2: __SifrStdlib_sifr_x2epathlib_x2ePath = __SifrStdlib_sifr_x2epathlib_x2ePath::new("/tmp/myfile.txt".to_string());
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(14usize + 0usize);
    __sifr_concat.push_str("with_suffix = ");
    __sifr_concat.push_str((p2.with_suffix(&".csv".to_string()).to_str()).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(12usize + 0usize);
    __sifr_concat.push_str("with_name = ");
    __sifr_concat.push_str((p2.with_name(&"other.txt".to_string()).to_str()).as_str());
    __sifr_concat
});
    let __sifr_try_res: Result<(), RegexError> = (|| {
    let pat: __SifrStdlib_sifr_x2ere_x2ePattern = compile(&"\\d+".to_string())?;
    let m: bool = pat.is_match(&"abc123".to_string())?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(19usize + 0usize);
    __sifr_concat.push_str("pattern is_match = ");
    __sifr_concat.push_str((format!("{}", m)).as_str());
    __sifr_concat
});
    let found: Option<String> = pat.search(&"hello 42 world".to_string())?;
    if let Some(found) = found {
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(23usize + 0usize);
    __sifr_concat.push_str("pattern search found = ");
    __sifr_concat.push_str((format!("{}", (found.chars().count() as i64) > (0_i64))).as_str());
    __sifr_concat
});
    }
    let nums: Vec<String> = pat.findall(&"1 plus 2 equals 3".to_string())?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(24usize + 0usize);
    __sifr_concat.push_str("pattern findall count = ");
    __sifr_concat.push_str((format!("{}", nums.len() as i64)).as_str());
    __sifr_concat
});
    let __sifr_try_res: Result<(), RegexError> = (|| {
    let fm_val: bool = fullmatch(&"\\d+".to_string(), &"12345".to_string())?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(19usize + 0usize);
    __sifr_concat.push_str("fullmatch digits = ");
    __sifr_concat.push_str((format!("{}", fm_val)).as_str());
    __sifr_concat
});
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e2 = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(17usize + 0usize);
    __sifr_concat.push_str("fullmatch error: ");
    __sifr_concat.push_str((e2.message).as_str());
    __sifr_concat
});
    }
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(13usize + 0usize);
    __sifr_concat.push_str("regex error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
    let mut log: __SifrStdlib_sifr_x2elogging_x2eLogger = getLogger(&"demo".to_string());
    log.set_level(DEBUG);
    log.debug(&"debug message".to_string());
    log.info(&"info message".to_string());
    log.warning(&"warning message".to_string());
    let csv_text: String = "name,age\nalice,30\nbob,25".to_string();
    let r: __SifrStdlib_sifr_x2ecsv_x2ereader = __SifrStdlib_sifr_x2ecsv_x2ereader::new(csv_text, None, ",".to_string(), "\"".to_string(), "".to_string(), true, false, 0_i64);
    let all_rows: Vec<Vec<String>> = r.rows();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(11usize + 0usize);
    __sifr_concat.push_str("csv rows = ");
    __sifr_concat.push_str((format!("{}", all_rows.len() as i64)).as_str());
    __sifr_concat
});
    let mut w: __SifrStdlib_sifr_x2ecsv_x2ewriter = __SifrStdlib_sifr_x2ecsv_x2ewriter::new(None, ",".to_string(), "\"".to_string(), "".to_string(), true, false, "\n".to_string(), 0_i64);
    let row1: Vec<String> = vec!["x".to_string(), "y".to_string()];
    let row2: Vec<String> = vec!["1".to_string(), "2".to_string()];
    w.writerow(&row1);
    w.writerow(&row2);
    let out: String = w.getvalue();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(20usize + out.len());
    __sifr_concat.push_str("csv writer output = ");
    __sifr_concat.push_str((out).as_str());
    __sifr_concat
});
    let dr: __SifrStdlib_sifr_x2ecsv_x2eDictReader = __SifrStdlib_sifr_x2ecsv_x2eDictReader::new("name,score\nalice,95\nbob,87".to_string(), None, "".to_string(), "".to_string(), None, ",".to_string(), "\"".to_string(), "".to_string(), true, false, 0_i64);
    let headers: Vec<String> = dr.fieldnames();
    let first_header: Option<String> = {
    let __sifr_index_list = &headers;
    let __sifr_index_i = 0_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).cloned()
};
    if let Some(first_header) = first_header {
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(21usize + first_header.len());
    __sifr_concat.push_str("dictreader headers = ");
    __sifr_concat.push_str((first_header).as_str());
    __sifr_concat
});
    }
    let dict_rows: Vec<HashMap<String, String>> = dr.rows();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(23usize + 0usize);
    __sifr_concat.push_str("dictreader row count = ");
    __sifr_concat.push_str((format!("{}", dict_rows.len() as i64)).as_str());
    __sifr_concat
});
    let mut dw: __SifrStdlib_sifr_x2ecsv_x2eDictWriter = __SifrStdlib_sifr_x2ecsv_x2eDictWriter::new(vec!["name".to_string(), "score".to_string()], "".to_string(), "raise".to_string(), None, ",".to_string(), "\"".to_string(), "".to_string(), true, false, "\n".to_string(), 0_i64);
    dw.writeheader();
    let row_data: HashMap<String, String> = HashMap::from([("name".to_string(), "charlie".to_string()), ("score".to_string(), "91".to_string())]);
    dw.writerow(&row_data);
    let dw_out: String = dw.getvalue();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(20usize + dw_out.len());
    __sifr_concat.push_str("dictwriter output = ");
    __sifr_concat.push_str((dw_out).as_str());
    __sifr_concat
});
}
