//! File-handle intrinsic lowerers for registry migration.

use crate::RustExpr;

fn owned_str(arg: &str) -> String {
    format!("({arg}).to_string()")
}

fn io_other_error_expr(message: &str) -> String {
    format!("IOError {{ message: \"{message}\".to_string(), kind: \"Other\".to_string() }}")
}

fn wrap_handle_result(
    hid_expr: &str,
    result_ty: &str,
    imports: &str,
    arm_pattern: &str,
    arm_body: &str,
    err_message: &str,
) -> String {
    let err_expr = io_other_error_expr(err_message);
    format!(
        "(|| -> Result<{result_ty}, IOError> {{ \
            {imports} \
            let __hid = ({hid_expr}); \
            let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); \
            match __handles.get_mut(&__hid) {{ \
                Some(SifrFileHandle::{arm_pattern}) => {{ {arm_body} }}, \
                _ => Err({err_expr}) \
            }} \
        }})()"
    )
}

fn remove_handle_stmt(hid_expr: &str) -> String {
    format!("{{ let __hid = ({hid_expr}); __SIFR_FILE_HANDLES.lock().unwrap().remove(&__hid); () }}")
}

fn open_arm(pattern: &str, open_expr: &str, variant: &str, success_expr: &str) -> String {
    let (buffer_ty, buffer_var) = if variant.ends_with("Read") {
        ("BufReader", "__reader")
    } else {
        ("BufWriter", "__writer")
    };
    format!(
        "{pattern} => {{ \
            let __f = {open_expr}.map_err(__io_err)?; \
            let {buffer_var} = {buffer_ty}::new(__f); \
            __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::{variant}({buffer_var})); \
            {success_expr} \
        }}"
    )
}

fn build_open_match(path_ref: &str, success_expr: &str, invalid_expr: &str) -> String {
    let arms = [
        open_arm(
            "\"r\" | \"rt\"",
            &format!("std::fs::File::open({path_ref})"),
            "TextRead",
            success_expr,
        ),
        open_arm(
            "\"w\" | \"wt\"",
            &format!("std::fs::File::create({path_ref})"),
            "TextWrite",
            success_expr,
        ),
        open_arm(
            "\"a\" | \"at\"",
            &format!("std::fs::OpenOptions::new().append(true).create(true).open({path_ref})"),
            "TextWrite",
            success_expr,
        ),
        open_arm(
            "\"rb\"",
            &format!("std::fs::File::open({path_ref})"),
            "BinaryRead",
            success_expr,
        ),
        open_arm(
            "\"wb\"",
            &format!("std::fs::File::create({path_ref})"),
            "BinaryWrite",
            success_expr,
        ),
        open_arm(
            "\"ab\"",
            &format!("std::fs::OpenOptions::new().append(true).create(true).open({path_ref})"),
            "BinaryWrite",
            success_expr,
        ),
    ];
    format!("match __mode.as_str() {{ {}, _ => {invalid_expr} }}", arms.join(", "))
}

pub(super) fn lower_builtin_open(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let path_expr = owned_str(&args[0]);
    let mode_expr = owned_str(&args[1]);
    let match_expr = build_open_match(
        "__path.as_str()",
        "FileHandle { _handle: __handle_id, _mode: __mode.to_string() }",
        "return Err(IOError { message: format!(\"invalid mode: {}\", __mode), kind: \"Other\".to_string() })",
    );
    let code = format!(
        "{{ use std::io::{{BufReader, BufWriter}}; \
            let __path = {path_expr}; \
            let __mode = {mode_expr}; \
            let __handle_id: i64 = {{ \
                use std::sync::atomic::{{AtomicI64, Ordering}}; \
                static __NEXT_FH_ID: AtomicI64 = AtomicI64::new(1); \
                __NEXT_FH_ID.fetch_add(1, Ordering::SeqCst) \
            }}; \
            {match_expr} \
        }}"
    );
    Some(RustExpr::RawCode(code))
}

