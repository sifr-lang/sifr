//! Environment intrinsic lowerers for registry migration.

use crate::RustExpr;

pub(super) fn lower_env_get(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __k = {}; if __k.is_empty() || __k.contains('=') || __k.as_bytes().contains(&0) {{ None }} else {{ std::env::var(__k).ok() }} }}",
        args[0]
    )))
}

pub(super) fn lower_env_set(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __k = {}; let __v = {}; if !__k.is_empty() && !__k.contains('=') && !__k.as_bytes().contains(&0) && !__v.as_bytes().contains(&0) {{ std::env::set_var(__k, __v); }} }}",
        args[0], args[1]
    )))
}

pub(super) fn lower_env_unset(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __k = {}; if !__k.is_empty() && !__k.contains('=') && !__k.as_bytes().contains(&0) {{ std::env::remove_var(__k); }} }}",
        args[0]
    )))
}

pub(super) fn lower_env_keys(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "std::env::vars_os().map(|(k, _)| k.to_string_lossy().to_string()).collect::<Vec<String>>()"
            .to_string(),
    ))
}

pub(super) fn lower_env_values(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "std::env::vars_os().map(|(_, v)| v.to_string_lossy().to_string()).collect::<Vec<String>>()"
            .to_string(),
    ))
}

pub(super) fn lower_env_items(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode(
        "std::env::vars_os().map(|(k, v)| format!(\"{}={}\", k.to_string_lossy(), v.to_string_lossy())).collect::<Vec<String>>()"
            .to_string(),
    ))
}
