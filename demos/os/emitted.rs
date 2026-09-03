// src/main.rs
mod sifr_generated_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IOError {
        pub message: String,
        pub kind: String,
    }
    impl ::std::fmt::Display for IOError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for IOError {}
}
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::IOError;
fn write_text(path: &str, content: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::write_text(path, content).map_err(sifr_generated_io_err)
}
fn listdir(path: &str) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::listdir(path).map_err(sifr_generated_io_err)
}
fn mkdir(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::mkdir(path).map_err(sifr_generated_io_err)
}
fn rmdir(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rmdir(path).map_err(sifr_generated_io_err)
}
fn remove_file(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::remove_file(path).map_err(sifr_generated_io_err)
}
fn stat_size(path: &str) -> Result<SifrInt, IOError> {
    ::sifr_stdlib::fs::stat_size(path)
        .map(::sifr_runtime::interop::SifrIntBridge::into_sifr_int)
        .map_err(sifr_generated_io_err)
}
fn is_file(path: &str) -> bool {
    ::sifr_stdlib::fs::is_file(path)
}
fn is_dir(path: &str) -> bool {
    ::sifr_stdlib::fs::is_dir(path)
}
fn run_command(cmd: &str) -> Result<String, IOError> {
    ::sifr_stdlib::sys::run_command(cmd).map_err(sifr_generated_io_err)
}
fn getpid() -> SifrInt {
    ::sifr_stdlib::sys::getpid().into_sifr_int()
}
fn stat(path: &str) -> Result<SifrInt, IOError> {
    stat_size(path)
}
fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
        assert_eq!(
            {
                let sifr_generated_condition_list = &actual;
                let sifr_generated_condition_index = i.clone();
                let sifr_generated_condition_normalized = sifr_generated_condition_index
                    .normalize_index_or_len(sifr_generated_condition_list.len());
                sifr_generated_condition_list
                    .get(sifr_generated_condition_normalized)
                    .copied()
            },
            {
                let sifr_generated_condition_list = &expected;
                let sifr_generated_condition_index = i.clone();
                let sifr_generated_condition_normalized = sifr_generated_condition_index
                    .normalize_index_or_len(sifr_generated_condition_list.len());
                sifr_generated_condition_list
                    .get(sifr_generated_condition_normalized)
                    .copied()
            }
        );
        i = &i + &SifrInt::from_i64(1);
    }
}
fn sifr_generated_io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
    let msg = e.to_string();
    let kind = {
        let sifr_generated_io_kind = (&e as &dyn ::std::any::Any)
            .downcast_ref::<std::io::Error>()
            .map(::std::io::Error::kind);
        match sifr_generated_io_kind {
            Some(::std::io::ErrorKind::NotFound) => "FileNotFound".to_string(),
            Some(::std::io::ErrorKind::PermissionDenied) => "PermissionDenied".to_string(),
            Some(::std::io::ErrorKind::AlreadyExists) => "FileExists".to_string(),
            Some(::std::io::ErrorKind::IsADirectory) => "IsADirectory".to_string(),
            Some(::std::io::ErrorKind::NotADirectory) => "NotADirectory".to_string(),
            Some(::std::io::ErrorKind::DirectoryNotEmpty) => "DirectoryNotEmpty".to_string(),
            _ => "Other".to_string(),
        }
    };
    IOError { message: msg, kind }
}
fn collect_runtime_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let mut shell_ok: bool = false;
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let output: String = run_command(&"echo sifr_os_demo".to_string())?;
        shell_ok = output == "sifr_os_demo";
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
    }
    actual.push(shell_ok);
    actual
}
fn collect_filesystem_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let base: String = {
        let mut sifr_generated_concat: String = String::with_capacity(21usize);
        sifr_generated_concat.push_str("/tmp/sifr_os_os_demo_");
        sifr_generated_concat.push_str(getpid().to_string().as_str());
        sifr_generated_concat
    };
    let file_path: String = {
        let mut sifr_generated_concat: String = String::with_capacity(base.len() + 9usize);
        sifr_generated_concat.push_str(base.as_str());
        sifr_generated_concat.push_str("/demo.txt");
        sifr_generated_concat
    };
    let mut os_flow_ok: bool = false;
    let mut list_ok: bool = false;
    let mut stat_ok: bool = false;
    let mut cleanup_ok: bool = false;
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        mkdir(&base)?;
        write_text(&file_path, &"demo".to_string())?;
        os_flow_ok = is_dir(&base) && is_file(&file_path);
        let entries: Vec<String> = listdir(&base)?;
        list_ok = &SifrInt::from(entries.len()) >= &SifrInt::from_i64(1);
        let size: SifrInt = stat(&file_path)?;
        stat_ok = &size > &SifrInt::from_i64(0);
        remove_file(&file_path)?;
        rmdir(&base)?;
        cleanup_ok = !is_dir(&base);
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
    }
    actual.push(os_flow_ok);
    actual.push(list_ok);
    actual.push(stat_ok);
    actual.push(cleanup_ok);
    actual
}
fn collect_missing_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let mut missing_rejected: bool = false;
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        rmdir(&"/tmp/sifr_os_os_demo_missing".to_string())?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
        missing_rejected = true;
    }
    actual.push(missing_rejected);
    actual
}
fn append_all(target: &mut Vec<bool>, values: &[bool]) {
    for value in values.iter().copied() {
        target.push(value);
    }
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let mut actual: Vec<bool> = Vec::new();
    append_all(&mut actual, &collect_runtime_actual());
    append_all(&mut actual, &collect_filesystem_actual());
    append_all(&mut actual, &collect_missing_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("os os parity demo: pass");
}
