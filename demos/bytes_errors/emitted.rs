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
    let mut bad_size: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let _: Result<Vec<u8>, ValueError> = {
    let __size = -(1 as i64);
    if __size < 0 {
        return Err(ValueError { message: "bytes(size) requires a non-negative size".to_string().to_string() });
    }
    Ok((0..__size).map(|_| 0 as u8).collect::<Vec<u8>>())
};
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        bad_size = true;
    }
    let mut bad_values: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let _: Result<Vec<u8>, ValueError> = {
    let __vals = vec![0 as i64, 999 as i64];
    let mut __out = Vec::new();
    for __pair in __vals.iter().enumerate() {
        if (*__pair.1 < 0) || (*__pair.1 > 255) {
            return Err(ValueError { message: format!("byte out of range at index {}: {}", __pair.0, *__pair.1) });
        }
        __out.push(*__pair.1 as u8);
    }
    Ok(__out)
};
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        bad_values = true;
    }
    let mut bad_hex: bool = false;
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let _: Result<Vec<u8>, ParseError> = {
    let s: String = "GG".to_string().to_string();
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
};
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        bad_hex = true;
    }
    let mut bad_codec: bool = false;
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let codec: String = "latin-1".to_string();
    let _encoded: Vec<u8> = ({
    let __encoding = codec;
    let __encoding_lower = __encoding.to_ascii_lowercase();
    if (__encoding_lower != "utf-8".to_string()) && (__encoding_lower != "utf8".to_string()) { Err(ParseError { message: format!("{} currently supports only UTF-8 encoding, got {}", "str.encode()".to_string(), __encoding) }) } else { Ok({
    let __s = "abc".to_string();
    __s.as_bytes().to_vec()
}) }
})?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        bad_codec = true;
    }
    let mut bad_utf8: bool = false;
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let _invalid_utf8: String = String::from_utf8(vec![(255 as i64) as u8].iter().copied().collect::<Vec<u8>>()).map_err(|e| ParseError { message: e.to_string() })?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        bad_utf8 = true;
    }
    assert!(bad_size);
    assert!(bad_values);
    assert!(bad_hex);
    assert!(bad_codec);
    assert!(bad_utf8);
    println!("bytes_bytes_errors_boundary_demo: ok");
}
