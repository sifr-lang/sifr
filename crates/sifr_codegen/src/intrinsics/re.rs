//! Regex intrinsic lowerers for registry migration.

use crate::RustExpr;

fn borrowed_str(expr: &str) -> String {
    format!("&({expr})")
}

fn replacer_str(expr: &str) -> String {
    format!("&*({expr})")
}

fn regex_error_map() -> &'static str {
    "map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })"
}

pub(super) fn lower_re_match(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "regex::Regex::new({}).map(|re| re.is_match({})).{}",
        borrowed_str(&args[0]),
        borrowed_str(&args[1]),
        regex_error_map()
    )))
}

pub(super) fn lower_re_find(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "regex::Regex::new({}).map(|re| re.find({}).map(|m| m.as_str().to_string())).{}",
        borrowed_str(&args[0]),
        borrowed_str(&args[1]),
        regex_error_map()
    )))
}

pub(super) fn lower_re_replace(args: &[String]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "regex::Regex::new({}).map(|re| re.replace_all({}, {}).to_string()).{}",
        borrowed_str(&args[0]),
        borrowed_str(&args[2]),
        replacer_str(&args[1]),
        regex_error_map()
    )))
}

pub(super) fn lower_re_findall(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "regex::Regex::new({}).map(|re| re.find_iter({}).map(|m| m.as_str().to_string()).collect::<Vec<String>>()).{}",
        borrowed_str(&args[0]),
        borrowed_str(&args[1]),
        regex_error_map()
    )))
}

pub(super) fn lower_re_split(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "regex::Regex::new({}).map(|re| re.split({}).map(|s| s.to_string()).collect::<Vec<String>>()).{}",
        borrowed_str(&args[0]),
        borrowed_str(&args[1]),
        regex_error_map()
    )))
}

pub(super) fn lower_re_find_start(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "regex::Regex::new({}).map(|re| re.find({}).map_or(-1_i64, |m| m.start() as i64)).{}",
        borrowed_str(&args[0]),
        borrowed_str(&args[1]),
        regex_error_map()
    )))
}

pub(super) fn lower_re_find_end(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "regex::Regex::new({}).map(|re| re.find({}).map_or(-1_i64, |m| m.end() as i64)).{}",
        borrowed_str(&args[0]),
        borrowed_str(&args[1]),
        regex_error_map()
    )))
}

fn lower_flags_common(args: &[String], mode: &str) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }

    let (result_ty, body) = match mode {
        "match" => (
            "bool",
            format!("Ok(__re.is_match({}))", borrowed_str(&args[1])),
        ),
        "find" => (
            "Option<String>",
            format!(
                "Ok(__re.find({}).map(|m| m.as_str().to_string()))",
                borrowed_str(&args[1])
            ),
        ),
        "findall" => (
            "Vec<String>",
            format!(
                "Ok(__re.find_iter({}).map(|m| m.as_str().to_string()).collect())",
                borrowed_str(&args[1])
            ),
        ),
        "split" => (
            "Vec<String>",
            format!(
                "Ok(__re.split({}).map(|s| s.to_string()).collect())",
                borrowed_str(&args[1])
            ),
        ),
        _ => return None,
    };

    Some(RustExpr::Ident(format!(
        "(|| -> Result<{result_ty}, RegexError> {{ let __flags_val = {}; let mut __flag_str = String::new(); if __flags_val & 2 != 0 {{ __flag_str.push_str(\"(?i)\"); }} if __flags_val & 8 != 0 {{ __flag_str.push_str(\"(?m)\"); }} if __flags_val & 16 != 0 {{ __flag_str.push_str(\"(?s)\"); }} if __flags_val & 64 != 0 {{ __flag_str.push_str(\"(?x)\"); }} let __pat = __flag_str + {}; let __re = regex::Regex::new(&__pat).map_err(|e| RegexError {{ message: e.to_string(), detail: e.to_string() }})?; {body} }})()",
        args[2],
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_re_match_flags(args: &[String]) -> Option<RustExpr> {
    lower_flags_common(args, "match")
}

pub(super) fn lower_re_find_flags(args: &[String]) -> Option<RustExpr> {
    lower_flags_common(args, "find")
}

pub(super) fn lower_re_findall_flags(args: &[String]) -> Option<RustExpr> {
    lower_flags_common(args, "findall")
}

pub(super) fn lower_re_split_flags(args: &[String]) -> Option<RustExpr> {
    lower_flags_common(args, "split")
}

pub(super) fn lower_re_replace_flags(args: &[String]) -> Option<RustExpr> {
    if args.len() != 4 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "(|| -> Result<String, RegexError> {{ let __flags_val = {}; let mut __flag_str = String::new(); if __flags_val & 2 != 0 {{ __flag_str.push_str(\"(?i)\"); }} if __flags_val & 8 != 0 {{ __flag_str.push_str(\"(?m)\"); }} if __flags_val & 16 != 0 {{ __flag_str.push_str(\"(?s)\"); }} if __flags_val & 64 != 0 {{ __flag_str.push_str(\"(?x)\"); }} let __pat = __flag_str + {}; let __re = regex::Regex::new(&__pat).map_err(|e| RegexError {{ message: e.to_string(), detail: e.to_string() }})?; Ok(__re.replace_all({}, {}).to_string()) }})()",
        args[3],
        borrowed_str(&args[0]),
        borrowed_str(&args[2]),
        replacer_str(&args[1])
    )))
}
