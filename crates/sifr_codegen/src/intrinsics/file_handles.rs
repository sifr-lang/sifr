//! File-handle intrinsic lowerers for registry migration.

use crate::RustExpr;

fn owned_str(arg: &str) -> String {
    format!("({arg}).to_string()")
}

pub(super) fn lower_builtin_open(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let mut code = String::new();
    code.push_str("{ use std::io::{BufReader, BufWriter}; let __path = ");
    code.push_str(&owned_str(&args[0]));
    code.push_str("; let __mode = ");
    code.push_str(&owned_str(&args[1]));
    code.push_str("; let __handle_id: i64 = { use std::sync::atomic::{AtomicI64, Ordering}; static __NEXT_FH_ID: AtomicI64 = AtomicI64::new(1); __NEXT_FH_ID.fetch_add(1, Ordering::SeqCst) }; match __mode.as_str() { \"r\" | \"rt\" => { let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?; let __reader = BufReader::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::TextRead(__reader)); FileHandle { _handle: __handle_id, _mode: __mode.to_string() } }, \"w\" | \"wt\" => { let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::TextWrite(__writer)); FileHandle { _handle: __handle_id, _mode: __mode.to_string() } }, \"a\" | \"at\" => { let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path.as_str()).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::TextWrite(__writer)); FileHandle { _handle: __handle_id, _mode: __mode.to_string() } }, \"rb\" => { let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?; let __reader = BufReader::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::BinaryRead(__reader)); FileHandle { _handle: __handle_id, _mode: __mode.to_string() } }, \"wb\" => { let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::BinaryWrite(__writer)); FileHandle { _handle: __handle_id, _mode: __mode.to_string() } }, \"ab\" => { let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path.as_str()).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::BinaryWrite(__writer)); FileHandle { _handle: __handle_id, _mode: __mode.to_string() } }, _ => return Err(IOError { message: format!(\"invalid mode: {}\", __mode), kind: \"Other\".to_string() }) } }");
    Some(RustExpr::RawCode(code))
}

pub(super) fn lower_open_file(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let mut code = String::new();
    code.push_str("(|| -> Result<i64, IOError> { use std::io::{BufReader, BufWriter}; let __path = ");
    code.push_str(&owned_str(&args[0]));
    code.push_str("; let __mode = ");
    code.push_str(&owned_str(&args[1]));
    code.push_str("; let __handle_id: i64 = { use std::sync::atomic::{AtomicI64, Ordering}; static __NEXT_ID: AtomicI64 = AtomicI64::new(1); __NEXT_ID.fetch_add(1, Ordering::SeqCst) }; let __mode_s: &str = __mode.as_str(); let __path_s: &str = __path.as_str(); match __mode_s { \"r\" | \"rt\" => { let __f = std::fs::File::open(__path_s).map_err(__io_err)?; let __reader = BufReader::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::TextRead(__reader)); Ok(__handle_id) }, \"w\" | \"wt\" => { let __f = std::fs::File::create(__path_s).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::TextWrite(__writer)); Ok(__handle_id) }, \"a\" | \"at\" => { let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path_s).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::TextWrite(__writer)); Ok(__handle_id) }, \"rb\" => { let __f = std::fs::File::open(__path_s).map_err(__io_err)?; let __reader = BufReader::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::BinaryRead(__reader)); Ok(__handle_id) }, \"wb\" => { let __f = std::fs::File::create(__path_s).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::BinaryWrite(__writer)); Ok(__handle_id) }, \"ab\" => { let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path_s).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::BinaryWrite(__writer)); Ok(__handle_id) }, _ => Err(IOError { message: format!(\"invalid mode: {}\", __mode), kind: \"Other\".to_string() }) } })()");
    Some(RustExpr::RawCode(code))
}

pub(super) fn lower_file_read(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<String, IOError> {{ use std::io::Read; let __hid = ({}); let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) {{ Some(SifrFileHandle::TextRead(ref mut __r)) => {{ let mut __s = String::new(); __r.read_to_string(&mut __s).map_err(__io_err)?; Ok(__s) }}, _ => Err(IOError {{ message: \"file not open for reading\".to_string(), kind: \"Other\".to_string() }}) }} }})()",
        args[0]
    )))
}

pub(super) fn lower_file_write(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<(), IOError> {{ use std::io::Write; let __hid = ({}); let __data: &str = ({}).as_ref(); let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) {{ Some(SifrFileHandle::TextWrite(ref mut __w)) => {{ __w.write_all(__data.as_bytes()).map_err(__io_err)?; Ok(()) }}, _ => Err(IOError {{ message: \"file not open for writing\".to_string(), kind: \"Other\".to_string() }}) }} }})()",
        args[0], args[1]
    )))
}

pub(super) fn lower_file_readline(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<Option<String>, IOError> {{ use std::io::BufRead; let __hid = ({}); let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) {{ Some(SifrFileHandle::TextRead(ref mut __r)) => {{ let mut __line = String::new(); let __n = __r.read_line(&mut __line).map_err(__io_err)?; if __n == 0 {{ Ok(None) }} else {{ if __line.ends_with('\\n') {{ __line.pop(); if __line.ends_with('\\r') {{ __line.pop(); }} }} Ok(Some(__line)) }} }}, _ => Err(IOError {{ message: \"file not open for reading\".to_string(), kind: \"Other\".to_string() }}) }} }})()",
        args[0]
    )))
}

pub(super) fn lower_file_readlines(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<Vec<String>, IOError> {{ use std::io::BufRead; let __hid = ({}); let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) {{ Some(SifrFileHandle::TextRead(ref mut __r)) => {{ let mut __lines: Vec<String> = Vec::new(); let mut __line = String::new(); loop {{ __line.clear(); let __n = __r.read_line(&mut __line).map_err(__io_err)?; if __n == 0 {{ break; }} let mut __l = __line.clone(); if __l.ends_with('\\n') {{ __l.pop(); if __l.ends_with('\\r') {{ __l.pop(); }} }} __lines.push(__l); }} Ok(__lines) }}, _ => Err(IOError {{ message: \"file not open for reading\".to_string(), kind: \"Other\".to_string() }}) }} }})()",
        args[0]
    )))
}

pub(super) fn lower_file_close(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __hid = ({}); __SIFR_FILE_HANDLES.lock().unwrap().remove(&__hid); () }}",
        args[0]
    )))
}

pub(super) fn lower_file_read_bytes(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<Vec<i64>, IOError> {{ use std::io::Read; let __hid = ({}); let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) {{ Some(SifrFileHandle::BinaryRead(ref mut __r)) => {{ let mut __buf = Vec::new(); __r.read_to_end(&mut __buf).map_err(__io_err)?; Ok(__buf.iter().map(|&b| b as i64).collect()) }}, _ => Err(IOError {{ message: \"file not open for binary reading\".to_string(), kind: \"Other\".to_string() }}) }} }})()",
        args[0]
    )))
}

pub(super) fn lower_file_write_bytes(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<(), IOError> {{ use std::io::Write; let __hid = ({}); let __data: Vec<u8> = ({}).iter().map(|&b| b as u8).collect(); let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) {{ Some(SifrFileHandle::BinaryWrite(ref mut __w)) => {{ __w.write_all(&__data).map_err(__io_err)?; Ok(()) }}, _ => Err(IOError {{ message: \"file not open for binary writing\".to_string(), kind: \"Other\".to_string() }}) }} }})()",
        args[0], args[1]
    )))
}
