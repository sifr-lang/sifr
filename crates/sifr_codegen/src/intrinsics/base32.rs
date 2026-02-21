//! Base32 intrinsic lowerers for registry migration.

use crate::RustExpr;

pub(super) fn lower_b32encode(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __b32_alpha = b\"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567\"; let __s = {}; let __data = __s.as_bytes(); let mut __out = String::new(); let mut __i = 0usize; while __i < __data.len() {{ let __b0 = __data[__i] as u64; let __b1 = if __i+1 < __data.len() {{ __data[__i+1] as u64 }} else {{ 0 }}; let __b2 = if __i+2 < __data.len() {{ __data[__i+2] as u64 }} else {{ 0 }}; let __b3 = if __i+3 < __data.len() {{ __data[__i+3] as u64 }} else {{ 0 }}; let __b4 = if __i+4 < __data.len() {{ __data[__i+4] as u64 }} else {{ 0 }}; let __buf = (__b0<<32)|(__b1<<24)|(__b2<<16)|(__b3<<8)|__b4; let __n = ((__data.len() - __i).min(5)) as u64; for __j in 0..8u64 {{ if __j < (__n*8+4)/5 {{ __out.push(__b32_alpha[((__buf >> (35 - __j*5)) & 0x1f) as usize] as char); }} else {{ __out.push('='); }} }} __i += 5; }} __out }}",
        args[0]
    )))
}

pub(super) fn lower_b32decode(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<String, ParseError> {{ let __b32_alpha = b\"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567\"; let __s_val = {}; let __s = __s_val.trim_end_matches('='); let mut __bits = 0u64; let mut __bit_count = 0u32; let mut __out: Vec<u8> = Vec::new(); for __c in __s.chars() {{ let __val = __b32_alpha.iter().position(|&b| b as char == __c.to_ascii_uppercase()).ok_or_else(|| ParseError {{ message: format!(\"invalid base32 char: {{}}\", __c) }})? as u64; __bits = (__bits << 5) | __val; __bit_count += 5; if __bit_count >= 8 {{ __bit_count -= 8; __out.push(((__bits >> __bit_count) & 0xff) as u8); }} }} String::from_utf8(__out).map_err(|e| ParseError {{ message: e.to_string() }}) }})()",
        args[0]
    )))
}

pub(super) fn lower_b32hexencode(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __b32_alpha = b\"0123456789ABCDEFGHIJKLMNOPQRSTUV\"; let __s = {}; let __data = __s.as_bytes(); let mut __out = String::new(); let mut __i = 0usize; while __i < __data.len() {{ let __b0 = __data[__i] as u64; let __b1 = if __i+1 < __data.len() {{ __data[__i+1] as u64 }} else {{ 0 }}; let __b2 = if __i+2 < __data.len() {{ __data[__i+2] as u64 }} else {{ 0 }}; let __b3 = if __i+3 < __data.len() {{ __data[__i+3] as u64 }} else {{ 0 }}; let __b4 = if __i+4 < __data.len() {{ __data[__i+4] as u64 }} else {{ 0 }}; let __buf = (__b0<<32)|(__b1<<24)|(__b2<<16)|(__b3<<8)|__b4; let __n = ((__data.len() - __i).min(5)) as u64; for __j in 0..8u64 {{ if __j < (__n*8+4)/5 {{ __out.push(__b32_alpha[((__buf >> (35 - __j*5)) & 0x1f) as usize] as char); }} else {{ __out.push('='); }} }} __i += 5; }} __out }}",
        args[0]
    )))
}

pub(super) fn lower_b32hexdecode(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<String, ParseError> {{ let __b32_alpha = b\"0123456789ABCDEFGHIJKLMNOPQRSTUV\"; let __s_val = {}; let __s = __s_val.trim_end_matches('='); let mut __bits = 0u64; let mut __bit_count = 0u32; let mut __out: Vec<u8> = Vec::new(); for __c in __s.chars() {{ let __val = __b32_alpha.iter().position(|&b| b as char == __c.to_ascii_uppercase()).ok_or_else(|| ParseError {{ message: format!(\"invalid base32hex char: {{}}\", __c) }})? as u64; __bits = (__bits << 5) | __val; __bit_count += 5; if __bit_count >= 8 {{ __bit_count -= 8; __out.push(((__bits >> __bit_count) & 0xff) as u8); }} }} String::from_utf8(__out).map_err(|e| ParseError {{ message: e.to_string() }}) }})()",
        args[0]
    )))
}
