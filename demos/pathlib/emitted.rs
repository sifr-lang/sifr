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
    pub use ::sifr_runtime::SifrInt;
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
    pub fn _file_read_bytes(handle: &String) -> Result<Vec<u8>, IOError> {
        ::sifr_stdlib::fs::file_read_bytes(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_write_bytes(handle: &String, data: &Vec<u8>) -> Result<(), IOError> {
        ::sifr_stdlib::fs::file_write_bytes(handle, data)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn open_file(
        path: &String,
        mode: &String,
    ) -> Result<__SifrIoNativeFileHandle, IOError> {
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
                return Err(IOError::new(e.message.clone()));
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
    pub fn file_read_bytes(handle: &__SifrIoNativeFileHandle) -> Result<Vec<u8>, IOError> {
        _file_read_bytes(&handle._id.clone())
    }
    pub fn file_write_bytes(
        handle: &__SifrIoNativeFileHandle,
        data: &Vec<u8>,
    ) -> Result<(), IOError> {
        _file_write_bytes(&handle._id.clone(), data)
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
            if parent == "" {
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
            if parent == "" {
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
        let last: Option<String> = __sifr_chars_base
            .get(
                ::sifr_runtime::to_usize_proven(
                    &(SifrInt::from(base.chars().count()) - SifrInt::from_i64(1)),
                ),
            )
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
        while &i >= &SifrInt::from_i64(0) {
            let ch: Option<String> = Some({
                let Some(__indexed_char) = __sifr_chars_path
                    .get(::sifr_runtime::to_usize_proven(&(i)))
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            });
            if let Some(ch) = ch {
                if ch == "/" {
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
        while &i >= &SifrInt::from_i64(0) {
            let ch: Option<String> = Some({
                let Some(__indexed_char) = __sifr_chars_path
                    .get(::sifr_runtime::to_usize_proven(&(i)))
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            });
            if let Some(ch) = ch {
                if ch == "/" {
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
        while &i >= &SifrInt::from_i64(0) {
            let ch: Option<String> = Some({
                let Some(__indexed_char) = __sifr_chars_path
                    .get(::sifr_runtime::to_usize_proven(&(i)))
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            });
            if let Some(ch) = ch {
                if ch == "." {
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
                if ch == "/" {
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
        while &i > &SifrInt::from_i64(0) {
            let ch: Option<String> = Some({
                let Some(__indexed_char) = __sifr_chars_base
                    .get(::sifr_runtime::to_usize_proven(&(i)))
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            });
            if let Some(ch) = ch {
                if ch == "." {
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
            let colon: Option<String> = __sifr_chars_path
                .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(1))))
                .map(|c| c.to_string());
            let sep: Option<String> = __sifr_chars_path
                .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(2))))
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
                .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(0))))
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
    pub fn _iter_list_str(entries: Vec<String>) -> Box<dyn Iterator<Item = String>> {
        let mut __sifr_generator_initialized: bool = false;
        let mut __sifr_generator_iter: ::std::vec::IntoIter<String> = Vec::new().into_iter();
        Box::new(
            ::std::iter::from_fn(move || {
                if !__sifr_generator_initialized {
                    let mut _yields: Vec<String> = Vec::new();
                    let mut i: SifrInt = SifrInt::from_i64(0);
                    while (&i < &SifrInt::from(entries.len())) {
                        _yields.push(entries[::sifr_runtime::to_usize_proven(&(i))].clone());
                        i = &i + &SifrInt::from_i64(1);
                    }
                    __sifr_generator_iter = _yields.into_iter();
                    __sifr_generator_initialized = true;
                }
                __sifr_generator_iter.next()
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
            return Ok(Ok(_iter_list_str(entries)));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(IOError::new(e.message.clone()));
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
            return Ok(Ok(_iter_list_str(entries)));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(IOError::new(e.message.clone()));
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
            return Ok(Ok(_iter_list_str(entries)));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(IOError::new(e.message.clone()));
            }
        }
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
}
pub use __sifr_project_nominals::IOError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2epathlib_x2ePath;
use ::sifr_runtime::SifrInt;
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
            return Err(IOError::new(e.message.clone()));
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
fn run_command(cmd: &String) -> Result<String, IOError> {
    ::sifr_stdlib::sys::run_command(cmd)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn env_get(key: &String) -> Option<String> {
    ::sifr_stdlib::sys::env_get(key)
}
fn env_keys() -> Vec<String> {
    ::sifr_stdlib::sys::env_keys()
}
fn env_values() -> Vec<String> {
    ::sifr_stdlib::sys::env_values()
}
fn env_items() -> Vec<String> {
    ::sifr_stdlib::sys::env_items()
}
fn get_args() -> Vec<String> {
    ::sifr_stdlib::sys::get_args()
}
fn sys_exit(code: SifrInt) {
    ::sifr_stdlib::sys::sys_exit(::sifr_runtime::interop::SifrIntBridge::from(code));
}
fn sys_version() -> String {
    ::sifr_stdlib::sys::sys_version()
}
fn sys_platform() -> String {
    ::sifr_stdlib::sys::sys_platform()
}
fn sys_maxsize() -> SifrInt {
    ::sifr_stdlib::sys::sys_maxsize().into_sifr_int()
}
fn getpid() -> SifrInt {
    ::sifr_stdlib::sys::getpid().into_sifr_int()
}
fn cpu_count() -> SifrInt {
    ::sifr_stdlib::sys::cpu_count().into_sifr_int()
}
fn which(name: &String) -> Option<String> {
    ::sifr_stdlib::sys::which(name)
}
fn os_sep() -> String {
    ::sifr_stdlib::sys::os_sep()
}
fn os_linesep() -> String {
    ::sifr_stdlib::sys::os_linesep()
}
fn os_name() -> String {
    ::sifr_stdlib::sys::os_name()
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
    let last: Option<String> = __sifr_chars_base
        .get(
            ::sifr_runtime::to_usize_proven(
                &(SifrInt::from(base.chars().count()) - SifrInt::from_i64(1)),
            ),
        )
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
    while &i >= &SifrInt::from_i64(0) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_path
                .get(::sifr_runtime::to_usize_proven(&(i)))
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch) = ch {
            if ch == "/" {
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
    while &i >= &SifrInt::from_i64(0) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_path
                .get(::sifr_runtime::to_usize_proven(&(i)))
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch) = ch {
            if ch == "/" {
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
    while &i >= &SifrInt::from_i64(0) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_path
                .get(::sifr_runtime::to_usize_proven(&(i)))
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch) = ch {
            if ch == "." {
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
            if ch == "/" {
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
    while &i > &SifrInt::from_i64(0) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_base
                .get(::sifr_runtime::to_usize_proven(&(i)))
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch) = ch {
            if ch == "." {
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
        let colon: Option<String> = __sifr_chars_path
            .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(1))))
            .map(|c| c.to_string());
        let sep: Option<String> = __sifr_chars_path
            .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(2))))
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
            .get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(0))))
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
                let mut i: SifrInt = SifrInt::from_i64(0);
                while (&i < &SifrInt::from(entries.len())) {
                    _yields.push(entries[::sifr_runtime::to_usize_proven(&(i))].clone());
                    i = &i + &SifrInt::from_i64(1);
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
            return Err(IOError::new(e.message.clone()));
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
            return Err(IOError::new(e.message.clone()));
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
            return Err(IOError::new(e.message.clone()));
        }
    }
}
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
        assert!(
            Some(actual[::sifr_runtime::to_usize_proven(& (i))]) == expected
            .get(::sifr_runtime::to_usize_proven(& (i))).copied()
        );
        i = &i + &SifrInt::from_i64(1);
    }
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
fn collect_path_helpers_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual
        .push(
            (basename(&"/tmp/demo.txt".to_string())).as_str()
                == ("demo.txt".to_string()).as_str(),
        );
    actual
        .push(
            (join_path(&"/tmp".to_string(), &"demo.txt".to_string())).as_str()
                == ("/tmp/demo.txt".to_string()).as_str(),
        );
    actual
}
fn collect_path_class_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let base: String = {
        let mut __sifr_concat: String = String::with_capacity(31usize + 0usize);
        __sifr_concat.push_str("/tmp/sifr_pathlib_pathlib_demo_");
        __sifr_concat.push_str((format!("{}", getpid())).as_str());
        __sifr_concat
    };
    let filep: __SifrStdlib_sifr_x2epathlib_x2ePath = __SifrStdlib_sifr_x2epathlib_x2ePath::new(
        format!("{}{}", base, "/demo.txt"),
    );
    let dirp: __SifrStdlib_sifr_x2epathlib_x2ePath = __SifrStdlib_sifr_x2epathlib_x2ePath::new(
        format!("{}{}", base, ""),
    );
    let mut path_flow_ok: bool = false;
    let mut glob_ok: bool = false;
    let mut cleanup_ok: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _m: () = dirp.mkdir()?;
        let _w: () = filep.write_text(&"hello".to_string())?;
        let content: String = filep.read_text()?;
        path_flow_ok = (((filep.exists()) && (filep.is_file())) && (content == "hello"));
        let matches_it: Box<dyn Iterator<Item = String>> = dirp
            .glob(&"*.txt".to_string())?;
        let matches: Vec<String> = matches_it.collect::<Vec<_>>();
        glob_ok = (&SifrInt::from(matches.len()) >= &SifrInt::from_i64(1));
        let _u: () = filep.unlink()?;
        let _r: () = dirp.rmdir()?;
        cleanup_ok = !(dirp.exists());
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
    }
    actual.push(path_flow_ok);
    actual.push(glob_ok);
    actual.push(cleanup_ok);
    actual
}
fn collect_missing_path_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut missing_rejected: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _bad: String = __SifrStdlib_sifr_x2epathlib_x2ePath::new(
                "/tmp/sifr_pathlib_pathlib_demo_missing.txt".to_string(),
            )
            .read_text()?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
        missing_rejected = true;
    }
    actual.push(missing_rejected);
    actual
}
fn append_all(target: &mut Vec<bool>, values: &Vec<bool>) {
    for value in values.iter().copied() {
        target.push(value);
    }
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let mut actual: Vec<bool> = vec![];
    append_all(&mut actual, &collect_path_helpers_actual());
    append_all(&mut actual, &collect_path_class_actual());
    append_all(&mut actual, &collect_missing_path_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("pathlib pathlib parity demo: pass");
}
