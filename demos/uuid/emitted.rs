// src/main.rs
// --- stdlib: sifr.test ---
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0_i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i += 1_i64;
    }
}

// --- stdlib: _sifr.uuid ---
fn uuid4() -> String {
    ::sifr_stdlib::uuid::uuid4()
}
fn uuid3_text(namespace: &String, name: &String) -> String {
    ::sifr_stdlib::uuid::uuid3_text(namespace, name)
}
fn uuid5_text(namespace: &String, name: &String) -> String {
    ::sifr_stdlib::uuid::uuid5_text(namespace, name)
}

// --- stdlib: sifr.uuid ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2euuid_x2eUUID {
    _hex: String,
}
impl __SifrStdlib_sifr_x2euuid_x2eUUID {
    fn new(hex_str: String) -> Self {
        let __sifr_field_init_0: String = {
            let mut __sifr_concat: String = String::with_capacity(
                hex_str.len() + 0usize,
            );
            __sifr_concat.push_str((hex_str).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        Self { _hex: __sifr_field_init_0 }
    }
}
impl __SifrStdlib_sifr_x2euuid_x2eUUID {
    fn hex(&self) -> String {
        let mut result: String = "".to_string();
        let mut i: i64 = 0_i64;
        while (i < (self._hex.chars().count() as i64)) {
            let ch: Option<String> = Some({
                let Some(__indexed_char) = self
                    ._hex
                    .clone()
                    .chars()
                    .nth(i as usize)
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            });
            if let Some(ch) = ch {
                if ch != "-" {
                    result.push_str((ch).as_str());
                }
            }
            i += 1_i64;
        }
        result
    }
}
impl __SifrStdlib_sifr_x2euuid_x2eUUID {
    fn urn(&self) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(9usize + 0usize);
            __sifr_concat.push_str("urn:uuid:");
            __sifr_concat.push_str((self._hex.clone()).as_str());
            __sifr_concat
        }
    }
}
impl __SifrStdlib_sifr_x2euuid_x2eUUID {
    fn to_str(&self) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((self._hex.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
}
impl __SifrStdlib_sifr_x2euuid_x2eUUID {
    fn version(&self) -> i64 {
        let marker: Option<String> = {
            let __sifr_index_str = &self._hex;
            let __sifr_index_i = 14_i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let Some(marker) = marker else {
            return -(1_i64);
        };
        _hex_digit_value(&marker)
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2euuid_x2eUUID {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "UUID(_hex={})", self._hex)
    }
}
fn _to_lower_hex_char(ch: &String) -> String {
    if (ch).as_str() == "A" {
        return "a".to_string();
    }
    if (ch).as_str() == "B" {
        return "b".to_string();
    }
    if (ch).as_str() == "C" {
        return "c".to_string();
    }
    if (ch).as_str() == "D" {
        return "d".to_string();
    }
    if (ch).as_str() == "E" {
        return "e".to_string();
    }
    if (ch).as_str() == "F" {
        return "f".to_string();
    }
    {
        let mut __sifr_concat: String = String::with_capacity(ch.len() + 0usize);
        __sifr_concat.push_str((ch).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn _is_hex_char(ch: &String) -> bool {
    if (ch).as_str() == "0" {
        return true;
    }
    if (ch).as_str() == "1" {
        return true;
    }
    if (ch).as_str() == "2" {
        return true;
    }
    if (ch).as_str() == "3" {
        return true;
    }
    if (ch).as_str() == "4" {
        return true;
    }
    if (ch).as_str() == "5" {
        return true;
    }
    if (ch).as_str() == "6" {
        return true;
    }
    if (ch).as_str() == "7" {
        return true;
    }
    if (ch).as_str() == "8" {
        return true;
    }
    if (ch).as_str() == "9" {
        return true;
    }
    if (ch).as_str() == "a" {
        return true;
    }
    if (ch).as_str() == "b" {
        return true;
    }
    if (ch).as_str() == "c" {
        return true;
    }
    if (ch).as_str() == "d" {
        return true;
    }
    if (ch).as_str() == "e" {
        return true;
    }
    if (ch).as_str() == "f" {
        return true;
    }
    if (ch).as_str() == "A" {
        return true;
    }
    if (ch).as_str() == "B" {
        return true;
    }
    if (ch).as_str() == "C" {
        return true;
    }
    if (ch).as_str() == "D" {
        return true;
    }
    if (ch).as_str() == "E" {
        return true;
    }
    if (ch).as_str() == "F" {
        return true;
    }
    false
}
fn _hex_digit_value(ch: &String) -> i64 {
    if (ch).as_str() == "0" {
        return 0_i64;
    }
    if (ch).as_str() == "1" {
        return 1_i64;
    }
    if (ch).as_str() == "2" {
        return 2_i64;
    }
    if (ch).as_str() == "3" {
        return 3_i64;
    }
    if (ch).as_str() == "4" {
        return 4_i64;
    }
    if (ch).as_str() == "5" {
        return 5_i64;
    }
    if (ch).as_str() == "6" {
        return 6_i64;
    }
    if (ch).as_str() == "7" {
        return 7_i64;
    }
    if (ch).as_str() == "8" {
        return 8_i64;
    }
    if (ch).as_str() == "9" {
        return 9_i64;
    }
    if ((ch).as_str() == "a") || ((ch).as_str() == "A") {
        return 10_i64;
    }
    if ((ch).as_str() == "b") || ((ch).as_str() == "B") {
        return 11_i64;
    }
    if ((ch).as_str() == "c") || ((ch).as_str() == "C") {
        return 12_i64;
    }
    if ((ch).as_str() == "d") || ((ch).as_str() == "D") {
        return 13_i64;
    }
    if ((ch).as_str() == "e") || ((ch).as_str() == "E") {
        return 14_i64;
    }
    if ((ch).as_str() == "f") || ((ch).as_str() == "F") {
        return 15_i64;
    }
    -(1_i64)
}
fn _substring(value: &String, start: i64, end: i64) -> String {
    let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    let mut result: String = "".to_string();
    let mut i: i64 = start;
    while i < end {
        let ch: Option<String> = __sifr_chars_value
            .get(i as usize)
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            result.push_str((ch).as_str());
        }
        i += 1_i64;
    }
    result
}
fn _starts_with(value: &String, prefix: &String) -> bool {
    let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    let __sifr_chars_prefix: Vec<char> = prefix.chars().collect::<Vec<char>>();
    if ((__sifr_chars_value.len() as i64) < (__sifr_chars_prefix.len() as i64)) {
        return false;
    }
    let mut i: i64 = 0_i64;
    while (i < (__sifr_chars_prefix.len() as i64)) {
        let left: Option<String> = __sifr_chars_value
            .get(i as usize)
            .map(|c| c.to_string());
        let right: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_prefix
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if (left != right) {
            return false;
        }
        i += 1_i64;
    }
    true
}
fn _canonical_uuid_text(input_text: &String) -> Result<String, ValueError> {
    let mut normalized_input: String = {
        let mut __sifr_concat: String = String::with_capacity(input_text.len() + 0usize);
        __sifr_concat.push_str((input_text).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if _starts_with(&normalized_input, &"urn:uuid:".to_string()) {
        normalized_input = _substring(
            &normalized_input,
            9_i64,
            normalized_input.chars().count() as i64,
        );
    }
    if ((normalized_input.chars().count() as i64) >= (2_i64)) {
        let first: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = 0_i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let last: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = (normalized_input.chars().count() as i64) - (1_i64);
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if (first == Some("{".to_string())) && (last == Some("}".to_string())) {
            normalized_input = _substring(
                &normalized_input,
                1_i64,
                (normalized_input.chars().count() as i64) - (1_i64),
            );
        }
    }
    let input_len: i64 = normalized_input.chars().count() as i64;
    let mut hex_only: String = "".to_string();
    let mut i: i64 = 0_i64;
    while i < input_len {
        let ch_opt: Option<String> = Some({
            let Some(__indexed_char) = normalized_input
                .chars()
                .nth(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            if ch == "-" {} else {
                if !(_is_hex_char(&ch)) {
                    return Err(ValueError::new("invalid UUID hex string".to_string()));
                }
                hex_only.push_str((_to_lower_hex_char(&ch)).as_str());
            }
        }
        i += 1_i64;
    }
    if ((hex_only.chars().count() as i64) != (32_i64)) {
        return Err(
            ValueError::new("UUID hex string must be 32 hex characters".to_string()),
        );
    }
    if input_len == (36_i64) {
        let h1: Option<String> = Some({
            let Some(__indexed_char) = normalized_input
                .chars()
                .nth((8_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        let h2: Option<String> = Some({
            let Some(__indexed_char) = normalized_input
                .chars()
                .nth((13_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        let h3: Option<String> = Some({
            let Some(__indexed_char) = normalized_input
                .chars()
                .nth((18_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        let h4: Option<String> = Some({
            let Some(__indexed_char) = normalized_input
                .chars()
                .nth((23_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if (((h1 != Some("-".to_string())) || (h2 != Some("-".to_string())))
            || (h3 != Some("-".to_string()))) || (h4 != Some("-".to_string()))
        {
            return Err(ValueError::new("invalid UUID hex string".to_string()));
        }
    } else {
        if input_len != (32_i64) {
            return Err(ValueError::new("invalid UUID hex string".to_string()));
        }
    }
    let mut canonical: String = "".to_string();
    let mut j: i64 = 0_i64;
    while (j < (hex_only.chars().count() as i64)) {
        if (((j == (8_i64)) || (j == (12_i64))) || (j == (16_i64))) || (j == (20_i64)) {
            canonical.push('-');
        }
        let part: Option<String> = Some({
            let Some(__indexed_char) = hex_only
                .chars()
                .nth(j as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(part) = part {
            canonical.push_str((part).as_str());
        }
        j += 1_i64;
    }
    Ok(canonical)
}
fn uuid4_obj() -> __SifrStdlib_sifr_x2euuid_x2eUUID {
    __SifrStdlib_sifr_x2euuid_x2eUUID::new(uuid4())
}
fn uuid_from_hex(
    hex_str: &String,
) -> Result<__SifrStdlib_sifr_x2euuid_x2eUUID, ValueError> {
    let __sifr_try_res: Result<
        Result<__SifrStdlib_sifr_x2euuid_x2eUUID, ValueError>,
        ValueError,
    > = (|| {
        let canonical: String = _canonical_uuid_text(hex_str)?;
        return Ok(Ok(__SifrStdlib_sifr_x2euuid_x2eUUID::new(canonical)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(ValueError::new(e.message));
        }
    }
}
fn uuid3(
    namespace: &__SifrStdlib_sifr_x2euuid_x2eUUID,
    name: &String,
) -> __SifrStdlib_sifr_x2euuid_x2eUUID {
    __SifrStdlib_sifr_x2euuid_x2eUUID::new(uuid3_text(&namespace.to_str(), name))
}
fn uuid5(
    namespace: &__SifrStdlib_sifr_x2euuid_x2eUUID,
    name: &String,
) -> __SifrStdlib_sifr_x2euuid_x2eUUID {
    __SifrStdlib_sifr_x2euuid_x2eUUID::new(uuid5_text(&namespace.to_str(), name))
}
fn NAMESPACE_DNS() -> __SifrStdlib_sifr_x2euuid_x2eUUID {
    __SifrStdlib_sifr_x2euuid_x2eUUID::new(
        "6ba7b810-9dad-11d1-80b4-00c04fd430c8".to_string(),
    )
}
// --- end stdlib ---

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl ::std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for ValueError {
}

fn is_canonical_shape(value: &String) -> bool {
    let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    if ((__sifr_chars_value.len() as i64) != (36_i64)) {
        return false;
    }
    let h1: Option<String> = __sifr_chars_value.get((8_i64) as usize).map(|c| c.to_string());
    let h2: Option<String> = __sifr_chars_value.get((13_i64) as usize).map(|c| c.to_string());
    let h3: Option<String> = __sifr_chars_value.get((18_i64) as usize).map(|c| c.to_string());
    let h4: Option<String> = __sifr_chars_value.get((23_i64) as usize).map(|c| c.to_string());
    (((((h1 == Some("-".to_string()))) && ((h2 == Some("-".to_string())))) && ((h3 == Some("-".to_string())))) && ((h4 == Some("-".to_string()))))
}

fn collect_generated_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let id_text: String = uuid4();
    let __sifr_chars_id_text: Vec<char> = id_text.chars().collect::<Vec<char>>();
    actual.push(is_canonical_shape(&id_text));
    actual.push(__sifr_chars_id_text.get((14_i64) as usize).map(|c| c.to_string()) == Some("4".to_string()));
    let obj: __SifrStdlib_sifr_x2euuid_x2eUUID = uuid4_obj();
    actual.push(is_canonical_shape(&obj.to_str()) && (obj.version() == (4_i64)));
    actual
}

fn collect_parse_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut parsed_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let parsed: __SifrStdlib_sifr_x2euuid_x2eUUID = uuid_from_hex(&"550E8400E29B41D4A716446655440000".to_string())?;
    parsed_ok = (parsed.to_str() == "550e8400-e29b-41d4-a716-446655440000");
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message);
        parsed_ok = false;
    }
    actual.push(parsed_ok);
    let mut parsed_v1_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let parsed_v1: __SifrStdlib_sifr_x2euuid_x2eUUID = uuid_from_hex(&"550e8400-e29b-11d4-a716-446655440000".to_string())?;
    parsed_v1_ok = (parsed_v1.version() == (1_i64));
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message);
        parsed_v1_ok = false;
    }
    actual.push(parsed_v1_ok);
    actual
}

fn collect_negative_and_class_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut invalid_rejected: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let _bad: __SifrStdlib_sifr_x2euuid_x2eUUID = uuid_from_hex(&"invalid".to_string())?;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message);
        invalid_rejected = true;
    }
    actual.push(invalid_rejected);
    let ctor_passthrough: __SifrStdlib_sifr_x2euuid_x2eUUID = __SifrStdlib_sifr_x2euuid_x2eUUID::new("550e8400-e29b-41d4-a716-44665544000z".to_string());
    actual.push((ctor_passthrough.to_str()).as_str() == ("550e8400-e29b-41d4-a716-44665544000z".to_string()).as_str());
    let mut ctor_curly_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let ctor_curly: __SifrStdlib_sifr_x2euuid_x2eUUID = uuid_from_hex(&"{550E8400-E29B-41D4-A716-446655440000}".to_string())?;
    ctor_curly_ok = (ctor_curly.to_str() == "550e8400-e29b-41d4-a716-446655440000");
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message);
        ctor_curly_ok = false;
    }
    actual.push(ctor_curly_ok);
    let obj: __SifrStdlib_sifr_x2euuid_x2eUUID = uuid4_obj();
    actual.push((obj.hex().chars().count() as i64) == (32_i64));
    actual.push(uuid3(&NAMESPACE_DNS(), &"python.org".to_string()).version() == (3_i64));
    actual.push(uuid5(&NAMESPACE_DNS(), &"python.org".to_string()).version() == (5_i64));
    actual
}

fn append_all(target: &mut Vec<bool>, values: &Vec<bool>) {
    for value in values.iter().copied() {
        target.push(value);
    }
}

fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true, true, true, true, true, true];
    let mut actual: Vec<bool> = vec![];
    append_all(&mut actual, &collect_generated_actual());
    append_all(&mut actual, &collect_parse_actual());
    append_all(&mut actual, &collect_negative_and_class_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("uuid uuid parity demo: pass");
}
