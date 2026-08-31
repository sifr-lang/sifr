// src/main.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct __SifrIoNativeFileHandle {
    pub _id: String,
}
impl __SifrIoNativeFileHandle {
    pub fn new(id: String) -> Self {
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

mod __sifr_project_nominals {
    use crate::__SifrIoNativeFileHandle;
    pub use ::std::collections::HashMap;
    pub use ::std::collections::VecDeque;
    pub use ::sifr_runtime::SifrInt;
    pub type __SifrStdlib___sifr_x2eregex_x2eCompiledPattern = ::sifr_runtime::interop::Handle<
        ::sifr_stdlib::regex::CompiledPattern,
    >;
    pub trait __SifrOpaque__SifrStdlib___sifr_x2eregex_x2eCompiledPatternMethods {
        fn search(&self, text: &String) -> Result<Option<String>, RegexError>;
        fn is_match(&self, text: &String) -> Result<bool, RegexError>;
        fn sub(&self, replacement: &String, text: &String) -> Result<String, RegexError>;
        fn findall(&self, text: &String) -> Result<Vec<String>, RegexError>;
        fn split(&self, text: &String) -> Result<Vec<String>, RegexError>;
        fn pattern(&self) -> Result<String, RegexError>;
        fn flags(&self) -> Result<SifrInt, RegexError>;
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
        fn flags(&self) -> Result<SifrInt, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_flags(self)
                .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
                .map_err(|__sifr_bridge_error| RegexError {
                    message: __sifr_bridge_error.to_string(),
                    detail: __sifr_bridge_error.to_string(),
                })
        }
    }
    pub fn compile_pattern(
        pattern: &String,
    ) -> Result<__SifrStdlib___sifr_x2eregex_x2eCompiledPattern, RegexError> {
        ::sifr_stdlib::regex::compile_pattern(pattern)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn compile_pattern_flags(
        pattern: &String,
        flags: SifrInt,
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
    pub fn re_match(pattern: &String, text: &String) -> Result<bool, RegexError> {
        ::sifr_stdlib::regex::re_match(pattern, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn re_find(pattern: &String, text: &String) -> Result<Option<String>, RegexError> {
        ::sifr_stdlib::regex::re_find(pattern, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn re_replace(
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
    pub fn re_findall(pattern: &String, text: &String) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::re_findall(pattern, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn re_split(pattern: &String, text: &String) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::re_split(pattern, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn re_find_start(pattern: &String, text: &String) -> Result<SifrInt, RegexError> {
        ::sifr_stdlib::regex::re_find_start(pattern, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn re_find_end(pattern: &String, text: &String) -> Result<SifrInt, RegexError> {
        ::sifr_stdlib::regex::re_find_end(pattern, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn re_match_flags(
        pattern: &String,
        text: &String,
        flags: SifrInt,
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
    pub fn re_find_flags(
        pattern: &String,
        text: &String,
        flags: SifrInt,
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
    pub fn re_replace_flags(
        pattern: &String,
        replacement: &String,
        text: &String,
        flags: SifrInt,
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
    pub fn re_findall_flags(
        pattern: &String,
        text: &String,
        flags: SifrInt,
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
    pub fn re_split_flags(
        pattern: &String,
        text: &String,
        flags: SifrInt,
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
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub _data: VecDeque<T>,
        pub maxlen: Option<SifrInt>,
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn new(items: Option<Vec<T>>, maxlen: Option<SifrInt>) -> Self {
            let mut data: Vec<T> = vec![];
            if let Some(items) = items {
                let mut start: SifrInt = SifrInt::from_i64(0);
                if let Some(maxlen) = maxlen.clone() {
                    if (&SifrInt::from(items.len()) > &maxlen) {
                        start = &SifrInt::from(items.len()) - &maxlen;
                    }
                }
                let mut i: SifrInt = start;
                while (&i < &SifrInt::from(items.len())) {
                    let item: Option<T> = {
                        let __sifr_checked_read_collection = &items;
                        let __sifr_checked_read_index = i.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(__sifr_checked_read_collection.len());
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(item) = item {
                        data.push(item.clone());
                    }
                    i = &i + &SifrInt::from_i64(1);
                }
            }
            let __sifr_field_init_0: Option<SifrInt> = maxlen.clone();
            let __sifr_field_init_1: VecDeque<T> = VecDeque::from(data);
            Self {
                maxlen: __sifr_field_init_0,
                _data: __sifr_field_init_1,
            }
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn append(&mut self, val: &T) {
            self._data.push_back(val.clone());
            let maxlen_opt: Option<SifrInt> = self.maxlen.clone();
            if let Some(maxlen_opt) = maxlen_opt.clone() {
                let maxlen: SifrInt = maxlen_opt.clone();
                if (&SifrInt::from(self._data.len()) > &maxlen) {
                    self._data.pop_front();
                }
            }
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn appendleft(&mut self, val: &T) {
            self._data.push_front(val.clone());
            let maxlen_opt: Option<SifrInt> = self.maxlen.clone();
            if let Some(maxlen_opt) = maxlen_opt.clone() {
                let maxlen: SifrInt = maxlen_opt.clone();
                if (&SifrInt::from(self._data.len()) > &maxlen) {
                    self._data.pop_back();
                }
            }
        }
    }
    impl<T> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn pop(&mut self) -> Option<T> {
            if (&SifrInt::from(self._data.len()) == &SifrInt::from_i64(0)) {
                return None;
            }
            Some({
                let __sifr_nonempty_pop_index = self._data.len() - (1_usize);
                let mut __sifr_nonempty_pop_values = self
                    ._data
                    .drain(__sifr_nonempty_pop_index..__sifr_nonempty_pop_index + (1_usize))
                    .collect::<Vec<_>>();
                __sifr_nonempty_pop_values.remove(0_usize)
            })
        }
    }
    impl<T> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn popleft(&mut self) -> Option<T> {
            if (&SifrInt::from(self._data.len()) == &SifrInt::from_i64(0)) {
                return None;
            }
            Some({
                let __sifr_nonempty_pop_index = 0_usize;
                let mut __sifr_nonempty_pop_values = self
                    ._data
                    .drain(__sifr_nonempty_pop_index..__sifr_nonempty_pop_index + (1_usize))
                    .collect::<Vec<_>>();
                __sifr_nonempty_pop_values.remove(0_usize)
            })
        }
    }
    impl<T> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn len(&self) -> SifrInt {
            SifrInt::from(self._data.len())
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn to_list(&self) -> Vec<T> {
            let mut result: Vec<T> = vec![];
            for v in self._data.clone().iter().cloned() {
                result.push(v.clone());
            }
            result
        }
    }
    impl<T> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn clear(&mut self) {
            self._data.clear();
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn extend(&mut self, items: &Vec<T>) {
            for v in items.iter().cloned() {
                self._data.push_back(v.clone());
            }
            let maxlen_opt: Option<SifrInt> = self.maxlen.clone();
            if let Some(maxlen_opt) = maxlen_opt.clone() {
                let maxlen: SifrInt = maxlen_opt.clone();
                while (&SifrInt::from(self._data.len()) > &maxlen) {
                    self._data.pop_front();
                }
            }
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn extendleft(&mut self, items: &Vec<T>) {
            for v in items.iter().cloned() {
                self._data.push_front(v.clone());
            }
            let maxlen_opt: Option<SifrInt> = self.maxlen.clone();
            if let Some(maxlen_opt) = maxlen_opt.clone() {
                let maxlen: SifrInt = maxlen_opt.clone();
                while (&SifrInt::from(self._data.len()) > &maxlen) {
                    self._data.pop_back();
                }
            }
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn copy(&self) -> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
            __SifrStdlib_sifr_x2ecollections_x2edeque::new(
                Some(self.to_list()),
                self.maxlen.clone(),
            )
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn reverse(&mut self) {
            let mut items: Vec<T> = self.to_list();
            items.reverse();
            self._data.clear();
            for item in items.iter().cloned() {
                self._data.push_back(item.clone());
            }
        }
    }
    impl<T: Clone> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn rotate(&mut self, n: &SifrInt) {
            let length: SifrInt = SifrInt::from(self._data.len());
            if &length == &SifrInt::from_i64(0) {
                return;
            }
            let mut steps: SifrInt = n.floor_mod_known_nonzero(&length);
            if &steps < &SifrInt::from_i64(0) {
                steps = &steps + &length;
            }
            let mut count: SifrInt = SifrInt::from_i64(0);
            while (&count < &steps) {
                let value: Option<T> = self._data.pop_back();
                if let Some(value) = value {
                    self._data.push_front(value.clone());
                }
                count = &count + &SifrInt::from_i64(1);
            }
        }
    }
    impl<T: Clone + PartialEq> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn count(&self, value: &T) -> SifrInt {
            let mut total: SifrInt = SifrInt::from_i64(0);
            for item in self._data.clone().iter().cloned() {
                if item == *value {
                    total = &total + &SifrInt::from_i64(1);
                }
            }
            total
        }
    }
    impl<T: Clone + PartialEq> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn index(
            &self,
            value: &T,
            start: &SifrInt,
            stop: &Option<SifrInt>,
        ) -> Option<SifrInt> {
            let size: SifrInt = SifrInt::from(self._data.len());
            let mut begin: SifrInt = start.clone();
            if &begin < &SifrInt::from_i64(0) {
                begin = &size + &begin;
                if &begin < &SifrInt::from_i64(0) {
                    begin = SifrInt::from_i64(0);
                }
            }
            let mut end: SifrInt = size.clone();
            if let Some(stop) = stop.as_ref() {
                end = stop.clone();
                if (&end < &SifrInt::from_i64(0)) {
                    end = &size + &end;
                }
                if (&end < &SifrInt::from_i64(0)) {
                    end = SifrInt::from_i64(0);
                }
                if (&end > &size) {
                    end = size;
                }
            }
            let mut i: SifrInt = begin.clone();
            while (&i < &end) {
                let current: Option<T> = {
                    let __sifr_checked_read_collection = &self._data;
                    let __sifr_checked_read_index = i.clone();
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                };
                if let Some(current) = current {
                    if current == *value {
                        return Some(i);
                    }
                }
                i = &i + &SifrInt::from_i64(1);
            }
            None
        }
    }
    impl<T: Clone + PartialEq> __SifrStdlib_sifr_x2ecollections_x2edeque<T> {
        pub fn remove(&mut self, value: &T) {
            let idx: Option<SifrInt> = self.index(value, &SifrInt::from_i64(0), &None);
            if let Some(idx) = idx.clone() {
                let mut rebuilt: Vec<T> = vec![];
                let mut i: SifrInt = SifrInt::from_i64(0);
                while (&i < &SifrInt::from(self._data.len())) {
                    let current: Option<T> = {
                        let __sifr_checked_read_collection = &self._data;
                        let __sifr_checked_read_index = i.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(__sifr_checked_read_collection.len());
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(current) = current {
                        if (&i != &idx) {
                            rebuilt.push(current.clone());
                        }
                    }
                    i = &i + &SifrInt::from_i64(1);
                }
                self._data.clear();
                for item in rebuilt.iter().cloned() {
                    self._data.push_back(item.clone());
                }
            }
        }
    }
    pub fn _encoding_is_supported_impl(label: &String) -> bool {
        ::sifr_stdlib::encoding::encoding_is_supported(label)
    }
    pub fn _encoding_canonical_label_impl(label: &String) -> Result<String, ParseError> {
        ::sifr_stdlib::encoding::encoding_canonical_label(label)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn _encoding_decode_text_impl(
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
    pub fn _encoding_decode_recoveries_impl(
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
    pub fn _encoding_decode_incremental_text_impl(
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
    pub fn _encoding_decode_incremental_recoveries_impl(
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
    pub fn _encoding_decode_incremental_pending_impl(
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
    pub fn _encoding_encode_bytes_impl(
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
    pub fn _encoding_encode_recoveries_impl(
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
    pub fn read_text(path: &String) -> Result<String, IOError> {
        ::sifr_stdlib::fs::read_text(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn write_text(path: &String, content: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::write_text(path, content)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn exists(path: &String) -> bool {
        ::sifr_stdlib::fs::exists(path)
    }
    pub fn read_lines(path: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::read_lines(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn append_text(path: &String, content: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::append_text(path, content)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _open_file(path: &String, mode: &String) -> Result<String, IOError> {
        ::sifr_stdlib::fs::open_file(path, mode)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_read(handle: &String) -> Result<String, IOError> {
        ::sifr_stdlib::fs::file_read(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_write(handle: &String, data: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::file_write(handle, data)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_readline(handle: &String) -> Result<Option<String>, IOError> {
        ::sifr_stdlib::fs::file_readline(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_readlines(handle: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::file_readlines(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_close(handle: &String) {
        ::sifr_stdlib::fs::file_close(handle);
    }
    pub fn _file_read_bytes(
        handle: &String,
        size: Option<SifrInt>,
    ) -> Result<Vec<u8>, IOError> {
        ::sifr_stdlib::fs::file_read_bytes(
                handle,
                size.map(::sifr_runtime::interop::SifrIntBridge::from),
            )
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_write_bytes(handle: &String, data: &Vec<u8>) -> Result<(), IOError> {
        ::sifr_stdlib::fs::file_write_bytes(handle, data)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_flush(handle: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::file_flush(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_seek(
        handle: &String,
        offset: SifrInt,
        whence: SifrInt,
    ) -> Result<SifrInt, IOError> {
        ::sifr_stdlib::fs::file_seek(
                handle,
                ::sifr_runtime::interop::SifrIntBridge::from(offset),
                ::sifr_runtime::interop::SifrIntBridge::from(whence),
            )
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_tell(handle: &String) -> Result<SifrInt, IOError> {
        ::sifr_stdlib::fs::file_tell(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn open_file(
        path: &String,
        mode: &String,
    ) -> Result<__SifrIoNativeFileHandle, IOError> {
        let __sifr_try_res: Result<Result<__SifrIoNativeFileHandle, IOError>, IOError> = (|| {
            let handle_id: String = _open_file(path, mode)?;
            Ok(Ok(__SifrIoNativeFileHandle::new(handle_id)))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(e);
            }
        }
    }
    pub fn file_read(handle: &__SifrIoNativeFileHandle) -> Result<String, IOError> {
        _file_read(&handle._id.clone())
    }
    pub fn file_write(
        handle: &__SifrIoNativeFileHandle,
        data: &String,
    ) -> Result<(), IOError> {
        _file_write(&handle._id.clone(), data)
    }
    pub fn file_readline(
        handle: &__SifrIoNativeFileHandle,
    ) -> Result<Option<String>, IOError> {
        _file_readline(&handle._id.clone())
    }
    pub fn file_readlines(
        handle: &__SifrIoNativeFileHandle,
    ) -> Result<Vec<String>, IOError> {
        _file_readlines(&handle._id.clone())
    }
    pub fn file_close(handle: &__SifrIoNativeFileHandle) {
        _file_close(&handle._id.clone());
    }
    pub fn file_read_bytes(
        handle: &__SifrIoNativeFileHandle,
        size: Option<SifrInt>,
    ) -> Result<Vec<u8>, IOError> {
        _file_read_bytes(&handle._id.clone(), (size).clone())
    }
    pub fn file_write_bytes(
        handle: &__SifrIoNativeFileHandle,
        data: &Vec<u8>,
    ) -> Result<(), IOError> {
        _file_write_bytes(&handle._id.clone(), data)
    }
    pub fn file_flush(handle: &__SifrIoNativeFileHandle) -> Result<(), IOError> {
        _file_flush(&handle._id.clone())
    }
    pub fn file_seek(
        handle: &__SifrIoNativeFileHandle,
        offset: SifrInt,
        whence: SifrInt,
    ) -> Result<SifrInt, IOError> {
        _file_seek(&handle._id.clone(), (offset).clone(), (whence).clone())
    }
    pub fn file_tell(handle: &__SifrIoNativeFileHandle) -> Result<SifrInt, IOError> {
        _file_tell(&handle._id.clone())
    }
    pub fn getcwd() -> Result<String, IOError> {
        ::sifr_stdlib::fs::getcwd()
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn listdir(path: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::listdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn mkdir(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::mkdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn rmdir(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rmdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn remove_file(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::remove_file(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn rename(src: &String, dst: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rename(src, dst)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn chdir(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::chdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn stat_size(path: &String) -> Result<SifrInt, IOError> {
        ::sifr_stdlib::fs::stat_size(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn disk_usage(path: &String) -> Vec<SifrInt> {
        ::sifr_stdlib::fs::disk_usage(path)
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
            .collect()
    }
    pub fn is_file(path: &String) -> bool {
        ::sifr_stdlib::fs::is_file(path)
    }
    pub fn is_dir(path: &String) -> bool {
        ::sifr_stdlib::fs::is_dir(path)
    }
    pub fn copy_file(src: &String, dst: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::copy_file(src, dst)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn walk_dir(path: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::walk_dir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn rmdir_all(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rmdir_all(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn gettempdir() -> String {
        ::sifr_stdlib::fs::gettempdir()
    }
    pub fn makedirs(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::makedirs(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn touch(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::touch(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn resolve_path(path: &String) -> Result<String, IOError> {
        ::sifr_stdlib::fs::resolve_path(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn iterdir(path: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::iterdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn glob_pattern(dir: &String, pattern: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::glob_pattern(dir, pattern)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn rglob_pattern(dir: &String, pattern: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::rglob_pattern(dir, pattern)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn __const_ENCODING_UTF8() -> String {
        "utf-8".to_string().to_string()
    }
    pub fn __const_ENCODING_UTF8_SIG() -> String {
        "utf-8-sig".to_string().to_string()
    }
    pub fn __const_ENCODING_ASCII() -> String {
        "ascii".to_string().to_string()
    }
    pub fn __const_ENCODING_LATIN1() -> String {
        "latin-1".to_string().to_string()
    }
    pub fn __const_ENCODING_UTF16_LE() -> String {
        "utf-16-le".to_string().to_string()
    }
    pub fn __const_ENCODING_UTF16_BE() -> String {
        "utf-16-be".to_string().to_string()
    }
    pub fn __const_ENCODING_WINDOWS_1250() -> String {
        "windows-1250".to_string().to_string()
    }
    pub fn __const_ENCODING_WINDOWS_1251() -> String {
        "windows-1251".to_string().to_string()
    }
    pub fn __const_ENCODING_WINDOWS_1252() -> String {
        "windows-1252".to_string().to_string()
    }
    pub fn __const_ENCODING_WINDOWS_1253() -> String {
        "windows-1253".to_string().to_string()
    }
    pub fn __const_ENCODING_WINDOWS_1254() -> String {
        "windows-1254".to_string().to_string()
    }
    pub fn __const_ENCODING_WINDOWS_1255() -> String {
        "windows-1255".to_string().to_string()
    }
    pub fn __const_ENCODING_WINDOWS_1256() -> String {
        "windows-1256".to_string().to_string()
    }
    pub fn __const_ENCODING_WINDOWS_1257() -> String {
        "windows-1257".to_string().to_string()
    }
    pub fn __const_ENCODING_WINDOWS_1258() -> String {
        "windows-1258".to_string().to_string()
    }
    pub fn __const_DECODE_ERRORS_STRICT() -> String {
        "strict".to_string().to_string()
    }
    pub fn __const_DECODE_ERRORS_REPLACE() -> String {
        "replace".to_string().to_string()
    }
    pub fn __const_DECODE_ERRORS_IGNORE() -> String {
        "ignore".to_string().to_string()
    }
    pub fn __const_DECODE_ERRORS_BACKSLASH_REPLACE() -> String {
        "backslashreplace".to_string().to_string()
    }
    pub fn __const_ENCODE_ERRORS_STRICT() -> String {
        "strict".to_string().to_string()
    }
    pub fn __const_ENCODE_ERRORS_REPLACE() -> String {
        "replace".to_string().to_string()
    }
    pub fn __const_ENCODE_ERRORS_IGNORE() -> String {
        "ignore".to_string().to_string()
    }
    pub fn __const_ENCODE_ERRORS_BACKSLASH_REPLACE() -> String {
        "backslashreplace".to_string().to_string()
    }
    pub fn __const_ENCODE_ERRORS_XMLCHARREF_REPLACE() -> String {
        "xmlcharrefreplace".to_string().to_string()
    }
    pub fn __const_ENCODE_ERRORS_NAME_REPLACE() -> String {
        "namereplace".to_string().to_string()
    }
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eencoding_x2eDecodeError {
        pub message: String,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eDecodeError {
        pub fn new(message: String) -> Self {
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
    pub struct __SifrStdlib_sifr_x2eencoding_x2eEncodeError {
        pub message: String,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eEncodeError {
        pub fn new(message: String) -> Self {
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
    pub struct __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        pub label: String,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        pub fn new(label: String) -> Self {
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
        pub fn canonical_label(
            &self,
        ) -> Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
            _encoding_canonical_label(&self.label)
        }
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        pub fn is_supported(&self) -> bool {
            _encoding_is_supported(&self.label)
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "Encoding(label={})", self.label)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
        pub name: String,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
        pub fn new(name: String) -> Self {
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
    pub struct __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        pub name: String,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        pub fn new(name: String) -> Self {
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
    pub struct __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
        pub text: String,
        pub recoveries: Vec<String>,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
        pub fn new(text: String, recoveries: Vec<String>) -> Self {
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
        pub fn get_text(&self) -> String {
            {
                let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
                __sifr_concat.push_str((self.text.clone()).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            }
        }
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
        pub fn get_recoveries(&self) -> Vec<String> {
            self.recoveries.clone()
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
        pub data: Vec<u8>,
        pub recoveries: Vec<String>,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
        pub fn new(data: Vec<u8>, recoveries: Vec<String>) -> Self {
            let __sifr_field_init_0: Vec<u8> = data;
            let __sifr_field_init_1: Vec<String> = recoveries;
            Self {
                data: __sifr_field_init_0,
                recoveries: __sifr_field_init_1,
            }
        }
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
        pub fn get_data(&self) -> Vec<u8> {
            self.data.clone()
        }
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
        pub fn get_recoveries(&self) -> Vec<String> {
            self.recoveries.clone()
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eencoding_x2eDecoder {
        pub _encoding: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
        pub _errors: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler,
        pub _exhausted: bool,
        pub _pending: Vec<u8>,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eDecoder {
        pub fn new(
            enc: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
            errors: Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
        ) -> Self {
            let __sifr_field_init_0: __SifrStdlib_sifr_x2eencoding_x2eEncoding = enc;
            let __sifr_field_init_1: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler = _decode_handler_or_strict(
                &errors,
            );
            let __sifr_field_init_2: bool = false;
            let __sifr_field_init_3: Vec<u8> = {
                let __sifr_empty_bytes_literal: Vec<u8> = vec![];
                __sifr_empty_bytes_literal
            };
            Self {
                _encoding: __sifr_field_init_0,
                _errors: __sifr_field_init_1,
                _exhausted: __sifr_field_init_2,
                _pending: __sifr_field_init_3,
            }
        }
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eDecoder {
        pub fn decode(
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
                let outcome: __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome = _encoding_decode_incremental_outcome(
                    data,
                    &self._pending,
                    &self._encoding.clone().label.clone(),
                    &self._errors.clone().name.clone(),
                    r#final,
                )?;
                let next_pending: Vec<u8> = _encoding_decode_incremental_pending(
                    data,
                    &self._pending,
                    &self._encoding.clone().label.clone(),
                    r#final,
                )?;
                self._pending = next_pending;
                if r#final {
                    self._pending = {
                        let __sifr_empty_bytes_literal: Vec<u8> = vec![];
                        __sifr_empty_bytes_literal
                    };
                    self._exhausted = true;
                }
                Ok(Ok(outcome))
            })();
            match __sifr_try_res {
                Ok(__sifr_ret_val) => {
                    return __sifr_ret_val;
                }
                Err(__sifr_try_err) => {
                    let e = __sifr_try_err.clone();
                    return Err(
                        __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
                    );
                }
            }
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eencoding_x2eEncoder {
        pub _encoding: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
        pub _errors: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler,
        pub _exhausted: bool,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eEncoder {
        pub fn new(
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
        pub fn encode(
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
                Ok(Ok(outcome))
            })();
            match __sifr_try_res {
                Ok(__sifr_ret_val) => {
                    return __sifr_ret_val;
                }
                Err(__sifr_try_err) => {
                    let e = __sifr_try_err.clone();
                    return Err(
                        __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
                    );
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
    pub fn _encoding_is_supported(label: &String) -> bool {
        _encoding_is_supported_impl(label)
    }
    pub fn _encoding_canonical_label(
        label: &String,
    ) -> Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
        let __sifr_try_res: Result<
            Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
            ParseError,
        > = (|| {
            let value: String = _encoding_canonical_label_impl(label)?;
            Ok(Ok(value))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
                );
            }
        }
    }
    pub fn _encoding_decode_text(
        data: &Vec<u8>,
        encoding: &String,
        errors: &String,
    ) -> Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
        let __sifr_try_res: Result<
            Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
            ParseError,
        > = (|| {
            let text: String = _encoding_decode_text_impl(data, encoding, errors)?;
            Ok(Ok(text))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
                );
            }
        }
    }
    pub fn _encoding_decode_recoveries(
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
            Ok(Ok(recoveries))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
                );
            }
        }
    }
    pub fn _encoding_decode_outcome(
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
            Ok(Ok(__SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome::new(text, recoveries)))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
                );
            }
        }
    }
    pub fn _encoding_decode_incremental_outcome(
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
            Ok(Ok(__SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome::new(text, recoveries)))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
                );
            }
        }
    }
    pub fn _encoding_decode_incremental_pending(
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
            Ok(Ok(next_pending))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
                );
            }
        }
    }
    pub fn _encoding_encode_bytes(
        text: &String,
        encoding: &String,
        errors: &String,
    ) -> Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError> {
        let __sifr_try_res: Result<
            Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError>,
            ParseError,
        > = (|| {
            let data: Vec<u8> = _encoding_encode_bytes_impl(text, encoding, errors)?;
            Ok(Ok(data))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
                );
            }
        }
    }
    pub fn _encoding_encode_recoveries(
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
            Ok(Ok(recoveries))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
                );
            }
        }
    }
    pub fn _encoding_encode_outcome(
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
            Ok(Ok(__SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome::new(data, recoveries)))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
                );
            }
        }
    }
    pub fn encoding(label: &String) -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new((label.clone()).clone())
    }
    pub fn utf8() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF8())
    }
    pub fn utf8_sig() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF8_SIG())
    }
    pub fn ascii() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_ASCII())
    }
    pub fn latin1() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_LATIN1())
    }
    pub fn utf16_le() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF16_LE())
    }
    pub fn utf16_be() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF16_BE())
    }
    pub fn windows1252() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_WINDOWS_1252())
    }
    pub fn strict_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
            __const_DECODE_ERRORS_STRICT(),
        )
    }
    pub fn replace_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
            __const_DECODE_ERRORS_REPLACE(),
        )
    }
    pub fn ignore_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
            __const_DECODE_ERRORS_IGNORE(),
        )
    }
    pub fn backslash_replace_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
            __const_DECODE_ERRORS_BACKSLASH_REPLACE(),
        )
    }
    pub fn strict_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
            __const_ENCODE_ERRORS_STRICT(),
        )
    }
    pub fn replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
            __const_ENCODE_ERRORS_REPLACE(),
        )
    }
    pub fn ignore_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
            __const_ENCODE_ERRORS_IGNORE(),
        )
    }
    pub fn backslash_replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
            __const_ENCODE_ERRORS_BACKSLASH_REPLACE(),
        )
    }
    pub fn xmlcharref_replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
            __const_ENCODE_ERRORS_XMLCHARREF_REPLACE(),
        )
    }
    pub fn name_replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
            __const_ENCODE_ERRORS_NAME_REPLACE(),
        )
    }
    pub fn _decode_handler_name(
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
    pub fn _encode_handler_name(
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
    pub fn _decode_handler_or_strict(
        errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
    ) -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
        if let Some(errors) = errors.as_ref() {
            return __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
                format!("{}{}", errors.name.clone(), ""),
            );
        }
        strict_decode_handler()
    }
    pub fn _encode_handler_or_strict(
        errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler>,
    ) -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        if let Some(errors) = errors.as_ref() {
            return __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
                format!("{}{}", errors.name.clone(), ""),
            );
        }
        strict_encode_handler()
    }
    pub fn decode_outcome(
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
        > = (|| { Ok(_encoding_decode_outcome(data, &enc.label.clone(), &handler_name)) })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
                );
            }
        }
    }
    pub fn decode(
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
            Ok(Ok(outcome.get_text()))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
                );
            }
        }
    }
    pub fn encode_outcome(
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
        > = (|| { Ok(_encoding_encode_outcome(text, &enc.label.clone(), &handler_name)) })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
                );
            }
        }
    }
    pub fn encode(
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
            Ok(Ok(outcome.get_data()))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
                );
            }
        }
    }
    #[derive(Debug, Clone)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
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
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
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
    pub struct __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub _closed: bool,
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn new() -> Self {
            let __sifr_field_init_0: bool = false;
            Self {
                _closed: __sifr_field_init_0,
            }
        }
    }
    impl ::std::default::Default for __SifrStdlib_sifr_x2eio_x2eIOBase {
        fn default() -> Self {
            Self::new()
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn close(&mut self) {
            self._closed = true;
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn closed(&self) -> bool {
            self._closed
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn flush(&self) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            Ok(())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn seek(&self, offset: &SifrInt, whence: &SifrInt) -> Result<SifrInt, IOError> {
            let _ = offset.clone();
            let _ = whence.clone();
            Err(IOError::new(_unsupported_seek_tell_error()))
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn tell(&self) -> Result<SifrInt, IOError> {
            Err(IOError::new(_unsupported_seek_tell_error()))
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn readable(&self) -> bool {
            false
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn writable(&self) -> bool {
            false
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn seekable(&self) -> bool {
            false
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2eio_x2eIOBase {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "IOBase(_closed={})", self._closed)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eio_x2eTextIOBase {
        pub iobase: __SifrStdlib_sifr_x2eio_x2eIOBase,
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
    pub struct __SifrStdlib_sifr_x2eio_x2eBinaryIOBase {
        pub iobase: __SifrStdlib_sifr_x2eio_x2eIOBase,
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
    pub struct __SifrIoFileHandle {
        pub _handle: __SifrIoNativeFileHandle,
        pub _mode: String,
        pub _closed: bool,
    }
    impl __SifrIoFileHandle {
        pub fn new(handle: __SifrIoNativeFileHandle, mode: String) -> Self {
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
        pub fn close(&mut self) {
            if self._closed {
                return;
            }
            file_close(&self._handle);
            self._closed = true;
        }
    }
    impl __SifrIoFileHandle {
        pub fn closed(&self) -> bool {
            self._closed
        }
    }
    impl __SifrIoFileHandle {
        pub fn flush(&self) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            file_flush(&self._handle)
        }
    }
    impl __SifrIoFileHandle {
        pub fn read(&self) -> Result<String, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !self.readable() {
                return Err(IOError::new("stream is not readable".to_string()));
            }
            file_read(&self._handle)
        }
    }
    impl __SifrIoFileHandle {
        pub fn write(&self, data: &String) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !self.writable() {
                return Err(IOError::new("stream is not writable".to_string()));
            }
            file_write(&self._handle, data)
        }
    }
    impl __SifrIoFileHandle {
        pub fn readline(&self) -> Result<Option<String>, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !self.readable() {
                return Err(IOError::new("stream is not readable".to_string()));
            }
            file_readline(&self._handle)
        }
    }
    impl __SifrIoFileHandle {
        pub fn readlines(&self) -> Result<Vec<String>, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !self.readable() {
                return Err(IOError::new("stream is not readable".to_string()));
            }
            file_readlines(&self._handle)
        }
    }
    impl __SifrIoFileHandle {
        pub fn read_bytes(&self, size: &Option<SifrInt>) -> Result<Vec<u8>, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !self.readable() {
                return Err(IOError::new("stream is not readable".to_string()));
            }
            file_read_bytes(&self._handle, (size.clone()).clone())
        }
    }
    impl __SifrIoFileHandle {
        pub fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !self.writable() {
                return Err(IOError::new("stream is not writable".to_string()));
            }
            file_write_bytes(&self._handle, data)
        }
    }
    impl __SifrIoFileHandle {
        pub fn seek(&self, offset: &SifrInt, whence: &SifrInt) -> Result<SifrInt, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            file_seek(&self._handle, (offset.clone()).clone(), (whence.clone()).clone())
        }
    }
    impl __SifrIoFileHandle {
        pub fn tell(&self) -> Result<SifrInt, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            file_tell(&self._handle)
        }
    }
    impl __SifrIoFileHandle {
        pub fn readable(&self) -> bool {
            _mode_is_readable(&self._mode)
        }
    }
    impl __SifrIoFileHandle {
        pub fn writable(&self) -> bool {
            _mode_is_writable(&self._mode)
        }
    }
    impl __SifrIoFileHandle {
        pub fn seekable(&self) -> bool {
            !(self._closed)
        }
    }
    impl __SifrIoFileHandle {
        pub fn __enter__(&self) -> __SifrIoFileHandle {
            self.clone()
        }
    }
    impl __SifrIoFileHandle {
        pub fn __exit__(&mut self) {
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
    pub struct __SifrIoBinaryFileHandle {
        pub _handle: __SifrIoNativeFileHandle,
        pub _mode: String,
        pub _closed: bool,
    }
    impl __SifrIoBinaryFileHandle {
        pub fn new(handle: __SifrIoNativeFileHandle, mode: String) -> Self {
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
        pub fn close(&mut self) {
            if self._closed {
                return;
            }
            file_close(&self._handle);
            self._closed = true;
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn closed(&self) -> bool {
            self._closed
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn flush(&self) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            file_flush(&self._handle)
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn read_bytes(&self, size: &Option<SifrInt>) -> Result<Vec<u8>, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !self.readable() {
                return Err(IOError::new("stream is not readable".to_string()));
            }
            file_read_bytes(&self._handle, (size.clone()).clone())
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !self.writable() {
                return Err(IOError::new("stream is not writable".to_string()));
            }
            file_write_bytes(&self._handle, data)
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn seek(&self, offset: &SifrInt, whence: &SifrInt) -> Result<SifrInt, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            file_seek(&self._handle, (offset.clone()).clone(), (whence.clone()).clone())
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn tell(&self) -> Result<SifrInt, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            file_tell(&self._handle)
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn readable(&self) -> bool {
            _mode_is_readable(&self._mode)
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn writable(&self) -> bool {
            _mode_is_writable(&self._mode)
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn seekable(&self) -> bool {
            !(self._closed)
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn __enter__(&self) -> __SifrIoBinaryFileHandle {
            self.clone()
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn __exit__(&mut self) {
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
    pub struct __SifrIoTextFileHandle {
        pub _binary: __SifrIoBinaryFileHandle,
        pub _encoding: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
        pub _decode_errors: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler,
        pub _encode_errors: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler,
    }
    impl __SifrIoTextFileHandle {
        pub fn new(
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
        pub fn close(&mut self) {
            self._binary.close();
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn closed(&self) -> bool {
            self._binary.closed()
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn flush(&self) -> Result<(), IOError> {
            self._binary.flush()
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn read(&self) -> Result<String, IOError> {
            let __sifr_try_res: Result<
                Result<String, IOError>,
                __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0,
            > = (|| {
                let data: Vec<u8> = (self._binary.read_bytes(&None))
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
                Ok(Ok(text))
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
                            return Err(e);
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
                                    __sifr_concat.push_str((e.message.clone()).as_str());
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
        pub fn write(&self, text: &String) -> Result<(), IOError> {
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
                Ok(Ok(()))
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
                            return Err(e);
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
                                    __sifr_concat.push_str((e.message.clone()).as_str());
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
        pub fn readline(&self) -> Result<Option<String>, IOError> {
            Err(
                IOError::new(
                    "TextFileHandle.readline is deferred; use read().split(\"\\n\")"
                        .to_string(),
                ),
            )
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn readlines(&self) -> Result<Vec<String>, IOError> {
            Err(
                IOError::new(
                    "TextFileHandle.readlines is deferred; use read().split(\"\\n\")"
                        .to_string(),
                ),
            )
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn seek(&self, offset: &SifrInt, whence: &SifrInt) -> Result<SifrInt, IOError> {
            self._binary.seek(offset, whence)
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn tell(&self) -> Result<SifrInt, IOError> {
            self._binary.tell()
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn readable(&self) -> bool {
            self._binary.readable()
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn writable(&self) -> bool {
            self._binary.writable()
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn seekable(&self) -> bool {
            self._binary.seekable()
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn __enter__(&self) -> __SifrIoTextFileHandle {
            self.clone()
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn __exit__(&mut self) {
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
    pub struct __SifrStdlib_sifr_x2eio_x2eTextReader {
        pub _closed: bool,
    }
    impl __SifrStdlib_sifr_x2eio_x2eTextReader {
        pub fn new() -> Self {
            let __sifr_field_init_0: bool = false;
            Self {
                _closed: __sifr_field_init_0,
            }
        }
    }
    impl ::std::default::Default for __SifrStdlib_sifr_x2eio_x2eTextReader {
        fn default() -> Self {
            Self::new()
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eTextReader {
        pub fn read(&self) -> Result<String, IOError> {
            Err(
                IOError::new(
                    "TextReader direct construction is unsupported; use open_text"
                        .to_string(),
                ),
            )
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eTextReader {
        pub fn readline(&self) -> Result<Option<String>, IOError> {
            Err(
                IOError::new(
                    "TextReader.readline is deferred; use read().split(\"\\n\")".to_string(),
                ),
            )
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eTextReader {
        pub fn readlines(&self) -> Result<Vec<String>, IOError> {
            Err(
                IOError::new(
                    "TextReader.readlines is deferred; use read().split(\"\\n\")".to_string(),
                ),
            )
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eTextReader {
        pub fn close(&mut self) {
            self._closed = true;
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2eio_x2eTextReader {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "TextReader(_closed={})", self._closed)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eio_x2eTextWriter {
        pub _closed: bool,
    }
    impl __SifrStdlib_sifr_x2eio_x2eTextWriter {
        pub fn new() -> Self {
            let __sifr_field_init_0: bool = false;
            Self {
                _closed: __sifr_field_init_0,
            }
        }
    }
    impl ::std::default::Default for __SifrStdlib_sifr_x2eio_x2eTextWriter {
        fn default() -> Self {
            Self::new()
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eTextWriter {
        pub fn write(&self, text: &String) -> Result<(), IOError> {
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
        pub fn close(&mut self) {
            self._closed = true;
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2eio_x2eTextWriter {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "TextWriter(_closed={})", self._closed)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub _buffer: String,
        pub _cursor: SifrInt,
        pub _closed: bool,
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn new(initial: String) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(
                    initial.len() + 0usize,
                );
                __sifr_concat.push_str((initial).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let __sifr_field_init_1: SifrInt = SifrInt::from_i64(0);
            let __sifr_field_init_2: bool = false;
            Self {
                _buffer: __sifr_field_init_0,
                _cursor: __sifr_field_init_1,
                _closed: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn close(&mut self) {
            self._closed = true;
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn closed(&self) -> bool {
            self._closed
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn flush(&self) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            Ok(())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn read(&mut self, size: &Option<SifrInt>) -> Result<String, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            let start: SifrInt = self._cursor.clone();
            let mut end: SifrInt = SifrInt::from(self._buffer.chars().count());
            if let Some(size) = size.as_ref() {
                let maybe_size: SifrInt = size.clone();
                if (&maybe_size >= &SifrInt::from_i64(0)) {
                    let requested: SifrInt = &start + &maybe_size;
                    if (&requested < &end) {
                        end = requested;
                    }
                }
            }
            let piece: String = {
                let _slice_src = &self._buffer.clone();
                let _slice_len = _slice_src.chars().count();
                let _slice_start = start.clamp_slice_bound(_slice_len);
                let _slice_stop = end.clamp_slice_bound(_slice_len);
                String::from_iter(
                    _slice_src
                        .chars()
                        .skip(_slice_start)
                        .take(_slice_stop.saturating_sub(_slice_start)),
                )
            };
            self._cursor = end.clone();
            Ok(piece)
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn write(&mut self, data: &String) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            let left: String = {
                let _slice_src = &self._buffer.clone();
                let _slice_len = _slice_src.chars().count();
                let _slice_start = 0;
                let _slice_stop = self._cursor.clone().clamp_slice_bound(_slice_len);
                String::from_iter(
                    _slice_src
                        .chars()
                        .skip(_slice_start)
                        .take(_slice_stop.saturating_sub(_slice_start)),
                )
            };
            let tail_start: SifrInt = &self._cursor.clone()
                + &SifrInt::from(data.chars().count());
            let mut right: String = "".to_string();
            if (&tail_start < &SifrInt::from(self._buffer.chars().count())) {
                right = {
                    let _slice_src = &self._buffer.clone();
                    let _slice_len = _slice_src.chars().count();
                    let _slice_start = tail_start.clamp_slice_bound(_slice_len);
                    let _slice_stop = _slice_len;
                    String::from_iter(
                        _slice_src
                            .chars()
                            .skip(_slice_start)
                            .take(_slice_stop.saturating_sub(_slice_start)),
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
            self._cursor = &self._cursor.clone() + &SifrInt::from(data.chars().count());
            Ok(())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn getvalue(&self) -> String {
            self._buffer.clone()
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn seek(
            &mut self,
            offset: &SifrInt,
            whence: &SifrInt,
        ) -> Result<SifrInt, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            let mut origin: SifrInt = SifrInt::from_i64(0);
            if (&whence.clone() == &SifrInt::from_i64(0)) {
                origin = SifrInt::from_i64(0);
            } else {
                if (&whence.clone() == &SifrInt::from_i64(1)) {
                    origin = self._cursor.clone();
                } else {
                    if (&whence.clone() == &SifrInt::from_i64(2)) {
                        origin = SifrInt::from(self._buffer.chars().count());
                    } else {
                        return Err(
                            IOError::new(_invalid_whence_error((whence.clone()).clone())),
                        );
                    }
                }
            }
            let mut next_pos: SifrInt = &origin + offset;
            if (&next_pos < &SifrInt::from_i64(0)) {
                return Err(IOError::new(_negative_seek_error((next_pos).clone())));
            }
            let end: SifrInt = SifrInt::from(self._buffer.chars().count());
            if &next_pos > &end {
                next_pos = end.clone();
            }
            self._cursor = next_pos.clone();
            Ok(self._cursor.clone())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn tell(&self) -> Result<SifrInt, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            Ok(self._cursor.clone())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn readable(&self) -> bool {
            !(self._closed)
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn writable(&self) -> bool {
            !(self._closed)
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn seekable(&self) -> bool {
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
    pub struct __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub _buffer: Vec<u8>,
        pub _cursor: SifrInt,
        pub _closed: bool,
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn new(initial: Vec<u8>) -> Self {
            let __sifr_field_init_0: Vec<u8> = initial;
            let __sifr_field_init_1: SifrInt = SifrInt::from_i64(0);
            let __sifr_field_init_2: bool = false;
            Self {
                _buffer: __sifr_field_init_0,
                _cursor: __sifr_field_init_1,
                _closed: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn close(&mut self) {
            self._closed = true;
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn closed(&self) -> bool {
            self._closed
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn flush(&self) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            Ok(())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn read_bytes(&mut self, size: &Option<SifrInt>) -> Result<Vec<u8>, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            let start: SifrInt = self._cursor.clone();
            let mut end: SifrInt = SifrInt::from(self._buffer.len());
            if let Some(size) = size.as_ref() {
                let maybe_size: SifrInt = size.clone();
                if (&maybe_size >= &SifrInt::from_i64(0)) {
                    let requested: SifrInt = &start + &maybe_size;
                    if (&requested < &end) {
                        end = requested;
                    }
                }
            }
            let chunk: Vec<u8> = {
                let _slice_src = &self._buffer.clone();
                let _slice_len = _slice_src.len();
                let _slice_start = start.clamp_slice_bound(_slice_len);
                let _slice_stop = end.clamp_slice_bound(_slice_len);
                Vec::from_iter(
                    _slice_src
                        .iter()
                        .skip(_slice_start)
                        .take(_slice_stop.saturating_sub(_slice_start))
                        .cloned(),
                )
            };
            self._cursor = end.clone();
            Ok(chunk)
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn write_bytes(&mut self, data: &Vec<u8>) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if (&self._cursor.clone() == &SifrInt::from(self._buffer.len())) {
                self._buffer = {
                    let mut __v = (self._buffer.clone()).clone();
                    __v.extend((data).iter().cloned());
                    __v
                };
                self._cursor = &self._cursor.clone() + &SifrInt::from(data.len());
                return Ok(());
            }
            let left: Vec<u8> = {
                let _slice_src = &self._buffer.clone();
                let _slice_len = _slice_src.len();
                let _slice_start = 0;
                let _slice_stop = self._cursor.clone().clamp_slice_bound(_slice_len);
                Vec::from_iter(
                    _slice_src
                        .iter()
                        .skip(_slice_start)
                        .take(_slice_stop.saturating_sub(_slice_start))
                        .cloned(),
                )
            };
            let tail_start: SifrInt = &self._cursor.clone() + &SifrInt::from(data.len());
            let mut right: Vec<u8> = {
                let __sifr_empty_bytes_literal: Vec<u8> = vec![];
                __sifr_empty_bytes_literal
            };
            if (&tail_start < &SifrInt::from(self._buffer.len())) {
                right = {
                    let _slice_src = &self._buffer.clone();
                    let _slice_len = _slice_src.len();
                    let _slice_start = tail_start.clamp_slice_bound(_slice_len);
                    let _slice_stop = _slice_len;
                    Vec::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start)
                            .take(_slice_stop.saturating_sub(_slice_start))
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
            self._cursor = &self._cursor.clone() + &SifrInt::from(data.len());
            Ok(())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn getvalue(&self) -> Vec<u8> {
            self._buffer.clone()
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn seek(
            &mut self,
            offset: &SifrInt,
            whence: &SifrInt,
        ) -> Result<SifrInt, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            let mut origin: SifrInt = SifrInt::from_i64(0);
            if (&whence.clone() == &SifrInt::from_i64(0)) {
                origin = SifrInt::from_i64(0);
            } else {
                if (&whence.clone() == &SifrInt::from_i64(1)) {
                    origin = self._cursor.clone();
                } else {
                    if (&whence.clone() == &SifrInt::from_i64(2)) {
                        origin = SifrInt::from(self._buffer.len());
                    } else {
                        return Err(
                            IOError::new(_invalid_whence_error((whence.clone()).clone())),
                        );
                    }
                }
            }
            let mut next_pos: SifrInt = &origin + offset;
            if (&next_pos < &SifrInt::from_i64(0)) {
                return Err(IOError::new(_negative_seek_error((next_pos).clone())));
            }
            let end: SifrInt = SifrInt::from(self._buffer.len());
            if &next_pos > &end {
                next_pos = end.clone();
            }
            self._cursor = next_pos.clone();
            Ok(self._cursor.clone())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn tell(&self) -> Result<SifrInt, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            Ok(self._cursor.clone())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn readable(&self) -> bool {
            !(self._closed)
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn writable(&self) -> bool {
            !(self._closed)
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn seekable(&self) -> bool {
            !(self._closed)
        }
    }
    pub fn _closed_stream_error() -> String {
        "I/O operation on closed stream".to_string()
    }
    pub fn _invalid_whence_error(whence: SifrInt) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
            __sifr_concat.push_str("invalid whence: ");
            __sifr_concat.push_str((format!("{}", whence)).as_str());
            __sifr_concat
        }
    }
    pub fn _negative_seek_error(offset: SifrInt) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(24usize + 0usize);
            __sifr_concat.push_str("negative seek position: ");
            __sifr_concat.push_str((format!("{}", offset)).as_str());
            __sifr_concat
        }
    }
    pub fn _unsupported_seek_tell_error() -> String {
        "seek/tell is unsupported for this stream".to_string()
    }
    pub fn _mode_is_readable(mode: &String) -> bool {
        mode.contains(&"r".to_string()) || mode.contains(&"+".to_string())
    }
    pub fn _mode_is_writable(mode: &String) -> bool {
        (mode.contains(&"w".to_string()) || mode.contains(&"a".to_string()))
            || mode.contains(&"+".to_string())
    }
    pub fn _text_binary_mode(mode: &String) -> Result<String, IOError> {
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
    pub fn _text_encoding_or_default(
        enc: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncoding>,
    ) -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        if let Some(enc) = enc.as_ref() {
            return __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(
                format!("{}{}", enc.label.clone(), ""),
            );
        }
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new("utf-8".to_string())
    }
    pub fn _decode_errors_or_default(
        errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
    ) -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
        if let Some(errors) = errors.as_ref() {
            return __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
                format!("{}{}", errors.name.clone(), ""),
            );
        }
        strict_decode_handler()
    }
    pub fn _encode_errors_from_decode_errors(
        errors: &__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler,
    ) -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
            format!("{}{}", errors.name.clone(), ""),
        )
    }
    pub fn open(path: &String, mode: &String) -> Result<__SifrIoFileHandle, IOError> {
        let __sifr_try_res: Result<Result<__SifrIoFileHandle, IOError>, IOError> = (|| {
            let handle: __SifrIoNativeFileHandle = open_file(path, mode)?;
            Ok(Ok(__SifrIoFileHandle::new(handle, (mode.clone()).clone())))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(e);
            }
        }
    }
    pub fn open_binary(
        path: &String,
        mode: &String,
    ) -> Result<__SifrIoBinaryFileHandle, IOError> {
        if !mode.contains(&"b".to_string()) {
            return Err(IOError::new("open_binary requires binary mode".to_string()));
        }
        let __sifr_try_res: Result<Result<__SifrIoBinaryFileHandle, IOError>, IOError> = (|| {
            let handle: __SifrIoNativeFileHandle = open_file(path, mode)?;
            Ok(Ok(__SifrIoBinaryFileHandle::new(handle, (mode.clone()).clone())))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(e);
            }
        }
    }
    pub fn open_text(
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
            Ok(
                Ok(
                    __SifrIoTextFileHandle::new(
                        binary,
                        text_encoding,
                        decode_errors,
                        encode_errors,
                    ),
                ),
            )
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(e);
            }
        }
    }
    pub fn __const_QUOTE_ALL() -> SifrInt {
        SifrInt::from_i64(1)
    }
    pub fn __const_QUOTE_NONNUMERIC() -> SifrInt {
        SifrInt::from_i64(2)
    }
    pub fn __const_QUOTE_NONE() -> SifrInt {
        SifrInt::from_i64(3)
    }
    pub fn __const_QUOTE_STRINGS() -> SifrInt {
        SifrInt::from_i64(4)
    }
    pub fn __const_QUOTE_NOTNULL() -> SifrInt {
        SifrInt::from_i64(5)
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2ecsv_x2eDialect {
        pub delimiter: String,
        pub quotechar: String,
        pub escapechar: String,
        pub doublequote: bool,
        pub skipinitialspace: bool,
        pub lineterminator: String,
        pub quoting: SifrInt,
    }
    impl __SifrStdlib_sifr_x2ecsv_x2eDialect {
        pub fn new(
            delimiter: String,
            quotechar: String,
            escapechar: String,
            doublequote: bool,
            skipinitialspace: bool,
            lineterminator: String,
            quoting: SifrInt,
        ) -> Self {
            let mut resolved_quoting: SifrInt = quoting.clone();
            _validate_char(&"delimiter".to_string(), &delimiter);
            if (quotechar != "") {
                _validate_char(&"quotechar".to_string(), &quotechar);
            }
            if (escapechar != "") {
                _validate_char(&"escapechar".to_string(), &escapechar);
            }
            if (quotechar == "") && (&resolved_quoting != &__const_QUOTE_NONE()) {
                resolved_quoting = __const_QUOTE_NONE().clone();
            }
            let __sifr_field_init_0: String = delimiter;
            let __sifr_field_init_1: String = quotechar;
            let __sifr_field_init_2: String = escapechar;
            let __sifr_field_init_3: bool = doublequote;
            let __sifr_field_init_4: bool = skipinitialspace;
            let __sifr_field_init_5: String = lineterminator;
            let __sifr_field_init_6: SifrInt = resolved_quoting.clone();
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
    pub struct __SifrStdlib_sifr_x2ecsv_x2ereader {
        pub _rows: Vec<Vec<String>>,
        pub _pos: SifrInt,
        pub dialect: __SifrStdlib_sifr_x2ecsv_x2eDialect,
    }
    impl __SifrStdlib_sifr_x2ecsv_x2ereader {
        pub fn new(
            text: String,
            dialect: Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
            delimiter: String,
            quotechar: String,
            escapechar: String,
            doublequote: bool,
            skipinitialspace: bool,
            quoting: SifrInt,
        ) -> Self {
            let resolved_dialect: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
                &dialect,
                &delimiter,
                &quotechar,
                &escapechar,
                doublequote,
                skipinitialspace,
                &"\n".to_string(),
                (quoting).clone(),
            );
            let rows: Vec<Vec<String>> = parse_csv(
                &text,
                &None,
                &format!("{}{}", resolved_dialect.delimiter.clone(), ""),
                &format!("{}{}", resolved_dialect.quotechar.clone(), ""),
                &format!("{}{}", resolved_dialect.escapechar.clone(), ""),
                resolved_dialect.doublequote,
                resolved_dialect.skipinitialspace,
                resolved_dialect.quoting.clone(),
            );
            let __sifr_field_init_0: __SifrStdlib_sifr_x2ecsv_x2eDialect = resolved_dialect;
            let __sifr_field_init_1: Vec<Vec<String>> = rows;
            let __sifr_field_init_2: SifrInt = SifrInt::from_i64(0);
            Self {
                dialect: __sifr_field_init_0,
                _rows: __sifr_field_init_1,
                _pos: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2ecsv_x2ereader {
        pub fn __next__(&mut self) -> Option<Vec<String>> {
            if (&self._pos.clone() >= &SifrInt::from(self._rows.len())) {
                return None;
            }
            let row: Option<Vec<String>> = {
                let __sifr_checked_read_collection = &self._rows;
                let __sifr_checked_read_index = self._pos.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
            };
            self._pos = &self._pos.clone() + &SifrInt::from_i64(1);
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
        pub fn rows(&self) -> Vec<Vec<String>> {
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
        pub fn line_num(&self) -> SifrInt {
            self._pos.clone()
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2ecsv_x2ewriter {
        pub _rows: Vec<Vec<String>>,
        pub dialect: __SifrStdlib_sifr_x2ecsv_x2eDialect,
    }
    impl __SifrStdlib_sifr_x2ecsv_x2ewriter {
        pub fn new(
            dialect: Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
            delimiter: String,
            quotechar: String,
            escapechar: String,
            doublequote: bool,
            skipinitialspace: bool,
            lineterminator: String,
            quoting: SifrInt,
        ) -> Self {
            let resolved_dialect: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
                &dialect,
                &delimiter,
                &quotechar,
                &escapechar,
                doublequote,
                skipinitialspace,
                &lineterminator,
                (quoting).clone(),
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
        pub fn writerow(&mut self, row: &Vec<String>) {
            let mut copied: Vec<String> = vec![];
            for value in row.iter().cloned() {
                copied.push(value.clone());
            }
            self._rows.push(copied.clone());
        }
    }
    impl __SifrStdlib_sifr_x2ecsv_x2ewriter {
        pub fn writerows(&mut self, rows: &Vec<Vec<String>>) {
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
        pub fn getvalue(&self) -> String {
            format_csv(
                &self._rows,
                &Some((self.dialect.clone()).clone()),
                &",".to_string(),
                &"\"".to_string(),
                &"".to_string(),
                true,
                false,
                &"\n".to_string(),
                SifrInt::from_i64(0),
            )
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2ecsv_x2eDictReader {
        pub _fieldnames: Vec<String>,
        pub _rows: Vec<Vec<String>>,
        pub _pos: SifrInt,
        pub restkey: String,
        pub restval: String,
        pub dialect: __SifrStdlib_sifr_x2ecsv_x2eDialect,
    }
    impl __SifrStdlib_sifr_x2ecsv_x2eDictReader {
        pub fn new(
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
            quoting: SifrInt,
        ) -> Self {
            let resolved_dialect: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
                &dialect,
                &delimiter,
                &quotechar,
                &escapechar,
                doublequote,
                skipinitialspace,
                &"\n".to_string(),
                (quoting).clone(),
            );
            let all_rows: Vec<Vec<String>> = parse_csv(
                &text,
                &None,
                &format!("{}{}", resolved_dialect.delimiter.clone(), ""),
                &format!("{}{}", resolved_dialect.quotechar.clone(), ""),
                &format!("{}{}", resolved_dialect.escapechar.clone(), ""),
                resolved_dialect.doublequote,
                resolved_dialect.skipinitialspace,
                resolved_dialect.quoting.clone(),
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
                        .map(|__pair| (
                            SifrInt::from(__pair.0) + SifrInt::from_i64(0),
                            __pair.1,
                        )),
                ) {
                    if (&index == &SifrInt::from_i64(0)) {
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
            let __sifr_field_init_3: SifrInt = SifrInt::from_i64(0);
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
        pub fn fieldnames(&self) -> Vec<String> {
            let mut copied: Vec<String> = vec![];
            for field in self._fieldnames.clone().iter().cloned() {
                copied.push(format!("{}{}", field, ""));
            }
            copied
        }
    }
    impl __SifrStdlib_sifr_x2ecsv_x2eDictReader {
        pub fn __next__(&mut self) -> Option<HashMap<String, String>> {
            while (&self._pos.clone() < &SifrInt::from(self._rows.len())) {
                let row: Option<Vec<String>> = {
                    let __sifr_checked_read_collection = &self._rows;
                    let __sifr_checked_read_index = self._pos.clone();
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                };
                self._pos = &self._pos.clone() + &SifrInt::from_i64(1);
                let Some(row) = row else {
                    return None;
                };
                if (&SifrInt::from(row.len()) == &SifrInt::from_i64(0)) {
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
        pub fn rows(&self) -> Vec<HashMap<String, String>> {
            let mut result: Vec<HashMap<String, String>> = vec![];
            for row in self._rows.clone().iter().cloned() {
                if (&SifrInt::from(row.len()) == &SifrInt::from_i64(0)) {
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
    pub struct __SifrStdlib_sifr_x2ecsv_x2eDictWriter {
        pub fieldnames: Vec<String>,
        pub restval: String,
        pub extrasaction: String,
        pub _writer: __SifrStdlib_sifr_x2ecsv_x2ewriter,
    }
    impl __SifrStdlib_sifr_x2ecsv_x2eDictWriter {
        pub fn new(
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
            quoting: SifrInt,
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
                (quoting).clone(),
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
        pub fn writeheader(&mut self) {
            let mut current_writer: __SifrStdlib_sifr_x2ecsv_x2ewriter = self
                ._writer
                .clone();
            current_writer.writerow(&self.fieldnames.clone());
            self._writer = current_writer;
        }
    }
    impl __SifrStdlib_sifr_x2ecsv_x2eDictWriter {
        pub fn writerow(&mut self, row: &HashMap<String, String>) {
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
        pub fn writerows(&mut self, rows: &Vec<HashMap<String, String>>) {
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
        pub fn getvalue(&self) -> String {
            self._writer.getvalue()
        }
    }
    pub fn _copy_dialect(
        dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect,
    ) -> __SifrStdlib_sifr_x2ecsv_x2eDialect {
        __SifrStdlib_sifr_x2ecsv_x2eDialect::new(
            format!("{}{}", dialect.delimiter.clone(), ""),
            format!("{}{}", dialect.quotechar.clone(), ""),
            format!("{}{}", dialect.escapechar.clone(), ""),
            dialect.doublequote,
            dialect.skipinitialspace,
            format!("{}{}", dialect.lineterminator.clone(), ""),
            dialect.quoting.clone(),
        )
    }
    pub fn _validate_char(name: &String, value: &String) {
        let _ = (name).clone();
        let _ = (value).clone();
    }
    pub fn _resolve_dialect(
        dialect: &Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
        delimiter: &String,
        quotechar: &String,
        escapechar: &String,
        doublequote: bool,
        skipinitialspace: bool,
        lineterminator: &String,
        quoting: SifrInt,
    ) -> __SifrStdlib_sifr_x2ecsv_x2eDialect {
        if let Some(dialect) = dialect.as_ref() {
            return _copy_dialect(dialect);
        }
        __SifrStdlib_sifr_x2ecsv_x2eDialect::new(
            (delimiter.clone()).clone(),
            (quotechar.clone()).clone(),
            (escapechar.clone()).clone(),
            doublequote,
            skipinitialspace,
            (lineterminator.clone()).clone(),
            (quoting).clone(),
        )
    }
    pub fn _quotechar_value(dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect) -> String {
        let quotechar: String = {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((dialect.quotechar.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        if (quotechar).as_str() == ("".to_string()).as_str() {
            return "\"".to_string();
        }
        quotechar
    }
    pub fn _append_field(row: &mut Vec<String>, field: String) {
        row.push(format!("{}{}", field, ""));
    }
    pub fn _append_row(rows: &mut Vec<Vec<String>>, row: Vec<String>) {
        rows.push(row.clone());
    }
    pub fn _char_at(text: &String, index: SifrInt) -> String {
        let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        if (&index < &SifrInt::from_i64(0))
            || (&index >= &SifrInt::from(__sifr_chars_text.len()))
        {
            return "".to_string();
        }
        let ch: Option<String> = ({
            let __sifr_string_index = index.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_text.len());
            __sifr_chars_text.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        let Some(ch) = ch else {
            return "".to_string();
        };
        ch
    }
    pub fn _list_value_at(values: &Vec<String>, index: SifrInt) -> String {
        if (&index < &SifrInt::from_i64(0)) || (&index >= &SifrInt::from(values.len())) {
            return "".to_string();
        }
        for (current_index, value) in Box::new(
            (values)
                .iter()
                .cloned()
                .enumerate()
                .map(|__pair| (SifrInt::from(__pair.0) + SifrInt::from_i64(0), __pair.1)),
        ) {
            if (&current_index == &index) {
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
    pub fn _dict_value_at(values: &HashMap<String, String>, key: &String) -> String {
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
    pub fn _first_char(text: &String) -> String {
        _char_at(text, SifrInt::from_i64(0))
    }
    pub fn _last_char(text: &String) -> String {
        let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        _char_at(text, SifrInt::from(text.chars().count()) - SifrInt::from_i64(1))
    }
    pub fn parse_csv(
        text: &String,
        dialect: &Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
        delimiter: &String,
        quotechar: &String,
        escapechar: &String,
        doublequote: bool,
        skipinitialspace: bool,
        quoting: SifrInt,
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
            (quoting).clone(),
        );
        let mut rows: Vec<Vec<String>> = vec![];
        let mut row: Vec<String> = vec![];
        let mut field: String = "".to_string();
        let mut in_quotes: bool = false;
        let mut field_started: bool = false;
        let mut i: SifrInt = SifrInt::from_i64(0);
        while (&i < &SifrInt::from(__sifr_chars_text.len())) {
            let ch_value: String = _char_at(text, (i).clone());
            if in_quotes {
                if (resolved.escapechar.clone() != "")
                    && (ch_value == resolved.escapechar.clone())
                {
                    if (&(&i + &SifrInt::from_i64(1))
                        < &SifrInt::from(__sifr_chars_text.len()))
                    {
                        let escaped_value: String = _char_at(
                            text,
                            &i + &SifrInt::from_i64(1),
                        );
                        field.push_str((escaped_value).as_str());
                        i = &i + &SifrInt::from_i64(2);
                        continue;
                    }
                    field.push_str((ch_value).as_str());
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                }
                if (resolved.quotechar.clone() != "")
                    && (ch_value == resolved.quotechar.clone())
                {
                    let quotechar: String = _quotechar_value(&resolved);
                    if (resolved.doublequote
                        && (&(&i + &SifrInt::from_i64(1))
                            < &SifrInt::from(__sifr_chars_text.len())))
                        && (_char_at(text, &i + &SifrInt::from_i64(1)) == quotechar.clone())
                    {
                        field.push_str((quotechar).as_str());
                        i = &i + &SifrInt::from_i64(2);
                        continue;
                    }
                    in_quotes = false;
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                }
                field.push_str((ch_value).as_str());
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            if (!field_started && resolved.skipinitialspace) && (ch_value == " ") {
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            if (resolved.escapechar.clone() != "")
                && (ch_value == resolved.escapechar.clone())
            {
                if (&(&i + &SifrInt::from_i64(1)) < &SifrInt::from(__sifr_chars_text.len()))
                {
                    let escaped_plain_value: String = _char_at(
                        text,
                        &i + &SifrInt::from_i64(1),
                    );
                    field.push_str((escaped_plain_value).as_str());
                    field_started = true;
                    i = &i + &SifrInt::from_i64(2);
                    continue;
                }
                field.push_str((ch_value).as_str());
                field_started = true;
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            if (&resolved.quoting.clone() != &__const_QUOTE_NONE())
                && (resolved.quotechar.clone() != "")
            {
                let quotechar2: String = _quotechar_value(&resolved);
                if (ch_value == quotechar2) {
                    in_quotes = true;
                    field_started = true;
                    i = &i + &SifrInt::from_i64(1);
                    continue;
                }
            }
            if (ch_value == resolved.delimiter.clone()) {
                _append_field(&mut row, field);
                field = "".to_string();
                field_started = false;
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            if (ch_value == "\n") || (ch_value == "\r") {
                if ((ch_value == "\r")
                    && (&(&i + &SifrInt::from_i64(1))
                        < &SifrInt::from(__sifr_chars_text.len())))
                    && (_char_at(text, &i + &SifrInt::from_i64(1)) == "\n")
                {
                    i = &i + &SifrInt::from_i64(1);
                }
                if (&SifrInt::from(row.len()) == &SifrInt::from_i64(0)) && (field == "") {
                    _append_row(&mut rows, vec![]);
                } else {
                    _append_field(&mut row, field);
                    _append_row(&mut rows, row);
                }
                row = vec![];
                field = "".to_string();
                field_started = false;
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            field.push_str((ch_value).as_str());
            field_started = true;
            i = &i + &SifrInt::from_i64(1);
        }
        if in_quotes {
            in_quotes = false;
        }
        if (&SifrInt::from(row.len()) > &SifrInt::from_i64(0)) || (field != "") {
            _append_field(&mut row, field);
            _append_row(&mut rows, row);
        }
        rows
    }
    pub fn _needs_quote(
        field: &String,
        dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect,
    ) -> bool {
        let __sifr_chars_field: Vec<char> = field.chars().collect::<Vec<char>>();
        if (&dialect.quoting.clone() == &__const_QUOTE_ALL()) {
            return true;
        }
        if (&dialect.quoting.clone() == &__const_QUOTE_NONNUMERIC()) {
            return true;
        }
        if (&dialect.quoting.clone() == &__const_QUOTE_STRINGS()) {
            return true;
        }
        if (&dialect.quoting.clone() == &__const_QUOTE_NOTNULL()) {
            return true;
        }
        if (&dialect.quoting.clone() == &__const_QUOTE_NONE()) {
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
        if (&SifrInt::from(__sifr_chars_field.len()) > &SifrInt::from_i64(0)) {
            let first: String = _first_char(field);
            let last: String = _last_char(field);
            if (first == " ") {
                return true;
            }
            if (last == " ") {
                return true;
            }
        }
        false
    }
    pub fn _quote_field(
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
    pub fn _escape_unquoted_field(
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
    pub fn format_row(
        fields: &Vec<String>,
        dialect: &Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
        delimiter: &String,
        quotechar: &String,
        escapechar: &String,
        doublequote: bool,
        skipinitialspace: bool,
        quoting: SifrInt,
    ) -> String {
        let resolved: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
            dialect,
            delimiter,
            quotechar,
            escapechar,
            doublequote,
            skipinitialspace,
            &"\n".to_string(),
            (quoting).clone(),
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
    pub fn format_csv(
        rows: &Vec<Vec<String>>,
        dialect: &Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
        delimiter: &String,
        quotechar: &String,
        escapechar: &String,
        doublequote: bool,
        skipinitialspace: bool,
        lineterminator: &String,
        quoting: SifrInt,
    ) -> String {
        let resolved: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
            dialect,
            delimiter,
            quotechar,
            escapechar,
            doublequote,
            skipinitialspace,
            lineterminator,
            (quoting).clone(),
        );
        let mut rendered: Vec<String> = vec![];
        let resolved_delimiter: String = {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((resolved.delimiter.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        let resolved_quotechar: String = {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((resolved.quotechar.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        let resolved_escapechar: String = {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((resolved.escapechar.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        let resolved_lineterminator: String = {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((resolved.lineterminator.clone()).as_str());
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
                        resolved.quoting.clone(),
                    ),
                );
        }
        rendered.join(&resolved_lineterminator)
    }
    pub fn _dict_reader_row(
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
                .map(|__pair| (SifrInt::from(__pair.0) + SifrInt::from_i64(0), __pair.1)),
        ) {
            if (&i < &SifrInt::from(row.len())) {
                {
                    let __assign_value = _list_value_at(row, (i).clone());
                    {
                        let __assign_key = key.clone();
                        result.insert(__assign_key, __assign_value);
                    }
                }
            } else {
                {
                    let __assign_value = {
                        let mut __sifr_concat: String = String::with_capacity(
                            restval.len() + 0usize,
                        );
                        __sifr_concat.push_str((restval).as_str());
                        __sifr_concat.push_str("");
                        __sifr_concat
                    };
                    {
                        let __assign_key = key.clone();
                        result.insert(__assign_key, __assign_value);
                    }
                }
            }
        }
        if ((restkey).as_str() != "")
            && (&SifrInt::from(row.len()) > &SifrInt::from(fieldnames.len()))
        {
            let mut extras: Vec<String> = vec![];
            let mut j: SifrInt = SifrInt::from(fieldnames.len());
            while (&j < &SifrInt::from(row.len())) {
                extras.push(_list_value_at(row, (j).clone()));
                j = &j + &SifrInt::from_i64(1);
            }
            {
                let __assign_value = format!("{:?}", extras);
                {
                    let __assign_key = restkey.clone();
                    result.insert(__assign_key, __assign_value);
                }
            }
        }
        result
    }
    pub fn datetime_now() -> String {
        ::sifr_stdlib::time::datetime_now()
    }
    pub fn datetime_now_struct() -> Vec<SifrInt> {
        ::sifr_stdlib::time::datetime_now_struct()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
            .collect()
    }
    pub fn datetime_format(dt: &String, fmt: &String) -> String {
        ::sifr_stdlib::time::datetime_format(dt, fmt)
    }
    pub fn datetime_from_timestamp(ts: f64) -> Result<String, ValueError> {
        ::sifr_stdlib::time::datetime_from_timestamp(ts)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn time_now() -> f64 {
        ::sifr_stdlib::time::time_now()
    }
    pub fn time_format(epoch: f64, fmt: &String) -> String {
        ::sifr_stdlib::time::time_format(epoch, fmt)
    }
    pub fn perf_counter() -> f64 {
        ::sifr_stdlib::time::perf_counter()
    }
    pub fn sleep(seconds: f64) {
        ::sifr_stdlib::time::sleep(seconds);
    }
    pub fn monotonic() -> f64 {
        ::sifr_stdlib::time::monotonic()
    }
    pub fn strptime(s: &String, fmt: &String) -> Result<String, ValueError> {
        ::sifr_stdlib::time::strptime(s, fmt)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn _strptime_intrinsic(s: &String, fmt: &String) -> Result<String, ValueError> {
        ::sifr_stdlib::time::strptime(s, fmt)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn gmtime(epoch: f64) -> String {
        ::sifr_stdlib::time::gmtime(epoch)
    }
    pub fn _gmtime_intrinsic(epoch: f64) -> String {
        ::sifr_stdlib::time::gmtime(epoch)
    }
    pub fn localtime(epoch: f64) -> String {
        ::sifr_stdlib::time::localtime(epoch)
    }
    pub fn _localtime_intrinsic(epoch: f64) -> String {
        ::sifr_stdlib::time::localtime(epoch)
    }
    pub fn time_strptime(s: &String, fmt: &String) -> Result<Vec<SifrInt>, ValueError> {
        ::sifr_stdlib::time::time_strptime(s, fmt)
            .map(|__sifr_bridge_ok| {
                __sifr_bridge_ok
                    .into_iter()
                    .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
                    .collect()
            })
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn time_gmtime() -> Vec<SifrInt> {
        ::sifr_stdlib::time::time_gmtime()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
            .collect()
    }
    pub fn time_localtime() -> Vec<SifrInt> {
        ::sifr_stdlib::time::time_localtime()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
            .collect()
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
        __SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(FloatOverflowError),
        __SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
            FloatPrecisionLossError,
        ),
    }
    impl From<FloatOverflowError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
        fn from(value: FloatOverflowError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
            }
        }
    }
    #[derive(Debug, Clone)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
        __SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(FloatOverflowError),
        __SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
            FloatPrecisionLossError,
        ),
        __SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(ValueError),
    }
    impl From<FloatOverflowError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
        fn from(value: FloatOverflowError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
            }
        }
    }
    #[derive(Debug, Clone)]
    pub struct __SifrStdlib_sifr_x2edatetime_x2etimezone {
        pub _offset: SifrInt,
    }
    impl __SifrStdlib_sifr_x2edatetime_x2etimezone {
        pub fn new(offset: SifrInt) -> Self {
            let __sifr_field_init_0: SifrInt = offset.clone();
            Self {
                _offset: __sifr_field_init_0,
            }
        }
    }
    impl __SifrStdlib_sifr_x2edatetime_x2etimezone {
        pub fn offset(&self) -> SifrInt {
            self._offset.clone()
        }
    }
    impl __SifrStdlib_sifr_x2edatetime_x2etimezone {
        pub fn iso_suffix(&self) -> String {
            let mut sign: String = "+".to_string();
            if (&self._offset.clone() < &SifrInt::from_i64(0)) {
                sign = "-".to_string();
            }
            let mut abs_offset: SifrInt = self._offset.clone();
            if &abs_offset < &SifrInt::from_i64(0) {
                abs_offset = -&abs_offset;
            }
            let h: SifrInt = abs_offset.floor_div_known_nonzero(&SifrInt::from_i64(3600));
            let m: SifrInt = abs_offset
                .floor_mod_known_nonzero(&SifrInt::from_i64(3600))
                .floor_div_known_nonzero(&SifrInt::from_i64(60));
            let mut hs: String = format!("{}", h);
            if (&SifrInt::from(hs.chars().count()) < &SifrInt::from_i64(2)) {
                hs = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + hs.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((hs).as_str());
                    __sifr_concat
                };
            }
            let mut ms: String = format!("{}", m);
            if (&SifrInt::from(ms.chars().count()) < &SifrInt::from_i64(2)) {
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
            if (&self._offset.clone() == &SifrInt::from_i64(0)) {
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
    pub struct __SifrStdlib_sifr_x2edatetime_x2edatetime {
        pub year: SifrInt,
        pub month: SifrInt,
        pub day: SifrInt,
        pub hour: SifrInt,
        pub minute: SifrInt,
        pub second: SifrInt,
        pub microsecond: SifrInt,
        pub _tz_offset: Option<SifrInt>,
    }
    impl __SifrStdlib_sifr_x2edatetime_x2edatetime {
        pub fn new(
            year: SifrInt,
            month: SifrInt,
            day: SifrInt,
            hour: SifrInt,
            minute: SifrInt,
            second: SifrInt,
            microsecond: SifrInt,
            tz_offset: Option<SifrInt>,
        ) -> Self {
            let __sifr_field_init_0: SifrInt = year.clone();
            let __sifr_field_init_1: SifrInt = month.clone();
            let __sifr_field_init_2: SifrInt = day.clone();
            let __sifr_field_init_3: SifrInt = hour.clone();
            let __sifr_field_init_4: SifrInt = minute.clone();
            let __sifr_field_init_5: SifrInt = second.clone();
            let __sifr_field_init_6: SifrInt = microsecond.clone();
            let __sifr_field_init_7: Option<SifrInt> = tz_offset.clone();
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
        pub fn isoformat(&self) -> String {
            let y: String = format!("{}", self.year.clone());
            let mut mo: String = format!("{}", self.month.clone());
            if (&SifrInt::from(mo.chars().count()) < &SifrInt::from_i64(2)) {
                mo = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + mo.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((mo).as_str());
                    __sifr_concat
                };
            }
            let mut d: String = format!("{}", self.day.clone());
            if (&SifrInt::from(d.chars().count()) < &SifrInt::from_i64(2)) {
                d = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + d.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((d).as_str());
                    __sifr_concat
                };
            }
            let mut h: String = format!("{}", self.hour.clone());
            if (&SifrInt::from(h.chars().count()) < &SifrInt::from_i64(2)) {
                h = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + h.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((h).as_str());
                    __sifr_concat
                };
            }
            let mut mi: String = format!("{}", self.minute.clone());
            if (&SifrInt::from(mi.chars().count()) < &SifrInt::from_i64(2)) {
                mi = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + mi.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((mi).as_str());
                    __sifr_concat
                };
            }
            let mut s: String = format!("{}", self.second.clone());
            if (&SifrInt::from(s.chars().count()) < &SifrInt::from_i64(2)) {
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
            if (&self.microsecond.clone() != &SifrInt::from_i64(0)) {
                base.push('.');
                base.push_str((_six_digits(self.microsecond.clone())).as_str());
            }
            let tz_offset_opt: Option<SifrInt> = self._tz_offset.clone();
            if let Some(tz_offset_opt) = tz_offset_opt.clone() {
                let offset: SifrInt = tz_offset_opt.clone();
                let mut sign: String = "+".to_string();
                let mut abs_offset: SifrInt = offset.clone();
                if (&abs_offset < &SifrInt::from_i64(0)) {
                    sign = "-".to_string();
                    abs_offset = -&abs_offset;
                }
                let h_off: SifrInt = abs_offset
                    .floor_div_known_nonzero(&SifrInt::from_i64(3600));
                let m_off: SifrInt = abs_offset
                    .floor_mod_known_nonzero(&SifrInt::from_i64(3600))
                    .floor_div_known_nonzero(&SifrInt::from_i64(60));
                let mut hs_off: String = format!("{}", h_off);
                if (&SifrInt::from(hs_off.chars().count()) < &SifrInt::from_i64(2)) {
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
                if (&SifrInt::from(ms_off.chars().count()) < &SifrInt::from_i64(2)) {
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
        pub fn timestamp(&self) -> SifrInt {
            let mut days: SifrInt = SifrInt::from_i64(0);
            if (&self.year.clone() >= &SifrInt::from_i64(1970)) {
                let mut y: SifrInt = SifrInt::from_i64(1970);
                while (&y < &self.year.clone()) {
                    days = &days + &_days_in_year((y).clone());
                    y = &y + &SifrInt::from_i64(1);
                }
            } else {
                let mut y: SifrInt = SifrInt::from_i64(1969);
                while (&y >= &self.year.clone()) {
                    days = &days - &_days_in_year((y).clone());
                    y = &y - &SifrInt::from_i64(1);
                }
            }
            let mut m: SifrInt = SifrInt::from_i64(1);
            while (&m < &self.month.clone()) {
                days = &days + &_days_in_month(self.year.clone(), (m).clone());
                m = &m + &SifrInt::from_i64(1);
            }
            days = &(&days + &self.day.clone()) - &SifrInt::from_i64(1);
            let naive_timestamp: SifrInt = &(&(&(&days * &SifrInt::from_i64(86400))
                + &(&self.hour.clone() * &SifrInt::from_i64(3600)))
                + &(&self.minute.clone() * &SifrInt::from_i64(60))) + &self.second.clone();
            let tz_offset_opt: Option<SifrInt> = self._tz_offset.clone();
            if let Some(tz_offset_opt) = tz_offset_opt.clone() {
                let offset: SifrInt = tz_offset_opt.clone();
                return &naive_timestamp - &offset;
            }
            naive_timestamp
        }
    }
    impl __SifrStdlib_sifr_x2edatetime_x2edatetime {
        pub fn timestamp_microseconds(&self) -> SifrInt {
            &(&self.timestamp() * &SifrInt::from_i64(1000000)) + &self.microsecond.clone()
        }
    }
    impl __SifrStdlib_sifr_x2edatetime_x2edatetime {
        pub fn astimezone(
            &self,
            tz: &Option<__SifrStdlib_sifr_x2edatetime_x2etimezone>,
        ) -> Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError> {
            let mut target: __SifrStdlib_sifr_x2edatetime_x2etimezone = __SifrStdlib_sifr_x2edatetime_x2etimezone::new(
                SifrInt::from_i64(0),
            );
            if let Some(tz) = tz.as_ref() {
                let __sifr_try_res: Result<(), ValueError> = (|| {
                    let tz_text: String = format!("{}", tz);
                    let target_offset: SifrInt = _timezone_offset_from_text(&tz_text)?;
                    target = __SifrStdlib_sifr_x2edatetime_x2etimezone::new(
                        (target_offset).clone(),
                    );
                    Ok(())
                })();
                if let Err(__sifr_try_err) = __sifr_try_res {
                    let e = __sifr_try_err.clone();
                    return Err(ValueError::new(e.message.clone()));
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
            (((((((((self.year.clone() == other.year.clone()))
                && ((self.month.clone() == other.month.clone())))
                && ((self.day.clone() == other.day.clone())))
                && ((self.hour.clone() == other.hour.clone())))
                && ((self.minute.clone() == other.minute.clone())))
                && ((self.second.clone() == other.second.clone())))
                && ((self.microsecond.clone() == other.microsecond.clone()))) && (same_tz))
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2edatetime_x2edatetime {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.isoformat())
        }
    }
    #[derive(Debug, Clone)]
    pub struct __SifrStdlib_sifr_x2edatetime_x2edate {
        pub year: SifrInt,
        pub month: SifrInt,
        pub day: SifrInt,
    }
    impl __SifrStdlib_sifr_x2edatetime_x2edate {
        pub fn new(year: SifrInt, month: SifrInt, day: SifrInt) -> Self {
            Self { year, month, day }
        }
    }
    impl __SifrStdlib_sifr_x2edatetime_x2edate {
        pub fn isoformat(&self) -> String {
            let y: String = format!("{}", self.year.clone());
            let mut mo: String = format!("{}", self.month.clone());
            if (&SifrInt::from(mo.chars().count()) < &SifrInt::from_i64(2)) {
                mo = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + mo.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((mo).as_str());
                    __sifr_concat
                };
            }
            let mut d: String = format!("{}", self.day.clone());
            if (&SifrInt::from(d.chars().count()) < &SifrInt::from_i64(2)) {
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
            ((((self.year.clone() == other.year.clone()))
                && ((self.month.clone() == other.month.clone())))
                && ((self.day.clone() == other.day.clone())))
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2edatetime_x2edate {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.isoformat())
        }
    }
    pub fn _is_leap_year(year: SifrInt) -> bool {
        (((&year.floor_mod_known_nonzero(&SifrInt::from_i64(4)) == &SifrInt::from_i64(0))
            && (&year.floor_mod_known_nonzero(&SifrInt::from_i64(100))
                != &SifrInt::from_i64(0)))
            || ((&year.floor_mod_known_nonzero(&SifrInt::from_i64(400))
                == &SifrInt::from_i64(0))))
    }
    pub fn _days_in_year(year: SifrInt) -> SifrInt {
        if _is_leap_year((year).clone()) {
            return SifrInt::from_i64(366);
        }
        SifrInt::from_i64(365)
    }
    pub fn _days_in_month(year: SifrInt, month: SifrInt) -> SifrInt {
        let month_days: Vec<SifrInt> = vec![
            SifrInt::from_i64(31), SifrInt::from_i64(28), SifrInt::from_i64(31),
            SifrInt::from_i64(30), SifrInt::from_i64(31), SifrInt::from_i64(30),
            SifrInt::from_i64(31), SifrInt::from_i64(31), SifrInt::from_i64(30),
            SifrInt::from_i64(31), SifrInt::from_i64(30), SifrInt::from_i64(31)
        ];
        let idx: SifrInt = &month - &SifrInt::from_i64(1);
        let d: Option<SifrInt> = {
            let __sifr_checked_read_collection = &month_days;
            let __sifr_checked_read_index = idx.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if (&month == &SifrInt::from_i64(2)) && _is_leap_year((year).clone()) {
            return SifrInt::from_i64(29);
        }
        if let Some(d) = d.clone() {
            return d;
        }
        SifrInt::from_i64(0)
    }
    pub fn _substring(value: &String, start: SifrInt, end: SifrInt) -> String {
        let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
        let mut result: String = "".to_string();
        let mut i: SifrInt = start.clone();
        while (&i < &end) {
            let ch: Option<String> = ({
                let __sifr_string_index = i.clone();
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_value.len());
                __sifr_chars_value.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            if let Some(ch) = ch {
                result.push_str((ch).as_str());
            }
            i = &i + &SifrInt::from_i64(1);
        }
        result
    }
    pub fn _six_digits(value: SifrInt) -> String {
        let mut rendered: String = format!("{}", value);
        let mut __sifr_chars_rendered: Vec<char> = rendered.chars().collect::<Vec<char>>();
        while (&SifrInt::from(__sifr_chars_rendered.len()) < &SifrInt::from_i64(6)) {
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
    pub fn _parse_datetime_iso(
        value: &String,
    ) -> Result<(SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt), ValueError> {
        let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
        let Some(__sifr_checked_value_2) = ({
            let __sifr_string_index = SifrInt::from_i64(4);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_value.len());
            __sifr_chars_value.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string()) else {
            return Err(ValueError::new("invalid datetime string".to_string()));
        };
        let Some(__sifr_checked_value_3) = ({
            let __sifr_string_index = SifrInt::from_i64(7);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_value.len());
            __sifr_chars_value.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string()) else {
            return Err(ValueError::new("invalid datetime string".to_string()));
        };
        let Some(__sifr_checked_value_4) = ({
            let __sifr_string_index = SifrInt::from_i64(10);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_value.len());
            __sifr_chars_value.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string()) else {
            return Err(ValueError::new("invalid datetime string".to_string()));
        };
        let Some(__sifr_checked_value_5) = ({
            let __sifr_string_index = SifrInt::from_i64(13);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_value.len());
            __sifr_chars_value.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string()) else {
            return Err(ValueError::new("invalid datetime string".to_string()));
        };
        let Some(__sifr_checked_value_6) = ({
            let __sifr_string_index = SifrInt::from_i64(16);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_value.len());
            __sifr_chars_value.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string()) else {
            return Err(ValueError::new("invalid datetime string".to_string()));
        };
        if ((((__sifr_checked_value_2.clone() != "-")
            || (__sifr_checked_value_3.clone() != "-"))
            || (__sifr_checked_value_4.clone() != "T"))
            || (__sifr_checked_value_5.clone() != ":"))
            || (__sifr_checked_value_6.clone() != ":")
        {
            return Err(ValueError::new("invalid datetime string".to_string()));
        }
        let __sifr_try_res: Result<
            Result<(SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt), ValueError>,
            ParseError,
        > = (|| {
            let year: SifrInt = SifrInt::parse_decimal(
                    &(_substring(value, SifrInt::from_i64(0), SifrInt::from_i64(4))),
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let month: SifrInt = SifrInt::parse_decimal(
                    &(_substring(value, SifrInt::from_i64(5), SifrInt::from_i64(7))),
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let day: SifrInt = SifrInt::parse_decimal(
                    &(_substring(value, SifrInt::from_i64(8), SifrInt::from_i64(10))),
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let hour: SifrInt = SifrInt::parse_decimal(
                    &(_substring(value, SifrInt::from_i64(11), SifrInt::from_i64(13))),
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let minute: SifrInt = SifrInt::parse_decimal(
                    &(_substring(value, SifrInt::from_i64(14), SifrInt::from_i64(16))),
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let second: SifrInt = SifrInt::parse_decimal(
                    &(_substring(value, SifrInt::from_i64(17), SifrInt::from_i64(19))),
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            Ok(
                Ok((
                    year.clone(),
                    month.clone(),
                    day.clone(),
                    hour.clone(),
                    minute.clone(),
                    second.clone(),
                )),
            )
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
    pub fn _timezone_offset_from_text(text: &String) -> Result<SifrInt, ValueError> {
        let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        if (text).as_str() == "UTC" {
            return Ok(SifrInt::from_i64(0));
        }
        if (&SifrInt::from(__sifr_chars_text.len()) != &SifrInt::from_i64(9)) {
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
        if (_substring(text, SifrInt::from_i64(0), SifrInt::from_i64(3)) != "UTC") {
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
        let sign_value: String = _substring(
            text,
            SifrInt::from_i64(3),
            SifrInt::from_i64(4),
        );
        if (sign_value != "+") && (sign_value != "-") {
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
        if (({
            let __sifr_string_index = SifrInt::from_i64(6);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_text.len());
            __sifr_chars_text.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string()) != Some(":".to_string()))
        {
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
        let __sifr_try_res: Result<Result<SifrInt, ValueError>, ParseError> = (|| {
            let hours: SifrInt = SifrInt::parse_decimal(
                    &(_substring(text, SifrInt::from_i64(4), SifrInt::from_i64(6))),
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let minutes: SifrInt = SifrInt::parse_decimal(
                    &(_substring(text, SifrInt::from_i64(7), SifrInt::from_i64(9))),
                    ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                )
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            let mut offset: SifrInt = &(&hours * &SifrInt::from_i64(3600))
                + &(&minutes * &SifrInt::from_i64(60));
            if (sign_value == "-") {
                offset = -&offset;
            }
            Ok(Ok(offset))
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
    pub fn _from_timestamp_with_tz(
        ts: f64,
        tz: &Option<__SifrStdlib_sifr_x2edatetime_x2etimezone>,
    ) -> Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError> {
        let __sifr_try_res: Result<
            Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError>,
            __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
        > = (|| {
            let whole_seconds: SifrInt = (SifrInt::from_f64_trunc(ts)
                .ok_or_else(|| ValueError {
                    message: "cannot convert non-finite float to int".to_string(),
                }))
                .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    __e,
                ))?;
            let whole_seconds_float: f64 = (whole_seconds
                .clone()
                .checked_to_f64()
                .map_err(|__sifr_float_error| match __sifr_float_error {
                    ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                            FloatOverflowError::new(
                                "exact integer is outside the finite float range".to_string(),
                            ),
                        )
                    }
                    ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                            FloatPrecisionLossError::new(
                                "exact integer cannot be represented without float precision loss"
                                    .to_string(),
                            ),
                        )
                    }
                }))
                .map_err(|__e| match __e {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        __sifr_union_value,
                    ) => {
                        __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                            __sifr_union_value,
                        )
                    }
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        __sifr_union_value,
                    ) => {
                        __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                            __sifr_union_value,
                        )
                    }
                })?;
            let fractional: f64 = ts - whole_seconds_float;
            let mut microsecond: SifrInt = (SifrInt::from_f64_trunc(
                    fractional * (1000000.0_f64),
                )
                .ok_or_else(|| ValueError {
                    message: "cannot convert non-finite float to int".to_string(),
                }))
                .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    __e,
                ))?;
            if (&microsecond < &SifrInt::from_i64(0)) {
                microsecond = -&microsecond;
            }
            let mut adjusted_seconds: SifrInt = whole_seconds.clone();
            let mut tz_offset_value: SifrInt = SifrInt::from_i64(0);
            let mut tz_has_offset: bool = false;
            if let Some(tz) = tz.as_ref() {
                let tz_text: String = format!("{}", tz);
                let tz_offset: SifrInt = (_timezone_offset_from_text(&tz_text))
                    .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                        __e,
                    ))?;
                adjusted_seconds = &whole_seconds + &tz_offset;
                tz_offset_value = tz_offset;
                tz_has_offset = true;
            }
            let adjusted_seconds_float: f64 = (adjusted_seconds
                .clone()
                .checked_to_f64()
                .map_err(|__sifr_float_error| match __sifr_float_error {
                    ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                            FloatOverflowError::new(
                                "exact integer is outside the finite float range".to_string(),
                            ),
                        )
                    }
                    ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                            FloatPrecisionLossError::new(
                                "exact integer cannot be represented without float precision loss"
                                    .to_string(),
                            ),
                        )
                    }
                }))
                .map_err(|__e| match __e {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        __sifr_union_value,
                    ) => {
                        __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                            __sifr_union_value,
                        )
                    }
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        __sifr_union_value,
                    ) => {
                        __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                            __sifr_union_value,
                        )
                    }
                })?;
            let rendered: String = (datetime_from_timestamp(adjusted_seconds_float))
                .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    __e,
                ))?;
            let parts: (SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt) = (_parse_datetime_iso(
                &rendered,
            ))
                .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    __e,
                ))?;
            let year_part: Option<SifrInt> = Some((parts).0.clone());
            let month_part: Option<SifrInt> = Some((parts).1.clone());
            let day_part: Option<SifrInt> = Some((parts).2.clone());
            let hour_part: Option<SifrInt> = Some((parts).3.clone());
            let minute_part: Option<SifrInt> = Some((parts).4.clone());
            let second_part: Option<SifrInt> = Some((parts).5.clone());
            let mut year: SifrInt = SifrInt::from_i64(0);
            let mut month: SifrInt = SifrInt::from_i64(1);
            let mut day: SifrInt = SifrInt::from_i64(1);
            let mut hour: SifrInt = SifrInt::from_i64(0);
            let mut minute: SifrInt = SifrInt::from_i64(0);
            let mut second: SifrInt = SifrInt::from_i64(0);
            if let Some(year_part) = year_part.clone() {
                year = year_part;
            }
            if let Some(month_part) = month_part.clone() {
                month = month_part;
            }
            if let Some(day_part) = day_part.clone() {
                day = day_part;
            }
            if let Some(hour_part) = hour_part.clone() {
                hour = hour_part;
            }
            if let Some(minute_part) = minute_part.clone() {
                minute = minute_part;
            }
            if let Some(second_part) = second_part.clone() {
                second = second_part;
            }
            if tz_has_offset {
                return Ok(
                    Ok(
                        __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                            (year).clone(),
                            (month).clone(),
                            (day).clone(),
                            (hour).clone(),
                            (minute).clone(),
                            (second).clone(),
                            (microsecond).clone(),
                            Some(tz_offset_value),
                        ),
                    ),
                );
            }
            Ok(
                Ok(
                    __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                        (year).clone(),
                        (month).clone(),
                        (day).clone(),
                        (hour).clone(),
                        (minute).clone(),
                        (second).clone(),
                        (microsecond).clone(),
                        None,
                    ),
                ),
            )
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                match __sifr_try_err {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        __sifr_try_variant_error,
                    ) => {
                        let e = __sifr_try_variant_error.clone();
                        return Err(ValueError::new(e.message.clone()));
                    }
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        __sifr_try_variant_error,
                    ) => {
                        let e = __sifr_try_variant_error.clone();
                        return Err(ValueError::new(e.message.clone()));
                    }
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                        __sifr_try_variant_error,
                    ) => {
                        let e = __sifr_try_variant_error.clone();
                        return Err(ValueError::new(e.message.clone()));
                    }
                }
            }
        }
    }
    pub fn _from_timestamp_microseconds_with_tz(
        value: SifrInt,
        tz: &Option<__SifrStdlib_sifr_x2edatetime_x2etimezone>,
    ) -> Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError> {
        let whole_seconds: SifrInt = value
            .floor_div_known_nonzero(&SifrInt::from_i64(1000000));
        let microsecond: SifrInt = value
            .floor_mod_known_nonzero(&SifrInt::from_i64(1000000));
        let __sifr_try_res: Result<
            Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError>,
            __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
        > = (|| {
            let whole_seconds_float: f64 = (whole_seconds
                .clone()
                .checked_to_f64()
                .map_err(|__sifr_float_error| match __sifr_float_error {
                    ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                            FloatOverflowError::new(
                                "exact integer is outside the finite float range".to_string(),
                            ),
                        )
                    }
                    ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                            FloatPrecisionLossError::new(
                                "exact integer cannot be represented without float precision loss"
                                    .to_string(),
                            ),
                        )
                    }
                }))
                .map_err(|__e| match __e {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        __sifr_union_value,
                    ) => {
                        __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                            __sifr_union_value,
                        )
                    }
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        __sifr_union_value,
                    ) => {
                        __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                            __sifr_union_value,
                        )
                    }
                })?;
            let result: __SifrStdlib_sifr_x2edatetime_x2edatetime = (_from_timestamp_with_tz(
                whole_seconds_float,
                tz,
            ))
                .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    __e,
                ))?;
            Ok(
                Ok(
                    __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                        result.year.clone(),
                        result.month.clone(),
                        result.day.clone(),
                        result.hour.clone(),
                        result.minute.clone(),
                        result.second.clone(),
                        (microsecond).clone(),
                        result._tz_offset.clone(),
                    ),
                ),
            )
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                match __sifr_try_err {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        __sifr_try_variant_error,
                    ) => {
                        let e = __sifr_try_variant_error.clone();
                        return Err(ValueError::new(e.message.clone()));
                    }
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        __sifr_try_variant_error,
                    ) => {
                        let e = __sifr_try_variant_error.clone();
                        return Err(ValueError::new(e.message.clone()));
                    }
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                        __sifr_try_variant_error,
                    ) => {
                        let e = __sifr_try_variant_error.clone();
                        return Err(ValueError::new(e.message.clone()));
                    }
                }
            }
        }
    }
    pub fn set_global_level(level: SifrInt) {
        ::sifr_stdlib::logging::set_global_level(
            ::sifr_runtime::interop::SifrIntBridge::from(level),
        );
    }
    pub fn get_global_level() -> SifrInt {
        ::sifr_stdlib::logging::get_global_level().into_sifr_int()
    }
    pub fn __const_DEBUG() -> SifrInt {
        SifrInt::from_i64(10)
    }
    pub fn __const_INFO() -> SifrInt {
        SifrInt::from_i64(20)
    }
    pub fn __const_WARNING() -> SifrInt {
        SifrInt::from_i64(30)
    }
    pub fn __const_ERROR() -> SifrInt {
        SifrInt::from_i64(40)
    }
    pub fn __const_CRITICAL() -> SifrInt {
        SifrInt::from_i64(50)
    }
    pub fn __const_NOTSET() -> SifrInt {
        SifrInt::from_i64(0)
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2elogging_x2eFormatter {
        pub _fmt: String,
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFormatter {
        pub fn new(fmt: String) -> Self {
            let __sifr_field_init_0: String = fmt;
            Self { _fmt: __sifr_field_init_0 }
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFormatter {
        pub fn template(&self) -> String {
            self._fmt.clone()
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFormatter {
        pub fn format(&self, level: &String, name: &String, msg: &String) -> String {
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
    pub struct __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        pub _level: SifrInt,
        pub _formatter: __SifrStdlib_sifr_x2elogging_x2eFormatter,
    }
    impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        pub fn new(level: SifrInt) -> Self {
            let __sifr_field_init_0: SifrInt = level.clone();
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
        pub fn set_level(&mut self, level: &SifrInt) {
            self._level = level.clone();
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        pub fn level(&self) -> SifrInt {
            self._level.clone()
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        pub fn set_formatter(&mut self, fmt: &__SifrStdlib_sifr_x2elogging_x2eFormatter) {
            self._formatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
                format!("{}{}", fmt._fmt.clone(), ""),
            );
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        pub fn format_template(&self) -> String {
            self._formatter.template()
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        pub fn _allows(&self, level_num: &SifrInt) -> bool {
            if (&self._level.clone() == &__const_NOTSET()) {
                return true;
            }
            (&level_num.clone() >= &self._level.clone())
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        pub fn emit(&self, level: &String, name: &String, msg: &String) {
            let level_num: SifrInt = _level_name_to_num(level);
            if !self._allows(&level_num) {
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
    pub struct __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub _path: String,
        pub _level: SifrInt,
        pub _formatter: __SifrStdlib_sifr_x2elogging_x2eFormatter,
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub fn new(path: String, level: SifrInt) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
                __sifr_concat.push_str((path).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let __sifr_field_init_1: SifrInt = level.clone();
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
        pub fn path(&self) -> String {
            self._path.clone()
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub fn set_level(&mut self, level: &SifrInt) {
            self._level = level.clone();
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub fn level(&self) -> SifrInt {
            self._level.clone()
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub fn set_formatter(&mut self, fmt: &__SifrStdlib_sifr_x2elogging_x2eFormatter) {
            self._formatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
                format!("{}{}", fmt._fmt.clone(), ""),
            );
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub fn format_template(&self) -> String {
            self._formatter.template()
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub fn _allows(&self, level_num: &SifrInt) -> bool {
            if (&self._level.clone() == &__const_NOTSET()) {
                return true;
            }
            (&level_num.clone() >= &self._level.clone())
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub fn emit(&self, level: &String, name: &String, msg: &String) {
            let level_num: SifrInt = _level_name_to_num(level);
            if !self._allows(&level_num) {
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
                    let _ = e2.message.clone();
                }
                fh.close();
                Ok(())
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                let _ = e.message.clone();
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
    pub struct __SifrStdlib_sifr_x2elogging_x2eNullHandler {
        pub _level: SifrInt,
        pub _formatter: __SifrStdlib_sifr_x2elogging_x2eFormatter,
    }
    impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
        pub fn new(level: SifrInt) -> Self {
            let __sifr_field_init_0: SifrInt = level.clone();
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
        pub fn set_level(&mut self, level: &SifrInt) {
            self._level = level.clone();
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
        pub fn level(&self) -> SifrInt {
            self._level.clone()
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
        pub fn set_formatter(&mut self, fmt: &__SifrStdlib_sifr_x2elogging_x2eFormatter) {
            self._formatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
                format!("{}{}", fmt._fmt.clone(), ""),
            );
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
        pub fn format_template(&self) -> String {
            self._formatter.template()
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
        pub fn emit(&self, level: &String, name: &String, msg: &String) {
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
    pub struct __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub _name: String,
        pub _level: SifrInt,
        pub _log_path: String,
        pub _handler_kind: String,
        pub _handler_path: String,
        pub _handler_level: SifrInt,
        pub _handler_fmt: String,
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn new(name: String, level: SifrInt) -> Self {
            let __sifr_field_init_0: String = name;
            let __sifr_field_init_1: SifrInt = level.clone();
            let __sifr_field_init_2: String = "".to_string();
            let __sifr_field_init_3: String = "".to_string();
            let __sifr_field_init_4: String = "".to_string();
            let __sifr_field_init_5: SifrInt = __const_NOTSET().clone();
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
        pub fn set_level(&mut self, level: &SifrInt) {
            self._level = level.clone();
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn set_file(&mut self, path: &String) {
            self._log_path = {
                let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
                __sifr_concat.push_str((path).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn add_handler(
            &mut self,
            handler: &__SifrStdlib_sifr_x2elogging_x2eFileHandler,
        ) {
            self._handler_kind = "file".to_string();
            self._handler_path = handler.path();
            self._handler_level = handler.level();
            self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn set_stream_handler(
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
        pub fn set_null_handler(
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
        pub fn clear_handler(&mut self) {
            self._handler_kind = "".to_string();
            self._handler_path = "".to_string();
            self._handler_level = __const_NOTSET().clone();
            self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn _handler_allows(&self, level_num: &SifrInt) -> bool {
            if (&self._handler_level.clone() == &__const_NOTSET()) {
                return true;
            }
            (&level_num.clone() >= &self._handler_level.clone())
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn _handler_line(&self, level: &String, msg: &String) -> String {
            let formatter: __SifrStdlib_sifr_x2elogging_x2eFormatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
                self._handler_fmt.clone(),
            );
            formatter.format(level, &self._name.clone(), msg)
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn _emit(&self, level: &String, level_num: &SifrInt, msg: &String) {
            if (&self._level.clone() > &level_num.clone()) {
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
                            let _ = e2.message.clone();
                        }
                        fh.close();
                        Ok(())
                    })();
                    if let Err(__sifr_try_err) = __sifr_try_res {
                        let e = __sifr_try_err.clone();
                        let _ = e.message.clone();
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
                        let _ = e2.message.clone();
                    }
                    fh.close();
                    Ok(())
                })();
                if let Err(__sifr_try_err) = __sifr_try_res {
                    let e = __sifr_try_err.clone();
                    let _ = e.message.clone();
                }
            }
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn debug(&self, msg: &String) {
            self._emit(&"DEBUG".to_string(), &__const_DEBUG(), msg);
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn info(&self, msg: &String) {
            self._emit(&"INFO".to_string(), &__const_INFO(), msg);
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn warning(&self, msg: &String) {
            self._emit(&"WARNING".to_string(), &__const_WARNING(), msg);
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn error(&self, msg: &String) {
            self._emit(&"ERROR".to_string(), &__const_ERROR(), msg);
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn critical(&self, msg: &String) {
            self._emit(&"CRITICAL".to_string(), &__const_CRITICAL(), msg);
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
    pub fn _level_name_to_num(level: &String) -> SifrInt {
        if (level).as_str() == "DEBUG" {
            return __const_DEBUG();
        }
        if (level).as_str() == "INFO" {
            return __const_INFO();
        }
        if (level).as_str() == "WARNING" {
            return __const_WARNING();
        }
        if (level).as_str() == "ERROR" {
            return __const_ERROR();
        }
        if (level).as_str() == "CRITICAL" {
            return __const_CRITICAL();
        }
        __const_NOTSET()
    }
    pub struct __SifrYielder<T> {
        pub slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    }
    pub struct __SifrYieldFuture<T> {
        pub slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        pub value: Option<T>,
    }
    impl<T> Unpin for __SifrYieldFuture<T> {}
    impl<T> ::std::future::Future for __SifrYieldFuture<T> {
        type Output = ();
        fn poll(
            self: ::std::pin::Pin<&mut Self>,
            _cx: &mut ::std::task::Context<'_>,
        ) -> ::std::task::Poll<()> {
            let state = self.get_mut();
            let Some(value) = state.value.take() else {
                return ::std::task::Poll::Ready(());
            };
            __sifr_store_suspended(&state.slot, value);
            ::std::task::Poll::Pending
        }
    }
    impl<T> __SifrYielder<T> {
        pub fn suspend(&self, value: T) -> __SifrYieldFuture<T> {
            __SifrYieldFuture {
                slot: ::std::sync::Arc::clone(&self.slot),
                value: Some(value),
            }
        }
    }
    pub fn __sifr_store_suspended<T>(
        slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        value: T,
    ) {
        match slot.lock() {
            Ok(mut state) => *state = Some(value),
            Err(poisoned) => *poisoned.into_inner() = Some(value),
        }
    }
    pub fn __sifr_take_suspended<T>(
        slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    ) -> Option<T> {
        match slot.lock() {
            Ok(mut state) => state.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        }
    }
    pub struct __SifrGenerator<T> {
        pub producer: Option<
            ::std::pin::Pin<Box<dyn ::std::future::Future<Output = ()> + 'static>>,
        >,
        pub yielded: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        pub complete: bool,
    }
    impl<T> __SifrGenerator<T> {
        pub fn new<
            F: FnOnce(__SifrYielder<T>) -> Fut + 'static,
            Fut: ::std::future::Future<Output = ()> + 'static,
        >(factory: F) -> Self {
            let yielded = ::std::sync::Arc::new(::std::sync::Mutex::new(None));
            let producer = factory(__SifrYielder {
                slot: ::std::sync::Arc::clone(&yielded),
            });
            Self {
                producer: Some(Box::pin(producer)),
                yielded,
                complete: false,
            }
        }
    }
    impl<T> Iterator for __SifrGenerator<T> {
        type Item = T;
        fn next(&mut self) -> Option<T> {
            if self.complete {
                return None;
            }
            let completed = {
                let Some(producer) = self.producer.as_mut() else {
                    self.complete = true;
                    return None;
                };
                let mut context = ::std::task::Context::from_waker(
                    ::std::task::Waker::noop(),
                );
                ::std::future::Future::poll(producer.as_mut(), &mut context).is_ready()
            };
            let yielded = __sifr_take_suspended(&self.yielded);
            if completed {
                self.complete = true;
                self.producer = None;
            }
            yielded
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub _path: String,
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn new(path: String) -> Self {
            let __sifr_field_init_0: String = path;
            Self { _path: __sifr_field_init_0 }
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn name(&self) -> String {
            basename(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn parent(&self) -> __SifrStdlib_sifr_x2epathlib_x2ePath {
            __SifrStdlib_sifr_x2epathlib_x2ePath::new(dirname(&self._path))
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn suffix(&self) -> String {
            extension(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn stem(&self) -> String {
            stem(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn exists(&self) -> bool {
            exists(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn is_file(&self) -> bool {
            is_file(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn is_dir(&self) -> bool {
            is_dir(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn is_absolute(&self) -> bool {
            is_absolute(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn read_text(&self) -> Result<String, IOError> {
            read_text(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn write_text(&self, content: &String) -> Result<(), IOError> {
            write_text(&self._path, content)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn mkdir(&self) -> Result<(), IOError> {
            mkdir(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn joinpath(&self, child: &String) -> __SifrStdlib_sifr_x2epathlib_x2ePath {
            __SifrStdlib_sifr_x2epathlib_x2ePath::new(join_path(&self._path, child))
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn to_str(&self) -> String {
            {
                let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
                __sifr_concat.push_str((self._path.clone()).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            }
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn touch(&self) -> Result<(), IOError> {
            touch(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn unlink(&self) -> Result<(), IOError> {
            remove_file(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn rmdir(&self) -> Result<(), IOError> {
            rmdir(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn resolve(&self) -> Result<String, IOError> {
            resolve_path(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn iterdir(&self) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
            _iterdir_to_iter(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn with_name(&self, name: &String) -> __SifrStdlib_sifr_x2epathlib_x2ePath {
            let parent: String = dirname(&self._path);
            if (parent == "") {
                return __SifrStdlib_sifr_x2epathlib_x2ePath::new(format!("{}{}", name, ""));
            }
            __SifrStdlib_sifr_x2epathlib_x2ePath::new(
                format!("{}{}", format!("{}{}", parent, "/"), name),
            )
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn with_suffix(&self, suffix: &String) -> __SifrStdlib_sifr_x2epathlib_x2ePath {
            let s: String = stem(&self._path);
            let parent: String = dirname(&self._path);
            if (parent == "") {
                return __SifrStdlib_sifr_x2epathlib_x2ePath::new(format!("{}{}", s, suffix));
            }
            __SifrStdlib_sifr_x2epathlib_x2ePath::new(
                format!("{}{}", format!("{}{}", format!("{}{}", parent, "/"), s), suffix),
            )
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn glob(
            &self,
            pattern: &String,
        ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
            _glob_to_iter(&self._path, pattern)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn rglob(
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
    pub fn join_path(base: &String, child: &String) -> String {
        let __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
        if (&SifrInt::from(__sifr_chars_base.len()) == &SifrInt::from_i64(0)) {
            return {
                let mut __sifr_concat: String = String::with_capacity(child.len() + 0usize);
                __sifr_concat.push_str((child).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
        }
        let last: Option<String> = ({
            let __sifr_string_index = SifrInt::from(base.chars().count())
                - SifrInt::from_i64(1);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_base.len());
            __sifr_chars_base.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(last) = last {
            if (last).as_str() == ("/".to_string()).as_str() {
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
    pub fn basename(path: &String) -> String {
        let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        let mut i: SifrInt = &SifrInt::from(__sifr_chars_path.len()) - &SifrInt::from_i64(1);
        while (&i >= &SifrInt::from_i64(0)) {
            let ch: Option<String> = ({
                let __sifr_string_index = i.clone();
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_path.len());
                __sifr_chars_path.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            if let Some(ch) = ch {
                if (ch == "/") {
                    return {
                        let _slice_src = &__sifr_chars_path;
                        let _slice_len = _slice_src.len();
                        let _slice_start = (&i + &SifrInt::from_i64(1))
                            .clamp_slice_bound(_slice_len);
                        let _slice_stop = _slice_len;
                        String::from_iter(
                            _slice_src
                                .iter()
                                .skip(_slice_start)
                                .take(_slice_stop.saturating_sub(_slice_start))
                                .copied(),
                        )
                    };
                }
            }
            i = &i - &SifrInt::from_i64(1);
        }
        {
            let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
            __sifr_concat.push_str((path).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
    pub fn dirname(path: &String) -> String {
        let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        let mut i: SifrInt = &SifrInt::from(__sifr_chars_path.len()) - &SifrInt::from_i64(1);
        while (&i >= &SifrInt::from_i64(0)) {
            let ch: Option<String> = ({
                let __sifr_string_index = i.clone();
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_path.len());
                __sifr_chars_path.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            if let Some(ch) = ch {
                if (ch == "/") {
                    return {
                        let _slice_src = &__sifr_chars_path;
                        let _slice_len = _slice_src.len();
                        let _slice_start = 0;
                        let _slice_stop = i.clamp_slice_bound(_slice_len);
                        String::from_iter(
                            _slice_src
                                .iter()
                                .skip(_slice_start)
                                .take(_slice_stop.saturating_sub(_slice_start))
                                .copied(),
                        )
                    };
                }
            }
            i = &i - &SifrInt::from_i64(1);
        }
        "".to_string()
    }
    pub fn extension(path: &String) -> String {
        let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        let mut i: SifrInt = &SifrInt::from(__sifr_chars_path.len()) - &SifrInt::from_i64(1);
        while (&i >= &SifrInt::from_i64(0)) {
            let ch: Option<String> = ({
                let __sifr_string_index = i.clone();
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_path.len());
                __sifr_chars_path.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            if let Some(ch) = ch {
                if (ch == ".") {
                    return {
                        let _slice_src = &__sifr_chars_path;
                        let _slice_len = _slice_src.len();
                        let _slice_start = i.clamp_slice_bound(_slice_len);
                        let _slice_stop = _slice_len;
                        String::from_iter(
                            _slice_src
                                .iter()
                                .skip(_slice_start)
                                .take(_slice_stop.saturating_sub(_slice_start))
                                .copied(),
                        )
                    };
                }
                if (ch == "/") {
                    return "".to_string();
                }
            }
            i = &i - &SifrInt::from_i64(1);
        }
        "".to_string()
    }
    pub fn stem(path: &String) -> String {
        let base: String = basename(path);
        let __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
        let mut i: SifrInt = &SifrInt::from(__sifr_chars_base.len()) - &SifrInt::from_i64(1);
        while (&i > &SifrInt::from_i64(0)) {
            let ch: Option<String> = ({
                let __sifr_string_index = i.clone();
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_base.len());
                __sifr_chars_base.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            if let Some(ch) = ch {
                if (ch == ".") {
                    return {
                        let _slice_src = &__sifr_chars_base;
                        let _slice_len = _slice_src.len();
                        let _slice_start = 0;
                        let _slice_stop = i.clamp_slice_bound(_slice_len);
                        String::from_iter(
                            _slice_src
                                .iter()
                                .skip(_slice_start)
                                .take(_slice_stop.saturating_sub(_slice_start))
                                .copied(),
                        )
                    };
                }
            }
            i = &i - &SifrInt::from_i64(1);
        }
        {
            let mut __sifr_concat: String = String::with_capacity(base.len() + 0usize);
            __sifr_concat.push_str((base).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
    pub fn is_absolute(path: &String) -> bool {
        let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        if (&SifrInt::from(__sifr_chars_path.len()) == &SifrInt::from_i64(0)) {
            return false;
        }
        if (&SifrInt::from(__sifr_chars_path.len()) >= &SifrInt::from_i64(3)) {
            let colon: Option<String> = ({
                let __sifr_string_index = SifrInt::from_i64(1);
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_path.len());
                __sifr_chars_path.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            let sep: Option<String> = ({
                let __sifr_string_index = SifrInt::from_i64(2);
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_path.len());
                __sifr_chars_path.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            if let Some(colon) = colon {
                if let Some(sep) = sep {
                    if (colon == ":") && ((sep == "/") || (sep == "\\")) {
                        return true;
                    }
                }
            }
        }
        let first: Option<String> = ({
            let __sifr_string_index = SifrInt::from_i64(0);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_path.len());
            __sifr_chars_path.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(first) = first {
            if (first == "/") || (first == "\\") {
                return true;
            }
        }
        false
    }
    pub fn _iter_list_str(entries: Vec<String>) -> Box<dyn Iterator<Item = String>> {
        Box::new(
            __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<String>| {
                let mut i: SifrInt = SifrInt::from_i64(0);
                while (&i < &SifrInt::from(entries.len())) {
                    let Some(__sifr_checked_value_7) = ({
                        let __sifr_checked_read_collection = &entries;
                        let __sifr_checked_read_index = i.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(__sifr_checked_read_collection.len());
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    }) else {
                        break;
                    };
                    __sifr_yielder.suspend(__sifr_checked_value_7.clone()).await;
                    i = &i + &SifrInt::from_i64(1);
                }
            }),
        )
    }
    pub fn _iterdir_list(path: &String) -> Result<Vec<String>, IOError> {
        iterdir(path)
    }
    pub fn _glob_list(path: &String, pattern: &String) -> Result<Vec<String>, IOError> {
        glob_pattern(path, pattern)
    }
    pub fn _rglob_list(path: &String, pattern: &String) -> Result<Vec<String>, IOError> {
        rglob_pattern(path, pattern)
    }
    pub fn _iterdir_to_iter(
        path: &String,
    ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        let __sifr_try_res: Result<
            Result<Box<dyn Iterator<Item = String>>, IOError>,
            IOError,
        > = (|| {
            let entries: Vec<String> = _iterdir_list(path)?;
            Ok(Ok(_iter_list_str(entries)))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(e);
            }
        }
    }
    pub fn _glob_to_iter(
        path: &String,
        pattern: &String,
    ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        let __sifr_try_res: Result<
            Result<Box<dyn Iterator<Item = String>>, IOError>,
            IOError,
        > = (|| {
            let entries: Vec<String> = _glob_list(path, pattern)?;
            Ok(Ok(_iter_list_str(entries)))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(e);
            }
        }
    }
    pub fn _rglob_to_iter(
        path: &String,
        pattern: &String,
    ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        let __sifr_try_res: Result<
            Result<Box<dyn Iterator<Item = String>>, IOError>,
            IOError,
        > = (|| {
            let entries: Vec<String> = _rglob_list(path, pattern)?;
            Ok(Ok(_iter_list_str(entries)))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(e);
            }
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2ere_x2eMatch {
        pub _matched: String,
        pub _start: SifrInt,
        pub _end: SifrInt,
    }
    impl __SifrStdlib_sifr_x2ere_x2eMatch {
        pub fn new(matched: String, start: SifrInt, end: SifrInt) -> Self {
            let __sifr_field_init_0: String = matched;
            let __sifr_field_init_1: SifrInt = start.clone();
            let __sifr_field_init_2: SifrInt = end.clone();
            Self {
                _matched: __sifr_field_init_0,
                _start: __sifr_field_init_1,
                _end: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2eMatch {
        pub fn group(&self) -> String {
            {
                let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
                __sifr_concat.push_str((self._matched.clone()).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            }
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2eMatch {
        pub fn start(&self) -> SifrInt {
            self._start.clone()
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2eMatch {
        pub fn end(&self) -> SifrInt {
            self._end.clone()
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2eMatch {
        pub fn span(&self) -> Vec<SifrInt> {
            let result: Vec<SifrInt> = vec![self._start.clone(), self._end.clone()];
            result
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2eMatch {
        pub fn to_str(&self) -> String {
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
    pub struct __SifrStdlib_sifr_x2ere_x2ePattern {
        pub _compiled: __SifrStdlib___sifr_x2eregex_x2eCompiledPattern,
        pub _pattern: String,
        pub _flags: SifrInt,
    }
    impl __SifrStdlib_sifr_x2ere_x2ePattern {
        pub fn new(
            compiled: __SifrStdlib___sifr_x2eregex_x2eCompiledPattern,
            pattern: String,
            flags: SifrInt,
        ) -> Self {
            let __sifr_field_init_0: __SifrStdlib___sifr_x2eregex_x2eCompiledPattern = compiled;
            let __sifr_field_init_1: String = pattern;
            let __sifr_field_init_2: SifrInt = flags.clone();
            Self {
                _compiled: __sifr_field_init_0,
                _pattern: __sifr_field_init_1,
                _flags: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2ePattern {
        pub fn search(&self, text: &String) -> Result<Option<String>, RegexError> {
            self._compiled.search(text)
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2ePattern {
        pub fn is_match(&self, text: &String) -> Result<bool, RegexError> {
            self._compiled.is_match(text)
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2ePattern {
        pub fn sub(
            &self,
            replacement: &String,
            text: &String,
        ) -> Result<String, RegexError> {
            self._compiled.sub(replacement, text)
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2ePattern {
        pub fn findall(&self, text: &String) -> Result<Vec<String>, RegexError> {
            self._compiled.findall(text)
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2ePattern {
        pub fn finditer(
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
                Ok(Ok(_iter_matches(matches)))
            })();
            match __sifr_try_res {
                Ok(__sifr_ret_val) => {
                    return __sifr_ret_val;
                }
                Err(__sifr_try_err) => {
                    let e = __sifr_try_err.clone();
                    return Err(RegexError::new(e.message.clone()));
                }
            }
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2ePattern {
        pub fn split(&self, text: &String) -> Result<Vec<String>, RegexError> {
            self._compiled.split(text)
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2ePattern {
        pub fn pattern(&self) -> String {
            {
                let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
                __sifr_concat.push_str((self._pattern.clone()).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            }
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2ePattern {
        pub fn flags(&self) -> SifrInt {
            self._flags.clone()
        }
    }
    pub fn _iter_matches(
        matches: Vec<__SifrStdlib_sifr_x2ere_x2eMatch>,
    ) -> Box<dyn Iterator<Item = __SifrStdlib_sifr_x2ere_x2eMatch>> {
        Box::new(
            __SifrGenerator::new(async move |
                __sifr_yielder: __SifrYielder<__SifrStdlib_sifr_x2ere_x2eMatch>|
            {
                let mut i: SifrInt = SifrInt::from_i64(0);
                while (&i < &SifrInt::from(matches.len())) {
                    let Some(__sifr_checked_value_0) = ({
                        let __sifr_checked_read_collection = &matches;
                        let __sifr_checked_read_index = i.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(__sifr_checked_read_collection.len());
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    }) else {
                        break;
                    };
                    __sifr_yielder.suspend(__sifr_checked_value_0.clone()).await;
                    i = &i + &SifrInt::from_i64(1);
                }
            }),
        )
    }
    pub fn _find_index_from(text: &String, needle: &String, start: SifrInt) -> SifrInt {
        let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        let __sifr_chars_needle: Vec<char> = needle.chars().collect::<Vec<char>>();
        if &start < &SifrInt::from_i64(0) {
            return -&SifrInt::from_i64(1);
        }
        if (&SifrInt::from(__sifr_chars_needle.len()) == &SifrInt::from_i64(0)) {
            if (&start <= &SifrInt::from(__sifr_chars_text.len())) {
                return start.clone();
            }
            return -&SifrInt::from_i64(1);
        }
        let max_start: SifrInt = &SifrInt::from(__sifr_chars_text.len())
            - &SifrInt::from(__sifr_chars_needle.len());
        let mut i: SifrInt = start.clone();
        while (&i <= &max_start) {
            if (({
                let _slice_src = &__sifr_chars_text;
                let _slice_len = _slice_src.len();
                let _slice_start = i.clamp_slice_bound(_slice_len);
                let _slice_stop = (&i + &SifrInt::from(__sifr_chars_needle.len()))
                    .clamp_slice_bound(_slice_len);
                String::from_iter(
                    _slice_src
                        .iter()
                        .skip(_slice_start)
                        .take(_slice_stop.saturating_sub(_slice_start))
                        .copied(),
                )
            }) == needle.clone())
            {
                return i.clone();
            }
            i = &i + &SifrInt::from_i64(1);
        }
        -&SifrInt::from_i64(1)
    }
    pub fn _finditer_from_items(
        found_items: &Vec<String>,
        text: &String,
    ) -> Vec<__SifrStdlib_sifr_x2ere_x2eMatch> {
        let mut matches: Vec<__SifrStdlib_sifr_x2ere_x2eMatch> = vec![];
        let mut cursor: SifrInt = SifrInt::from_i64(0);
        for found in found_items.iter().cloned() {
            let __sifr_chars_found: Vec<char> = found.chars().collect::<Vec<char>>();
            let mut start: SifrInt = _find_index_from(text, &found, (cursor).clone());
            if (&start < &SifrInt::from_i64(0)) {
                start = cursor.clone();
            }
            let found_len: SifrInt = SifrInt::from(__sifr_chars_found.len());
            let end: SifrInt = &start + &found_len;
            matches
                .push(
                    __SifrStdlib_sifr_x2ere_x2eMatch::new(found, start.clone(), end.clone()),
                );
            if (&found_len == &SifrInt::from_i64(0)) {
                cursor = &end + &SifrInt::from_i64(1);
            } else {
                cursor = end;
            }
        }
        matches
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IOError {
        pub message: String,
        pub kind: String,
    }
    impl IOError {
        pub fn new(message: String) -> Self {
            Self {
                message,
                kind: "Other".to_string(),
            }
        }
    }
    impl ::std::fmt::Display for IOError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for IOError {}
    pub fn __io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
        let msg = e.to_string();
        let kind = {
            let __sifr_io_kind = (&e as &dyn ::std::any::Any)
                .downcast_ref::<std::io::Error>()
                .map(::std::io::Error::kind);
            match __sifr_io_kind {
                Some(::std::io::ErrorKind::NotFound) => "FileNotFound".to_string(),
                Some(::std::io::ErrorKind::PermissionDenied) => {
                    "PermissionDenied".to_string()
                }
                Some(::std::io::ErrorKind::AlreadyExists) => "FileExists".to_string(),
                Some(::std::io::ErrorKind::IsADirectory) => "IsADirectory".to_string(),
                Some(::std::io::ErrorKind::NotADirectory) => "NotADirectory".to_string(),
                Some(::std::io::ErrorKind::DirectoryNotEmpty) => {
                    "DirectoryNotEmpty".to_string()
                }
                _ => "Other".to_string(),
            }
        };
        IOError { message: msg, kind }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Error {
        pub message: String,
    }
    impl Error {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for Error {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for Error {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ParseError {
        pub message: String,
    }
    impl ParseError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ParseError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ParseError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct RegexError {
        pub message: String,
        pub detail: String,
    }
    impl RegexError {
        pub fn new(message: String) -> Self {
            Self {
                message,
                detail: String::new(),
            }
        }
    }
    impl ::std::fmt::Display for RegexError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for RegexError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct FloatOverflowError {
        pub message: String,
    }
    impl FloatOverflowError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for FloatOverflowError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for FloatOverflowError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct FloatPrecisionLossError {
        pub message: String,
    }
    impl FloatPrecisionLossError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for FloatPrecisionLossError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for FloatPrecisionLossError {}
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
    impl From<FloatOverflowError> for Error {
        fn from(err: FloatOverflowError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<FloatPrecisionLossError> for Error {
        fn from(err: FloatPrecisionLossError) -> Self {
            Self::new(err.message)
        }
    }
}
pub use __sifr_project_nominals::Error;
pub use __sifr_project_nominals::FloatOverflowError;
pub use __sifr_project_nominals::FloatPrecisionLossError;
pub use __sifr_project_nominals::IOError;
pub use __sifr_project_nominals::ParseError;
pub use __sifr_project_nominals::RegexError;
pub use __sifr_project_nominals::ValueError;
pub use __sifr_project_nominals::__SifrIoBinaryFileHandle;
pub use __sifr_project_nominals::__SifrIoFileHandle;
pub use __sifr_project_nominals::__SifrIoTextFileHandle;
pub use __sifr_project_nominals::__SifrStdlib___sifr_x2eregex_x2eCompiledPattern;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ecollections_x2edeque;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ecsv_x2eDialect;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ecsv_x2eDictReader;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ecsv_x2eDictWriter;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ecsv_x2ereader;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ecsv_x2ewriter;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2edatetime_x2edate;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2edatetime_x2edatetime;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2edatetime_x2etimezone;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eencoding_x2eDecodeError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eencoding_x2eDecoder;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eencoding_x2eEncodeError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eencoding_x2eEncoder;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eencoding_x2eEncoding;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eio_x2eBinaryIOBase;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eio_x2eBytesIO;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eio_x2eIOBase;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eio_x2eStringIO;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eio_x2eTextIOBase;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eio_x2eTextReader;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eio_x2eTextWriter;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2elogging_x2eFileHandler;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2elogging_x2eFormatter;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2elogging_x2eLogger;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2elogging_x2eNullHandler;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2elogging_x2eStreamHandler;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2epathlib_x2ePath;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ere_x2eMatch;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ere_x2ePattern;
use ::std::collections::HashMap;
use ::std::collections::VecDeque;
use ::sifr_runtime::SifrInt;
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
fn _file_read_bytes(handle: &String, size: Option<SifrInt>) -> Result<Vec<u8>, IOError> {
    ::sifr_stdlib::fs::file_read_bytes(
            handle,
            size.map(::sifr_runtime::interop::SifrIntBridge::from),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_write_bytes(handle: &String, data: &Vec<u8>) -> Result<(), IOError> {
    ::sifr_stdlib::fs::file_write_bytes(handle, data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_flush(handle: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::file_flush(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_seek(
    handle: &String,
    offset: SifrInt,
    whence: SifrInt,
) -> Result<SifrInt, IOError> {
    ::sifr_stdlib::fs::file_seek(
            handle,
            ::sifr_runtime::interop::SifrIntBridge::from(offset),
            ::sifr_runtime::interop::SifrIntBridge::from(whence),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_tell(handle: &String) -> Result<SifrInt, IOError> {
    ::sifr_stdlib::fs::file_tell(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn open_file(path: &String, mode: &String) -> Result<__SifrIoNativeFileHandle, IOError> {
    let __sifr_try_res: Result<Result<__SifrIoNativeFileHandle, IOError>, IOError> = (|| {
        let handle_id: String = _open_file(path, mode)?;
        Ok(Ok(__SifrIoNativeFileHandle::new(handle_id)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(e);
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
fn file_read_bytes(
    handle: &__SifrIoNativeFileHandle,
    size: Option<SifrInt>,
) -> Result<Vec<u8>, IOError> {
    _file_read_bytes(&handle._id.clone(), (size).clone())
}
fn file_write_bytes(
    handle: &__SifrIoNativeFileHandle,
    data: &Vec<u8>,
) -> Result<(), IOError> {
    _file_write_bytes(&handle._id.clone(), data)
}
fn file_flush(handle: &__SifrIoNativeFileHandle) -> Result<(), IOError> {
    _file_flush(&handle._id.clone())
}
fn file_seek(
    handle: &__SifrIoNativeFileHandle,
    offset: SifrInt,
    whence: SifrInt,
) -> Result<SifrInt, IOError> {
    _file_seek(&handle._id.clone(), (offset).clone(), (whence).clone())
}
fn file_tell(handle: &__SifrIoNativeFileHandle) -> Result<SifrInt, IOError> {
    _file_tell(&handle._id.clone())
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
fn stat_size(path: &String) -> Result<SifrInt, IOError> {
    ::sifr_stdlib::fs::stat_size(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn disk_usage(path: &String) -> Vec<SifrInt> {
    ::sifr_stdlib::fs::disk_usage(path)
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
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
fn _encoding_is_supported(label: &String) -> bool {
    _encoding_is_supported_impl(label)
}
fn _encoding_canonical_label(
    label: &String,
) -> Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
    let __sifr_try_res: Result<
        Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
        ParseError,
    > = (|| {
        let value: String = _encoding_canonical_label_impl(label)?;
        Ok(Ok(value))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_decode_text(
    data: &Vec<u8>,
    encoding: &String,
    errors: &String,
) -> Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
    let __sifr_try_res: Result<
        Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
        ParseError,
    > = (|| {
        let text: String = _encoding_decode_text_impl(data, encoding, errors)?;
        Ok(Ok(text))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_decode_recoveries(
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
        Ok(Ok(recoveries))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_decode_outcome(
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
        Ok(Ok(__SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome::new(text, recoveries)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_decode_incremental_outcome(
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
        Ok(Ok(__SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome::new(text, recoveries)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_decode_incremental_pending(
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
        Ok(Ok(next_pending))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_encode_bytes(
    text: &String,
    encoding: &String,
    errors: &String,
) -> Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError> {
    let __sifr_try_res: Result<
        Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError>,
        ParseError,
    > = (|| {
        let data: Vec<u8> = _encoding_encode_bytes_impl(text, encoding, errors)?;
        Ok(Ok(data))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_encode_recoveries(
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
        Ok(Ok(recoveries))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_encode_outcome(
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
        Ok(Ok(__SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome::new(data, recoveries)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
            );
        }
    }
}
fn encoding(label: &String) -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new((label.clone()).clone())
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
    > = (|| { Ok(_encoding_decode_outcome(data, &enc.label.clone(), &handler_name)) })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
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
        Ok(Ok(outcome.get_text()))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
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
    > = (|| { Ok(_encoding_encode_outcome(text, &enc.label.clone(), &handler_name)) })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
            );
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
        Ok(Ok(outcome.get_data()))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
            );
        }
    }
}
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
fn _closed_stream_error() -> String {
    "I/O operation on closed stream".to_string()
}
fn _invalid_whence_error(whence: SifrInt) -> String {
    {
        let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("invalid whence: ");
        __sifr_concat.push_str((format!("{}", whence)).as_str());
        __sifr_concat
    }
}
fn _negative_seek_error(offset: SifrInt) -> String {
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
        Ok(Ok(__SifrIoFileHandle::new(handle, (mode.clone()).clone())))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(e);
        }
    }
}
fn open_binary(
    path: &String,
    mode: &String,
) -> Result<__SifrIoBinaryFileHandle, IOError> {
    if !mode.contains(&"b".to_string()) {
        return Err(IOError::new("open_binary requires binary mode".to_string()));
    }
    let __sifr_try_res: Result<Result<__SifrIoBinaryFileHandle, IOError>, IOError> = (|| {
        let handle: __SifrIoNativeFileHandle = open_file(path, mode)?;
        Ok(Ok(__SifrIoBinaryFileHandle::new(handle, (mode.clone()).clone())))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(e);
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
        Ok(
            Ok(
                __SifrIoTextFileHandle::new(
                    binary,
                    text_encoding,
                    decode_errors,
                    encode_errors,
                ),
            ),
        )
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(e);
        }
    }
}
fn __const_QUOTE_ALL() -> SifrInt {
    SifrInt::from_i64(1)
}
fn __const_QUOTE_NONNUMERIC() -> SifrInt {
    SifrInt::from_i64(2)
}
fn __const_QUOTE_NONE() -> SifrInt {
    SifrInt::from_i64(3)
}
fn __const_QUOTE_STRINGS() -> SifrInt {
    SifrInt::from_i64(4)
}
fn __const_QUOTE_NOTNULL() -> SifrInt {
    SifrInt::from_i64(5)
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
        dialect.quoting.clone(),
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
    quoting: SifrInt,
) -> __SifrStdlib_sifr_x2ecsv_x2eDialect {
    if let Some(dialect) = dialect.as_ref() {
        return _copy_dialect(dialect);
    }
    __SifrStdlib_sifr_x2ecsv_x2eDialect::new(
        (delimiter.clone()).clone(),
        (quotechar.clone()).clone(),
        (escapechar.clone()).clone(),
        doublequote,
        skipinitialspace,
        (lineterminator.clone()).clone(),
        (quoting).clone(),
    )
}
fn _quotechar_value(dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect) -> String {
    let quotechar: String = {
        let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
        __sifr_concat.push_str((dialect.quotechar.clone()).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if (quotechar).as_str() == ("".to_string()).as_str() {
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
fn _char_at(text: &String, index: SifrInt) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    if (&index < &SifrInt::from_i64(0))
        || (&index >= &SifrInt::from(__sifr_chars_text.len()))
    {
        return "".to_string();
    }
    let ch: Option<String> = ({
        let __sifr_string_index = index.clone();
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_text.len());
        __sifr_chars_text.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string());
    let Some(ch) = ch else {
        return "".to_string();
    };
    ch
}
fn _list_value_at(values: &Vec<String>, index: SifrInt) -> String {
    if (&index < &SifrInt::from_i64(0)) || (&index >= &SifrInt::from(values.len())) {
        return "".to_string();
    }
    for (current_index, value) in Box::new(
        (values)
            .iter()
            .cloned()
            .enumerate()
            .map(|__pair| (SifrInt::from(__pair.0) + SifrInt::from_i64(0), __pair.1)),
    ) {
        if (&current_index == &index) {
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
    _char_at(text, SifrInt::from_i64(0))
}
fn _last_char(text: &String) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    _char_at(text, SifrInt::from(text.chars().count()) - SifrInt::from_i64(1))
}
fn parse_csv(
    text: &String,
    dialect: &Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    quoting: SifrInt,
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
        (quoting).clone(),
    );
    let mut rows: Vec<Vec<String>> = vec![];
    let mut row: Vec<String> = vec![];
    let mut field: String = "".to_string();
    let mut in_quotes: bool = false;
    let mut field_started: bool = false;
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_text.len())) {
        let ch_value: String = _char_at(text, (i).clone());
        if in_quotes {
            if (resolved.escapechar.clone() != "")
                && (ch_value == resolved.escapechar.clone())
            {
                if (&(&i + &SifrInt::from_i64(1))
                    < &SifrInt::from(__sifr_chars_text.len()))
                {
                    let escaped_value: String = _char_at(
                        text,
                        &i + &SifrInt::from_i64(1),
                    );
                    field.push_str((escaped_value).as_str());
                    i = &i + &SifrInt::from_i64(2);
                    continue;
                }
                field.push_str((ch_value).as_str());
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            if (resolved.quotechar.clone() != "")
                && (ch_value == resolved.quotechar.clone())
            {
                let quotechar: String = _quotechar_value(&resolved);
                if (resolved.doublequote
                    && (&(&i + &SifrInt::from_i64(1))
                        < &SifrInt::from(__sifr_chars_text.len())))
                    && (_char_at(text, &i + &SifrInt::from_i64(1)) == quotechar.clone())
                {
                    field.push_str((quotechar).as_str());
                    i = &i + &SifrInt::from_i64(2);
                    continue;
                }
                in_quotes = false;
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            field.push_str((ch_value).as_str());
            i = &i + &SifrInt::from_i64(1);
            continue;
        }
        if (!field_started && resolved.skipinitialspace) && (ch_value == " ") {
            i = &i + &SifrInt::from_i64(1);
            continue;
        }
        if (resolved.escapechar.clone() != "")
            && (ch_value == resolved.escapechar.clone())
        {
            if (&(&i + &SifrInt::from_i64(1)) < &SifrInt::from(__sifr_chars_text.len()))
            {
                let escaped_plain_value: String = _char_at(
                    text,
                    &i + &SifrInt::from_i64(1),
                );
                field.push_str((escaped_plain_value).as_str());
                field_started = true;
                i = &i + &SifrInt::from_i64(2);
                continue;
            }
            field.push_str((ch_value).as_str());
            field_started = true;
            i = &i + &SifrInt::from_i64(1);
            continue;
        }
        if (&resolved.quoting.clone() != &__const_QUOTE_NONE())
            && (resolved.quotechar.clone() != "")
        {
            let quotechar2: String = _quotechar_value(&resolved);
            if (ch_value == quotechar2) {
                in_quotes = true;
                field_started = true;
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
        }
        if (ch_value == resolved.delimiter.clone()) {
            _append_field(&mut row, field);
            field = "".to_string();
            field_started = false;
            i = &i + &SifrInt::from_i64(1);
            continue;
        }
        if (ch_value == "\n") || (ch_value == "\r") {
            if ((ch_value == "\r")
                && (&(&i + &SifrInt::from_i64(1))
                    < &SifrInt::from(__sifr_chars_text.len())))
                && (_char_at(text, &i + &SifrInt::from_i64(1)) == "\n")
            {
                i = &i + &SifrInt::from_i64(1);
            }
            if (&SifrInt::from(row.len()) == &SifrInt::from_i64(0)) && (field == "") {
                _append_row(&mut rows, vec![]);
            } else {
                _append_field(&mut row, field);
                _append_row(&mut rows, row);
            }
            row = vec![];
            field = "".to_string();
            field_started = false;
            i = &i + &SifrInt::from_i64(1);
            continue;
        }
        field.push_str((ch_value).as_str());
        field_started = true;
        i = &i + &SifrInt::from_i64(1);
    }
    if in_quotes {
        in_quotes = false;
    }
    if (&SifrInt::from(row.len()) > &SifrInt::from_i64(0)) || (field != "") {
        _append_field(&mut row, field);
        _append_row(&mut rows, row);
    }
    rows
}
fn _needs_quote(field: &String, dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect) -> bool {
    let __sifr_chars_field: Vec<char> = field.chars().collect::<Vec<char>>();
    if (&dialect.quoting.clone() == &__const_QUOTE_ALL()) {
        return true;
    }
    if (&dialect.quoting.clone() == &__const_QUOTE_NONNUMERIC()) {
        return true;
    }
    if (&dialect.quoting.clone() == &__const_QUOTE_STRINGS()) {
        return true;
    }
    if (&dialect.quoting.clone() == &__const_QUOTE_NOTNULL()) {
        return true;
    }
    if (&dialect.quoting.clone() == &__const_QUOTE_NONE()) {
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
    if (&SifrInt::from(__sifr_chars_field.len()) > &SifrInt::from_i64(0)) {
        let first: String = _first_char(field);
        let last: String = _last_char(field);
        if (first == " ") {
            return true;
        }
        if (last == " ") {
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
    quoting: SifrInt,
) -> String {
    let resolved: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
        dialect,
        delimiter,
        quotechar,
        escapechar,
        doublequote,
        skipinitialspace,
        &"\n".to_string(),
        (quoting).clone(),
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
    quoting: SifrInt,
) -> String {
    let resolved: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
        dialect,
        delimiter,
        quotechar,
        escapechar,
        doublequote,
        skipinitialspace,
        lineterminator,
        (quoting).clone(),
    );
    let mut rendered: Vec<String> = vec![];
    let resolved_delimiter: String = {
        let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
        __sifr_concat.push_str((resolved.delimiter.clone()).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    let resolved_quotechar: String = {
        let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
        __sifr_concat.push_str((resolved.quotechar.clone()).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    let resolved_escapechar: String = {
        let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
        __sifr_concat.push_str((resolved.escapechar.clone()).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    let resolved_lineterminator: String = {
        let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
        __sifr_concat.push_str((resolved.lineterminator.clone()).as_str());
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
                    resolved.quoting.clone(),
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
            .map(|__pair| (SifrInt::from(__pair.0) + SifrInt::from_i64(0), __pair.1)),
    ) {
        if (&i < &SifrInt::from(row.len())) {
            {
                let __assign_value = _list_value_at(row, (i).clone());
                {
                    let __assign_key = key.clone();
                    result.insert(__assign_key, __assign_value);
                }
            }
        } else {
            {
                let __assign_value = {
                    let mut __sifr_concat: String = String::with_capacity(
                        restval.len() + 0usize,
                    );
                    __sifr_concat.push_str((restval).as_str());
                    __sifr_concat.push_str("");
                    __sifr_concat
                };
                {
                    let __assign_key = key.clone();
                    result.insert(__assign_key, __assign_value);
                }
            }
        }
    }
    if ((restkey).as_str() != "")
        && (&SifrInt::from(row.len()) > &SifrInt::from(fieldnames.len()))
    {
        let mut extras: Vec<String> = vec![];
        let mut j: SifrInt = SifrInt::from(fieldnames.len());
        while (&j < &SifrInt::from(row.len())) {
            extras.push(_list_value_at(row, (j).clone()));
            j = &j + &SifrInt::from_i64(1);
        }
        {
            let __assign_value = format!("{:?}", extras);
            {
                let __assign_key = restkey.clone();
                result.insert(__assign_key, __assign_value);
            }
        }
    }
    result
}
fn datetime_now() -> String {
    ::sifr_stdlib::time::datetime_now()
}
fn datetime_now_struct() -> Vec<SifrInt> {
    ::sifr_stdlib::time::datetime_now_struct()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
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
fn time_strptime(s: &String, fmt: &String) -> Result<Vec<SifrInt>, ValueError> {
    ::sifr_stdlib::time::time_strptime(s, fmt)
        .map(|__sifr_bridge_ok| {
            __sifr_bridge_ok
                .into_iter()
                .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
                .collect()
        })
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn time_gmtime() -> Vec<SifrInt> {
    ::sifr_stdlib::time::time_gmtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
fn time_localtime() -> Vec<SifrInt> {
    ::sifr_stdlib::time::time_localtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    __SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(FloatOverflowError),
    __SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
        FloatPrecisionLossError,
    ),
}
impl From<FloatOverflowError>
for __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    fn from(value: FloatOverflowError) -> Self {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
            value,
        )
    }
}
impl ::std::fmt::Display
for __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
        }
    }
}
#[derive(Debug, Clone)]
enum __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    __SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(FloatOverflowError),
    __SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
        FloatPrecisionLossError,
    ),
    __SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(ValueError),
}
impl From<FloatOverflowError>
for __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    fn from(value: FloatOverflowError) -> Self {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
            value,
        )
    }
}
impl ::std::fmt::Display
for __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
        }
    }
}
fn _is_leap_year(year: SifrInt) -> bool {
    (((&year.floor_mod_known_nonzero(&SifrInt::from_i64(4)) == &SifrInt::from_i64(0))
        && (&year.floor_mod_known_nonzero(&SifrInt::from_i64(100))
            != &SifrInt::from_i64(0)))
        || ((&year.floor_mod_known_nonzero(&SifrInt::from_i64(400))
            == &SifrInt::from_i64(0))))
}
fn _days_in_year(year: SifrInt) -> SifrInt {
    if _is_leap_year((year).clone()) {
        return SifrInt::from_i64(366);
    }
    SifrInt::from_i64(365)
}
fn _days_in_month(year: SifrInt, month: SifrInt) -> SifrInt {
    let month_days: Vec<SifrInt> = vec![
        SifrInt::from_i64(31), SifrInt::from_i64(28), SifrInt::from_i64(31),
        SifrInt::from_i64(30), SifrInt::from_i64(31), SifrInt::from_i64(30),
        SifrInt::from_i64(31), SifrInt::from_i64(31), SifrInt::from_i64(30),
        SifrInt::from_i64(31), SifrInt::from_i64(30), SifrInt::from_i64(31)
    ];
    let idx: SifrInt = &month - &SifrInt::from_i64(1);
    let d: Option<SifrInt> = {
        let __sifr_checked_read_collection = &month_days;
        let __sifr_checked_read_index = idx.clone();
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    if (&month == &SifrInt::from_i64(2)) && _is_leap_year((year).clone()) {
        return SifrInt::from_i64(29);
    }
    if let Some(d) = d.clone() {
        return d;
    }
    SifrInt::from_i64(0)
}
fn _substring(value: &String, start: SifrInt, end: SifrInt) -> String {
    let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    let mut result: String = "".to_string();
    let mut i: SifrInt = start.clone();
    while (&i < &end) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_value.len());
            __sifr_chars_value.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            result.push_str((ch).as_str());
        }
        i = &i + &SifrInt::from_i64(1);
    }
    result
}
fn _six_digits(value: SifrInt) -> String {
    let mut rendered: String = format!("{}", value);
    let mut __sifr_chars_rendered: Vec<char> = rendered.chars().collect::<Vec<char>>();
    while (&SifrInt::from(__sifr_chars_rendered.len()) < &SifrInt::from_i64(6)) {
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
) -> Result<(SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt), ValueError> {
    let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    let Some(__sifr_checked_value_2) = ({
        let __sifr_string_index = SifrInt::from_i64(4);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_value.len());
        __sifr_chars_value.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string()) else {
        return Err(ValueError::new("invalid datetime string".to_string()));
    };
    let Some(__sifr_checked_value_3) = ({
        let __sifr_string_index = SifrInt::from_i64(7);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_value.len());
        __sifr_chars_value.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string()) else {
        return Err(ValueError::new("invalid datetime string".to_string()));
    };
    let Some(__sifr_checked_value_4) = ({
        let __sifr_string_index = SifrInt::from_i64(10);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_value.len());
        __sifr_chars_value.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string()) else {
        return Err(ValueError::new("invalid datetime string".to_string()));
    };
    let Some(__sifr_checked_value_5) = ({
        let __sifr_string_index = SifrInt::from_i64(13);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_value.len());
        __sifr_chars_value.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string()) else {
        return Err(ValueError::new("invalid datetime string".to_string()));
    };
    let Some(__sifr_checked_value_6) = ({
        let __sifr_string_index = SifrInt::from_i64(16);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_value.len());
        __sifr_chars_value.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string()) else {
        return Err(ValueError::new("invalid datetime string".to_string()));
    };
    if ((((__sifr_checked_value_2.clone() != "-")
        || (__sifr_checked_value_3.clone() != "-"))
        || (__sifr_checked_value_4.clone() != "T"))
        || (__sifr_checked_value_5.clone() != ":"))
        || (__sifr_checked_value_6.clone() != ":")
    {
        return Err(ValueError::new("invalid datetime string".to_string()));
    }
    let __sifr_try_res: Result<
        Result<(SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt), ValueError>,
        ParseError,
    > = (|| {
        let year: SifrInt = SifrInt::parse_decimal(
                &(_substring(value, SifrInt::from_i64(0), SifrInt::from_i64(4))),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let month: SifrInt = SifrInt::parse_decimal(
                &(_substring(value, SifrInt::from_i64(5), SifrInt::from_i64(7))),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let day: SifrInt = SifrInt::parse_decimal(
                &(_substring(value, SifrInt::from_i64(8), SifrInt::from_i64(10))),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let hour: SifrInt = SifrInt::parse_decimal(
                &(_substring(value, SifrInt::from_i64(11), SifrInt::from_i64(13))),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let minute: SifrInt = SifrInt::parse_decimal(
                &(_substring(value, SifrInt::from_i64(14), SifrInt::from_i64(16))),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let second: SifrInt = SifrInt::parse_decimal(
                &(_substring(value, SifrInt::from_i64(17), SifrInt::from_i64(19))),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        Ok(
            Ok((
                year.clone(),
                month.clone(),
                day.clone(),
                hour.clone(),
                minute.clone(),
                second.clone(),
            )),
        )
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
fn _timezone_offset_from_text(text: &String) -> Result<SifrInt, ValueError> {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    if (text).as_str() == "UTC" {
        return Ok(SifrInt::from_i64(0));
    }
    if (&SifrInt::from(__sifr_chars_text.len()) != &SifrInt::from_i64(9)) {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    if (_substring(text, SifrInt::from_i64(0), SifrInt::from_i64(3)) != "UTC") {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    let sign_value: String = _substring(
        text,
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
    );
    if (sign_value != "+") && (sign_value != "-") {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    if (({
        let __sifr_string_index = SifrInt::from_i64(6);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_text.len());
        __sifr_chars_text.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string()) != Some(":".to_string()))
    {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    let __sifr_try_res: Result<Result<SifrInt, ValueError>, ParseError> = (|| {
        let hours: SifrInt = SifrInt::parse_decimal(
                &(_substring(text, SifrInt::from_i64(4), SifrInt::from_i64(6))),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let minutes: SifrInt = SifrInt::parse_decimal(
                &(_substring(text, SifrInt::from_i64(7), SifrInt::from_i64(9))),
                ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let mut offset: SifrInt = &(&hours * &SifrInt::from_i64(3600))
            + &(&minutes * &SifrInt::from_i64(60));
        if (sign_value == "-") {
            offset = -&offset;
        }
        Ok(Ok(offset))
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
        __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
    > = (|| {
        let whole_seconds: SifrInt = (SifrInt::from_f64_trunc(ts)
            .ok_or_else(|| ValueError {
                message: "cannot convert non-finite float to int".to_string(),
            }))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                __e,
            ))?;
        let whole_seconds_float: f64 = (whole_seconds
            .clone()
            .checked_to_f64()
            .map_err(|__sifr_float_error| match __sifr_float_error {
                ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        FloatOverflowError::new(
                            "exact integer is outside the finite float range".to_string(),
                        ),
                    )
                }
                ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        FloatPrecisionLossError::new(
                            "exact integer cannot be represented without float precision loss"
                                .to_string(),
                        ),
                    )
                }
            }))
            .map_err(|__e| match __e {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    __sifr_union_value,
                ) => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        __sifr_union_value,
                    )
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    __sifr_union_value,
                ) => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        __sifr_union_value,
                    )
                }
            })?;
        let fractional: f64 = ts - whole_seconds_float;
        let mut microsecond: SifrInt = (SifrInt::from_f64_trunc(
                fractional * (1000000.0_f64),
            )
            .ok_or_else(|| ValueError {
                message: "cannot convert non-finite float to int".to_string(),
            }))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                __e,
            ))?;
        if (&microsecond < &SifrInt::from_i64(0)) {
            microsecond = -&microsecond;
        }
        let mut adjusted_seconds: SifrInt = whole_seconds.clone();
        let mut tz_offset_value: SifrInt = SifrInt::from_i64(0);
        let mut tz_has_offset: bool = false;
        if let Some(tz) = tz.as_ref() {
            let tz_text: String = format!("{}", tz);
            let tz_offset: SifrInt = (_timezone_offset_from_text(&tz_text))
                .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    __e,
                ))?;
            adjusted_seconds = &whole_seconds + &tz_offset;
            tz_offset_value = tz_offset;
            tz_has_offset = true;
        }
        let adjusted_seconds_float: f64 = (adjusted_seconds
            .clone()
            .checked_to_f64()
            .map_err(|__sifr_float_error| match __sifr_float_error {
                ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        FloatOverflowError::new(
                            "exact integer is outside the finite float range".to_string(),
                        ),
                    )
                }
                ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        FloatPrecisionLossError::new(
                            "exact integer cannot be represented without float precision loss"
                                .to_string(),
                        ),
                    )
                }
            }))
            .map_err(|__e| match __e {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    __sifr_union_value,
                ) => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        __sifr_union_value,
                    )
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    __sifr_union_value,
                ) => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        __sifr_union_value,
                    )
                }
            })?;
        let rendered: String = (datetime_from_timestamp(adjusted_seconds_float))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                __e,
            ))?;
        let parts: (SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt) = (_parse_datetime_iso(
            &rendered,
        ))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                __e,
            ))?;
        let year_part: Option<SifrInt> = Some((parts).0.clone());
        let month_part: Option<SifrInt> = Some((parts).1.clone());
        let day_part: Option<SifrInt> = Some((parts).2.clone());
        let hour_part: Option<SifrInt> = Some((parts).3.clone());
        let minute_part: Option<SifrInt> = Some((parts).4.clone());
        let second_part: Option<SifrInt> = Some((parts).5.clone());
        let mut year: SifrInt = SifrInt::from_i64(0);
        let mut month: SifrInt = SifrInt::from_i64(1);
        let mut day: SifrInt = SifrInt::from_i64(1);
        let mut hour: SifrInt = SifrInt::from_i64(0);
        let mut minute: SifrInt = SifrInt::from_i64(0);
        let mut second: SifrInt = SifrInt::from_i64(0);
        if let Some(year_part) = year_part.clone() {
            year = year_part;
        }
        if let Some(month_part) = month_part.clone() {
            month = month_part;
        }
        if let Some(day_part) = day_part.clone() {
            day = day_part;
        }
        if let Some(hour_part) = hour_part.clone() {
            hour = hour_part;
        }
        if let Some(minute_part) = minute_part.clone() {
            minute = minute_part;
        }
        if let Some(second_part) = second_part.clone() {
            second = second_part;
        }
        if tz_has_offset {
            return Ok(
                Ok(
                    __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                        (year).clone(),
                        (month).clone(),
                        (day).clone(),
                        (hour).clone(),
                        (minute).clone(),
                        (second).clone(),
                        (microsecond).clone(),
                        Some(tz_offset_value),
                    ),
                ),
            );
        }
        Ok(
            Ok(
                __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                    (year).clone(),
                    (month).clone(),
                    (day).clone(),
                    (hour).clone(),
                    (minute).clone(),
                    (second).clone(),
                    (microsecond).clone(),
                    None,
                ),
            ),
        )
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            match __sifr_try_err {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let e = __sifr_try_variant_error.clone();
                    return Err(ValueError::new(e.message.clone()));
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let e = __sifr_try_variant_error.clone();
                    return Err(ValueError::new(e.message.clone()));
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let e = __sifr_try_variant_error.clone();
                    return Err(ValueError::new(e.message.clone()));
                }
            }
        }
    }
}
fn _from_timestamp_microseconds_with_tz(
    value: SifrInt,
    tz: &Option<__SifrStdlib_sifr_x2edatetime_x2etimezone>,
) -> Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError> {
    let whole_seconds: SifrInt = value
        .floor_div_known_nonzero(&SifrInt::from_i64(1000000));
    let microsecond: SifrInt = value
        .floor_mod_known_nonzero(&SifrInt::from_i64(1000000));
    let __sifr_try_res: Result<
        Result<__SifrStdlib_sifr_x2edatetime_x2edatetime, ValueError>,
        __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
    > = (|| {
        let whole_seconds_float: f64 = (whole_seconds
            .clone()
            .checked_to_f64()
            .map_err(|__sifr_float_error| match __sifr_float_error {
                ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        FloatOverflowError::new(
                            "exact integer is outside the finite float range".to_string(),
                        ),
                    )
                }
                ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        FloatPrecisionLossError::new(
                            "exact integer cannot be represented without float precision loss"
                                .to_string(),
                        ),
                    )
                }
            }))
            .map_err(|__e| match __e {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    __sifr_union_value,
                ) => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        __sifr_union_value,
                    )
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    __sifr_union_value,
                ) => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        __sifr_union_value,
                    )
                }
            })?;
        let result: __SifrStdlib_sifr_x2edatetime_x2edatetime = (_from_timestamp_with_tz(
            whole_seconds_float,
            tz,
        ))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                __e,
            ))?;
        Ok(
            Ok(
                __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
                    result.year.clone(),
                    result.month.clone(),
                    result.day.clone(),
                    result.hour.clone(),
                    result.minute.clone(),
                    result.second.clone(),
                    (microsecond).clone(),
                    result._tz_offset.clone(),
                ),
            ),
        )
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            match __sifr_try_err {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let e = __sifr_try_variant_error.clone();
                    return Err(ValueError::new(e.message.clone()));
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let e = __sifr_try_variant_error.clone();
                    return Err(ValueError::new(e.message.clone()));
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a323_x3a5_x3aclass10_x3aValueError1_x3a031_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aValueError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let e = __sifr_try_variant_error.clone();
                    return Err(ValueError::new(e.message.clone()));
                }
            }
        }
    }
}
fn set_global_level(level: SifrInt) {
    ::sifr_stdlib::logging::set_global_level(
        ::sifr_runtime::interop::SifrIntBridge::from(level),
    );
}
fn get_global_level() -> SifrInt {
    ::sifr_stdlib::logging::get_global_level().into_sifr_int()
}
fn __const_DEBUG() -> SifrInt {
    SifrInt::from_i64(10)
}
fn __const_INFO() -> SifrInt {
    SifrInt::from_i64(20)
}
fn __const_WARNING() -> SifrInt {
    SifrInt::from_i64(30)
}
fn __const_ERROR() -> SifrInt {
    SifrInt::from_i64(40)
}
fn __const_CRITICAL() -> SifrInt {
    SifrInt::from_i64(50)
}
fn __const_NOTSET() -> SifrInt {
    SifrInt::from_i64(0)
}
fn _level_name_to_num(level: &String) -> SifrInt {
    if (level).as_str() == "DEBUG" {
        return __const_DEBUG();
    }
    if (level).as_str() == "INFO" {
        return __const_INFO();
    }
    if (level).as_str() == "WARNING" {
        return __const_WARNING();
    }
    if (level).as_str() == "ERROR" {
        return __const_ERROR();
    }
    if (level).as_str() == "CRITICAL" {
        return __const_CRITICAL();
    }
    __const_NOTSET()
}
fn getLogger(name: &String) -> __SifrStdlib_sifr_x2elogging_x2eLogger {
    let level: SifrInt = get_global_level();
    __SifrStdlib_sifr_x2elogging_x2eLogger::new((name.clone()).clone(), (level).clone())
}
struct __SifrYielder<T> {
    slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
}
struct __SifrYieldFuture<T> {
    slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    value: Option<T>,
}
impl<T> Unpin for __SifrYieldFuture<T> {}
impl<T> ::std::future::Future for __SifrYieldFuture<T> {
    type Output = ();
    fn poll(
        self: ::std::pin::Pin<&mut Self>,
        _cx: &mut ::std::task::Context<'_>,
    ) -> ::std::task::Poll<()> {
        let state = self.get_mut();
        let Some(value) = state.value.take() else {
            return ::std::task::Poll::Ready(());
        };
        __sifr_store_suspended(&state.slot, value);
        ::std::task::Poll::Pending
    }
}
impl<T> __SifrYielder<T> {
    fn suspend(&self, value: T) -> __SifrYieldFuture<T> {
        __SifrYieldFuture {
            slot: ::std::sync::Arc::clone(&self.slot),
            value: Some(value),
        }
    }
}
fn __sifr_store_suspended<T>(
    slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    value: T,
) {
    match slot.lock() {
        Ok(mut state) => *state = Some(value),
        Err(poisoned) => *poisoned.into_inner() = Some(value),
    }
}
fn __sifr_take_suspended<T>(
    slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
) -> Option<T> {
    match slot.lock() {
        Ok(mut state) => state.take(),
        Err(poisoned) => poisoned.into_inner().take(),
    }
}
struct __SifrGenerator<T> {
    producer: Option<
        ::std::pin::Pin<Box<dyn ::std::future::Future<Output = ()> + 'static>>,
    >,
    yielded: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    complete: bool,
}
impl<T> __SifrGenerator<T> {
    fn new<
        F: FnOnce(__SifrYielder<T>) -> Fut + 'static,
        Fut: ::std::future::Future<Output = ()> + 'static,
    >(factory: F) -> Self {
        let yielded = ::std::sync::Arc::new(::std::sync::Mutex::new(None));
        let producer = factory(__SifrYielder {
            slot: ::std::sync::Arc::clone(&yielded),
        });
        Self {
            producer: Some(Box::pin(producer)),
            yielded,
            complete: false,
        }
    }
}
impl<T> Iterator for __SifrGenerator<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.complete {
            return None;
        }
        let completed = {
            let Some(producer) = self.producer.as_mut() else {
                self.complete = true;
                return None;
            };
            let mut context = ::std::task::Context::from_waker(
                ::std::task::Waker::noop(),
            );
            ::std::future::Future::poll(producer.as_mut(), &mut context).is_ready()
        };
        let yielded = __sifr_take_suspended(&self.yielded);
        if completed {
            self.complete = true;
            self.producer = None;
        }
        yielded
    }
}
fn join_path(base: &String, child: &String) -> String {
    let __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
    if (&SifrInt::from(__sifr_chars_base.len()) == &SifrInt::from_i64(0)) {
        return {
            let mut __sifr_concat: String = String::with_capacity(child.len() + 0usize);
            __sifr_concat.push_str((child).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    let last: Option<String> = ({
        let __sifr_string_index = SifrInt::from(base.chars().count())
            - SifrInt::from_i64(1);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_base.len());
        __sifr_chars_base.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string());
    if let Some(last) = last {
        if (last).as_str() == ("/".to_string()).as_str() {
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
    let mut i: SifrInt = &SifrInt::from(__sifr_chars_path.len()) - &SifrInt::from_i64(1);
    while (&i >= &SifrInt::from_i64(0)) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_path.len());
            __sifr_chars_path.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            if (ch == "/") {
                return {
                    let _slice_src = &__sifr_chars_path;
                    let _slice_len = _slice_src.len();
                    let _slice_start = (&i + &SifrInt::from_i64(1))
                        .clamp_slice_bound(_slice_len);
                    let _slice_stop = _slice_len;
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start)
                            .take(_slice_stop.saturating_sub(_slice_start))
                            .copied(),
                    )
                };
            }
        }
        i = &i - &SifrInt::from_i64(1);
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
    let mut i: SifrInt = &SifrInt::from(__sifr_chars_path.len()) - &SifrInt::from_i64(1);
    while (&i >= &SifrInt::from_i64(0)) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_path.len());
            __sifr_chars_path.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            if (ch == "/") {
                return {
                    let _slice_src = &__sifr_chars_path;
                    let _slice_len = _slice_src.len();
                    let _slice_start = 0;
                    let _slice_stop = i.clamp_slice_bound(_slice_len);
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start)
                            .take(_slice_stop.saturating_sub(_slice_start))
                            .copied(),
                    )
                };
            }
        }
        i = &i - &SifrInt::from_i64(1);
    }
    "".to_string()
}
fn extension(path: &String) -> String {
    let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
    let mut i: SifrInt = &SifrInt::from(__sifr_chars_path.len()) - &SifrInt::from_i64(1);
    while (&i >= &SifrInt::from_i64(0)) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_path.len());
            __sifr_chars_path.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            if (ch == ".") {
                return {
                    let _slice_src = &__sifr_chars_path;
                    let _slice_len = _slice_src.len();
                    let _slice_start = i.clamp_slice_bound(_slice_len);
                    let _slice_stop = _slice_len;
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start)
                            .take(_slice_stop.saturating_sub(_slice_start))
                            .copied(),
                    )
                };
            }
            if (ch == "/") {
                return "".to_string();
            }
        }
        i = &i - &SifrInt::from_i64(1);
    }
    "".to_string()
}
fn stem(path: &String) -> String {
    let base: String = basename(path);
    let __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
    let mut i: SifrInt = &SifrInt::from(__sifr_chars_base.len()) - &SifrInt::from_i64(1);
    while (&i > &SifrInt::from_i64(0)) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_base.len());
            __sifr_chars_base.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            if (ch == ".") {
                return {
                    let _slice_src = &__sifr_chars_base;
                    let _slice_len = _slice_src.len();
                    let _slice_start = 0;
                    let _slice_stop = i.clamp_slice_bound(_slice_len);
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start)
                            .take(_slice_stop.saturating_sub(_slice_start))
                            .copied(),
                    )
                };
            }
        }
        i = &i - &SifrInt::from_i64(1);
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
    if (&SifrInt::from(__sifr_chars_path.len()) == &SifrInt::from_i64(0)) {
        return false;
    }
    if (&SifrInt::from(__sifr_chars_path.len()) >= &SifrInt::from_i64(3)) {
        let colon: Option<String> = ({
            let __sifr_string_index = SifrInt::from_i64(1);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_path.len());
            __sifr_chars_path.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        let sep: Option<String> = ({
            let __sifr_string_index = SifrInt::from_i64(2);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_path.len());
            __sifr_chars_path.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(colon) = colon {
            if let Some(sep) = sep {
                if (colon == ":") && ((sep == "/") || (sep == "\\")) {
                    return true;
                }
            }
        }
    }
    let first: Option<String> = ({
        let __sifr_string_index = SifrInt::from_i64(0);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_path.len());
        __sifr_chars_path.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string());
    if let Some(first) = first {
        if (first == "/") || (first == "\\") {
            return true;
        }
    }
    false
}
fn _iter_list_str(entries: Vec<String>) -> Box<dyn Iterator<Item = String>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<String>| {
            let mut i: SifrInt = SifrInt::from_i64(0);
            while (&i < &SifrInt::from(entries.len())) {
                let Some(__sifr_checked_value_7) = ({
                    let __sifr_checked_read_collection = &entries;
                    let __sifr_checked_read_index = i.clone();
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                }) else {
                    break;
                };
                __sifr_yielder.suspend(__sifr_checked_value_7.clone()).await;
                i = &i + &SifrInt::from_i64(1);
            }
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
        Ok(Ok(_iter_list_str(entries)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(e);
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
        Ok(Ok(_iter_list_str(entries)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(e);
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
        Ok(Ok(_iter_list_str(entries)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(e);
        }
    }
}
trait __SifrOpaque__SifrStdlib___sifr_x2eregex_x2eCompiledPatternMethods {
    fn search(&self, text: &String) -> Result<Option<String>, RegexError>;
    fn is_match(&self, text: &String) -> Result<bool, RegexError>;
    fn sub(&self, replacement: &String, text: &String) -> Result<String, RegexError>;
    fn findall(&self, text: &String) -> Result<Vec<String>, RegexError>;
    fn split(&self, text: &String) -> Result<Vec<String>, RegexError>;
    fn pattern(&self) -> Result<String, RegexError>;
    fn flags(&self) -> Result<SifrInt, RegexError>;
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
    flags: SifrInt,
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
fn re_find_start(pattern: &String, text: &String) -> Result<SifrInt, RegexError> {
    ::sifr_stdlib::regex::re_find_start(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_find_end(pattern: &String, text: &String) -> Result<SifrInt, RegexError> {
    ::sifr_stdlib::regex::re_find_end(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_match_flags(
    pattern: &String,
    text: &String,
    flags: SifrInt,
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
    flags: SifrInt,
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
    flags: SifrInt,
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
    flags: SifrInt,
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
    flags: SifrInt,
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
fn _iter_matches(
    matches: Vec<__SifrStdlib_sifr_x2ere_x2eMatch>,
) -> Box<dyn Iterator<Item = __SifrStdlib_sifr_x2ere_x2eMatch>> {
    Box::new(
        __SifrGenerator::new(async move |
            __sifr_yielder: __SifrYielder<__SifrStdlib_sifr_x2ere_x2eMatch>|
        {
            let mut i: SifrInt = SifrInt::from_i64(0);
            while (&i < &SifrInt::from(matches.len())) {
                let Some(__sifr_checked_value_0) = ({
                    let __sifr_checked_read_collection = &matches;
                    let __sifr_checked_read_index = i.clone();
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                }) else {
                    break;
                };
                __sifr_yielder.suspend(__sifr_checked_value_0.clone()).await;
                i = &i + &SifrInt::from_i64(1);
            }
        }),
    )
}
fn _find_index_from(text: &String, needle: &String, start: SifrInt) -> SifrInt {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let __sifr_chars_needle: Vec<char> = needle.chars().collect::<Vec<char>>();
    if &start < &SifrInt::from_i64(0) {
        return -&SifrInt::from_i64(1);
    }
    if (&SifrInt::from(__sifr_chars_needle.len()) == &SifrInt::from_i64(0)) {
        if (&start <= &SifrInt::from(__sifr_chars_text.len())) {
            return start.clone();
        }
        return -&SifrInt::from_i64(1);
    }
    let max_start: SifrInt = &SifrInt::from(__sifr_chars_text.len())
        - &SifrInt::from(__sifr_chars_needle.len());
    let mut i: SifrInt = start.clone();
    while (&i <= &max_start) {
        if (({
            let _slice_src = &__sifr_chars_text;
            let _slice_len = _slice_src.len();
            let _slice_start = i.clamp_slice_bound(_slice_len);
            let _slice_stop = (&i + &SifrInt::from(__sifr_chars_needle.len()))
                .clamp_slice_bound(_slice_len);
            String::from_iter(
                _slice_src
                    .iter()
                    .skip(_slice_start)
                    .take(_slice_stop.saturating_sub(_slice_start))
                    .copied(),
            )
        }) == needle.clone())
        {
            return i.clone();
        }
        i = &i + &SifrInt::from_i64(1);
    }
    -&SifrInt::from_i64(1)
}
fn _finditer_from_items(
    found_items: &Vec<String>,
    text: &String,
) -> Vec<__SifrStdlib_sifr_x2ere_x2eMatch> {
    let mut matches: Vec<__SifrStdlib_sifr_x2ere_x2eMatch> = vec![];
    let mut cursor: SifrInt = SifrInt::from_i64(0);
    for found in found_items.iter().cloned() {
        let __sifr_chars_found: Vec<char> = found.chars().collect::<Vec<char>>();
        let mut start: SifrInt = _find_index_from(text, &found, (cursor).clone());
        if (&start < &SifrInt::from_i64(0)) {
            start = cursor.clone();
        }
        let found_len: SifrInt = SifrInt::from(__sifr_chars_found.len());
        let end: SifrInt = &start + &found_len;
        matches
            .push(
                __SifrStdlib_sifr_x2ere_x2eMatch::new(found, start.clone(), end.clone()),
            );
        if (&found_len == &SifrInt::from_i64(0)) {
            cursor = &end + &SifrInt::from_i64(1);
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
        Ok(
            Ok(
                __SifrStdlib_sifr_x2ere_x2ePattern::new(
                    compiled,
                    (pattern.clone()).clone(),
                    SifrInt::from_i64(0),
                ),
            ),
        )
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(RegexError::new(error.message.clone()));
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
fn __io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
    let msg = e.to_string();
    let kind = {
        let __sifr_io_kind = (&e as &dyn ::std::any::Any)
            .downcast_ref::<std::io::Error>()
            .map(::std::io::Error::kind);
        match __sifr_io_kind {
            Some(::std::io::ErrorKind::NotFound) => "FileNotFound".to_string(),
            Some(::std::io::ErrorKind::PermissionDenied) => {
                "PermissionDenied".to_string()
            }
            Some(::std::io::ErrorKind::AlreadyExists) => "FileExists".to_string(),
            Some(::std::io::ErrorKind::IsADirectory) => "IsADirectory".to_string(),
            Some(::std::io::ErrorKind::NotADirectory) => "NotADirectory".to_string(),
            Some(::std::io::ErrorKind::DirectoryNotEmpty) => {
                "DirectoryNotEmpty".to_string()
            }
            _ => "Other".to_string(),
        }
    };
    IOError { message: msg, kind }
}
fn main() {
    let mut d: __SifrStdlib_sifr_x2ecollections_x2edeque<SifrInt> = __SifrStdlib_sifr_x2ecollections_x2edeque::new(
        None,
        Some(SifrInt::from_i64(3)),
    );
    d.append(&SifrInt::from_i64(1));
    d.append(&SifrInt::from_i64(2));
    d.append(&SifrInt::from_i64(3));
    d.append(&SifrInt::from_i64(4));
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(23usize + 0usize);
        __sifr_concat.push_str("deque len (maxlen=3) = "); __sifr_concat
        .push_str((format!("{}", SifrInt::from(d.len()))).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("deque popleft = "); __sifr_concat.push_str(((d.popleft())
        .map_or("None".to_string().to_string(), | __v | format!("{}", __v))).as_str());
        __sifr_concat }
    );
    let dt: __SifrStdlib_sifr_x2edatetime_x2edatetime = __SifrStdlib_sifr_x2edatetime_x2edatetime::new(
        SifrInt::from_i64(2024),
        SifrInt::from_i64(6),
        SifrInt::from_i64(15),
        SifrInt::from_i64(9),
        SifrInt::from_i64(30),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        None,
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(21usize + 0usize);
        __sifr_concat.push_str("datetime isoformat = "); __sifr_concat.push_str((dt
        .isoformat()).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("datetime year = "); __sifr_concat.push_str((format!("{}",
        dt.year.clone())).as_str()); __sifr_concat }
    );
    let today: __SifrStdlib_sifr_x2edatetime_x2edate = __SifrStdlib_sifr_x2edatetime_x2edate::new(
        SifrInt::from_i64(2024),
        SifrInt::from_i64(6),
        SifrInt::from_i64(15),
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(17usize + 0usize);
        __sifr_concat.push_str("date isoformat = "); __sifr_concat.push_str((today
        .isoformat()).as_str()); __sifr_concat }
    );
    let p: __SifrStdlib_sifr_x2epathlib_x2ePath = __SifrStdlib_sifr_x2epathlib_x2ePath::new(
        "/tmp/demo_file.txt".to_string(),
    );
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _ = p.touch()?;
        println!("path touch ok = true");
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(14usize +
            0usize); __sifr_concat.push_str("path exists = "); __sifr_concat
            .push_str((format!("{}", p.exists())).as_str()); __sifr_concat }
        );
        let _2: () = p.unlink()?;
        println!("path unlink ok = true");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(12usize +
            0usize); __sifr_concat.push_str("path error: "); __sifr_concat.push_str((e
            .message.clone()).as_str()); __sifr_concat }
        );
    }
    let p2: __SifrStdlib_sifr_x2epathlib_x2ePath = __SifrStdlib_sifr_x2epathlib_x2ePath::new(
        "/tmp/myfile.txt".to_string(),
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(14usize + 0usize);
        __sifr_concat.push_str("with_suffix = "); __sifr_concat.push_str((p2
        .with_suffix(& ".csv".to_string()).to_str()).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(12usize + 0usize);
        __sifr_concat.push_str("with_name = "); __sifr_concat.push_str((p2.with_name(&
        "other.txt".to_string()).to_str()).as_str()); __sifr_concat }
    );
    let __sifr_try_res: Result<(), RegexError> = (|| {
        let pat: __SifrStdlib_sifr_x2ere_x2ePattern = compile(&"\\d+".to_string())?;
        let m: bool = pat.is_match(&"abc123".to_string())?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(19usize +
            0usize); __sifr_concat.push_str("pattern is_match = "); __sifr_concat
            .push_str((format!("{}", m)).as_str()); __sifr_concat }
        );
        let found: Option<String> = pat.search(&"hello 42 world".to_string())?;
        if let Some(found) = found {
            println!(
                "{}", { let mut __sifr_concat : String = String::with_capacity(23usize +
                0usize); __sifr_concat.push_str("pattern search found = "); __sifr_concat
                .push_str((format!("{}", SifrInt::from(found.chars().count()) >
                SifrInt::from_i64(0))).as_str()); __sifr_concat }
            );
        }
        let nums: Vec<String> = pat.findall(&"1 plus 2 equals 3".to_string())?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(24usize +
            0usize); __sifr_concat.push_str("pattern findall count = "); __sifr_concat
            .push_str((format!("{}", SifrInt::from(nums.len()))).as_str()); __sifr_concat
            }
        );
        let __sifr_try_res: Result<(), RegexError> = (|| {
            let fm_val: bool = fullmatch(&"\\d+".to_string(), &"12345".to_string())?;
            println!(
                "{}", { let mut __sifr_concat : String = String::with_capacity(19usize +
                0usize); __sifr_concat.push_str("fullmatch digits = "); __sifr_concat
                .push_str((format!("{}", fm_val)).as_str()); __sifr_concat }
            );
            Ok(())
        })();
        if let Err(__sifr_try_err) = __sifr_try_res {
            let e2 = __sifr_try_err.clone();
            println!(
                "{}", { let mut __sifr_concat : String = String::with_capacity(17usize +
                0usize); __sifr_concat.push_str("fullmatch error: "); __sifr_concat
                .push_str((e2.message.clone()).as_str()); __sifr_concat }
            );
        }
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(13usize +
            0usize); __sifr_concat.push_str("regex error: "); __sifr_concat.push_str((e
            .message.clone()).as_str()); __sifr_concat }
        );
    }
    let mut log: __SifrStdlib_sifr_x2elogging_x2eLogger = getLogger(&"demo".to_string());
    log.set_level(&__const_DEBUG());
    log.debug(&"debug message".to_string());
    log.info(&"info message".to_string());
    log.warning(&"warning message".to_string());
    let csv_text: String = "name,age\nalice,30\nbob,25".to_string();
    let r: __SifrStdlib_sifr_x2ecsv_x2ereader = __SifrStdlib_sifr_x2ecsv_x2ereader::new(
        csv_text,
        None,
        ",".to_string(),
        "\"".to_string(),
        "".to_string(),
        true,
        false,
        SifrInt::from_i64(0),
    );
    let all_rows: Vec<Vec<String>> = r.rows();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(11usize + 0usize);
        __sifr_concat.push_str("csv rows = "); __sifr_concat.push_str((format!("{}",
        SifrInt::from(all_rows.len()))).as_str()); __sifr_concat }
    );
    let mut w: __SifrStdlib_sifr_x2ecsv_x2ewriter = __SifrStdlib_sifr_x2ecsv_x2ewriter::new(
        None,
        ",".to_string(),
        "\"".to_string(),
        "".to_string(),
        true,
        false,
        "\n".to_string(),
        SifrInt::from_i64(0),
    );
    let row1: Vec<String> = vec!["x".to_string(), "y".to_string()];
    let row2: Vec<String> = vec!["1".to_string(), "2".to_string()];
    w.writerow(&row1);
    w.writerow(&row2);
    let out: String = w.getvalue();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(20usize + out
        .len()); __sifr_concat.push_str("csv writer output = "); __sifr_concat
        .push_str((out).as_str()); __sifr_concat }
    );
    let dr: __SifrStdlib_sifr_x2ecsv_x2eDictReader = __SifrStdlib_sifr_x2ecsv_x2eDictReader::new(
        "name,score\nalice,95\nbob,87".to_string(),
        None,
        "".to_string(),
        "".to_string(),
        None,
        ",".to_string(),
        "\"".to_string(),
        "".to_string(),
        true,
        false,
        SifrInt::from_i64(0),
    );
    let headers: Vec<String> = dr.fieldnames();
    let first_header: Option<String> = {
        let __sifr_checked_read_collection = &headers;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    if let Some(first_header) = first_header {
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(21usize +
            first_header.len()); __sifr_concat.push_str("dictreader headers = ");
            __sifr_concat.push_str((first_header).as_str()); __sifr_concat }
        );
    }
    let dict_rows: Vec<HashMap<String, String>> = dr.rows();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(23usize + 0usize);
        __sifr_concat.push_str("dictreader row count = "); __sifr_concat
        .push_str((format!("{}", SifrInt::from(dict_rows.len()))).as_str());
        __sifr_concat }
    );
    let mut dw: __SifrStdlib_sifr_x2ecsv_x2eDictWriter = __SifrStdlib_sifr_x2ecsv_x2eDictWriter::new(
        vec!["name".to_string(), "score".to_string()],
        "".to_string(),
        "raise".to_string(),
        None,
        ",".to_string(),
        "\"".to_string(),
        "".to_string(),
        true,
        false,
        "\n".to_string(),
        SifrInt::from_i64(0),
    );
    dw.writeheader();
    let row_data: HashMap<String, String> = HashMap::from([
        ("name".to_string(), "charlie".to_string()),
        ("score".to_string(), "91".to_string()),
    ]);
    dw.writerow(&row_data);
    let dw_out: String = dw.getvalue();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(20usize + dw_out
        .len()); __sifr_concat.push_str("dictwriter output = "); __sifr_concat
        .push_str((dw_out).as_str()); __sifr_concat }
    );
}
