#[derive(Debug, Clone)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ParseError {
}

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ValueError {
}

fn main() {
    let mut size_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let zeros: Vec<u8> = ({
    let __size = 6 as i64;
    if __size < 0 {
        return Err(ValueError { message: "bytes(size) requires a non-negative size".to_string().to_string() });
    }
    Ok((0..__size).map(|_| 0 as u8).collect::<Vec<u8>>())
})?;
    assert!((zeros.len() as i64) == (6 as i64));
    size_ok = true;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "unexpected ValueError: ".to_string(), e.message));
    }
    assert!(size_ok);
    let mut from_ints_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let from_list: Vec<u8> = ({
    let __vals = vec![83 as i64, 105 as i64, 102 as i64, 114 as i64];
    let mut __out = Vec::new();
    for __pair in __vals.iter().enumerate() {
        if (*__pair.1 < 0) || (*__pair.1 > 255) {
            return Err(ValueError { message: format!("byte out of range at index {}: {}", __pair.0, *__pair.1) });
        }
        __out.push(*__pair.1 as u8);
    }
    Ok(__out)
})?;
    assert!(from_list.get((0 as i64) as usize).map(|__byte| *__byte as i64) == Some(83 as i64));
    assert!(from_list.get((3 as i64) as usize).map(|__byte| *__byte as i64) == Some(114 as i64));
    from_ints_ok = true;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "unexpected ValueError: ".to_string(), e.message));
    }
    assert!(from_ints_ok);
    let mut from_hex_ok: bool = false;
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let from_hex: Vec<u8> = ({
    let s: String = "53 69 66 72".to_string().to_string();
    let mut cleaned = String::new();
    for ch in s.chars() {
        if ch.is_ascii_whitespace() {
            continue;
        }
        if !ch.is_ascii_hexdigit() {
            return Err(ParseError { message: format!("invalid hex character: {}", ch) });
        }
        cleaned.push(ch);
    }
    if (cleaned.len() % 2) != 0 {
        return Err(ParseError { message: "fromhex() arg must contain an even number of hexadecimal digits".to_string().to_string() });
    }
    let mut result = Vec::new();
    for pair in cleaned.as_bytes().chunks(2) {
        let pair_str = std::str::from_utf8(pair).map_err(|e| ParseError { message: e.to_string() })?;
        result.push(u8::from_str_radix(pair_str, 16).map_err(|e| ParseError { message: e.to_string() })?);
    }
    Ok(result)
})?;
    let from_hex_text: String = String::from_utf8(from_hex.iter().copied().collect::<Vec<u8>>()).map_err(|e| ParseError { message: e.to_string() })?;
    assert!(from_hex_text == "Sifr".to_string());
    from_hex_ok = true;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "unexpected ParseError: ".to_string(), e.message));
    }
    assert!(from_hex_ok);
    let mut encode_ok: bool = false;
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let encoded: Vec<u8> = ({
    let __encoding = "utf-8".to_string();
    let __encoding_lower = __encoding.to_ascii_lowercase();
    if (__encoding_lower != "utf-8".to_string()) && (__encoding_lower != "utf8".to_string()) { Err(ParseError { message: format!("{} currently supports only UTF-8 encoding, got {}", "str.encode()".to_string(), __encoding) }) } else { Ok({
    let __s = "bytes_constructors-demo".to_string();
    __s.as_bytes().to_vec()
}) }
})?;
    let decoded: String = ({
    let __encoding = "utf-8".to_string();
    let __encoding_lower = __encoding.to_ascii_lowercase();
    if (__encoding_lower != "utf-8".to_string()) && (__encoding_lower != "utf8".to_string()) { Err(ParseError { message: format!("{} currently supports only UTF-8 encoding, got {}", "bytes.decode()".to_string(), __encoding) }) } else { String::from_utf8(encoded.iter().copied().collect::<Vec<u8>>()).map_err(|e| ParseError { message: e.to_string() }) }
})?;
    assert!(decoded == "bytes_constructors-demo".to_string());
    encode_ok = true;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "unexpected ParseError: ".to_string(), e.message));
    }
    assert!(encode_ok);
    println!("bytes_bytes_constructors_surface_demo: ok");
}
