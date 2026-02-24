//! File-handle intrinsic lowerers for registry migration.

use crate::RustExpr;

fn owned_str(arg: &str) -> String {
    format!("({arg}).to_string()")
}

fn io_other_error_expr(message: &str) -> String {
    format!("IOError {{ message: \"{message}\".to_string(), kind: \"Other\".to_string() }}")
}

fn invalid_mode_error_expr(with_return: bool) -> String {
    let mut code = String::new();
    if with_return {
        code.push_str("return ");
    }
    code.push_str("Err(IOError { message: format!(\"invalid mode: {}\", __mode), ");
    code.push_str("kind: \"Other\".to_string() })");
    code
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
    let mut code = String::new();
    code.push_str("(|| -> Result<");
    code.push_str(result_ty);
    code.push_str(", IOError> { ");
    code.push_str(imports);
    code.push_str(" let __hid = (");
    code.push_str(hid_expr);
    code.push_str("); let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); ");
    code.push_str("match __handles.get_mut(&__hid) { Some(SifrFileHandle::");
    code.push_str(arm_pattern);
    code.push_str(") => { ");
    code.push_str(arm_body);
    code.push_str(" }, _ => Err(");
    code.push_str(&err_expr);
    code.push_str(") } })()");
    code
}

fn remove_handle_stmt(hid_expr: &str) -> String {
    let mut code = String::new();
    code.push_str("{ let __hid = (");
    code.push_str(hid_expr);
    code.push_str("); __SIFR_FILE_HANDLES.lock().unwrap().remove(&__hid); () }");
    code
}

fn open_arm(pattern: &str, open_expr: &str, variant: &str, success_expr: &str) -> String {
    let (buffer_ty, buffer_var) = if variant.ends_with("Read") {
        ("BufReader", "__reader")
    } else {
        ("BufWriter", "__writer")
    };
    let mut code = String::new();
    code.push_str(pattern);
    code.push_str(" => { let __f = ");
    code.push_str(open_expr);
    code.push_str(".map_err(__io_err)?; let ");
    code.push_str(buffer_var);
    code.push_str(" = ");
    code.push_str(buffer_ty);
    code.push_str("::new(__f); ");
    code.push_str("__SIFR_FILE_HANDLES.lock().unwrap().insert(");
    code.push_str("__handle_id, SifrFileHandle::");
    code.push_str(variant);
    code.push('(');
    code.push_str(buffer_var);
    code.push_str(")); ");
    code.push_str(success_expr);
    code.push_str(" }");
    code
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
        &invalid_mode_error_expr(true),
    );
    let mut code = String::new();
    code.push_str("{ use std::io::{BufReader, BufWriter}; ");
    code.push_str("let __path = ");
    code.push_str(&path_expr);
    code.push_str("; let __mode = ");
    code.push_str(&mode_expr);
    code.push_str("; let __handle_id: i64 = { ");
    code.push_str("use std::sync::atomic::{AtomicI64, Ordering}; ");
    code.push_str("static __NEXT_FH_ID: AtomicI64 = AtomicI64::new(1); ");
    code.push_str("__NEXT_FH_ID.fetch_add(1, Ordering::SeqCst) }; ");
    code.push_str(&match_expr);
    code.push_str(" }");
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
        &invalid_mode_error_expr(false),
    );
    let mut code = String::new();
    code.push_str("(|| -> Result<i64, IOError> { ");
    code.push_str("use std::io::{BufReader, BufWriter}; ");
    code.push_str("let __path = ");
    code.push_str(&path_expr);
    code.push_str("; let __mode = ");
    code.push_str(&mode_expr);
    code.push_str("; let __handle_id: i64 = { ");
    code.push_str("use std::sync::atomic::{AtomicI64, Ordering}; ");
    code.push_str("static __NEXT_ID: AtomicI64 = AtomicI64::new(1); ");
    code.push_str("__NEXT_ID.fetch_add(1, Ordering::SeqCst) }; ");
    code.push_str(&match_expr);
    code.push_str(" })()");
    Some(RustExpr::RawCode(code))
}

