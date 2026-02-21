//! Bytes intrinsic lowerers for registry migration.

use crate::{RustExpr, RustParam, RustType};

pub(super) fn lower_encode_utf8(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(format!("({})", args[0]))),
                    method: "as_bytes".to_string(),
                    args: vec![],
                }),
                method: "iter".to_string(),
                args: vec![],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "b".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::Cast {
                    expr: Box::new(RustExpr::Deref(Box::new(RustExpr::Ident(
                        "b".to_string(),
                    )))),
                    ty: RustType::I64,
                }),
                is_move: false,
            }],
        }),
        method: "collect::<Vec<i64>>".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_decode_utf8(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<String, ParseError> {{ let __vals = {}; let mut __bytes: Vec<u8> = Vec::with_capacity(__vals.len()); for (__idx, __b) in __vals.iter().enumerate() {{ if *__b < 0 || *__b > 255 {{ return Err(ParseError {{ message: format!(\"byte out of range at index {{}}: {{}}\", __idx, *__b) }}); }} __bytes.push(*__b as u8); }} String::from_utf8(__bytes).map_err(|e| ParseError {{ message: e.to_string() }}) }})()",
        args[0]
    )))
}

pub(super) fn lower_bytes_to_hex(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<String, ParseError> {{ let __vals = {}; let mut __out = String::new(); for (__idx, __b) in __vals.iter().enumerate() {{ if *__b < 0 || *__b > 255 {{ return Err(ParseError {{ message: format!(\"byte out of range at index {{}}: {{}}\", __idx, *__b) }}); }} __out.push_str(&format!(\"{{:02x}}\", *__b as u8)); }} Ok(__out) }})()",
        args[0]
    )))
}

pub(super) fn lower_bytes_from_hex(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<Vec<i64>, ParseError> {{ let s = {}; let mut cleaned = String::new(); for ch in s.chars() {{ if ch.is_ascii_whitespace() {{ continue; }} if !ch.is_ascii_hexdigit() {{ return Err(ParseError {{ message: format!(\"invalid hex character: {{}}\", ch) }}); }} cleaned.push(ch); }} if cleaned.len() % 2 != 0 {{ return Err(ParseError {{ message: \"fromhex() arg must contain an even number of hexadecimal digits\".to_string() }}); }} let mut result = Vec::new(); for pair in cleaned.as_bytes().chunks(2) {{ let pair_str = std::str::from_utf8(pair).map_err(|e| ParseError {{ message: e.to_string() }})?; result.push(i64::from_str_radix(pair_str, 16).map_err(|e| ParseError {{ message: e.to_string() }})?); }} Ok(result) }})()",
        args[0]
    )))
}
