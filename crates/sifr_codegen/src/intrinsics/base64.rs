//! Base64 intrinsic lowerers for registry migration.

use crate::RustExpr;

fn borrowed_str(expr: &str) -> String {
    format!("&({expr})")
}

pub(super) fn lower_base64_encode(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "{{ use base64::Engine; base64::engine::general_purpose::STANDARD.encode({}.as_bytes()) }}",
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_base64_decode(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "(|| -> Result<String, ParseError> {{ use base64::Engine; let bytes = base64::engine::general_purpose::STANDARD.decode({}.as_bytes()).map_err(|e| ParseError {{ message: e.to_string() }})?; String::from_utf8(bytes).map_err(|e| ParseError {{ message: e.to_string() }}) }})()",
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_base64_encode_opts(args: &[String]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "(|| -> Result<String, ParseError> {{ use base64::Engine; let __s = {}; let __alt = {}; let __wrap = {}; if __wrap < 0 {{ return Err(ParseError {{ message: \"wrapcol must be >= 0\".to_string() }}); }} let mut __encoded = base64::engine::general_purpose::STANDARD.encode(__s.as_bytes()); if !__alt.is_empty() {{ if __alt.chars().count() != 2 {{ return Err(ParseError {{ message: format!(\"invalid altchars: {{}}\", __alt) }}); }} let mut __it = __alt.chars(); let __a = __it.next().unwrap_or('+'); let __b = __it.next().unwrap_or('/'); __encoded = __encoded.chars().map(|c| if c == '+' {{ __a }} else if c == '/' {{ __b }} else {{ c }}).collect::<String>(); }} if __wrap == 0 {{ return Ok(__encoded); }} let __w = __wrap as usize; let mut __wrapped = String::new(); for (i, ch) in __encoded.chars().enumerate() {{ if i > 0 && i % __w == 0 {{ __wrapped.push('\\n'); }} __wrapped.push(ch); }} Ok(__wrapped) }})()",
        borrowed_str(&args[0]),
        borrowed_str(&args[1]),
        args[2]
    )))
}

pub(super) fn lower_base64_decode_opts(args: &[String]) -> Option<RustExpr> {
    if args.len() != 4 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "(|| -> Result<String, ParseError> {{ use base64::Engine; let __s = {}; let __alt = {}; let __validate = {}; let __ignore = {}; let mut __has_alt = false; let mut __alt_a = '+'; let mut __alt_b = '/'; if !__alt.is_empty() {{ if __alt.chars().count() != 2 {{ return Err(ParseError {{ message: format!(\"invalid altchars: {{}}\", __alt) }}); }} let mut __it = __alt.chars(); __alt_a = __it.next().unwrap_or('+'); __alt_b = __it.next().unwrap_or('/'); __has_alt = true; }} let mut __ignore_set = std::collections::HashSet::<char>::new(); for ch in __ignore.chars() {{ __ignore_set.insert(ch); }} let mut __normalized = String::new(); for ch in __s.chars() {{ if __ignore_set.contains(&ch) {{ continue; }} let mut mapped = ch; if __has_alt {{ if ch == __alt_a {{ mapped = '+'; }} else if ch == __alt_b {{ mapped = '/'; }} }} let is_base64 = (mapped >= 'A' && mapped <= 'Z') || (mapped >= 'a' && mapped <= 'z') || (mapped >= '0' && mapped <= '9') || mapped == '+' || mapped == '/' || mapped == '='; if is_base64 {{ __normalized.push(mapped); }} else if __validate {{ return Err(ParseError {{ message: format!(\"invalid base64 character: {{}}\", ch) }}); }} }} let __bytes = base64::engine::general_purpose::STANDARD.decode(__normalized.as_bytes()).map_err(|e| ParseError {{ message: e.to_string() }})?; String::from_utf8(__bytes).map_err(|e| ParseError {{ message: e.to_string() }}) }})()",
        borrowed_str(&args[0]),
        borrowed_str(&args[1]),
        args[2],
        borrowed_str(&args[3])
    )))
}

pub(super) fn lower_urlsafe_b64encode(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "{{ use base64::Engine; base64::engine::general_purpose::URL_SAFE.encode({}.as_bytes()) }}",
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_urlsafe_b64decode(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "(|| -> Result<String, ParseError> {{ use base64::Engine; let bytes = base64::engine::general_purpose::URL_SAFE.decode({}.as_bytes()).map_err(|e| ParseError {{ message: e.to_string() }})?; String::from_utf8(bytes).map_err(|e| ParseError {{ message: e.to_string() }}) }})()",
        borrowed_str(&args[0])
    )))
}