pub(super) fn lower_open_file(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let path_expr = owned_str(&args[0]);
    let mode_expr = owned_str(&args[1]);
    let match_expr = build_open_match(
        "__path.as_str()",
        "Ok(__handle_id)",
        "Err(IOError { message: format!(\"invalid mode: {}\", __mode), kind: \"Other\".to_string() })",
    );
    let code = format!(
        "(|| -> Result<i64, IOError> {{ \
            use std::io::{{BufReader, BufWriter}}; \
            let __path = {path_expr}; \
            let __mode = {mode_expr}; \
            let __handle_id: i64 = {{ \
                use std::sync::atomic::{{AtomicI64, Ordering}}; \
                static __NEXT_ID: AtomicI64 = AtomicI64::new(1); \
                __NEXT_ID.fetch_add(1, Ordering::SeqCst) \
            }}; \
            {match_expr} \
        }})()"
    );
    Some(RustExpr::RawCode(code))
}

pub(super) fn lower_file_read(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(wrap_handle_result(
        &args[0],
        "String",
        "use std::io::Read;",
        "TextRead(ref mut __r)",
        "let mut __s = String::new(); __r.read_to_string(&mut __s).map_err(__io_err)?; Ok(__s)",
        "file not open for reading",
    )))
}

pub(super) fn lower_file_write(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let write_body = format!(
        "let __data: &str = ({}).as_ref(); __w.write_all(__data.as_bytes()).map_err(__io_err)?; Ok(())",
        args[1]
    );
    Some(RustExpr::RawCode(wrap_handle_result(
        &args[0],
        "()",
        "use std::io::Write;",
        "TextWrite(ref mut __w)",
        &write_body,
        "file not open for writing",
    )))
}

pub(super) fn lower_file_readline(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let readline_body = "let mut __line = String::new(); let __n = __r.read_line(&mut __line).map_err(__io_err)?; if __n == 0 { Ok(None) } else { if __line.ends_with('\\n') { __line.pop(); if __line.ends_with('\\r') { __line.pop(); } } Ok(Some(__line)) }";
    Some(RustExpr::RawCode(wrap_handle_result(
        &args[0],
        "Option<String>",
        "use std::io::BufRead;",
        "TextRead(ref mut __r)",
        readline_body,
        "file not open for reading",
    )))
}

pub(super) fn lower_file_readlines(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let readlines_body = "let mut __lines: Vec<String> = Vec::new(); let mut __line = String::new(); loop { __line.clear(); let __n = __r.read_line(&mut __line).map_err(__io_err)?; if __n == 0 { break; } let mut __l = __line.clone(); if __l.ends_with('\\n') { __l.pop(); if __l.ends_with('\\r') { __l.pop(); } } __lines.push(__l); } Ok(__lines)";
    Some(RustExpr::RawCode(wrap_handle_result(
        &args[0],
        "Vec<String>",
        "use std::io::BufRead;",
        "TextRead(ref mut __r)",
        readlines_body,
        "file not open for reading",
    )))
}

pub(super) fn lower_file_close(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(remove_handle_stmt(&args[0])))
}

pub(super) fn lower_file_read_bytes(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let read_bytes_body =
        "let mut __buf = Vec::new(); __r.read_to_end(&mut __buf).map_err(__io_err)?; Ok(__buf.iter().map(|&b| b as i64).collect())";
    Some(RustExpr::RawCode(wrap_handle_result(
        &args[0],
        "Vec<i64>",
        "use std::io::Read;",
        "BinaryRead(ref mut __r)",
        read_bytes_body,
        "file not open for binary reading",
    )))
}

pub(super) fn lower_file_write_bytes(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let write_bytes_body = format!(
        "let __data: Vec<u8> = ({}).iter().map(|&b| b as u8).collect(); __w.write_all(&__data).map_err(__io_err)?; Ok(())",
        args[1]
    );
    Some(RustExpr::RawCode(wrap_handle_result(
        &args[0],
        "()",
        "use std::io::Write;",
        "BinaryWrite(ref mut __w)",
        &write_bytes_body,
        "file not open for binary writing",
    )))
}