pub(super) fn lower_file_read(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let mut read_body = String::new();
    read_body.push_str("let mut __s = String::new(); ");
    read_body.push_str("__r.read_to_string(&mut __s).map_err(__io_err)?; ");
    read_body.push_str("Ok(__s)");
    Some(RustExpr::RawCode(wrap_handle_result(
        &args[0],
        "String",
        "use std::io::Read;",
        "TextRead(ref mut __r)",
        &read_body,
        "file not open for reading",
    )))
}

pub(super) fn lower_file_write(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let mut write_body = String::new();
    write_body.push_str("let __data: &str = (");
    write_body.push_str(&args[1]);
    write_body.push_str(").as_ref(); ");
    write_body.push_str("__w.write_all(__data.as_bytes()).map_err(__io_err)?; ");
    write_body.push_str("Ok(())");
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
    let mut readline_body = String::new();
    readline_body.push_str("let mut __line = String::new(); ");
    readline_body.push_str("let __n = __r.read_line(&mut __line).map_err(__io_err)?; ");
    readline_body.push_str("if __n == 0 { Ok(None) } else { ");
    readline_body.push_str("if __line.ends_with('\\n') { __line.pop(); ");
    readline_body.push_str("if __line.ends_with('\\r') { __line.pop(); } } ");
    readline_body.push_str("Ok(Some(__line)) }");
    Some(RustExpr::RawCode(wrap_handle_result(
        &args[0],
        "Option<String>",
        "use std::io::BufRead;",
        "TextRead(ref mut __r)",
        &readline_body,
        "file not open for reading",
    )))
}

pub(super) fn lower_file_readlines(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let mut readlines_body = String::new();
    readlines_body.push_str("let mut __lines: Vec<String> = Vec::new(); ");
    readlines_body.push_str("let mut __line = String::new(); loop { __line.clear(); ");
    readlines_body.push_str("let __n = __r.read_line(&mut __line).map_err(__io_err)?; ");
    readlines_body.push_str("if __n == 0 { break; } let mut __l = __line.clone(); ");
    readlines_body.push_str("if __l.ends_with('\\n') { __l.pop(); ");
    readlines_body.push_str("if __l.ends_with('\\r') { __l.pop(); } } ");
    readlines_body.push_str("__lines.push(__l); } Ok(__lines)");
    Some(RustExpr::RawCode(wrap_handle_result(
        &args[0],
        "Vec<String>",
        "use std::io::BufRead;",
        "TextRead(ref mut __r)",
        &readlines_body,
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
    let mut read_bytes_body = String::new();
    read_bytes_body.push_str("let mut __buf = Vec::new(); ");
    read_bytes_body.push_str("__r.read_to_end(&mut __buf).map_err(__io_err)?; ");
    read_bytes_body.push_str("Ok(__buf.iter().map(|&b| b as i64).collect())");
    Some(RustExpr::RawCode(wrap_handle_result(
        &args[0],
        "Vec<i64>",
        "use std::io::Read;",
        "BinaryRead(ref mut __r)",
        &read_bytes_body,
        "file not open for binary reading",
    )))
}

pub(super) fn lower_file_write_bytes(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    let mut write_bytes_body = String::new();
    write_bytes_body.push_str("let __data: Vec<u8> = (");
    write_bytes_body.push_str(&args[1]);
    write_bytes_body.push_str(").iter().map(|&b| b as u8).collect(); ");
    write_bytes_body.push_str("__w.write_all(&__data).map_err(__io_err)?; ");
    write_bytes_body.push_str("Ok(())");
    Some(RustExpr::RawCode(wrap_handle_result(
        &args[0],
        "()",
        "use std::io::Write;",
        "BinaryWrite(ref mut __w)",
        &write_bytes_body,
        "file not open for binary writing",
    )))
}
